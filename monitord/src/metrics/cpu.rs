use crate::error::Result;
use std::collections::HashMap;

pub struct CpuMetricCollector {
    sys: sysinfo::System,
}

impl CpuMetricCollector {
    pub fn new() -> Result<Self> {
        Ok(Self {
            sys: sysinfo::System::new_with_specifics(
                sysinfo::RefreshKind::nothing().with_cpu(sysinfo::CpuRefreshKind::everything()),
            ),
        })
    }

    pub fn collect(
        &mut self,
        request: &monitord_types::service::CpuRequest,
    ) -> Result<Vec<monitord_types::service::CpuResponse>> {
        self.sys.refresh_cpu_all();
        let cpus = split_cpus(self.sys.cpus());
        let mut cpu_metrics = Vec::new();

        // Iterate over each branded CPU
        for (brand, cores) in cpus.iter() {
            let utilization = self.sys.global_cpu_usage() as f64;
            let frequency_mhz = cores
                .iter()
                .max_by(|x, y| x.frequency().cmp(&y.frequency()))
                .map(|cpu| cpu.frequency())
                .unwrap_or_default() as u32;
            // TODO: implement CPU temperature
            let temperature = 0.0;
            // Per-core metrics
            let cores = if request.per_core {
                cores
                    .iter()
                    .map(|core| monitord_types::service::Core {
                        utilization: core.cpu_usage() as f64,
                        frequency_mhz: core.frequency() as u32,
                    })
                    .collect()
            } else {
                Vec::new()
            };

            cpu_metrics.push(monitord_types::service::CpuResponse {
                brand: brand.to_string(),
                utilization,
                frequency_mhz,
                temperature,
                cores,
            });
        }

        Ok(cpu_metrics)
    }
}

fn split_cpus(cpus: &[sysinfo::Cpu]) -> HashMap<String, Vec<&sysinfo::Cpu>> {
    let mut map = HashMap::new();
    for cpu in cpus.iter() {
        map.entry(cpu.brand().to_string())
            .or_insert_with(Vec::new)
            .push(cpu);
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_metrics() -> Result<()> {
        let request = monitord_types::service::CpuRequest { per_core: true };

        let mut metric_cache = CpuMetricCollector::new()?;
        let _ = metric_cache.collect(&request)?;
        // pause to allow second capture for accurate info
        std::thread::sleep(std::time::Duration::from_millis(1000));
        let cpu_metrics = metric_cache.collect(&request)?;

        println!("{:?}", cpu_metrics);

        Ok(())
    }

    #[test]
    fn test_temperature() -> Result<()> {
        let components = sysinfo::Components::new_with_refreshed_list();
        for component in components.iter() {
            let temperature = component.temperature().unwrap_or_default();
            let label = component.label().to_string();
            println!("Component [{label}] Temperature: {temperature:.2}°C");
        }

        Ok(())
    }
}
