/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! CPU topology discovery and cache
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use procfs::Current;

use crate::collector::helpers::sysfs;

// DATA STRUCTURES

#[derive(Debug, Clone)]
pub struct Topology {
    pub packages: BTreeMap<u32, Package>,
}

impl Default for Topology {
    fn default() -> Self {
        Self {
            packages: BTreeMap::new(),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Package {
    pub vendor_id: String,
    pub model_name: String,
    pub family: u32,
    pub model: u32,
    pub stepping: u32,
    pub microcode_version: String,
    pub cpufreq_driver: String,
    pub cpufreq_governor: String,
    pub cpufreq_mode: Option<String>,
    pub clusters: BTreeMap<u32, Cluster>,
}

#[derive(Default, Debug, Clone)]
pub struct Cluster {
    pub cores: BTreeMap<u32, Core>,
    pub shared_caches: Vec<Cache>,
}

#[derive(Default, Debug, Clone)]
pub struct Core {
    pub base_freq_mhz: u32,
    pub max_freq_mhz: u32,
    pub threads: BTreeMap<u32, Thread>,
    pub private_caches: Vec<Cache>,
}

#[derive(Default, Debug, Clone)]
pub struct Thread {
    pub os_cpu_id: u32,
    pub thread_index: u32, // 0 or 1 within the core
}

#[derive(Debug, Clone)]
pub struct Cache {
    pub level: u32,
    pub cache_type: CacheType,
    pub size_kb: u32,
    pub line_size_bytes: u32,
    pub associativity: u32,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum CacheType {
    Unknown,
    Instruction,
    Data,
    Unified,
}

impl From<CacheType> for i32 {
    fn from(value: CacheType) -> Self {
        match value {
            CacheType::Unknown => 0,
            CacheType::Instruction => 1,
            CacheType::Data => 2,
            CacheType::Unified => 3,
        }
    }
}

impl Topology {
    pub fn discover() -> anyhow::Result<Self> {
        let cpuinfo = procfs::CpuInfo::current()?;
        let mut topo = Self {
            packages: BTreeMap::new(),
        };

        for cpu_idx in 0..cpuinfo.num_cores() {
            topo.insert_cpu(&cpuinfo, cpu_idx as u32);
        }

        // Second pass: attach caches (thread counts need to be calculated first)
        for cpu_idx in 0..cpuinfo.num_cores() {
            topo.attach_caches(cpu_idx as u32);
        }

        Ok(topo)
    }

    fn insert_cpu(&mut self, cpuinfo: &procfs::CpuInfo, cpu_idx: u32) {
        let package_id = cpuinfo.physical_id(cpu_idx as usize).unwrap_or(0);
        let cluster_id = read_cluster_id(cpu_idx);
        let core_id = cpuinfo
            .get_field(cpu_idx as usize, "core id")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let os_cpu_id = cpuinfo
            .get_field(cpu_idx as usize, "processor")
            .and_then(|s| s.parse().ok())
            .unwrap_or(cpu_idx);

        let pkg = self
            .packages
            .entry(package_id)
            .or_insert_with(|| Package::from_cpuinfo(cpuinfo, cpu_idx));

        let cluster = pkg.clusters.entry(cluster_id).or_default();

        let core = cluster
            .cores
            .entry(core_id)
            .or_insert_with(|| Core::from_sysfs(cpu_idx));

        let thread_index = core.threads.len() as u32;
        core.threads.entry(os_cpu_id).or_insert(Thread {
            os_cpu_id,
            thread_index,
        });
    }

    fn attach_caches(&mut self, cpu_idx: u32) {
        let Some((package_id, cluster_id, core_id, thread_count)) = self.locate_cpu(cpu_idx) else {
            return;
        };

        let cache_dir = PathBuf::from(format!("/sys/devices/system/cpu/cpu{cpu_idx}/cache"));
        let Ok(entries) = std::fs::read_dir(&cache_dir) else {
            return;
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if !path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("index"))
            {
                continue;
            }

            let Some(cache) = Cache::from_sysfs(&path) else {
                continue;
            };

            let shared_count = sysfs::read_string(&path.join("shared_cpu_list"))
                .and_then(|s| sysfs::count_cpu_list(&s))
                .unwrap_or(1);

            if shared_count > thread_count {
                // This cache entry is shared across multiple CPUs, so we need to attach it to all of them (probably L3 cache)
                let Some(cluster) = self
                    .packages
                    .get_mut(&package_id)
                    .and_then(|p| p.clusters.get_mut(&cluster_id))
                else {
                    tracing::error!(
                        "Could not locate cluster {cluster_id} for package {package_id}"
                    );
                    continue;
                };
                if !cluster
                    .shared_caches
                    .iter()
                    .any(|c| c.level == cache.level && c.cache_type == cache.cache_type)
                {
                    cluster.shared_caches.push(cache);
                }
            } else {
                // Private to this core (e.g. L1, L2)
                let Some(core) = self
                    .packages
                    .get_mut(&package_id)
                    .and_then(|p| p.clusters.get_mut(&cluster_id))
                    .and_then(|c| c.cores.get_mut(&core_id))
                else {
                    tracing::error!(
                        "Could not locate cluster {cluster_id} for package {package_id}"
                    );
                    continue;
                };
                if !core
                    .private_caches
                    .iter()
                    .any(|c| c.level == cache.level && c.cache_type == cache.cache_type)
                {
                    core.private_caches.push(cache);
                }
            }
        }
    }

    // Returns: (package_id, cluster_id, core_id, thread_count)
    fn locate_cpu(&self, cpu_idx: u32) -> Option<(u32, u32, u32, u32)> {
        let os_id = cpu_idx;
        for (&pkg_id, pkg) in &self.packages {
            for (&cl_id, cluster) in &pkg.clusters {
                for (&core_id, core) in &cluster.cores {
                    if core.threads.contains_key(&os_id) {
                        return Some((pkg_id, cl_id, core_id, core.threads.len() as u32));
                    }
                }
            }
        }
        None
    }
}

impl Package {
    fn from_cpuinfo(cpuinfo: &procfs::CpuInfo, cpu_idx: u32) -> Self {
        let cpu_idx = cpu_idx as usize;
        let vendor_id = cpuinfo
            .vendor_id(cpu_idx)
            .map(|v| v.to_string())
            .unwrap_or_default();
        let model_name = cpuinfo
            .model_name(cpu_idx)
            .map(|v| v.to_string())
            .unwrap_or_default();
        let family = cpuinfo
            .get_field(cpu_idx, "cpu family")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);
        let model = cpuinfo
            .get_field(cpu_idx, "model")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);
        let stepping = cpuinfo
            .get_field(cpu_idx, "stepping")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);
        let microcode_version = cpuinfo
            .get_field(cpu_idx, "microcode")
            .map(|v| v.to_string())
            .unwrap_or_default();
        let (cpufreq_driver, cpufreq_governor, cpufreq_mode) =
            sysfs::get_cpufreq_info(cpu_idx as u32);

        Self {
            vendor_id,
            model_name,
            family,
            model,
            stepping,
            microcode_version,
            cpufreq_driver,
            cpufreq_governor,
            cpufreq_mode,
            clusters: BTreeMap::new(),
        }
    }
}

impl Core {
    fn from_sysfs(cpu_idx: u32) -> Self {
        let base_freq_mhz = sysfs::read_u32(&PathBuf::from(format!(
            "/sys/devices/system/cpu/cpu{cpu_idx}/cpufreq/scaling_min_freq"
        )))
        .unwrap_or(0);
        let max_freq_mhz = sysfs::read_u32(&PathBuf::from(format!(
            "/sys/devices/system/cpu/cpu{cpu_idx}/cpufreq/scaling_max_freq"
        )))
        .unwrap_or(0);
        Self {
            base_freq_mhz,
            max_freq_mhz,
            threads: BTreeMap::new(),
            private_caches: Vec::new(),
        }
    }
}

impl Cache {
    fn from_sysfs(path: &Path) -> Option<Self> {
        let level = sysfs::read_u32(&path.join("level")).unwrap_or(0);
        let cache_type =
            CacheType::from_string(&sysfs::read_string(&path.join("type")).unwrap_or_default());
        let size_kb = sysfs::read_string(&path.join("size"))
            .as_ref()
            .and_then(|s| s.strip_suffix('K'))
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        let line_size_bytes = sysfs::read_u32(&path.join("coherency_line_size")).unwrap_or(0);
        let associativity = sysfs::read_u32(&path.join("ways_of_associativity")).unwrap_or(0);
        Some(Self {
            level,
            cache_type,
            size_kb,
            line_size_bytes,
            associativity,
        })
    }
}

impl CacheType {
    fn from_string(s: &str) -> Self {
        match s {
            "Instruction" => CacheType::Instruction,
            "Data" => CacheType::Data,
            "Unified" => CacheType::Unified,
            _ => CacheType::Unknown,
        }
    }
}

fn read_cluster_id(cpu_idx: u32) -> u32 {
    let die_id_path = PathBuf::from(format!(
        "/sys/devices/system/cpu/cpu{cpu_idx}/topology/die_id"
    ));
    std::fs::read_to_string(&die_id_path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}
