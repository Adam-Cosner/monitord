mod iceoryx;
mod grpc;

pub(crate) use iceoryx::IceoryxTransport;
pub(crate) use iceoryx::IceoryxConfig;
pub(crate) use grpc::GrpcTransport;
pub(crate) use grpc::GrpcConfig;

pub(crate) enum TransportVariant {
    Iceoryx(IceoryxTransport),
    Grpc(GrpcTransport),
}

pub(crate) trait Transport: Send + Sync {

}