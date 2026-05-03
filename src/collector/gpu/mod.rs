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

use std::collections::HashMap;
use std::collections::HashSet;
use std::os::fd::OwnedFd;
use std::path::PathBuf;
use std::rc::Rc;

use crate::collector::helpers::*;
use crate::collector::staging;

#[doc(inline)]
pub use crate::metrics::gpu::*;

/// Collects GPU metrics
pub struct Collector {
    // Optimization so we don't have to traverse to /sys/class/drm every time
    drm_root: discovery::Discovery<OwnedFd>,
    cards: HashMap<CardIdentity, Box<dyn Card>>,
    nvml: discovery::Discovery<Rc<nvml_wrapper::Nvml>>,
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector {
    pub fn new() -> Self {
        Self {
            drm_root: discovery::Discovery::default(),
            cards: HashMap::new(),
            nvml: discovery::Discovery::default(),
        }
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

        let drm_root = self.drm_root.require(|| {
            rustix::fs::open(
                "/sys/class/drm",
                rustix::fs::OFlags::RDONLY
                    | rustix::fs::OFlags::DIRECTORY
                    | rustix::fs::OFlags::CLOEXEC,
                rustix::fs::Mode::empty(),
            )
            .map_err(|e| anyhow::anyhow!(e))
        })?;

        let mut seen: HashSet<CardIdentity> = HashSet::with_capacity(self.cards.len());
        let mut gpus = Vec::new();

        let dir = rustix::fs::Dir::read_from(drm_root)?;

        for entry in dir {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy();
            // skip non card* entries
            if !name.starts_with("card") || !name.contains("-") || name == "." || name == ".." {
                continue;
            }

            let card = match rustix::fs::openat(
                drm_root,
                name.as_ref(),
                rustix::fs::OFlags::RDONLY
                    | rustix::fs::OFlags::CLOEXEC
                    | rustix::fs::OFlags::DIRECTORY,
                rustix::fs::Mode::empty(),
            ) {
                Ok(file) => file,
                Err(rustix::io::Errno::NOENT) => continue,
                Err(rustix::io::Errno::NOTDIR) => continue,
                Err(e) => {
                    anyhow::bail!(e)
                }
            };
            let st = rustix::fs::fstat(&card)?;
            let id = CardIdentity {
                dev: st.st_dev,
                ino: st.st_ino,
            };
            seen.insert(id);

            match self.cards.get_mut(&id) {
                // already a fd, we can get rid of the new one
                Some(_) => {
                    drop(card);
                }
                None => {
                    let device = match new_card(
                        card,
                        self.nvml.probe(|| {
                            nvml_wrapper::Nvml::init()
                                .map_err(|e| anyhow::anyhow!(e))
                                .map(Rc::new)
                        }),
                    ) {
                        Ok(device) => device,
                        Err(e) => {
                            tracing::warn!("failed to create card tracker: {}", e);
                            continue;
                        }
                    };
                    self.cards.insert(id, device);
                }
            }

            // Usually I try to avoid unwrap whenever I can but in this case, if it's not present and has hit this part, there's a memory issue
            let gpu = self.cards.get_mut(&id).unwrap();
            let snap = match gpu.collect(config) {
                Ok(snap) => snap,
                Err(e) => {
                    tracing::warn!("failed to collect GPU snapshot: {}", e);
                    continue;
                }
            };
            gpus.push(snap);
        }

        self.cards.retain(|id, _| seen.contains(id));
        Ok(Snapshot { gpus })
    }
}

impl super::Resolver for Collector {
    fn resolve(
        &mut self,
        staging: &staging::Staging,
        output: Self::Output,
    ) -> anyhow::Result<Self::Output> {
        let mut gpus = Vec::new();
        for gpu in output.gpus.into_iter() {
            let (_, card) = self
                .cards
                .iter_mut()
                .find(|(_, card)| card.primary_node() == gpu.render_node.as_str())
                .ok_or_else(|| anyhow::anyhow!("no card found for GPU {}", gpu.brand_name))?;
            gpus.push(card.resolve(staging, gpu)?);
        }
        Ok(Snapshot { gpus })
    }
}

trait Card {
    // Collects a single snapshot of the GPU
    fn collect(&mut self, config: &Config) -> anyhow::Result<Gpu>;
    // Gets the primary node for this card (e.g. /dev/dri/card0)
    fn primary_node(&self) -> String;
    // Resolves a snapshot based on the staging
    fn resolve(&mut self, staging: &staging::Staging, output: Gpu) -> anyhow::Result<Gpu>;
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
struct CardIdentity {
    dev: u64,
    ino: u64,
}

fn new_card<'a>(
    fd: OwnedFd,
    nvml: Option<&Rc<nvml_wrapper::Nvml>>,
) -> anyhow::Result<Box<dyn Card + 'a>> {
    let driver = rustix::fs::readlinkat(&fd, "device/driver", Vec::new())?
        .to_string_lossy()
        .to_string();
    let device = match PathBuf::from(driver)
        .file_name()
        .map(|n| n.to_string_lossy())
    {
        Some(name) => {
            // match the driver name to the device type
            match name.as_ref() {
                "nvidia" => {
                    let Some(nvml) = nvml else {
                        anyhow::bail!("nvml not available for nvidia card");
                    };
                    Box::new(nvidia::Card::new(fd, nvml)?) as Box<dyn Card>
                }
                "nouveau" => Box::new(nouveau::Card::new(fd)?) as Box<dyn Card>,
                "amdgpu" => Box::new(amdgpu::Card::new(fd)?) as Box<dyn Card>,
                "i915" => Box::new(i915::Card::new(fd)?) as Box<dyn Card>,
                "xe" => Box::new(xe::Card::new(fd)?) as Box<dyn Card>,
                _ => anyhow::bail!("unsupported driver: {}", name),
            }
        }
        None => {
            anyhow::bail!("could not read driver symlink!");
        }
    };
    Ok(device)
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
