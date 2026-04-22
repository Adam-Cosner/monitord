/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
use crate::collector::helpers::sysfs;
use crate::metrics::gpu::*;
use std::os::fd::BorrowedFd;
use std::path::Path;

pub fn collect(path: &Path, fd: BorrowedFd, config: &Config) -> anyhow::Result<Gpu> {
    tracing::trace!("collecting metrics for i915 device {}", path.display());

    let brand_name = String::from("todo");

    let drivers = config
        .drivers
        .then(|| {
            Some(Drivers {
                kernel: "i915".to_string(),
                ..Default::default()
            })
        })
        .unwrap_or_default();
    let engines = Vec::new();
    let clocks = config.clocks.then(|| clocks(path)).unwrap_or_default();
    let memory = config.memory.then(|| memory(fd)).unwrap_or_default();
    let power = config.power.then(|| power(path)).unwrap_or_default();
    let thermals = config.thermals.then(|| thermals(path)).unwrap_or_default();
    let processes = Vec::new();

    Ok(Gpu {
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

/// i915 only exposes a single "gt" clock and the actual clocks are privately calculated inside the chip firmware
fn clocks(path: &Path) -> Vec<Clock> {
    let Some(current_frequency_mhz) = sysfs::read_u32(&path.join("gt_cur_freq_mhz")) else {
        return Vec::new();
    };

    let Some(max_frequency_mhz) = sysfs::read_u32(&path.join("gt_max_freq_mhz")) else {
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

fn memory(fd: BorrowedFd) -> Vec<Memory> {
    // First we must get the length of the QueryMemoryRegions data
    let mut query_item = drm_i915::QueryItem {
        query_id: 4, // DRM_I915_QUERY_MEMORY_REGIONS
        length: 0,   // will be populated by first ioctl
        flags: 0,
        data_ptr: 0, // must wait for length to be gotten before receiving data
    };
    let query_len = drm_i915::Query {
        num_items: 1,
        flags: 0,
        items_ptr: &mut query_item as *mut _ as u64,
    };
    unsafe {
        let Ok(()) = rustix::ioctl::ioctl(fd, query_len)
            .inspect_err(|e| tracing::error!("failed to get i915 memory regions length: {e}"))
        else {
            return Vec::new();
        };
    }

    // Now query_item.length is populated, we can reserve a buffer
    let mut bytes = Vec::new();
    bytes.resize(query_item.length as usize, 0);
    query_item.data_ptr = bytes.as_mut_ptr() as u64;
    let query_dat = drm_i915::Query {
        num_items: 1,
        flags: 0,
        items_ptr: &mut query_item as *mut _ as u64, // we can reuse query_item
    };
    unsafe {
        let Ok(()) = rustix::ioctl::ioctl(fd, query_dat)
            .inspect_err(|e| tracing::error!("failed to get i915 memory regions data: {e}"))
        else {
            return Vec::new();
        };
    }

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
                MemoryType::Gtt as i32
            } else {
                MemoryType::Vram as i32
            },
            total_memory: region.probed_size,
            used_memory: region.probed_size - region.unallocated_size,
        })
        .collect()
}

fn power(path: &Path) -> Option<Power> {
    // First check for hwmon (only on discrete GPUs)
    std::fs::exists(path.join("device/hwmon"))
        .ok()?
        .then(|| power_hwmon(path))
        .flatten()
}

fn power_hwmon(path: &Path) -> Option<Power> {
    None // i915 uses incrementing counters, todo for refactor
}

fn thermals(path: &Path) -> Vec<Thermal> {
    // First check for hwmon (only on discrete GPUs)
    std::fs::exists(path.join("device/hwmon"))
        .unwrap_or_default()
        .then(|| thermals_hwmon(path))
        .unwrap_or_default()
}

fn thermals_hwmon(path: &Path) -> Vec<Thermal> {
    // Same as above
    Vec::new()
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

    /// struct drm_i915_query_item (linux/include/uapi/drm/i915_drm.h)
    /// a single query to the i915 driver
    #[repr(C)]
    pub struct QueryItem {
        pub query_id: u64,
        pub length: i32,
        pub flags: u32,
        pub data_ptr: u64,
    }

    /// struct drm_i915_query (linux/include/uapi/drm/i915_drm.h)
    /// holds a list of query items
    #[repr(C)]
    pub struct Query {
        pub num_items: u32,
        pub flags: u32,
        pub items_ptr: u64,
    }

    unsafe impl rustix::ioctl::Ioctl for Query {
        type Output = ();
        const IS_MUTATING: bool = true;

        fn opcode(&self) -> rustix::ioctl::Opcode {
            DRM_IOCTL_I915_QUERY
        }

        fn as_ptr(&mut self) -> *mut std::ffi::c_void {
            self as *mut _ as *mut std::ffi::c_void
        }

        unsafe fn output_from_ptr(
            _: rustix::ioctl::IoctlOutput,
            _: *mut std::ffi::c_void,
        ) -> rustix::io::Result<Self::Output> {
            Ok(())
        }
    }

    /// struct drm_i915_query_memory_regions (linux/include/uapi/drm/i915_drm.h)
    /// memory region list implemented as a flexible array member pattern
    #[repr(C)]
    pub struct QueryMemoryRegions {
        pub num_regions: u32,
        _rsvd: [u32; 3], // reserved
        pub regions: fam::FAM<MemoryRegionInfo>,
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
