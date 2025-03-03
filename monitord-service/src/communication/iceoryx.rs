use prost::Message;
use tracing::debug;

use iceoryx2::{
    port::{publisher::Publisher, subscriber::Subscriber},
    prelude::*,
};
use monitord_protocols::{
    config::ServiceConfig,
    monitord::{CpuInfo, GpuInfo, MemoryInfo, NetworkInfo, ProcessInfo, StorageInfo, SystemInfo},
};

use crate::{config::IceoryxConfig, error::CommunicationError};

pub struct IceoryxManager {
    node: Node<ipc::Service>,
    config: IceoryxConfig,
    // Subscriptions are unnecessary as clients need only hook into the iceoryx topic

    // Listen for config updates
    config_listener: Subscriber<ipc::Service, [u8], ()>,

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

        let system_info_publisher = match node
            .service_builder(
                &format!("{}/system", config.service_name)
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
                Ok(publisher) => publisher,
                Err(e) => return Err(CommunicationError::InitError(e.to_string())),
            },
            Err(e) => return Err(CommunicationError::InitError(e.to_string())),
        };

        let cpu_info_publisher = match node
            .service_builder(
                &format!("{}/cpu", config.service_name)
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
                Ok(publisher) => publisher,
                Err(e) => return Err(CommunicationError::InitError(e.to_string())),
            },
            Err(e) => return Err(CommunicationError::InitError(e.to_string())),
        };

        let memory_info_publisher = match node
            .service_builder(
                &format!("{}/memory", config.service_name)
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
                Ok(publisher) => publisher,
                Err(e) => return Err(CommunicationError::InitError(e.to_string())),
            },
            Err(e) => return Err(CommunicationError::InitError(e.to_string())),
        };

        let gpu_info_publisher = match node
            .service_builder(
                &format!("{}/gpu", config.service_name)
                    .as_str()
                    .try_into()
                    .unwrap(),
            )
            .publish_subscribe::<[u8]>()
            .open_or_create()
        {
            Ok(port_factory) => match port_factory
                .publisher_builder()
                .max_loaned_samples(16)
                .initial_max_slice_len(config.buffer_size)
                .create()
            {
                Ok(publisher) => publisher,
                Err(e) => return Err(CommunicationError::InitError(e.to_string())),
            },
            Err(e) => return Err(CommunicationError::InitError(e.to_string())),
        };

        let net_info_publisher = match node
            .service_builder(
                &format!("{}/net", config.service_name)
                    .as_str()
                    .try_into()
                    .unwrap(),
            )
            .publish_subscribe::<[u8]>()
            .open_or_create()
        {
            Ok(port_factory) => match port_factory
                .publisher_builder()
                .max_loaned_samples(16)
                .initial_max_slice_len(config.buffer_size)
                .create()
            {
                Ok(publisher) => publisher,
                Err(e) => return Err(CommunicationError::InitError(e.to_string())),
            },
            Err(e) => return Err(CommunicationError::InitError(e.to_string())),
        };

        let storage_info_publisher = match node
            .service_builder(
                &format!("{}/storage", config.service_name)
                    .as_str()
                    .try_into()
                    .unwrap(),
            )
            .publish_subscribe::<[u8]>()
            .open_or_create()
        {
            Ok(port_factory) => match port_factory
                .publisher_builder()
                .max_loaned_samples(32)
                .initial_max_slice_len(config.buffer_size)
                .create()
            {
                Ok(publisher) => publisher,
                Err(e) => return Err(CommunicationError::InitError(e.to_string())),
            },
            Err(e) => return Err(CommunicationError::InitError(e.to_string())),
        };

        let process_info_publisher = match node
            .service_builder(
                &format!("{}/process", config.service_name)
                    .as_str()
                    .try_into()
                    .unwrap(),
            )
            .publish_subscribe::<[u8]>()
            .open_or_create()
        {
            Ok(port_factory) => match port_factory
                .publisher_builder()
                .max_loaned_samples(32768)
                .initial_max_slice_len(config.buffer_size)
                .create()
            {
                Ok(publisher) => publisher,
                Err(e) => return Err(CommunicationError::InitError(e.to_string())),
            },
            Err(e) => return Err(CommunicationError::InitError(e.to_string())),
        };

        Ok(Self {
            node,
            config,
            config_listener,
            system_info_publisher,
            cpu_info_publisher,
            memory_info_publisher,
            gpu_info_publisher,
            net_info_publisher,
            storage_info_publisher,
            process_info_publisher,
        })
    }

    pub fn receive_config(&self) -> Result<Option<ServiceConfig>, CommunicationError> {
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

    pub fn send_system_info(&self, info: SystemInfo) -> Result<(), CommunicationError> {
        let buf = info.encode_to_vec();

        match self
            .system_info_publisher
            .loan_slice_uninit(info.encoded_len())
        {
            Ok(sample) => match sample.write_from_slice(buf.as_slice()).send() {
                Ok(count) => debug!("Sent system info snapshot to {count} clients"),
                Err(e) => return Err(CommunicationError::SendError(e.to_string())),
            },
            Err(e) => return Err(CommunicationError::IceoryxError(e.to_string())),
        }
        Ok(())
    }

    pub fn send_cpu_info(&self, info: CpuInfo) -> Result<(), CommunicationError> {
        let buf = info.encode_to_vec();

        match self
            .cpu_info_publisher
            .loan_slice_uninit(info.encoded_len())
        {
            Ok(sample) => match sample.write_from_slice(buf.as_slice()).send() {
                Ok(count) => debug!("Sent CPU info snapshot to {count} clients"),
                Err(e) => return Err(CommunicationError::SendError(e.to_string())),
            },
            Err(e) => return Err(CommunicationError::IceoryxError(e.to_string())),
        }
        Ok(())
    }

    pub fn send_memory_info(&self, info: MemoryInfo) -> Result<(), CommunicationError> {
        let buf = info.encode_to_vec();

        match self
            .memory_info_publisher
            .loan_slice_uninit(info.encoded_len())
        {
            Ok(sample) => match sample.write_from_slice(buf.as_slice()).send() {
                Ok(count) => debug!("Sent memory info snapshot to {count} clients"),
                Err(e) => return Err(CommunicationError::SendError(e.to_string())),
            },
            Err(e) => return Err(CommunicationError::IceoryxError(e.to_string())),
        }
        Ok(())
    }

    pub fn send_gpu_info(&self, info: &[GpuInfo]) -> Result<(), CommunicationError> {
        for gpu in info.iter() {
            let buf = gpu.encode_to_vec();
            match self.gpu_info_publisher.loan_slice_uninit(gpu.encoded_len()) {
                Ok(sample) => match sample.write_from_slice(buf.as_slice()).send() {
                    Ok(count) => debug!("Sent GPU info snapshot to {count} clients"),
                    Err(e) => return Err(CommunicationError::SendError(e.to_string())),
                },
                Err(e) => return Err(CommunicationError::IceoryxError(e.to_string())),
            }
        }
        Ok(())
    }

    pub fn send_net_info(&self, info: &[NetworkInfo]) -> Result<(), CommunicationError> {
        for net in info.iter() {
            let buf = net.encode_to_vec();
            match self.net_info_publisher.loan_slice_uninit(net.encoded_len()) {
                Ok(sample) => match sample.write_from_slice(buf.as_slice()).send() {
                    Ok(count) => debug!("Sent Network info snapshot to {count} clients"),
                    Err(e) => return Err(CommunicationError::SendError(e.to_string())),
                },
                Err(e) => return Err(CommunicationError::IceoryxError(e.to_string())),
            }
        }
        Ok(())
    }

    pub fn send_storage_info(&self, info: &[StorageInfo]) -> Result<(), CommunicationError> {
        for storage in info.iter() {
            let buf = storage.encode_to_vec();
            match self
                .storage_info_publisher
                .loan_slice_uninit(storage.encoded_len())
            {
                Ok(sample) => match sample.write_from_slice(buf.as_slice()).send() {
                    Ok(count) => debug!("Sent storage info snapshot to {count} clients"),
                    Err(e) => return Err(CommunicationError::SendError(e.to_string())),
                },
                Err(e) => return Err(CommunicationError::IceoryxError(e.to_string())),
            }
        }
        Ok(())
    }

    pub fn send_process_info(&self, info: &[ProcessInfo]) -> Result<(), CommunicationError> {
        for process in info.iter() {
            let buf = process.encode_to_vec();
            match self
                .process_info_publisher
                .loan_slice_uninit(process.encoded_len())
            {
                Ok(sample) => match sample.write_from_slice(buf.as_slice()).send() {
                    Ok(_) => return Ok(()),
                    Err(e) => return Err(CommunicationError::SendError(e.to_string())),
                },
                Err(e) => return Err(CommunicationError::IceoryxError(e.to_string())),
            }
        }
        Ok(())
    }
}
