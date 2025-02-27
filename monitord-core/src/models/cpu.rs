/// CPU Model
use super::Model;
use crate::error::ModelError;
use monitord_protocols::monitord::CoreInfo as ProtoCoreInfo;
use monitord_protocols::monitord::CpuCache as ProtoCpuCache;
use monitord_protocols::protocols::CpuInfo as ProtoCpuInfo;

#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub model_name: String,
    pub physical_cores: u32,
    pub logical_cores: u32,
    pub global_utilization: f64,
    pub cores: Vec<CoreInfo>,
    pub cache: CpuCache,
    pub scaling_governor: Option<String>,

    pub architecture: String,
    pub cpu_flags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CoreInfo {
    pub core_id: u32,
    pub frequency_mhz: f64,
    pub utilization: f64,
    pub temperature: Option<f64>,

    // Additional fields
    pub min_frequency_mhz: Option<f64>,
    pub max_frequency_mhz: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct CpuCache {
    pub l1_data_kb: u32,
    pub l1_instruction_kb: u32,
    pub l2_kb: u32,
    pub l3_kb: u32,
}

impl Model for CpuInfo {
    type ProtoType = ProtoCpuInfo;

    fn into_proto(self) -> Self::ProtoType {
        ProtoCpuInfo {
            model_name: self.model_name,
            physical_cores: self.physical_cores,
            logical_cores: self.logical_cores,
            global_utilization_percent: self.global_utilization,
            core_info: self
                .cores
                .into_iter()
                .map(|core| core.into_proto())
                .collect(),
            cache_info: Some(self.cache.into_proto()),
            scaling_governor: self.scaling_governor,
        }
    }

    fn from_proto(proto: Self::ProtoType) -> Self {
        let cache_info = proto.cache_info.unwrap_or_default();

        Self {
            model_name: proto.model_name,
            physical_cores: proto.physical_cores,
            logical_cores: proto.logical_cores,
            global_utilization: proto.global_utilization_percent,
            cores: proto
                .core_info
                .into_iter()
                .map(CoreInfo::from_proto)
                .collect(),
            cache: CpuCache::from_proto(cache_info),
            scaling_governor: proto.scaling_governor,

            architecture: String::new(),
            cpu_flags: Vec::new(),
        }
    }

    fn validate(&self) -> Result<(), ModelError> {
        // Validate core count
        if self.logical_cores < self.physical_cores {
            return Err(ModelError::Validation(
                "Logical cores cannot be less than physical cores".to_owned(),
            ));
        }

        if self.global_utilization < 0.0 || self.global_utilization > 100.0 {
            return Err(ModelError::Validation(
                "Global utilization must be between 0 and 100".to_owned(),
            ));
        }

        for core in &self.cores {
            core.validate()?;
        }

        Ok(())
    }
}

impl Model for CoreInfo {
    type ProtoType = ProtoCoreInfo;

    fn into_proto(self) -> Self::ProtoType {
        ProtoCoreInfo {
            core_id: self.core_id,
            frequency_mhz: self.frequency_mhz,
            utilization_percent: self.utilization,
            temperature_celsius: self.temperature.unwrap_or_default(),
        }
    }

    fn from_proto(proto: Self::ProtoType) -> Self {
        Self {
            core_id: proto.core_id,
            frequency_mhz: proto.frequency_mhz,
            utilization: proto.utilization_percent,
            temperature: Some(proto.temperature_celsius),
            min_frequency_mhz: None,
            max_frequency_mhz: None,
        }
    }

    fn validate(&self) -> Result<(), ModelError> {
        if self.frequency_mhz < 0.0 {
            return Err(ModelError::Validation(
                "Frequency cannot be negative".to_owned(),
            ));
        }
        if self.utilization < 0.0 || self.utilization > 100.0 {
            return Err(ModelError::Validation(
                "Core utilization must be between 0 and 100".to_owned(),
            ));
        }

        Ok(())
    }
}

impl Model for CpuCache {
    type ProtoType = ProtoCpuCache;

    fn into_proto(self) -> Self::ProtoType {
        ProtoCpuCache {
            l1_data_kb: self.l1_data_kb,
            l1_instruction_kb: self.l1_instruction_kb,
            l2_kb: self.l2_kb,
            l3_kb: self.l3_kb,
        }
    }

    fn from_proto(proto: Self::ProtoType) -> Self {
        Self {
            l1_data_kb: proto.l1_data_kb,
            l1_instruction_kb: proto.l1_instruction_kb,
            l2_kb: proto.l2_kb,
            l3_kb: proto.l3_kb,
        }
    }

    fn validate(&self) -> Result<(), ModelError> {
        Ok(())
    }
}

// Additional methods for the CpuInfo struct
impl CpuInfo {
    /// Calculate frequency across all cores
    pub fn average_frequency(&self) -> f64 {
        if self.cores.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.cores.iter().map(|c| c.frequency_mhz).sum();
        sum / self.cores.len() as f64
    }

    /// Determine if the cpu is under high load (over 80% utilization)
    pub fn is_under_high_load(&self) -> bool {
        self.global_utilization > 80.0
    }

    /// Get the number of cores above a certain temperature threshold
    pub fn cores_above_temperature(&self, threshold_celsius: f64) -> usize {
        self.cores
            .iter()
            .filter(|c| c.temperature.map_or(false, |t| t > threshold_celsius))
            .count()
    }
}
