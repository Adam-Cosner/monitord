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
        request: &monitord_types::service::ProcessRequest,
    ) -> Result<HashMap<u32, monitord_types::service::ProcessResponse>> {
        self.sys
            .refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        let users = sysinfo::Users::new_with_refreshed_list();

        Ok(self
            .sys
            .processes()
            .iter()
            .filter(|(_, process)| process.thread_kind().is_none())
            .map(|(mpid, process)| {
                let name = if request.name {
                    process.name().to_string_lossy().to_string()
                } else {
                    "".to_string()
                };

                let user = if request.user {
                    process
                        .user_id()
                        .map(|uid| {
                            users
                                .list()
                                .iter()
                                .find(|user| user.id() == uid)
                                .map(|user| user.name().to_string())
                                .unwrap_or_default()
                        })
                        .unwrap_or_default()
                } else {
                    "".to_string()
                };

                let pid = if request.pid {
                    process.pid().as_u32()
                } else {
                    mpid.as_u32()
                };

                let cpu = if request.cpu {
                    process.cpu_usage() as f64
                } else {
                    0.0
                };

                let memory = if request.memory { process.memory() } else { 0 };

                // GPU usage is filled out separately after collectors have gathered data
                let gpu = 0.0;

                let (disk_read, disk_write) = if request.disk {
                    (
                        process.disk_usage().read_bytes,
                        process.disk_usage().written_bytes,
                    )
                } else {
                    (0, 0)
                };

                let priority = if request.priority {
                    unsafe { libc::getpriority(libc::PRIO_PROCESS, mpid.as_u32()) }
                } else {
                    0
                };

                (
                    mpid.as_u32(),
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
                    },
                )
            })
            .collect())
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
        let request = monitord_types::service::ProcessRequest {
            name: true,
            user: true,
            pid: true,
            cpu: true,
            memory: true,
            gpu: false,
            disk: true,
            priority: true,
        };

        let mut collector = ProcessMetricCollector::new()?;
        let _ = collector.collect(&request)?;
        // pause to allow second capture
        std::thread::sleep(std::time::Duration::from_secs(1));
        let proc_metrics = collector.collect(&request)?;

        println!(
            "{:<15} {:<24} {:>5} {:>5} {:>7} {:>9} {:>9} {:>4}",
            "Name", "User", "PID", "CPU", "Mem", "R", "W", "Nice"
        );
        for (_, process) in proc_metrics {
            //   name    user   pid    cpu mem  r    w    n
            println!(
                "{:<15} {:<24} {:5} {:4.1}% {} {}/s {}/s {:4}",
                process.name,
                process.user,
                process.pid,
                process.cpu,
                format_byte(process.memory),
                format_byte(process.disk_read),
                format_byte(process.disk_write),
                process.priority
            );
        }

        Ok(())
    }
}
