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
pub use crate::metrics::cpu::Snapshot;

pub struct Collector {
    // Fields
}

impl Collector {
    pub fn new() -> Self {
        tracing::info!("Creating CPU collector");
        Self {
            // Initialize fields
        }
    }

    pub fn collect(&mut self) -> anyhow::Result<Snapshot> {
        todo!()
    }
}

// /// The metric collector, create an instance with `cpu::Collector::new()` and collect with `collector.collect()`
// pub struct Collector {
//     last: Option<procfs::KernelStats>,
//     temp_path: Option<PathBuf>,
//     // this is due to k10temp and zenpower having different driver instances for different sockets
//     amdtemp_sorted: Option<Vec<PathBuf>>,
// }

// impl Collector {
//     /// Create a new instance of the collector
//     pub fn new() -> Self {
//         tracing::info!("Creating CPU collector");
//         Self {
//             last: None,
//             temp_path: None,
//             amdtemp_sorted: None,
//         }
//     }

//     /// Collects a Vec of `cpu::Snapshot`s that is separated by socket.
//     /// Returns an empty Vec on first call, on subsequent calls it returns `cpu::Snapshot`s for each CPU socket
//     pub fn collect(&mut self) -> anyhow::Result<Vec<Snapshot>> {
//         tracing::debug!("Collecting CPU metrics");

//         let stat_bench = std::time::Instant::now();
//         let stat = procfs::KernelStats::current()
//             .with_context(|| format!("{} at {}", file!(), line!()))?;
//         tracing::trace!("Read /proc/stat in {:?}", stat_bench.elapsed());

//         match &mut self.last {
//             Some(stat_last) => {
//                 struct LogicalCpu {}

//                 let cpuinfo_bench = std::time::Instant::now();

//                 tracing::trace!("Read /proc/cpuinfo in {:?}", cpuinfo_bench.elapsed());

//                 let mut cpus = vec![None, None];
//                 for i in 0..cpu_info.num_cores() {
//                     // Edit the CPU in the correct socket
//                     let cpu = &mut cpus[cpu_info.physical_id(i).unwrap_or(0) as usize];
//                     if cpu.is_none() {
//                         *cpu = Some(Snapshot::default());
//                     }
//                     let cpu = cpu.as_mut().unwrap();

//                     // CPU model name
//                     cpu.brand_name = cpu_info
//                         .get_field(i, "model name")
//                         .unwrap_or("")
//                         .to_string();

//                     // Core frequency mhz
//                     let frequency_mhz = cpu_info
//                         .get_field(i, "cpu MHz")
//                         .map(|mhz_str| mhz_str.parse::<f32>().unwrap_or(0.0).floor() as u32)
//                         .unwrap_or(0);
//                     cpu.frequency_mhz = if cpu.frequency_mhz < frequency_mhz {
//                         frequency_mhz
//                     } else {
//                         cpu.frequency_mhz
//                     };

//                     // Core utilization calculation
//                     let cpu_time_last = &stat_last.cpu_time[i];
//                     let cpu_time = &stat.cpu_time[i];
//                     let active = (cpu_time.user - cpu_time_last.user)
//                         + (cpu_time.nice - cpu_time_last.nice)
//                         + (cpu_time.system - cpu_time.system)
//                         + (cpu_time.irq.unwrap_or(0) - cpu_time.irq.unwrap_or(0))
//                         + (cpu_time.softirq.unwrap_or(0) - cpu_time.softirq.unwrap_or(0))
//                         + (cpu_time.steal.unwrap_or(0) - cpu_time.steal.unwrap_or(0));
//                     let idle = (cpu_time.idle - cpu_time_last.idle)
//                         + (cpu_time.iowait.unwrap_or(0) - cpu_time.iowait.unwrap_or(0));
//                     let total = active + idle;
//                     let utilization = (active as f64 * 100.0) / total as f64;

//                     cpu.cores.push(Core {
//                         utilization,
//                         frequency_mhz,
//                     })
//                 }
//                 // Iterate over cpus and calculate stats
//                 for i in 0..2 {
//                     let mut utilization = 0.0;
//                     if let Some(cpu) = &mut cpus[i] {
//                         for core in cpu.cores.iter() {
//                             utilization += core.utilization;
//                         }
//                         cpu.utilization = utilization / cpu.cores.len() as f64;
//                         cpu.temperature_c = self.get_temperature(i as u32).unwrap_or(0);
//                     }
//                 }

//                 Ok(cpus.into_iter().filter_map(|x| x).collect())
//             }
//             None => {
//                 tracing::debug!("Previous metrics not available, returning empty");
//                 self.last = Some(stat);

//                 Ok(vec![])
//             }
//         }
//     }

//     fn get_temperature(&mut self, physical_id: u32) -> anyhow::Result<u32> {
//         match &self.temp_path {
//             Some(path) => {
//                 // /sys/class/hwmon
//                 let name_path = path.join("name");
//                 if let Ok(name_val) = std::fs::read_to_string(&name_path) {
//                     match name_val.trim() {
//                         "coretemp" => return read_temp_coretemp(path.clone(), physical_id),
//                         "k10temp" => return read_temp_k10temp(path.clone()),
//                         _ => {}
//                     }
//                 }

//                 // /sys/class/thermal
//                 let type_path = path.join("type");
//                 if let Ok(type_val) = std::fs::read_to_string(&type_path) {
//                     match type_val.trim() {
//                         // Todo: read thermal
//                         _ => {}
//                     }
//                 }
//             }
//             None => {
//                 // /sys/class/hwmon
//                 for entry in std::fs::read_dir("/sys/class/hwmon")
//                     .with_context(|| format!("{} at {}", file!(), line!()))?
//                     .flatten()
//                 {
//                     let path = entry.path();

//                     // Read the name file to check for the list of supported hwmon drivers
//                     let name_path = path.join("name");
//                     if let Ok(name_val) = std::fs::read_to_string(&name_path) {
//                         match name_val.trim() {
//                             "coretemp" => {
//                                 self.temp_path = Some(path.clone());
//                                 return read_temp_coretemp(path, physical_id);
//                             }
//                             "k10temp" => {
//                                 if self.amdtemp_sorted.is_none() {
//                                     self.amdtemp_sorted = Some(get_k10temp_sorted()?);
//                                 }
//                                 self.temp_path = Some(path.clone());
//                                 return read_temp_k10temp(path);
//                             }
//                             _ => {}
//                         }
//                     }
//                 }
//                 // /sys/class/thermal
//                 for entry in std::fs::read_dir("/sys/class/thermal")
//                     .with_context(|| format!("{} at {}", file!(), line!()))?
//                     .flatten()
//                 {
//                     let path = entry.path();

//                     // Only read thermal_zones
//                     if path.file_name().is_some_and(|file_name| {
//                         file_name.to_string_lossy().starts_with("thermal_zone")
//                     }) {
//                         // Read the type file to check for the list of supported thermal drivers
//                         let type_path = path.join("type");
//                         if let Ok(type_val) = std::fs::read_to_string(&type_path) {
//                             match type_val.trim() {
//                                 // TODO: thermal drivers
//                                 _ => {}
//                             }
//                         }
//                     }
//                 }
//             }
//         }

//         Err(anyhow::anyhow!(
//             "Could not find a supported CPU temperature driver, please report this with the system's CPU model"
//         ))
//     }
// }

// fn read_temp_coretemp(hwmon_path: PathBuf, socket: u32) -> anyhow::Result<u32> {
//     let mut temperature = 0;
//     for entry in std::fs::read_dir(&hwmon_path)
//         .with_context(|| format!("{} at {}", file!(), line!()))?
//         .flatten()
//     {
//         let path = entry.path();

//         if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
//             if file_name.ends_with("_label") {
//                 if let Ok(label) = std::fs::read_to_string(&path) {
//                     // If package id exists, just return it
//                     if label.trim() == format!("Package id {socket}") {
//                         if let Ok(package_value) = std::fs::read_to_string(
//                             hwmon_path.join(file_name.replace("label", "input")),
//                         ) {
//                             return package_value
//                                 .parse::<u32>()
//                                 .with_context(|| format!("{} at {}", file!(), line!()));
//                         }
//                     }
//                     if label.trim() == format!("Core {socket}") {
//                         if let Ok(core_value) = std::fs::read_to_string(
//                             hwmon_path.join(file_name.replace("label", "input")),
//                         ) {
//                             temperature += core_value
//                                 .parse::<u32>()
//                                 .with_context(|| format!("{} at {}", file!(), line!()))?;
//                         }
//                     }
//                 }
//             }
//         }
//     }
//     Ok(temperature)
// }

// fn read_temp_k10temp(hwmon_path: PathBuf) -> anyhow::Result<u32> {
//     let mut temperature = 0;
//     for entry in std::fs::read_dir(&hwmon_path)
//         .with_context(|| format!("{} at {}", file!(), line!()))?
//         .flatten()
//     {
//         let path = entry.path();

//         if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
//             if file_name.ends_with("_label") {
//                 if let Ok(label) = std::fs::read_to_string(&path) {
//                     // If Tdie is given, that's the full die temperature
//                     if label.trim() == "Tdie" {
//                         if let Ok(tdie_value) = std::fs::read_to_string(
//                             hwmon_path.join(file_name.replace("label", "input")),
//                         ) {
//                             return tdie_value
//                                 .parse::<u32>()
//                                 .with_context(|| format!("{} at {}", file!(), line!()));
//                         }
//                     }
//                     // Fallback to Tccd maximum
//                     if label.trim().starts_with("Tccd") {
//                         if let Ok(tccd_value) = std::fs::read_to_string(
//                             hwmon_path.join(file_name.replace("label", "input")),
//                         ) {
//                             temperature = core::cmp::max(
//                                 tccd_value
//                                     .parse::<u32>()
//                                     .with_context(|| format!("{} at {}", file!(), line!()))?,
//                                 temperature,
//                             );
//                         }
//                     }
//                 }
//             }
//         }
//     }
//     if temperature != 0 {
//         Ok(temperature)
//     } else {
//         Err(anyhow::anyhow!("Failed to read k10temp hwmon"))
//     }
// }

// // This function is used to get a sorted list of k10temp hwmon drivers due to AMD spawning separate hwmon instances per CPU socket >:(
// fn get_k10temp_sorted() -> anyhow::Result<Vec<PathBuf>> {
//     let mut paths = Vec::new();
//     for entry in std::fs::read_dir("/sys/class/hwmon")
//         .with_context(|| format!("{} at {}", file!(), line!()))?
//         .flatten()
//     {
//         let path = entry.path();
//         let name_path = path.join("name");
//         if let Ok(name_val) = std::fs::read_to_string(&name_path) {
//             if name_val.trim() == "k10temp" {
//                 paths.push(path);
//             }
//         }
//     }

//     if paths.len() == 1 {
//         return Ok(paths);
//     }

//     paths.sort_by(|a, b| {
//         if let Ok(a_device) = std::fs::read_link(a.join("device"))
//             && let Ok(b_device) = std::fs::read_link(b.join("device"))
//         {
//             a_device.file_name().cmp(&b_device.file_name())
//         } else {
//             core::cmp::Ordering::Equal
//         }
//     });
//     Ok(paths)
// }

// TODO: Support more temperature sources
