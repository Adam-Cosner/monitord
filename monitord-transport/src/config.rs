
#[derive(Debug, Clone)]
pub enum TransportType {
    Nng(NngConfig),
    Iceoryx(IceoryxConfig),
    Grpc,
    Intra,
}

impl Default for TransportType {
    fn default() -> Self {
        Self::Nng(NngConfig::default())
    }
}

#[derive(Debug, Clone, Default)]
pub struct TransportConfig {
    pub(crate) transport_config: TransportType,
}

#[derive(Debug, Clone)]
pub struct NngConfig {
    /// Transport method (ipc, tcp, ws)
    pub transport: String,
    /// URL portion
    pub url: String,
    /// Timeout for operations in milliseconds
    pub timeout_ms: u32,
}

impl Default for NngConfig {
    fn default() -> Self {
        Self {
            transport: "ipc".to_string(),
            #[cfg(unix)]
            url: "/tmp/monitord".to_string(),
            #[cfg(windows)]
            topic_format: "monitord".to_string(),
            timeout_ms: 1000,
        }
    }
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