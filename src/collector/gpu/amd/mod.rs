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
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}
const DEFAULT_NAME: &'static str = "AMD Radeon Graphics";

impl Collector {
    pub fn new() -> Self {
        tracing::debug!("[gpu/amd] initializing AMD GPU collector");
        Collector {
            device_names: HashMap::new(),
        }
    }

    pub fn collect(&mut self, path: &Path) -> anyhow::Result<super::Gpu> {
        let gpu_metrics_path = path.join("device/gpu_metrics");
        let metrics = gpu_metrics::GpuMetrics::read(&gpu_metrics_path)?;

        let brand_name = self.get_brand_name(path);
        let kernel_driver = String::from("amdgpu");
        let opengl_driver = String::new();
        let vulkan_driver = String::new();

        let graphics_utilization = metrics.graphics_utilization();
        let graphics_clock = metrics.graphics_clock();
        let memory_capacity = match get_memory_capacity(path) {
            Ok(mem) => mem,
            Err(e) => {
                tracing::warn!("{e}");
                0u64
            }
        };
        let memory_usage = match get_memory_usage(path) {
            Ok(mem) => mem,
            Err(e) => {
                tracing::warn!("{e}");
                0u64
            }
        };
        let memory_clock = metrics.memory_clock();
        let encoder_utilization = metrics.video_enc_dec_util();
        let decoder_utilization = metrics.video_enc_dec_util();
        let encoder_clock = metrics.encoder_clock();
        let decoder_clock = metrics.decoder_clock();
        let power_milliwatt = metrics.power_milliwatt();
        let temperature = metrics.temperature();

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
            kernel_driver,
            opengl_driver,
            vulkan_driver,
            graphics_utilization,
            graphics_clock,
            memory_capacity,
            memory_usage,
            memory_clock,
            encoder_utilization,
            decoder_utilization,
            encoder_clock,
            decoder_clock,
            power_milliwatt,
            temperature,
            processes,
        })
    }

    fn get_fdinfo(&mut self) -> anyhow::Result<Vec<super::Process>> {
        Ok(vec![])
    }

    fn get_brand_name(&mut self, path: &Path) -> String {
        let Some(card_id) = path
            .file_name()
            .map(|card| card.to_string_lossy().to_string())
        else {
            tracing::info!("[gpu/amd] could not get card name!");
            return DEFAULT_NAME.to_string();
        };

        let Some(name) = self.device_names.get(&card_id) else {
            let Some(device) = sysfs::read_string(&path.join("device/device"))
                .and_then(|dev| dev.strip_prefix("0x").map(|dev| dev.to_string()))
                .map(|dev| dev.to_string())
            else {
                tracing::info!("[gpu/amd] could not get device id file!");
                self.device_names.insert(card_id, DEFAULT_NAME.to_string());
                return DEFAULT_NAME.to_string();
            };

            let Some(revision) = sysfs::read_string(&path.join("device/revision"))
                .and_then(|dev| dev.strip_prefix("0x").map(|dev| dev.to_string()))
                .map(|dev| dev.to_string())
            else {
                tracing::info!("[gpu/amd] could not get revision id file!");
                self.device_names.insert(card_id, DEFAULT_NAME.to_string());
                return DEFAULT_NAME.to_string();
            };

            let Some(amdgpu_ids) =
                sysfs::read_string(&PathBuf::from("/usr/share/libdrm/amdgpu.ids"))
            else {
                tracing::info!("[gpu/amd] could not open amdgpu.ids file!");
                self.device_names.insert(card_id, DEFAULT_NAME.to_string());
                return DEFAULT_NAME.to_string();
            };

            tracing::info!("[gpu/amd] checking amdgpu.ids file for {device} {revision}");

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

            tracing::info!("[gpu/amd] could not find a match for gpu name, returning default");
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
