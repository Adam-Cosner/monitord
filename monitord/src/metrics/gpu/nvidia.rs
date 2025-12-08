use crate::error::Result;

pub struct NvidiaMetricCache {
    nvml: nvml_wrapper::Nvml,
}

impl NvidiaMetricCache {
    pub fn new() -> Result<Self> {
        let nvml = nvml_wrapper::Nvml::init().map_err(|e| {
            tracing::error!("Nvml::init() failed: {}", e);
            e
        })?;
        Ok(Self { nvml })
    }

    pub fn collect(
        &self,
        request: &monitord_types::service::GpuRequest,
    ) -> Result<Vec<monitord_types::service::GpuResponse>> {
        let mut responses = Vec::new();
        for i in 0..self.nvml.device_count().map_err(|e| {
            tracing::error!("Nvml::device_count() failed: {}", e);
            e
        })? {
            let device = self.nvml.device_by_index(i).map_err(|e| {
                tracing::error!("Nvml::device_by_index({}) failed: {}", i, e);
                e
            })?;
            let brand = device.name().map_err(|e| {
                tracing::error!("Device::name() failed: {}", e);
                e
            })?;
            let utilization = device
                .utilization_rates()
                .map_err(|e| {
                    tracing::error!("Device::utilization_rates() failed: {}", e);
                    e
                })?
                .gpu as f64;
            let vram_total = device
                .memory_info()
                .map_err(|e| {
                    tracing::error!("Device::memory_info() failed (vram_total): {}", e);
                    e
                })?
                .total;
            let vram_utilization = device
                .memory_info()
                .map_err(|e| {
                    tracing::error!("Device::memory_info() failed (vram_utilization): {}", e);
                    e
                })?
                .used;
            let wattage = device.power_usage().map_err(|e| {
                tracing::error!("Device::power_usage() failed: {}", e);
                e
            })? as f64
                / 1000.0;
            let temperature = device
                .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
                .map_err(|e| {
                    tracing::error!("Device::temperature() failed: {}", e);
                    e
                })? as f64;

            let mut processes = Vec::new();
            for process in device.process_utilization_stats(None).map_err(|e| {
                tracing::error!("Device::process_utilization_stats() failed: {}", e);
                e
            })? {
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
