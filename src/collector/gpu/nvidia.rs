/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub struct Card {
    // todo
}

impl Card {
    pub fn new(path: std::path::PathBuf, nvml: &nvml_wrapper::Nvml) -> anyhow::Result<Self> {
        Ok(Self {})
    }
}

impl super::Card for Card {
    fn collect(&mut self, config: &super::Config) -> anyhow::Result<super::Gpu> {
        todo!()
    }
    fn resolve(
        &mut self,
        staging: &crate::collector::staging::Staging,
        output: super::Gpu,
    ) -> anyhow::Result<super::Gpu> {
        todo!()
    }
}

/*
use anyhow::Context;
use std::path::Path;

use crate::collector::helpers::discovery::Discovery;

pub(super) struct Collector {
    nvml: Discovery<nvml_wrapper::Nvml>,
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector {
    pub fn new() -> Self {
        tracing::debug!("initializing NVIDIA GPU collector");
        Collector {
            nvml: Discovery::default(),
        }
    }

    pub fn collect(&mut self, path: &Path, config: &super::Config) -> anyhow::Result<super::Gpu> {
        let driver_path = path.join("device/driver");
        if let Ok(driver_link) = std::fs::read_link(driver_path) {
            if driver_link.file_name().is_some_and(|name| name == "nvidia") {
                self.collect_nvidia(config, path)
            } else {
                self.collect_nouveau(path)
            }
        } else {
            Err(anyhow::anyhow!(
                "Failed to determine NVIDIA GPU driver, this is a bug"
            ))
        }
    }

    /// On nvml error, logs the error and returns a partial snapshot.
    /// If there's no nvml, an error is returned.
    fn collect_nvidia(
        &mut self,
        config: &super::Config,
        path: &Path,
    ) -> anyhow::Result<super::Gpu> {
        tracing::trace!("collecting metrics for nvidia device {}", path.display());

        let mut gpu = super::Gpu::default();
        let nvml = self
            .nvml
            .require(|| nvml_wrapper::Nvml::init().with_context(|| "failed to initialize nvml"))?;

        let Ok(kernel_driver) = nvml.sys_driver_version() else {
            return Ok(gpu);
        };

        gpu.drivers = if config.drivers {
            Some(super::Drivers {
                kernel: kernel_driver,
                opengl: String::new(),
                vulkan: String::new(),
            })
        } else {
            None
        };

        let device_path = path.join("device");
        let pci_bus_id = std::fs::read_link(device_path)
            .map(|device_link| {
                device_link
                    .file_name()
                    .map(|filename| filename.to_string_lossy().to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default();
        tracing::debug!("getting device from pci bus id: {:?}", pci_bus_id);
        let _nvml_device = nvml.device_by_pci_bus_id(pci_bus_id)?;

        gpu.engines = if config.engines {
            unimplemented!()
        } else {
            Vec::new()
        };

        gpu.clocks = if config.clocks {
            unimplemented!()
        } else {
            Vec::new()
        };

        gpu.memory = if config.memory {
            unimplemented!()
        } else {
            Vec::new()
        };

        gpu.power = if config.power { unimplemented!() } else { None };

        gpu.thermals = if config.thermals {
            unimplemented!()
        } else {
            Vec::new()
        };

        gpu.processes = if config.processes {
            unimplemented!()
        } else {
            Vec::new()
        };

        // let graphics_utilization = device
        //     .utilization_rates()
        //     .map(|util| util.gpu as f64)
        //     .unwrap_or_default();
        // let graphics_clock = device.clock_info(Clock::Graphics).unwrap_or_default();

        // let (memory_capacity, memory_usage) = device
        //     .memory_info()
        //     .map(|info| (info.total, info.used))
        //     .unwrap_or_default();
        // let memory_clock = device.clock_info(Clock::Memory).unwrap_or_default();

        // let encoder_utilization = device
        //     .encoder_utilization()
        //     .map(|enc_util| enc_util.utilization as f64)
        //     .unwrap_or_default();

        // let decoder_utilization = device
        //     .decoder_utilization()
        //     .map(|dec_util| dec_util.utilization as f64)
        //     .unwrap_or_default();
        // let video_clock = device.clock_info(Clock::Video).unwrap_or_default();

        // let power_milliwatt = device.power_usage().unwrap_or_default();

        // let temperature = device
        //     .temperature(TemperatureSensor::Gpu)
        //     .unwrap_or_default() as i32;

        // tracing::trace!(
        //     "collected metrics for nvidia device {} in {:?}",
        //     path.display(),
        //     nvml_bench.elapsed()
        // );

        Ok(gpu)
    }

    /// Currently does nothing as I haven't read up on nouveau's metric reporting if there even is any
    fn collect_nouveau(&mut self, path: &Path) -> anyhow::Result<super::Gpu> {
        tracing::trace!("collecting metrics for nouveau device {}", path.display());
        Err(anyhow::anyhow!("nouveau not yet implemented"))
    }
}

// fn collect_processes(device: &nvml_wrapper::Device) -> Vec<super::Process> {
//     let mut processes = Vec::new();
//     for process in device.process_utilization_stats(None).iter().flatten() {
//         let pid = process.pid;
//         let graphics_utilization = process.sm_util as f64;
//         let memory_usage = process.mem_util as u64;
//         let encode_utilization = process.enc_util as f64;
//         let decode_utilization = process.dec_util as f64;

//         processes.push(unimplemented!())
//     }
//     processes
// }
 */
