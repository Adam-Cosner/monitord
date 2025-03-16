//! Monitord Client Library
//!
//! This library provides a client interface to connect to and interact with the monitord service.
//! It handles connection management, subscription creation, and data retrieval from the monitoring service.

pub mod client;
pub mod config;
pub(crate) mod connection;
pub mod error;
pub mod subscription;
pub(crate) mod transport;
pub mod types;

// Public API exports
pub use client::MonitordClient;
pub use config::ClientConfig;
pub use connection::config::ConnectionConfig;
pub use error::ClientError;
pub use subscription::config::SubscriptionConfig;
pub use subscription::{Subscription, SubscriptionBuilder, SubscriptionType};
pub use transport::config::{GrpcConfig, IceoryxConfig, TransportConfig, TransportType};
pub use types::{
    CpuInfo, GpuInfo, MemoryInfo, NetworkInfo, ProcessInfo, StorageInfo, SystemInfo, SystemSnapshot,
};

/// Re-export of the Result type with ClientError as the error type
pub type Result<T> = std::result::Result<T, error::ClientError>;

/// High-level convenience functions for common operations
pub mod prelude {
    pub use crate::client::MonitordClient;
    pub use crate::config::ClientConfig;
    pub use crate::connection::config::ConnectionConfig;
    pub use crate::subscription::config::SubscriptionConfig;
    pub use crate::subscription::{Subscription, SubscriptionBuilder, SubscriptionType};
    pub use crate::transport::config::{GrpcConfig, IceoryxConfig, TransportConfig, TransportType};
    pub use crate::types::{
        CpuInfo, GpuInfo, MemoryInfo, NetworkInfo, ProcessInfo, StorageInfo, SystemInfo,
        SystemSnapshot,
    };
    pub use crate::Result;
}

/// Creates a new client with default configuration
///
/// # Examples
///
/// ```
/// use monitord_client::prelude::*;
///
/// async fn example() -> Result<()> {
///     let client = monitord_client::connect().await?;
///     let snapshot = client.get_system_snapshot().await?;
///     println!("System: {}", snapshot.system_info.hostname);
///     Ok(())
/// }
/// ```
pub async fn connect() -> Result<client::MonitordClient> {
    let config = config::ClientConfig::default();
    client::MonitordClient::connect(config).await
}
