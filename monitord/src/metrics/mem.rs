use crate::error::Result;

pub struct MemoryMetricCollector {
    sys: sysinfo::System,
}

impl MemoryMetricCollector {
    pub fn new() -> Result<Self> {
        Ok(Self {
            sys: sysinfo::System::new_with_specifics(
                sysinfo::RefreshKind::nothing()
                    .with_memory(sysinfo::MemoryRefreshKind::everything()),
            ),
        })
    }

    pub fn collect(
        &mut self,
        request: &monitord_types::service::MemoryRequest,
    ) -> Result<Option<monitord_types::service::MemoryResponse>> {
        self.sys.refresh_memory();

        let capacity = if request.capacity {
            self.sys.total_memory()
        } else {
            0
        };

        let in_use = if request.in_use {
            self.sys.used_memory()
        } else {
            0
        };

        let sw_capacity = if request.sw_capacity {
            self.sys.total_swap()
        } else {
            0
        };

        let sw_use = if request.sw_use {
            self.sys.used_swap()
        } else {
            0
        };

        Ok(Some(monitord_types::service::MemoryResponse {
            capacity,
            in_use,
            sw_capacity,
            sw_use,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mem_metrics() -> Result<()> {
        let request = monitord_types::service::MemoryRequest {
            capacity: true,
            in_use: true,
            sw_capacity: true,
            sw_use: true,
        };

        let mut metric_cache = MemoryMetricCollector::new()?;
        let mem_metrics = metric_cache.collect(&request)?;

        println!("{:?}", mem_metrics);

        Ok(())
    }
}
