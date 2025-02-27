/// Memory model definitions
use super::Model;
use crate::error::ModelError;
use monitord_protocols::monitord::{DramInfo as ProtoDramInfo, MemoryInfo as ProtoMemoryInfo};

#[derive(Debug, Clone)]
pub struct MemoryInfo {
    pub total_memory_bytes: u64,
    pub used_memory_bytes: u64,
    pub free_memory_bytes: u64,
    pub available_memory_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub swap_free_bytes: u64,
    pub dram_info: Option<DramInfo>,
    
    // Additional fields not in proto
    pub cached_memory_bytes: u64,
    pub shared_memory_bytes: u64,
    pub memory_load_percent: f64,
}

#[derive(Debug, Clone)]
pub struct DramInfo {
    pub frequency_mhz: f64,
    pub memory_type: String,
    pub slots_total: u32,
    pub slots_used: u32,
    
    // Additional fields not in proto
    pub manufacturer: Option<String>,
    pub part_number: Option<String>,
}

impl Model for MemoryInfo {
    type ProtoType = ProtoMemoryInfo;

    fn into_proto(self) -> Self::ProtoType {
        ProtoMemoryInfo {
            total_memory_bytes: self.total_memory_bytes,
            used_memory_bytes: self.used_memory_bytes,
            free_memory_bytes: self.free_memory_bytes,
            available_memory_bytes: self.available_memory_bytes,
            swap_total_bytes: self.swap_total_bytes,
            swap_used_bytes: self.swap_used_bytes,
            swap_free_bytes: self.swap_free_bytes,
            dram_info: self.dram_info.map(|d| d.into_proto()),
            cached_memory_bytes: self.cached_memory_bytes,
            shared_memory_bytes: self.shared_memory_bytes,
            memory_load_percent: self.memory_load_percent,
        }
    }

    fn from_proto(proto: Self::ProtoType) -> Self {
        Self {
            total_memory_bytes: proto.total_memory_bytes,
            used_memory_bytes: proto.used_memory_bytes,
            free_memory_bytes: proto.free_memory_bytes,
            available_memory_bytes: proto.available_memory_bytes,
            swap_total_bytes: proto.swap_total_bytes,
            swap_used_bytes: proto.swap_used_bytes,
            swap_free_bytes: proto.swap_free_bytes,
            dram_info: proto.dram_info.map(DramInfo::from_proto),
            
            cached_memory_bytes: proto.cached_memory_bytes,
            shared_memory_bytes: proto.shared_memory_bytes,
            memory_load_percent: proto.memory_load_percent,
        }
    }

    fn validate(&self) -> Result<(), ModelError> {
        // Validate memory values
        if self.used_memory_bytes > self.total_memory_bytes {
            return Err(ModelError::Validation(
                "Used memory cannot exceed total memory".to_owned(),
            ));
        }

        if self.free_memory_bytes > self.total_memory_bytes {
            return Err(ModelError::Validation(
                "Free memory cannot exceed total memory".to_owned(),
            ));
        }

        if self.swap_used_bytes > self.swap_total_bytes {
            return Err(ModelError::Validation(
                "Used swap cannot exceed total swap".to_owned(),
            ));
        }

        if let Some(dram_info) = &self.dram_info {
            dram_info.validate()?;
        }

        Ok(())
    }
}

impl Model for DramInfo {
    type ProtoType = ProtoDramInfo;

    fn into_proto(self) -> Self::ProtoType {
        ProtoDramInfo {
            frequency_mhz: self.frequency_mhz,
            memory_type: self.memory_type,
            slots_total: self.slots_total,
            slots_used: self.slots_used,
            manufacturer: self.manufacturer,
            part_number: self.part_number,
        }
    }

    fn from_proto(proto: Self::ProtoType) -> Self {
        Self {
            frequency_mhz: proto.frequency_mhz,
            memory_type: proto.memory_type,
            slots_total: proto.slots_total,
            slots_used: proto.slots_used,
            
            manufacturer: proto.manufacturer,
            part_number: proto.part_number,
        }
    }

    fn validate(&self) -> Result<(), ModelError> {
        if self.frequency_mhz < 0.0 {
            return Err(ModelError::Validation(
                "Memory frequency cannot be negative".to_owned(),
            ));
        }

        if self.slots_used > self.slots_total {
            return Err(ModelError::Validation(
                "Used memory slots cannot exceed total slots".to_owned(),
            ));
        }

        Ok(())
    }
}

// Additional helper methods for MemoryInfo
impl MemoryInfo {
    /// Calculate memory utilization percentage
    pub fn memory_utilization_percent(&self) -> f64 {
        if self.total_memory_bytes == 0 {
            return 0.0;
        }
        
        (self.used_memory_bytes as f64 / self.total_memory_bytes as f64) * 100.0
    }
    
    /// Calculate swap utilization percentage
    pub fn swap_utilization_percent(&self) -> f64 {
        if self.swap_total_bytes == 0 {
            return 0.0;
        }
        
        (self.swap_used_bytes as f64 / self.swap_total_bytes as f64) * 100.0
    }
    
    /// Determine if memory pressure is high (over 90% used)
    pub fn is_under_memory_pressure(&self) -> bool {
        self.memory_utilization_percent() > 90.0
    }
}
