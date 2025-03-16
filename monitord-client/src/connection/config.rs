//! Configuration for connection management

/// Configuration options for connection management
#[derive(Debug, Clone, Default)]
pub struct ConnectionConfig {
    /// Address of the monitord service (default: "localhost")
    pub address: String,

    /// Port of the monitord service (default: 50051)
    pub port: u16,

    /// Connection timeout in milliseconds (default: 5000)
    pub connection_timeout_ms: u32,

    /// Request timeout in milliseconds (default: 10000)
    pub request_timeout_ms: u32,

    /// Enable automatic reconnection (default: true)
    pub enable_reconnect: bool,

    /// Maximum number of reconnection attempts (default: 5)
    pub max_reconnect_attempts: u32,
}

impl ConnectionConfig {
    /// Creates a new connection configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the address for the monitord service
    pub fn with_address(mut self, address: impl Into<String>) -> Self {
        self.address = address.into();
        self
    }

    /// Sets the port for the monitord service
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets the connection timeout in milliseconds
    pub fn with_connection_timeout(mut self, timeout_ms: u32) -> Self {
        self.connection_timeout_ms = timeout_ms;
        self
    }

    /// Sets the request timeout in milliseconds
    pub fn with_request_timeout(mut self, timeout_ms: u32) -> Self {
        self.request_timeout_ms = timeout_ms;
        self
    }

    /// Enables or disables automatic reconnection
    pub fn with_reconnect(mut self, enable: bool) -> Self {
        self.enable_reconnect = enable;
        self
    }

    /// Sets the maximum number of reconnection attempts
    pub fn with_max_reconnect_attempts(mut self, attempts: u32) -> Self {
        self.max_reconnect_attempts = attempts;
        self
    }
}

// Default implementation
impl ConnectionConfig {
    fn default() -> Self {
        Self {
            address: "localhost".to_string(),
            port: 50051,
            connection_timeout_ms: 5000,
            request_timeout_ms: 10000,
            enable_reconnect: true,
            max_reconnect_attempts: 5,
        }
    }
}
