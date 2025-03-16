use crate::communication::transport::{GrpcConfig, IceoryxConfig};

#[derive(Default)]
pub enum CommunicationBackend {
    #[default]
    Iceoryx(IceoryxConfig),
    Grpc(GrpcConfig),
}

pub struct CommunicationConfig {
    backend: CommunicationBackend,
}
