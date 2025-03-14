use async_trait::async_trait;
use crate::communication::core::ClientConnection;
use crate::communication::Transport;
use crate::config::IceoryxConfig;
use crate::error::{CommunicationError};
use iceoryx2::prelude::*;
use crate::communication::transports::common::TopicFormatter;

/// Implementation of the Transport trait for iceoryx2
pub struct IceoryxTransport {
    config: IceoryxConfig,
    topic_formatter: TopicFormatter,
    active: bool,
    node: Node<ipc::Service>,
}

impl IceoryxTransport {
    pub fn new(config: IceoryxConfig) -> Result<Self, CommunicationError> {
        let topic_formatter = TopicFormatter::new(&config.service_name);
        let node = NodeBuilder::default().create().map_err(|e| CommunicationError::Transport(e.to_string()))?;
        Ok(Self { topic_formatter, config, active: false, node })
    }
}

#[async_trait]
impl Transport for IceoryxTransport {
    async fn initialize(&mut self) -> Result<(), CommunicationError> {
        // TODO: Start iceoryx2 and set up channels


        self.active = true;
        Ok(())
    }

    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<(), CommunicationError> {
        if !self.active {
            return Err(CommunicationError::Transport("iceoryx2 transport is not active".into()))
        }

        // TODO: Implement iceoryx2 publishing logic

        Ok(())
    }

    async fn listen_for_connections(&self) -> Result<Option<ClientConnection>, CommunicationError> {
        if !self.active {
            return Err(CommunicationError::Transport("iceoryx2 transport is not active".into()));
        }

        // TODO: Implement connection handling for iceoryx2

        Ok(None)
    }

    async fn send_response(&self, client_id: &str, response: &[u8]) -> Result<(), CommunicationError> {
        if !self.active {
            return Err(CommunicationError::Transport("iceoryx2 transport is not active".into()))
        }

        // TODO: Implement response sending for iceoryx2

        Ok(())
    }

    fn name(&self) -> &str {
        "iceoryx2"
    }

    fn is_active(&self) -> bool {
        self.active
    }
}