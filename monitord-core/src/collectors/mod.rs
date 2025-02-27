use anyhow::Result;
use prost::Message;
use std::fmt::Debug;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;
use tracing::{error, info};

/// Trait that must be implemented by all hardware collectors
pub trait Collector: Debug + Send + Sync {
    /// The type of protobuf message this collector produces
    type Output: Message + Default + Clone + Send + 'static;

    /// Initialize the collector with its required resources
    fn init(&mut self) -> Result<()>;

    /// Collect data once and return the appropriate protobuf message
    fn collect(&self) -> Result<Self::Output>;

    /// Clean up resources when the collector is no longer needed
    fn shutdown(&mut self) -> Result<()>;

    /// Check if this collector is available on the current system
    fn is_available(&self) -> bool;

    /// Get the name of this collector for logging and diagnostics
    fn name(&self) -> &str;

    /// Get the collection interval in milliseconds
    fn interval_ms(&self) -> u32;

    /// Set the collection interval in milliseconds
    fn set_interval_ms(&mut self, interval_ms: u32);

    /// Start collecting data at the configured interval, sending results to the provided channel
    fn start_collecting(
        &mut self,
        tx: mpsc::Sender<Self::Output>,
    ) -> Result<tokio::task::JoinHandle<()>> {
        if !self.is_available() {
            return Err(anyhow::anyhow!(
                "Collector {} is not available on this system",
                self.name()
            ));
        }

        let name = self.name().to_string();
        let interval = Duration::from_millis(self.interval_ms() as u64);

        // Create a clone of self that can be moved into the async task
        // This is a bit tricky since we can't clone self directly
        // In practice, each collector implementation will need to provide
        // a way to create a reference or clone that can be used inside the task
        let collector_ref = self.get_async_collector_ref()?;

        // Start a background task that collects data at the specified interval
        let handle = tokio::spawn(async move {
            info!(
                "Starting collection for {} at {:?} intervals",
                name, interval
            );
            let mut interval_timer = time::interval(interval);

            loop {
                interval_timer.tick().await;

                match collector_ref.collect() {
                    Ok(data) => {
                        if let Err(e) = tx.send(data).await {
                            error!("Failed to send {} data: {}", name, e);
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Failed to collect {} data: {}", name, e);
                    }
                }
            }

            info!("Stopping collection for {}", name);
        });

        Ok(handle)
    }

    /// Get a reference or clone of this collector that can be used in an async task
    /// This is necessary because we can't move self directly into the async task
    fn get_async_collector_ref(&self) -> Result<Box<dyn CollectorRef<Output = Self::Output>>>;
}

/// Reference to a collector that can be used in an async task
pub trait CollectorRef: Send + Sync {
    /// The type of protobuf message this collector produces
    type Output: Message + Default + Clone + Send + 'static;

    /// Collect data once and return the appropriate protobuf message
    fn collect(&self) -> Result<Self::Output>;

    /// Get the name of this collector
    fn name(&self) -> &str;
}

/// Base configuration for all collectors
#[derive(Debug, Clone)]
pub struct CollectorConfig {
    /// Whether this collector is enabled
    pub enabled: bool,

    /// Collection interval in milliseconds
    pub interval_ms: u32,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_ms: 1000, // Default to 1 second
        }
    }
}

pub mod cpu;
pub mod gpu;
pub mod memory;
pub mod network;
pub mod process;
pub mod storage;
pub mod system;
