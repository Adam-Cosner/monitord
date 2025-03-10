use crate::collectors::gpu::VendorGpuCollector;
use crate::error::CollectionError;
use monitord_protocols::monitord::{GpuDriverInfo, GpuInfo, GpuProcessInfo};
use tracing::warn;

#[cfg(target_os = "linux")]
pub struct AmdGpuCollector {
    wgpu_instance: wgpu::Instance,
    devices: Vec<String>,
}

#[cfg(target_os = "linux")]
impl AmdGpuCollector {
    pub fn new() -> Result<Self, CollectionError> {
        if !Self::is_amdgpu_available() {
            return Err(CollectionError::Generic("No AMD GPUs in system".to_owned()));
        }
        let wgpu_instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::from_env_or_default());
        let mut collector = Self {
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

        Ok(())
    }
    fn collect(&mut self) -> Result<Vec<GpuInfo>, CollectionError> {
        match self.collect_sysfs() {
            Ok(infos) => return Ok(infos),
            Err(err) => warn!("Failed to collect AMD GPU metrics with sysfs: {}", err),
        }

        warn!("Using fallback method for AMD GPU metrics!");
        self.collect_fallback()
    }
}
