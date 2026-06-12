/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::collector::helpers::sysfs;
use crate::metrics::gpu::*;
use std::path::PathBuf;

use rustix::fd::{AsFd, OwnedFd};
use rustix::fs::{AtFlags, Mode, OFlags};

pub struct Card {
    card_fd: OwnedFd,
    primary_node: PathBuf,
    render_node: PathBuf,
    render_node_fd: OwnedFd,
}

impl Card {
    pub fn new(fd: OwnedFd) -> anyhow::Result<Self> {
        let drm_subsystem = rustix::fs::openat(
            &fd,
            "device/drm",
            rustix::fs::OFlags::RDONLY
                | rustix::fs::OFlags::DIRECTORY
                | rustix::fs::OFlags::CLOEXEC,
            rustix::fs::Mode::empty(),
        )?;
        let mut primary_node = PathBuf::new();
        let mut render_node_path = PathBuf::new();
        let mut render_node = None;
        for entry in rustix::fs::Dir::read_from(&drm_subsystem)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("card") {
                primary_node = PathBuf::from(format!("/dev/dri/{}", name));
            } else if name.starts_with("renderD") {
                render_node_path = PathBuf::from(format!("/dev/dri/{}", name));
                render_node = Some(rustix::fs::open(
                    format!("/dev/dri/{}", name),
                    rustix::fs::OFlags::RDWR
                        | rustix::fs::OFlags::CLOEXEC
                        | rustix::fs::OFlags::NONBLOCK,
                    rustix::fs::Mode::empty(),
                )?);
            }
        }
        Ok(Self {
            card_fd: fd,
            primary_node,
            render_node: render_node_path,
            render_node_fd: render_node.ok_or_else(|| anyhow::anyhow!("render node not found"))?,
        })
    }

    fn clocks(&self) -> Vec<Clock> {
        let Ok(device_fd) = rustix::fs::openat(
            self.card_fd.as_fd(),
            "device",
            OFlags::RDONLY | OFlags::CLOEXEC | OFlags::DIRECTORY,
            Mode::empty(),
        ) else {
            return Vec::new();
        };
        let mut clocks = Vec::new();
        // XE_MAX_TILES_PER_DEVICE
        for i in 0..2 {
            let Ok(tile_fd) = rustix::fs::openat(
                device_fd.as_fd(),
                format!("tile{}", i),
                OFlags::RDONLY | OFlags::CLOEXEC | OFlags::DIRECTORY,
                Mode::empty(),
            ) else {
                continue;
            };

            let Ok(dir) = rustix::fs::Dir::read_from(tile_fd.as_fd()) else {
                continue;
            };
            for entry in dir {
                let Ok(gt) = entry else {
                    continue;
                };

                let Ok(gt_fd) = rustix::fs::openat(
                    tile_fd.as_fd(),
                    gt.file_name().to_string_lossy(),
                    OFlags::RDONLY | OFlags::CLOEXEC | OFlags::DIRECTORY,
                    Mode::empty(),
                ) else {
                    continue;
                };

                let mut is_graphics = false;
                let mut is_video = false;

                // Check if the render command streamer exists on this gt
                if rustix::fs::statat(gt_fd.as_fd(), "engines/rcs", AtFlags::empty()).is_ok() {
                    is_graphics = true;
                }
                // Check if the video command streamer exists
                if rustix::fs::statat(gt_fd.as_fd(), "engines/vcs", AtFlags::empty()).is_ok() {
                    is_video = true;
                }

                let Some(current_frequency_mhz) =
                    sysfs::readat_u32(gt_fd.as_fd(), "engines/cur_freq")
                else {
                    continue;
                };
                let Some(max_frequency_mhz) = sysfs::readat_u32(gt_fd.as_fd(), "engines/max_freq")
                else {
                    continue;
                };

                clocks.push(Clock {
                    identifier: Some(ClockIdentifier {
                        domain: match (is_graphics, is_video) {
                            (true, true) => ClockDomain::Gt as i32,
                            (true, false) => ClockDomain::Graphics as i32,
                            (false, true) => ClockDomain::VideoUnified as i32,
                            (false, false) => continue,
                        },
                        index: i,
                    }),
                    current_frequency_mhz,
                    max_frequency_mhz,
                })
            }
        }
        clocks
    }

    fn memory(&self) -> Vec<Memory> {
        let query = drm_xe::DeviceQuery {
            extensions: 0,
            query: 1, // DRM_XE_DEVICE_QUERY_MEM_REGIONS
            size: 0,
            data: 0,
            _reserved: [0; 2],
        };

        let Ok(mut query) = unsafe { rustix::ioctl::ioctl(self.render_node_fd.as_fd(), query) }
            .inspect_err(|e| tracing::error!("failed to get xe memory regions: {e}"))
        else {
            return Vec::new();
        };

        // Now that query.size is populated, we can reserve a buffer
        let mut bytes = Vec::new();
        bytes.resize(query.size as usize, 0u8);
        query.data = bytes.as_mut_ptr() as u64;

        let Ok(_) = unsafe { rustix::ioctl::ioctl(self.render_node_fd.as_fd(), query) }
            .inspect_err(|e| tracing::error!("failed to get xe memory regions: {e}"))
        else {
            return Vec::new();
        };

        // Now we transmute the bytes pointer to a drm_xe::QueryItem pointer
        let Some(region_info) = core::ptr::NonNull::new(unsafe {
            std::mem::transmute::<*const u8, *mut drm_xe::MemRegions>(bytes.as_ptr())
        })
        .map(|ptr| unsafe { ptr.as_ref() }) else {
            return Vec::new();
        };

        // Now we extract the MemRegion values
        let regions = unsafe {
            region_info
                .regions
                .flex_ref(region_info.num_mem_regions as usize)
        };

        regions
            .iter()
            .map(|region| Memory {
                r#type: if region.mem_class == 0 {
                    MemoryType::System as i32
                } else {
                    MemoryType::Vram as i32
                },
                total_memory: region.total_size,
                used_memory: region.used,
            })
            .collect()
    }

    fn power(&self) -> Option<Power> {
        let Some(_hwmon_fd) = sysfs::first_hwmon_subdir_at(self.card_fd.as_fd(), "device/hwmon")
        else {
            return None;
        };
        None // I don't have an Arc GPU to figure out where the power file is located
    }

    fn thermals(&self) -> Vec<Thermal> {
        let Some(_hwmon_fd) = sysfs::first_hwmon_subdir_at(self.card_fd.as_fd(), "device/hwmon")
        else {
            return Vec::new();
        };
        Vec::new() // Same as above
    }
}

impl super::Card for Card {
    fn identify(&self) -> (String, String, Option<String>, Option<String>) {
        (
            sysfs::readat_string(self.card_fd.as_fd(), "device/vendor")
                .and_then(|v| v.strip_prefix("0x").map(|v| v.to_string()))
                .map(String::from)
                .unwrap_or_default(),
            sysfs::readat_string(self.card_fd.as_fd(), "device/device")
                .and_then(|d| d.strip_prefix("0x").map(|d| d.to_string()))
                .map(String::from)
                .unwrap_or_default(),
            sysfs::readat_string(self.card_fd.as_fd(), "device/subsystem_vendor")
                .and_then(|sv| sv.strip_prefix("0x").map(|sv| sv.to_string()))
                .map(String::from),
            sysfs::readat_string(self.card_fd.as_fd(), "device/subsystem_device")
                .and_then(|sd| sd.strip_prefix("0x").map(|sd| sd.to_string()))
                .map(String::from),
        )
    }

    fn collect(&mut self, config: &Config) -> anyhow::Result<Gpu> {
        let mut gpu = Gpu::default();
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
        gpu.drivers = config.drivers.then(|| Drivers {
            kernel: Some(KernelDriver {
                name: "xe".to_string(),
                version: None,
            }),
            opengl: None,
            vulkan: None,
        });
        gpu.clocks = config.clocks.then(|| self.clocks()).unwrap_or_default();
        gpu.memory = config.memory.then(|| self.memory()).unwrap_or_default();
        gpu.power = config.power.then(|| self.power()).unwrap_or_default();
        gpu.thermals = config.thermals.then(|| self.thermals()).unwrap_or_default();
        Ok(gpu)
    }
    fn resolve(
        &mut self,
        _staging: &crate::collector::staging::Staging,
        _output: Gpu,
    ) -> anyhow::Result<Gpu> {
        todo!()
    }

    fn primary_node(&self) -> String {
        todo!()
    }
}

mod drm_xe {
    use crate::collector::helpers::{ioctl::*, *};
    use crate::{_ioc, _iowr};

    /// Helper for a read-write DRM ioctl command
    macro_rules! drm_iowr {
        ($nr:expr, $ty:ty) => {
            _iowr!(DRM_IOCTL_BASE, $nr, $ty)
        };
    }

    /// The ioctl directory (drm)
    const DRM_IOCTL_BASE: u32 = 'd' as u32;

    /// The offset at which driver-specific drm commands begin
    const DRM_COMMAND_BASE: u32 = 0x40;

    /// The offset of the device query command
    const DRM_XE_DEVICE_QUERY: u32 = 0x00;

    /// The query command for drm ioctl
    const DRM_IOCTL_XE_DEVICE_QUERY: u32 =
        drm_iowr!(DRM_COMMAND_BASE + DRM_XE_DEVICE_QUERY, DeviceQuery);

    /// struct drm_xe_device_query (linux/include/uapi/drm/xe_drm.h)
    /// Holds a list of device queries
    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct DeviceQuery {
        pub extensions: u64,
        pub query: u32,
        pub size: u32,
        pub data: u64,
        pub _reserved: [u64; 2],
    }

    unsafe impl rustix::ioctl::Ioctl for DeviceQuery {
        type Output = Self;
        const IS_MUTATING: bool = true;

        fn opcode(&self) -> rustix::ioctl::Opcode {
            DRM_IOCTL_XE_DEVICE_QUERY
        }

        fn as_ptr(&mut self) -> *mut std::ffi::c_void {
            self as *mut _ as *mut std::ffi::c_void
        }

        unsafe fn output_from_ptr(
            _: rustix::ioctl::IoctlOutput,
            extract_ptr: *mut std::ffi::c_void,
        ) -> rustix::io::Result<Self::Output> {
            let casted_ptr = core::ptr::NonNull::new(extract_ptr as *mut _ as *mut Self)
                .map(|ptr| unsafe { ptr.as_ref() })
                .ok_or(rustix::io::Errno::FAULT)?;

            Ok(casted_ptr.clone())
        }
    }

    /// struct drm_xe_device_query_mem_regions (linux/include/uapi/drm/xe_drm.h)
    /// Holds memory region information for the device
    #[repr(C)]
    pub struct MemRegions {
        pub num_mem_regions: u32,
        pub _pad: u32,
        pub regions: FAM<MemRegion>,
    }

    /// struct drm_xe_mem_region (linux/include/uapi/drm/xe_drm.h)
    /// Holds memory region information
    #[repr(C)]
    pub struct MemRegion {
        pub mem_class: u16,
        pub instance: u16,
        pub min_page_size: u32,
        pub total_size: u64,
        pub used: u64,
        pub cpu_visible_size: u64,
        pub cpu_visible_used: u64,
        pub _reserved: [u64; 6],
    }
}
