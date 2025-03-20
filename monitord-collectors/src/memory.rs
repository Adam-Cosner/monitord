use crate::config::MemoryCollectorConfig;
use crate::error::{CollectorError, Result};
use crate::traits::Collector;
use monitord_protocols::monitord::{DramInfo, MemoryInfo};
use sysinfo::System;
use tracing::{debug, info};

pub struct MemoryCollector {
    config: MemoryCollectorConfig,
    system: System,
}

impl Collector for MemoryCollector {
    type Data = MemoryInfo;
    type Config = MemoryCollectorConfig;

    fn new(config: Self::Config) -> Result<Self> {
        debug!("Initializing Memory collector with config: {:?}", config);

        if !config.enabled {
            info!("Memory collector is disabled");
            return Err(CollectorError::ConfigurationError(
                "Memory collector is disabled".into(),
            ));
        }

        let mut system = System::new();
        system.refresh_memory();

        info!("Memory collector initialized");
        Ok(Self { config, system })
    }

    fn collect(&mut self) -> Result<Self::Data> {
        debug!("Collecting memory information");

        // Refresh memory information
        self.system.refresh_memory();

        // Get total and used memory
        let total_memory = self.system.total_memory();
        let used_memory = self.system.used_memory();
        let free_memory = self.system.free_memory();
        let available_memory = self.system.available_memory();

        // Get swap information if enabled
        let (swap_total, swap_used, swap_free) = if self.config.collect_swap_info {
            (
                self.system.total_swap(),
                self.system.used_swap(),
                self.system.free_swap(),
            )
        } else {
            (0, 0, 0)
        };

        // Calculate memory load percentage
        let memory_load = if total_memory > 0 {
            (used_memory as f64 / total_memory as f64) * 100.0
        } else {
            0.0
        };

        // Mock DRAM info - this would require additional libraries in production
        let dram_info = if self.config.collect_dram_info {
            Some(DramInfo {
                frequency_mhz: 0.0, // Not available through sysinfo
                memory_type: "Unknown".to_string(),
                slots_total: 0,
                slots_used: 0,
                manufacturer: None,
                part_number: None,
            })
        } else {
            None
        };

        // Build the memory info message
        let memory_info = MemoryInfo {
            total_memory_bytes: total_memory,
            used_memory_bytes: used_memory,
            free_memory_bytes: free_memory,
            available_memory_bytes: available_memory,
            swap_total_bytes: swap_total,
            swap_used_bytes: swap_used,
            swap_free_bytes: swap_free,
            cached_memory_bytes: 0, // Not directly available through sysinfo
            shared_memory_bytes: 0, // Not directly available through sysinfo
            memory_load_percent: memory_load,
            dram_info,
        };

        debug!("Memory information collected");
        Ok(memory_info)
    }
}
