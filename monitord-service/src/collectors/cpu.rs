use crate::config::CollectorConfig;
use crate::error::CollectionError;
use monitord_protocols::protocols::CpuInfo;

/// Collects CPU information
pub struct CpuCollector {
    system: sysinfo::System,
    cpuid: raw_cpuid::CpuId<raw_cpuid::CpuIdReaderNative>,
    config: CollectorConfig,
}

impl CpuCollector {
    pub fn new(config: CollectorConfig) -> Result<Self, CollectionError> {
        let system = sysinfo::System::new_with_specifics(
            sysinfo::RefreshKind::nothing().with_cpu(sysinfo::CpuRefreshKind::everything()),
        );

        let cpuid = raw_cpuid::CpuId::new();

        Ok(Self {
            system,
            cpuid,
            config,
        })
    }
}

impl super::Collector for CpuCollector {
    type CollectedData = CpuInfo;

    fn name(&self) -> &'static str {
        "cpu"
    }

    fn config(&self) -> &CollectorConfig {
        &self.config
    }

    fn collect(&mut self) -> Result<Self::CollectedData, CollectionError> {
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

        for (i, cpu) in self.system.cpus().iter().enumerate() {
            core_info.push(monitord_protocols::monitord::CoreInfo {
                core_id: i as u32,
                frequency_mhz: cpu.frequency() as f64,
                utilization_percent: cpu.cpu_usage() as f64,
                temperature_celsius: 0.0, // Could be obtained with additional thermal sensors
                min_frequency_mhz: None,
                max_frequency_mhz: None,
            });
        }

        // Build the CPU info
        let cpu_info = monitord_protocols::protocols::CpuInfo {
            model_name: processor_brand
                .map_or_else(|| "Unknown".to_string(), |brand| brand.as_str().to_string()),
            physical_cores,
            logical_cores: self.system.cpus().len() as u32,
            global_utilization_percent: global_cpu_usage,
            core_info,
            cache_info: Some(cache),
            scaling_governor: None, // Would need to read from /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
            architecture: std::env::consts::ARCH.to_string(),
            cpu_flags,
        };

        Ok(cpu_info)
    }
}
