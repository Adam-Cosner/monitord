use crate::collectors::gpu::VendorGpuCollector;
use crate::error::CollectionError;
use monitord_protocols::monitord::{GpuDriverInfo, GpuInfo, GpuProcessInfo};

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

        false
    }

    fn collect_processes(&self) -> Result<Vec<GpuProcessInfo>, CollectionError> {
        // todo
        Ok(vec![])
    }

    fn get_amd_device_name(device_path: &std::path::Path) -> Result<String, CollectionError> {

        // If product file doesn't exist, try to read the device model ID
        if let Ok(device_id) = std::fs::read_to_string(device_path.join("device/device")) {
            // Convert device ID to a more friendly name using a lookup table

            return Ok(format!("AMD GPU {}", device_id.trim()));
        }

        // Fallback to a generic name with the path
        Ok(format!("AMD GPU ({})", device_path.display()))
    }

    fn get_vram_size(device_path: &std::path::Path) -> Result<u64, CollectionError> {
        // For AMD GPUs, VRAM info is typically in:
        // /sys/class/drm/card0/device/mem_info_vram_total
        let mem_info_path = device_path.join("device/mem_info_vram_total");
        if let Ok(content) = std::fs::read_to_string(mem_info_path) {
            if let Ok(bytes) = content.trim().parse::<u64>() {
                return Ok(bytes);
            }
        }
        Err(CollectionError::Generic("Failed to read VRAM size".to_string()))
    }

    fn get_vram_used(device_path: &std::path::Path) -> Result<u64, CollectionError> {
        // For AMD GPUs, used VRAM is typically in:
        // /sys/class/drm/card0/device/mem_info_vram_used
        let mem_info_path = device_path.join("device/mem_info_vram_used");
        if let Ok(content) = std::fs::read_to_string(mem_info_path) {
            if let Ok(bytes) = content.trim().parse::<u64>() {
                return Ok(bytes);
            }
        }
        Err(CollectionError::Generic("Failed to read VRAM usage".to_string()))
    }

    fn get_gpu_busy(device_path: &std::path::Path) -> Result<f64, CollectionError> {
        // GPU busy percent can be found in:
        // /sys/class/drm/card0/device/gpu_busy_percent
        let busy_path = device_path.join("device/gpu_busy_percent");
        if let Ok(content) = std::fs::read_to_string(busy_path) {
            if let Ok(percent) = content.trim().parse::<f64>() {
                return Ok(percent);
            }
        }
        Err(CollectionError::Generic("Failed to read GPU utilization".to_string()))
    }

    fn get_temperature(device_path: &std::path::Path) -> Result<f64, CollectionError> {
        // Temperature is often found in hwmon subdirectories
        // First, find the hwmon directory
        let hwmon_dir = device_path.join("device/hwmon");
        if hwmon_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&hwmon_dir) {
                for entry in entries.flatten() {
                    // Look for temp1_input which typically has the GPU temperature in millidegrees Celsius
                    let temp_path = entry.path().join("temp1_input");
                    if temp_path.exists() {
                        if let Ok(content) = std::fs::read_to_string(&temp_path) {
                            if let Ok(temp) = content.trim().parse::<u32>() {
                                // Convert from millidegrees to degrees
                                return Ok(temp as f64 / 1000.0);
                            }
                        }
                    }
                }
            }
        }
        Err(CollectionError::Generic("Failed to read temperature".to_string()))
    }

    fn get_power_usage(device_path: &std::path::Path) -> Option<f64> {
        // todo
        None
    }

    fn get_core_frequency(device_path: &std::path::Path) -> Option<f64> {
        // todo
        None
    }

    fn get_memory_frequency(device_path: &std::path::Path) -> Option<f64> {
        // todo
        None
    }

    fn get_userspace_driver() -> String {
        // todo
        "".to_owned()
    }

    fn get_userspace_driver_version() -> String {
        // todo
        "".to_owned()
    }

    fn get_driver_info(&self) -> GpuDriverInfo {
        GpuDriverInfo {
            kernel_driver: "amdgpu".to_owned(),
            userspace_driver: Self::get_userspace_driver(),
            driver_version: Self::get_userspace_driver_version(),
        }
    }

    fn collect_sysfs(&self) -> Result<Vec<GpuInfo>, CollectionError> {
        let mut gpus = Vec::new();

        // Check each directory in /sys/class/drm for AMD GPUs
        if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
            for entry in entries.flatten() {
                let path = entry.path();

                // Skip entries that don't represent physical devices (like renderD*)
                if !path.join("device").exists() {
                    continue;
                }

                // Check if this is an AMD GPU
                if let Ok(vendor) = std::fs::read_to_string(path.join("device/vendor")) {
                    if vendor.trim() == "0x1002" {
                        // This is an AMD GPU
                        let name = Self::get_amd_device_name(&path)
                            .unwrap_or_else(|_| "Unknown AMD GPU".to_string());

                        // Get VRAM information
                        let vram_total = Self::get_vram_size(&path).unwrap_or(0);

                        // Read other metrics like core and memory utilization,
                        // temperatures, frequencies, etc.

                        gpus.push(GpuInfo {
                            name,
                            vendor: "AMD".to_string(),
                            vram_total_bytes: vram_total,
                            vram_used_bytes: Self::get_vram_used(&path).unwrap_or(0),
                            core_utilization_percent: Self::get_gpu_busy(&path).unwrap_or(0.0),
                            memory_utilization_percent: if vram_total > 0 {
                                Self::get_vram_used(&path).unwrap_or(0) as f64 / vram_total as f64 * 100.0
                            } else {
                                0.0
                            },
                            temperature_celsius: Self::get_temperature(&path).unwrap_or(0.0),
                            power_usage_watts: Self::get_power_usage(&path),
                            core_frequency_mhz: Self::get_core_frequency(&path),
                            memory_frequency_mhz: Self::get_memory_frequency(&path),
                            driver_info: Some(self.get_driver_info()),
                            encoder_info: None, // AMD GPU doesn't support reporting encoder info
                            process_info: self.collect_processes()?, // This would need another method to populate
                        });
                    }
                }
            }
        }

        if gpus.is_empty() {
            return Err(CollectionError::Generic("No AMD GPUs found using sysfs".to_string()));
        }

        Ok(gpus)
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
        self.collect_sysfs()
    }
}
