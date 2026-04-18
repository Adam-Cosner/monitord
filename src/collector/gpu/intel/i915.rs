/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
use crate::collector::helpers::sysfs;
use crate::metrics::gpu::*;
use rustix::ioctl::{IoctlOutput, Opcode};
use std::os::fd::BorrowedFd;
use std::os::raw::c_void;
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
    let memory = config.memory.then(|| memory(path, fd)).unwrap_or_default();
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

fn memory(path: &Path, fd: BorrowedFd) -> Vec<Memory> {
    #[repr(C)]
    struct DrmI915QueryItem {
        query_id: u64,
        length: i32,
        flags: u32,
        data_ptr: u64,
    }

    #[repr(C)]
    struct DrmI915Query {
        num_items: u32,
        flags: u32,
        items_ptr: u64,
    }

    unsafe impl rustix::ioctl::Ioctl for DrmI915Query {
        type Output = ();
        const IS_MUTATING: bool = true;

        fn opcode(&self) -> Opcode {
            4 // DRM_I915_QUERY_MEMORY_REGIONS
        }

        fn as_ptr(&mut self) -> *mut c_void {
            self as *mut _ as *mut c_void
        }

        unsafe fn output_from_ptr(
            out: IoctlOutput,
            extract_output: *mut c_void,
        ) -> rustix::io::Result<Self::Output> {
            Ok(())
        }
    }

    #[repr(C)]
    pub struct DrmI915MemoryRegionInfo {
        pub region: MemoryRegion,
        pub rsvd0: u32,
        pub probed_size: u64,
        pub unallocated_size: u64,
        pub union_data: UnionData,
    }

    #[repr(C)]
    pub struct MemoryRegion {
        pub memory_class: u16,
        pub memory_instance: u16,
    }

    #[repr(C)]
    pub union UnionData {
        pub rsvd1: [u64; 8],
        pub visible_sizes: VisibleSizes,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct VisibleSizes {
        pub probed_cpu_visible_size: u64,
        pub unallocated_cpu_visible_size: u64,
    }

    let mut item = DrmI915QueryItem {
        query_id: 4, // DRM_I915_QUERY_MEMORY_REGIONS
        length: 0,
        flags: 0,
        data_ptr: 0,
    };
    let q = DrmI915Query {
        num_items: 1,
        flags: 0,
        items_ptr: &mut item as *mut _ as u64,
    };

    // Get the size
    let Ok(()) = (unsafe { rustix::ioctl::ioctl(fd, q) }) else {
        return Vec::new();
    };
    let mut bytes = Vec::new();
    bytes.resize(item.length as usize, 0u8);

    let q = DrmI915Query {
        num_items: 1,
        flags: 0,
        items_ptr: &mut item as *mut _ as u64,
    };

    item.data_ptr = bytes.as_mut_ptr() as u64;
    let Ok(()) = (unsafe { rustix::ioctl::ioctl(fd, q) }) else {
        return Vec::new();
    };

    todo!()
}

fn power(path: &Path) -> Option<Power> {
    todo!()
}

fn thermals(path: &Path) -> Vec<Thermal> {
    todo!()
}
