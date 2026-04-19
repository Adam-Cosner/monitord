/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
use crate::collector::helpers::sysfs;
use crate::metrics::gpu::*;
use rustix::ioctl::{IoctlOutput, Opcode};
use std::marker::PhantomData;
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

const DRM_IOCTL_BASE: u32 = 'd' as u32;
const _IOC_NRBITS: u32 = 8;
const _IOC_TYPEBITS: u32 = 8;
const _IOC_SIZEBITS: u32 = 14;
const _IOC_DIRBITS: u32 = 2;

const _IOC_NRMASK: u32 = (1 << _IOC_NRBITS) - 1;
const _IOC_TYPEMASK: u32 = (1 << _IOC_TYPEBITS) - 1;
const _IOC_SIZEMASK: u32 = (1 << _IOC_SIZEBITS) - 1;
const _IOC_DIRMASK: u32 = (1 << _IOC_DIRBITS) - 1;

const _IOC_NRSHIFT: u32 = 0;
const _IOC_TYPESHIFT: u32 = _IOC_NRSHIFT + _IOC_NRBITS;
const _IOC_SIZESHIFT: u32 = _IOC_TYPESHIFT + _IOC_TYPEBITS;
const _IOC_DIRSHIFT: u32 = _IOC_SIZESHIFT + _IOC_SIZEBITS;

const _IOC_NONE: u32 = 0;
const _IOC_WRITE: u32 = 1;
const _IOC_READ: u32 = 2;

macro_rules! _IOC {
    ($dir:expr, $ty:expr, $nr:expr, $size:expr) => {
        ((($dir) << _IOC_DIRSHIFT)
            | (($ty) << _IOC_TYPESHIFT)
            | (($nr) << _IOC_NRSHIFT)
            | (($size) << _IOC_SIZESHIFT))
    };
}

macro_rules! _IOWR {
    ($ty:expr, $nr:expr, $argtype:ty) => {
        _IOC!(
            _IOC_READ | _IOC_WRITE,
            $ty,
            $nr,
            size_of::<$argtype>() as u32
        )
    };
}

macro_rules! DRM_IOWR {
    ($nr:expr, $ty:ty) => {
        _IOWR!(DRM_IOCTL_BASE, $nr, $ty)
    };
}

const DRM_COMMAND_BASE: u32 = 0x40;
const DRM_I915_QUERY: u32 = 0x39;

const DRM_IOCTL_I915_QUERY: u32 = DRM_IOWR!(DRM_COMMAND_BASE + DRM_I915_QUERY, DrmI915Query);

unsafe impl rustix::ioctl::Ioctl for DrmI915Query {
    type Output = ();
    const IS_MUTATING: bool = true;

    fn opcode(&self) -> Opcode {
        DRM_IOCTL_I915_QUERY
    }

    fn as_ptr(&mut self) -> *mut c_void {
        self as *mut _ as *mut c_void
    }

    unsafe fn output_from_ptr(
        out: IoctlOutput,
        extract_output: *mut c_void,
    ) -> rustix::io::Result<Self::Output> {
        tracing::info!("ioctl output: {}", out);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct DrmI915MemoryRegionInfo {
    pub region: MemoryRegion,
    pub rsvd0: u32,
    pub probed_size: u64,
    pub unallocated_size: u64,
    pub union_data: UnionData,
}

#[derive(Clone, Copy)]
#[repr(C)]
struct FAM<T>(PhantomData<T>, [T; 0]);

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct DrmI915QueryMemoryRegions {
    num_regions: u32,
    rsvd: [u32; 3],
    regions: FAM<DrmI915MemoryRegionInfo>,
}

impl<T> FAM<T> {
    #[inline]
    const fn as_ptr(&self) -> *const T {
        self as *const _ as *const T
    }

    #[inline]
    unsafe fn flex_ref(&self, len: usize) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.as_ptr(), len) }
    }
}

impl<T> core::fmt::Debug for FAM<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("FlexibleArrayMember").finish()
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct MemoryRegion {
    pub memory_class: u16,
    pub memory_instance: u16,
}

#[derive(Clone, Copy)]
#[repr(C)]
union UnionData {
    pub rsvd1: [u64; 8],
    pub visible_sizes: VisibleSizes,
}

impl core::fmt::Debug for UnionData {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("UnionData").finish()
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct VisibleSizes {
    pub probed_cpu_visible_size: u64,
    pub unallocated_cpu_visible_size: u64,
}

#[allow(unused_assignments)]
fn memory(path: &Path, fd: BorrowedFd) -> Vec<Memory> {
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

    // Get the length of the MemoryRegionInfo
    let Ok(()) = (unsafe { rustix::ioctl::ioctl(fd, q).inspect_err(|e| tracing::error!("{e}")) })
    else {
        return Vec::new();
    };
    let mut bytes = Vec::new();
    bytes.resize(item.length as usize, 0u8);

    // Query the MemoryRegionInfo into bytes
    let q = DrmI915Query {
        num_items: 1,
        flags: 0,
        items_ptr: &mut item as *mut _ as u64,
    };
    item.data_ptr = bytes.as_mut_ptr() as u64;

    let Ok(()) = (unsafe { rustix::ioctl::ioctl(fd, q) }) else {
        return Vec::new();
    };

    let region_info = unsafe {
        std::mem::transmute::<*const u8, *const DrmI915QueryMemoryRegions>(bytes.as_ptr())
    };

    if region_info.is_null() {
        return Vec::new();
    }

    let region_info = unsafe { &*region_info };

    let regions = unsafe {
        region_info
            .regions
            .flex_ref(region_info.num_regions as usize)
    };

    regions
        .iter()
        .map(|region| {
            Memory {
                // TODO: I think region might say whether it's Gtt or VRAM?
                r#type: MemoryType::Gtt as i32,
                total_memory: region.probed_size,
                used_memory: region.probed_size - region.unallocated_size,
            }
        })
        .collect()
}

fn power(path: &Path) -> Option<Power> {
    None
}

fn thermals(path: &Path) -> Vec<Thermal> {
    Vec::new()
}
