use error::CollectionError;
use prost::Message;

pub trait Collector: Send + Sync {
    type CollectedData: Message + Send + Sync;
    type CollectorConfig;

    fn name(&self) -> &'static str;

    /// Get config
    fn config(&self) -> &Self::CollectorConfig;

    /// Collect data
    fn collect(&mut self) -> Result<Self::CollectedData, CollectionError>;
}

mod cpu;
mod gpu;
mod memory;
mod network;
mod process;
mod storage;
mod system;

pub mod config;
pub mod error;

mod manager;
pub use manager::CollectorManager;
