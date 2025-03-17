use prost::Message;
use crate::config::{TransportConfig, TransportType};
use crate::core::traits::Transport;
use crate::error::TransportError;
use crate::transports::{IceoryxTransport, TransportVariant};
use std::sync::{Arc};
use futures::lock::{Mutex, MutexGuard};

pub mod core;
pub mod error;
pub mod transports;
pub mod config;

pub struct TransportManager {
    variant: Arc<Mutex<TransportVariant>>,
    config: TransportConfig,
}

impl TransportManager {
    pub fn new(config: TransportConfig) -> Result<Self, TransportError> {
        let variant = match &config.transport_config {
            TransportType::Iceoryx(config) => TransportVariant::Iceoryx(IceoryxTransport::new(config.clone())?),
            TransportType::Grpc => TransportVariant::Grpc(),
            TransportType::Intra => TransportVariant::Intra(),
        };


        Ok(Self {
            variant: Arc::new(Mutex::new(variant)),
            config
        })
    }

    pub async fn initialize(&mut self) -> Result<(), TransportError> {
        let mut variant = self.variant.lock().await;
        match &mut *variant {
            TransportVariant::Iceoryx(variant) => variant.initialize().await,
            TransportVariant::Grpc() => Err(TransportError::Initialize("gRPC unavailable".to_owned())),
            TransportVariant::Intra() => Err(TransportError::Initialize("Intra unavailable".to_owned())),

        }
    }

    pub async fn publish<T: Message + Default>(&mut self, destination: &str, message: T) -> Result<(), TransportError> {
        let message = message.encode_to_vec();
        let variant = self.variant.lock().await;

        match &*variant {
            TransportVariant::Iceoryx(transport) => transport.publish(destination, message.as_slice()).await,
            TransportVariant::Grpc() => Err(TransportError::Publish("gRPC unavailable".to_owned())),
            TransportVariant::Intra() => Err(TransportError::Publish("Intra unavailable".to_owned())),
        }
    }

    pub async fn receive<T: Message + Default>(&mut self, destination: &str) -> Result<Option<T>, TransportError> {
        let variant = self.variant.lock().await;

        let message = match &*variant {
            TransportVariant::Iceoryx(transport) => transport.receive(destination).await?,
            TransportVariant::Grpc() => return Err(TransportError::Receive("gRPC unavailable".to_owned())),
            TransportVariant::Intra() => return Err(TransportError::Receive("Intra unavailable".to_owned())),
        };
        if let Some(message) = message {
            Ok(Some(T::decode(message.as_slice()).map_err(|e| TransportError::Serialize(e.to_string()))?))
        } else {
            Ok(None)
        }
    }
}

impl Clone for TransportManager {
    fn clone(&self) -> Self {
        Self {
            variant: self.variant.clone(),
            config: self.config.clone(),
        }
    }
}