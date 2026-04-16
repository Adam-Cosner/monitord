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
//!
//! ```
mod sensors;
mod topology;
mod utilization;

use crate::collector::staging;
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

    /// Collects one full snapshot of the CPU and emplaces it into the associated Store slot.
    /// If collection fails critically, the store slot is not modified and an error is returned.
    /// On non-critical errors, the store slot is emplaced with empty data and a warning is logged.
    fn collect(&mut self, config: &crate::metrics::Config) -> anyhow::Result<Self::Output> {
        self.collect_cpus(config.cpu.as_ref())
            .inspect_err(|e| tracing::error!("collector failed: {e}"))
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

    fn collect_cpus(&mut self, config: Option<&Config>) -> anyhow::Result<Snapshot> {
        let Some(config) = config else {
            anyhow::bail!("cpu collector did not receive a config");
        };

        let topo = if config.topology {
            Some(
                self.topology
                    .require(|| topology::Topology::discover(Some(config)))?,
            )
        } else {
            None
        };

        let utilization = self.utilization.sample()?;
        let sensors = topo.and_then(|topo| self.sensors.read(topo).ok());

        Ok(assemble(topo, &utilization, sensors.as_ref()))
    }
}

/// Assembles a [`Snapshot`] from the given topology, utilization, and sensor data.
fn assemble(
    topo: Option<&topology::Topology>,
    utilization: &[utilization::Utilization],
    sensors: Option<&sensors::Sample>,
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
    let Some(topo) = topo else {
        return snapshot;
    };
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
                    core_temperature_c: sensors
                        .and_then(|sensors| sensors.core_temp((package_id, cluster_id, core_id))),
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
                cluster_temperature_c: sensors
                    .and_then(|sensors| sensors.cluster_temp((package_id, cluster_id))),
                cores,
                shared_caches,
            });
        }
        snapshot.packages.push(Package {
            package_id,
            hwid: package.hwid.clone(),
            drivers: package.drivers.clone(),
            package_temperature_c: sensors.and_then(|sensors| sensors.package_temp(package_id)),
            package_power_w: sensors.and_then(|sensors| sensors.package_power(package_id)),
            clusters,
        });
    }
    snapshot
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collector::Collector;

    #[tracing_test::traced_test]
    #[test]
    fn collect() -> anyhow::Result<()> {
        let mut collector = super::Collector::new();
        let mut config = crate::metrics::Config::default();
        config.cpu = Some(Config {
            topology: true,
            hwid: true,
            drivers: true,
        });

        let _ = collector.collect(&config)?;
        std::thread::sleep(std::time::Duration::from_secs(1));
        let snapshot = collector.collect(&config)?;
        assert!(!snapshot.logical.is_empty() && !snapshot.packages.is_empty(),);
        println!("{:#?}", snapshot);
        Ok(())
    }
}
