#[derive(Debug, Clone)]
pub struct CommuncationConfig {
    // Whether to publish metrics to iceoryx2
    iceoryx: Option<IceoryxConfig>,
    // Whether to publish metrics through gRPC
    grpc: Option<GrpcConfig>,
}

#[derive(Debug, Clone)]
pub struct IceoryxConfig {
    // todo
}

#[derive(Debug, Clone)]
pub struct GrpcConfig {
    // todo
}
