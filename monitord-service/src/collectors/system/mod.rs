use crate::error::CollectionError;
use monitord_protocols::monitord::SystemInfo;

pub mod config;

pub struct SystemCollector {
    config: config::SystemCollectorConfig,
}

impl SystemCollector {
    pub fn new(config: config::SystemCollectorConfig) -> Result<Self, CollectionError> {
        Ok(Self { config })
    }
}

impl super::Collector for SystemCollector {
    type CollectedData = SystemInfo;
    type CollectorConfig = config::SystemCollectorConfig;

    fn name(&self) -> &'static str {
        "system"
    }

    fn config(&self) -> &Self::CollectorConfig {
        &self.config
    }

    fn collect(&mut self) -> Result<Self::CollectedData, CollectionError> {
        let load_average = sysinfo::System::load_average();

        Ok(SystemInfo {
            hostname: sysinfo::System::host_name().unwrap_or_default(),
            os_name: sysinfo::System::distribution_id(),
            os_version: sysinfo::System::os_version().unwrap_or_default(),
            kernel_version: sysinfo::System::kernel_version().unwrap_or_default(),
            process_count: 0,   // todo
            thread_count: 0,    // todo
            open_file_count: 0, // todo
            uptime_seconds: sysinfo::System::uptime(),
            load_average_1m: load_average.one,
            load_average_5m: load_average.five,
            load_average_15m: load_average.fifteen,
            architecture: sysinfo::System::cpu_arch(),
            boot_time: sysinfo::System::boot_time(),
            vendor: None,              // todo
            virtualization: None,      // todo
            security_features: vec![], // todo
        })
    }
}
