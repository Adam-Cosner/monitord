use crate::collectors::gpu::VendorGpuCollector;
use crate::error::CollectionError;
use monitord_protocols::monitord::{GpuDriverInfo, GpuInfo};
use rocm_smi_lib::RocmSmi;
use rocm_smi_lib_sys::bindings::RsmiTemperatureSensor;
use rocm_smi_lib_sys::bindings::{RsmiClkType, RsmiTemperatureMetric};
use tracing::warn;

#[cfg(target_os = "linux")]
pub struct AmdGpuCollector {
    smi: Option<RocmSmi>,
    wgpu_instance: wgpu::Instance,
    devices: Vec<String>,
}

#[cfg(target_os = "linux")]
impl AmdGpuCollector {
    pub fn new() -> Result<Self, CollectionError> {
        if !Self::is_amdgpu_available() {
            return Err(CollectionError::Generic("No AMD GPUs in system".to_owned()));
        }
        let mut smi = RocmSmi::init().ok();
        let wgpu_instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::from_env_or_default());
        let mut collector = Self {
            smi,
            wgpu_instance,
            devices: vec![],
        };

        collector.init()?;

        Ok(collector)
    }

    fn is_amdgpu_available() -> bool {
        // Check sysfs for AMDGPU devices
        {
            if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.join("device/vendor").exists() {
                            if let Ok(vendor) = std::fs::read_to_string(path.join("device/vendor"))
                            {
                                // AMD Vendor ID
                                if vendor.trim() == "0x1002" {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }

        false
    }

    fn collect_rocm(&mut self) -> Result<Vec<GpuInfo>, CollectionError> {
        let mut gpus = Vec::new();
        let driver_info = self.collect_driver_info()?;
        if let Some(rocm) = self.smi.as_mut() {
            let device_count = rocm.get_device_count();
            for i in 0..device_count {
                let (name, vendor) = match rocm.get_device_identifiers(i) {
                    Ok(identifiers) => (
                        match identifiers.brand.clone() {
                            Ok(name) => name,
                            Err(err) => {
                                warn!("Failed to get device name: {:?}", err);
                                continue;
                            }
                        },
                        match identifiers.vendor_name.clone() {
                            Ok(vendor) => vendor,
                            Err(err) => {
                                warn!("Failed to get device vendor: {:?}", err);
                                continue;
                            }
                        },
                    ),
                    Err(err) => {
                        warn!("Failed to get device identifiers: {:?}", err);
                        continue;
                    }
                };

                let (vram_total_bytes, vram_used_bytes) = match rocm.get_device_memory_data(i) {
                    Ok(vram_data) => (vram_data.vram_total, vram_data.vram_used),
                    Err(err) => 
                        (0, 0),
                    
                };

                let core_utilization_percent = match rocm.get_device_busy_percent(i) {
                    Ok(core_utilization_percent) => core_utilization_percent as f64,
                    Err(err) => {
                        warn!("Failed to get {name} busy percent: {err:?}");
                        continue;
                    }
                };

                let temperature_celsius = match rocm.get_device_temperature_metric(
                    i,
                    RsmiTemperatureSensor::RsmiTempTypeEdge,
                    RsmiTemperatureMetric::RsmiTempCurrent,
                ) {
                    Ok(temp) => temp,
                    Err(err) => {
                        warn!("Failed to get {name} temperature: {err:?}");
                        continue;
                    }
                };

                let power_usage_watts = match rocm.get_device_power_data(i) {
                    Ok(power) => Some(power.current_power as f64),
                    Err(err) => None,
                };

                let core_frequency_mhz =
                    match rocm.get_device_frequency(i, RsmiClkType::RsmiClkTypeSoc) {
                        Ok(core_frequency) => Some(core_frequency.current as f64),
                        Err(err) => None,
                    };

                let memory_frequency_mhz =
                    match rocm.get_device_frequency(i, RsmiClkType::RsmiClkTypeMem) {
                        Ok(memory_frequency) => Some(memory_frequency.current as f64),
                        Err(err) => None,
                    };

                let process_info = self.collect_processes(&self);

                gpus.push(GpuInfo {
                    name,
                    vendor,
                    vram_total_bytes,
                    vram_used_bytes,
                    core_utilization_percent,
                    memory_utilization_percent: vram_used_bytes as f64 / vram_total_bytes as f64,
                    temperature_celsius,
                    power_usage_watts,
                    core_frequency_mhz,
                    memory_frequency_mhz,
                    driver_info: Some(driver_info.clone()),
                    encoder_info: None,
                    process_info,
                })
            }
        }

       
        Ok(gpus)
    }

    fn collect_processes(&self) -> Result<Vec<GpuProcessInfo>, CollectionError> {
        todo!()
    }

    fn collect_sysfs(&self) -> Result<Vec<GpuInfo>, CollectionError> {
        todo!()
    }

    fn collect_fallback(&self) -> Result<Vec<GpuInfo>, CollectionError> {
        todo!()
    }

    fn collect_driver_info(&self) -> Result<GpuDriverInfo, CollectionError> {
        let adapters = self
            .wgpu_instance
            .enumerate_adapters(wgpu::Backends::PRIMARY);

        for adapter in adapters {
            if adapter.get_info().vendor == 0x1002 {
                return Ok(GpuDriverInfo {
                    kernel_driver: "amdgpu".to_owned(),
                    userspace_driver: adapter.get_info().driver,
                    driver_version: adapter.get_info().driver_info,
                });
            }
        }

        Err(CollectionError::Generic("No AMD GPU found".to_owned()))
    }
}

#[cfg(target_os = "linux")]
impl VendorGpuCollector for AmdGpuCollector {
    fn init(&mut self) -> Result<(), CollectionError> {
        if let None = self.smi.as_mut() {
            warn!("You do not have ROCm SMI installed, this limits the functionality of the AMD GPU collector.");
            // Manually parse the sysfs for the devices
            if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.join("device/vendor").exists() {
                        if let Ok(vendor) = std::fs::read_to_string(path.join("device/vendor")) {
                            if vendor.trim() == "0x1002" {
                                self.devices.push(path.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
    fn collect(&mut self) -> Result<Vec<GpuInfo>, CollectionError> {
        if let Some(_) = self.smi.as_ref() {
            match self.collect_rocm() {
                Ok(infos) => return Ok(infos),
                Err(err) => warn!("Failed to collect AMD GPU metrics with rocm-smi: {}", err),
            }
        }

        match self.collect_sysfs() {
            Ok(infos) => return Ok(infos),
            Err(err) => warn!("Failed to collect AMD GPU metrics with sysfs: {}", err),
        }

        warn!("Using fallback method for AMD GPU metrics!");
        self.collect_fallback()
    }
}
