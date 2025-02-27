/// Storage device model
use super::Model;
use crate::error::ModelError;
use monitord_protocols::monitord::StorageInfo as ProtoStorageInfo;

#[derive(Debug, Clone)]
pub struct StorageInfo {
    pub device_name: String,
    pub device_type: String,
    pub model: String,
    pub filesystem_type: String,
    pub mount_point: String,
    pub total_space_bytes: u64,
    pub available_space_bytes: u64,
    pub read_bytes_per_sec: u64,
    pub write_bytes_per_sec: u64,
    pub io_time_ms: u64,
    pub temperature_celsius: Option<f64>,
    pub lifetime_writes_bytes: Option<u64>,
    
    // Additional fields not in proto
    pub serial_number: Option<String>,
    pub partition_label: Option<String>,
    pub used_space_bytes: u64,
    pub smart_data: Option<SmartData>,
}

#[derive(Debug, Clone)]
pub struct SmartData {
    pub health_status: String,
    pub power_on_hours: Option<u64>,
    pub power_cycle_count: Option<u32>,
    pub reallocated_sectors: Option<u32>,
    pub remaining_life_percent: Option<u8>,
}

impl Model for StorageInfo {
    type ProtoType = ProtoStorageInfo;

    fn into_proto(self) -> Self::ProtoType {
        ProtoStorageInfo {
            device_name: self.device_name,
            device_type: self.device_type,
            model: self.model,
            filesystem_type: self.filesystem_type,
            mount_point: self.mount_point,
            total_space_bytes: self.total_space_bytes,
            available_space_bytes: self.available_space_bytes,
            read_bytes_per_sec: self.read_bytes_per_sec,
            write_bytes_per_sec: self.write_bytes_per_sec,
            io_time_ms: self.io_time_ms,
            temperature_celsius: self.temperature_celsius,
            lifetime_writes_bytes: self.lifetime_writes_bytes,
            serial_number: self.serial_number,
            partition_label: self.partition_label,
            used_space_bytes: self.used_space_bytes,
            smart_data: self.smart_data.map(|s| s.into_proto()),
        }
    }

    fn from_proto(proto: Self::ProtoType) -> Self {
        Self {
            device_name: proto.device_name,
            device_type: proto.device_type,
            model: proto.model,
            filesystem_type: proto.filesystem_type,
            mount_point: proto.mount_point,
            total_space_bytes: proto.total_space_bytes,
            available_space_bytes: proto.available_space_bytes,
            read_bytes_per_sec: proto.read_bytes_per_sec,
            write_bytes_per_sec: proto.write_bytes_per_sec,
            io_time_ms: proto.io_time_ms,
            temperature_celsius: proto.temperature_celsius,
            lifetime_writes_bytes: proto.lifetime_writes_bytes,
            
            serial_number: proto.serial_number,
            partition_label: proto.partition_label,
            used_space_bytes: proto.used_space_bytes,
            smart_data: proto.smart_data.map(SmartData::from_proto),
        }
    }

    fn validate(&self) -> Result<(), ModelError> {
        if self.available_space_bytes > self.total_space_bytes {
            return Err(ModelError::Validation(
                "Available space cannot exceed total space".to_owned(),
            ));
        }

        if let Some(temp) = self.temperature_celsius {
            if temp < -20.0 || temp > 120.0 {
                return Err(ModelError::OutOfRange {
                    field: "temperature_celsius".to_owned(),
                    value: temp.to_string(),
                    min: "-20.0".to_owned(),
                    max: "120.0".to_owned(),
                });
            }
        }

        Ok(())
    }
}

// Implement the Model trait for SmartData
impl Model for SmartData {
    type ProtoType = monitord_protocols::monitord::SmartData;

    fn into_proto(self) -> Self::ProtoType {
        monitord_protocols::monitord::SmartData {
            health_status: self.health_status,
            power_on_hours: self.power_on_hours,
            power_cycle_count: self.power_cycle_count,
            reallocated_sectors: self.reallocated_sectors,
            remaining_life_percent: self.remaining_life_percent.map(|p| p as u32),
        }
    }

    fn from_proto(proto: Self::ProtoType) -> Self {
        Self {
            health_status: proto.health_status,
            power_on_hours: proto.power_on_hours,
            power_cycle_count: proto.power_cycle_count,
            reallocated_sectors: proto.reallocated_sectors,
            remaining_life_percent: proto.remaining_life_percent.map(|p| p as u8),
        }
    }

    fn validate(&self) -> Result<(), ModelError> {
        if let Some(percent) = self.remaining_life_percent {
            if percent > 100 {
                return Err(ModelError::Validation(
                    "Remaining life percentage cannot exceed 100".to_owned(),
                ));
            }
        }
        Ok(())
    }
}

// Additional helper methods
impl StorageInfo {
    /// Calculate the percentage of used space
    pub fn usage_percent(&self) -> f64 {
        if self.total_space_bytes == 0 {
            return 0.0;
        }
        
        let used = self.total_space_bytes - self.available_space_bytes;
        (used as f64 / self.total_space_bytes as f64) * 100.0
    }
    
    /// Calculate total IO bytes per second
    pub fn total_io_bytes_per_sec(&self) -> u64 {
        self.read_bytes_per_sec + self.write_bytes_per_sec
    }
    
    /// Check if storage is almost full (>90% used)
    pub fn is_almost_full(&self) -> bool {
        self.usage_percent() > 90.0
    }
    
    /// Check if this is an external device based on the mount point
    pub fn is_external(&self) -> bool {
        self.mount_point.contains("/media/") || self.mount_point.contains("/mnt/")
    }
    
    /// Check if this is a solid state drive
    pub fn is_ssd(&self) -> bool {
        self.device_type.to_lowercase().contains("ssd") ||
        self.device_type.to_lowercase().contains("nvme")
    }
}
