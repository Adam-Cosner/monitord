use crate::config::CpuCollectorConfig;
use crate::error::{CollectorError, Result};
use crate::traits::Collector;
use monitord_protocols::monitord::{CoreInfo, CpuCache, CpuInfo};
use raw_cpuid::{CpuId, CpuIdReaderNative};
use std::any::Any;
use std::time::Duration;
use sysinfo::{CpuRefreshKind, System};
use tracing::{debug, info, warn};

pub struct CpuCollector {
    config: CpuCollectorConfig,
    system: System,
    cpuid: Option<CpuId<CpuIdReaderNative>>,
}

impl Collector for CpuCollector {
    type Data = CpuInfo;
    type Config = CpuCollectorConfig;

    fn new(config: Self::Config) -> Result<Self> {
        debug!("Initializing CPU collector with config: {:?}", config);

        if !config.enabled {
            info!("CPU collector is disabled");
            return Err(CollectorError::ConfigurationError(
                "CPU collector is disabled".into(),
            ));
        }

        let mut system = System::new();
        system.refresh_cpu_specifics(CpuRefreshKind::everything());

        // Initialize cpuid if possible
        let cpuid = match CpuId::new() {
            cpuid => Some(cpuid),
            #[allow(clippy::needless_return)]
            _ => {
                warn!("Unable to initialize CPUID, some CPU information will be unavailable");
                return Ok(Self {
                    config,
                    system,
                    cpuid: None,
                });
            }
        };

        info!("CPU collector initialized");
        Ok(Self {
            config,
            system,
            cpuid,
        })
    }

    fn collect(&mut self) -> Result<Self::Data> {
        debug!("Collecting CPU information");

        // Refresh CPU information
        self.system.refresh_cpu_all();

        // Get global CPU usage
        let global_usage = self.system.global_cpu_usage() as f64;

        // Collect core information if enabled
        let mut core_info = Vec::new();
        if self.config.collect_per_core {
            for (idx, cpu) in self.system.cpus().iter().enumerate() {
                let core = CoreInfo {
                    core_id: idx as u32,
                    frequency_mhz: cpu.frequency() as f64,
                    utilization_percent: cpu.cpu_usage() as f64,
                    temperature_celsius: 0.0, // Not available through sysinfo
                    min_frequency_mhz: None,
                    max_frequency_mhz: None,
                };
                core_info.push(core);
            }
        }

        // Get CPU cache information if enabled and available
        let cache_info = if self.config.collect_cache_info {
            match self.get_cache_info() {
                Ok(cache) => cache,
                Err(e) => {
                    warn!("Failed to collect CPU cache info: {}", e);
                    CpuCache::default()
                }
            }
        } else {
            CpuCache::default()
        };

        // Get CPU flags if cpuid is available
        let cpu_flags = self.get_cpu_flags();

        // Build the CPU info message
        let cpu_info = CpuInfo {
            model_name: self.get_model_name(),
            physical_cores: self.get_physical_core_count(),
            logical_cores: self.system.cpus().len() as u32,
            global_utilization_percent: global_usage,
            core_info,
            cache_info: Some(cache_info),
            scaling_governor: None, // Not available through sysinfo
            architecture: std::env::consts::ARCH.to_string(),
            cpu_flags,
        };

        debug!("CPU information collected");
        Ok(cpu_info)
    }
}

impl CpuCollector {
    /// Get CPU model name
    fn get_model_name(&self) -> String {
        if let Some(ref cpuid) = self.cpuid {
            if let Some(brand_string) = cpuid.get_processor_brand_string() {
                return brand_string.as_str().trim().to_string();
            }
        }
        // Fallback to sysinfo name or unknown
        self.system
            .cpus()
            .first()
            .map(|cpu| cpu.name().to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    }

    /// Get physical core count
    fn get_physical_core_count(&self) -> u32 {
        self.system
            .physical_core_count()
            .unwrap_or_else(|| self.system.cpus().len() / 2) as u32
    }

    /// Get CPU cache information
    fn get_cache_info(&self) -> Result<CpuCache> {
        if let Some(ref cpuid) = self.cpuid {
            if let Some(cache_info) = cpuid.get_cache_parameters() {
                // For simplicity, we assume all cores have the same cache configuration
                let mut l1d = 0;
                let mut l1i = 0;
                let mut l2 = 0;
                let mut l3 = 0;

                for cache in cache_info {
                    let size = cache.associativity()
                        * cache.physical_line_partitions()
                        * cache.coherency_line_size()
                        * cache.sets();
                    match cache.level() {
                        1 => {
                            if cache.cache_type() == raw_cpuid::CacheType::Data {
                                l1d = size as u32 / 1024; // Convert bytes to KB
                            } else if cache.cache_type() == raw_cpuid::CacheType::Instruction {
                                l1i = size as u32 / 1024;
                            }
                        }
                        2 => l2 = size as u32 / 1024,
                        3 => l3 = size as u32 / 1024,
                        _ => (),
                    }
                }

                return Ok(CpuCache {
                    l1_data_kb: l1d,
                    l1_instruction_kb: l1i,
                    l2_kb: l2,
                    l3_kb: l3,
                });
            }
        }

        Err(CollectorError::ResourceNotAvailable(
            "CPU cache information not available".into(),
        ))
    }

    /// Get CPU flags
    fn get_cpu_flags(&self) -> Vec<String> {
        if let Some(ref cpuid) = self.cpuid {
            if let Some(features) = cpuid.get_feature_info() {
                let mut flags = Vec::new();

                // This is not exhaustive but covers common flags
                if features.has_sse() {
                    flags.push("sse".to_string());
                }
                if features.has_sse2() {
                    flags.push("sse2".to_string());
                }
                if features.has_sse3() {
                    flags.push("sse3".to_string());
                }
                if features.has_ssse3() {
                    flags.push("ssse3".to_string());
                }
                if features.has_sse41() {
                    flags.push("sse4.1".to_string());
                }
                if features.has_sse42() {
                    flags.push("sse4.2".to_string());
                }
                if features.has_avx() {
                    flags.push("avx".to_string());
                }

                if let Some(extended_features) = cpuid.get_extended_feature_info() {
                    if extended_features.has_avx2() {
                        flags.push("avx2".to_string());
                    }
                    if extended_features.has_avx512f() {
                        flags.push("avx512f".to_string());
                    }
                }

                return flags;
            }
        }

        Vec::new()
    }
}
