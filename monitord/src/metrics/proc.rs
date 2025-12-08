use std::collections::HashMap;

use crate::error::Result;

pub struct ProcessMetricCollector {
    sys: sysinfo::System,
}

impl ProcessMetricCollector {
    pub fn new() -> Result<Self> {
        Ok(Self {
            sys: sysinfo::System::new_with_specifics(
                sysinfo::RefreshKind::nothing()
                    .with_processes(sysinfo::ProcessRefreshKind::everything()),
            ),
        })
    }

    pub fn collect(
        &mut self,
        request: &Vec<monitord_types::service::ProcessFilter>,
    ) -> Result<HashMap<u32, monitord_types::service::ProcessResponse>> {
        self.sys
            .refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        let users = sysinfo::Users::new_with_refreshed_list();
        let mut process_metrics = HashMap::new();

        'process_loop: for (pid, process) in self.sys.processes().iter() {
            if process.thread_kind().is_some() {
                continue;
            }
            let name = process.name().to_string_lossy().to_string();
            let user = process
                .user_id()
                .map(|uid| {
                    users
                        .list()
                        .iter()
                        .find(|user| user.id() == uid)
                        .map(|user| user.name().to_string())
                        .unwrap_or_default()
                })
                .unwrap_or_default();
            let pid = pid.as_u32();
            let cpu = process.cpu_usage() as f64;
            let memory = process.memory();
            // GPU usage is filled out separately after collectors have gathered data
            let gpu = 0.0;
            let disk_read = process.disk_usage().read_bytes;
            let disk_write = process.disk_usage().written_bytes;

            use monitord_types::service::process_filter::Filter;

            for filter_entry in request {
                match &filter_entry.filter {
                    Some(Filter::User(user_filter)) => {
                        if user != user_filter.to_string() {
                            continue 'process_loop;
                        }
                    }
                    Some(Filter::Range(range_filter)) => {
                        if pid < range_filter.lower || pid >= range_filter.higher {
                            continue 'process_loop;
                        }
                    }
                    Some(Filter::Regex(regex_filter)) => {
                        let regex = regex::Regex::new(regex_filter.as_str());
                        if regex.is_ok_and(|r| !r.is_match(name.as_str())) {
                            continue 'process_loop;
                        }
                    }
                    Some(Filter::StatusList(status_list_filter)) => {
                        let sysinfo_filters: Vec<sysinfo::ProcessStatus> = status_list_filter
                            .list
                            .iter()
                            .cloned()
                            .map(proto_to_sysinfo)
                            .collect();
                        if sysinfo_filters
                            .into_iter()
                            .find(|status| status.clone() == process.status())
                            .is_none()
                        {
                            continue 'process_loop;
                        }
                    }
                    None => {}
                }
            }

            let priority = unsafe { libc::getpriority(libc::PRIO_PROCESS, pid) };
            let status = Some(sysinfo_to_proto(process.status()));
            process_metrics.insert(
                pid,
                monitord_types::service::ProcessResponse {
                    name,
                    user,
                    pid,
                    cpu,
                    memory,
                    gpu,
                    disk_read,
                    disk_write,
                    priority,
                    status,
                },
            );
        }

        Ok(process_metrics)
    }
}

fn sysinfo_to_proto(status: sysinfo::ProcessStatus) -> monitord_types::service::ProcessStatus {
    use monitord_types::service::{ProcessStatus, process_status::Status};

    match status {
        sysinfo::ProcessStatus::Idle => ProcessStatus {
            status: Some(Status::Known(0)),
        },
        sysinfo::ProcessStatus::Run => ProcessStatus {
            status: Some(Status::Known(1)),
        },
        sysinfo::ProcessStatus::Sleep => ProcessStatus {
            status: Some(Status::Known(2)),
        },
        sysinfo::ProcessStatus::Stop => ProcessStatus {
            status: Some(Status::Known(3)),
        },
        sysinfo::ProcessStatus::Zombie => ProcessStatus {
            status: Some(Status::Known(4)),
        },
        sysinfo::ProcessStatus::Tracing => ProcessStatus {
            status: Some(Status::Known(5)),
        },
        sysinfo::ProcessStatus::Dead => ProcessStatus {
            status: Some(Status::Known(6)),
        },
        sysinfo::ProcessStatus::Wakekill => ProcessStatus {
            status: Some(Status::Known(7)),
        },
        sysinfo::ProcessStatus::Waking => ProcessStatus {
            status: Some(Status::Known(8)),
        },
        sysinfo::ProcessStatus::Parked => ProcessStatus {
            status: Some(Status::Known(9)),
        },
        sysinfo::ProcessStatus::LockBlocked => ProcessStatus {
            status: Some(Status::Known(10)),
        },
        sysinfo::ProcessStatus::UninterruptibleDiskSleep => ProcessStatus {
            status: Some(Status::Known(11)),
        },
        sysinfo::ProcessStatus::Unknown(unknown) => ProcessStatus {
            status: Some(Status::Unknown(unknown)),
        },
    }
}

fn proto_to_sysinfo(status: monitord_types::service::ProcessStatus) -> sysinfo::ProcessStatus {
    use monitord_types::service::process_status::Status;
    match status.status.unwrap() {
        Status::Known(s) => match s {
            0 => sysinfo::ProcessStatus::Idle,
            1 => sysinfo::ProcessStatus::Run,
            2 => sysinfo::ProcessStatus::Sleep,
            3 => sysinfo::ProcessStatus::Stop,
            4 => sysinfo::ProcessStatus::Zombie,
            5 => sysinfo::ProcessStatus::Tracing,
            6 => sysinfo::ProcessStatus::Dead,
            7 => sysinfo::ProcessStatus::Wakekill,
            8 => sysinfo::ProcessStatus::Waking,
            9 => sysinfo::ProcessStatus::Parked,
            10 => sysinfo::ProcessStatus::LockBlocked,
            11 => sysinfo::ProcessStatus::UninterruptibleDiskSleep,
            _ => sysinfo::ProcessStatus::Unknown(s as u32),
        },
        Status::Unknown(s) => sysinfo::ProcessStatus::Unknown(s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format_byte(size: u64) -> String {
        if size < 1024_u64.pow(1) {
            format!("{:4}  B", size)
        } else if size < 1024_u64.pow(2) {
            format!("{:4}KiB", size / 1024_u64.pow(1))
        } else if size < 1024_u64.pow(3) {
            format!("{:4}MiB", size / 1024_u64.pow(2))
        } else if size < 1024_u64.pow(4) {
            format!("{:4}GiB", size / 1024_u64.pow(3))
        } else {
            format!("{:4}TiB", size / 1024_u64.pow(4))
        }
    }

    #[test]
    fn test_proc_metrics() -> Result<()> {
        let mut collector = ProcessMetricCollector::new()?;
        let _ = collector.collect(&Vec::new())?;
        // pause to allow second capture
        std::thread::sleep(std::time::Duration::from_secs(1));
        let proc_metrics = collector.collect(&Vec::new())?;

        println!(
            "{:<15} {:<24} {:>5} {:>5} {:>7} {:>9} {:>9} {:>4} {:>24}",
            "Name", "User", "PID", "CPU", "Mem", "R", "W", "Nice", "Status"
        );
        for (_, process) in proc_metrics {
            //   name    user   pid    cpu mem  r    w    n
            println!(
                "{:<15} {:<24} {:5} {:4.1}% {} {}/s {}/s {:4} {:>24}",
                process.name,
                process.user,
                process.pid,
                process.cpu,
                format_byte(process.memory),
                format_byte(process.disk_read),
                format_byte(process.disk_write),
                process.priority,
                proto_to_sysinfo(process.status.unwrap()).to_string()
            );
        }

        Ok(())
    }
}
