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

mod api_drivers;

use std::collections::HashMap;
use std::collections::HashSet;

use std::path::PathBuf;
use std::rc::Rc;

use crate::collector::helpers::*;
use crate::metrics::process;
use rustix::fd::{AsFd, OwnedFd};

#[doc(inline)]
pub use crate::metrics::gpu::*;

/// Collects GPU metrics
pub struct Collector {
    // Optimization so we don't have to traverse to /sys/class/drm every time
    drm_root: Discovery<OwnedFd>,
    pci_ids: Discovery<PciIds>,
    cards: HashMap<CardFileId, Box<dyn Card>>,
    nvml: Discovery<Rc<nvml_wrapper::Nvml>>,
    drivers: Discovery<api_drivers::DriverInfo>,
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector {
    pub fn new() -> Self {
        Self {
            drm_root: Discovery::default(),
            pci_ids: Discovery::default(),
            cards: HashMap::default(),
            nvml: Discovery::default(),
            drivers: Discovery::default(),
        }
    }
}

impl super::Collector for Collector {
    type Output = Snapshot;

    fn name(&self) -> &'static str {
        "gpu"
    }

    fn collect(&mut self, config: &crate::metrics::Config) -> anyhow::Result<Self::Output> {
        tracing::trace!("collecting GPU metrics");
        let Some(api_drivers) = self.drivers.probe(|| Ok(api_drivers::get_drivers())) else {
            anyhow::bail!("failed to collect graphics API drivers");
        };

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

        let mut seen: HashSet<CardFileId> = HashSet::with_capacity(self.cards.len());
        let mut gpus = Vec::new();

        let dir = rustix::fs::Dir::read_from(drm_root)?;

        for entry in dir {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy();
            // skip non card* entries
            if !name.starts_with("card") || name.contains("-") || name == "." || name == ".." {
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
            let id = CardFileId {
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
                    let device = match new_card(card, &mut self.nvml) {
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
            let mut snap = match gpu.collect(config) {
                Ok(snap) => snap,
                Err(e) => {
                    tracing::warn!("failed to collect GPU snapshot: {}", e);
                    continue;
                }
            };
            // GPU name fallback
            if snap.brand_name.is_empty() {
                snap.brand_name = sysfs::read_string_path("/usr/share/hwdata/pci.ids")
                    .or_else(|| sysfs::read_string_path("/usr/share/misc/pci.ids"))
                    .and_then(|pci_ids| self.pci_ids.probe(|| PciIds::parse(&pci_ids)))
                    .and_then(|pci_ids| {
                        let (vendor, device, subvendor, subdevice) = gpu.identify();
                        pci_ids.lookup(&vendor, &device, subvendor.as_deref(), subdevice.as_deref())
                    })
                    .map(String::from)
                    .unwrap_or_default();
            }
            // Driver association
            if let Some(drivers) = snap.drivers.as_mut() {
                if let Some(opengl) = api_drivers.gl_drivers.get(
                    &PathBuf::from(&snap.render_node)
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                ) {
                    drivers.opengl = Some(opengl.clone());
                }
                if let Some(vulkan) = api_drivers.vk_drivers.get(
                    &PathBuf::from(&snap.pci_id)
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                ) {
                    drivers.vulkan = Some(vulkan.clone());
                }
            }

            gpus.push(snap);
        }

        self.cards.retain(|id, _| seen.contains(id));
        Ok(Snapshot { gpus })
    }
}

impl super::Resolver for Collector {
    type Input = crate::metrics::process::Snapshot;

    fn resolve(&mut self, input: &Self::Input, output: &mut Self::Output) -> anyhow::Result<()> {
        let mut gpus = Vec::new();
        for gpu in output.gpus.iter_mut() {
            let (_, card) = self
                .cards
                .iter_mut()
                .find(|(_, card)| card.pci_id() == gpu.pci_id.as_str())
                .ok_or_else(|| anyhow::anyhow!("no card found for GPU {}", gpu.brand_name))?;
            gpus.push(card.resolve(input, gpu)?);
        }
        Ok(())
    }
}

trait Card {
    // Gets the identity of the card (vendor:device:subvendor:subdevice)
    fn identify(&self) -> (String, String, Option<String>, Option<String>);
    // Collects a single snapshot of the GPU
    fn collect(&mut self, config: &Config) -> anyhow::Result<Gpu>;
    // Gets the pci id of the card (e.g. 0000:01:00.0)
    fn pci_id(&self) -> String;
    // Resolves a snapshot based on the staging
    fn resolve(&mut self, input: &process::Snapshot, output: &mut Gpu) -> anyhow::Result<()>;
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
struct CardFileId {
    dev: u64,
    ino: u64,
}

fn new_card<'a>(
    fd: OwnedFd,
    nvml: &mut Discovery<Rc<nvml_wrapper::Nvml>>,
) -> anyhow::Result<Box<dyn Card + 'a>> {
    let driver = rustix::fs::readlinkat(fd.as_fd(), "device/driver", Vec::new())?
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
                    let Some(nvml) = nvml.probe(|| {
                        nvml_wrapper::Nvml::init()
                            .map_err(|e| anyhow::anyhow!(e))
                            .map(Rc::new)
                    }) else {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collector::Collector;

    #[test]
    fn gpu() -> Result<(), Box<dyn std::error::Error>> {
        tracing_subscriber::fmt::init();
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
