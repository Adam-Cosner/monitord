/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! GPU Metric Collection
//!
//! # Example
//!
//! ```no_run
//!
//! ```
mod amdgpu;
mod i915;
mod nouveau;
mod nvidia;
mod xe;

use crate::collector::helpers::*;
use crate::collector::staging;

#[doc(inline)]
pub use crate::metrics::gpu::*;

/// Collects GPU metrics
pub struct Collector {
    cards: discovery::Discovery<Vec<Box<dyn Card>>>,
    nvml: discovery::Discovery<nvml_wrapper::Nvml>,
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector {
    pub fn new() -> Self {
        Self {
            cards: discovery::Discovery::default(),
            nvml: discovery::Discovery::default(),
        }
    }

    fn enumerate_cards(&mut self) -> anyhow::Result<&mut [Box<dyn Card>]> {
        self.cards
            .probe_mut(|| {
                let mut cards = Vec::new();
                for entry in std::fs::read_dir("/sys/class/drm")? {
                    let entry = entry?;
                    let Ok(name) = entry.file_name().into_string() else {
                        continue;
                    };
                    if name.starts_with("card") && !name.contains('-') {
                        // check the symlink of device/driver
                        let Ok(driver) = std::fs::read_link(entry.path().join("device/driver"))
                            .map_err(|e| anyhow::anyhow!(e))
                            .and_then(|driver_path| {
                                driver_path
                                    .file_name()
                                    .map(|file_name| file_name.to_string_lossy().to_string())
                                    .ok_or(anyhow::anyhow!("invalid driver symlink"))
                            })
                        else {
                            continue;
                        };
                        match driver.as_str() {
                            "amdgpu" => {
                                let Ok(gpu) = amdgpu::Card::new(entry.path()) else {
                                    continue;
                                };
                                cards.push(Box::new(gpu) as Box<dyn Card>)
                            }
                            "i915" => {
                                let Ok(gpu) = i915::Card::new(entry.path()) else {
                                    continue;
                                };
                                cards.push(Box::new(gpu) as Box<dyn Card>)
                            }
                            "xe" => {
                                let Ok(gpu) = xe::Card::new(entry.path()) else {
                                    continue;
                                };
                                cards.push(Box::new(gpu) as Box<dyn Card>)
                            }
                            "nvidia" => {
                                let Some(nvml) = self.nvml.probe(|| {
                                    nvml_wrapper::Nvml::init().map_err(|e| anyhow::anyhow!(e))
                                }) else {
                                    continue;
                                };
                                let Ok(gpu) = nvidia::Card::new(entry.path(), nvml) else {
                                    continue;
                                };
                                cards.push(Box::new(gpu) as Box<dyn Card>)
                            }
                            "nouveau" => {
                                let Ok(gpu) = nouveau::Card::new(entry.path()) else {
                                    continue;
                                };
                                cards.push(Box::new(gpu) as Box<dyn Card>)
                            }
                            _ => {
                                tracing::warn!("unsupported GPU driver: {}", driver);
                                continue;
                            }
                        }
                    }
                }
                Ok(cards)
            })
            .map(|cards| cards.as_mut_slice())
            .ok_or_else(|| anyhow::anyhow!("GPU Collector failed to enumerate cards"))
    }
}

impl super::Collector for Collector {
    type Output = Snapshot;

    fn name(&self) -> &'static str {
        "gpu"
    }

    fn collect(&mut self, config: &crate::metrics::Config) -> anyhow::Result<Self::Output> {
        let Some(config) = &config.gpu else {
            anyhow::bail!("GPU Collector did not receive a config");
        };
        let mut gpus = Vec::new();
        let cards = self.enumerate_cards()?;
        for gpu in cards.iter_mut() {
            gpus.push(gpu.collect(config)?);
        }
        Ok(Snapshot { gpus })
    }
}

trait Card {
    // Collects a single snapshot of the GPU
    fn collect(&mut self, config: &Config) -> anyhow::Result<Gpu>;
    // Resolves a snapshot based on the staging
    fn resolve(&mut self, staging: &staging::Staging, output: Gpu) -> anyhow::Result<Gpu>;
}

/*
mod amd;
mod intel;
mod nvidia;
mod opengl_vulkan;

use anyhow::Context;
use std::path::PathBuf;

use crate::collector::helpers::*;
use crate::collector::staging;
#[doc(inline)]
pub use crate::metrics::gpu::*;

pub struct Collector {
    gpus: Vec<GpuCache>,
    oglv_collector: opengl_vulkan::Collector,
    nvidia: nvidia::Collector,
    intel: intel::Collector,
    amd: amd::Collector,
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for GpuVendor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuVendor::Intel => write!(f, "Intel"),
            GpuVendor::Nvidia => write!(f, "Nvidia"),
            GpuVendor::Amd => write!(f, "AMD"),
        }
    }
}

impl super::Collector for Collector {
    type Output = Snapshot;

    fn name(&self) -> &'static str {
        "gpu"
    }

    fn collect(&mut self, config: &crate::metrics::Config) -> anyhow::Result<Self::Output> {
        self.collect_gpus(config.gpu.as_ref())
            .inspect_err(|e| tracing::error!("collector failed: {e}"))
    }
}

impl Collector {
    pub fn new() -> Self {
        tracing::info!("creating collector");
        Collector {
            gpus: Vec::new(),
            oglv_collector: opengl_vulkan::Collector::new(),
            nvidia: nvidia::Collector::new(),
            intel: intel::Collector::new(),
            amd: amd::Collector::new(),
        }
    }

    /// Collects the GPU metrics and returns a snapshot.
    fn collect_gpus(&mut self, config: Option<&Config>) -> anyhow::Result<Snapshot> {
        let mut gpus = Vec::new();
        let Some(config) = config else {
            anyhow::bail!("gpu collector did not receive a config");
        };

        if self.gpus.is_empty() {
            self.gpus = self.collect_static_info()?;
        }
        for gpu in self.gpus.iter() {
            let snapshot = match gpu.vendor {
                GpuVendor::Intel => self.intel.collect(&gpu.path, &config),
                GpuVendor::Nvidia => self.nvidia.collect(&gpu.path, &config),
                GpuVendor::Amd => self.amd.collect(&gpu.path, &config),
            };

            match snapshot {
                Ok(mut snapshot) => {
                    if config.drivers {
                        snapshot.drivers.as_mut().map(|drivers| {
                            drivers.opengl = gpu.opengl_driver.clone();
                            drivers.vulkan = gpu.vulkan_driver.clone();
                        });
                    }
                    gpus.push(snapshot)
                }
                Err(e) => tracing::warn!("failed to collect a GPU's metrics: {}", e),
            };
        }
        Ok(Snapshot { gpus })
    }

    /// Iterates over /sys/class/drm to find the GPU devices. This is the best way to get them in a consistent order.
    fn collect_static_info(&mut self) -> anyhow::Result<Vec<GpuCache>> {
        let mut paths = Vec::new();
        for entry in std::fs::read_dir("/sys/class/drm")
            .with_context(|| format!("{} at {}", file!(), line!()))?
            .flatten()
            .filter(|entry| entry.file_name().to_string_lossy().starts_with("renderD"))
        {
            let path = entry.path();

            // Read vendor name
            let vendor_path = path.join("device/vendor");
            let device_path = path.join("device/device");
            let Some(vendor_val) = sysfs::read_string(&vendor_path) else {
                continue;
            };
            let Some(device_val) = sysfs::read_hex(&device_path) else {
                continue;
            };

            let vendor = match vendor_val.as_str() {
                "0x8086" => GpuVendor::Intel,
                "0x10de" => GpuVendor::Nvidia,
                "0x1002" => GpuVendor::Amd,
                _ => continue,
            };

            // Get OpenGL and Vulkan drivers
            let (opengl_driver, vulkan_driver) =
                self.oglv_collector
                    .get_drivers(&path, &vendor, device_val as u16);

            paths.push(GpuCache {
                path,
                vendor,
                opengl_driver,
                vulkan_driver,
            });
        }
        Ok(paths)
    }
}

/// Caches information about a GPU device.
struct GpuCache {
    path: PathBuf,
    vendor: GpuVendor,
    opengl_driver: String,
    vulkan_driver: String,
}

/// The vendor of a GPU device.
enum GpuVendor {
    Intel,
    Nvidia,
    Amd,
    // TODO: Add support for smaller vendors at a later date
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collector::Collector;

    #[tracing_test::traced_test]
    #[test]
    fn gpu() -> Result<(), Box<dyn std::error::Error>> {
        let mut collector = super::Collector::new();
        let mut config = crate::metrics::Config::default();
        config.gpu = Some(Config {
            drivers: true,
            engines: true,
            clocks: true,
            memory: true,
            power: true,
            thermals: true,
            processes: true,
        });
        let _ = collector.collect(&config)?;
        std::thread::sleep(std::time::Duration::from_secs(1));
        let snapshot = collector.collect(&config)?;
        assert!(!snapshot.gpus.is_empty());
        println!("{:#?}", snapshot);
        Ok(())
    }
}
 */
