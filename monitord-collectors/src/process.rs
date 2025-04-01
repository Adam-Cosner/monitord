use crate::config::ProcessCollectorConfig;
use crate::error::{CollectorError, Result};
use crate::traits::Collector;
use crate::CollectorConfig;
use monitord_protocols::monitord::{KeyValuePair, ProcessInfo, ProcessList};
use std::collections::HashMap;
use sysinfo::{ProcessesToUpdate, System};
use tracing::{debug, info};

pub struct ProcessCollector {
    config: ProcessCollectorConfig,
    system: System,
}

impl Collector for ProcessCollector {
    type Data = ProcessList;
    type Config = ProcessCollectorConfig;

    fn new(config: Self::Config) -> Result<Self> {
        debug!("Initializing Process collector with config: {:?}", config);

        if !config.is_enabled() {
            info!("Process collector is disabled");
            return Err(CollectorError::ConfigurationError(
                "Process collector is disabled".into(),
            ));
        }

        let mut system = System::new_all();
        // Refresh process list to initialize
        system.refresh_processes(ProcessesToUpdate::All, true);

        info!("Process collector initialized");
        Ok(Self { config, system })
    }

    fn collect(&mut self) -> Result<Self::Data> {
        debug!("Collecting process information");

        // Refresh process information
        self.system.refresh_processes(ProcessesToUpdate::All, true);

        let mut process_infos = Vec::new();

        for (pid, process) in self.system.processes() {
            let pid_u32 = pid.as_u32();

            if process.thread_kind().is_some() {
                continue;
            }

            // Get process name
            let name = process.name().to_string_lossy().to_string();

            // Get process owner
            let username = process
                .user_id()
                .map(|uid| uid.to_string())
                .unwrap_or_else(|| "unknown".to_string());

            // Get process state
            let state = match process.status() {
                sysinfo::ProcessStatus::Run => "running",
                sysinfo::ProcessStatus::Sleep => "sleeping",
                sysinfo::ProcessStatus::Stop => "stopped",
                sysinfo::ProcessStatus::Idle => "idle",
                sysinfo::ProcessStatus::Zombie => "zombie",
                _ => "unknown",
            }
            .to_string();

            // Get CPU usage
            let cpu_usage_percent = process.cpu_usage() as f64;

            // Get memory usage
            let physical_memory_bytes = process.memory();
            let virtual_memory_bytes = process.virtual_memory();

            // Calculate disk IO rates if enabled
            let (disk_read_rate, disk_write_rate) = if self.config.collect_io_stats {
                (
                    (process.disk_usage().read_bytes as f64
                        / (self.config.interval_ms as f64 / 1000.0)) as u64,
                    (process.disk_usage().written_bytes as f64
                        / (self.config.interval_ms as f64 / 1000.0)) as u64,
                )
            } else {
                (0, 0)
            };

            // Thread count not available through sysinfo
            let threads = 0; // todo

            // Get open file count - not directly available through sysinfo
            let open_files = 0; // todo

            // Get start time
            let start_time_epoch_seconds = process.start_time() as i64;

            // Get parent PID
            let parent_pid = process.parent().map(|p| p.as_u32());

            // Get command line if enabled
            let cmdline = if self.config.collect_command_line {
                Some(
                    process
                        .cmd()
                        .iter()
                        .map(|cmd| cmd.to_string_lossy().to_string())
                        .collect::<Vec<String>>()
                        .join(" "),
                )
            } else {
                None
            };

            // Get current working directory - not directly available through sysinfo
            let cwd = None;

            // Get environment variables if enabled
            let environment = if self.config.collect_environment {
                process
                    .environ()
                    .iter()
                    .filter_map(|env| {
                        let env = env.to_string_lossy().to_string();
                        let parts: Vec<&str> = env.split('=').collect();
                        if parts.len() >= 2 {
                            Some(KeyValuePair {
                                key: parts[0].to_string(),
                                value: parts[1..].join("="),
                            })
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            };

            // Create ProcessInfo object
            let process_info = ProcessInfo {
                pid: pid_u32,
                name,
                username,
                state,
                cpu_usage_percent,
                physical_memory_bytes,
                virtual_memory_bytes,
                disk_read_bytes_per_sec: disk_read_rate,
                disk_write_bytes_per_sec: disk_write_rate,
                threads,
                open_files,
                start_time_epoch_seconds,
                gpu_usage: None, // Would need to be correlated with GPU processes
                parent_pid,
                cmdline,
                cwd,
                environment,
                io_priority: None, // Not available through sysinfo
                nice_value: None,  // Not easily available through sysinfo
            };

            process_infos.push(process_info);

            // Limit the number of processes if configured
            if process_infos.len() >= self.config.max_processes as usize {
                break;
            }
        }

        debug!(
            "Process information collected for {} process(es)",
            process_infos.len()
        );
        Ok(ProcessList {
            processes: process_infos,
        })
    }
}
