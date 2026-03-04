/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! CPU metric collection
//!
//! First sample will be empty due to needing two samples to calculate usage values
//!
//! # Example
//!
//! ```
//! let collector = monitord_metrics::cpu::Collector::new();
//! // The first collect call will return nothing as collection requires a current and last sample to calculate usages
//! let empty = collector.collect().unwrap();
//! assert!(empty.is_empty());
//! std::thread::sleep(std::time::Duration::from_secs(1));
//! let result = collector.collect().unwrap();
//! assert!(!result.is_empty());
//! ```

#[doc(inline)]
pub use crate::metrics::cpu::*;

use procfs::{Current, CurrentSI};
use std::cell::OnceCell;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct Collector {
    topology: OnceCell<anyhow::Result<cached::Topology>>,
    last_stat: Option<procfs::KernelStats>,
}

impl Collector {
    pub fn new() -> Self {
        tracing::info!("Creating CPU collector");
        Self {
            topology: std::cell::OnceCell::new(),
            last_stat: None,
        }
    }

    pub fn collect(&mut self) -> anyhow::Result<Snapshot> {
        let logical = collect::logical(&mut self.last_stat)?;
        let physical = collect::physical(&mut self.topology)?;

        Ok(Snapshot { logical, physical })
    }
}

mod helpers {
    use super::*;
    pub fn first_hwmon_subdir(hwmon_parent: &PathBuf) -> Option<PathBuf> {
        std::fs::read_dir(hwmon_parent)
            .ok()?
            .flatten()
            .find(|e| e.file_name().to_string_lossy().starts_with("hwmon"))
            .map(|e| e.path())
    }

    pub fn find_pci_driver_hwmon(driver_name: &str) -> Option<PathBuf> {
        let driver_path = PathBuf::from(format!("/sys/bus/pci/drivers/{driver_name}"));
        for entry in std::fs::read_dir(&driver_path).ok()?.flatten() {
            let path = entry.path();
            if path.join("hwmon").exists() {
                if let Some(hwmon) = first_hwmon_subdir(&path.join("hwmon")) {
                    return Some(hwmon);
                }
            }
        }
        None
    }

    pub fn cluster_id(cpu_idx: usize) -> u32 {
        let die_id_path = PathBuf::from(format!(
            "/sys/devices/system/cpu/cpu{cpu_idx}/topology/die_id"
        ));
        std::fs::read_to_string(&die_id_path)
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0)
    }

    pub fn cpufreq_driver_governor_mode(cpu_idx: usize) -> (String, String, Option<String>) {
        let cpufreq_path = PathBuf::from(format!("/sys/devices/system/cpu/cpu{cpu_idx}/cpufreq"));
        let driver = std::fs::read_to_string(cpufreq_path.join("scaling_driver"))
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or_else(String::new);
        let governor = std::fs::read_to_string(cpufreq_path.join("scaling_governor"))
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or_else(String::new);
        let mode = match driver.as_str() {
            "intel_pstate" => std::fs::read_to_string(PathBuf::from(
                "/sys/devices/system/cpu/intel_pstate/status",
            ))
            .ok()
            .and_then(|s| s.trim().parse().ok()),
            "amd-pstate" | "amd-pstate-epp" => {
                std::fs::read_to_string(PathBuf::from("/sys/devices/system/cpu/amd_pstate/status"))
                    .ok()
                    .and_then(|s| s.trim().parse().ok())
            }
            _ => None,
        };
        (driver, governor, mode)
    }

    pub fn get_private_shared_caches(
        cpu_idx: usize,
        topology: &cached::Topology,
    ) -> anyhow::Result<(Vec<Cache>, Vec<Cache>)> {
        let mut private = Vec::new();
        let mut shared = Vec::new();

        let cache_dir = PathBuf::from(format!("/sys/devices/system/cpu/cpu{cpu_idx}/cache"));

        let Ok(entries) = std::fs::read_dir(&cache_dir) else {
            return Ok((private, shared));
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("index"))
            {
                continue;
            }

            let level = std::fs::read_to_string(path.join("level"))
                .ok()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0);
            let cache_type = parse_cache_type(
                std::fs::read_to_string(path.join("type"))
                    .ok()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(String::new)
                    .as_str(),
            );
            let size_kb = parse_cache_size(
                std::fs::read_to_string(path.join("size"))
                    .ok()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(String::new)
                    .as_str(),
            );
            let line_size_bytes = std::fs::read_to_string(path.join("coherency_line_size"))
                .ok()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0);
            let associativity = std::fs::read_to_string(path.join("ways_of_associativity"))
                .ok()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0);

            let shared_cpus = std::fs::read_to_string(path.join("shared_cpu_list"))
                .ok()
                .map(|s| s.trim().to_string())
                .unwrap_or_else(String::new);
            let shared_count = count_cpus_in_list(&shared_cpus);

            let cache = Cache {
                level,
                cache_type: cache_type as i32,
                size_kb,
                line_size_bytes,
                associativity,
            };

            // If shared across more CPUs than a single core's thread count,
            // it's a shared cache (L3 typically)
            let core_thread_count = topology.threads_per_core(cpu_idx as u32);
            if shared_count > core_thread_count {
                shared.push(cache);
            } else {
                private.push(cache);
            }
        }

        Ok((private, shared))
    }

    fn parse_cache_type(s: &str) -> cache::CacheType {
        match s.trim() {
            "Instruction" => cache::CacheType::Instruction,
            "Data" => cache::CacheType::Data,
            "Unified" => cache::CacheType::Unified,
            _ => cache::CacheType::Unknown,
        }
    }

    fn parse_cache_size(s: &str) -> u32 {
        let s = s.trim();
        if let Some(val) = s.strip_suffix('K') {
            val.parse().unwrap_or(0)
        } else if let Some(val) = s.strip_suffix('M') {
            val.parse::<u32>().unwrap_or(0) * 1024
        } else {
            tracing::warn!("Unknown cache size format: {}", s);
            0
        }
    }

    /// Parses strings like "0-1", "0-3,8-11", "0"
    fn count_cpus_in_list(list: &str) -> u32 {
        let mut count = 0;
        for part in list.trim().split(',') {
            if let Some((start, end)) = part.split_once('-') {
                let s: u32 = start.parse().unwrap_or(0);
                let e: u32 = end.parse().unwrap_or(0);
                count += (e - s) + 1;
            } else {
                count += 1;
            }
        }
        count
    }

    pub fn cpufreq_frequency(cpu_idx: usize) -> (u32, u32) {
        let base_freq = std::fs::read_to_string(PathBuf::from(format!(
            "/sys/devices/system/cpu/cpu{cpu_idx}/cpufreq/cpuinfo_min_freq"
        )))
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);
        let max_freq = std::fs::read_to_string(PathBuf::from(format!(
            "/sys/devices/system/cpu/cpu{cpu_idx}/cpufreq/cpuinfo_max_freq"
        )))
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);
        (base_freq, max_freq)
    }
}

/// Cached data for usage with `Collector`
mod cached {
    use super::*;

    pub mod thermal {
        use super::*;

        #[allow(unused)]
        #[derive(Debug, Clone, Default)]
        pub enum Source {
            /// Intel coretemp via platform device
            Coretemp { hwmon: PathBuf },
            /// AMD k10temp via PCI hwmon
            K10temp { hwmon: PathBuf },
            /// AMD zenpower (third-party, mutually exclusive with k10temp)
            Zenpower { hwmon: PathBuf },
            /// VIA/Centaur via platform device
            ViaCputemp { hwmon: PathBuf },
            /// ARM/generic thermal zone
            ThermalZone { zone: PathBuf },
            /// No supported driver found
            #[default]
            None,
        }

        impl Source {
            pub fn detect(package_id: u32, vendor: &str) -> Self {
                match vendor {
                    "GenuineIntel" => Self::detect_intel(package_id),
                    "AuthenticAMD" => Self::detect_amd(),
                    "CentaurHauls" | "VIA" => Self::detect_via(package_id),
                    _ => Self::detect_generic(),
                }
            }
            fn detect_intel(package_id: u32) -> Self {
                let platform =
                    PathBuf::from(format!("/sys/devices/platform/coretemp.{package_id}/hwmon"));
                if let Some(hwmon) = helpers::first_hwmon_subdir(&platform) {
                    return Self::Coretemp { hwmon };
                }
                Self::detect_generic()
            }
            fn detect_amd() -> Self {
                // Prefer zenpower if loaded (mutually exclusive with k10temp)
                if let Some(hwmon) = helpers::find_pci_driver_hwmon("zenpower") {
                    return Self::Zenpower { hwmon };
                }
                if let Some(hwmon) = helpers::find_pci_driver_hwmon("k10temp") {
                    return Self::K10temp { hwmon };
                }
                Self::detect_generic()
            }
            fn detect_via(package_id: u32) -> Self {
                let platform = PathBuf::from(format!(
                    "/sys/devices/platform/via_cputemp.{package_id}/hwmon"
                ));
                if let Some(hwmon) = helpers::first_hwmon_subdir(&platform) {
                    return Self::ViaCputemp { hwmon };
                }
                Self::detect_generic()
            }
            fn detect_generic() -> Self {
                let thermal_dir = PathBuf::from("/sys/class/thermal");
                let Ok(entries) = std::fs::read_dir(&thermal_dir) else {
                    return Self::None;
                };
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path
                        .file_name()
                        .is_some_and(|n| n.to_string_lossy().starts_with("thermal_zone"))
                    {
                        continue;
                    }
                    let type_path = path.join("type");
                    if let Ok(zone_type) = std::fs::read_to_string(&type_path) {
                        let trimmed = zone_type.trim().to_lowercase();
                        // Match common CPU-related thermal zone type strings
                        if trimmed.contains("cpu")
                            || trimmed.contains("bigcore")
                            || trimmed.contains("littlecore")
                            || trimmed.contains("big-")
                            || trimmed.contains("little-")
                            || trimmed.contains("soc")
                            || trimmed == "x86_pkg_temp"
                        {
                            return Self::ThermalZone { zone: path };
                        }
                    }
                }
                Self::None
            }
        }

        impl Source {
            pub fn read_package(&self, _package_id: u32) -> Option<f32> {
                // TODO
                None
            }
            pub fn read_cluster(&self, _cluster_id: u32) -> Option<f32> {
                // TODO
                None
            }
            pub fn read_core(&self, _core_id: u32) -> Option<f32> {
                // TODO
                None
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct Topology {
        pub packages: HashMap<u32, Package>,
        pub os_cpu_id_to_location: HashMap<u32, CpuLocation>,
    }

    #[derive(Debug, Clone)]
    pub struct CpuLocation {
        pub package_id: u32,
        pub cluster_id: u32,
        pub core_phys_id: u32,
    }

    impl Topology {
        pub fn build() -> anyhow::Result<Self> {
            let mut topo = Self {
                packages: HashMap::new(),
                os_cpu_id_to_location: HashMap::new(),
            };
            let cpuinfo = procfs::CpuInfo::current()?;
            for i in 0..cpuinfo.num_cores() {
                let cpu_loc = topo.cpu_insert(&cpuinfo, i);
                topo.os_cpu_id_to_location.insert(i as u32, cpu_loc);
            }

            for i in 0..cpuinfo.num_cores() {
                let (private, shared) = helpers::get_private_shared_caches(i, &topo)
                    .ok()
                    .unwrap_or_else(|| (vec![], vec![]));
                if let Some(loc) = topo.os_cpu_id_to_location.get(&(i as u32)).cloned() {
                    if let Some(cl) = topo.get_cluster_mut(loc.package_id, loc.cluster_id) {
                        cl.shared_caches = shared;
                    }
                    if let Some(core) =
                        topo.get_core_mut(loc.package_id, loc.cluster_id, loc.core_phys_id)
                    {
                        core.private_caches = private;
                    }
                }
            }
            Ok(topo)
        }

        #[allow(unused)]
        fn get_package(&self, package_id: u32) -> Option<&Package> {
            self.packages.get(&package_id)
        }

        #[allow(unused)]
        fn get_package_mut(&mut self, package_id: u32) -> Option<&mut Package> {
            self.packages.get_mut(&package_id)
        }

        #[allow(unused)]
        fn get_cluster(&self, package_id: u32, cluster_id: u32) -> Option<&Cluster> {
            self.packages
                .get(&package_id)
                .and_then(|pkg| pkg.clusters.get(&cluster_id))
        }

        #[allow(unused)]
        fn get_cluster_mut(&mut self, package_id: u32, cluster_id: u32) -> Option<&mut Cluster> {
            self.packages
                .get_mut(&package_id)
                .and_then(|pkg| pkg.clusters.get_mut(&cluster_id))
        }

        #[allow(unused)]
        fn get_core(
            &self,
            package_id: u32,
            cluster_id: u32,
            core_phys_id: u32,
        ) -> Option<&CorePhysical> {
            self.packages
                .get(&package_id)
                .and_then(|pkg| pkg.clusters.get(&cluster_id))
                .and_then(|cluster| cluster.cores.get(&core_phys_id))
        }

        #[allow(unused)]
        fn get_core_mut(
            &mut self,
            package_id: u32,
            cluster_id: u32,
            core_phys_id: u32,
        ) -> Option<&mut CorePhysical> {
            self.packages
                .get_mut(&package_id)
                .and_then(|pkg| pkg.clusters.get_mut(&cluster_id))
                .and_then(|cluster| cluster.cores.get_mut(&core_phys_id))
        }

        #[allow(unused)]
        fn get_logical_core(
            &self,
            package_id: u32,
            cluster_id: u32,
            core_phys_id: u32,
            logical_core_id: u32,
        ) -> Option<&CoreLogical> {
            self.packages
                .get(&package_id)
                .and_then(|pkg| pkg.clusters.get(&cluster_id))
                .and_then(|cluster| cluster.cores.get(&core_phys_id))
                .and_then(|core| core.logical_cores.get(&logical_core_id))
        }

        #[allow(unused)]
        fn get_logical_core_mut(
            &mut self,
            package_id: u32,
            cluster_id: u32,
            core_phys_id: u32,
            logical_core_id: u32,
        ) -> Option<&mut CoreLogical> {
            self.packages
                .get_mut(&package_id)
                .and_then(|pkg| pkg.clusters.get_mut(&cluster_id))
                .and_then(|cluster| cluster.cores.get_mut(&core_phys_id))
                .and_then(|core| core.logical_cores.get_mut(&logical_core_id))
        }

        pub fn threads_per_core(&self, cpu_idx: u32) -> u32 {
            self.os_cpu_id_to_location
                .get(&cpu_idx)
                .and_then(|loc| {
                    self.get_core(loc.package_id, loc.cluster_id, loc.core_phys_id)
                        .map(|core| core.logical_cores.len() as u32)
                })
                .unwrap_or(0)
        }

        fn cpu_insert(&mut self, cpuinfo: &procfs::CpuInfo, cpu_idx: usize) -> CpuLocation {
            let package_id = cpuinfo.physical_id(cpu_idx).unwrap_or(0);
            let cluster_id = helpers::cluster_id(cpu_idx);
            let core_phys_id = cpuinfo
                .get_field(cpu_idx, "core id")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let logical_id = cpuinfo
                .get_field(cpu_idx, "processor")
                .and_then(|s| s.parse().ok())
                .unwrap_or(cpu_idx as u32);

            // Package info
            let package = self.packages.entry(package_id).or_insert_with(|| {
                let mut package = Package::default();
                package.vendor_id = cpuinfo
                    .get_field(cpu_idx, "vendor_id")
                    .map(String::from)
                    .unwrap_or_else(String::new);
                package.model_name = cpuinfo
                    .model_name(cpu_idx)
                    .map(String::from)
                    .unwrap_or_else(String::new);
                package.family = cpuinfo
                    .get_field(cpu_idx, "cpu family")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                package.model = cpuinfo
                    .get_field(cpu_idx, "model")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                package.stepping = cpuinfo
                    .get_field(cpu_idx, "stepping")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                package.thermal_source = thermal::Source::detect(package_id, &package.vendor_id);
                package.microcode_version = cpuinfo
                    .get_field(cpu_idx, "microcode")
                    .map(String::from)
                    .unwrap_or_else(String::new);
                (
                    package.cpufreq_driver,
                    package.cpufreq_governor,
                    package.cpufreq_mode,
                ) = helpers::cpufreq_driver_governor_mode(cpu_idx);
                package
            });

            let cluster = package.clusters.entry(cluster_id).or_insert_with(|| {
                let mut cluster = Cluster::default();
                cluster.thermal_source = package.thermal_source.clone();
                cluster.shared_caches = vec![];
                cluster
            });

            let core = cluster.cores.entry(core_phys_id).or_insert_with(|| {
                let mut core = CorePhysical::default();
                (core.base_freq_mhz, core.max_freq_mhz) = helpers::cpufreq_frequency(cpu_idx);
                core.thermal_source = package.thermal_source.clone();
                core.private_caches = vec![];
                core
            });

            let core_idx = core.logical_cores.len() as u32;
            let _ = core.logical_cores.entry(logical_id).or_insert_with(|| {
                let mut logical_core = CoreLogical::default();
                logical_core.core_idx = core_idx;
                logical_core
            });

            CpuLocation {
                package_id,
                cluster_id,
                core_phys_id,
            }
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct Package {
        pub vendor_id: String,
        pub model_name: String,
        pub family: u32,
        pub model: u32,
        pub stepping: u32,
        pub thermal_source: thermal::Source,
        // pub power_source: Option<power::Source>,
        pub microcode_version: String,
        pub cpufreq_driver: String,
        pub cpufreq_governor: String,
        pub cpufreq_mode: Option<String>,
        pub clusters: HashMap<u32, Cluster>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct Cluster {
        pub thermal_source: thermal::Source,
        // pub power_source: Option<power::Source>,
        pub cores: HashMap<u32, CorePhysical>,
        pub shared_caches: Vec<Cache>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct CorePhysical {
        pub base_freq_mhz: u32,
        pub max_freq_mhz: u32,
        pub thermal_source: thermal::Source,
        // pub power_source: Option<power::Source>,
        pub logical_cores: HashMap<u32, CoreLogical>,
        pub private_caches: Vec<Cache>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct CoreLogical {
        pub core_idx: u32,
    }
}

mod collect {
    use super::*;

    // Gets the logical CPU metrics
    pub fn logical(last_stat: &mut Option<procfs::KernelStats>) -> anyhow::Result<Vec<Logical>> {
        let stat = procfs::KernelStats::current()?;

        // Calculates the utilization from CPU times
        fn calculate_utilization(cur: &procfs::CpuTime, last: &procfs::CpuTime) -> f32 {
            let total_cur = cur.user
                + cur.nice
                + cur.system
                + cur.idle
                + cur.iowait.unwrap_or(0)
                + cur.irq.unwrap_or(0)
                + cur.softirq.unwrap_or(0);
            let total_last = last.user
                + last.nice
                + last.system
                + last.idle
                + last.iowait.unwrap_or(0)
                + last.irq.unwrap_or(0)
                + last.softirq.unwrap_or(0);
            let delta_total = total_cur - total_last;
            let delta_idle = cur.idle - last.idle;
            let delta_active = delta_total - delta_idle;
            (100.0 * delta_active as f32 / delta_total as f32) as f32
        }

        // Gets the current frequency of CPU core `id` in MHz.
        fn get_frequency(id: u32) -> u32 {
            std::fs::read_to_string(format!(
                "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_cur_freq",
                id
            ))
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(0)
                / 1000 // convert kHz to MHz
        }

        // Last stat exists, so we can calculate the utilization
        if let Some(last) = last_stat.take() {
            let mut logicals = Vec::new();
            assert_eq!(last.cpu_time.len(), stat.cpu_time.len());

            for i in 0..stat.cpu_time.len() {
                let os_cpu_id = i as u32;
                let utilization = calculate_utilization(&stat.cpu_time[i], &last.cpu_time[i]);
                let frequency = get_frequency(i as u32);
                logicals.push(Logical {
                    os_cpu_id,
                    utilization,
                    cur_freq_mhz: frequency,
                });
            }

            Ok(logicals)
        } else {
            *last_stat = Some(stat);
            Ok(Vec::new())
        }
    }

    // Gets the physical CPU metrics
    pub fn physical(
        topology: &mut std::cell::OnceCell<anyhow::Result<cached::Topology>>,
    ) -> anyhow::Result<Vec<Physical>> {
        let topo = match topology.get_or_init(|| cached::Topology::build()) {
            Ok(topo) => topo,
            Err(_) => return Ok(Vec::new()),
        };

        fn logical_cores(logical_cores: &HashMap<u32, cached::CoreLogical>) -> Vec<CoreLogical> {
            logical_cores
                .iter()
                .map(|(i, c)| CoreLogical {
                    os_cpu_id: *i,
                    core_index: c.core_idx,
                })
                .collect()
        }

        fn cores(cores: &HashMap<u32, cached::CorePhysical>) -> Vec<CorePhysical> {
            cores
                .iter()
                .map(|(i, c)| CorePhysical {
                    core_id: *i,
                    base_freq_mhz: c.base_freq_mhz,
                    max_freq_mhz: c.max_freq_mhz,
                    core_temperature_c: temp::core(*i, &c),
                    logical_cores: logical_cores(&c.logical_cores),
                    private_caches: c.private_caches.clone(),
                })
                .collect()
        }

        fn clusters(clusters: &HashMap<u32, cached::Cluster>) -> Vec<Cluster> {
            clusters
                .iter()
                .map(|(i, c)| Cluster {
                    cluster_id: *i,
                    cluster_temperature_c: temp::cluster(*i, &c),
                    cores: cores(&c.cores),
                    shared_caches: c.shared_caches.clone(),
                })
                .collect()
        }

        Ok(topo
            .packages
            .iter()
            .map(|(i, p)| Physical {
                package_id: *i,
                vendor_id: p.vendor_id.clone(),
                model_name: p.model_name.clone(),
                family: p.family,
                model: p.model,
                stepping: p.stepping,
                microcode_version: p.microcode_version.clone(),
                cpufreq_driver: p.cpufreq_driver.clone(),
                cpufreq_governor: p.cpufreq_governor.clone(),
                cpufreq_mode: p.cpufreq_mode.clone(),
                package_temperature_c: temp::package(*i, &p),
                package_power_w: power::package(&p),
                clusters: clusters(&p.clusters),
            })
            .collect())
    }

    mod temp {
        use super::*;
        pub fn package(package_id: u32, package: &cached::Package) -> Option<f32> {
            let source = &package.thermal_source;
            source.read_package(package_id)
        }
        pub fn cluster(cluster_id: u32, cluster: &cached::Cluster) -> Option<f32> {
            let source = &cluster.thermal_source;
            source.read_cluster(cluster_id)
        }
        pub fn core(core_id: u32, core: &cached::CorePhysical) -> Option<f32> {
            let source = &core.thermal_source;
            source.read_core(core_id)
        }
    }

    #[allow(unused)]
    mod power {
        use super::*;
        pub fn package(package: &cached::Package) -> Option<f32> {
            // TODO
            None
        }
        pub fn cluster(cluster: &cached::Cluster) -> Option<f32> {
            // TODO
            None
        }
        pub fn core(core: &cached::CorePhysical) -> Option<f32> {
            // TODO
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu() -> anyhow::Result<()> {
        tracing_subscriber::fmt::init();
        let mut collector = Collector::new();

        let _ = collector.collect()?;
        std::thread::sleep(std::time::Duration::from_secs(1));
        let snapshot = collector.collect()?;

        println!("{:#?}", snapshot);
        Ok(())
    }
}
