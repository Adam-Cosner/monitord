/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Memory metric collection
//!
//! # Example
//!
//! ```
//! let collector = monitord_metrics::memory::Collector::new();
//! let result = collector.collect().unwrap();
//! assert!()
//! ```
use std::{collections::BTreeMap, path::PathBuf};

use super::helpers::cached::Cached;
use anyhow::Context;
use procfs::Current;

#[doc(inline)]
pub use crate::metrics::memory::*;

/// The metric collector, create an instance with `memory::Collector::new()` and collect with `collector.collect()`
pub struct Collector {
    cached_dimms: Cached<Vec<Dimm>>,
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector {
    /// Create a new instance of the collector
    pub fn new() -> Self {
        tracing::info!("[MEMORY] initializing new collector");
        Self {
            cached_dimms: Cached::default(),
        }
    }

    /// Collects a `memory::Snapshot`
    pub fn collect(&mut self) -> anyhow::Result<Snapshot> {
        tracing::debug!("[MEMORY] collecting metrics");
        let meminfo =
            procfs::Meminfo::current().with_context(|| format!("{} on {}", file!(), line!()))?;
        tracing::trace!("Read /proc/meminfo");

        let capacity = meminfo.mem_total;
        let in_use = meminfo.mem_total - meminfo.mem_free;
        let free = meminfo.mem_free;
        let cached = meminfo.cached;
        let available = meminfo.mem_available.unwrap_or(0);
        let swap_capacity = meminfo.swap_total;
        let swap_in_use = meminfo.swap_total - meminfo.swap_free;
        let logical = Some(Logical {
            capacity,
            in_use,
            free,
            cached,
            available,
            swap_capacity,
            swap_in_use,
        });

        let dimms = self
            .cached_dimms
            .get_or_try(collect_dimms)
            .cloned()
            .unwrap_or_default();

        Ok(Snapshot { logical, dimms })
    }
}

fn collect_dimms() -> anyhow::Result<Vec<Dimm>> {
    match collect_from_dmi() {
        Ok(dimms) => return Ok(dimms),
        Err(e) => tracing::warn!(
            "[MEMORY] DMI reading failed, falling back to udev (this is okay, just means the program doesn't have access): {e}"
        ),
    }
    match collect_from_udev_database() {
        Ok(dimms) => return Ok(dimms),
        Err(e) => tracing::warn!(
            "[MEMORY] udev database reading failed (this happens on non-systemd distros): {e}"
        ),
    }
    Ok(Vec::new())
}

fn collect_from_dmi() -> anyhow::Result<Vec<Dimm>> {
    tracing::debug!("[MEMORY] attempting to parse DMI tables");
    // read in bytes from /sys/firmware/dmi/tables/DMI
    let bytes = std::fs::read(PathBuf::from("/sys/firmware/dmi/tables/DMI"))?;
    let entrypoint = dmidecode::EntryPoint::search(bytes.as_slice())?;

    let memory_devices = entrypoint
        .structures(&bytes[entrypoint.smbios_address() as usize..])
        .filter(|s| matches!(s, Ok(dmidecode::Structure::MemoryDevice(_))));

    let mut dimms = Vec::new();
    for memory_device in memory_devices {
        let memory_device = memory_device?;
        match memory_device {
            dmidecode::Structure::MemoryDevice(memory_device) => {
                if memory_device.size.is_some_and(|size| size != 0) {
                    dimms.push(Dimm {
                        locator: memory_device.device_locator.to_string(),
                        capacity: memory_device
                            .size
                            .filter(|&size| size != 0x7FFF)
                            .map(|size| size as u64)
                            .unwrap_or(memory_device.extended_size as u64),
                        speed_mts: memory_device
                            .configured_memory_speed
                            .map(|speed| speed as u64)
                            .unwrap_or(memory_device.speed.unwrap_or(0) as u64),
                        form_factor: formfactor_to_string(memory_device.form_factor),
                        ram_type: ramtype_to_string(memory_device.memory_type),
                    });
                }
            }

            _ => unreachable!(),
        }
    }

    Ok(dimms)
}

fn formfactor_to_string(form_factor: dmidecode::memory_device::FormFactor) -> String {
    use dmidecode::memory_device::FormFactor;
    match form_factor {
        FormFactor::Other => "Other",
        FormFactor::Unknown => "Unknown",
        FormFactor::Simm => "SIMM",
        FormFactor::Sip => "SIP",
        FormFactor::Chip => "Chip",
        FormFactor::Dip => "DIP",
        FormFactor::Zip => "ZIP",
        FormFactor::ProprietaryCard => "ProprietaryCard",
        FormFactor::Dimm => "DIMM",
        FormFactor::Tsop => "TSOP",
        FormFactor::RowOfChips => "RowOfChips",
        FormFactor::Rimm => "RIMM",
        FormFactor::SoDimm => "SoDIMM",
        FormFactor::Srimm => "SRIMM",
        FormFactor::FbDimm => "FBDIMM",
        FormFactor::Undefined(_) => "Undefined",
    }
    .to_string()
}

fn ramtype_to_string(ram_type: dmidecode::memory_device::Type) -> String {
    use dmidecode::memory_device::Type;
    match ram_type {
        Type::Other => "Other",
        Type::Unknown => "Unknown",
        Type::Dram => "DRAM",
        Type::Edram => "EDRAM",
        Type::Vram => "VRAM",
        Type::Sram => "SRAM",
        Type::Ram => "RAM",
        Type::Rom => "ROM",
        Type::Flash => "Flash",
        Type::Eeprom => "EEPROM",
        Type::Feprom => "FEPROM",
        Type::Eprom => "EPROM",
        Type::Cdram => "CDRAM",
        Type::ThreeDram => "3DRAM",
        Type::Sdram => "SDRAM",
        Type::Sgram => "SGRAM",
        Type::Rdram => "RDDRAM",
        Type::Ddr => "DDR",
        Type::Ddr2 => "DDR2",
        Type::Ddr2FbDimm => "DDR2FbDIMM",
        Type::Reserved => "Reserved",
        Type::Ddr3 => "DDR3",
        Type::Fbd2 => "FBD2",
        Type::Ddr4 => "DDR4",
        Type::Ddr5 => "DDR5",
        Type::LpDdr => "LPDDR",
        Type::LpDdr2 => "LPDDR2",
        Type::LpDdr3 => "LPDDR3",
        Type::LpDdr4 => "LPDDR4",
        Type::LpDdr5 => "LPDDR5",
        Type::LogicalNonVolatileDevice => "LogicalNonVolatileDevice",
        Type::Hbm => "HBM",
        Type::Hbm2 => "HBM2",
        Type::Undefined(_) => "Undefined",
    }
    .to_string()
}

fn collect_from_udev_database() -> anyhow::Result<Vec<Dimm>> {
    tracing::debug!("[MEMORY] attempting to read udev database");
    let udev_filedata = std::fs::read_to_string("/run/udev/data/+dmi:id")?;
    let udev_filedata_lines = udev_filedata.lines().collect::<Vec<&str>>();

    let mut dimms: BTreeMap<u64, Dimm> = BTreeMap::new();
    let mut skip_slot: Option<u64> = None;

    for line in udev_filedata_lines {
        let Some(slot_key_value) = line.strip_prefix("E:MEMORY_DEVICE_") else {
            continue;
        };

        // Split it up into slot ID, key, and value
        let Some((slot_str, rest)) = slot_key_value.split_once('_') else {
            continue;
        };
        let Some((key, value)) = rest.split_once('=') else {
            continue;
        };

        let Ok(slot) = slot_str.parse::<u64>() else {
            continue;
        };

        // Skip this slot if we've determined it's not present
        if skip_slot == Some(slot) {
            continue;
        }

        if key == "PRESENT" && value != "1" {
            skip_slot = Some(slot);
            continue;
        }

        let dimm = dimms.entry(slot).or_default();

        match key {
            "LOCATOR" => dimm.locator = value.to_string(),
            "SIZE" => dimm.capacity = value.parse::<u64>().unwrap_or(0),
            "SPEED_MTS" => dimm.speed_mts = value.parse::<u64>().unwrap_or(0),
            "FORM_FACTOR" => dimm.form_factor = value.to_string(),
            "TYPE" => dimm.ram_type = value.to_string(),
            _ => {}
        }
    }

    Ok(dimms.into_values().collect())
}

mod tests {
    #[test]
    fn memory() -> anyhow::Result<()> {
        let mut collector = super::Collector::new();
        let snapshot = collector.collect()?;
        println!("{:#?}", snapshot);
        Ok(())
    }
}
