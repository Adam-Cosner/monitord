use prost::{bytes, Message};
use std::collections::HashMap;

use iceoryx2::{
    port::{publisher::Publisher, subscriber::Subscriber},
    prelude::*,
};
use monitord_protocols::{
    config::ServiceConfig,
    monitord::{CpuInfo, GpuInfo, MemoryInfo, NetworkInfo, ProcessInfo, StorageInfo, SystemInfo},
};

use super::{config::IceoryxConfig, error::CommunicationError};

pub struct IceoryxManager {
    node: Node<ipc::Service>,
    // Subscriptions are unnecessary as clients need only hook into the iceoryx topic

    // Listen for config updates
    config_listener: Subscriber<ipc::Service, ServiceConfig, ()>,

    // Publishers
    system_info_publisher: Publisher<ipc::Service, [u8], ()>,
    cpu_info_publisher: Publisher<ipc::Service, [u8], ()>,
    memory_info_publisher: Publisher<ipc::Service, [u8], ()>,
    gpu_info_publisher: Publisher<ipc::Service, [u8], ()>,
    net_info_publisher: Publisher<ipc::Service, [u8], ()>,
    storage_info_publisher: Publisher<ipc::Service, [u8], ()>,
    process_info_publisher: Publisher<ipc::Service, [u8], ()>,
}

impl IceoryxManager {
    pub fn init(config: IceoryxConfig) -> Result<Self, CommunicationError> {
        todo!()
    }

    pub async fn send_system_info(&self, info: SystemInfo) -> Result<(), CommunicationError> {
        todo!()
    }

    pub async fn send_cpu_info(&self, info: CpuInfo) -> Result<(), CommunicationError> {
        todo!()
    }

    pub async fn send_memory_info(&self, info: MemoryInfo) -> Result<(), CommunicationError> {
        todo!()
    }

    pub async fn send_gpu_info(&self, info: Vec<GpuInfo>) -> Result<(), CommunicationError> {
        todo!()
    }

    pub async fn send_net_info(&self, info: Vec<NetworkInfo>) -> Result<(), CommunicationError> {
        todo!()
    }

    pub async fn send_storage_info(
        &self,
        info: Vec<StorageInfo>,
    ) -> Result<(), CommunicationError> {
        todo!()
    }

    pub async fn send_process_info(
        &self,
        info: HashMap<u32, ProcessInfo>,
    ) -> Result<(), CommunicationError> {
        todo!()
    }
}
