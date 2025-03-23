#[derive(Debug, Clone)]
pub struct GrpcConfig {
    /// Server address for gRPC transport (host:port format)
    pub server_address: String,
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            server_address: "127.0.0.1:50051".to_string(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CommunicationConfig {
    pub grpc_config: GrpcConfig,
}
