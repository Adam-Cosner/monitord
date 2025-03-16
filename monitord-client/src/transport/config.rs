//! Configuration for transport options

/// Transport protocols supported by the client
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportType {
    /// gRPC transport (network capability)
    Grpc,

    /// Iceoryx transport (shared memory, local host only)
    Iceoryx,
}

impl Default for TransportType {
    fn default() -> Self {
        TransportType::Grpc
    }
}

/// Configuration for transport options
#[derive(Debug, Clone, Default)]
pub struct TransportConfig {
    /// Transport type to use
    pub transport_type: TransportType,

    /// gRPC-specific configuration options
    pub grpc: GrpcConfig,

    /// Iceoryx-specific configuration options
    pub iceoryx: IceoryxConfig,
}

/// Configuration options for gRPC transport
#[derive(Debug, Clone, Default)]
pub struct GrpcConfig {
    /// Use TLS for connection (default: false)
    pub use_tls: bool,
}

/// Configuration options for Iceoryx transport
#[derive(Debug, Clone, Default)]
pub struct IceoryxConfig {
    /// Service instance name (default: "monitord-client")
    pub instance_name: String,
}

impl TransportConfig {
    /// Creates a new transport configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the transport type
    pub fn with_transport_type(mut self, transport_type: TransportType) -> Self {
        self.transport_type = transport_type;
        self
    }

    /// Sets the gRPC configuration
    pub fn with_grpc_config(mut self, grpc: GrpcConfig) -> Self {
        self.grpc = grpc;
        self
    }

    /// Sets the Iceoryx configuration
    pub fn with_iceoryx_config(mut self, iceoryx: IceoryxConfig) -> Self {
        self.iceoryx = iceoryx;
        self
    }
}

impl GrpcConfig {
    /// Creates a new gRPC configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether to use TLS for the connection
    pub fn with_tls(mut self, use_tls: bool) -> Self {
        self.use_tls = use_tls;
        self
    }
}

impl IceoryxConfig {
    /// Creates a new Iceoryx configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the service instance name
    pub fn with_instance_name(mut self, name: impl Into<String>) -> Self {
        self.instance_name = name.into();
        self
    }
}

// Default implementations
impl TransportConfig {
    fn default() -> Self {
        Self {
            transport_type: TransportType::Grpc,
            grpc: GrpcConfig::default(),
            iceoryx: IceoryxConfig::default(),
        }
    }
}

impl GrpcConfig {
    fn default() -> Self {
        Self { use_tls: false }
    }
}

impl IceoryxConfig {
    fn default() -> Self {
        Self {
            instance_name: "monitord-client".to_string(),
        }
    }
}
