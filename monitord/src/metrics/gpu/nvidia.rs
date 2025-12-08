use crate::error::Result;

pub struct NvidiaMetricCache {
    nvml: nvml_wrapper::Nvml,
}

impl NvidiaMetricCache {
    pub fn new() -> Result<Self> {
        let nvml = nvml_wrapper::Nvml::init()?;
        Ok(Self { nvml })
    }

    pub fn collect(
        &self,
        request: &monitord_types::service::GpuRequest,
    ) -> Result<Vec<monitord_types::service::GpuResponse>> {
        let mut responses = Vec::new();
        for i in 0..self.nvml.device_count()? {
            let device = self.nvml.device_by_index(i)?;
            let brand = device.name()?;
            let utilization = device.utilization_rates()?.gpu as f64;
            let vram_total = device.memory_info()?.total;
            let vram_utilization = device.memory_info()?.used;
            let wattage = device.power_usage()? as f64 / 1000.0;
            let temperature = device
                .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)?
                as f64;

            let mut processes = Vec::new();
            for process in device.process_utilization_stats(None)? {
                processes.push(monitord_types::service::GpuProcess {
                    pid: process.pid,
                    utilization: process.sm_util as f64,
                    vram: process.mem_util as u64,
                });
            }
            responses.push(monitord_types::service::GpuResponse {
                brand,
                utilization,
                vram_total,
                vram_utilization,
                wattage,
                temperature,
                processes,
            });
        }

        // Implementation details
        Ok(responses)
    }
}
