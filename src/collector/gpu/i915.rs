/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::collector::helpers::sysfs;
use crate::metrics::gpu::*;
use std::path::PathBuf;

use rustix::fd::{AsFd, OwnedFd};

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
        let Some(current_frequency_mhz) =
            sysfs::readat_u32(self.card_fd.as_fd(), "gt_cur_freq_mhz")
        else {
            return Vec::new();
        };

        let Some(max_frequency_mhz) = sysfs::readat_u32(self.card_fd.as_fd(), "gt_max_freq_mhz")
        else {
            return Vec::new();
        };
        vec![Clock {
            identifier: Some(ClockIdentifier {
                domain: ClockDomain::Gt as i32,
                index: 0,
            }),
            current_frequency_mhz,
            max_frequency_mhz,
        }]
    }

    #[allow(unused_assignments)]
    fn memory(&self) -> Vec<Memory> {
        let mut query_item = drm_i915::QueryItem {
            query_id: 4, // DRM_I915_QUERY_MEMORY_REGIONS
            length: 0,   // will be filled in by ioctl
            flags: 0,
            data_ptr: 0, // must wait for length to be gotten before providing a data pointer to receive
        };

        let query = drm_i915::Query {
            num_items: 1,
            flags: 0,
            items_ptr: &mut query_item as *mut _ as u64,
        };
        let Ok(query) = unsafe { rustix::ioctl::ioctl(self.render_node_fd.as_fd(), query) }
            .inspect_err(|e| tracing::error!("failed to get i915 memory regions length: {e}"))
        else {
            return Vec::new();
        };

        // Now that query_item.length is populated, we can reserve a buffer
        let mut bytes = Vec::new();
        bytes.resize(query_item.length as usize, 0u8);
        query_item.data_ptr = bytes.as_mut_ptr() as u64;
        let Ok(_) = unsafe { rustix::ioctl::ioctl(self.render_node_fd.as_fd(), query) }
            .inspect_err(|e| tracing::error!("failed to get i915 memory regions data: {e}"))
        else {
            return Vec::new();
        };

        // Now we transmute the bytes pointer to a QueryMemoryRegions struct
        let Some(region_info) = core::ptr::NonNull::new(unsafe {
            std::mem::transmute::<*const u8, *mut drm_i915::QueryMemoryRegions>(bytes.as_ptr())
        })
        .map(|ptr| unsafe { ptr.as_ref() }) else {
            return Vec::new();
        };

        // Now we finally extract the MemoryRegionInfo values
        let regions = unsafe {
            region_info
                .regions
                .flex_ref(region_info.num_regions as usize)
        };

        regions
            .iter()
            .map(|region| Memory {
                r#type: if region.memory_class == 0 {
                    MemoryType::System as i32
                } else {
                    MemoryType::Vram as i32
                },
                total_memory: region.probed_size,
                used_memory: region.probed_size - region.unallocated_size,
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

    fn collect(&mut self, config: &super::Config) -> anyhow::Result<super::Gpu> {
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
                name: "i915".to_string(),
                version: None,
            }),
            opengl: None,
            vulkan: None,
        });
        gpu.engines = Vec::new();
        gpu.clocks = config.clocks.then(|| self.clocks()).unwrap_or_default();
        gpu.memory = config.memory.then(|| self.memory()).unwrap_or_default();
        gpu.power = config.power.then(|| self.power()).unwrap_or_default();
        gpu.thermals = config.thermals.then(|| self.thermals()).unwrap_or_default();

        Ok(gpu)
    }

    fn resolve(
        &mut self,
        _staging: &crate::collector::staging::Staging,
        _output: super::Gpu,
    ) -> anyhow::Result<super::Gpu> {
        todo!()
    }

    fn primary_node(&self) -> String {
        self.primary_node.to_string_lossy().to_string()
    }
}

mod drm_i915 {
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

    /// The part at which driver-specific drm commands begin
    const DRM_COMMAND_BASE: u32 = 0x40;

    /// The offset of the query command
    const DRM_I915_QUERY: u32 = 0x39;

    /// The query command for drm ioctl
    const DRM_IOCTL_I915_QUERY: u32 = drm_iowr!(DRM_COMMAND_BASE + DRM_I915_QUERY, Query);

    /// struct drm_i915_query (linux/include/uapi/drm/i915_drm.h)
    /// holds a list of query items
    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct Query {
        pub num_items: u32,
        pub flags: u32,
        pub items_ptr: u64,
    }

    /// struct drm_i915_query_item (linux/include/uapi/drm/i915_drm.h)
    /// a single query to the i915 driver
    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct QueryItem {
        pub query_id: u64,
        pub length: i32,
        pub flags: u32,
        pub data_ptr: u64,
    }

    unsafe impl rustix::ioctl::Ioctl for Query {
        type Output = Self;
        const IS_MUTATING: bool = true;

        fn opcode(&self) -> rustix::ioctl::Opcode {
            DRM_IOCTL_I915_QUERY
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

    /// struct drm_i915_query_memory_regions (linux/include/uapi/drm/i915_drm.h)
    /// memory region list implemented as a flexible array member pattern
    #[repr(C)]
    pub struct QueryMemoryRegions {
        pub num_regions: u32,
        _rsvd: [u32; 3], // reserved
        pub regions: FAM<MemoryRegionInfo>,
    }

    /// Represents a memory region accessible to the GPU
    #[repr(C)]
    pub struct MemoryRegionInfo {
        pub memory_class: u16,
        pub memory_instance: u16,
        _rsvd0: u32,
        pub probed_size: u64,
        pub unallocated_size: u64,
        pub probed_cpu_visible_size: u64,
        pub unallocated_cpu_visible_size: u64,
        _rsvd1: [u64; 6],
    }
}
