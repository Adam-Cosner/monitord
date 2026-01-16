use procfs::{Current, CurrentSI};

#[derive(Default)]
pub struct Snapshot {
    brand_name: String,
    utilization: f64,
    frequency_mhz: u32,
    temperature_c: u32,
    cores: Vec<Core>,
}
pub struct Core {
    utilization: f64,
    frequency_mhz: u32,
}

pub struct Collector {
    last: Option<(procfs::CpuInfo, procfs::KernelStats)>,
}

impl Collector {
    pub fn new() -> Self {
        Self { last: None }
    }

    pub fn collect(&mut self) -> crate::Result<Vec<Snapshot>> {
        let cpu_info = procfs::CpuInfo::current()
            .map_err(|e| crate::error::Error::Collector("CPU".to_string(), e.to_string()))?;
        let stat = procfs::KernelStats::current()
            .map_err(|e| crate::error::Error::Collector("CPU".to_string(), e.to_string()))?;

        match &mut self.last {
            Some((cpu_last, stat_last)) => {
                let mut cpus = vec![Snapshot::default(), Snapshot::default()];
                for i in 0..cpu_info.num_cores() {
                    let cpu = &mut cpus[cpu_info.physical_id(i).unwrap_or(0) as usize];
                    cpu.brand_name = cpu_info
                        .get_field(i, "model name")
                        .unwrap_or("")
                        .to_string();
                    let frequency_mhz = cpu_info
                        .get_field(i, "cpu MHz")
                        .map(|mhz_str| mhz_str.parse::<f32>().unwrap_or(0.0).floor() as u32)
                        .unwrap_or(0);
                    cpu.frequency_mhz = if cpu.frequency_mhz < frequency_mhz {
                        frequency_mhz
                    } else {
                        cpu.frequency_mhz
                    };

                    let cpu_time_last = &stat_last.cpu_time[i];
                    let cpu_time = &stat.cpu_time[i];

                    let active = (cpu_time.user - cpu_time_last.user)
                        + (cpu_time.nice - cpu_time_last.nice)
                        + (cpu_time.system - cpu_time.system)
                        + (cpu_time.irq.unwrap_or(0) - cpu_time.irq.unwrap_or(0))
                        + (cpu_time.softirq.unwrap_or(0) - cpu_time.softirq.unwrap_or(0))
                        + (cpu_time.steal.unwrap_or(0) - cpu_time.steal.unwrap_or(0));
                    let idle = (cpu_time.idle - cpu_time_last.idle)
                        + (cpu_time.iowait.unwrap_or(0) - cpu_time.iowait.unwrap_or(0));
                    let total = active + idle;
                    let utilization = (active as f64 * 100.0) / total as f64;

                    cpu.cores.push(Core {
                        utilization,
                        frequency_mhz,
                    })
                }
                // Iterate over cpus and calculate stats
                for cpu in cpus.iter_mut() {
                    let mut utilization = 0.0;
                    for core in cpu.cores.iter() {
                        utilization += core.utilization;
                    }
                    cpu.utilization = utilization / cpu.cores.len() as f64;
                    let temperature_c = todo!();
                }

                Ok(cpus)
            }
            None => {
                self.last = Some((cpu_info, stat));

                Ok(vec![])
            }
        }
    }
}
