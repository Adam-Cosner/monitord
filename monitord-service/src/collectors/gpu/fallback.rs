use crate::error::CollectionError;
use monitord_protocols::monitord::GpuInfo;

pub struct FallbackGpuCollector {}

impl FallbackGpuCollector {
    pub fn new() -> Result<Self, CollectionError> {
        todo!()
    }
}

impl super::VendorGpuCollector for FallbackGpuCollector {
    fn init(&mut self) -> Result<(), CollectionError> {
        todo!()
    }

    fn collect(&mut self) -> Result<Vec<GpuInfo>, CollectionError> {
        todo!()
    }
}
