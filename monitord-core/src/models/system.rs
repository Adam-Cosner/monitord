// System wide information model
use super::Model;
use crate::error::ModelError;
use monitord_protocols::monitord::{SystemInfo as ProtoSystemInfo, SystemSnapshot as ProtoSystemSnapshot};
use monitord_protocols::protocols::prost_types::Timestamp;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub process_count: u32,
    pub thread_count: u32,
    pub open_file_count: u32,
    pub uptime_seconds: u64,
    pub load_average_1m: f64,
    pub load_average_5m: f64,
    pub load_average_15m: f64,
    
    // Additional fields not in proto
    pub architecture: String,
    pub boot_time: u64,
    pub vendor: Option<String>,
    pub virtualization: Option<String>,
    pub security_features: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    pub timestamp: SystemTime,
    pub system_info: SystemInfo,
    pub cpu_info: Option<super::cpu::CpuInfo>,
    pub memory_info: Option<super::memory::MemoryInfo>,
    pub gpu_info: Vec<super::gpu::GpuInfo>,
    pub network_info: Vec<super::network::NetworkInfo>,
    pub processes: Vec<super::process::ProcessInfo>,
    pub storage_devices: Vec<super::storage::StorageInfo>,
}

impl Model for SystemInfo {
    type ProtoType = ProtoSystemInfo;

    fn into_proto(self) -> Self::ProtoType {
        ProtoSystemInfo {
            hostname: self.hostname,
            os_name: self.os_name,
            os_version: self.os_version,
            kernel_version: self.kernel_version,
            process_count: self.process_count,
            thread_count: self.thread_count,
            open_file_count: self.open_file_count,
            uptime_seconds: self.uptime_seconds,
            load_average_1m: self.load_average_1m,
            load_average_5m: self.load_average_5m,
            load_average_15m: self.load_average_15m,
            architecture: self.architecture,
            boot_time: self.boot_time,
            vendor: self.vendor,
            virtualization: self.virtualization,
            security_features: self.security_features,
        }
    }

    fn from_proto(proto: Self::ProtoType) -> Self {
        Self {
            hostname: proto.hostname,
            os_name: proto.os_name,
            os_version: proto.os_version,
            kernel_version: proto.kernel_version,
            process_count: proto.process_count,
            thread_count: proto.thread_count,
            open_file_count: proto.open_file_count,
            uptime_seconds: proto.uptime_seconds,
            load_average_1m: proto.load_average_1m,
            load_average_5m: proto.load_average_5m,
            load_average_15m: proto.load_average_15m,
            
            architecture: proto.architecture,
            boot_time: proto.boot_time,
            vendor: proto.vendor,
            virtualization: proto.virtualization,
            security_features: proto.security_features,
        }
    }

    fn validate(&self) -> Result<(), ModelError> {
        if self.load_average_1m < 0.0 || self.load_average_5m < 0.0 || self.load_average_15m < 0.0 {
            return Err(ModelError::Validation(
                "Load averages cannot be negative".to_owned(),
            ));
        }

        Ok(())
    }
}

impl Model for SystemSnapshot {
    type ProtoType = ProtoSystemSnapshot;

    fn into_proto(self) -> Self::ProtoType {
        // Convert SystemTime to Timestamp for protobuf
        let duration_since_epoch = self.timestamp
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        
        let timestamp = Timestamp {
            seconds: duration_since_epoch.as_secs() as i64,
            nanos: duration_since_epoch.subsec_nanos() as i32,
        };

        ProtoSystemSnapshot {
            timestamp: Some(timestamp),
            system_info: Some(self.system_info.into_proto()),
            cpu_info: self.cpu_info.map(|cpu| cpu.into_proto()),
            memory_info: self.memory_info.map(|mem| mem.into_proto()),
            gpu_info: self.gpu_info.into_iter().map(|gpu| gpu.into_proto()).collect(),
            network_info: self.network_info.into_iter().map(|net| net.into_proto()).collect(),
            processes: self.processes.into_iter().map(|proc| proc.into_proto()).collect(),
            storage_devices: self.storage_devices.into_iter().map(|storage| storage.into_proto()).collect(),
        }
    }

    fn from_proto(proto: Self::ProtoType) -> Self {
        // Convert Timestamp to SystemTime
        let timestamp = proto.timestamp.unwrap_or_default();
        let system_time = UNIX_EPOCH + std::time::Duration::new(
            timestamp.seconds as u64,
            timestamp.nanos as u32,
        );

        Self {
            timestamp: system_time,
            system_info: proto.system_info.map(SystemInfo::from_proto).unwrap_or_else(|| {
                // Create a default SystemInfo if none is provided
                SystemInfo {
                    hostname: String::new(),
                    os_name: String::new(),
                    os_version: String::new(),
                    kernel_version: String::new(),
                    process_count: 0,
                    thread_count: 0,
                    open_file_count: 0,
                    uptime_seconds: 0,
                    load_average_1m: 0.0,
                    load_average_5m: 0.0,
                    load_average_15m: 0.0,
                    architecture: String::new(),
                    boot_time: 0,
                    vendor: None,
                    virtualization: None,
                    security_features: Vec::new(),
                }
            }),
            cpu_info: proto.cpu_info.map(super::cpu::CpuInfo::from_proto),
            memory_info: proto.memory_info.map(super::memory::MemoryInfo::from_proto),
            gpu_info: proto.gpu_info.into_iter().map(super::gpu::GpuInfo::from_proto).collect(),
            network_info: proto.network_info.into_iter().map(super::network::NetworkInfo::from_proto).collect(),
            processes: proto.processes.into_iter().map(super::process::ProcessInfo::from_proto).collect(),
            storage_devices: proto.storage_devices.into_iter().map(super::storage::StorageInfo::from_proto).collect(),
        }
    }

    fn validate(&self) -> Result<(), ModelError> {
        // Validate system info
        self.system_info.validate()?;
        
        // Validate CPU info if present
        if let Some(cpu) = &self.cpu_info {
            cpu.validate()?;
        }
        
        // Validate memory info if present
        if let Some(memory) = &self.memory_info {
            memory.validate()?;
        }
        
        // Validate all GPUs
        for gpu in &self.gpu_info {
            gpu.validate()?;
        }
        
        // Validate all network interfaces
        for network in &self.network_info {
            network.validate()?;
        }
        
        // Validate all processes
        for process in &self.processes {
            process.validate()?;
        }
        
        // Validate all storage devices
        for storage in &self.storage_devices {
            storage.validate()?;
        }
        
        Ok(())
    }
}

// Additional helper methods
impl SystemInfo {
    /// Calculate threads per process
    pub fn threads_per_process(&self) -> f64 {
        if self.process_count == 0 {
            return 0.0;
        }
        
        self.thread_count as f64 / self.process_count as f64
    }
    
    /// Calculate open files per process
    pub fn open_files_per_process(&self) -> f64 {
        if self.process_count == 0 {
            return 0.0;
        }
        
        self.open_file_count as f64 / self.process_count as f64
    }
    
    /// Determine if load is high (1 minute load average > number of logical CPUs)
    pub fn is_load_high(&self, logical_cpus: u32) -> bool {
        if logical_cpus == 0 {
            return false;
        }
        
        self.load_average_1m > logical_cpus as f64
    }
    
    /// Get the boot time as a system time
    pub fn boot_time_systemtime(&self) -> SystemTime {
        let boot_time_secs = self.boot_time;
        UNIX_EPOCH + std::time::Duration::from_secs(boot_time_secs)
    }
}

impl SystemSnapshot {
    /// Create a new SystemSnapshot with current timestamp
    pub fn new(system_info: SystemInfo) -> Self {
        Self {
            timestamp: SystemTime::now(),
            system_info,
            cpu_info: None,
            memory_info: None,
            gpu_info: Vec::new(),
            network_info: Vec::new(),
            processes: Vec::new(),
            storage_devices: Vec::new(),
        }
    }
    
    /// Get the snapshot age in seconds
    pub fn age_seconds(&self) -> u64 {
        SystemTime::now().duration_since(self.timestamp)
            .unwrap_or_default()
            .as_secs()
    }
    
    /// Check if the snapshot is fresh (less than 10 seconds old)
    pub fn is_fresh(&self) -> bool {
        self.age_seconds() < 10
    }
    
    /// Get a count of total monitored devices
    pub fn total_monitored_devices(&self) -> usize {
        let mut count = 0;
        
        // Count 1 for CPU if present
        if self.cpu_info.is_some() {
            count += 1;
        }
        
        // Count 1 for memory if present
        if self.memory_info.is_some() {
            count += 1;
        }
        
        // Add count of GPUs, network interfaces, and storage devices
        count += self.gpu_info.len() + self.network_info.len() + self.storage_devices.len();
        
        count
    }
}
