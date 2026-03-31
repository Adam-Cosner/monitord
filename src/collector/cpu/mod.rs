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
//! ```no_run
//! let mut collector = monitord::collector::cpu::Collector::new();
//! let store = monitord::collector::store::Store::new();
//! // The first collect call will return nothing as collection requires a current and last sample to calculate usages
//! collector.collect(&store).unwrap();
//! assert!(store.cpu.get().is_some_and(|c| !c.logical.is_empty()));
//! std::thread::sleep(std::time::Duration::from_secs(1));
//! let store = monitord::collector::store::Store::new();
//! collector.collect(&store).unwrap();
//! assert!(store.cpu.get().is_some_and(|c| !c.logical.is_empty()));
//! ```
mod sensors;
mod topology;
mod utilization;

use crate::collector::store;
#[doc(inline)]
pub use crate::metrics::cpu::*;

use super::helpers::discovery::Discovery;

pub struct Collector {
    topology: Discovery<topology::Topology>,
    utilization: utilization::Tracker,
    sensors: sensors::Tracker,
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Collector for Collector {
    type Output = Snapshot;

    fn name(&self) -> &'static str {
        "cpu"
    }

    fn dependencies(&self) -> &[&'static str] {
        &[]
    }

    /// Collects one full snapshot of the CPU and emplaces it into the associated Store slot.
    /// If collection fails critically, the store slot is not modified and an error is returned.
    /// On non-critical errors, the store slot is emplaced with empty data and a warning is logged.
    fn collect(&mut self, store: &store::Store) -> anyhow::Result<()> {
        match self.collect_cpu() {
            Ok(cpu) => store
                .cpu
                .set(cpu)
                .expect("cpu snapshot was already set previously, do not reuse Store instances!"),
            Err(e) => {
                tracing::error!("collect failed: {e}");
                return Err(e);
            }
        }
        Ok(())
    }
}

impl Collector {
    pub fn new() -> Self {
        tracing::info!("creating collector");
        Self {
            topology: Discovery::default(),
            utilization: utilization::Tracker::new(),
            sensors: sensors::Tracker::new(),
        }
    }

    fn collect_cpu(&mut self) -> anyhow::Result<Snapshot> {
        let topo = self.topology.require(topology::Topology::discover)?;

        let utilization = self.utilization.sample()?;
        let sensors = self.sensors.read(topo)?;

        Ok(assemble(topo, &utilization, &sensors))
    }
}

/// Assembles a [`Snapshot`] from the given topology, utilization, and sensor data.
fn assemble(
    topo: &topology::Topology,
    utilization: &[utilization::Utilization],
    sensors: &sensors::Sample,
) -> Snapshot {
    let mut snapshot = Snapshot {
        logical: utilization
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
    // Assemble the physical part
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
                    min_freq_mhz: core.min_freq_mhz,
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
    use crate::collector::Collector;

    #[test]
    fn cpu() -> anyhow::Result<()> {
        let _ = tracing_subscriber::fmt::try_init();
        let mut collector = super::Collector::new();
        let mut store = store::Store::new();
        collector.collect(&store)?;
        std::thread::sleep(std::time::Duration::from_secs(1));
        store = store::Store::new();
        collector.collect(&store)?;
        assert!(
            store
                .cpu
                .get()
                .is_some_and(|c| !c.logical.is_empty() && !c.packages.is_empty())
        );
        println!("{:#?}", store.cpu.get());
        Ok(())
    }
}
