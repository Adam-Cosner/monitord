use crate::error::Result;
use std::collections::HashMap;

pub struct CpuMetricCache {
    sys: sysinfo::System,
}

impl CpuMetricCache {
    pub fn new() -> Result<Self> {
        Ok(Self {
            sys: sysinfo::System::new(),
        })
    }

    pub fn collect(
        &mut self,
        request: &monitord_types::service::CpuRequest,
    ) -> Result<Vec<monitord_types::service::CpuResponse>> {
        self.sys.refresh_cpu_all();
        let mut brand_to_cpu = HashMap::new();
        // Iterate over cpus and group cpus by brand name (temporary until it's able to differentiate logical CPUs among physical CPUs)
        for cpu in self.sys.cpus() {
            let brand = cpu.brand();

            brand_to_cpu.entry(brand).or_insert_with(Vec::new).push(cpu);
        }

        let mut cpu_metrics = Vec::new();

        // Iterate over each branded CPU
        for (brand, cpus) in brand_to_cpu {
            let mut overall_frequency_mhz: u32 = 0;

            let overall_temperature = 0.0; // todo: get CPU temperature
            tracing::debug!("CPU temperature not yet implemented");

            // Get core info
            let mut cores = Vec::new();
            for cpu in cpus.iter() {
                if cpu.frequency() as u32 > overall_frequency_mhz {
                    overall_frequency_mhz = cpu.frequency() as u32;
                }
                if request.per_core {
                    let core = monitord_types::service::Core {
                        utilization: cpu.cpu_usage() as f64,
                        frequency_mhz: cpu.frequency() as u32,
                        temperature: 0.0, // todo: get per-core temperature
                    };
                    cores.push(core);
                }
            }
            cpu_metrics.push(monitord_types::service::CpuResponse {
                brand: brand.to_string(),
                overall_utilization: self.sys.global_cpu_usage() as f64,
                overall_frequency_mhz,
                overall_temperature,
                cores,
            });
        }

        Ok(cpu_metrics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_metrics() -> Result<()> {
        let request = monitord_types::service::CpuRequest {
            utilization: true,
            frequency: true,
            per_core: true,
            temperature: false,
        };

        let mut metric_cache = CpuMetricCache::new()?;
        let _ = metric_cache.collect(&request)?;
        // pause to allow second capture for accurate info
        std::thread::sleep(std::time::Duration::from_millis(1000));
        let cpu_metrics = metric_cache.collect(&request)?;

        println!("{:?}", cpu_metrics);

        Ok(())
    }
}
