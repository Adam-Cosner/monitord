use crate::error::CollectionError;
use config::GpuCollectorConfig;
use monitord_protocols::protocols::GpuInfo;
use tracing::{debug, info, warn};

pub mod config;

mod amd;
mod fallback;
mod intel;
mod nvidia;

// Main GPU collector that manages vendor-specific collectors
pub struct GpuCollector {
    config: GpuCollectorConfig,
    nvidia_collector: Option<nvidia::NvidiaGpuCollector>,
    amd_collector: Option<amd::AmdGpuCollector>,
    intel_collector: Option<intel::IntelGpuCollector>,
}

impl GpuCollector {
    pub fn new(config: GpuCollectorConfig) -> Result<Self, CollectionError> {
        let mut collector = Self {
            config,
            nvidia_collector: None,
            amd_collector: None,
            intel_collector: None,
        };

        // Initialize vendor-specific collectors based on configuration
        collector.init_collectors()?;

        Ok(collector)
    }

    fn init_collectors(&mut self) -> Result<(), CollectionError> {
        // Initialize NVIDIA collector if enabled
        if self.config.nvidia_enabled {
            match nvidia::NvidiaGpuCollector::new() {
                Ok(collector) => {
                    debug!("Initialized NVIDIA GPU collector");
                    self.nvidia_collector = Some(collector);
                }
                Err(e) => {
                    warn!("Failed to initialize NVIDIA GPU collector: {}", e);
                }
            }
        }

        // Initialize AMD collector if enabled
        if self.config.amd_enabled {
            match amd::AmdGpuCollector::new() {
                Ok(collector) => {
                    debug!("Initialized AMD GPU collector");
                    self.amd_collector = Some(collector);
                }
                Err(e) => {
                    warn!("Failed to initialize AMD GPU collector: {}", e);
                }
            }
        }

        // Initialize Intel collector if enabled
        if self.config.intel_enabled {
            match intel::IntelGpuCollector::new() {
                Ok(collector) => {
                    debug!("Initialized Intel GPU collector");
                    self.intel_collector = Some(collector);
                }
                Err(e) => {
                    warn!("Failed to initialize Intel GPU collector: {}", e);
                }
            }
        }

        Ok(())
    }
}

impl super::Collector for GpuCollector {
    type CollectedData = Vec<GpuInfo>;
    type CollectorConfig = GpuCollectorConfig;

    fn name(&self) -> &'static str {
        "gpu"
    }

    fn config(&self) -> &Self::CollectorConfig {
        &self.config
    }

    fn collect(&mut self) -> Result<Self::CollectedData, CollectionError> {
        if !self.config.enabled {
            return Err(CollectionError::Disabled);
        }

        let mut gpu_infos = Vec::new();

        // Collect from NVIDIA
        if let Some(collector) = &mut self.nvidia_collector {
            match collector.collect() {
                Ok(infos) => gpu_infos.extend(infos),
                Err(e) => warn!("Error collecting NVIDIA GPU info: {}", e),
            }
        }

        // Collect from AMD
        if let Some(collector) = &mut self.amd_collector {
            match collector.collect() {
                Ok(infos) => gpu_infos.extend(infos),
                Err(e) => warn!("Error collecting AMD GPU info: {}", e),
            }
        }

        // Collect from Intel
        if let Some(collector) = &mut self.intel_collector {
            match collector.collect() {
                Ok(infos) => gpu_infos.extend(infos),
                Err(e) => warn!("Error collecting Intel GPU info: {}", e),
            }
        }

        if gpu_infos.is_empty() {
            warn!("No GPU information collected!");
        }

        Ok(gpu_infos)
    }
}

// To be placed in mod.rs
trait VendorGpuCollector: Send + Sync {
    fn init(&mut self) -> Result<(), CollectionError>;
    fn collect(&mut self) -> Result<Vec<GpuInfo>, CollectionError>;
}
