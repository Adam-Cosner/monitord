use crate::error::CollectionError;
use monitord_protocols::monitord::{GpuDriverInfo, GpuInfo};
use nvml_wrapper::enum_wrappers::device::{Clock, TemperatureSensor};
use nvml_wrapper::Nvml;

pub struct NvidiaGpuCollector {
    nvml: Nvml,
}

impl NvidiaGpuCollector {
    pub fn new() -> Result<Self, CollectionError> {
        let nvml = Nvml::init().map_err(|_| CollectionError::Disabled)?;
        Ok(Self { nvml })
    }
}

impl super::VendorGpuCollector for NvidiaGpuCollector {
    fn init(&mut self) -> Result<(), CollectionError> {
        Ok(())
    }

    fn collect(&mut self) -> Result<Vec<GpuInfo>, CollectionError> {
        let mut gpu_infos: Vec<GpuInfo> = Vec::new();
        let device_count = self
            .nvml
            .device_count()
            .map_err(|e| CollectionError::Generic(e.to_string()))?;
        for i in 0..device_count {
            let device = self
                .nvml
                .device_by_index(i)
                .map_err(|e| CollectionError::Generic(e.to_string()))?;
            gpu_infos.push(GpuInfo {
                name: device.name().unwrap_or_default(),
                vendor: "NVIDIA".to_string(),
                vram_total_bytes: device
                    .memory_info()
                    .map(|meminfo| meminfo.total)
                    .unwrap_or(0),
                vram_used_bytes: device
                    .memory_info()
                    .map(|meminfo| meminfo.used)
                    .unwrap_or(0),
                core_utilization_percent: device
                    .utilization_rates()
                    .map(|util| util.gpu as f64)
                    .unwrap_or(0.0),
                memory_utilization_percent: device
                    .utilization_rates()
                    .map(|util| util.memory as f64)
                    .unwrap_or(0.0),
                temperature_celsius: device
                    .temperature(TemperatureSensor::Gpu)
                    .map(|temp| temp as f64)
                    .unwrap_or(0.0),
                power_usage_watts: device.power_usage().map(|usage| usage as f64).ok(),
                core_frequency_mhz: device
                    .clock_info(Clock::Graphics)
                    .map(|clock| clock as f64)
                    .ok(),
                memory_frequency_mhz: device
                    .clock_info(Clock::Memory)
                    .map(|clock| clock as f64)
                    .ok(),
                driver_info: Some(GpuDriverInfo {
                    kernel_driver: "nvidia".to_owned(),
                    userspace_driver: "nvidia".to_owned(),
                    driver_version: self.nvml.sys_driver_version().unwrap_or("".to_owned()),
                }),
                encoder_info: None,
                process_info: vec![],
            })
        }
        Ok(gpu_infos)
    }
}
