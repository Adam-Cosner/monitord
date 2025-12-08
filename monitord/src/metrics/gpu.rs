use crate::error::Result;
mod amd;
mod intel;
mod nvidia;

struct GpuMetricCollector {
    gpu_ids: Vec<String>,
    nvidia_collector: Option<nvidia::NvidiaMetricCache>,
    amd_collector: Option<amd::AmdMetricCache>,
    intel_collector: Option<intel::IntelMetricCache>,
}

impl GpuMetricCollector {
    pub fn new() -> Result<Self> {
        // Iterate over drm devices
        let mut gpu_ids = Vec::new();
        for entry in std::fs::read_dir("/dev/dri")? {
            let entry = entry?;
            if entry.file_name().to_string_lossy().starts_with("card") {
                let card_id = entry.file_name().to_string_lossy().to_string();
                gpu_ids.push(card_id);
            }
        }

        Ok(GpuMetricCollector {
            gpu_ids,
            nvidia_collector: nvidia::NvidiaMetricCache::new().ok(),
            amd_collector: amd::AmdMetricCache::new().ok(),
            intel_collector: intel::IntelMetricCache::new().ok(),
        })
    }

    pub fn collect(
        &mut self,
        request: &monitord_types::service::GpuRequest,
    ) -> Result<Vec<monitord_types::service::GpuResponse>> {
        let mut responses = Vec::new();
        for id in &self.gpu_ids {
            // Check the vendor ID in the /sys/class/drm/<card>/device/vendor
            let vendor_id =
                std::fs::read_to_string(format!("/sys/class/drm/{}/device/vendor", id))?;
            let vendor_id = vendor_id.trim();

            // Get the PCI bus ID in /sys/class/drm/<card>/device/uevent
            let bus_id = std::fs::read_to_string(format!("/sys/class/drm/{}/device/uevent", id))?;
            let bus_id = bus_id
                .lines()
                .find(|line| line.starts_with("PCI_SLOT_NAME="))
                .ok_or_else(|| {
                    crate::error::Error::Parse("Could not parse PCI bus ID".to_string())
                })?
                .split('=')
                .nth(1)
                .ok_or_else(|| {
                    crate::error::Error::Parse("Could not parse PCI bus ID".to_string())
                })?
                .trim()
                .to_string();

            match vendor_id {
                "0x1002" => {
                    if let Some(amd_collector) = &mut self.amd_collector {
                        let collected = amd_collector.collect(bus_id.clone(), request);
                        if let Ok(collected) = collected {
                            responses.push(collected);
                        } else {
                            tracing::error!(
                                "Failed to collect AMD GPU metrics for bus ID {}: {}",
                                bus_id,
                                collected.err().unwrap()
                            );
                        }
                    }
                }
                "0x8086" => {
                    if let Some(intel_collector) = &mut self.intel_collector {
                        let collected = intel_collector.collect(bus_id.clone(), request);
                        if let Ok(collected) = collected {
                            responses.push(collected);
                        } else {
                            tracing::error!(
                                "Failed to collect Intel GPU metrics for bus ID {}: {}",
                                bus_id,
                                collected.err().unwrap()
                            );
                        }
                    }
                }
                "0x10de" => {
                    if let Some(nvidia_collector) = &mut self.nvidia_collector {
                        let collected = nvidia_collector.collect(bus_id.clone(), request);
                        if let Ok(collected) = collected {
                            responses.push(collected);
                        } else {
                            tracing::error!(
                                "Failed to collect NVIDIA GPU metrics for bus ID {}: {}",
                                bus_id,
                                collected.err().unwrap()
                            );
                        }
                    }
                }
                _ => continue,
            }
        }
        Ok(responses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() -> Result<()> {
        let collector = GpuMetricCollector::new()?;
        println!("GPU IDs: {:?}", collector.gpu_ids);
        Ok(())
    }

    #[test]
    fn test_collect() -> Result<()> {
        let mut collector = GpuMetricCollector::new()?;
        let request = monitord_types::service::GpuRequest { process_data: true };

        let _ = collector.collect(&request);
        std::thread::sleep(std::time::Duration::from_millis(1000));

        let response = collector.collect(&request)?;
        println!("GPU Response: {:?}", response);
        Ok(())
    }
}
