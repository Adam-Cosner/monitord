use crate::error::CollectionError;
use monitord_protocols::monitord::GpuInfo;

pub struct IntelGpuCollector {}

impl IntelGpuCollector {
    pub fn new() -> Result<Self, CollectionError> {
        Err(CollectionError::Disabled)
    }
}

impl super::VendorGpuCollector for IntelGpuCollector {
    fn init(&mut self) -> Result<(), CollectionError> {
        todo!()
    }

    fn collect(&mut self) -> Result<Vec<GpuInfo>, CollectionError> {
        todo!()
    }
}
