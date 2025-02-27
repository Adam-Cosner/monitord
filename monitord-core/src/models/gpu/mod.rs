/// GPU model traits and common functionality
use super::Model;
use crate::error::ModelError;
use monitord_protocols::monitord::{
    GpuDriverInfo as ProtoGpuDriverInfo,
    GpuEncoderInfo as ProtoGpuEncoderInfo,
    GpuInfo as ProtoGpuInfo,
};

pub mod amd;
pub mod intel;
pub mod nvidia;

#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: String,
    pub vram_total_bytes: u64,
    pub vram_used_bytes: u64,
    pub core_utilization_percent: f64,
    pub memory_utilization_percent: f64,
    pub temperature_celsius: f64,
    pub power_usage_watts: Option<f64>,
    pub core_frequency_mhz: Option<f64>,
    pub memory_frequency_mhz: Option<f64>,
    pub driver_info: Option<GpuDriverInfo>,
    pub encoder_info: Option<GpuEncoderInfo>,
    
    // Additional fields not in proto
    pub device_id: String,
    pub pci_address: Option<String>,
    pub max_power_watts: Option<f64>,
    pub architecture: Option<String>,
    pub compute_capability: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GpuDriverInfo {
    pub kernel_driver: String,
    pub userspace_driver: String,
    pub driver_version: String,
    
    // Additional fields not in proto
    pub cuda_version: Option<String>,
    pub opencl_version: Option<String>,
    pub vulkan_version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GpuEncoderInfo {
    pub video_encode_utilization_percent: f64,
    pub video_decode_utilization_percent: f64,
    
    // Additional fields not in proto
    pub encoder_engines: Option<u32>,
    pub decoder_engines: Option<u32>,
    pub supported_codecs: Vec<String>,
}

impl Model for GpuInfo {
    type ProtoType = ProtoGpuInfo;

    fn into_proto(self) -> Self::ProtoType {
        ProtoGpuInfo {
            name: self.name,
            vendor: self.vendor,
            vram_total_bytes: self.vram_total_bytes,
            vram_used_bytes: self.vram_used_bytes,
            core_utilization_percent: self.core_utilization_percent,
            memory_utilization_percent: self.memory_utilization_percent,
            temperature_celsius: self.temperature_celsius,
            power_usage_watts: self.power_usage_watts,
            core_frequency_mhz: self.core_frequency_mhz,
            memory_frequency_mhz: self.memory_frequency_mhz,
            driver_info: self.driver_info.map(|d| d.into_proto()),
            encoder_info: self.encoder_info.map(|e| e.into_proto()),
            process_info: Vec::new(), // Not implemented here as it's collected separately
        }
    }

    fn from_proto(proto: Self::ProtoType) -> Self {
        Self {
            name: proto.name,
            vendor: proto.vendor,
            vram_total_bytes: proto.vram_total_bytes,
            vram_used_bytes: proto.vram_used_bytes,
            core_utilization_percent: proto.core_utilization_percent,
            memory_utilization_percent: proto.memory_utilization_percent,
            temperature_celsius: proto.temperature_celsius,
            power_usage_watts: proto.power_usage_watts,
            core_frequency_mhz: proto.core_frequency_mhz,
            memory_frequency_mhz: proto.memory_frequency_mhz,
            driver_info: proto.driver_info.map(GpuDriverInfo::from_proto),
            encoder_info: proto.encoder_info.map(GpuEncoderInfo::from_proto),
            
            // Initialize additional fields
            device_id: String::new(),
            pci_address: None,
            max_power_watts: None,
            architecture: None,
            compute_capability: None,
        }
    }

    fn validate(&self) -> Result<(), ModelError> {
        // Validate utilization percentages
        if self.core_utilization_percent < 0.0 || self.core_utilization_percent > 100.0 {
            return Err(ModelError::Validation(
                format!(
                    "Core utilization percent must be between 0 and 100, got {}",
                    self.core_utilization_percent
                ),
            ));
        }

        if self.memory_utilization_percent < 0.0 || self.memory_utilization_percent > 100.0 {
            return Err(ModelError::Validation(
                format!(
                    "Memory utilization percent must be between 0 and 100, got {}",
                    self.memory_utilization_percent
                ),
            ));
        }

        // Validate VRAM usage
        if self.vram_used_bytes > self.vram_total_bytes {
            return Err(ModelError::Validation(
                "Used VRAM cannot exceed total VRAM".to_owned(),
            ));
        }

        // Validate temperature - GPUs can operate in a wide range but let's set reasonable limits
        if self.temperature_celsius < -20.0 || self.temperature_celsius > 120.0 {
            return Err(ModelError::OutOfRange {
                field: "temperature_celsius".to_owned(),
                value: self.temperature_celsius.to_string(),
                min: "-20.0".to_owned(),
                max: "120.0".to_owned(),
            });
        }

        // Validate power usage if present
        if let Some(power) = self.power_usage_watts {
            if power < 0.0 {
                return Err(ModelError::Validation(
                    "Power usage cannot be negative".to_owned(),
                ));
            }
        }

        // Validate driver info if present
        if let Some(driver) = &self.driver_info {
            driver.validate()?;
        }

        // Validate encoder info if present
        if let Some(encoder) = &self.encoder_info {
            encoder.validate()?;
        }

        Ok(())
    }
}

impl Model for GpuDriverInfo {
    type ProtoType = ProtoGpuDriverInfo;

    fn into_proto(self) -> Self::ProtoType {
        ProtoGpuDriverInfo {
            kernel_driver: self.kernel_driver,
            userspace_driver: self.userspace_driver,
            driver_version: self.driver_version,
        }
    }

    fn from_proto(proto: Self::ProtoType) -> Self {
        Self {
            kernel_driver: proto.kernel_driver,
            userspace_driver: proto.userspace_driver,
            driver_version: proto.driver_version,
            
            // Initialize additional fields
            cuda_version: None,
            opencl_version: None,
            vulkan_version: None,
        }
    }

    fn validate(&self) -> Result<(), ModelError> {
        // Basic validation - ensure fields aren't empty
        if self.kernel_driver.is_empty() {
            return Err(ModelError::MissingField("kernel_driver".to_owned()));
        }

        if self.driver_version.is_empty() {
            return Err(ModelError::MissingField("driver_version".to_owned()));
        }

        Ok(())
    }
}

impl Model for GpuEncoderInfo {
    type ProtoType = ProtoGpuEncoderInfo;

    fn into_proto(self) -> Self::ProtoType {
        ProtoGpuEncoderInfo {
            video_encode_utilization_percent: self.video_encode_utilization_percent,
            video_decode_utilization_percent: self.video_decode_utilization_percent,
        }
    }

    fn from_proto(proto: Self::ProtoType) -> Self {
        Self {
            video_encode_utilization_percent: proto.video_encode_utilization_percent,
            video_decode_utilization_percent: proto.video_decode_utilization_percent,
            
            // Initialize additional fields
            encoder_engines: None,
            decoder_engines: None,
            supported_codecs: Vec::new(),
        }
    }

    fn validate(&self) -> Result<(), ModelError> {
        // Validate utilization percentages
        if self.video_encode_utilization_percent < 0.0 || self.video_encode_utilization_percent > 100.0 {
            return Err(ModelError::Validation(
                format!(
                    "Video encode utilization percent must be between 0 and 100, got {}",
                    self.video_encode_utilization_percent
                ),
            ));
        }

        if self.video_decode_utilization_percent < 0.0 || self.video_decode_utilization_percent > 100.0 {
            return Err(ModelError::Validation(
                format!(
                    "Video decode utilization percent must be between 0 and 100, got {}",
                    self.video_decode_utilization_percent
                ),
            ));
        }

        Ok(())
    }
}

// Additional helper methods
impl GpuInfo {
    /// Calculate VRAM utilization percentage
    pub fn vram_utilization_percent(&self) -> f64 {
        if self.vram_total_bytes == 0 {
            return 0.0;
        }
        
        (self.vram_used_bytes as f64 / self.vram_total_bytes as f64) * 100.0
    }
    
    /// Check if temperature is high (> 80°C)
    pub fn is_temperature_high(&self) -> bool {
        self.temperature_celsius > 80.0
    }
    
    /// Check if this is a discrete GPU (as opposed to integrated)
    pub fn is_discrete(&self) -> bool {
        self.vendor.to_lowercase() == "nvidia" || 
        self.vendor.to_lowercase() == "amd" && self.vram_total_bytes > 1024 * 1024 * 1024
    }
    
    /// Determine the GPU generation/family based on name and vendor
    pub fn get_gpu_family(&self) -> &'static str {
        let name_lower = self.name.to_lowercase();
        let vendor_lower = self.vendor.to_lowercase();
        
        match vendor_lower.as_str() {
            "nvidia" => {
                if name_lower.contains("rtx 40") {
                    "NVIDIA Ada Lovelace"
                } else if name_lower.contains("rtx 30") {
                    "NVIDIA Ampere"
                } else if name_lower.contains("rtx 20") {
                    "NVIDIA Turing"
                } else if name_lower.contains("gtx 16") {
                    "NVIDIA Turing" 
                } else if name_lower.contains("gtx 10") {
                    "NVIDIA Pascal"
                } else if name_lower.contains("tesla") {
                    "NVIDIA Tesla"
                } else {
                    "NVIDIA Other"
                }
            },
            "amd" => {
                if name_lower.contains("radeon rx 7") {
                    "AMD RDNA 3"
                } else if name_lower.contains("radeon rx 6") {
                    "AMD RDNA 2"
                } else if name_lower.contains("radeon rx 5") {
                    "AMD RDNA"
                } else if name_lower.contains("vega") {
                    "AMD Vega"
                } else {
                    "AMD Other"
                }
            },
            "intel" => {
                if name_lower.contains("arc") {
                    "Intel Arc"
                } else if name_lower.contains("xe") {
                    "Intel Xe"
                } else if name_lower.contains("iris") {
                    "Intel Iris"
                } else {
                    "Intel Integrated"
                }
            },
            _ => "Unknown"
        }
    }
}
