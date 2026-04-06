/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
mod gpu_metrics;

use crate::collector::helpers::*;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

pub(super) struct Collector {
    device_names: HashMap<String, String>,
    last_counters: HashMap<String, gpu_metrics::PowerCounters>,
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}
const DEFAULT_NAME: &'static str = "AMD Radeon Graphics";

impl Collector {
    pub fn new() -> Self {
        tracing::debug!("initializing AMD GPU collector");
        Collector {
            device_names: HashMap::new(),
            last_counters: HashMap::new(),
        }
    }

    pub fn collect(&mut self, path: &Path, config: &super::Config) -> anyhow::Result<super::Gpu> {
        let gpu_metrics_path = path.join("device/gpu_metrics");
        let metrics = gpu_metrics::GpuMetrics::read(&gpu_metrics_path)?;

        let brand_name = self.get_brand_name(path);
        let drivers = if config.drivers {
            Some(super::Drivers {
                kernel: String::from("amdgpu"),
                opengl: String::new(),
                vulkan: String::new(),
            })
        } else {
            None
        };

        let engines = if config.engines {
            metrics.engines()
        } else {
            Vec::new()
        };

        let mut clocks = if config.clocks {
            metrics.clocks()
        } else {
            Vec::new()
        };

        let memory = if config.memory {
            let mut memory = Vec::new();
            match (get_memory_usage(path), get_memory_capacity(path)) {
                (Err(e1), Err(e2)) => {
                    tracing::warn!("failed to read vram usage: {e1}");
                    tracing::warn!("failed to read vram capacity: {e2}");
                }
                (Err(e), _) => {
                    tracing::warn!("failed to read vram usage: {e}")
                }
                (_, Err(e)) => {
                    tracing::warn!("failed to read vram capacity: {e}")
                }
                (Ok(used_memory), Ok(total_memory)) => {
                    memory.push(super::Memory {
                        r#type: super::MemoryType::Vram as i32,
                        total_memory,
                        used_memory,
                    });
                }
            }
            match (get_gtt_usage(path), get_gtt_capacity(path)) {
                (Err(e1), Err(e2)) => {
                    tracing::warn!("failed to read gtt usage: {e1}");
                    tracing::warn!("failed to read gtt capacity: {e2}");
                }
                (Err(e), _) => {
                    tracing::warn!("failed to read gtt usage: {e}")
                }
                (_, Err(e)) => {
                    tracing::warn!("failed to read gtt capacity: {e}")
                }
                (Ok(used_memory), Ok(total_memory)) => {
                    memory.push(super::Memory {
                        r#type: super::MemoryType::Gtt as i32,
                        total_memory,
                        used_memory,
                    });
                }
            }
            memory
        } else {
            Vec::new()
        };

        let mut power = if config.power {
            if let Some((pwr, counters)) = metrics.power(
                self.last_counters.get(
                    &path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                ),
            ) {
                self.last_counters.insert(
                    path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    counters,
                );
                Some(pwr)
            } else {
                None
            }
        } else {
            None
        };

        let mut thermals = if config.thermals {
            metrics.thermals()
        } else {
            Vec::new()
        };

        // Read max frequencies
        match populate_clocks(&mut clocks, path) {
            Ok(()) => {}
            Err(e) => {
                tracing::warn!("could not read amdgpu clock frequencies: {e}");
            }
        }

        // read max power
        match populate_power(power.as_mut(), path) {
            Ok(()) => {}
            Err(e) => {
                tracing::warn!("could not read amdgpu power: {e}");
            }
        }

        // read max thermal
        match populate_thermal(&mut thermals, path) {
            Ok(()) => {}
            Err(e) => {
                tracing::warn!("could not read amdgpu thermal: {e}");
            }
        }

        // Read processes from fdinfo
        let processes = match self.get_fdinfo(/* params */) {
            Ok(processes) => processes,
            Err(e) => {
                tracing::warn!("could not read amdgpu process fdinfo: {e}");
                Vec::new()
            }
        };

        Ok(super::Gpu {
            brand_name,
            drivers,
            engines,
            clocks,
            memory,
            power,
            thermals,
            processes,
        })
    }

    fn get_fdinfo(&mut self) -> anyhow::Result<Vec<super::Process>> {
        tracing::warn!("amdgpu get_fdinfo is unimplemented! returning no data");
        Ok(vec![])
    }

    fn get_brand_name(&mut self, path: &Path) -> String {
        let Some(card_id) = path
            .file_name()
            .map(|card| card.to_string_lossy().to_string())
        else {
            tracing::info!("could not get card name!");
            return DEFAULT_NAME.to_string();
        };

        let Some(name) = self.device_names.get(&card_id) else {
            let Some(device) = sysfs::read_string(&path.join("device/device"))
                .and_then(|dev| dev.strip_prefix("0x").map(|dev| dev.to_string()))
                .map(|dev| dev.to_string())
            else {
                tracing::info!("could not get device id file!");
                self.device_names.insert(card_id, DEFAULT_NAME.to_string());
                return DEFAULT_NAME.to_string();
            };

            let Some(revision) = sysfs::read_string(&path.join("device/revision"))
                .and_then(|dev| dev.strip_prefix("0x").map(|dev| dev.to_string()))
                .map(|dev| dev.to_string())
            else {
                tracing::info!("could not get revision id file!");
                self.device_names.insert(card_id, DEFAULT_NAME.to_string());
                return DEFAULT_NAME.to_string();
            };

            let Some(amdgpu_ids) =
                sysfs::read_string(&PathBuf::from("/usr/share/libdrm/amdgpu.ids"))
            else {
                tracing::info!("could not open amdgpu.ids file!");
                self.device_names.insert(card_id, DEFAULT_NAME.to_string());
                return DEFAULT_NAME.to_string();
            };

            tracing::info!("checking amdgpu.ids file for {device} {revision}");

            for line in amdgpu_ids.lines() {
                let dev_rev_name: Vec<&str> = line.split(',').map(|val| val.trim()).collect();
                let Some(((dev, rev), name)) = dev_rev_name
                    .get(0)
                    .zip(dev_rev_name.get(1))
                    .zip(dev_rev_name.get(2))
                else {
                    continue;
                };
                if dev.eq_ignore_ascii_case(device.as_str()) {
                    if rev.eq_ignore_ascii_case(revision.as_str()) {
                        self.device_names.insert(card_id, name.to_string());
                        return name.to_string();
                    }
                }
            }

            tracing::info!("could not find a match for gpu name, returning default");
            self.device_names.insert(card_id, DEFAULT_NAME.to_string());
            return DEFAULT_NAME.to_string();
        };
        name.clone()
    }
}

fn get_memory_capacity(path: &Path) -> anyhow::Result<u64> {
    let memory_path = path.join("device/mem_info_vram_total");
    sysfs::read_u64(&memory_path).ok_or_else(|| anyhow::anyhow!("could not read total vram"))
}

fn get_memory_usage(path: &Path) -> anyhow::Result<u64> {
    let memory_path = path.join("device/mem_info_vram_used");
    sysfs::read_u64(&memory_path).ok_or_else(|| anyhow::anyhow!("could not read current vram"))
}

fn get_gtt_capacity(path: &Path) -> anyhow::Result<u64> {
    let gtt_path = path.join("device/mem_info_gtt_total");
    sysfs::read_u64(&gtt_path).ok_or_else(|| anyhow::anyhow!("could not read total gtt"))
}

fn get_gtt_usage(path: &Path) -> anyhow::Result<u64> {
    let gtt_path = path.join("device/mem_info_gtt_used");
    sysfs::read_u64(&gtt_path).ok_or_else(|| anyhow::anyhow!("could not read current gtt"))
}

fn populate_clocks(clocks: &mut Vec<super::Clock>, path: &Path) -> anyhow::Result<()> {
    for clock in clocks.iter_mut() {
        let Some(identifier) = clock.identifier.as_ref() else {
            continue;
        };
        let max_freq = match identifier.domain() {
            super::ClockDomain::Graphics => {
                let Some(gfxclk) = sysfs::read_string(&path.join("device/pp_dpm_sclk")) else {
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
                let Some(vclk) = sysfs::read_string(&path.join("device/pp_dpm_vclk")) else {
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
                let Some(dclk) = sysfs::read_string(&path.join("device/pp_dpm_dclk")) else {
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
                let Some(socclk) = sysfs::read_string(&path.join("device/pp_dpm_socclk")) else {
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
                let Some(mclk) = sysfs::read_string(&path.join("device/pp_dpm_mclk")) else {
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
    Ok(())
}

fn populate_power(power: Option<&mut super::Power>, path: &Path) -> anyhow::Result<()> {
    let Some(power) = power else {
        return Ok(());
    };
    let Some(power1_cap) = sysfs::first_hwmon_subdir(&path.join("device/hwmon"))
        .and_then(|p| sysfs::read_u32(&p.join("power1_cap")))
    else {
        return Ok(());
    };
    power.max_power_mw = power1_cap;
    Ok(())
}

fn populate_thermal(thermals: &mut Vec<super::Thermal>, path: &Path) -> anyhow::Result<()> {
    let Some(hwmon) = sysfs::first_hwmon_subdir(&path.join("device/hwmon")) else {
        return Ok(());
    };
    for thermal in thermals.iter_mut() {
        let Some(temp) = (match thermal.location() {
            super::ThermalLocation::Edge => sysfs::read_u32(&hwmon.join("temp1_crit")),
            super::ThermalLocation::Hotspot => sysfs::read_u32(&hwmon.join("temp2_crit")),
            super::ThermalLocation::Memory => sysfs::read_u32(&hwmon.join("temp3_crit")),
            _ => continue,
        }) else {
            continue;
        };
        thermal.max_celsius = temp / 1000;
    }
    Ok(())
}
