use crate::config::GpuCollectorConfig;
use crate::error::{CollectorError, Result};
use crate::traits::Collector;
use monitord_protocols::monitord::{
    GpuDriverInfo, GpuEncoderInfo, GpuInfo, GpuList, GpuProcessInfo,
};
use nvml_wrapper::enum_wrappers::device::{Clock, TemperatureSensor};
use nvml_wrapper::enums::device::UsedGpuMemory;
use nvml_wrapper::Nvml;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, warn};

pub struct GpuCollector {
    config: GpuCollectorConfig,
    nvml: Option<Arc<Nvml>>, // Wrapped in Arc to allow cloning for stream creation
    process_usages: HashMap<u32, (Instant, HashMap<String, u128>)>,
}

impl Collector for GpuCollector {
    type Data = GpuList;
    type Config = GpuCollectorConfig;

    fn new(config: Self::Config) -> Result<Self> {
        debug!("Initializing GPU collector with config: {:?}", config);

        if !config.enabled {
            info!("GPU collector is disabled");
            return Err(CollectorError::ConfigurationError(
                "GPU collector is disabled".into(),
            ));
        }

        // Initialize NVIDIA NVML if requested
        let nvml = if config.collect_nvidia {
            match Nvml::init() {
                Ok(nvml) => {
                    info!("NVIDIA NVML initialized successfully");
                    Some(Arc::new(nvml))
                }
                Err(err) => {
                    warn!("Failed to initialize NVIDIA NVML: {}", err.to_string());
                    None
                }
            }
        } else {
            None
        };

        info!("GPU collector initialized");
        Ok(Self {
            config,
            nvml,
            process_usages: HashMap::new(),
        })
    }

    fn collect(&mut self) -> Result<Self::Data> {
        debug!("Collecting GPU information");

        let mut gpus = Vec::new();

        // Collect NVIDIA GPU information if enabled and available
        if self.config.collect_nvidia {
            if let Some(ref nvml) = self.nvml {
                match self.collect_nvidia_gpus(nvml) {
                    Ok(nvidia_gpus) => gpus.extend(nvidia_gpus),
                    Err(e) => warn!("Failed to collect NVIDIA GPU info: {}", e),
                }
            }
        }

        // Collect AMD GPU information if enabled
        if self.config.collect_amd {
            match self.collect_amd_gpus() {
                Ok(amd_gpus) => gpus.extend(amd_gpus),
                Err(e) => warn!("Failed to collect AMD GPU info: {}", e),
            }
        }

        // Collect Intel GPU information if enabled
        if self.config.collect_intel {
            match self.collect_intel_gpus() {
                Ok(intel_gpus) => gpus.extend(intel_gpus),
                Err(e) => warn!("Failed to collect Intel GPU info: {}", e),
            }
        }

        // If no GPUs were found, return a fallback placeholder
        if gpus.is_empty() {
            // Add a fallback that just shows that no GPUs were detected
            gpus.push(GpuInfo {
                name: "No GPU detected".into(),
                vendor: "Unknown".into(),
                vram_total_bytes: 0,
                vram_used_bytes: 0,
                core_utilization_percent: 0.0,
                memory_utilization_percent: 0.0,
                temperature_celsius: 0.0,
                power_usage_watts: None,
                core_frequency_mhz: None,
                memory_frequency_mhz: None,
                driver_info: None,
                encoder_info: None,
                process_info: Vec::new(),
            });
        }

        debug!("GPU information collected for {} GPU(s)", gpus.len());
        Ok(GpuList { gpus })
    }
}

impl GpuCollector {
    /// Collect information from NVIDIA GPUs
    fn collect_nvidia_gpus(&self, nvml: &Nvml) -> Result<Vec<GpuInfo>> {
        let mut gpu_infos = Vec::new();

        // Get the device count
        let device_count = match nvml.device_count() {
            Ok(count) => count,
            Err(e) => {
                return Err(CollectorError::GpuError(format!(
                    "Failed to get NVIDIA device count: {:?}",
                    e
                )));
            }
        };

        // Collect information for each GPU
        for i in 0..device_count {
            match nvml.device_by_index(i) {
                Ok(device) => {
                    // Basic device information
                    let name = device.name().unwrap_or_else(|e| {
                        warn!("Failed to get GPU name: {:?}", e);
                        "Unknown NVIDIA GPU".to_string()
                    });

                    // Memory information
                    let memory_info = match device.memory_info() {
                        Ok(mem) => (mem.total, mem.used),
                        Err(e) => {
                            warn!("Failed to get GPU memory info: {:?}", e);
                            (0, 0)
                        }
                    };

                    // Utilization information
                    let utilization = match device.utilization_rates() {
                        Ok(util) => (util.gpu, util.memory),
                        Err(e) => {
                            warn!("Failed to get GPU utilization: {:?}", e);
                            (0, 0)
                        }
                    };

                    // Temperature
                    let temperature = match device.temperature(TemperatureSensor::Gpu) {
                        Ok(temp) => temp as f64,
                        Err(e) => {
                            warn!("Failed to get GPU temperature: {:?}", e);
                            0.0
                        }
                    };

                    // Power usage
                    let power = match device.power_usage() {
                        Ok(power) => Some(power as f64 / 1000.0), // Convert mW to W
                        Err(e) => {
                            warn!("Failed to get GPU power usage: {:?}", e);
                            None
                        }
                    };

                    // Clock speeds
                    let gpu_clock = match device.clock_info(Clock::Graphics) {
                        Ok(clock) => Some(clock as f64),
                        Err(e) => {
                            warn!("Failed to get GPU clock: {:?}", e);
                            None
                        }
                    };

                    let memory_clock = match device.clock_info(Clock::Memory) {
                        Ok(clock) => Some(clock as f64),
                        Err(e) => {
                            warn!("Failed to get memory clock: {:?}", e);
                            None
                        }
                    };

                    // Driver information
                    let driver_info = match nvml.sys_driver_version() {
                        Ok(driver) => Some(GpuDriverInfo {
                            kernel_driver: "nvidia".to_string(),
                            userspace_driver: "nvidia".to_string(),
                            driver_version: driver,
                        }),
                        Err(e) => {
                            warn!("Failed to get driver info: {:?}", e);
                            None
                        }
                    };

                    // Encoder utilization
                    let encoder_info = match device.encoder_utilization() {
                        Ok(encoder_util) => match device.decoder_utilization() {
                            Ok(decoder_util) => Some(GpuEncoderInfo {
                                video_encode_utilization_percent: encoder_util.utilization as f64,
                                video_decode_utilization_percent: decoder_util.utilization as f64,
                            }),
                            Err(_) => Some(GpuEncoderInfo {
                                video_encode_utilization_percent: encoder_util.utilization as f64,
                                video_decode_utilization_percent: 0.0,
                            }),
                        },
                        Err(_) => None,
                    };

                    // Process information if enabled
                    let mut process_info = Vec::new();
                    if self.config.collect_processes {
                        match device.running_graphics_processes() {
                            Ok(processes) => {
                                for proc in processes {
                                    // We would need additional libraries to get process names
                                    // For now, just include the PID and memory usage
                                    process_info.push(GpuProcessInfo {
                                        pid: proc.pid,
                                        process_name: format!("PID {}", proc.pid), // Would need additional lookup
                                        gpu_utilization_percent: 0.0, // Not available from NVML this way
                                        vram_bytes: match proc.used_gpu_memory {
                                            UsedGpuMemory::Unavailable => 0,
                                            UsedGpuMemory::Used(used_gpu_memory) => used_gpu_memory,
                                        },
                                        gpu_device_id: Some(i.to_string()),
                                    });
                                }
                            }
                            Err(e) => {
                                warn!("Failed to get GPU processes: {:?}", e);
                            }
                        }
                    }

                    // Create the GpuInfo object
                    let gpu_info = GpuInfo {
                        name,
                        vendor: "NVIDIA".to_string(),
                        vram_total_bytes: memory_info.0,
                        vram_used_bytes: memory_info.1,
                        core_utilization_percent: utilization.0 as f64,
                        memory_utilization_percent: utilization.1 as f64,
                        temperature_celsius: temperature,
                        power_usage_watts: power,
                        core_frequency_mhz: gpu_clock,
                        memory_frequency_mhz: memory_clock,
                        driver_info,
                        encoder_info,
                        process_info,
                    };

                    gpu_infos.push(gpu_info);
                }
                Err(e) => {
                    warn!("Failed to access NVIDIA GPU at index {}: {:?}", i, e);
                }
            }
        }

        Ok(gpu_infos)
    }
}

/// Linux AMD GPU
#[cfg(target_os = "linux")]
impl GpuCollector {
    /// Collect information from AMD GPUs using sysfs interface
    fn collect_amd_gpus(&mut self) -> Result<Vec<GpuInfo>> {
        let mut gpus = Vec::new();

        debug!("Collecting AMD GPU information from sysfs");

        // Detect AMD GPUs through sysfs
        if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
            for entry in entries.flatten() {
                let path = entry.path();

                if path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with("renderD")
                {
                    continue;
                }

                // Skip entries that don't represent physical devices (like renderD*)
                if !path.join("device").exists() {
                    continue;
                }

                // Check if this is an AMD GPU by vendor ID (0x1002)
                if let Ok(vendor) = std::fs::read_to_string(path.join("device/vendor")) {
                    if vendor.trim() == "0x1002" {
                        debug!("Found AMD GPU at {}", path.display());
                        match self.collect_amd_gpu_info(&path) {
                            Ok(gpu_info) => gpus.push(gpu_info),
                            Err(e) => warn!(
                                "Failed to collect info for AMD GPU at {}: {}",
                                path.display(),
                                e
                            ),
                        }
                    }
                }
            }
        }

        if gpus.is_empty() {
            return Err(CollectorError::GpuError(
                "No AMD GPUs found in system".into(),
            ));
        }

        Ok(gpus)
    }

    /// Collect info for a single AMD GPU
    fn collect_amd_gpu_info(&mut self, device_path: &std::path::Path) -> Result<GpuInfo> {
        // Get device name
        let name = self
            .get_amd_device_name(device_path)
            .unwrap_or_else(|_| "Unknown AMD GPU".to_string());

        // Get VRAM information
        let vram_total = self.get_amd_vram_size(device_path).unwrap_or(0);
        let vram_used = self.get_amd_vram_used(device_path).unwrap_or(0);

        // Get utilization, temperature, etc.
        let core_utilization = self.get_amd_gpu_busy(device_path).unwrap_or(0.0);
        let memory_utilization = if vram_total > 0 {
            vram_used as f64 / vram_total as f64 * 100.0
        } else {
            0.0
        };

        // Get driver information
        let driver_info = Some(self.get_amd_driver_info());

        // Get process information if enabled
        let process_info = if self.config.collect_processes {
            self.collect_amd_processes(device_path)?
        } else {
            Vec::new()
        };

        let gpu_info = GpuInfo {
            name,
            vendor: "AMD".to_string(),
            vram_total_bytes: vram_total,
            vram_used_bytes: vram_used,
            core_utilization_percent: core_utilization,
            memory_utilization_percent: memory_utilization,
            temperature_celsius: self.get_amd_temperature(device_path).unwrap_or(0.0),
            power_usage_watts: self.get_amd_power_usage(device_path),
            core_frequency_mhz: self.get_amd_core_frequency(device_path),
            memory_frequency_mhz: self.get_amd_memory_frequency(device_path),
            driver_info,
            encoder_info: None, // AMD doesn't provide encoder info via sysfs
            process_info,
        };

        Ok(gpu_info)
    }

    /// Get AMD GPU name from sysfs
    fn get_amd_device_name(&self, device_path: &std::path::Path) -> Result<String> {
        // First try to read the product name
        if let Ok(product) = std::fs::read_to_string(device_path.join("device/product_name")) {
            return Ok(product.trim().to_string());
        }

        // If product file doesn't exist, try to read the device model ID
        if let Ok(device_id) = std::fs::read_to_string(device_path.join("device/device")) {
            // Convert device ID to a more friendly name
            return Ok(format!("AMD GPU {}", device_id.trim()));
        }

        // Fallback to a generic name with the path
        Ok(format!("AMD GPU ({})", device_path.display()))
    }

    /// Get AMD GPU VRAM total size from sysfs
    fn get_amd_vram_size(&self, device_path: &std::path::Path) -> Result<u64> {
        let mem_info_path = device_path.join("device/mem_info_vram_total");
        if let Ok(content) = std::fs::read_to_string(mem_info_path) {
            if let Ok(bytes) = content.trim().parse::<u64>() {
                return Ok(bytes);
            }
        }
        Err(CollectorError::GpuError(
            "Failed to read VRAM size".to_string(),
        ))
    }

    /// Get AMD GPU VRAM used from sysfs
    fn get_amd_vram_used(&self, device_path: &std::path::Path) -> Result<u64> {
        let mem_info_path = device_path.join("device/mem_info_vram_used");
        if let Ok(content) = std::fs::read_to_string(mem_info_path) {
            if let Ok(bytes) = content.trim().parse::<u64>() {
                return Ok(bytes);
            }
        }
        Err(CollectorError::GpuError(
            "Failed to read VRAM usage".to_string(),
        ))
    }

    /// Get AMD GPU utilization percentage from sysfs
    fn get_amd_gpu_busy(&self, device_path: &std::path::Path) -> Result<f64> {
        let busy_path = device_path.join("device/gpu_busy_percent");
        if let Ok(content) = std::fs::read_to_string(busy_path) {
            if let Ok(percent) = content.trim().parse::<f64>() {
                return Ok(percent);
            }
        }
        Err(CollectorError::GpuError(
            "Failed to read GPU utilization".to_string(),
        ))
    }

    /// Get AMD GPU temperature from sysfs hwmon
    fn get_amd_temperature(&self, device_path: &std::path::Path) -> Result<f64> {
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
        Err(CollectorError::GpuError(
            "Failed to read temperature".to_string(),
        ))
    }

    /// Get AMD GPU power usage from sysfs hwmon
    fn get_amd_power_usage(&self, device_path: &std::path::Path) -> Option<f64> {
        let hwmon_dir = device_path.join("device/hwmon");
        if hwmon_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&hwmon_dir) {
                for entry in entries.flatten() {
                    // Try power_average first
                    let power_path = entry.path().join("power1_average");
                    if power_path.exists() {
                        if let Ok(content) = std::fs::read_to_string(&power_path) {
                            if let Ok(power_uw) = content.trim().parse::<u64>() {
                                // Convert from microwatts to watts
                                return Some(power_uw as f64 / 1_000_000.0);
                            }
                        }
                    }

                    // Try power_input if average is not available
                    let power_input = entry.path().join("power1_input");
                    if power_input.exists() {
                        if let Ok(content) = std::fs::read_to_string(&power_input) {
                            if let Ok(power_uw) = content.trim().parse::<u64>() {
                                // Convert from microwatts to watts
                                return Some(power_uw as f64 / 1_000_000.0);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Get AMD GPU core frequency from sysfs hwmon
    fn get_amd_core_frequency(&self, device_path: &std::path::Path) -> Option<f64> {
        let hwmon_dir = device_path.join("device/hwmon");
        if hwmon_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&hwmon_dir) {
                for entry in entries.flatten() {
                    let freq_path = entry.path().join("freq1_input");
                    if freq_path.exists() {
                        if let Ok(content) = std::fs::read_to_string(&freq_path) {
                            if let Ok(freq_hz) = content.trim().parse::<u64>() {
                                // Convert from Hz to MHz
                                return Some(freq_hz as f64 / 1_000_000.0);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Get AMD GPU memory frequency from sysfs hwmon
    fn get_amd_memory_frequency(&self, device_path: &std::path::Path) -> Option<f64> {
        let hwmon_dir = device_path.join("device/hwmon");
        if hwmon_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&hwmon_dir) {
                for entry in entries.flatten() {
                    let freq_path = entry.path().join("freq2_input");
                    if freq_path.exists() {
                        if let Ok(content) = std::fs::read_to_string(&freq_path) {
                            if let Ok(freq_hz) = content.trim().parse::<u64>() {
                                // Convert from Hz to MHz
                                return Some(freq_hz as f64 / 1_000_000.0);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Get AMD GPU driver information
    fn get_amd_driver_info(&self) -> GpuDriverInfo {
        // Get userspace driver info by running vulkaninfo command
        let userspace_driver = Self::get_amd_userspace_driver();
        let driver_version = Self::get_amd_userspace_driver_version();

        GpuDriverInfo {
            kernel_driver: "amdgpu".to_string(),
            userspace_driver,
            driver_version,
        }
    }

    /// Get AMD userspace driver from vulkaninfo
    fn get_amd_userspace_driver() -> String {
        if let Ok(output) = std::process::Command::new("vulkaninfo").output() {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if line.contains("driverName") && line.contains("AMD") {
                        if line.contains("RADV") {
                            return "Mesa RADV (Vulkan)".to_string();
                        } else if line.contains("AMDVLK") {
                            return "AMDVLK (Vulkan)".to_string();
                        } else if line.contains("AMD Proprietary") {
                            return "AMD Proprietary (Vulkan)".to_string();
                        }
                    }
                }
            }
        }
        "Unknown".to_string()
    }

    /// Get AMD userspace driver version from vulkaninfo
    fn get_amd_userspace_driver_version() -> String {
        if let Ok(output) = std::process::Command::new("vulkaninfo").output() {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if line.contains("driverVersion") {
                        let parts: Vec<&str> = line.split(':').collect();
                        if parts.len() >= 2 {
                            return parts[1].trim().to_string();
                        }
                    }
                }
            }
        }
        "Unknown".to_string()
    }

    /// Collect process information for AMD GPUs
    fn collect_amd_processes(
        &mut self,
        device_path: &std::path::Path,
    ) -> Result<Vec<GpuProcessInfo>> {
        let mut processes = Vec::new();
        let device_id = device_path
            .join("device")
            .read_link()
            .expect("Failed to read symlink for AMD GPU")
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or("unknown".to_string());

        info!("Checking if any processes are using GPU {:?}", device_id);

        // Parse /proc for processes using this GPU
        if let Ok(proc_entries) = std::fs::read_dir("/proc") {
            for proc_entry in proc_entries.flatten() {
                // Check if this is a PID directory
                let pid = match proc_entry.file_name().to_string_lossy().parse::<u32>() {
                    Ok(pid) => pid,
                    Err(_) => continue,
                };

                let path = proc_entry.path();

                // Get process name
                let process_name = std::fs::read_to_string(path.join("comm"))
                    .map(|s| s.trim().to_owned())
                    .unwrap_or_else(|_| format!("PID {}", pid));

                let timestamp = Instant::now();

                // Track GPU usage per device
                let mut accumulated_per_device_usages: HashMap<String, u128> = HashMap::new();
                let mut accumulated_per_device_vram: HashMap<String, u128> = HashMap::new();

                // Parse fdinfo for DRM usage
                if let Ok(fdinfo_dir) = path.join("fdinfo").read_dir() {
                    for fdinfo in fdinfo_dir.flatten() {
                        if let Ok(content) = std::fs::read_to_string(fdinfo.path()) {
                            // Look for DRM device references
                            if let Some(drm_pdev_line) =
                                content.lines().find(|l| l.starts_with("drm-pdev:"))
                            {
                                // Check if this is for our GPU
                                if let Some(drm_pdev) = drm_pdev_line.split_whitespace().nth(1) {
                                    if drm_pdev.contains(device_id.as_str()) {
                                        // Extract GPU engine usage
                                        if let Some(usage) = content
                                            .lines()
                                            .find(|l| l.starts_with("drm-engine-gfx:"))
                                            .and_then(|line| line.split_whitespace().nth(1))
                                            .and_then(|usage| usage.parse::<u128>().ok())
                                        {
                                            // Add to accumulated usage for this device
                                            *accumulated_per_device_usages
                                                .entry(drm_pdev.to_string())
                                                .or_insert(0) += usage;
                                        }
                                        if let Some(vram) = content
                                            .lines()
                                            .find(|l| l.starts_with("drm-memory-vram"))
                                            .and_then(|line| line.split_whitespace().nth(1))
                                            .and_then(|usage| usage.parse::<u128>().ok())
                                        {
                                            // Add to accumulated vram for this device
                                            *accumulated_per_device_vram
                                                .entry(drm_pdev.to_string())
                                                .or_insert(0) += vram;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if !accumulated_per_device_usages.is_empty() {
                    info!(
                        "Accumulated GPU usage for PID {}: {:?}",
                        pid, accumulated_per_device_usages
                    );
                    info!("Process usages: {:?}", self.process_usages);
                    info!("Process usages: {:?}", self.process_usages.get(&pid));
                    // Calculate utilization based on previous usage data
                    if let Some((old_timestamp, old_usages)) = self
                        .process_usages
                        .insert(pid, (timestamp, accumulated_per_device_usages.clone()))
                    {
                        info!("Previous GPU Usage for PID {}: {:?}", pid, old_usages);
                        for (drm_pdev, accumulated_usage) in accumulated_per_device_usages.iter() {
                            let vram_bytes =
                                *accumulated_per_device_vram.get(drm_pdev).unwrap_or(&0u128) as u64;
                            if let Some(previous_usage) = old_usages.get(drm_pdev) {
                                let delta_time = (timestamp - old_timestamp).as_nanos();
                                if delta_time > 0 {
                                    let delta_usages = *accumulated_usage - *previous_usage;
                                    let usage = delta_usages as f64 / delta_time as f64 * 100.0;

                                    info!("Read a GPU Process: {}", process_name);

                                    // Add to process list
                                    processes.push(GpuProcessInfo {
                                        pid,
                                        process_name: process_name.clone(),
                                        gpu_utilization_percent: usage,
                                        vram_bytes,
                                        gpu_device_id: Some(drm_pdev.clone()),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        info!("Processes list: {:?}", processes);

        Ok(processes)
    }
}

impl GpuCollector {
    /// Collect information from Intel GPUs
    fn collect_intel_gpus(&self) -> Result<Vec<GpuInfo>> {
        Ok(vec![])
    }
}
