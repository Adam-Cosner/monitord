/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::path::PathBuf;

pub fn read_string(path: &PathBuf) -> Option<String> {
    std::fs::read_to_string(path)
        .map(|s| s.trim().to_string())
        .ok()
}

pub fn read_u32(path: &PathBuf) -> Option<u32> {
    read_string(path).and_then(|s| s.parse::<u32>().ok())
}

pub fn read_u64(path: &PathBuf) -> Option<u64> {
    read_string(path).and_then(|s| s.parse::<u64>().ok())
}

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

pub fn get_cpufreq_info(cpu_idx: u32) -> (String, String, Option<String>) {
    let driver = read_string(&PathBuf::from(format!(
        "/sys/devices/system/cpu/cpu{cpu_idx}/cpufreq/scaling_driver"
    )))
    .unwrap_or_default();
    let governor = read_string(&PathBuf::from(format!(
        "/sys/devices/system/cpu/cpu{cpu_idx}/cpufreq/scaling_governor"
    )))
    .unwrap_or_default();
    let mode = match driver.as_str() {
        "intel_pstate" => read_string(&PathBuf::from(
            "/sys/devices/system/cpu/intel_pstate/status",
        )),
        "amd-pstate" | "amd-pstate-epp" => {
            read_string(&PathBuf::from("/sys/devices/system/cpu/amd_pstate/status"))
        }
        _ => None,
    };
    (driver, governor, mode)
}
