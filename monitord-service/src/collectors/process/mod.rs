use crate::error::CollectionError;
use monitord_protocols::monitord::ProcessInfo;
use tracing::debug;

pub mod config;

pub struct ProcessCollector {
    config: config::ProcessCollectorConfig,
    system: sysinfo::System,
}

impl ProcessCollector {
    pub fn new(config: config::ProcessCollectorConfig) -> Result<Self, CollectionError> {
        let system = sysinfo::System::new_with_specifics(
            sysinfo::RefreshKind::nothing()
                .with_processes(sysinfo::ProcessRefreshKind::everything()),
        );
        Ok(Self { config, system })
    }
}

impl super::Collector for ProcessCollector {
    type CollectedData = Vec<ProcessInfo>;
    type CollectorConfig = config::ProcessCollectorConfig;

    fn name(&self) -> &'static str {
        "process"
    }

    fn config(&self) -> &config::ProcessCollectorConfig {
        &self.config
    }

    fn collect(&mut self) -> Result<Self::CollectedData, CollectionError> {
        if !self.config.enabled {
            return Err(CollectionError::Disabled);
        }

        let mut processes = Vec::new();
        self.system.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::All,
            true,
            sysinfo::ProcessRefreshKind::everything(),
        );
        let users = sysinfo::Users::new_with_refreshed_list();
        for (pid, process) in self.system.processes().iter() {
            let username = users
                .iter()
                .find(|u| Some(u.id().clone()) == process.user_id().cloned())
                .map(|user| user.name().to_string())
                .unwrap_or_default();
            processes.push(ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string_lossy().to_string(),
                username,
                state: process.status().to_string(),
                cpu_usage_percent: process.cpu_usage() as f64,
                physical_memory_bytes: process.memory(),
                virtual_memory_bytes: process.virtual_memory(),
                disk_read_bytes_per_sec: process.disk_usage().read_bytes,
                disk_write_bytes_per_sec: process.disk_usage().written_bytes,
                threads: 0,    // todo
                open_files: 0, // todo
                start_time_epoch_seconds: process.start_time() as i64,
                gpu_usage: None, // should just be populated by the user from GPU subscription
                parent_pid: process.parent().map(|parent| parent.as_u32()),
                cmdline: Some(
                    process
                        .cmd()
                        .iter()
                        .map(|cmdopt| cmdopt.to_string_lossy().to_string())
                        .collect(),
                ),
                cwd: process.cwd().map(|cwd| cwd.to_string_lossy().to_string()),
                environment: vec![], // todo
                io_priority: None,   // todo
                nice_value: None,    // todo
            });
        }

        Ok(processes)
    }
}
