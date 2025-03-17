//! Common data models for the transport layer
use monitord_protocols::subscription::TransportType as ProtoTransportType;

/// Represents a connection
#[derive(Debug, Clone)]
pub struct ServiceConnection {
    /// The unique identifier of the remote process (this is always "monitord" if the remote is the service)
    pub remote_id: String,
    /// The time when the connection was established
    pub connected_at: chrono::DateTime<chrono::Utc>,
    /// The transport type used by this client
    pub transport_type: TransportType,
}

/// Enum representing different transport types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportType {
    /// Iceoryx shared memory transport
    Iceoryx,
    /// gRPC transport
    Grpc,
    /// Intra-process transport, for testing purposes only
    Intra,
}

impl From<ProtoTransportType> for TransportType {
    fn from(transport: ProtoTransportType) -> Self {
        match transport {
            ProtoTransportType::Iceoryx => Self::Iceoryx,
            ProtoTransportType::Grpc => Self::Grpc,
            ProtoTransportType::Intra => Self::Intra,
        }
    }
}