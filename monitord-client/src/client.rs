//! Main client implementation for interacting with the monitord service

use std::sync::Arc;

use crate::{
    config::ClientConfig,
    connection::Connection,
    error::ClientError,
    subscription::{Subscription, SubscriptionBuilder, SubscriptionType},
    types::{CpuInfo, GpuInfo, MemoryInfo, NetworkInfo, ProcessInfo, SystemSnapshot},
    Result,
};

/// Main client for interacting with the monitord service
#[derive(Debug, Clone)]
pub struct MonitordClient {
    // Use Arc to allow cloning the client
    connection: Arc<std::sync::Mutex<Connection>>,
    config: ClientConfig,
}

impl MonitordClient {
    /// Connects to the monitord service with the given configuration
    pub async fn connect(config: ClientConfig) -> Result<Self> {
        let connection = Connection::new(config.clone()).await?;
        Ok(Self {
            connection: Arc::new(std::sync::Mutex::new(connection)),
            config,
        })
    }

    /// Gets a single system snapshot
    pub async fn get_system_snapshot(&self) -> Result<SystemSnapshot> {
        // TODO: Implement system snapshot retrieval
        // This would involve serializing a request, sending it, and deserializing the response
        todo!("Implement system snapshot retrieval")
    }

    /// Gets the current CPU information
    pub async fn get_cpu_info(&self) -> Result<CpuInfo> {
        // TODO: Implement CPU info retrieval
        todo!("Implement CPU info retrieval")
    }

    /// Gets the current memory information
    pub async fn get_memory_info(&self) -> Result<MemoryInfo> {
        // TODO: Implement memory info retrieval
        todo!("Implement memory info retrieval")
    }

    /// Gets the current GPU information
    pub async fn get_gpu_info(&self) -> Result<Vec<GpuInfo>> {
        // TODO: Implement GPU info retrieval
        todo!("Implement GPU info retrieval")
    }

    /// Gets the current network information
    pub async fn get_network_info(&self) -> Result<Vec<NetworkInfo>> {
        // TODO: Implement network info retrieval
        todo!("Implement network info retrieval")
    }

    /// Gets information about running processes
    pub async fn get_process_info(&self) -> Result<Vec<ProcessInfo>> {
        // TODO: Implement process info retrieval
        todo!("Implement process info retrieval")
    }

    /// Creates a new subscription
    pub async fn subscribe(&self, builder: SubscriptionBuilder) -> Result<Subscription> {
        // Validation
        builder.validate()?;

        // TODO: Implement subscription creation
        todo!("Implement subscription creation")
    }

    /// Creates a subscription for system snapshots
    pub async fn subscribe_system(&self, interval_ms: Option<u32>) -> Result<Subscription> {
        let interval = interval_ms.unwrap_or(self.config.subscription.default_interval_ms);
        let builder = Subscription::builder()
            .subscription_type(SubscriptionType::System)
            .interval_ms(interval);

        self.subscribe(builder).await
    }

    /// Creates a subscription for CPU information
    pub async fn subscribe_cpu(&self, interval_ms: Option<u32>) -> Result<Subscription> {
        let interval = interval_ms.unwrap_or(self.config.subscription.default_interval_ms);
        let builder = Subscription::builder()
            .subscription_type(SubscriptionType::Cpu)
            .interval_ms(interval);

        self.subscribe(builder).await
    }

    /// Creates a subscription for memory information
    pub async fn subscribe_memory(&self, interval_ms: Option<u32>) -> Result<Subscription> {
        let interval = interval_ms.unwrap_or(self.config.subscription.default_interval_ms);
        let builder = Subscription::builder()
            .subscription_type(SubscriptionType::Memory)
            .interval_ms(interval);

        self.subscribe(builder).await
    }

    /// Unsubscribes from a subscription
    pub async fn unsubscribe(&self, _subscription: &Subscription) -> Result<()> {
        // TODO: Implement unsubscribe functionality
        todo!("Implement unsubscribe functionality")
    }

    /// Closes the client connection
    pub async fn close(self) -> Result<()> {
        // We need to unwrap the Arc to get the single reference to the connection
        // If this is the only reference, we can unwrap and close the connection
        if let Ok(mut connection) = Arc::try_unwrap(self.connection) {
            // We can unwrap the mutex safely here since we have the only reference
            let conn = connection.get_mut().unwrap();
            conn.close().await
        } else {
            Err(ClientError::ConnectionError(
                "Cannot close connection because there are other client references".to_string(),
            ))
        }
    }

    /// Returns the current client configuration
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Checks if the client is currently connected
    pub fn is_connected(&self) -> bool {
        if let Ok(connection) = self.connection.lock() {
            connection.is_connected()
        } else {
            false
        }
    }
}
