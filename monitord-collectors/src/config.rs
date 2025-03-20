use std::time::Duration;

/// The base configuration interface for all collectors.
/// Provides a standard way to configure collector intervals and features.
pub trait CollectorConfig {
    /// Get the collection interval for this collector
    fn interval(&self) -> Duration;
    
    /// Check if this collector is enabled
    fn is_enabled(&self) -> bool;
}

/// Configuration for the CPU collector
#[derive(Debug, Clone)]
pub struct CpuCollectorConfig {
    /// Whether this collector is enabled
    pub enabled: bool,
    
    /// How often to collect CPU metrics (in milliseconds)
    pub interval_ms: u64,
    
    /// Whether to collect per-core metrics
    pub collect_per_core: bool,
    
    /// Whether to collect CPU cache information
    pub collect_cache_info: bool,
    
    /// Whether to collect CPU temperature (if available)
    pub collect_temperature: bool,
    
    /// Whether to collect CPU frequency information
    pub collect_frequency: bool,
}

impl Default for CpuCollectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_ms: 1000,
            collect_per_core: true,
            collect_cache_info: true,
            collect_temperature: true,
            collect_frequency: true,
        }
    }
}

impl CollectorConfig for CpuCollectorConfig {
    fn interval(&self) -> Duration {
        Duration::from_millis(self.interval_ms)
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Configuration for the Memory collector
#[derive(Debug, Clone)]
pub struct MemoryCollectorConfig {
    /// Whether this collector is enabled
    pub enabled: bool,
    
    /// How often to collect memory metrics (in milliseconds)
    pub interval_ms: u64,
    
    /// Whether to collect DRAM information
    pub collect_dram_info: bool,
    
    /// Whether to collect swap information
    pub collect_swap_info: bool,
}

impl Default for MemoryCollectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_ms: 1000,
            collect_dram_info: true,
            collect_swap_info: true,
        }
    }
}

impl CollectorConfig for MemoryCollectorConfig {
    fn interval(&self) -> Duration {
        Duration::from_millis(self.interval_ms)
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Configuration for the GPU collector
#[derive(Debug, Clone)]
pub struct GpuCollectorConfig {
    /// Whether this collector is enabled
    pub enabled: bool,
    
    /// How often to collect GPU metrics (in milliseconds)
    pub interval_ms: u64,
    
    /// Whether to collect NVIDIA GPU metrics
    pub collect_nvidia: bool,
    
    /// Whether to collect AMD GPU metrics
    pub collect_amd: bool,
    
    /// Whether to collect Intel GPU metrics
    pub collect_intel: bool,
    
    /// Whether to collect GPU process usage
    pub collect_processes: bool,
}

impl Default for GpuCollectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_ms: 1000,
            collect_nvidia: true,
            collect_amd: true,
            collect_intel: true,
            collect_processes: true,
        }
    }
}

impl CollectorConfig for GpuCollectorConfig {
    fn interval(&self) -> Duration {
        Duration::from_millis(self.interval_ms)
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Configuration for the Network collector
#[derive(Debug, Clone)]
pub struct NetworkCollectorConfig {
    /// Whether this collector is enabled
    pub enabled: bool,
    
    /// How often to collect network metrics (in milliseconds)
    pub interval_ms: u64,
    
    /// Whether to collect packet statistics
    pub collect_packets: bool,
    
    /// Whether to collect error statistics
    pub collect_errors: bool,
}

impl Default for NetworkCollectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_ms: 1000,
            collect_packets: true,
            collect_errors: true,
        }
    }
}

impl CollectorConfig for NetworkCollectorConfig {
    fn interval(&self) -> Duration {
        Duration::from_millis(self.interval_ms)
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Configuration for the Storage collector
#[derive(Debug, Clone)]
pub struct StorageCollectorConfig {
    /// Whether this collector is enabled
    pub enabled: bool,
    
    /// How often to collect storage metrics (in milliseconds)
    pub interval_ms: u64,
    
    /// Whether to collect S.M.A.R.T. data
    pub collect_smart: bool,
    
    /// Whether to collect I/O statistics
    pub collect_io_stats: bool,
}

impl Default for StorageCollectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_ms: 2000, // Storage metrics don't need to be as frequent
            collect_smart: true,
            collect_io_stats: true,
        }
    }
}

impl CollectorConfig for StorageCollectorConfig {
    fn interval(&self) -> Duration {
        Duration::from_millis(self.interval_ms)
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Configuration for the Process collector
#[derive(Debug, Clone)]
pub struct ProcessCollectorConfig {
    /// Whether this collector is enabled
    pub enabled: bool,
    
    /// How often to collect process metrics (in milliseconds)
    pub interval_ms: u64,
    
    /// Maximum number of processes to collect
    pub max_processes: u32,
    
    /// Whether to collect command line arguments
    pub collect_command_line: bool,
    
    /// Whether to collect environment variables
    pub collect_environment: bool,
    
    /// Whether to collect I/O statistics
    pub collect_io_stats: bool,
}

impl Default for ProcessCollectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_ms: 2000, // Process metrics don't need to be as frequent
            max_processes: 100,
            collect_command_line: true,
            collect_environment: false, // This can be sensitive
            collect_io_stats: true,
        }
    }
}

impl CollectorConfig for ProcessCollectorConfig {
    fn interval(&self) -> Duration {
        Duration::from_millis(self.interval_ms)
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Configuration for the System collector
#[derive(Debug, Clone)]
pub struct SystemCollectorConfig {
    /// Whether this collector is enabled
    pub enabled: bool,
    
    /// How often to collect system metrics (in milliseconds)
    pub interval_ms: u64,
    
    /// Whether to collect system load averages
    pub collect_load_avg: bool,
    
    /// Whether to collect open file count
    pub collect_open_files: bool,
    
    /// Whether to collect thread count
    pub collect_thread_count: bool,
}

impl Default for SystemCollectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_ms: 5000, // System info changes very infrequently
            collect_load_avg: true,
            collect_open_files: true,
            collect_thread_count: true,
        }
    }
}

impl CollectorConfig for SystemCollectorConfig {
    fn interval(&self) -> Duration {
        Duration::from_millis(self.interval_ms)
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// A container for all collector configurations
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct CollectorsConfig {
    pub cpu: CpuCollectorConfig,
    pub memory: MemoryCollectorConfig,
    pub gpu: GpuCollectorConfig,
    pub network: NetworkCollectorConfig,
    pub storage: StorageCollectorConfig,
    pub process: ProcessCollectorConfig,
    pub system: SystemCollectorConfig,
}

