use crate::error::CollectionError;
use monitord_protocols::monitord::GpuInfo;

pub struct NvidiaGpuCollector {
    nvml: nvml_wrapper::Nvml,
}

impl NvidiaGpuCollector {
    pub fn new() -> Result<Self, CollectionError> {
        Err(CollectionError::Disabled)
    }
}

impl super::VendorGpuCollector for NvidiaGpuCollector {
    fn init(&mut self) -> Result<(), CollectionError> {
        todo!()
    }

    fn collect(&mut self) -> Result<Vec<GpuInfo>, CollectionError> {
        todo!()
    }
}
