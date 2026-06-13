/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Contains helper functions for reading from /sys files.
use std::os::fd::{BorrowedFd, OwnedFd};

use rustix::fd::AsFd;
use rustix::fs::{Mode, OFlags};

pub fn read_bin(fd: BorrowedFd) -> Option<Vec<u8>> {
    let mut buf = Vec::new();
    let chunk_size = 4096;

    for _ in 0..4096 {
        let start = buf.len();
        buf.resize(start + chunk_size, 0u8);

        let read_slice = &mut buf[start..];
        match rustix::io::read(fd, read_slice) {
            Ok(0) => {
                buf.truncate(start);
                break;
            }
            Ok(bytes_read) => {
                buf.truncate(start + bytes_read);
            }
            Err(rustix::io::Errno::INTR) => {
                // sys call interrupted, try again
                buf.truncate(start);
                continue;
            }
            Err(e) => {
                tracing::warn!("read_bin: read error: {}", e);
                return None;
            }
        }
    }
    Some(buf)
}

/// Reads a string from a given fd, trimming whitespace and converting to a `String`.
pub fn read_string(fd: BorrowedFd) -> Option<String> {
    read_bin(fd).map(|buf| String::from_utf8_lossy(buf.as_slice()).trim().to_string())
}

/// Reads a string from a given path relative to fd, trimming whitespace and converting to a `String`.
pub fn readat_string(fd: BorrowedFd, path: &str) -> Option<String> {
    rustix::fs::openat(fd, path, OFlags::RDONLY | OFlags::CLOEXEC, Mode::empty())
        .ok()
        .and_then(|fd| read_string(fd.as_fd()))
}

/// Reads a string from a given path, trimming whitespace and converting to a `String`.
pub fn read_string_path<P: rustix::path::Arg>(path: P) -> Option<String> {
    rustix::fs::open(path, OFlags::RDONLY | OFlags::CLOEXEC, Mode::empty())
        .ok()
        .and_then(|fd| read_string(fd.as_fd()))
}

/// Reads a 32-bit unsigned integer from a given fd.
pub fn read_u32(fd: BorrowedFd) -> Option<u32> {
    read_string(fd).and_then(|s| s.parse::<u32>().ok())
}

/// Reads a 32-bit unsigned integer from a given path relative to fd.
pub fn readat_u32(fd: BorrowedFd, path: &str) -> Option<u32> {
    readat_string(fd, path).and_then(|s| s.parse::<u32>().ok())
}

/// Reads a 32-bit unsigned integer from a given path.
pub fn read_u32_path<P: rustix::path::Arg>(path: P) -> Option<u32> {
    read_string_path(path).and_then(|s| s.parse::<u32>().ok())
}

/// Reads a 64-bit unsigned integer from a given fd.
pub fn read_u64(fd: BorrowedFd) -> Option<u64> {
    read_string(fd).and_then(|s| s.parse::<u64>().ok())
}

/// Reads a 64-bit unsigned integer from a given path relative to fd.
pub fn readat_u64(fd: BorrowedFd, path: &str) -> Option<u64> {
    readat_string(fd, path).and_then(|s| s.parse::<u64>().ok())
}

#[allow(dead_code)]
/// Reads a 64-bit unsigned integer from a given path.
pub fn read_u64_path<P: rustix::path::Arg>(path: P) -> Option<u64> {
    read_string_path(path).and_then(|s| s.parse::<u64>().ok())
}

#[allow(dead_code)]
/// Reads a hexadecimal value from a given fd, converting it to a `u64`.
pub fn read_hex(fd: BorrowedFd) -> Option<u64> {
    read_string(fd)
        .as_ref()
        .and_then(|s| s.strip_prefix("0x"))
        .and_then(|s| u64::from_str_radix(&s, 16).ok())
}

#[allow(dead_code)]
/// Reads a hexadecimal value from a given path relative to fd, converting it to a `u64`.
pub fn readat_hex(fd: BorrowedFd, path: &str) -> Option<u64> {
    readat_string(fd, path)
        .as_ref()
        .and_then(|s| s.strip_prefix("0x"))
        .and_then(|s| u64::from_str_radix(&s, 16).ok())
}

#[allow(dead_code)]
/// Reads a hexadecimal value from a given path, converting it to a `u64`.
pub fn read_hex_path<P: rustix::path::Arg>(path: P) -> Option<u64> {
    read_string_path(path)
        .as_ref()
        .and_then(|s| s.strip_prefix("0x"))
        .and_then(|s| u64::from_str_radix(&s, 16).ok())
}

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
    read_u32(fd).map(|milli| milli as f32 / 1000.0)
}

/// Reads a temperature from a given hwmon path relative to fd, converting from millidegrees Celsius to degrees Celsius.
pub fn readat_hwmon_temp(fd: BorrowedFd, path: &str) -> Option<f32> {
    readat_u32(fd, path).map(|milli| milli as f32 / 1000.0)
}

#[allow(dead_code)]
/// Reads a temperature from a given hwmon path, converting from millidegrees Celsius to degrees Celsius.
pub fn read_hwmon_temp_path<P: rustix::path::Arg>(path: P) -> Option<f32> {
    read_u32_path(path).map(|milli| milli as f32 / 1000.0)
}

/// Reads a power value from a given hwmon fd, converting from microwatts to watts.
pub fn read_hwmon_power(fd: BorrowedFd) -> Option<f32> {
    // hwmon power is in microwatts
    read_u64(fd)
        .map(|uw| uw as f64 / 1_000_000.0)
        .map(|w| w as f32)
}

#[allow(dead_code)]
/// Reads a power value from a given hwmon path relative to fd, converting from microwatts to watts.
pub fn readat_hwmon_power(fd: BorrowedFd, path: &str) -> Option<f32> {
    readat_u64(fd, path)
        .map(|uw| uw as f64 / 1_000_000.0)
        .map(|w| w as f32)
}

#[allow(dead_code)]
/// Reads a power value from a given hwmon path, converting from microwatts to watts.
pub fn read_hwmon_power_path<P: rustix::path::Arg>(path: P) -> Option<f32> {
    read_u64_path(path)
        .map(|uw| uw as f64 / 1_000_000.0)
        .map(|w| w as f32)
}

/// Returns the first hwmon subdirectory under a given path.
pub fn first_hwmon_subdir(hwmon_parent: BorrowedFd) -> Option<OwnedFd> {
    let mut dir_stream = rustix::fs::Dir::read_from(hwmon_parent).ok()?;
    dir_stream.next().and_then(|e| {
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
    let mut dir_stream = rustix::fs::Dir::read_from(rel_fd.as_fd()).ok()?;
    dir_stream.next().and_then(|e| {
        let e = e.ok()?;
        rustix::fs::openat(
            rel_fd.as_fd(),
            format!("{}/{}", path, e.file_name().to_string_lossy()),
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .ok()
    })
}

/// Returns the first hwmon subdirectory underr a given path.
pub fn first_hwmon_subdir_path<P: rustix::path::Arg>(path: P) -> Option<OwnedFd> {
    let fd = rustix::fs::open(
        path,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .ok()?;
    let mut dir_stream = rustix::fs::Dir::read_from(fd.as_fd()).ok()?;
    dir_stream.next().and_then(|e| {
        let e = e.ok()?;
        rustix::fs::openat(
            fd,
            e.file_name().to_string_lossy(),
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .ok()
    })
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
        let entry_fd = rustix::fs::openat(
            &driver,
            entry.file_name().to_string_lossy().to_string(),
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
