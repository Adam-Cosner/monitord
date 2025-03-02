use crate::error::CollectionError;
use monitord_protocols::monitord::DramInfo as ProtoDramInfo;
use monitord_protocols::monitord::MemoryInfo as ProtoMemoryInfo;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::Command;

use super::config::CollectorConfig;

pub struct MemoryCollector {
    system: sysinfo::System,
    config: CollectorConfig,
}

impl MemoryCollector {
    pub fn new(config: CollectorConfig) -> Result<Self, CollectionError> {
        Ok(Self {
            system: sysinfo::System::new_with_specifics(
                sysinfo::RefreshKind::nothing()
                    .with_memory(sysinfo::MemoryRefreshKind::everything()),
            ),
            config,
        })
    }

    #[cfg(target_os = "linux")]
    fn get_cached_memory() -> Result<u64, CollectionError> {
        let file = File::open("/proc/meminfo").map_err(|e| {
            CollectionError::Generic(format!("Failed to open /proc/meminfo: {}", e))
        })?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.map_err(|e| {
                CollectionError::Generic(format!("Failed to read /proc/meminfo: {}", e))
            })?;

            if line.starts_with("Cached:") {
                // Format is typically "Cached:       12345678 kB"
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let kb_value = parts[1].parse::<u64>().map_err(|e| {
                        CollectionError::Generic(format!(
                            "Failed to parse cached memory value: {}",
                            e
                        ))
                    })?;
                    // Convert from KB to bytes
                    return Ok(kb_value * 1024);
                }
            }
        }

        // If not found, return 0
        Ok(0)
    }

    #[cfg(not(target_os = "linux"))]
    fn get_cached_memory() -> Result<u64, CollectionError> {
        // On non-Linux platforms, we might not have easy access to cached memory
        // Return 0 or try to find platform-specific ways to get this information
        Ok(0)
    }

    #[cfg(target_os = "linux")]
    fn get_shared_memory() -> Result<u64, CollectionError> {
        let file = File::open("/proc/meminfo").map_err(|e| {
            CollectionError::Generic(format!("Failed to open /proc/meminfo: {}", e))
        })?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.map_err(|e| {
                CollectionError::Generic(format!("Failed to read /proc/meminfo: {}", e))
            })?;

            if line.starts_with("Shmem:") {
                // Format is typically "Shmem:       12345678 kB"
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let kb_value = parts[1].parse::<u64>().map_err(|e| {
                        CollectionError::Generic(format!(
                            "Failed to parse shared memory value: {}",
                            e
                        ))
                    })?;
                    // Convert from KB to bytes
                    return Ok(kb_value * 1024);
                }
            }
        }

        // If not found, return 0
        Ok(0)
    }

    #[cfg(not(target_os = "linux"))]
    fn get_shared_memory() -> Result<u64, CollectionError> {
        // On non-Linux platforms, we might not have easy access to shared memory
        // Return 0 or try to find platform-specific ways to get this information
        Ok(0)
    }

    fn get_dram_info() -> Result<Option<ProtoDramInfo>, CollectionError> {
        #[cfg(target_os = "linux")]
        {
            // Try to get DRAM info using dmidecode (requires root privileges)
            // This won't work without elevated permissions, so we'll handle the error gracefully
            let output = Command::new("dmidecode").arg("-t").arg("memory").output();

            match output {
                Ok(output) if output.status.success() => {
                    let output_str = String::from_utf8_lossy(&output.stdout);

                    // Parse memory information from dmidecode output
                    let mut memory_type = String::new();
                    let mut frequency = 0.0;
                    let mut slots_total = 0;
                    let mut slots_used = 0;
                    let mut manufacturer = None;
                    let mut part_number = None;

                    // Count memory device sections
                    for line in output_str.lines() {
                        if line.contains("Memory Device") {
                            slots_total += 1;
                        }

                        if line.contains("Size:") && !line.contains("No Module Installed") {
                            slots_used += 1;
                        }

                        // Get the memory type from the first populated slot
                        if memory_type.is_empty()
                            && line.contains("Type:")
                            && !line.contains("Unknown")
                        {
                            let parts: Vec<&str> = line.split(':').collect();
                            if parts.len() >= 2 {
                                memory_type = parts[1].trim().to_string();
                            }
                        }

                        // Get the frequency from the first populated slot
                        if frequency == 0.0 && line.contains("Speed:") && line.contains("MHz") {
                            let parts: Vec<&str> = line.split(':').collect();
                            if parts.len() >= 2 {
                                let speed_part = parts[1].trim();
                                if let Some(mhz_pos) = speed_part.find("MHz") {
                                    if let Ok(freq) = speed_part[..mhz_pos].trim().parse::<f64>() {
                                        frequency = freq;
                                    }
                                }
                            }
                        }

                        // Get manufacturer
                        if manufacturer.is_none()
                            && line.contains("Manufacturer:")
                            && !line.contains("Unknown")
                        {
                            let parts: Vec<&str> = line.split(':').collect();
                            if parts.len() >= 2 && !parts[1].trim().is_empty() {
                                manufacturer = Some(parts[1].trim().to_string());
                            }
                        }

                        // Get part number
                        if part_number.is_none()
                            && line.contains("Part Number:")
                            && !line.contains("Unknown")
                        {
                            let parts: Vec<&str> = line.split(':').collect();
                            if parts.len() >= 2 && !parts[1].trim().is_empty() {
                                part_number = Some(parts[1].trim().to_string());
                            }
                        }
                    }

                    if slots_total > 0 {
                        return Ok(Some(ProtoDramInfo {
                            frequency_mhz: frequency,
                            memory_type,
                            slots_total: slots_total as u32,
                            slots_used: slots_used as u32,
                            manufacturer,
                            part_number,
                        }));
                    }
                }
                _ => {} // Ignore errors, as dmidecode typically requires root privileges
            }
        }

        // If we couldn't get detailed DRAM info, return None
        Ok(None)
    }
}

impl super::Collector for MemoryCollector {
    type CollectedData = ProtoMemoryInfo;

    fn name(&self) -> &'static str {
        "memory"
    }

    fn config(&self) -> &CollectorConfig {
        &self.config
    }

    fn collect(&mut self) -> Result<Self::CollectedData, CollectionError> {
        if !self.config.enabled {
            return Err(CollectionError::Disabled);
        }
        self.system.refresh_memory();

        // Get DRAM information if available
        let dram_info = Self::get_dram_info()?;

        // Get cached and shared memory
        let cached_memory_bytes = Self::get_cached_memory()?;
        let shared_memory_bytes = Self::get_shared_memory()?;

        Ok(ProtoMemoryInfo {
            total_memory_bytes: self.system.total_memory(),
            used_memory_bytes: self.system.used_memory(),
            free_memory_bytes: self.system.free_memory(),
            available_memory_bytes: self.system.available_memory(),
            swap_total_bytes: self.system.total_swap(),
            swap_used_bytes: self.system.used_swap(),
            swap_free_bytes: self.system.free_swap(),
            dram_info,
            cached_memory_bytes,
            shared_memory_bytes,
            memory_load_percent: (self.system.used_memory() + self.system.used_swap()) as f64
                / (self.system.total_memory() + self.system.total_swap()) as f64,
        })
    }
}
