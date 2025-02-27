use crate::error::CollectionError;
use prost::Message;

pub trait Collector: Send + Sync {
    type CollectedData: Message + Send + Sync;

    /// The name of the collector, used mainly for logging
    fn name(&self) -> &'static str;
    /// Whether the collector is enabled
    fn is_enabled(&self) -> bool;
    /// Set the collector's enabled state
    fn set_enabled(&mut self, enabled: bool);
    /// Collect
    fn collect(&mut self) -> Result<Self::CollectedData, CollectionError>;
}

pub mod cpu;
pub mod gpu;
pub mod memory;
pub mod network;
pub mod process;
pub mod storage;
