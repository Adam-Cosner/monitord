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

    pub fn collect(&mut self) -> Result<Vec<monitord_types::service::DiskResponse>> {
        self.disks.refresh(true);
        let mut disks = Vec::new();

        for disk in self.disks.list().iter() {
            let name = disk.name().to_string_lossy().to_string();
            let capacity = disk.total_space();
            let total_read = disk.usage().total_read_bytes;
            let reading = disk.usage().read_bytes;
            let total_write = disk.usage().total_written_bytes;
            let writing = disk.usage().written_bytes;

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
        let mut metric_cache = DiskMetricCollector::new()?;
        let _ = metric_cache.collect()?;
        // pause to allow second capture for usage info
        std::thread::sleep(std::time::Duration::from_secs(1));
        let disk_metrics = metric_cache.collect()?;

        println!("{:?}", disk_metrics);
        Ok(())
    }
}
