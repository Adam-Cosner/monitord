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
//! let mut collector = monitord::collector::gpu::Collector::new();
//! let store = monitord::collector::store::Store::new();
//! // The first collect call will only return NVIDIA GPUs due to it being the only driver that stores metrics
//! collector.collect(&store).unwrap();
//! std::thread::sleep(std::time::Duration::from_secs(1));
//! // Subsequent collect calls will return all supported GPUs
//! collector.collect(&store).unwrap();
//! assert!(store.gpu.get().is_some_and(|g| g.len() > 0));
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
pub use crate::metrics::gpu::Gpu;
#[doc(inline)]
pub use crate::metrics::gpu::Process;
#[doc(inline)]
pub use crate::metrics::gpu::Snapshot;

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

    fn collect(&mut self, store: &store::Store) -> anyhow::Result<()> {
        match self.collect_gpus() {
            Ok(gpus) => store
                .gpu
                .set(gpus)
                .expect("gpu snapshot was already set previously, do not reuse Store instances!"),
            Err(e) => {
                tracing::error!("[gpu] collector failed: {e}");
                return Err(e);
            }
        }
        Ok(())
    }
}

impl Collector {
    pub fn new() -> Self {
        tracing::info!("Creating GPU collector");
        Collector {
            gpus: Vec::new(),
            oglv_collector: opengl_vulkan::Collector::new(),
            nvidia: nvidia::Collector::new(),
            intel: intel::Collector::new(),
            amd: amd::Collector::new(),
        }
    }

    /// Collects the GPU metrics and returns a snapshot.
    fn collect_gpus(&mut self) -> anyhow::Result<Snapshot> {
        let mut gpus = Vec::new();
        if self.gpus.is_empty() {
            self.gpus = self.enumerate_devices()?;
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
                    gpus.push(snapshot)
                }
                Err(e) => tracing::warn!("Failed to collect a GPU's metrics: {}", e),
            };
        }
        Ok(Snapshot { gpus })
    }

    /// Iterates over /sys/class/drm to find the GPU devices. This is the best way to get them in a consistent order.
    fn enumerate_devices(&mut self) -> anyhow::Result<Vec<GpuCache>> {
        let enumerate_bench = std::time::Instant::now();
        tracing::debug!("Enumerating GPU device paths");
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
            if let Some(vendor_val) = sysfs::read_string(&vendor_path) {
                let vendor = match vendor_val.trim() {
                    "0x8086" => GpuVendor::Intel,
                    "0x10de" => GpuVendor::Nvidia,
                    "0x1002" => GpuVendor::Amd,
                    _ => continue,
                };

                // Get OpenGL and Vulkan drivers
                let (opengl_driver, vulkan_driver) =
                    self.oglv_collector.get_drivers(&path, &vendor);

                tracing::trace!(
                    "Found a {} GPU at {}, OpenGL: {}, Vulkan: {}",
                    vendor,
                    path.display(),
                    opengl_driver,
                    vulkan_driver
                );
                paths.push(GpuCache {
                    path,
                    vendor,
                    opengl_driver,
                    vulkan_driver,
                });
            }
        }
        tracing::debug!(
            "Enumerated GPU device paths in {:?}",
            enumerate_bench.elapsed()
        );
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
        collector.collect(&store)?;
        std::thread::sleep(std::time::Duration::from_secs(1));
        store = store::Store::new();
        collector.collect(&store)?;
        println!("{:#?}", store.gpu.get());
        Ok(())
    }
}
