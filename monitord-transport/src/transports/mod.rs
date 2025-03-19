mod iceoryx;
mod grpc;
mod intra;
mod nng;

pub(crate) use iceoryx::IceoryxTransport;
pub(crate) use nng::NngTransport;

pub enum TransportVariant {
    Nng(NngTransport),
    Iceoryx(IceoryxTransport),
    Grpc(/*GrpcTransport*/),
    Intra(/*IntraTransport*/),
}