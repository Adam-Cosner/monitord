use crate::communication::core::ClientConnection;
use crate::communication::error::CommunicationError;
use async_trait::async_trait;

/// Transport trait defines the interface for different transport mechanisms
#[async_trait]
pub trait Transport: Send + Sync + 'static {
    /// Initialize the transport
    async fn initialize(&mut self) -> Result<(), CommunicationError>;

    /// Publish data to a specific topic
    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<(), CommunicationError>;

    /// Listen for client connection requests
    async fn listen_for_connections(&self) -> Result<Option<ClientConnection>, CommunicationError>;

    /// Send a response to a specific client
    async fn send_response(
        &self,
        client_id: &str,
        response: &[u8],
    ) -> Result<(), CommunicationError>;

    /// Get transport name for identification
    fn name(&self) -> &str;

    /// Check if transport is active
    fn is_active(&self) -> bool;
}

/// Enum representing the different message types we handle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    CpuInfo,
    MemoryInfo,
    GpuInfo,
    NetworkInfo,
    ProcessInfo,
    StorageInfo,
    SystemInfo,
    // Other message types
}

pub trait MessageHandler: Send + Sync + 'static {
    fn serialize_bytes(
        &self,
        message_type: MessageType,
        message_bytes: Vec<u8>,
    ) -> Result<Vec<u8>, CommunicationError>;

    fn deserialize_bytes(
        &self,
        message_type: MessageType,
        data: &[u8],
    ) -> Result<Vec<u8>, CommunicationError>;
}

/// Helper functions for MessageHandler
pub mod message_utils {
    use super::*;
    use prost::Message;

    pub fn serialize<T: Message, H: MessageHandler + ?Sized>(
        handler: &H,
        message_type: MessageType,
        message: &T,
    ) -> Result<Vec<u8>, CommunicationError> {
        let bytes = message.encode_to_vec();
        handler.serialize_bytes(message_type, bytes)
    }

    pub fn deserialize<T: Message + Default, H: MessageHandler>(
        handler: &H,
        message_type: MessageType,
        data: &[u8],
    ) -> Result<T, CommunicationError> {
        let bytes = handler.deserialize_bytes(message_type, data)?;
        T::decode(&bytes[..]).map_err(|e| CommunicationError::Deserialization(e.to_string()))
    }
}
