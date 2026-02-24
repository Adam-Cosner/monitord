/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! GPU Metric Collection
//!
//! # Example
//!
//! ```
//! let collector = monitord_metrics::memory::Collector::new();
//! let result = collector.collect().unwrap();
//! assert!(!result.is_empty());
//! ```

mod amd;
mod intel;
mod nvidia;

use anyhow::Context;
use std::path::PathBuf;

#[doc(inline)]
pub use crate::metrics::Gpu as Snapshot;
#[doc(inline)]
pub use crate::metrics::GpuProcess as Process;

pub struct Collector {
    gpus: Vec<Gpu>,
    // Fields tba
    nvidia: nvidia::Collector,
    intel: intel::Collector,
    amd: amd::Collector,
}

struct Gpu {
    path: PathBuf,
    vendor: GpuVendor,
    opengl_driver: String,
    vulkan_driver: String,
}

enum GpuVendor {
    Intel,
    Nvidia,
    Amd,
    // TODO: Add support for smaller vendors at a later date
}

impl Collector {
    pub fn new() -> Self {
        Collector {
            gpus: Vec::new(),
            nvidia: nvidia::Collector::new(),
            intel: intel::Collector::new(),
            amd: amd::Collector::new(),
        }
    }

    pub fn collect(&mut self) -> anyhow::Result<Vec<Snapshot>> {
        let mut snapshots = Vec::new();
        if self.gpus.is_empty() {
            self.gpus = Self::enumerate_devices()?;
        }
        for gpu in self.gpus.iter() {
            let snapshot = match gpu.vendor {
                GpuVendor::Intel => self.intel.collect(&gpu.path),
                GpuVendor::Nvidia => self.nvidia.collect(&gpu.path),
                GpuVendor::Amd => self.amd.collect(&gpu.path),
            };

            match snapshot {
                Ok(mut snapshot) => {
                    snapshot.opengl_driver = gpu.opengl_driver.clone();
                    snapshot.vulkan_driver = gpu.vulkan_driver.clone();
                    snapshots.push(snapshot)
                }
                Err(e) => tracing::warn!("Failed to collect a GPU's metrics: {}", e),
            };
        }
        Ok(snapshots)
    }

    // Iterates over /sys/class/drm to find the GPU devices. This is the best way to get them in a consistent order.
    fn enumerate_devices() -> anyhow::Result<Vec<Gpu>> {
        let mut paths = Vec::new();
        for entry in std::fs::read_dir("/sys/class/drm")
            .with_context(|| format!("{} at {}", file!(), line!()))?
            .flatten()
            .filter(|entry| entry.file_name().to_string_lossy().starts_with("card"))
        {
            let path = entry.path();

            // Read vendor name
            let vendor_path = path.join("device/vendor");
            // If there is no vendor file, it's likely either a connector or a render node so it's okay to skip
            if let Ok(vendor_val) = std::fs::read_to_string(&vendor_path) {
                let vendor = match vendor_val.trim() {
                    "0x8086" => GpuVendor::Intel,
                    "0x10de" => GpuVendor::Nvidia,
                    "0x1002" => GpuVendor::Amd,
                    _ => continue,
                };

                // Get OpenGL and Vulkan drivers
                let (opengl_driver, vulkan_driver) = get_oglv_driver(&path, &vendor);

                paths.push(Gpu {
                    path,
                    vendor,
                    opengl_driver,
                    vulkan_driver,
                });
            }
        }
        Ok(paths)
    }
}

fn get_oglv_driver(path: &PathBuf, vendor: &GpuVendor) -> (String, String) {
    let device_path = path.join("device");
    let mut gl = String::from("none");
    let mut vk = String::from("none");
    if let Ok(device_real) = std::fs::read_link(&device_path) {
        let pci_id = device_real
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let instance_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN | wgpu::Backends::GL,
            flags: wgpu::InstanceFlags::empty(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds {
                for_device_loss: None,
                for_resource_creation: None,
            },
            backend_options: wgpu::BackendOptions::default(),
        };
        let instance = wgpu::Instance::new(&instance_descriptor);

        let adapters = pollster::block_on(
            instance.enumerate_adapters(wgpu::Backends::GL | wgpu::Backends::VULKAN),
        );

        for adapter in adapters {
            let adapter_info = adapter.get_info();
            // Special case: Zink!
            if adapter_info.name.contains("zink") {
                gl = format!("[zink] {}", adapter_info.driver_info);
            }
            if adapter_info.device_pci_bus_id == pci_id {
                if adapter_info.backend == wgpu::Backend::Gl {
                    gl = format!("{}", adapter_info.driver_info);
                } else if adapter_info.backend == wgpu::Backend::Vulkan {
                    vk = format!("[{}] {}", adapter_info.driver, adapter_info.driver_info);
                }
            } else if adapter_info.vendor
                == match vendor {
                    GpuVendor::Nvidia => 0x10DE,
                    GpuVendor::Amd => 0x1002,
                    GpuVendor::Intel => 0x8086,
                }
            {
                if adapter_info.backend == wgpu::Backend::Gl {
                    let driver_name = if adapter_info.name.contains("radeonsi") {
                        "radeonsi"
                    } else if adapter_info.name.contains("NVIDIA") {
                        "NVIDIA"
                    } else if adapter_info.name.contains("i915") {
                        "i915"
                    } else {
                        adapter_info.name.as_str()
                    };
                    gl = format!("[{}] {}", driver_name, adapter_info.driver_info);
                } else if adapter_info.backend == wgpu::Backend::Vulkan {
                    vk = format!("[{}] {}", adapter_info.driver, adapter_info.driver_info);
                }
            }
        }
    }
    (gl, vk)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu() -> Result<(), Box<dyn std::error::Error>> {
        tracing_subscriber::fmt::init();
        let mut collector = Collector::new();
        let _ = collector.collect()?;
        std::thread::sleep(std::time::Duration::from_secs(1));
        let second = collector.collect()?;
        for gpu in second.iter() {
            tracing::info!("GPU {}", gpu.brand_name);
            tracing::info!("  {}% Graphics", gpu.graphics_utilization);
            for proc in gpu.processes.iter() {
                tracing::info!("    {} {}% Graphics", proc.pid, proc.graphics_utilization);
            }
        }
        Ok(())
    }
}
