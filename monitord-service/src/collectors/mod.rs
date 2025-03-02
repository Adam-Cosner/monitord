use config::CollectorConfig;
use error::CollectionError;
use prost::Message;

pub trait Collector: Send + Sync {
    type CollectedData: Message + Send + Sync;

    async fn run(
        &mut self,
        channel: tokio::sync::broadcast::Sender<Self::CollectedData>,
    ) -> Result<(), CollectionError> {
        if !self.config().enabled {
            return Err(CollectionError::Disabled);
        }

        loop {
            let collected = self.collect()?;
            match channel.send(collected) {
                Ok(num) => {
                    tracing::debug!(
                        "Published collection data from {} collector to internal channels",
                        self.name()
                    )
                }
                Err(e) => {
                    return Err(CollectionError::ChannelError(e.to_string()));
                }
            }
            tokio::time::sleep(tokio::time::Duration::from(
                self.config().interval.to_std().unwrap(),
            ))
            .await;
        }
    }

    fn name(&self) -> &'static str;

    /// Get config
    fn config(&self) -> &CollectorConfig;

    /// Collect data
    fn collect(&mut self) -> Result<Self::CollectedData, CollectionError>;
}

pub mod cpu;
pub mod gpu;
pub mod memory;
pub mod network;
pub mod process;
pub mod storage;
pub mod system;

pub mod config;
pub mod error;
