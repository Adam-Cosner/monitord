use crate::error::CollectionError;
use config::CpuCollectorConfig;
use monitord_protocols::protocols::CpuInfo;
use tracing::{info, warn};

#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::path::Path;

pub mod config;

/// Collects CPU information
pub struct CpuCollector {
    system: sysinfo::System,
    cpuid: raw_cpuid::CpuId<raw_cpuid::CpuIdReaderNative>,
    config: CpuCollectorConfig,
}

impl CpuCollector {
    pub fn new(config: CpuCollectorConfig) -> Result<Self, CollectionError> {
        let system = sysinfo::System::new_with_specifics(
            sysinfo::RefreshKind::nothing().with_cpu(sysinfo::CpuRefreshKind::everything()),
        );

        let cpuid = raw_cpuid::CpuId::new();

        info!("Initialized CPU collector");
        Ok(Self {
            system,
            cpuid,
            config,
        })
    }

    #[cfg(target_os = "linux")]
    /// Get the CPU temperature in Celsius
    /// This function attempts to read the CPU temperature from the thermal zone
    /// On Linux, we use /sys/class/thermal/thermal_zone* to get thermal information
    fn get_cpu_temperature(&self) -> Option<f64> {
        // Try to find CPU thermal zones
        // This is a simplified approach that returns the first thermal zone that looks like a CPU
        for i in 0..20 {
            // Check the first 20 thermal zones
            let zone_path = format!("/host-sys/class/thermal/thermal_zone{}", i);
            if !Path::new(&zone_path).exists() {
                continue;
            }

            // Try to read the type of this thermal zone
            let type_path = format!("{}/type", zone_path);
            match fs::read_to_string(&type_path) {
                Ok(zone_type) => {
                    // Look for zones that contain CPU, processor, x86, or core in their name
                    let zone_type = zone_type.trim().to_lowercase();
                    if zone_type.contains("cpu")
                        || zone_type.contains("processor")
                        || zone_type.contains("x86")
                        || zone_type.contains("core")
                    {
                        // Read the temperature
                        let temp_path = format!("{}/temp", zone_path);
                        if let Ok(temp_str) = fs::read_to_string(&temp_path) {
                            if let Ok(temp) = temp_str.trim().parse::<u32>() {
                                // Temperature is usually reported in millidegrees Celsius
                                return Some(temp as f64 / 1000.0);
                            }
                        }
                    }
                }
                Err(_) => continue,
            }
        }

        None
    }

    #[cfg(not(target_os = "linux"))]
    fn get_cpu_temperature(&self) -> Option<f64> {
        None
    }

    #[cfg(target_os = "linux")]
    /// Get the scaling governor for a specific CPU core
    /// On Linux, this is read from /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
    fn get_scaling_governor(&self, core_id: u32) -> Option<String> {
        let governor_path = format!(
            "/host-sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor",
            core_id
        );

        match fs::read_to_string(&governor_path) {
            Ok(governor) => Some(governor.trim().to_string()),
            Err(e) => {
                warn!(
                    "Failed to read scaling governor for core {}: {}",
                    core_id, e
                );
                None
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn get_scaling_governor(&self, _core_id: u32) -> Option<String> {
        None
    }

    #[cfg(target_os = "linux")]
    /// Get the minimum frequency for a specific CPU core in MHz
    /// On Linux, this is read from /sys/devices/system/cpu/cpu*/cpufreq/scaling_min_freq
    fn get_min_frequency(&self, core_id: u32) -> Option<f64> {
        let freq_path = format!(
            "/host-sys/devices/system/cpu/cpu{}/cpufreq/scaling_min_freq",
            core_id
        );

        match fs::read_to_string(&freq_path) {
            Ok(freq_str) => {
                if let Ok(freq_khz) = freq_str.trim().parse::<u32>() {
                    // Convert from KHz to MHz
                    Some(freq_khz as f64 / 1000.0)
                } else {
                    warn!("Failed to parse min frequency for core {}", core_id);
                    None
                }
            }
            Err(e) => {
                warn!("Failed to read min frequency for core {}: {}", core_id, e);
                None
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn get_min_frequency(&self, _core_id: u32) -> Option<f64> {
        None
    }

    #[cfg(target_os = "linux")]
    /// Get the maximum frequency for a specific CPU core in MHz
    /// On Linux, this is read from /sys/devices/system/cpu/cpu*/cpufreq/scaling_max_freq
    fn get_max_frequency(&self, core_id: u32) -> Option<f64> {
        let freq_path = format!(
            "/host-sys/devices/system/cpu/cpu{}/cpufreq/scaling_max_freq",
            core_id
        );

        match fs::read_to_string(&freq_path) {
            Ok(freq_str) => {
                if let Ok(freq_khz) = freq_str.trim().parse::<u32>() {
                    // Convert from KHz to MHz
                    Some(freq_khz as f64 / 1000.0)
                } else {
                    warn!("Failed to parse max frequency for core {}", core_id);
                    None
                }
            }
            Err(e) => {
                warn!("Failed to read max frequency for core {}: {}", core_id, e);
                None
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn get_max_frequency(&self, _core_id: u32) -> Option<f64> {
        None
    }

    /// Get the global scaling governor for the system
    /// This reads the governor from the first CPU and assumes it's the same for all CPUs
    fn get_global_scaling_governor(&self) -> Option<String> {
        self.get_scaling_governor(0)
    }
}

impl super::Collector for CpuCollector {
    type CollectedData = CpuInfo;
    type CollectorConfig = CpuCollectorConfig;

    fn name(&self) -> &'static str {
        "cpu"
    }

    fn config(&self) -> &Self::CollectorConfig {
        &self.config
    }

    fn collect(&mut self) -> Result<Self::CollectedData, CollectionError> {
        if !self.config.enabled {
            return Err(CollectionError::Disabled);
        }
        // Refresh the system
        self.system.refresh_cpu_all();

        // Get processor and feature data from cpuid
        let feature_info = self.cpuid.get_feature_info();
        let extended_features = self.cpuid.get_extended_feature_info();
        let processor_brand = self.cpuid.get_processor_brand_string();

        // Get cache info if available
        let cache_info = self.cpuid.get_cache_parameters();

        // Get CPU flags
        let mut cpu_flags = Vec::new();
        if let Some(features) = feature_info {
            // Add basic CPU flags
            if features.has_sse() {
                cpu_flags.push("sse".to_string());
            }
            if features.has_sse2() {
                cpu_flags.push("sse2".to_string());
            }
            if features.has_sse3() {
                cpu_flags.push("sse3".to_string());
            }
            if features.has_ssse3() {
                cpu_flags.push("ssse3".to_string());
            }
            if features.has_sse41() {
                cpu_flags.push("sse4.1".to_string());
            }
            if features.has_sse42() {
                cpu_flags.push("sse4.2".to_string());
            }
            if features.has_avx() {
                cpu_flags.push("avx".to_string());
            }
            if features.has_fma() {
                cpu_flags.push("fma".to_string());
            }
        }

        if let Some(features) = extended_features {
            // Add extended CPU flags
            if features.has_avx2() {
                cpu_flags.push("avx2".to_string());
            }
            if features.has_avx512f() {
                cpu_flags.push("avx512f".to_string());
            }
        }

        // Get caches
        let mut cache = monitord_protocols::monitord::CpuCache {
            l1_data_kb: 0,
            l1_instruction_kb: 0,
            l2_kb: 0,
            l3_kb: 0,
        };

        if let Some(cache_params) = cache_info {
            for param in cache_params {
                let cache_size = param.associativity()
                    * param.physical_line_partitions()
                    * param.coherency_line_size()
                    * param.sets();
                match param.level() {
                    1 => {
                        if param.cache_type() == raw_cpuid::CacheType::Data {
                            cache.l1_data_kb = cache_size as u32 / 1024;
                        } else if param.cache_type() == raw_cpuid::CacheType::Instruction {
                            cache.l1_instruction_kb = cache_size as u32 / 1024;
                        }
                    }
                    2 => cache.l2_kb = cache_size as u32 / 1024,
                    3 => cache.l3_kb = cache_size as u32 / 1024,
                    _ => {}
                }
            }
        }

        // Get core info
        let mut core_info = Vec::new();
        let physical_cores = self.system.physical_core_count().unwrap_or(1) as u32;
        let global_cpu_usage = self.system.global_cpu_usage() as f64;

        // Get the global CPU temperature
        let cpu_temp = self.get_cpu_temperature();

        for (i, cpu) in self.system.cpus().iter().enumerate() {
            let core_id = i as u32;
            core_info.push(monitord_protocols::monitord::CoreInfo {
                core_id,
                frequency_mhz: cpu.frequency() as f64,
                utilization_percent: cpu.cpu_usage() as f64,
                temperature_celsius: cpu_temp.unwrap_or(0.0), // Use the same temperature for all cores
                min_frequency_mhz: self.get_min_frequency(core_id),
                max_frequency_mhz: self.get_max_frequency(core_id),
            });
        }

        // Build the CPU info
        let cpu_info = CpuInfo {
            model_name: processor_brand
                .map_or_else(|| "Unknown".to_string(), |brand| brand.as_str().to_string()),
            physical_cores,
            logical_cores: self.system.cpus().len() as u32,
            global_utilization_percent: global_cpu_usage,
            core_info,
            cache_info: Some(cache),
            scaling_governor: self.get_global_scaling_governor(),
            architecture: std::env::consts::ARCH.to_string(),
            cpu_flags,
        };

        Ok(cpu_info)
    }
}
