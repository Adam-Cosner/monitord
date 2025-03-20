use crate::config::StorageCollectorConfig;
use crate::error::{CollectorError, Result};
use crate::traits::Collector;
use crate::CollectorConfig;
use monitord_protocols::monitord::{SmartData, StorageInfo, StorageList};
use std::collections::HashMap;
use std::time::Duration;
use sysinfo::{Disks, System};
use tracing::{debug, info, warn};

pub struct StorageCollector {
    config: StorageCollectorConfig,
    disks: Disks,
    // Store previous values to calculate rates
    previous_read_bytes: HashMap<String, u64>,
    previous_write_bytes: HashMap<String, u64>,
    previous_time: std::time::Instant,
}

impl Collector for StorageCollector {
    type Data = StorageList;
    type Config = StorageCollectorConfig;

    fn new(config: Self::Config) -> Result<Self> {
        debug!("Initializing Storage collector with config: {:?}", config);

        if !config.is_enabled() {
            info!("Storage collector is disabled");
            return Err(CollectorError::ConfigurationError(
                "Storage collector is disabled".into(),
            ));
        }

        let disks = Disks::new_with_refreshed_list();

        // Initialize previous values
        let previous_read_bytes = HashMap::new();
        let previous_write_bytes = HashMap::new();

        info!("Storage collector initialized");
        Ok(Self {
            config,
            disks,
            previous_read_bytes,
            previous_write_bytes,
            previous_time: std::time::Instant::now(),
        })
    }

    fn collect(&mut self) -> Result<Self::Data> {
        debug!("Collecting storage information");

        // Refresh storage information

        // Calculate time elapsed since last collection
        let now = std::time::Instant::now();
        let elapsed_secs = now.duration_since(self.previous_time).as_secs_f64();
        self.previous_time = now;

        let mut storage_infos = Vec::new();

        for disk in self.disks.iter() {
            let device_name = disk.name().to_string_lossy().to_string();

            // Get current read/write values
            let read_bytes = disk.usage().read_bytes;
            let write_bytes = disk.usage().written_bytes;

            // Calculate IO rates if enabled
            let (read_rate, write_rate) = if self.config.collect_io_stats {
                let read_rate = if let Some(&prev_read) = self.previous_read_bytes.get(&device_name)
                {
                    if elapsed_secs > 0.0 {
                        ((read_bytes - prev_read) as f64 / elapsed_secs) as u64
                    } else {
                        0
                    }
                } else {
                    0
                };

                let write_rate =
                    if let Some(&prev_write) = self.previous_write_bytes.get(&device_name) {
                        if elapsed_secs > 0.0 {
                            ((write_bytes - prev_write) as f64 / elapsed_secs) as u64
                        } else {
                            0
                        }
                    } else {
                        0
                    };

                (read_rate, write_rate)
            } else {
                (0, 0)
            };

            // Get disk type - this is a simplistic heuristic
            let device_type = if device_name.contains("nvme") {
                "NVMe"
            } else if device_name.contains("sd") {
                "SSD"
            } else {
                "Unknown"
            }
            .to_string();

            // Get filesystem info
            let filesystem_type = match disk.file_system().to_string_lossy().to_string() {
                s if s.is_empty() => "Unknown".to_string(),
                s => s,
            };

            // Get mount point
            let mount_point = disk.mount_point().to_string_lossy().to_string();

            // Get space information
            let total_space_bytes = disk.total_space();
            let available_space_bytes = disk.available_space();
            let used_space_bytes = total_space_bytes - available_space_bytes;

            // Get S.M.A.R.T data if enabled
            let smart_data = if self.config.collect_smart {
                // S.M.A.R.T. data requires specialized libraries
                // This is a placeholder for actual implementation
                Some(SmartData {
                    health_status: "OK".to_string(),
                    power_on_hours: None,
                    power_cycle_count: None,
                    reallocated_sectors: None,
                    remaining_life_percent: None,
                })
            } else {
                None
            };

            // Create StorageInfo object
            let storage_info = StorageInfo {
                device_name,
                device_type,
                model: "Unknown".to_string(), // Not available from sysinfo
                filesystem_type,
                mount_point,
                total_space_bytes,
                available_space_bytes,
                read_bytes_per_sec: read_rate,
                write_bytes_per_sec: write_rate,
                io_time_ms: 0,               // Not available from sysinfo
                temperature_celsius: None,   // Not available from sysinfo
                lifetime_writes_bytes: None, // Not available from sysinfo
                serial_number: None,         // Not available from sysinfo
                partition_label: None,       // Not available from sysinfo
                used_space_bytes,
                smart_data,
            };

            storage_infos.push(storage_info);

            // Update previous values
            self.previous_read_bytes
                .insert(disk.name().to_string_lossy().to_string(), read_bytes);
            self.previous_write_bytes
                .insert(disk.name().to_string_lossy().to_string(), write_bytes);
        }

        debug!(
            "Storage information collected for {} device(s)",
            storage_infos.len()
        );
        Ok(StorageList {
            storages: storage_infos,
        })
    }
}
