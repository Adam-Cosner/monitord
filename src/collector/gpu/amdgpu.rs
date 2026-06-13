/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::path::PathBuf;

use rustix::fd::{AsFd, BorrowedFd, OwnedFd};

use crate::collector::helpers::*;
use crate::metrics::gpu::*;

mod gpu_metrics;

pub struct Card {
    card_fd: OwnedFd,
    primary_node: PathBuf,
    render_node: PathBuf,
    gpu_metrics: OwnedFd,
    pci_id: String,

    brand_name: Discovery<String>,
    power_counters: Option<gpu_metrics::PowerCounters>,
    memory_total: Discovery<u64>,
    system_total: Discovery<u64>,
}

impl Card {
    pub fn new(fd: OwnedFd) -> anyhow::Result<Self> {
        let pci_id = PathBuf::from(
            rustix::fs::readlinkat(&fd, "device", Vec::new())?
                .to_string_lossy()
                .to_string(),
        )
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("could not read GPU PCI address"))?
        .to_string_lossy()
        .to_string();
        let drm_root = rustix::fs::openat(
            &fd,
            "device/drm",
            rustix::fs::OFlags::DIRECTORY
                | rustix::fs::OFlags::RDONLY
                | rustix::fs::OFlags::CLOEXEC,
            rustix::fs::Mode::empty(),
        )?;
        let mut primary_node = PathBuf::new();
        let mut render_node = PathBuf::new();
        for entry in rustix::fs::Dir::read_from(&drm_root)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("card") {
                primary_node = PathBuf::from(format!("/dev/dri/{}", name));
            } else if name.starts_with("renderD") {
                render_node = PathBuf::from(format!("/dev/dri/{}", name));
            }
        }
        let gpu_metrics = rustix::fs::openat(
            &fd,
            "device/gpu_metrics",
            rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC,
            rustix::fs::Mode::empty(),
        )?;
        Ok(Self {
            card_fd: fd,
            primary_node,
            render_node,
            gpu_metrics,
            pci_id,
            brand_name: Discovery::default(),
            power_counters: None,
            memory_total: Discovery::default(),
            system_total: Discovery::default(),
        })
    }

    fn memory(&mut self) -> Vec<Memory> {
        let mut memory = Vec::new();
        let vram_total = self.memory_total.probe(|| {
            sysfs::readat_u64(self.card_fd.as_fd(), "device/mem_info_vram_total")
                .ok_or(anyhow::anyhow!("could not read mem_info_vram_total"))
        });
        let vram_used = sysfs::readat_u64(self.card_fd.as_fd(), "device/mem_info_vram_used");
        let system_total = self.system_total.probe(|| {
            sysfs::readat_u64(self.card_fd.as_fd(), "device/mem_info_gtt_total")
                .ok_or(anyhow::anyhow!("could not read mem_info_gtt_total"))
        });
        let system_used = sysfs::readat_u64(self.card_fd.as_fd(), "device/mem_info_gtt_used");

        if let Some(&vram_total) = vram_total {
            memory.push(Memory {
                r#type: MemoryType::Vram as i32,
                total_memory: vram_total,
                used_memory: vram_used.unwrap_or(0),
            });
        }
        if let Some(&system_total) = system_total {
            memory.push(Memory {
                r#type: MemoryType::System as i32,
                total_memory: system_total,
                used_memory: system_used.unwrap_or(0),
            });
        }
        memory
    }
}

impl super::Card for Card {
    fn identify(&self) -> (String, String, Option<String>, Option<String>) {
        (String::new(), String::new(), None, None)
    }

    fn collect(&mut self, config: &super::Config) -> anyhow::Result<super::Gpu> {
        rustix::fs::seek(self.gpu_metrics.as_fd(), rustix::fs::SeekFrom::Start(0))?;
        let bytes = sysfs::read_bin(self.gpu_metrics.as_fd())
            .ok_or_else(|| anyhow::anyhow!("could not read gpu_metrics file!"))?;
        let gpu_metrics = gpu_metrics::GpuMetrics::read(&bytes)?;

        let mut gpu = super::Gpu::default();
        gpu.brand_name = self
            .brand_name
            .probe(|| get_brand_name(self.card_fd.as_fd()))
            .cloned()
            .unwrap_or_default();
        gpu.drivers = config.drivers.then(|| Drivers {
            kernel: Some(KernelDriver {
                name: "amdgpu".to_string(),
                version: None,
            }),
            opengl: None,
            vulkan: None,
        });
        gpu.primary_node = self.primary_node.to_string_lossy().to_string();
        gpu.render_node = self.render_node.to_string_lossy().to_string();
        gpu.pci_id = rustix::fs::readlinkat(self.card_fd.as_fd(), "device", [])
            .ok()
            .and_then(|p| {
                PathBuf::from(p.to_string_lossy().to_string())
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
            })
            .unwrap_or_default();
        gpu.engines = config
            .engines
            .then(|| gpu_metrics.engines())
            .unwrap_or_default();
        gpu.clocks = config
            .clocks
            .then(|| gpu_metrics.clocks())
            .unwrap_or_default();
        gpu.memory = config.memory.then(|| self.memory()).unwrap_or_default();
        gpu.power = config
            .power
            .then(|| {
                let Some((power, counters)) = gpu_metrics.power(self.power_counters.as_ref())
                else {
                    return None;
                };
                self.power_counters = Some(counters);
                Some(power)
            })
            .unwrap_or_default();
        gpu.thermals = config
            .thermals
            .then(|| gpu_metrics.thermals())
            .unwrap_or_default();

        populate_max_clocks(self.card_fd.as_fd(), gpu.clocks.as_mut());
        populate_max_power(self.card_fd.as_fd(), gpu.power.as_mut());
        populate_max_thermal(self.card_fd.as_fd(), gpu.thermals.as_mut());

        Ok(gpu)
    }

    fn resolve(
        &mut self,
        _input: &super::process::Snapshot,
        _output: &mut Gpu,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn pci_id(&self) -> String {
        self.pci_id.clone()
    }
}

fn get_brand_name(fd: BorrowedFd) -> anyhow::Result<String> {
    let device = sysfs::readat_string(fd.as_fd(), "device/device")
        .and_then(|dev| dev.strip_prefix("0x").map(|dev| dev.to_string()))
        .map(|dev| dev.to_string())
        .ok_or(anyhow::anyhow!("failed to read card device id"))?;
    let revision = sysfs::readat_string(fd.as_fd(), "device/revision")
        .and_then(|rev| rev.strip_prefix("0x").map(|rev| rev.to_string()))
        .map(|rev| rev.to_string())
        .ok_or(anyhow::anyhow!("failed to read card revision"))?;

    let amdgpu_ids = sysfs::read_string_path("/usr/share/libdrm/amdgpu.ids").ok_or(
        anyhow::anyhow!("amdgpu.ids file not found, falling back to pci ids"),
    )?;

    for line in amdgpu_ids.lines() {
        let mut token = line.split(',');
        let Some(dev_token) = token.next().map(|t| t.trim()) else {
            continue;
        };
        let Some(rev_token) = token.next().map(|t| t.trim()) else {
            continue;
        };
        let Some(name) = token.next().map(|t| t.trim()) else {
            continue;
        };
        if dev_token.eq_ignore_ascii_case(&device) && rev_token.eq_ignore_ascii_case(&revision) {
            return Ok(name.to_string());
        }
    }
    Err(anyhow::anyhow!(
        "no matching amdgpu id found for device {device} revision {revision}"
    ))
}

fn populate_max_clocks(fd: BorrowedFd, clocks: &mut [Clock]) {
    for clock in clocks.iter_mut() {
        let Some(identifier) = clock.identifier.as_ref() else {
            continue;
        };
        let max_freq = match identifier.domain() {
            super::ClockDomain::Graphics => {
                let Some(gfxclk) = sysfs::readat_string(fd, "device/pp_dpm_sclk") else {
                    continue;
                };
                let mut max_freq = 0u32;
                for line in gfxclk.lines() {
                    let Some(freq) = line
                        .split_whitespace()
                        .nth(1)
                        .and_then(|f| f.strip_suffix("Mhz"))
                        .and_then(|f| f.parse::<u32>().ok())
                    else {
                        continue;
                    };
                    max_freq = max_freq.max(freq);
                }
                max_freq
            }
            super::ClockDomain::VideoUnified => {
                let Some(vclk) = sysfs::readat_string(fd, "device/pp_dpm_vclk") else {
                    continue;
                };
                let mut max_freq = 0u32;
                for line in vclk.lines() {
                    let Some(freq) = line
                        .split_whitespace()
                        .nth(1)
                        .and_then(|f| f.strip_suffix("Mhz"))
                        .and_then(|f| f.parse::<u32>().ok())
                    else {
                        continue;
                    };
                    max_freq = max_freq.max(freq);
                }
                max_freq
            }
            super::ClockDomain::VideoDecode => {
                let Some(dclk) = sysfs::readat_string(fd, "device/pp_dpm_dclk") else {
                    continue;
                };
                let mut max_freq = 0u32;
                for line in dclk.lines() {
                    let Some(freq) = line
                        .split_whitespace()
                        .nth(1)
                        .and_then(|f| f.strip_suffix("Mhz"))
                        .and_then(|f| f.parse::<u32>().ok())
                    else {
                        continue;
                    };
                    max_freq = max_freq.max(freq);
                }
                max_freq
            }
            super::ClockDomain::Soc => {
                let Some(socclk) = sysfs::readat_string(fd, "device/pp_dpm_socclk") else {
                    continue;
                };
                let mut max_freq = 0u32;
                for line in socclk.lines() {
                    let Some(freq) = line
                        .split_whitespace()
                        .nth(1)
                        .and_then(|f| f.strip_suffix("Mhz"))
                        .and_then(|f| f.parse::<u32>().ok())
                    else {
                        continue;
                    };
                    max_freq = max_freq.max(freq);
                }
                max_freq
            }
            super::ClockDomain::Memory => {
                let Some(mclk) = sysfs::readat_string(fd, "device/pp_dpm_mclk") else {
                    continue;
                };
                let mut max_freq = 0u32;
                for line in mclk.lines() {
                    let Some(freq) = line
                        .split_whitespace()
                        .nth(1)
                        .and_then(|f| f.strip_suffix("Mhz"))
                        .and_then(|f| f.parse::<u32>().ok())
                    else {
                        continue;
                    };
                    max_freq = max_freq.max(freq);
                }
                max_freq
            }
            _ => continue,
        };
        clock.max_frequency_mhz = max_freq;
    }
}

fn populate_max_power(fd: BorrowedFd, power: Option<&mut Power>) {
    let Some(power) = power else {
        return;
    };
    let Some(power1_cap) = sysfs::first_hwmon_subdir_at(fd, "device/hwmon")
        .and_then(|hwmon| sysfs::readat_u32(hwmon.as_fd(), "power1_cap"))
    else {
        return;
    };
    power.max_power_mw = power1_cap;
}

fn populate_max_thermal(fd: BorrowedFd, thermals: &mut [Thermal]) {
    let Some(hwmon) = sysfs::first_hwmon_subdir_at(fd, "device/hwmon") else {
        return;
    };
    for thermal in thermals.iter_mut() {
        let Some(temp) = (match thermal.location() {
            super::ThermalLocation::Edge => sysfs::readat_u32(hwmon.as_fd(), "temp1_crit"),
            super::ThermalLocation::Hotspot => sysfs::readat_u32(hwmon.as_fd(), "temp2_crit"),
            super::ThermalLocation::Memory => sysfs::readat_u32(hwmon.as_fd(), "temp3_crit"),
            _ => continue,
        }) else {
            continue;
        };
        thermal.max_celsius = temp / 1000;
    }
}
