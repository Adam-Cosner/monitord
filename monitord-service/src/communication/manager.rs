use super::{config::CommunicationConfig, error::CommunicationError};
use crate::communication::iceoryx::IceoryxSubscriptionRequest;
use monitord_protocols::monitord::{
    CpuInfo, GpuInfo, MemoryInfo, NetworkInfo, ProcessInfo, StorageInfo, SystemInfo,
};
use monitord_protocols::subscription::{SubscriptionType, TransportType};
use tokio::sync::broadcast::Receiver;
use tokio::sync::{Mutex, RwLock};
use tracing::info;

pub struct CommunicationManager {
    config: CommunicationConfig,
    iceoryx: Option<Mutex<super::iceoryx::IceoryxManager>>,
    pub iceoryx_subscription_tx: tokio::sync::broadcast::Sender<IceoryxSubscriptionRequest>,
    grpc: Option<Mutex<super::grpc::GrpcService>>,
    subscription_manager: RwLock<super::subscription::SubscriptionManager>,
}

impl CommunicationManager {
    pub fn init(config: CommunicationConfig) -> Result<Self, CommunicationError> {
        let iceoryx = if let Some(iceoryx_config) = config.iceoryx.clone() {
            Some(Mutex::new(super::iceoryx::IceoryxManager::init(
                iceoryx_config,
            )?))
        } else {
            None
        };

        let grpc = if let Some(_grpc_config) = config.grpc.clone() {
            // Implementation for gRPC would go here
            None
        } else {
            None
        };

        let subscription_manager = RwLock::new(super::subscription::SubscriptionManager::init(
            config.subscription.clone(),
        )?);

        let (iceoryx_subscription_tx, _) = tokio::sync::broadcast::channel(100);

        Ok(Self {
            config,
            iceoryx,
            iceoryx_subscription_tx,
            grpc,
            subscription_manager,
        })
    }

    pub async fn run(
        &self,
        mut iceoryx_subscription_rx: Receiver<IceoryxSubscriptionRequest>,
        mut cpu_rx: Receiver<CpuInfo>,
        mut memory_rx: Receiver<MemoryInfo>,
    ) -> Result<(), CommunicationError> {
        tokio::select! {
            cpu_info = async {
                loop {
                    match cpu_rx.recv().await {
                        Ok(info) => self.publish_cpu_info(info).await?,
                        Err(e) => return Err::<(), CommunicationError>(CommunicationError::ReceiveError(e.to_string())),
                    }
                }
            } => {
                match cpu_info {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
            }
            memory_info = async {
                loop {
                    match memory_rx.recv().await {
                        Ok(info) => self.publish_memory_info(info).await?,
                        Err(e) => return Err::<(), CommunicationError>(CommunicationError::ReceiveError(e.to_string())),
                    }
                }
            } => {
                match memory_info {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
            }
            iceoryx_connections = async {
                info!("Responding to connections");
                loop {
                    tokio::time::sleep(self.config.connection_frequency).await;
                    match self.iceoryx.as_ref() {
                        Some(iceoryx) => {
                                match iceoryx.lock().await.respond_to_connections().await {
                                    Ok(_) => tokio::task::yield_now().await,
                                    Err(e) => return Err::<(), CommunicationError>(e),
                                }
                        }
                        None => {
                            tokio::task::yield_now().await;
                        }
                    }
                }
            } => {
                match iceoryx_connections {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
            }
            iceoryx_subscriptions = async {
                loop {
                    match iceoryx_subscription_rx.recv().await {
                        Ok(subscription_request) => {
                            match subscription_request {
                                IceoryxSubscriptionRequest::NewSubscription((client_id, request)) => {
                                    let response = self.subscription_manager.write().await.create_subscription(client_id.clone(), request, TransportType::Iceoryx).await?;
                                    if let Some(iceoryx) = &self.iceoryx {
                                        iceoryx.lock().await.send_subscribe_response(client_id, response).await?;
                                    }
                                    todo!()
                                }
                                IceoryxSubscriptionRequest::ModifySubscription((client_id, request)) => {
                                    let response = self.subscription_manager.write().await.modify_subscription(request).await?;
                                    if let Some(iceoryx) = &self.iceoryx {
                                        iceoryx.lock().await.send_modify_subscribe_response(client_id, response).await?;
                                    }
                                }
                                IceoryxSubscriptionRequest::CancelSubscription((client_id, request)) => {
                                    let response = self.subscription_manager.write().await.cancel_subscription(client_id.clone(), request).await?;
                                    if let Some(iceoryx) = &self.iceoryx {
                                        iceoryx.lock().await.send_unsubscribe_response(client_id, response).await?;
                                    }
                                }
                            }

                        }
                        Err(e) => return Err::<(), CommunicationError>(CommunicationError::ReceiveError(e.to_string())),
                    }
                }
            } => {
                match iceoryx_subscriptions {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
            }

        }
        Ok(())
    }

    async fn publish_system_info(&self, info: SystemInfo) -> Result<(), CommunicationError> {
        for subscription in self
            .subscription_manager
            .read()
            .await
            .list_subscriptions()
            .await?
            .subscriptions
        {
            let subscription_type = subscription.r#type();
            if subscription_type == SubscriptionType::All
                || subscription_type == SubscriptionType::System
            {
                match subscription.transport_type() {
                    TransportType::Iceoryx => {
                        if let Some(iceoryx) = &self.iceoryx {
                            iceoryx
                                .lock()
                                .await
                                .send_system_info_to_subscriber(info.clone(), &subscription)
                                .await?;
                        } else {
                            return Err(CommunicationError::InvalidSubscription(
                                "No iceoryx transport configured!".to_owned(),
                            ));
                        }
                    }
                    TransportType::Grpc => {
                        if let Some(grpc) = &self.grpc {
                            todo!()
                        } else {
                            return Err(CommunicationError::InvalidSubscription(
                                "No gRPC transport configured!".to_owned(),
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn publish_cpu_info(&self, info: CpuInfo) -> Result<(), CommunicationError> {
        for subscription in self
            .subscription_manager
            .read()
            .await
            .list_subscriptions()
            .await?
            .subscriptions
        {
            let subscription_type = subscription.r#type();
            if subscription_type == SubscriptionType::All
                || subscription_type == SubscriptionType::Cpu
            {
                match subscription.transport_type() {
                    TransportType::Iceoryx => {
                        if let Some(iceoryx) = &self.iceoryx {
                            iceoryx
                                .lock()
                                .await
                                .send_cpu_info_to_subscriber(info.clone(), &subscription)
                                .await?;
                        } else {
                            return Err(CommunicationError::InvalidSubscription(
                                "No iceoryx transport configured!".to_owned(),
                            ));
                        }
                    }
                    TransportType::Grpc => {
                        if let Some(grpc) = &self.grpc {
                            todo!()
                        } else {
                            return Err(CommunicationError::InvalidSubscription(
                                "No gRPC transport configured!".to_owned(),
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn publish_memory_info(&self, info: MemoryInfo) -> Result<(), CommunicationError> {
        for subscription in self
            .subscription_manager
            .read()
            .await
            .list_subscriptions()
            .await?
            .subscriptions
        {
            let subscription_type = subscription.r#type();
            if subscription_type == SubscriptionType::All
                || subscription_type == SubscriptionType::Memory
            {
                match subscription.transport_type() {
                    TransportType::Iceoryx => {
                        if let Some(iceoryx) = &self.iceoryx {
                            iceoryx
                                .lock()
                                .await
                                .send_memory_info_to_subscriber(info.clone(), &subscription)
                                .await?;
                        } else {
                            return Err(CommunicationError::InvalidSubscription(
                                "No iceoryx transport configured!".to_owned(),
                            ));
                        }
                    }
                    TransportType::Grpc => {
                        if let Some(grpc) = &self.grpc {
                            todo!()
                        } else {
                            return Err(CommunicationError::InvalidSubscription(
                                "No gRPC transport configured!".to_owned(),
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn publish_gpu_info(&self, info: &[GpuInfo]) -> Result<(), CommunicationError> {
        for subscription in self
            .subscription_manager
            .read()
            .await
            .list_subscriptions()
            .await?
            .subscriptions
        {
            let subscription_type = subscription.r#type();
            if subscription_type == SubscriptionType::All
                || subscription_type == SubscriptionType::Gpu
            {
                match subscription.transport_type() {
                    TransportType::Iceoryx => {
                        if let Some(iceoryx) = &self.iceoryx {
                            iceoryx
                                .lock()
                                .await
                                .send_gpu_info_to_subscriber(info, &subscription)
                                .await?;
                        } else {
                            return Err(CommunicationError::InvalidSubscription(
                                "No iceoryx transport configured!".to_owned(),
                            ));
                        }
                    }
                    TransportType::Grpc => {
                        if let Some(grpc) = &self.grpc {
                            todo!()
                        } else {
                            return Err(CommunicationError::InvalidSubscription(
                                "No gRPC transport configured!".to_owned(),
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn publish_network_info(&self, info: &[NetworkInfo]) -> Result<(), CommunicationError> {
        for subscription in self
            .subscription_manager
            .read()
            .await
            .list_subscriptions()
            .await?
            .subscriptions
        {
            let subscription_type = subscription.r#type();
            if subscription_type == SubscriptionType::All
                || SubscriptionType::Network == subscription_type
            {
                match subscription.transport_type() {
                    TransportType::Iceoryx => {
                        if let Some(iceoryx) = &self.iceoryx {
                            iceoryx
                                .lock()
                                .await
                                .send_network_info_to_subscriber(info, &subscription)
                                .await?;
                        } else {
                            return Err(CommunicationError::InvalidSubscription(
                                "No iceoryx transport configured!".to_owned(),
                            ));
                        }
                    }
                    TransportType::Grpc => {
                        if let Some(grpc) = &self.grpc {
                            todo!()
                        } else {
                            return Err(CommunicationError::InvalidSubscription(
                                "No gRPC transport configured!".to_owned(),
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn publish_storage_info(&self, info: &[StorageInfo]) -> Result<(), CommunicationError> {
        for subscription in self
            .subscription_manager
            .read()
            .await
            .list_subscriptions()
            .await?
            .subscriptions
        {
            let subscription_type = subscription.r#type();
            if subscription_type == SubscriptionType::All
                || subscription_type == SubscriptionType::Storage
            {
                match subscription.transport_type() {
                    TransportType::Iceoryx => {
                        if let Some(iceoryx) = &self.iceoryx {
                            iceoryx
                                .lock()
                                .await
                                .send_storage_info_to_subscriber(info, &subscription)
                                .await?
                        } else {
                            return Err(CommunicationError::InvalidSubscription(
                                "No iceoryx transport configured!".to_owned(),
                            ));
                        }
                    }
                    TransportType::Grpc => {
                        if let Some(grpc) = &self.grpc {
                            todo!()
                        } else {
                            return Err(CommunicationError::InvalidSubscription(
                                "No gRPC transport configured!".to_owned(),
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn publish_process_info(&self, info: &[ProcessInfo]) -> Result<(), CommunicationError> {
        for subscription in self
            .subscription_manager
            .read()
            .await
            .list_subscriptions()
            .await?
            .subscriptions
        {
            let subscription_type = subscription.r#type();
            if subscription_type == SubscriptionType::All
                || subscription_type == SubscriptionType::Process
            {
                match subscription.transport_type() {
                    TransportType::Iceoryx => {
                        if let Some(iceoryx) = &self.iceoryx {
                            iceoryx
                                .lock()
                                .await
                                .send_process_info_to_subscriber(info, &subscription)
                                .await?;
                        } else {
                            return Err(CommunicationError::InvalidSubscription(
                                "No iceoryx transport configured!".to_owned(),
                            ));
                        }
                    }
                    TransportType::Grpc => {
                        if let Some(grpc) = &self.grpc {
                            todo!()
                        } else {
                            return Err(CommunicationError::InvalidSubscription(
                                "No gRPC transport configured!".to_owned(),
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
