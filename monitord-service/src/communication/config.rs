//! Configuration structures for communication module

use std::time::Duration;

pub use super::subscription::config::SubscriptionConfig;

/// Main configuration for the communication module
#[derive(Debug, Clone)]
pub struct CommunicationConfig {
    /// The time between refreshes of the connection listeners
    pub connection_frequency: Duration,
    /// Whether to publish metrics to iceoryx2
    pub iceoryx: Option<IceoryxConfig>,
    /// Whether to publish metrics through gRPC
    pub grpc: Option<GrpcConfig>,
    /// The subscription manager configuration
    pub subscription: SubscriptionConfig,
}

/// Configuration for Iceoryx transport
#[derive(Debug, Clone)]
pub struct IceoryxConfig {
    /// Service name for iceoryx2 communication
    pub service_name: String,
    /// Max buffer size per connection
    pub buffer_size: usize,
}

/// Configuration for gRPC transport
#[derive(Debug, Clone)]
pub struct GrpcConfig {
    /// Address to bind the gRPC server to
    pub address: String,
    /// Port to bind the gRPC server to
    pub port: u16,
    /// Max connections
    pub max_connections: usize,
    /// Timeout in seconds for connections
    pub connection_timeout_secs: u64,
    /// Enable TLS
    pub enable_tls: bool,
    /// TLS certificate path (if TLS is enabled)
    pub tls_cert_path: Option<String>,
    /// TLS key path (if TLS is enabled)
    pub tls_key_path: Option<String>,
}

impl Default for CommunicationConfig {
    fn default() -> Self {
        Self {
            connection_frequency: Duration::from_millis(100),
            iceoryx: Some(IceoryxConfig {
                service_name: "monitord".to_string(),
                buffer_size: 1024 * 1024,
            }),
            grpc: None,
            subscription: SubscriptionConfig::default(),
        }
    }
}
