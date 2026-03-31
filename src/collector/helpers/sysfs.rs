/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Contains helper functions for reading from /sys files.

use std::path::{Path, PathBuf};

/// Reads a string from a given path, trimming whitespace and converting to a `String`.
pub fn read_string(path: &Path) -> Option<String> {
    std::fs::read_to_string(path)
        .map(|s| s.trim().to_string())
        .ok()
}

/// Reads a 32-bit unsigned integer from a given path.
pub fn read_u32(path: &Path) -> Option<u32> {
    read_string(path).and_then(|s| s.parse::<u32>().ok())
}

/// Reads a 64-bit unsigned integer from a given path.
pub fn read_u64(path: &Path) -> Option<u64> {
    read_string(path).and_then(|s| s.parse::<u64>().ok())
}

/// Reads a hexadecimal value from a given path, converting it to a `u64`.
pub fn read_hex(path: &Path) -> Option<u64> {
    read_string(path)
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

/// Reads a temperature from a given hwmon path, converting from millidegrees Celsius to degrees Celsius.
pub fn read_hwmon_temp(path: &Path) -> Option<f32> {
    // hwmon temperatures are in millidegrees Celsius
    read_u32(path).map(|milli| milli as f32 / 1000.0)
}

/// Reads a power value from a given hwmon path, converting from microwatts to watts.
pub fn read_hwmon_power(path: &Path) -> Option<f32> {
    // hwmon power is in microwatts
    read_u64(path)
        .map(|uw| uw as f64 / 1_000_000.0)
        .map(|w| w as f32)
}

/// Returns the first hwmon subdirectory under a given path.
pub fn first_hwmon_subdir(hwmon_parent: &Path) -> Option<PathBuf> {
    std::fs::read_dir(hwmon_parent)
        .ok()?
        .flatten()
        .find(|e| e.file_name().to_string_lossy().starts_with("hwmon"))
        .map(|e| e.path())
}

/// Finds the hwmon path for a given PCI driver name.
pub fn find_pci_driver_hwmon(driver_name: &str) -> Option<PathBuf> {
    let driver_path = PathBuf::from(format!("/sys/bus/pci/drivers/{driver_name}"));
    for entry in std::fs::read_dir(&driver_path).ok()?.flatten() {
        let path = entry.path();
        if let Some(hwmon) = first_hwmon_subdir(&path.join("hwmon")) {
            return Some(hwmon);
        }
    }
    None
}
