//! Connection management for the monitord client

pub mod config;

use crate::{config::ClientConfig, error::ClientError, transport::TransportLayer, Result};

/// Manages the connection to the monitord service
#[derive(Debug)]
pub(crate) struct Connection {
    config: ClientConfig,
    transport: TransportLayer,
}

impl Connection {
    /// Creates a new connection with the given configuration
    pub async fn new(config: ClientConfig) -> Result<Self> {
        let transport = TransportLayer::new(
            config.transport.transport_type,
            &config.connection.address,
            config.connection.port,
        )
        .await?;

        Ok(Self { config, transport })
    }

    /// Checks if the connection is currently active
    pub fn is_connected(&self) -> bool {
        self.transport.is_connected()
    }

    /// Attempts to reconnect to the service
    pub async fn reconnect(&mut self) -> Result<()> {
        if !self.config.connection.enable_reconnect {
            return Err(ClientError::ConnectionError(
                "Automatic reconnection is disabled".to_string(),
            ));
        }

        // Close the existing connection
        let _ = self.close().await;

        // Create a new transport
        self.transport = TransportLayer::new(
            self.config.transport.transport_type,
            &self.config.connection.address,
            self.config.connection.port,
        )
        .await?;

        Ok(())
    }

    /// Closes the connection
    pub async fn close(&mut self) -> Result<()> {
        self.transport.close().await
    }

    /// Returns the current connection configuration
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Sends a request to the monitord service
    pub async fn send_request(&self, req_type: &str, data: Vec<u8>) -> Result<Vec<u8>> {
        // TODO: Add request timeout handling
        self.transport.send_request(req_type, data).await
    }
}
