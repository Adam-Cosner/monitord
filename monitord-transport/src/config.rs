
#[derive(Debug, Clone)]
pub enum TransportType {
    Iceoryx(IceoryxConfig),
    Grpc,
    Intra,
}

impl Default for TransportType {
    fn default() -> Self {
        Self::Iceoryx(IceoryxConfig::default())
    }
}

#[derive(Debug, Clone, Default)]
pub struct TransportConfig {
    pub(crate) transport_config: TransportType,
}

/// Configuration for Iceoryx transport
#[derive(Debug, Clone)]
pub struct IceoryxConfig {
    /// Service name for iceoryx2 communication
    pub service_name: String,
    /// Max buffer size per connection
    pub buffer_size: usize,
}

impl Default for IceoryxConfig {
    fn default() -> Self {
        Self {
            service_name: uuid::Uuid::new_v4().to_string(),
            buffer_size: 1024 * 1024,
        }
    }
}