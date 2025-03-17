use crate::error::{TransportError};

/// Transport trait defines the interface for the transport mechanisms
pub trait Transport: Send + Sync + 'static {
    /// Initialize the transport mechanism
    async fn initialize(&mut self) -> Result<(), TransportError>;

    /// Publish data to a specific topic
    async fn publish(&self, topic: &str, message: &[u8]) -> Result<(), TransportError>;

    /// Receive data from a specific topic
    async fn receive(&self, topic: &str) -> Result<Option<Vec<u8>>, TransportError>;

    /// Get the name of the transport method for identification purposes
    fn name(&self) -> &str;

    /// Check if transport layer is active
    fn is_active(&self) -> bool;
}