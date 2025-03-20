use crate::config::SystemCollectorConfig;
use crate::error::{CollectorError, Result};
use crate::traits::Collector;
use crate::CollectorConfig;
use monitord_protocols::monitord::SystemInfo;
use sysinfo::{ProcessesToUpdate, System};
use tracing::{debug, info};

pub struct SystemCollector {
    config: SystemCollectorConfig,
    system: System,
}

impl Collector for SystemCollector {
    type Data = SystemInfo;
    type Config = SystemCollectorConfig;

    fn new(config: Self::Config) -> Result<Self> {
        debug!("Initializing System collector with config: {:?}", config);

        if !config.is_enabled() {
            info!("System collector is disabled");
            return Err(CollectorError::ConfigurationError(
                "System collector is disabled".into(),
            ));
        }

        let mut system = System::new();
        // Load system information
        system.refresh_all();

        info!("System collector initialized");
        Ok(Self { config, system })
    }

    fn collect(&mut self) -> Result<Self::Data> {
        debug!("Collecting system information");

        // Refresh system information
        self.system.refresh_all();

        // Get process and thread counts if enabled
        let (process_count, thread_count) = if self.config.collect_thread_count {
            self.system.refresh_processes(ProcessesToUpdate::All, true);
            let processes = self
                .system
                .processes()
                .iter()
                .filter(|(_, proc)| proc.thread_kind().is_none())
                .count() as u32;
            let threads = self.system.processes().len() as u32;
            (processes, threads)
        } else {
            self.system.refresh_processes(ProcessesToUpdate::All, true);
            let processes = self
                .system
                .processes()
                .iter()
                .filter(|(_, proc)| proc.thread_kind().is_none())
                .count() as u32;
            (processes, 0)
        };

        // Get load averages if enabled
        let (load_1m, load_5m, load_15m) = if self.config.collect_load_avg {
            match System::load_average() {
                l => (l.one, l.five, l.fifteen),
            }
        } else {
            (0.0, 0.0, 0.0)
        };

        // Get open file count
        let open_file_count = if self.config.collect_open_files {
            // Not directly available through sysinfo, would need platform-specific code
            0
        } else {
            0
        };

        // Get system uptime
        let uptime_seconds = System::uptime();

        // Get boot time
        let boot_time = chrono::Utc::now().timestamp() as u64 - uptime_seconds;

        // Build the system info message
        let system_info = SystemInfo {
            hostname: System::host_name().unwrap_or_else(|| "unknown".to_string()),
            os_name: System::distribution_id(),
            os_version: System::os_version().unwrap_or_else(|| "unknown".to_string()),
            kernel_version: System::kernel_version().unwrap_or_else(|| "unknown".to_string()),
            process_count,
            thread_count,
            open_file_count,
            uptime_seconds,
            load_average_1m: load_1m,
            load_average_5m: load_5m,
            load_average_15m: load_15m,
            architecture: std::env::consts::ARCH.to_string(),
            boot_time,
            vendor: None,                  // Not easily available through sysinfo
            virtualization: None,          // Not easily available through sysinfo
            security_features: Vec::new(), // Would need platform-specific detection
        };

        debug!("System information collected");
        Ok(system_info)
    }
}
