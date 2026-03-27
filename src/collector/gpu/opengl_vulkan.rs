/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Reader for OpenGL and Vulkan driver information

use crate::collector::{gpu::GpuVendor, helpers::discovery::Discovery};
use std::path::Path;

/// Collects OpenGL and Vulkan driver information
pub struct Collector {
    instance: Discovery<wgpu::Instance>,
    adapters: Discovery<Vec<wgpu::Adapter>>,
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector {
    pub fn new() -> Self {
        Self {
            instance: Discovery::default(),
            adapters: Discovery::default(),
        }
    }

    /// Returns the OpenGL and Vulkan drivers for the GPU at the given path respectively.
    pub fn get_drivers(&mut self, path: &Path, vendor: &GpuVendor) -> (String, String) {
        let instance = self.instance.require(|| {
            let instance_descriptor = wgpu::InstanceDescriptor {
                backends: wgpu::Backends::VULKAN | wgpu::Backends::GL,
                flags: wgpu::InstanceFlags::empty(),
                memory_budget_thresholds: wgpu::MemoryBudgetThresholds {
                    for_device_loss: None,
                    for_resource_creation: None,
                },
                display: None,
                backend_options: wgpu::BackendOptions::default(),
            };
            Ok(wgpu::Instance::new(instance_descriptor))
        });
        match instance {
            Ok(instance) => {
                let driver_bench = std::time::Instant::now();
                tracing::debug!("Getting OpenGL and Vulkan drivers for GPU {:?}", path);
                let device_path = path.join("device");
                let mut gl = String::from("none");
                let mut vk = String::from("none");
                if let Ok(device_real) = std::fs::read_link(&device_path) {
                    let pci_id = device_real
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default();

                    let adapters = self.adapters.require(|| {
                        Ok(pollster::block_on(instance.enumerate_adapters(
                            wgpu::Backends::GL | wgpu::Backends::VULKAN,
                        )))
                    });

                    if let Ok(adapters) = adapters {
                        (gl, vk) = iterate_adapters(&adapters, &pci_id, vendor);
                    }
                }
                tracing::debug!(
                    "Got OpenGL and Vulkan drivers for GPU {:?} took {:?}",
                    path,
                    driver_bench.elapsed()
                );
                (gl, vk)
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to get OpenGL and Vulkan drivers for GPU {:?}: {}",
                    path,
                    e
                );
                (String::from("none"), String::from("none"))
            }
        }
    }
}

/// Iterates over adapters to find the OpenGL and Vulkan drivers for a given PCI ID and vendor.
fn iterate_adapters(
    adapters: &[wgpu::Adapter],
    pci_id: &str,
    vendor: &GpuVendor,
) -> (String, String) {
    let mut gl = String::from("none");
    let mut vk = String::from("none");
    for adapter in adapters {
        let adapter_info = adapter.get_info();
        // Special case: Zink
        if adapter_info.name.contains("zink") {
            gl = format!("[zink] {}", adapter_info.driver_info);
        } else if adapter_info.device_pci_bus_id == pci_id {
            if adapter_info.backend == wgpu::Backend::Gl {
                gl = adapter_info.driver_info.to_string();
            } else if adapter_info.backend == wgpu::Backend::Vulkan {
                vk = format!("[{}] {}", adapter_info.driver, adapter_info.driver_info);
            }
        } else if adapter_info.vendor
            == match vendor {
                GpuVendor::Nvidia => 0x10de,
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
    (gl, vk)
}
