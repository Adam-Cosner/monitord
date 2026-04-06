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
//! use monitord::collector::Collector;
//! let mut collector = monitord::collector::gpu::Collector::new();
//! let store = monitord::collector::store::Store::new();
//! // The first collect call will only return NVIDIA GPUs due to it being the only driver that stores metrics
//! collector.collect(&store).unwrap();
//! std::thread::sleep(std::time::Duration::from_secs(1));
//! // Subsequent collect calls will return all supported GPUs
//! collector.collect(&store).unwrap();
//! assert!(store.gpu.get().is_some_and(|g| g.gpus.len() > 0));
//! ```

mod amd;
mod intel;
mod nvidia;
mod opengl_vulkan;

use anyhow::Context;
use std::path::PathBuf;

use crate::collector::helpers::*;
use crate::collector::store;
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

    fn dependencies(&self) -> &[&'static str] {
        &[]
    }

    fn collect(
        &mut self,
        config: &crate::metrics::Config,
        store: &store::Store,
    ) -> anyhow::Result<()> {
        match self.collect_gpus(config) {
            Ok(gpus) => store
                .gpu
                .set(gpus)
                .expect("gpu snapshot was already set previously, do not reuse Store instances!"),
            Err(e) => {
                tracing::error!("collector failed: {e}");
                return Err(e);
            }
        }
        Ok(())
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
    fn collect_gpus(&mut self, config: &crate::metrics::Config) -> anyhow::Result<Snapshot> {
        let mut gpus = Vec::new();
        let Some(config) = config.gpu else {
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

    #[test]
    fn gpu() -> Result<(), Box<dyn std::error::Error>> {
        let _ = tracing_subscriber::fmt::try_init();
        let mut collector = super::Collector::new();
        let mut store = store::Store::new();
        let mut config = crate::metrics::Config::default();
        config.gpu = Some(crate::metrics::gpu::Config {
            drivers: true,
            engines: true,
            clocks: true,
            memory: true,
            power: true,
            thermals: true,
            processes: true,
        });
        collector.collect(&config, &store)?;
        std::thread::sleep(std::time::Duration::from_secs(1));
        store = store::Store::new();
        collector.collect(&config, &store)?;
        println!("{:#?}", store.gpu.get());
        Ok(())
    }
}
