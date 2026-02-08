use std::process::Command;

/// Memory metric collection
///
/// # Example
///
/// ```
/// let collector = monitord_metrics::memory::Collector::new();
/// let result = collector.collect().unwrap();
/// assert!()
/// ```
use procfs::Current;
use anyhow::Context;

#[doc(inline)]
pub use crate::metrics::Memory as Snapshot;

/// The metric collector, create an instance with `memory::Collector::new()` and collect with `collector.collect()`
pub struct Collector {
    sp_rt_ff: Option<(u64, String, String)>,
    tried_udevadm: bool,
}

impl Collector {
    /// Create a new instance of the collector
    pub fn new() -> Self {
        Self {
            sp_rt_ff: None,
            tried_udevadm: false,
        }
    }

    /// Collects a `memory::Snapshot`
    pub fn collect(&mut self) -> anyhow::Result<Snapshot> {
        let meminfo =
            procfs::Meminfo::current().with_context(|| format!("{} on {}", file!(), line!()))?;
        let capacity = meminfo.mem_total;
        let in_use = meminfo.mem_total - meminfo.mem_free;
        let free = meminfo.mem_free;
        let cached = meminfo.cached;
        let available = meminfo.mem_available.unwrap_or(0);
        let swap_capacity = meminfo.swap_total;
        let swap_in_use = meminfo.swap_total - meminfo.swap_free;

        let (speed, form_factor, ram_type) = match self.sp_rt_ff.clone() {
            Some((speed, form_factor, ram_type)) => (speed, form_factor, ram_type),
            None => {
                self.collect_from_udevadm().unwrap_or_default()
            }
        };

        Ok(Snapshot {
            capacity,
            in_use,
            free,
            cached,
            available,
            swap_capacity,
            swap_in_use,
            speed,
            form_factor,
            ram_type,
        })
    }

    fn collect_from_udevadm(&mut self) -> anyhow::Result<(u64, String, String)> {
        let mut cmd = Command::new("udevadm");
        cmd.arg("info").arg("-q").arg("property").arg("-p").arg("/sys/devices/virtual/dmi/id");
        cmd.env_remove("LD_PRELOAD");

        self.tried_udevadm = true;

        let output = cmd.output().with_context(|| format!("{} on {}", file!(), line!()))?;
        let stdout = String::from_utf8(output.stdout).with_context(|| format!("{} on {}", file!(), line!()))?;

        let mut speed = 0;
        let mut form_factor = String::new();
        let mut ram_type = String::new();
        let lines = stdout.lines().collect::<Vec<&str>>();
        let slot_number = lines.iter().find(|line| line.starts_with("MEMORY_ARRAY_NUM_DEVICES")).map(|line| line.split('=').nth(1).unwrap().parse::<u32>().unwrap_or(0));

        // Speed
        for slot_index in 0..slot_number.unwrap_or(0) {
            if let Some(speed_mts) = lines.iter().find(|line| line.starts_with(format!("MEMORY_DEVICE_{}_SPEED_MTS=", slot_index).as_str())).map(|line| line.split('=').nth(1).unwrap().parse::<u64>().unwrap_or(0)) {
                speed = core::cmp::max(speed, speed_mts);
            }
        }

        // RAM Type
        for slot_index in 0..slot_number.unwrap_or(0) {
            if let Some(ram_type_value) = lines.iter().find(|line| line.starts_with(format!("MEMORY_DEVICE_{}_TYPE=", slot_index).as_str())).map(|line| line.split('=').nth(1).unwrap()) {
                if ram_type_value == "Unknown" {
                    continue;
                }
                ram_type = ram_type_value.to_string();
                break;
            }
        }

        // Form Factor
        for slot_index in 0..slot_number.unwrap_or(0) {
            if let Some(form_factor_value) = lines.iter().find(|line| line.starts_with(format!("MEMORY_DEVICE_{}_FORM_FACTOR=", slot_index).as_str())).map(|line| line.split('=').nth(1).unwrap()) {
                if form_factor_value == "Unknown" {
                    continue;
                }

                form_factor = form_factor_value.to_string();
                break;
            }
        }

        self.sp_rt_ff = Some((speed.clone(), ram_type.clone(), form_factor.clone()));

        Ok((speed, ram_type, form_factor))
    }
}



mod tests {
    #[test]
    fn test_memory_snapshot() -> anyhow::Result<()> {
        let mut collector = super::Collector::new();
        let snapshot = collector.collect()?;
        println!("{:?}", snapshot);
        assert!(snapshot.capacity > 0);
        assert!(snapshot.in_use > 0);
        assert!(snapshot.free > 0);
        assert!(snapshot.cached > 0);
        assert!(snapshot.available > 0);
        assert!(snapshot.swap_capacity > 0);
        assert!(snapshot.swap_in_use > 0);
        assert!(snapshot.speed > 0);
        assert!(!snapshot.form_factor.is_empty());
        assert!(!snapshot.ram_type.is_empty());
        Ok(())
    }
}
