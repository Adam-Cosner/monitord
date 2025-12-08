use crate::error::Result;

pub struct DiskMetricCollector {
    disks: sysinfo::Disks,
}

impl DiskMetricCollector {
    pub fn new() -> Result<Self> {
        Ok(Self {
            disks: sysinfo::Disks::new_with_refreshed_list(),
        })
    }

    pub fn collect(
        &mut self,
        request: &monitord_types::service::DiskRequest,
    ) -> Result<Vec<monitord_types::service::DiskResponse>> {
        self.disks.refresh(true);
        let mut disks = Vec::new();

        for disk in self.disks.list().iter() {
            let name = disk.name().to_string_lossy().to_string();

            let capacity = if request.capacity {
                disk.total_space()
            } else {
                0
            };

            let total_read = if request.total_read {
                disk.usage().total_read_bytes
            } else {
                0
            };

            let reading = if request.reading {
                disk.usage().read_bytes
            } else {
                0
            };

            let total_write = if request.total_write {
                disk.usage().total_written_bytes
            } else {
                0
            };

            let writing = if request.writing {
                disk.usage().written_bytes
            } else {
                0
            };

            disks.push(monitord_types::service::DiskResponse {
                name,
                capacity,
                total_read,
                reading,
                total_write,
                writing,
            })
        }

        Ok(disks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_metrics() -> Result<()> {
        let request = monitord_types::service::DiskRequest {
            capacity: true,
            total_read: true,
            reading: true,
            total_write: true,
            writing: true,
        };

        let mut metric_cache = DiskMetricCollector::new()?;
        let _ = metric_cache.collect(&request)?;
        // pause to allow second capture for usage info
        std::thread::sleep(std::time::Duration::from_secs(1));
        let disk_metrics = metric_cache.collect(&request)?;

        println!("{:?}", disk_metrics);
        Ok(())
    }
}
