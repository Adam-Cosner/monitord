use prost::Message;
use monitord_protocols::subscription;
use crate::communication::manager::CommunicationManager;
use crate::config::ClientConfig;
use crate::error::ClientError;

pub struct MonitordClient {
    communication_manager: CommunicationManager,
}

impl MonitordClient {
    pub fn connect(config: ClientConfig) -> Result<MonitordClient, ClientError> {
        Ok(Self {
            communication_manager: CommunicationManager::new(config.communication_config)?,
        })
    }

    /// Function for subscribing to single metrics, like SystemInfo or CpuInfo
    pub async fn subscribe<T: Message>(&self, filter: subscription::SubscriptionRequest) -> Result<futures::channel::mpsc::Receiver<T>, ClientError> {
        todo!()
    }

    /// Function for subscribing to metrics that contain a list of objects, like GpuInfo or NetworkInfo
    pub async fn subscribe_collection<T: Message>(&self, filter: subscription::SubscriptionRequest) -> Result<futures::channel::mpsc::Receiver<Vec<T>>, ClientError> {
        todo!()
    }

    /// Function for modifying a subscription's information. Will modify the channel inline so no need to get a new channel
    pub async fn modify_subscription(&mut self, filter: subscription::ModifySubscriptionRequest) -> Result<(), ClientError> {
        todo!()
    }

    /// Function for unsubscribing from a metric. Channel will be dropped
    pub async fn unsubscribe(&mut self, request: subscription::UnsubscribeRequest) -> Result<(), ClientError> {
        todo!()
    }
}

