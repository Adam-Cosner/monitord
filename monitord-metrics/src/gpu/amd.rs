/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
use anyhow::Context;
use libamdgpu_top::AMDGPU::MetricsInfo;
use std::path::PathBuf;
use std::str::FromStr;

pub(super) struct Collector {
    app: std::collections::HashMap<PathBuf, (libamdgpu_top::app::AppAmdgpuTop, std::time::Instant)>,
}

impl Collector {
    pub fn new() -> Self {
        Collector {
            app: std::collections::HashMap::new(),
        }
    }

    pub fn collect(&mut self, path: &PathBuf) -> anyhow::Result<super::Snapshot> {
        let (app, timestamp) = self.app.entry(path.clone()).or_insert_with(|| {
            let dev_path = path.join("device");
            let device_link = std::fs::read_link(dev_path)
                .expect("Could not read device link, your sysfs is either broken or weird, please report this!")
                .file_name()
                .map(|file| file.to_string_lossy().to_string())
                .unwrap_or_else(|| "".to_string());
            let pci_bus = libamdgpu_top::PCI::BUS_INFO::from_str(device_link.as_str()).expect("Could not parse PCI Bus ID, your sysfs is either broken or weird, please report this!");
            let mut device_path = libamdgpu_top::DevicePath::try_from(pci_bus).expect("Could not get /sys paths from the PCI Bus ID, your sysfs is either broken or weird, please report this!");
            device_path.libdrm_amdgpu = libamdgpu_top::LibDrmAmdgpu::new().ok();
            let handle = device_path
                .init()
                .expect("Could not get device handle from device path, your sysfs is either broken or weird, please report this!");

            (libamdgpu_top::app::AppAmdgpuTop::new(
                handle,
                device_path,
                &libamdgpu_top::app::AppOption::default(),
            )
            .expect("Could not create AMDGPU monitor"), std::time::Instant::now())
        });
        // AMDGPU stuff
        app.update(timestamp.elapsed());

        let brand_name = libamdgpu_top::AMDGPU::find_device_name_or_default(
            app.device_info.ext_info.device_id,
            app.device_info.ext_info.pci_rev,
        );

        let kernel_driver = app
            .get_drm_version_struct()
            .map(|version| {
                format!(
                    "{} {}.{}.{}",
                    version.name,
                    version.version_major,
                    version.version_minor,
                    version.version_patchlevel
                )
            })
            .unwrap_or_else(String::new);

        let graphics_utilization = app.stat.activity.gfx.unwrap_or(0) as f64;
        let graphics_clock = app
            .stat
            .metrics
            .as_ref()
            .map(|metrics| metrics.get_average_gfxclk_frequency().unwrap_or(0))
            .unwrap_or(0) as u32;

        let memory_capacity = app.device_info.memory_info.vram.total_heap_size;
        let memory_usage = app.device_info.memory_info.vram.heap_usage;
        let memory_clock = app
            .stat
            .metrics
            .as_ref()
            .map(|metrics| metrics.get_average_uclk_frequency().unwrap_or(0))
            .unwrap_or(0) as u32;

        let (encoder_utilization, decoder_utilization) = if let Some(metrics) = &app.stat.metrics {
            if let Some(vcn_activity) = metrics.get_all_vcn_activity() {
                // For gpu_metrics v1.4 and v1.5
                let mut sum = 0.0;
                for i in 0..libamdgpu_top::AMDGPU::NUM_VCN {
                    sum += vcn_activity[i as usize] as f64;
                }
                let utilization = sum / libamdgpu_top::AMDGPU::NUM_VCN as f64;
                (utilization, utilization)
            } else if let Some(mm_activity) = metrics.get_average_mm_activity() {
                // For all other versions
                (mm_activity as f64, mm_activity as f64)
            } else {
                (0.0, 0.0)
            }
        } else {
            (0.0, 0.0)
        };

        let (encoder_clock, decoder_clock) = app
            .stat
            .metrics
            .as_ref()
            .map(|metrics| {
                let mut encoder_clock = metrics.get_average_vclk_frequency().unwrap_or(0) as u32;
                if let Some(vclk1) = metrics.get_average_vclk1_frequency() {
                    encoder_clock = (encoder_clock + vclk1 as u32) / 2;
                }
                let mut decoder_clock = metrics.get_average_dclk_frequency().unwrap_or(0) as u32;
                if let Some(dclk1) = metrics.get_average_dclk1_frequency() {
                    decoder_clock = (decoder_clock + dclk1 as u32) / 2;
                }
                (encoder_clock, decoder_clock)
            })
            .unwrap_or((0, 0));

        let power_milliwatt = app
            .stat
            .sensors
            .as_ref()
            .map(|sensors| {
                if let Some(average) = &sensors.average_power {
                    average.value as u32
                } else if let Some(input) = &sensors.input_power {
                    input.value as u32
                } else {
                    0
                }
            })
            .unwrap_or(0);

        let temperature = app
            .stat
            .sensors
            .as_ref()
            .map(|sensors| {
                if let Some(edge) = &sensors.edge_temp {
                    edge.current as i32
                } else {
                    0
                }
            })
            .unwrap_or(0);

        let mut processes = app
            .stat
            .fdinfo
            .proc_usage
            .iter()
            .map(|proc| {
                let usage = &proc.usage;
                super::Process {
                    pid: proc.pid as u32,
                    graphics_utilization: usage.gfx as f64,
                    memory_usage: usage.vram_usage << 10,
                    encode_utilization: if app.stat.fdinfo.has_vcn_unified {
                        usage.vcn_unified as f64
                    } else {
                        usage.enc as f64
                    },
                    decode_utilization: if app.stat.fdinfo.has_vcn_unified {
                        usage.vcn_unified as f64
                    } else {
                        usage.dec as f64
                    },
                }
            })
            .collect();

        *timestamp = std::time::Instant::now();

        Ok(super::Snapshot {
            brand_name,
            kernel_driver,
            opengl_driver: "".to_string(),
            vulkan_driver: "".to_string(),
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
}
