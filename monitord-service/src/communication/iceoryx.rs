use prost::Message;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

use crate::{config::IceoryxConfig, error::CommunicationError};
use iceoryx2::{
    port::{publisher::Publisher as IceoryxPublisher, subscriber::Subscriber as IceoryxSubscriber},
    prelude::*,
};
use monitord_protocols::monitord::{Connection, ConnectionResponse};
use monitord_protocols::subscription::{
    active_subscription::Filter, ActiveSubscription, ModifySubscriptionRequest,
    SubscriptionRequest, SubscriptionResponse, UnsubscribeRequest, UnsubscribeResponse,
};
use monitord_protocols::{
    config::ServiceConfig,
    monitord::{CpuInfo, GpuInfo, MemoryInfo, NetworkInfo, ProcessInfo, StorageInfo, SystemInfo},
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum IceoryxSubscriptionRequest {
    NewSubscription((String, SubscriptionRequest)),
    ModifySubscription((String, ModifySubscriptionRequest)),
    CancelSubscription((String, UnsubscribeRequest)),
}

pub struct IceoryxManager {
    node: Node<ipc::Service>,
    config: IceoryxConfig,

    // Connection buses
    connection_listener: IceoryxSubscriber<ipc::Service, [u8], ()>,
    connection_publisher: IceoryxPublisher<ipc::Service, [u8], ()>,

    // Listen for config updates
    config_listener: IceoryxSubscriber<ipc::Service, [u8], ()>,

    // Publishers (hashmap from topic i.e. monitord/cpu/uuid to publisher)
    publishers: Mutex<HashMap<String, Arc<IceoryxPublisher<ipc::Service, [u8], ()>>>>,
}

impl IceoryxManager {
    pub fn init(config: IceoryxConfig) -> Result<Self, CommunicationError> {
        let node = match NodeBuilder::new().create::<ipc::Service>() {
            Ok(node) => node,
            Err(e) => return Err(CommunicationError::IceoryxError(e.to_string())),
        };

        let config_listener = match node
            .service_builder(
                &format!("{}/config", config.service_name)
                    .as_str()
                    .try_into()
                    .unwrap(),
            )
            .publish_subscribe::<[u8]>()
            .subscriber_max_buffer_size(config.buffer_size)
            .open_or_create()
        {
            Ok(port_factory) => match port_factory
                .subscriber_builder()
                .buffer_size(config.buffer_size)
                .create()
            {
                Ok(config_listener) => config_listener,
                Err(e) => return Err(CommunicationError::InitError(e.to_string())),
            },
            Err(e) => return Err(CommunicationError::InitError(e.to_string())),
        };
        info!("Initialized config listener");

        let connection_listener = match node
            .service_builder(
                &format!("{}/connection/incoming", config.service_name)
                    .as_str()
                    .try_into()
                    .unwrap(),
            )
            .publish_subscribe::<[u8]>()
            .subscriber_max_buffer_size(config.buffer_size)
            .open_or_create()
        {
            Ok(port_factory) => match port_factory
                .subscriber_builder()
                .buffer_size(config.buffer_size)
                .create()
            {
                Ok(connection_listener) => connection_listener,
                Err(e) => return Err(CommunicationError::InitError(e.to_string())),
            },
            Err(e) => return Err(CommunicationError::InitError(e.to_string())),
        };
        info!("Initialized connection listener");

        let connection_publisher = match node
            .service_builder(
                &format!("{}/connection/outgoing", config.service_name)
                    .as_str()
                    .try_into()
                    .unwrap(),
            )
            .publish_subscribe::<[u8]>()
            .open_or_create()
        {
            Ok(port_factory) => match port_factory
                .publisher_builder()
                .initial_max_slice_len(config.buffer_size)
                .create()
            {
                Ok(connection_publisher) => connection_publisher,
                Err(e) => return Err(CommunicationError::InitError(e.to_string())),
            },
            Err(e) => return Err(CommunicationError::InitError(e.to_string())),
        };

        Ok(Self {
            node,
            config,
            connection_listener,
            connection_publisher,
            config_listener,
            publishers: Mutex::new(HashMap::new()),
        })
    }

    pub async fn respond_to_connections(&self) -> Result<(), CommunicationError> {
        match self.connection_listener.receive() {
            Ok(sample) => match sample {
                Some(sample) => {
                    let connection = Connection::decode(sample.payload()).unwrap();
                    let client_id = Uuid::new_v4().to_string();
                    let response = ConnectionResponse {
                        pid: connection.pid,
                        client_id,
                    };
                    let buf = response.encode_to_vec();
                    let publisher = &self.connection_publisher;
                    match publisher.loan_slice_uninit(buf.len()) {
                        Ok(sample) => match sample.write_from_slice(buf.as_slice()).send() {
                            Ok(count) => {
                                debug!("Sent connection response to {count} clients");
                                Ok(())
                            }
                            Err(e) => return Err(CommunicationError::SendError(e.to_string())),
                        },
                        Err(e) => return Err(CommunicationError::IceoryxError(e.to_string())),
                    }
                }
                None => Ok(()),
            },
            Err(e) => return Err(CommunicationError::ReceiveError(e.to_string())),
        }
    }

    pub async fn receive_config(&self) -> Result<Option<ServiceConfig>, CommunicationError> {
        match self.config_listener.receive() {
            Ok(sample) => match sample {
                Some(sample) => match ServiceConfig::decode(sample.payload()) {
                    Ok(config) => Ok(Some(config)),
                    Err(e) => return Err(CommunicationError::DeserializationError(e.to_string())),
                },
                None => Ok(None),
            },
            Err(e) => Err(CommunicationError::ReceiveError(e.to_string())),
        }
    }

    async fn get_or_create_publisher(
        &mut self,
        topic: &str,
        size: usize,
    ) -> Result<Arc<IceoryxPublisher<ipc::Service, [u8], ()>>, CommunicationError> {
        let mut publishers = self.publishers.lock().await;
        match publishers.get(&topic.to_string()) {
            Some(publisher) => Ok(Arc::clone(publisher)),
            None => Ok(
                match self
                    .node
                    .service_builder(&topic.try_into().unwrap())
                    .publish_subscribe::<[u8]>()
                    .open_or_create()
                {
                    Ok(port_factory) => match port_factory
                        .publisher_builder()
                        .initial_max_slice_len(size)
                        .create()
                    {
                        Ok(publisher) => {
                            publishers.insert(topic.to_string(), Arc::new(publisher));
                            Arc::clone(publishers.get(&topic.to_string()).unwrap())
                        }
                        Err(e) => return Err(CommunicationError::InitError(e.to_string())),
                    },
                    Err(e) => return Err(CommunicationError::InitError(e.to_string())),
                },
            ),
        }
    }

    async fn send_to_subscriber(
        &mut self,
        info: &[u8],
        topic: &str,
    ) -> Result<(), CommunicationError> {
        let publisher = self.get_or_create_publisher(topic, info.len()).await?;

        match publisher.loan_slice_uninit(info.len()) {
            Ok(sample) => match sample.write_from_slice(info).send() {
                Ok(count) => debug!("Sent info snapshot to {count} clients"),
                Err(e) => return Err(CommunicationError::SendError(e.to_string())),
            },
            Err(e) => return Err(CommunicationError::IceoryxError(e.to_string())),
        }
        Ok(())
    }

    pub async fn send_system_info_to_subscriber(
        &mut self,
        info: SystemInfo,
        subscription: &ActiveSubscription,
    ) -> Result<(), CommunicationError> {
        let buf = info.encode_to_vec();
        let topic = format!(
            "{}/system/{}",
            self.config.service_name, subscription.subscription_id
        );

        self.send_to_subscriber(&buf, &topic).await
    }

    pub async fn send_cpu_info_to_subscriber(
        &mut self,
        info: CpuInfo,
        subscription: &ActiveSubscription,
    ) -> Result<(), CommunicationError> {
        let buf = info.encode_to_vec();
        let topic = format!(
            "{}/cpu/{}",
            self.config.service_name, subscription.subscription_id
        );
        self.send_to_subscriber(&buf, &topic).await
    }

    pub async fn send_memory_info_to_subscriber(
        &mut self,
        info: MemoryInfo,
        subscription: &ActiveSubscription,
    ) -> Result<(), CommunicationError> {
        let buf = info.encode_to_vec();
        let topic = format!(
            "{}/memory/{}",
            self.config.service_name, subscription.subscription_id
        );
        self.send_to_subscriber(&buf, &topic).await
    }

    pub async fn send_gpu_info_to_subscriber(
        &mut self,
        info: &[GpuInfo],
        subscription: &ActiveSubscription,
    ) -> Result<(), CommunicationError> {
        let topic = format!(
            "{}/gpu/{}",
            self.config.service_name, subscription.subscription_id
        );
        for gpu in info.iter() {
            if let Some(Filter::GpuFilter(filter)) = &subscription.filter {
                if filter.name.contains(&gpu.name) && filter.vendor.contains(&gpu.vendor) {
                    let buf = if filter.include_processes {
                        gpu.encode_to_vec()
                    } else {
                        let mut gpu = gpu.clone();
                        gpu.process_info = prost::alloc::vec![];
                        gpu.encode_to_vec()
                    };
                    self.send_to_subscriber(&buf, &topic).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn send_network_info_to_subscriber(
        &mut self,
        info: &[NetworkInfo],
        subscription: &ActiveSubscription,
    ) -> Result<(), CommunicationError> {
        let topic = format!(
            "{}/network/{}",
            self.config.service_name, subscription.subscription_id
        );
        for net in info.iter() {
            if let Some(Filter::NetworkFilter(filter)) = &subscription.filter {
                if filter.interface_name.contains(&net.interface_name) {
                    let buf = net.encode_to_vec();
                    self.send_to_subscriber(&buf, &topic).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn send_storage_info_to_subscriber(
        &mut self,
        info: &[StorageInfo],
        subscription: &ActiveSubscription,
    ) -> Result<(), CommunicationError> {
        let topic = format!(
            "{}/storage/{}",
            self.config.service_name, subscription.subscription_id
        );
        for storage in info.iter() {
            if let Some(Filter::StorageFilter(filter)) = &subscription.filter {
                if filter.device_name.contains(&storage.device_name)
                    && filter.mount_point.contains(&storage.mount_point)
                {
                    let buf = storage.encode_to_vec();
                    self.send_to_subscriber(&buf, &topic).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn send_process_info_to_subscriber(
        &mut self,
        info: &[ProcessInfo],
        subscription: &ActiveSubscription,
    ) -> Result<(), CommunicationError> {
        let topic = format!(
            "{}/process/{}",
            self.config.service_name, subscription.subscription_id
        );
        for process in info.iter() {
            if let Some(Filter::ProcessFilter(filter)) = &subscription.filter {
                if filter.pid.contains(&process.pid)
                    && filter.name.contains(&process.name)
                    && filter.username.contains(&process.username)
                    && process.cpu_usage_percent as u32 >= filter.top_by_cpu
                    && process.physical_memory_bytes as u32 >= filter.top_by_memory
                    && process.virtual_memory_bytes as u32 >= filter.top_by_memory
                    && process.disk_read_bytes_per_sec as u32
                        + process.disk_write_bytes_per_sec as u32
                        >= filter.top_by_disk
                {
                    let buf = process.encode_to_vec();
                    self.send_to_subscriber(&buf, &topic).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn send_subscribe_response(
        &mut self,
        client_id: String,
        response: SubscriptionResponse,
    ) -> Result<(), CommunicationError> {
        let buf = response.encode_to_vec();
        let topic = format!("{}/subscribe/{}", self.config.service_name, client_id);
        self.send_to_subscriber(&buf, &topic).await
    }

    pub async fn send_modify_subscribe_response(
        &mut self,
        client_id: String,
        response: SubscriptionResponse,
    ) -> Result<(), CommunicationError> {
        let buf = response.encode_to_vec();
        let topic = format!("{}/modify/{}", self.config.service_name, client_id);
        self.send_to_subscriber(&buf, &topic).await
    }

    pub async fn send_unsubscribe_response(
        &mut self,
        client_id: String,
        response: UnsubscribeResponse,
    ) -> Result<(), CommunicationError> {
        let buf = response.encode_to_vec();
        let topic = format!("{}/unsubscribe/{}", self.config.service_name, client_id);
        self.send_to_subscriber(&buf, &topic).await
    }
}
