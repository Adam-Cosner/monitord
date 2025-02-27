/// Process model
use super::Model;
use crate::error::ModelError;
use monitord_protocols::monitord::{GpuProcessInfo as ProtoGpuProcessInfo, ProcessInfo as ProtoProcessInfo};

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub username: String,
    pub state: String,
    pub cpu_usage_percent: f64,
    pub physical_memory_bytes: u64,
    pub virtual_memory_bytes: u64,
    pub disk_read_bytes_per_sec: u64,
    pub disk_write_bytes_per_sec: u64,
    pub threads: u64,
    pub open_files: u64,
    pub start_time_epoch_seconds: i64,
    pub gpu_usage: Option<GpuProcessInfo>,
    
    // Additional fields not in proto
    pub parent_pid: Option<u32>,
    pub cmdline: Option<String>,
    pub cwd: Option<String>,
    pub environment: Vec<(String, String)>,
    pub io_priority: Option<u8>,
    pub nice_value: Option<i8>,
}

#[derive(Debug, Clone)]
pub struct GpuProcessInfo {
    pub pid: u32,
    pub process_name: String,
    pub gpu_utilization_percent: f64,
    pub vram_bytes: u64,
    
    // Additional fields not in proto
    pub gpu_device_id: Option<String>,
}

impl Model for ProcessInfo {
    type ProtoType = ProtoProcessInfo;

    fn into_proto(self) -> Self::ProtoType {
        // Convert environment Vec<(String, String)> to Vec<KeyValuePair>
        let environment = self.environment.into_iter()
            .map(|(key, value)| monitord_protocols::monitord::KeyValuePair { key, value })
            .collect();

        ProtoProcessInfo {
            pid: self.pid,
            name: self.name,
            username: self.username,
            state: self.state,
            cpu_usage_percent: self.cpu_usage_percent,
            physical_memory_bytes: self.physical_memory_bytes,
            virtual_memory_bytes: self.virtual_memory_bytes,
            disk_read_bytes_per_sec: self.disk_read_bytes_per_sec,
            disk_write_bytes_per_sec: self.disk_write_bytes_per_sec,
            threads: self.threads,
            open_files: self.open_files,
            start_time_epoch_seconds: self.start_time_epoch_seconds,
            gpu_usage: self.gpu_usage.map(|gpu| gpu.into_proto()),
            parent_pid: self.parent_pid,
            cmdline: self.cmdline,
            cwd: self.cwd,
            environment,
            io_priority: self.io_priority.map(|p| p as u32),
            nice_value: self.nice_value.map(|n| n as i32),
        }
    }

    fn from_proto(proto: Self::ProtoType) -> Self {
        Self {
            pid: proto.pid,
            name: proto.name,
            username: proto.username,
            state: proto.state,
            cpu_usage_percent: proto.cpu_usage_percent,
            physical_memory_bytes: proto.physical_memory_bytes,
            virtual_memory_bytes: proto.virtual_memory_bytes,
            disk_read_bytes_per_sec: proto.disk_read_bytes_per_sec,
            disk_write_bytes_per_sec: proto.disk_write_bytes_per_sec,
            threads: proto.threads,
            open_files: proto.open_files,
            start_time_epoch_seconds: proto.start_time_epoch_seconds,
            gpu_usage: proto.gpu_usage.map(GpuProcessInfo::from_proto),
            
            parent_pid: proto.parent_pid,
            cmdline: proto.cmdline,
            cwd: proto.cwd,
            environment: proto.environment.into_iter().map(|kv| (kv.key, kv.value)).collect(),
            io_priority: proto.io_priority.map(|p| p as u8),
            nice_value: proto.nice_value.map(|n| n as i8),
        }
    }

    fn validate(&self) -> Result<(), ModelError> {
        if self.cpu_usage_percent < 0.0 {
            return Err(ModelError::Validation(
                "CPU usage percent cannot be negative".to_owned(),
            ));
        }

        if self.cpu_usage_percent > 100.0 * (self.threads as f64) {
            return Err(ModelError::Validation(
                format!(
                    "CPU usage percent ({}) exceeds maximum possible value for {} threads",
                    self.cpu_usage_percent, self.threads
                ),
            ));
        }

        if let Some(gpu_usage) = &self.gpu_usage {
            gpu_usage.validate()?;
        }

        Ok(())
    }
}

impl Model for GpuProcessInfo {
    type ProtoType = ProtoGpuProcessInfo;

    fn into_proto(self) -> Self::ProtoType {
        ProtoGpuProcessInfo {
            pid: self.pid,
            process_name: self.process_name,
            gpu_utilization_percent: self.gpu_utilization_percent,
            vram_bytes: self.vram_bytes,
            gpu_device_id: self.gpu_device_id,
        }
    }

    fn from_proto(proto: Self::ProtoType) -> Self {
        Self {
            pid: proto.pid,
            process_name: proto.process_name,
            gpu_utilization_percent: proto.gpu_utilization_percent,
            vram_bytes: proto.vram_bytes,
            
            gpu_device_id: proto.gpu_device_id,
        }
    }

    fn validate(&self) -> Result<(), ModelError> {
        if self.gpu_utilization_percent < 0.0 || self.gpu_utilization_percent > 100.0 {
            return Err(ModelError::Validation(
                format!(
                    "GPU utilization percent must be between 0 and 100, got {}",
                    self.gpu_utilization_percent
                ),
            ));
        }

        Ok(())
    }
}

// Additional helper methods
impl ProcessInfo {
    /// Calculate total IO bytes per second
    pub fn total_io_bytes_per_sec(&self) -> u64 {
        self.disk_read_bytes_per_sec + self.disk_write_bytes_per_sec
    }
    
    /// Calculate process uptime in seconds
    pub fn uptime_seconds(&self, current_time: i64) -> i64 {
        current_time - self.start_time_epoch_seconds
    }
    
    /// Determine if the process is using a high amount of memory (>2GB)
    pub fn is_memory_intensive(&self) -> bool {
        self.physical_memory_bytes > 2 * 1024 * 1024 * 1024
    }
    
    /// Determine if the process is using a high amount of CPU (>90%)
    pub fn is_cpu_intensive(&self) -> bool {
        self.cpu_usage_percent > 90.0
    }
}
