use chrono::{DateTime, Utc};

// Define the data structures for collector output

#[derive(Debug, Clone)]
pub struct CpuData {
    pub timestamp: DateTime<Utc>,
    // Add actual CPU data fields
    pub utilization: f64,
    // Additional CPU metrics
}

#[derive(Debug, Clone)]
pub struct MemoryData {
    pub timestamp: DateTime<Utc>,
    // Add actual memory data fields
    pub total_bytes: u64,
    pub used_bytes: u64,
    // Additional memory metrics
}

#[derive(Debug, Clone)]
pub struct GpuData {
    pub timestamp: DateTime<Utc>,
    // Add actual GPU data fields
    pub devices: Vec<GpuDevice>,
}

#[derive(Debug, Clone)]
pub struct GpuDevice {
    pub name: String,
    pub utilization: f64,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    // Additional GPU metrics
}

#[derive(Debug, Clone)]
pub struct NetworkData {
    pub timestamp: DateTime<Utc>,
    // Add actual network data fields
    pub interfaces: Vec<NetworkInterface>,
}

#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    // Additional network metrics
}

#[derive(Debug, Clone)]
pub struct ProcessData {
    pub timestamp: DateTime<Utc>,
    // Add actual process data fields
    pub processes: Vec<ProcessInfo>,
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f64,
    pub memory_usage: u64,
    // Additional process metrics
}

#[derive(Debug, Clone)]
pub struct StorageData {
    pub timestamp: DateTime<Utc>,
    // Add actual storage data fields
    pub devices: Vec<StorageDevice>,
}

#[derive(Debug, Clone)]
pub struct StorageDevice {
    pub name: String,
    pub mount_point: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    // Additional storage metrics
}

#[derive(Debug, Clone)]
pub struct SystemData {
    pub timestamp: DateTime<Utc>,
    // Add actual system data fields
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub uptime_seconds: u64,
    // Additional system metrics
}