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
mod sensors;
mod topology;
mod utilization;

#[doc(inline)]
pub use crate::metrics::cpu::*;

use super::helpers::cached::Cached;

pub struct Collector {
    topology: Cached<topology::Topology>,
    utilization: utilization::Tracker,
    sensors: sensors::Tracker,
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector {
    pub fn new() -> Self {
        tracing::info!("Creating CPU collector");
        Self {
            topology: Cached::default(),
            utilization: utilization::Tracker::new(),
            sensors: sensors::Tracker::new(),
        }
    }

    pub fn collect(&mut self) -> anyhow::Result<Snapshot> {
        let topo = self.topology.get_or_require(topology::Topology::discover)?;

        let utilization = self.utilization.sample()?;
        let sensors = self.sensors.read(topo)?;

        Ok(assemble(topo, &utilization, &sensors))
    }
}

fn assemble(
    topo: &topology::Topology,
    utilization: &utilization::Sample,
    sensors: &sensors::Sample,
) -> Snapshot {
    let mut snapshot = Snapshot {
        logical: utilization
            .per_core
            .iter()
            .enumerate()
            .map(|(os_cpu_id, util)| Logical {
                os_cpu_id: os_cpu_id as u32,
                utilization: util.usage,
                cur_freq_mhz: util.cur_freq_mhz,
            })
            .collect::<Vec<_>>(),
        packages: Vec::new(),
    };
    // create the physical part
    for (&package_id, package) in topo.packages.iter() {
        let mut clusters = Vec::new();
        for (&cluster_id, cluster) in package.clusters.iter() {
            let mut cores = Vec::new();
            for (&core_id, core) in cluster.cores.iter() {
                let mut threads = Vec::new();
                for thread in core.threads.values() {
                    threads.push(Thread {
                        os_cpu_id: thread.os_cpu_id,
                        core_index: thread.thread_index,
                    });
                }
                let mut private_caches = Vec::new();
                for cache in core.private_caches.iter() {
                    private_caches.push(Cache {
                        level: cache.level,
                        cache_type: cache.cache_type.into(),
                        size_kb: cache.size_kb,
                        line_size_bytes: cache.line_size_bytes,
                        associativity: cache.associativity,
                    });
                }
                cores.push(Core {
                    core_id,
                    base_freq_mhz: core.base_freq_mhz,
                    max_freq_mhz: core.max_freq_mhz,
                    core_temperature_c: sensors.core_temp((package_id, cluster_id, core_id)),
                    threads,
                    private_caches,
                });
            }
            let mut shared_caches = Vec::new();
            for cache in cluster.shared_caches.iter() {
                shared_caches.push(Cache {
                    level: cache.level,
                    cache_type: cache.cache_type.into(),
                    size_kb: cache.size_kb,
                    line_size_bytes: cache.line_size_bytes,
                    associativity: cache.associativity,
                });
            }
            clusters.push(Cluster {
                cluster_id,
                cluster_temperature_c: sensors.cluster_temp((package_id, cluster_id)),
                cores,
                shared_caches,
            });
        }
        snapshot.packages.push(Package {
            package_id,
            vendor_id: package.vendor_id.clone(),
            model_name: package.model_name.clone(),
            family: package.family,
            model: package.model,
            stepping: package.stepping,
            microcode_version: package.microcode_version.clone(),
            cpufreq_driver: package.cpufreq_driver.clone(),
            cpufreq_governor: package.cpufreq_governor.clone(),
            cpufreq_mode: package.cpufreq_mode.clone(),
            package_temperature_c: sensors.package_temp(package_id),
            package_power_w: sensors.package_power(package_id),
            clusters,
        });
    }
    snapshot
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu() -> anyhow::Result<()> {
        tracing_subscriber::fmt::init();
        let mut collector = Collector::new();

        let _ = collector.collect()?;
        std::thread::sleep(std::time::Duration::from_secs(1));
        let snapshot = collector.collect()?;

        println!("{:#?}", snapshot);
        Ok(())
    }
}
