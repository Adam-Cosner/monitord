/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! CPU temperature and power sensor tracking.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use crate::collector::helpers::{
    discovery::Discovery,
    sampler::{Differential, Sampler},
    *,
};

/// Tracker for the CPU sensors.
#[derive(Debug, Clone)]
pub struct Tracker {
    sources: Discovery<Sources>,
    last_energy: BTreeMap<u32, Sampler<u64>>, // for RAPL diff
}

/// A single sample of CPU sensor data.
#[derive(Debug, Clone)]
pub struct Sample {
    pub temperatures: Temperatures,
    pub power: Power,
}

/// A sample of CPU temperature sensor data.
#[derive(Debug, Clone)]
pub struct Temperatures {
    pub package: BTreeMap<u32, Option<f32>>,
    pub cluster: BTreeMap<(u32, u32), Option<f32>>,
    pub core: BTreeMap<(u32, u32, u32), Option<f32>>,
}

/// A sample of CPU power sensor data.
#[derive(Debug, Clone)]
pub struct Power {
    pub package: BTreeMap<u32, Option<f32>>,
}

impl Tracker {
    /// Creates a new `Tracker`
    pub fn new() -> Self {
        Self {
            sources: Discovery::default(),
            last_energy: BTreeMap::new(),
        }
    }

    /// Reads the CPU sensor data and returns a `Sample`.
    pub fn read(&mut self, topology: &super::topology::Topology) -> anyhow::Result<Sample> {
        let sources = self
            .sources
            .probe_mut(|| Ok(Sources::detect(topology)))
            .ok_or_else(|| anyhow::anyhow!("Failed to detect sensors"))?;
        let temperatures = sources.read_temperatures(topology);
        let power = sources.read_power(&mut self.last_energy);
        Ok(Sample {
            temperatures,
            power,
        })
    }
}

impl Sample {
    pub fn package_temp(&self, package_id: u32) -> Option<f32> {
        self.temperatures
            .package
            .get(&package_id)
            .copied()
            .flatten()
    }

    pub fn cluster_temp(&self, cluster_id: (u32, u32)) -> Option<f32> {
        self.temperatures
            .cluster
            .get(&cluster_id)
            .copied()
            .flatten()
    }

    pub fn core_temp(&self, core_id: (u32, u32, u32)) -> Option<f32> {
        self.temperatures.core.get(&core_id).copied().flatten()
    }

    pub fn package_power(&self, package_id: u32) -> Option<f32> {
        self.power.package.get(&package_id).copied().flatten()
    }
}

impl Differential for u64 {
    type Delta = u64;

    fn delta(&self, other: &Self) -> Self::Delta {
        self.wrapping_sub(*other)
    }
}

#[derive(Debug, Clone)]
struct Sources {
    thermal: BTreeMap<u32, ThermalSource>,
    power: BTreeMap<u32, PowerSource>,
}

#[derive(Debug, Clone)]
enum ThermalSource {
    /// Intel coretemp via platform hwmon
    Coretemp { hwmon: PathBuf },
    /// AMD k10temp via PCI hwmon
    K10temp { hwmon: PathBuf },
    /// AMD zenpower (third-party, mutually exclusive with k10temp)
    Zenpower { hwmon: PathBuf },
    /// VIA/Centaur via platform hwmon
    ViaCputemp { hwmon: PathBuf },
    /// ARM or generic thermal zone fallback
    ThermalZone { zone: PathBuf },
    /// No supported source found
    None,
}

#[derive(Debug, Clone)]
enum PowerSource {
    /// Intel RAPL energy counters (requires two-sample diffing)
    Rapl { energy_path: PathBuf },
    /// AMD power via same hwmon as thermal (instantaneous reading)
    Hwmon { path: PathBuf },
    /// No supported source found
    None,
}

// === Detection ===
impl Sources {
    fn detect(topology: &super::topology::Topology) -> Self {
        let mut thermal = BTreeMap::new();
        let mut power = BTreeMap::new();

        for (&package_id, package) in topology.packages.iter() {
            let vendor = package.vendor_id.as_str();
            thermal.insert(package_id, detect_thermal(package_id, vendor));
            power.insert(package_id, detect_power(package_id, vendor));
        }

        Self { thermal, power }
    }

    fn read_temperatures(&self, topology: &super::topology::Topology) -> Temperatures {
        let mut temps = Temperatures {
            package: BTreeMap::new(),
            cluster: BTreeMap::new(),
            core: BTreeMap::new(),
        };

        for (&package_id, source) in self.thermal.iter() {
            temps.package.insert(package_id, read_package_temp(source));

            if let Some(package) = topology.packages.get(&package_id) {
                for (&cluster_id, cluster) in package.clusters.iter() {
                    temps.cluster.insert(
                        (package_id, cluster_id),
                        read_cluster_temp(source, cluster_id),
                    );

                    for (&core_id, _) in cluster.cores.iter() {
                        temps.core.insert(
                            (package_id, cluster_id, core_id),
                            read_core_temp(source, core_id),
                        );
                    }
                }
            }
        }
        temps
    }

    fn read_power(&mut self, last_energy: &mut BTreeMap<u32, Sampler<u64>>) -> Power {
        let mut package = BTreeMap::new();
        for (&package_id, source) in self.power.iter() {
            let watts = match source {
                PowerSource::Rapl { energy_path } => {
                    read_rapl_energy(package_id, energy_path, last_energy)
                }
                PowerSource::Hwmon { path } => sysfs::read_hwmon_power(path),
                PowerSource::None => None,
            };
            package.insert(package_id, watts);
        }

        Power { package }
    }
}

// === Thermal Detection per vendor ===
fn detect_thermal(package_id: u32, vendor: &str) -> ThermalSource {
    match vendor {
        "GenuineIntel" => detect_coretemp(package_id),
        "AuthenticAMD" => detect_amd_thermal(),
        "CentaurHauls" | "VIA" => detect_via_thermal(package_id),
        _ => detect_thermal_zone(),
    }
}

fn detect_coretemp(package_id: u32) -> ThermalSource {
    let platform = PathBuf::from(format!("/sys/devices/platform/coretemp.{package_id}/hwmon"));
    match sysfs::first_hwmon_subdir(&platform) {
        Some(hwmon) => ThermalSource::Coretemp { hwmon },
        None => detect_thermal_zone(),
    }
}

fn detect_amd_thermal() -> ThermalSource {
    if let Some(hwmon) = sysfs::find_pci_driver_hwmon("zenpower") {
        return ThermalSource::Zenpower { hwmon };
    }
    if let Some(hwmon) = sysfs::find_pci_driver_hwmon("k10temp") {
        return ThermalSource::K10temp { hwmon };
    }
    detect_thermal_zone()
}

fn detect_via_thermal(package_id: u32) -> ThermalSource {
    let platform = PathBuf::from(format!(
        "/sys/devices/platform/via_cputemp.{package_id}/hwmon"
    ));
    match sysfs::first_hwmon_subdir(&platform) {
        Some(hwmon) => ThermalSource::ViaCputemp { hwmon },
        None => detect_thermal_zone(),
    }
}

fn detect_thermal_zone() -> ThermalSource {
    let thermal_dir = PathBuf::from("/sys/class/thermal");
    let Ok(entries) = std::fs::read_dir(&thermal_dir) else {
        return ThermalSource::None;
    };

    let cpu_zone_types = [
        "cpu",
        "bigcore",
        "littlecore",
        "big-",
        "little-",
        "soc",
        "x86_pkg_temp",
    ];

    for entry in entries.flatten() {
        let path = entry.path();
        if !path
            .file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with("thermal_zone"))
        {
            continue;
        }
        if let Some(zone_type) = sysfs::read_string(&path.join("type")) {
            let lower = zone_type.to_lowercase();
            if cpu_zone_types.iter().any(|pat| lower.contains(pat)) {
                return ThermalSource::ThermalZone { zone: path };
            }
        }
    }

    ThermalSource::None
}

// === Power Detection per vendor ===
fn detect_power(package_id: u32, vendor: &str) -> PowerSource {
    match vendor {
        "GenuineIntel" => detect_rapl(package_id),
        "AuthenticAMD" => detect_amd_power(),
        _ => PowerSource::None,
    }
}

fn detect_rapl(package_id: u32) -> PowerSource {
    let energy_path = PathBuf::from(format!(
        "/sys/class/powercap/intel-rapl:{package_id}/energy_uj"
    ));
    if energy_path.exists() {
        PowerSource::Rapl { energy_path }
    } else {
        PowerSource::None
    }
}

fn detect_amd_power() -> PowerSource {
    // AMD exposes power through the same hwmon as thermal ON SOME SYSTEMS
    if let Some(hwmon) = sysfs::find_pci_driver_hwmon("zenpower") {
        let path = hwmon.join("power1_input");
        if path.exists() {
            return PowerSource::Hwmon { path };
        }
    }
    if let Some(hwmon) = sysfs::find_pci_driver_hwmon("k10temp") {
        let path = hwmon.join("power1_input");
        if path.exists() {
            return PowerSource::Hwmon { path };
        }
    }
    PowerSource::None
}

// === Temperature reading ===

fn read_package_temp(source: &ThermalSource) -> Option<f32> {
    match source {
        ThermalSource::Coretemp { hwmon } => {
            // temp1_input is typically the package temperature
            sysfs::read_hwmon_temp(&hwmon.join("temp1_input"))
        }
        ThermalSource::K10temp { hwmon } | ThermalSource::Zenpower { hwmon } => {
            // Tctl or Tdie — temp1 is usually Tctl
            sysfs::read_hwmon_temp(&hwmon.join("temp1_input"))
        }
        ThermalSource::ViaCputemp { hwmon } => sysfs::read_hwmon_temp(&hwmon.join("temp1_input")),
        ThermalSource::ThermalZone { zone } => sysfs::read_hwmon_temp(&zone.join("temp")),
        ThermalSource::None => None,
    }
}

fn read_cluster_temp(source: &ThermalSource, cluster_id: u32) -> Option<f32> {
    match source {
        ThermalSource::K10temp { hwmon } | ThermalSource::Zenpower { hwmon } => {
            // CCD temperatures: temp3_input, temp4_input, etc.
            // CCD n maps to temp(n+3)_input on most AMD chips
            let path = hwmon.join(format!("temp{}_input", cluster_id + 3));
            sysfs::read_hwmon_temp(&path)
        }
        _ => None, // Most other sources don't expose per-cluster temps
    }
}

fn read_core_temp(source: &ThermalSource, core_id: u32) -> Option<f32> {
    match source {
        ThermalSource::Coretemp { hwmon } => {
            // Core temps start at temp2_input (temp1 is package)
            let path = hwmon.join(format!("temp{}_input", core_id + 2));
            sysfs::read_hwmon_temp(&path)
        }
        _ => None, // AMD k10temp/zenpower don't expose per-core temps
    }
}

// === Power Reading ===

fn read_rapl_energy(
    package_id: u32,
    energy_path: &Path,
    energy: &mut BTreeMap<u32, Sampler<u64>>,
) -> Option<f32> {
    let energy_uj = sysfs::read_u64(energy_path).unwrap_or_default();
    let delta = energy
        .entry(package_id)
        .or_insert_with(Sampler::new)
        .push(energy_uj);
    delta.map(|d| (d.change as f64 / (d.interval.as_secs_f64() * 1_000_000.0)) as f32)
}
