//! Common data models for the communication module

use monitord_protocols::subscription::TransportType as ProtoTransportType;
use std::time::Instant;

/// Represents a client connection to the service
#[derive(Debug, Clone)]
pub struct ClientConnection {
    /// Client identifier
    pub client_id: String,
    /// Process ID of the client
    pub pid: u32,
    /// Time when the connection was established
    pub connected_at: Instant,
    /// Transport type used by this client
    pub transport_type: TransportType,
}

/// Enum representing different transport types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportType {
    /// Iceoryx shared memory transport
    Iceoryx,
    /// gRPC transport
    Grpc,
}

impl From<ProtoTransportType> for TransportType {
    fn from(proto_type: ProtoTransportType) -> Self {
        match proto_type {
            ProtoTransportType::Iceoryx => TransportType::Iceoryx,
            ProtoTransportType::Grpc => TransportType::Grpc,
        }
    }
}

impl From<TransportType> for ProtoTransportType {
    fn from(transport_type: TransportType) -> Self {
        match transport_type {
            TransportType::Iceoryx => ProtoTransportType::Iceoryx,
            TransportType::Grpc => ProtoTransportType::Grpc,
        }
    }
}

/// Enum representing different types of system data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataType {
    /// Overall system information
    System,
    /// CPU information
    Cpu,
    /// Memory information
    Memory,
    /// GPU information
    Gpu,
    /// Network information
    Network,
    /// Process information
    Process,
    /// Storage information
    Storage,
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::System => write!(f, "system"),
            DataType::Cpu => write!(f, "cpu"),
            DataType::Memory => write!(f, "memory"),
            DataType::Gpu => write!(f, "gpu"),
            DataType::Network => write!(f, "network"),
            DataType::Process => write!(f, "process"),
            DataType::Storage => write!(f, "storage"),
        }
    }
}