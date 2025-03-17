mod iceoryx;
mod grpc;
mod intra;

pub(crate) use iceoryx::IceoryxTransport;

pub enum TransportVariant {
    Iceoryx(IceoryxTransport),
    Grpc(/*GrpcTransport*/),
    Intra(/*IntraTransport*/),
}