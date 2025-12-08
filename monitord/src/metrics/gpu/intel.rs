use crate::error::Result;

pub struct IntelMetricCache {
    // Implementation details
}

impl IntelMetricCache {
    pub fn new() -> Result<Self> {
        // Implementation details
        Ok(Self {})
    }

    pub fn collect(
        &self,
        id: String,
        request: &monitord_types::service::GpuRequest,
    ) -> Result<monitord_types::service::GpuResponse> {
        // Implementation details
        todo!()
    }
}
