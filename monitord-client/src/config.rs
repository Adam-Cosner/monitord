//! Configuration for the monitord client

use crate::connection::config::ConnectionConfig;
use crate::subscription::config::SubscriptionConfig;
use crate::transport::config::{TransportConfig, TransportType};

/// Top-level configuration for the monitord client
#[derive(Debug, Clone, Default)]
pub struct ClientConfig {
    /// Connection configuration
    pub connection: ConnectionConfig,

    /// Transport configuration
    pub transport: TransportConfig,

    /// Subscription configuration
    pub subscription: SubscriptionConfig,
}

impl ClientConfig {
    /// Creates a new client configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the connection configuration
    pub fn with_connection(mut self, connection: ConnectionConfig) -> Self {
        self.connection = connection;
        self
    }

    /// Sets the transport configuration
    pub fn with_transport(mut self, transport: TransportConfig) -> Self {
        self.transport = transport;
        self
    }

    /// Sets the subscription configuration
    pub fn with_subscription(mut self, subscription: SubscriptionConfig) -> Self {
        self.subscription = subscription;
        self
    }

    // Shorthand methods for backward compatibility

    /// Sets the address for the monitord service
    pub fn with_address(mut self, address: impl Into<String>) -> Self {
        self.connection = self.connection.with_address(address);
        self
    }

    /// Sets the port for the monitord service
    pub fn with_port(mut self, port: u16) -> Self {
        self.connection = self.connection.with_port(port);
        self
    }

    /// Sets the connection timeout in milliseconds
    pub fn with_connection_timeout(mut self, timeout_ms: u32) -> Self {
        self.connection = self.connection.with_connection_timeout(timeout_ms);
        self
    }

    /// Sets the request timeout in milliseconds
    pub fn with_request_timeout(mut self, timeout_ms: u32) -> Self {
        self.connection = self.connection.with_request_timeout(timeout_ms);
        self
    }

    /// Sets the default interval for subscriptions in milliseconds
    pub fn with_default_interval(mut self, interval_ms: u32) -> Self {
        self.subscription = self.subscription.with_default_interval(interval_ms);
        self
    }

    /// Sets the transport type
    pub fn with_transport_type(mut self, transport: TransportType) -> Self {
        self.transport = self.transport.with_transport_type(transport);
        self
    }

    /// Enables or disables automatic reconnection
    pub fn with_reconnect(mut self, enable: bool) -> Self {
        self.connection = self.connection.with_reconnect(enable);
        self
    }

    /// Sets the maximum number of reconnection attempts
    pub fn with_max_reconnect_attempts(mut self, attempts: u32) -> Self {
        self.connection = self.connection.with_max_reconnect_attempts(attempts);
        self
    }
}
