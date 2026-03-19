use std::path::PathBuf;

/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
use procfs::CurrentSI;

use crate::collector::helpers::sysfs;

pub struct Tracker {
    last: Option<procfs::KernelStats>,
}

#[derive(Debug, Clone)]
pub struct Sample {
    pub per_core: Vec<Utilization>,
}

#[derive(Debug, Clone)]
pub struct Utilization {
    pub usage: f32,
    pub cur_freq_mhz: u32,
}

impl Tracker {
    pub fn new() -> Self {
        Self { last: None }
    }

    pub fn sample(&mut self) -> anyhow::Result<Sample> {
        let stat = procfs::KernelStats::current()?;
        let mut per_core = Vec::with_capacity(stat.cpu_time.len());
        if let Some(last) = self.last.take() {
            for i in 0..stat.cpu_time.len() {
                let usage = diff_stats(i, &last, &stat);
                let cur_freq_mhz = get_cur_freq_mhz(i);
                per_core.push(Utilization {
                    usage,
                    cur_freq_mhz,
                })
            }
        }
        self.last = Some(stat);
        Ok(Sample { per_core })
    }
}

fn diff_stats(
    cpu_idx: usize,
    last_stat: &procfs::KernelStats,
    cur_stat: &procfs::KernelStats,
) -> f32 {
    let Some(cur) = cur_stat.cpu_time.get(cpu_idx) else {
        return 0.0;
    };
    let Some(last) = last_stat.cpu_time.get(cpu_idx) else {
        return 0.0;
    };
    let (active_cur, total_cur) = cpu_times(cur);
    let (active_last, total_last) = cpu_times(last);
    ((active_cur - active_last) as f32 / (total_cur - total_last) as f32) * 100.0
}

// Active, total
fn cpu_times(time: &procfs::CpuTime) -> (u64, u64) {
    let active = time.user
        + time.nice
        + time.system
        + time.irq.unwrap_or(0)
        + time.softirq.unwrap_or(0)
        + time.steal.unwrap_or(0);
    (active, active + time.idle + time.iowait.unwrap_or(0))
}

fn get_cur_freq_mhz(cpu_idx: usize) -> u32 {
    sysfs::read_u32(&PathBuf::from(format!(
        "/sys/devices/system/cpu/cpu{cpu_idx}/cpufreq/scaling_cur_freq"
    )))
    .unwrap_or(0)
        / 1000
}
