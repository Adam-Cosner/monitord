use std::path::PathBuf;

/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
use procfs::CurrentSI;

use crate::collector::helpers::{
    sampler::{Differential, Sampler},
    *,
};

pub struct Tracker {
    sampler: Sampler<procfs::KernelStats>,
}

impl Tracker {
    pub fn new() -> Self {
        Self {
            sampler: Sampler::new(),
        }
    }

    pub fn sample(&mut self) -> anyhow::Result<Vec<Utilization>> {
        match procfs::KernelStats::current() {
            Ok(stat) => match self.sampler.push(stat) {
                Some(delta) => Ok(delta.change),
                None => Ok(Vec::new()),
            },
            Err(e) => {
                tracing::warn!("[CPU] failed to read /proc/stat: {}", e);
                Ok(Vec::new())
            }
        }
    }
}

impl Differential for procfs::KernelStats {
    type Delta = Vec<Utilization>;

    fn delta(&self, other: &Self) -> Self::Delta {
        let mut per_core = Vec::with_capacity(self.cpu_time.len());
        for i in 0..other.cpu_time.len() {
            let usage = diff_stats(i, other, self);
            let cur_freq_mhz = get_cur_freq_mhz(i);
            per_core.push(Utilization {
                usage,
                cur_freq_mhz,
            })
        }
        per_core
    }
}

#[derive(Debug, Clone)]
pub struct Utilization {
    pub usage: f32,
    pub cur_freq_mhz: u32,
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
