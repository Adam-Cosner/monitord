/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Contains helper functions for reading from /sys files.
use rustix::fd::{AsFd, BorrowedFd, OwnedFd};
use rustix::fs::{Mode, OFlags};

/// Counts the number of CPUs in a given CPU list string (e.g. "0-3,5,7-9")
pub fn count_cpu_list(cpu_list: &str) -> Option<u32> {
    let mut count = 0;
    for range in cpu_list.trim().split(',') {
        if let Some((start, end)) = range.split_once('-') {
            count += end.parse::<u32>().ok()? - start.parse::<u32>().ok()? + 1;
        } else {
            count += 1;
        }
    }
    Some(count)
}

#[allow(dead_code)]
/// Reads a temperature from a given hwmon fd, converting from millidegrees Celsius to degrees Celsius.
pub fn read_hwmon_temp(fd: BorrowedFd) -> Option<f32> {
    // hwmon temperatures are in millidegrees Celsius
    crate::helpers::io::read_u32(fd).map(|milli| milli as f32 / 1000.0)
}

/// Reads a temperature from a given hwmon path relative to fd, converting from millidegrees Celsius to degrees Celsius.
pub fn readat_hwmon_temp(fd: BorrowedFd, path: &str) -> Option<f32> {
    crate::helpers::io::readat_u32(fd, path).map(|milli| milli as f32 / 1000.0)
}

#[allow(dead_code)]
/// Reads a temperature from a given hwmon path, converting from millidegrees Celsius to degrees Celsius.
pub fn read_hwmon_temp_path<P: rustix::path::Arg>(path: P) -> Option<f32> {
    crate::helpers::io::read_u32_path(path).map(|milli| milli as f32 / 1000.0)
}

/// Reads a power value from a given hwmon fd, converting from microwatts to watts.
pub fn read_hwmon_power(fd: BorrowedFd) -> Option<f32> {
    // hwmon power is in microwatts
    crate::helpers::io::read_u64(fd)
        .map(|uw| uw as f64 / 1_000_000.0)
        .map(|w| w as f32)
}

#[allow(dead_code)]
/// Reads a power value from a given hwmon path relative to fd, converting from microwatts to watts.
pub fn readat_hwmon_power(fd: BorrowedFd, path: &str) -> Option<f32> {
    crate::helpers::io::readat_u64(fd, path)
        .map(|uw| uw as f64 / 1_000_000.0)
        .map(|w| w as f32)
}

#[allow(dead_code)]
/// Reads a power value from a given hwmon path, converting from microwatts to watts.
pub fn read_hwmon_power_path<P: rustix::path::Arg>(path: P) -> Option<f32> {
    crate::helpers::io::read_u64_path(path)
        .map(|uw| uw as f64 / 1_000_000.0)
        .map(|w| w as f32)
}

/// Returns the first hwmon subdirectory under a given path.
pub fn first_hwmon_subdir(hwmon_parent: BorrowedFd) -> Option<OwnedFd> {
    rustix::fs::Dir::read_from(hwmon_parent)
        .ok()?
        .skip_while(|e| {
            e.as_ref()
                .is_ok_and(|e| e.file_name().to_string_lossy().starts_with("."))
        })
        .next()
        .and_then(|e| {
            let e = e.ok()?;
            rustix::fs::openat(
                hwmon_parent,
                e.file_name().to_string_lossy().to_string(),
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .ok()
        })
}

/// Returns the first hwmon subdirectory under a given path relative to fd.
pub fn first_hwmon_subdir_at(fd: BorrowedFd, path: &str) -> Option<OwnedFd> {
    let rel_fd = rustix::fs::openat(
        fd,
        path,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .ok()?;
    first_hwmon_subdir(rel_fd.as_fd())
}

/// Returns the first hwmon subdirectory underr a given path.
pub fn first_hwmon_subdir_path<P: rustix::path::Arg>(path: P) -> Option<OwnedFd> {
    let fd = rustix::fs::open(
        path,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .ok()?;
    first_hwmon_subdir(fd.as_fd())
}

/// Opens the first hwmon subdirectory for a given PCI driver name.
pub fn find_pci_driver_hwmon(driver_name: &str) -> Option<OwnedFd> {
    let driver = rustix::fs::open(
        format!("/sys/bus/pci/drivers/{driver_name}"),
        OFlags::RDONLY | OFlags::CLOEXEC | OFlags::DIRECTORY,
        Mode::empty(),
    )
    .ok()?;
    let driver_dir_stream = rustix::fs::Dir::read_from(&driver).ok()?;
    for entry in driver_dir_stream.flatten() {
        let entry_name = entry.file_name().to_string_lossy().to_string();
        if entry_name.starts_with(".") {
            continue;
        }
        let entry_fd = rustix::fs::openat(
            &driver,
            entry_name,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .ok()?;
        if let Some(hwmon) = first_hwmon_subdir(entry_fd.as_fd()) {
            return Some(hwmon);
        }
    }
    None
}
