/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::metrics::gpu::*;
use std::{os::fd::OwnedFd, path::PathBuf, rc::Rc};

pub struct Card {
    nvml: Rc<nvml_wrapper::Nvml>,
    pci: String,
    primary_node: PathBuf,
    render_node: PathBuf,
    // todo
}

impl Card {
    pub fn new(fd: OwnedFd, nvml: &Rc<nvml_wrapper::Nvml>) -> anyhow::Result<Self> {
        let pci = PathBuf::from(
            rustix::fs::readlinkat(&fd, "device", Vec::new())?
                .to_string_lossy()
                .to_string(),
        )
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("could not read NVIDIA PCI address"))?
        .to_string_lossy()
        .to_string();
        let mut primary_node = PathBuf::new();
        let mut render_node = PathBuf::new();
        let drm_root = rustix::fs::openat(
            &fd,
            "device/drm",
            rustix::fs::OFlags::DIRECTORY
                | rustix::fs::OFlags::RDONLY
                | rustix::fs::OFlags::CLOEXEC,
            rustix::fs::Mode::empty(),
        )?;
        for entry in rustix::fs::Dir::read_from(&drm_root)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("card") {
                primary_node = PathBuf::from(format!("/dev/dri/{}", name));
            } else if name.starts_with("renderD") {
                render_node = PathBuf::from(format!("/dev/dri/{}", name));
            }
        }
        Ok(Self {
            nvml: nvml.clone(),
            pci,
            primary_node,
            render_node,
        })
    }

    fn engines<'a>(&self, device: &nvml_wrapper::Device<'a>) -> Vec<Engine> {
        vec![
            Engine {
                identifier: Some(EngineIdentifier {
                    r#type: EngineType::EngineType3d as i32,
                    index: 0,
                    clock: Some(ClockIdentifier {
                        domain: ClockDomain::Graphics as i32,
                        index: 0,
                    }),
                }),
                utilization: match device.utilization_rates() {
                    Ok(rates) => rates.gpu as u64,
                    Err(err) => {
                        tracing::warn!("could not get gpu utilization rates: {}", err);
                        return Vec::new();
                    }
                },
            },
            Engine {
                identifier: Some(EngineIdentifier {
                    r#type: EngineType::MemoryController as i32,
                    index: 0,
                    clock: Some(ClockIdentifier {
                        domain: ClockDomain::Memory as i32,
                        index: 0,
                    }),
                }),
                utilization: match device.utilization_rates() {
                    Ok(rates) => rates.memory as u64,
                    Err(err) => {
                        tracing::warn!("could not get memory utilization rates: {}", err);
                        return Vec::new();
                    }
                },
            },
            Engine {
                identifier: Some(EngineIdentifier {
                    r#type: EngineType::VideoEncode as i32,
                    index: 0,
                    clock: Some(ClockIdentifier {
                        domain: ClockDomain::VideoUnified as i32,
                        index: 0,
                    }),
                }),
                utilization: match device.encoder_utilization() {
                    Ok(utilization) => utilization.utilization as u64,
                    Err(err) => {
                        tracing::warn!("could not get encoder utilization: {}", err);
                        return Vec::new();
                    }
                },
            },
            Engine {
                identifier: Some(EngineIdentifier {
                    r#type: EngineType::VideoDecode as i32,
                    index: 0,
                    clock: Some(ClockIdentifier {
                        domain: ClockDomain::VideoUnified as i32,
                        index: 0,
                    }),
                }),
                utilization: match device.decoder_utilization() {
                    Ok(utilization) => utilization.utilization as u64,
                    Err(err) => {
                        tracing::warn!("could not get encoder utilization: {}", err);
                        return Vec::new();
                    }
                },
            },
        ]
    }

    fn clocks<'a>(&self, device: &nvml_wrapper::Device<'a>) -> Vec<Clock> {
        vec![
            Clock {
                identifier: Some(ClockIdentifier {
                    domain: ClockDomain::Graphics as i32,
                    index: 0,
                }),
                current_frequency_mhz: device
                    .clock(
                        nvml_wrapper::enum_wrappers::device::Clock::Graphics,
                        nvml_wrapper::enum_wrappers::device::ClockId::Current,
                    )
                    .unwrap_or_default(),
                max_frequency_mhz: device
                    .max_clock_info(nvml_wrapper::enum_wrappers::device::Clock::Graphics)
                    .unwrap_or_default(),
            },
            Clock {
                identifier: Some(ClockIdentifier {
                    domain: ClockDomain::Compute as i32,
                    index: 0,
                }),
                current_frequency_mhz: device
                    .clock(
                        nvml_wrapper::enum_wrappers::device::Clock::SM,
                        nvml_wrapper::enum_wrappers::device::ClockId::Current,
                    )
                    .unwrap_or_default(),
                max_frequency_mhz: device
                    .max_clock_info(nvml_wrapper::enum_wrappers::device::Clock::SM)
                    .unwrap_or_default(),
            },
            Clock {
                identifier: Some(ClockIdentifier {
                    domain: ClockDomain::Memory as i32,
                    index: 0,
                }),
                current_frequency_mhz: device
                    .clock(
                        nvml_wrapper::enum_wrappers::device::Clock::Memory,
                        nvml_wrapper::enum_wrappers::device::ClockId::Current,
                    )
                    .unwrap_or_default(),
                max_frequency_mhz: device
                    .max_clock_info(nvml_wrapper::enum_wrappers::device::Clock::Memory)
                    .unwrap_or_default(),
            },
            Clock {
                identifier: Some(ClockIdentifier {
                    domain: ClockDomain::VideoUnified as i32,
                    index: 0,
                }),
                current_frequency_mhz: device
                    .clock(
                        nvml_wrapper::enum_wrappers::device::Clock::Video,
                        nvml_wrapper::enum_wrappers::device::ClockId::Current,
                    )
                    .unwrap_or_default(),
                max_frequency_mhz: device
                    .max_clock_info(nvml_wrapper::enum_wrappers::device::Clock::Video)
                    .unwrap_or_default(),
            },
        ]
    }

    // NVML doesn't expose unified values for system memory mapped to the GPU, only ReBAR which is the opposite
    fn memory<'a>(&self, device: &nvml_wrapper::Device<'a>) -> Vec<Memory> {
        vec![Memory {
            r#type: MemoryType::Vram as i32,
            total_memory: match device.memory_info() {
                Ok(info) => info.total,
                Err(_) => return Vec::new(),
            },
            used_memory: match device.memory_info() {
                Ok(info) => info.used,
                Err(_) => return Vec::new(),
            },
        }]
    }

    fn power<'a>(&self, device: &nvml_wrapper::Device<'a>) -> Option<Power> {
        Some(Power {
            current_power_mw: device.power_usage().ok()?,
            max_power_mw: device.power_management_limit().ok()?,
            is_power_throttled: device.current_throttle_reasons().is_ok_and(|reasons| {
                reasons
                    .contains(nvml_wrapper::bitmasks::device::ThrottleReasons::SW_THERMAL_SLOWDOWN)
            }),
            is_thermal_throttled: device.current_throttle_reasons().is_ok_and(|reasons| {
                reasons.contains(nvml_wrapper::bitmasks::device::ThrottleReasons::SW_POWER_CAP)
            }),
        })
    }

    // I have to use the raw bindings since nvml_wrapper doesn't expose the newer thermal settings
    fn thermal<'a>(&self, device: &nvml_wrapper::Device<'a>) -> Vec<Thermal> {
        let mut thermal_settings: nvml_wrapper_sys::bindings::nvmlGpuThermalSettings_t =
            unsafe { std::mem::zeroed() };
        unsafe {
            self.nvml.lib().nvmlDeviceGetThermalSettings(
                device.handle(),
                15,
                &mut thermal_settings as *mut _,
            )
        };

        let mut thermals = Vec::new();

        for i in 0..thermal_settings.count {
            let thermal = &thermal_settings.sensor[i as usize];
            thermals.push(Thermal {
                location: match thermal.target {
                    nvml_wrapper_sys::bindings::nvmlThermalTarget_t_NVML_THERMAL_TARGET_GPU => {
                        ThermalLocation::Edge as i32
                    }
                    nvml_wrapper_sys::bindings::nvmlThermalTarget_t_NVML_THERMAL_TARGET_MEMORY => {
                        ThermalLocation::Memory as i32
                    }
                    nvml_wrapper_sys::bindings::nvmlThermalTarget_t_NVML_THERMAL_TARGET_POWER_SUPPLY => {
                        ThermalLocation::Vrsoc as i32
                    }
                    _ => continue,
                },
                current_celsius: thermal.currentTemp as u32,
                max_celsius: thermal.defaultMaxTemp as u32,
            })
        }

        thermals
    }

    fn processes<'a>(&self, device: &nvml_wrapper::Device<'a>) -> Vec<Process> {
        let mut processes = Vec::new();
        let utilization_stats = match device.process_utilization_stats(None) {
            Ok(stats) => stats,
            Err(_) => return Vec::new(),
        };
        for process in utilization_stats.iter() {
            processes.push(Process {
                pid: process.pid,
                engine_utilization: vec![
                    Engine {
                        identifier: Some(EngineIdentifier {
                            r#type: EngineType::EngineType3d as i32,
                            index: 0,
                            clock: Some(ClockIdentifier {
                                domain: ClockDomain::Graphics as i32,
                                index: 0,
                            }),
                        }),
                        utilization: process.sm_util as u64,
                    },
                    Engine {
                        identifier: Some(EngineIdentifier {
                            r#type: EngineType::VideoEncode as i32,
                            index: 0,
                            clock: Some(ClockIdentifier {
                                domain: ClockDomain::VideoUnified as i32,
                                index: 0,
                            }),
                        }),
                        utilization: process.enc_util as u64,
                    },
                    Engine {
                        identifier: Some(EngineIdentifier {
                            r#type: EngineType::VideoDecode as i32,
                            index: 0,
                            clock: Some(ClockIdentifier {
                                domain: ClockDomain::VideoUnified as i32,
                                index: 0,
                            }),
                        }),
                        utilization: process.dec_util as u64,
                    },
                ],
                vram_usage: process.mem_util as u64,
                gtt_usage: 0,
            });
        }
        processes
    }
}

impl super::Card for Card {
    fn collect(&mut self, config: &Config) -> anyhow::Result<super::Gpu> {
        let mut gpu = Gpu::default();

        let device = self.nvml.device_by_pci_bus_id(self.pci.clone())?;

        gpu.brand_name = device.name().unwrap_or_default();
        gpu.primary_node = self.primary_node.to_string_lossy().to_string();
        gpu.render_node = self.render_node.to_string_lossy().to_string();

        gpu.drivers = config.drivers.then(|| Drivers {
            kernel: "nvidia".to_string(),
            opengl: String::new(),
            vulkan: String::new(),
        });

        gpu.engines = config
            .engines
            .then(|| self.engines(&device))
            .unwrap_or_default();
        gpu.clocks = config
            .clocks
            .then(|| self.clocks(&device))
            .unwrap_or_default();
        gpu.memory = config
            .memory
            .then(|| self.memory(&device))
            .unwrap_or_default();
        gpu.power = config
            .power
            .then(|| self.power(&device))
            .unwrap_or_default();
        gpu.thermals = config
            .thermals
            .then(|| self.thermal(&device))
            .unwrap_or_default();
        gpu.processes = config
            .processes
            .then(|| self.processes(&device))
            .unwrap_or_default();

        Ok(gpu)
    }

    fn resolve(
        &mut self,
        _: &crate::collector::staging::Staging,
        output: super::Gpu,
    ) -> anyhow::Result<super::Gpu> {
        // NVML already fills out all the important details
        Ok(output)
    }

    fn primary_node(&self) -> String {
        self.primary_node.to_string_lossy().to_string()
    }
}
