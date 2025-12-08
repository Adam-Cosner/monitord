use crate::error::Result;

pub struct AmdMetricCache {
    // Implementation details
}

impl AmdMetricCache {
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
        Err(crate::error::Error::NotImplemented(
            "AMD GPU metrics are not implemented".to_string(),
        ))
    }
}
