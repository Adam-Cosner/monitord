/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
use anyhow::Context;
use std::path::PathBuf;

use super::{Process, Snapshot};

pub(super) struct Collector {
    nvml: std::cell::OnceCell<anyhow::Result<nvml_wrapper::Nvml>>,
}

impl Collector {
    pub fn new() -> Self {
        tracing::debug!("Initializing NVIDIA GPU collector");
        Collector {
            nvml: std::cell::OnceCell::new(),
        }
    }

    pub fn collect(&mut self, path: &PathBuf) -> anyhow::Result<super::Snapshot> {
        let driver_path = path.join("device/device/driver");
        if let Ok(driver_link) = std::fs::read_link(driver_path) {
            if driver_link.file_name().is_some_and(|name| name == "nvidia") {
                self.collect_nvidia(path)
            } else {
                self.collect_nouveau(path)
            }
        } else {
            Err(anyhow::anyhow!(
                "Failed to determine NVIDIA GPU driver, this is a bug"
            ))
        }
    }

    // Returns an error if there's an actual NVML error so it can be logged, but if there's no NVML, just return an empty GPU snapshot
    fn collect_nvidia(&mut self, path: &PathBuf) -> anyhow::Result<super::Snapshot> {
        let nvml_bench = std::time::Instant::now();
        tracing::trace!("Collecting metrics for nvidia device {}", path.display());
        let nvml = self.nvml.get_or_init(|| {
            nvml_wrapper::Nvml::init()
                .with_context(|| "Failed to initialize NVML")
                .inspect_err(|err| tracing::error!("{}", err))
        });
        match nvml {
            Ok(nvml) => {
                use nvml_wrapper::enum_wrappers::device::{Clock, TemperatureSensor};

                let brand_name = "NVIDIA".to_string();
                let kernel_driver = nvml.sys_driver_version()?;

                let device_path = path.join("device");
                let device_real = std::fs::read_link(device_path)
                    .map(|device_real| {
                        device_real
                            .file_name()
                            .map(|filename| filename.to_string_lossy().to_string())
                            .unwrap_or_default()
                    })
                    .unwrap_or_default();
                tracing::info!("Checking device_real: {:?}", device_real);
                let device = nvml.device_by_pci_bus_id(device_real)?;

                let graphics_utilization = device
                    .utilization_rates()
                    .map(|util| util.gpu as f64)
                    .unwrap_or_default();
                let graphics_clock = device.clock_info(Clock::Graphics).unwrap_or_default();

                let (memory_capacity, memory_usage) = device
                    .memory_info()
                    .map(|info| (info.total as u64, info.used as u64))
                    .unwrap_or_default();
                let memory_clock = device.clock_info(Clock::Memory).unwrap_or_default();

                let encoder_utilization = device
                    .encoder_utilization()
                    .map(|enc_util| {
                        enc_util.utilization as f64 * 100.0 / enc_util.sampling_period as f64
                    })
                    .unwrap_or_default();

                let decoder_utilization = device
                    .decoder_utilization()
                    .map(|dec_util| {
                        dec_util.utilization as f64 * 100.0 / dec_util.sampling_period as f64
                    })
                    .unwrap_or_default();
                let video_clock = device.clock_info(Clock::Video).unwrap_or_default();

                let power_milliwatt = device.power_usage().unwrap_or_default();

                let temperature = device
                    .temperature(TemperatureSensor::Gpu)
                    .unwrap_or_default() as i32;

                let mut processes = Vec::new();
                for process in device.process_utilization_stats(None).iter().flatten() {
                    let pid = process.pid;
                    let graphics_utilization = process.sm_util as f64;
                    let memory_usage = process.mem_util as u64;
                    let encode_utilization = process.enc_util as f64;
                    let decode_utilization = process.dec_util as f64;

                    processes.push(Process {
                        pid,
                        graphics_utilization,
                        memory_usage,
                        encode_utilization,
                        decode_utilization,
                    })
                }

                tracing::trace!(
                    "Collected metrics for nvidia device {} in {:?}",
                    path.display(),
                    nvml_bench.elapsed()
                );

                Ok(Snapshot {
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
                    encoder_clock: video_clock,
                    decoder_clock: video_clock,
                    power_milliwatt,
                    temperature,
                    processes,
                })
            }
            Err(err) => Err(anyhow::anyhow!(
                "Tried to get NVIDIA GPU metrics with no NVML: {}",
                err
            )),
        }
    }

    fn collect_nouveau(&mut self, path: &PathBuf) -> anyhow::Result<super::Snapshot> {
        tracing::trace!("Collecting metrics for nouveau device {}", path.display());
        Err(anyhow::anyhow!("nouveau not yet implemented"))
    }
}
