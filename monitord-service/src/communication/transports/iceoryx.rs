use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::{mpsc, oneshot};
use std::thread::JoinHandle;
use tokio::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

use iceoryx2::port::{publisher::*, subscriber::*};
use iceoryx2::prelude::*;

use crate::communication::core::models::TransportType;
use crate::communication::core::traits::Transport;
use crate::communication::core::ClientConnection;
use crate::communication::error::CommunicationError;
use crate::communication::transports::common::TopicFormatter;
use crate::config::IceoryxConfig;

// Commands sent to the iceoryx worker thread
enum IceoryxCommand {
    Publish {
        topic: String,
        payload: Vec<u8>,
        response_tx: oneshot::Sender<Result<(), String>>,
    },
    CheckConnections {
        response_tx: oneshot::Sender<Option<ClientConnection>>,
    },
    SendResponse {
        client_id: String,
        payload: Vec<u8>,
        response_tx: oneshot::Sender<Result<(), String>>,
    },
    Shutdown,
}

/// Implementation of the Transport trait for iceoryx2
pub struct IceoryxTransport {
    config: IceoryxConfig,
    topic_formatter: TopicFormatter,
    active: bool,
    command_tx: std::sync::mpsc::Sender<IceoryxCommand>,
    worker_handle: Option<JoinHandle<()>>,
}

impl IceoryxTransport {
    pub fn new(config: IceoryxConfig) -> Result<Self, CommunicationError> {
        let topic_formatter = TopicFormatter::new(&config.service_name);

        // Create a channel for sending commands to the worker thread
        let (command_tx, command_rx) = std::sync::mpsc::channel();

        // Clone config for the worker thread
        let worker_config = config.clone();
        let worker_topic_formatter = topic_formatter.clone();

        // Spawn a worker thread that will handle the actual iceoryx operations
        let worker_handle = std::thread::spawn(move || { Self::run_worker(worker_config, worker_topic_formatter, command_rx )} );

        Ok(Self {
            config,
            topic_formatter,
            active: false,
            command_tx,
            worker_handle: Some(worker_handle),
        })
    }

    // The worker function that runs in a separate thread
    fn run_worker(
        config: IceoryxConfig,
        topic_formatter: TopicFormatter,
        mut command_rx: std::sync::mpsc::Receiver<IceoryxCommand>,
    ) {
        use iceoryx2::prelude::*;

        // Initialize iceoryx in this thread
        let node = match NodeBuilder::new()
            .name(&config.service_name.as_str().try_into().unwrap())
            .create()
        {
            Ok(node) => node,
            Err(e) => {
                error!("Failed to create iceoryx node: {}", e);
                return;
            }
        };

        let mut publishers: HashMap<String, Publisher<ipc::Service, [u8], ()>> = HashMap::new();
        let mut client_connections: HashMap<String, (Instant, Publisher<ipc::Service, [u8], ()>)> =
            HashMap::new();

        // Set up connection subscriber
        let connection_topic = topic_formatter.format_connection_topic("requests");
        let connection_subscriber = match node
            .service_builder(&connection_topic.as_str().try_into().unwrap())
            .publish_subscribe::<[u8]>()
            .open_or_create()
        {
            Ok(port_factory) => match port_factory.subscriber_builder().create() {
                Ok(subscriber) => subscriber,
                Err(e) => {
                    error!("Failed to create connection subscriber: {}", e);
                    return;
                }
            },
            Err(e) => {
                error!("Failed to create connection subscriber: {}", e);
                return;
            }
        };

        info!("Iceoryx worker thread started");

        // Process commands
        while let Some(cmd) = command_rx.recv().ok() {
            match cmd {
                IceoryxCommand::Publish {
                    topic,
                    payload,
                    response_tx,
                } => {
                    // Get or create publisher
                    let publisher = if let Some(publisher) = publishers.get(&topic) {
                        publisher
                    } else {
                        match node
                            .service_builder(&topic.as_str().try_into().unwrap())
                            .publish_subscribe::<[u8]>()
                            .history_size(16)
                            .subscriber_max_buffer_size(config.buffer_size)
                            .open_or_create()
                        {
                            Ok(port_factory) => match port_factory.publisher_builder().create() {
                                Ok(pub_handle) => {
                                    publishers.insert(topic.clone(), pub_handle);
                                    publishers.get(&topic).unwrap()
                                }
                                Err(e) => {
                                    let _ = response_tx
                                        .send(Err(format!("Failed to create publisher: {}", e)));
                                    continue;
                                }
                            },
                            Err(e) => {
                                let _ = response_tx.send(Err(format!(
                                    "Failed to create publisher factory: {}",
                                    e
                                )));
                                continue;
                            }
                        }
                    };

                    let result = match publisher.loan_slice_uninit(payload.len()) {
                        Ok(sample) => match sample.write_from_slice(payload.as_slice()).send() {
                            Ok(_) => {
                                debug!("Published data to topic: {}", topic);
                                Ok(())
                            }
                            Err(e) => Err(format!("Failed to publish to {}: {}", topic, e)),
                        },
                        Err(e) => Err(format!(
                            "Failed to loan data slice for publishing to topic {}: {}",
                            topic, e
                        )),
                    };

                    let _ = response_tx.send(result);
                }

                IceoryxCommand::CheckConnections { response_tx } => {
                    // Check for new connection requests
                    if let Some(sample) = connection_subscriber.receive().ok().flatten() {
                        let request_data = sample.payload();

                        // Process connection request (same logic as before)
                        if request_data.len() < 8 {
                            let _ = response_tx.send(None);
                            continue;
                        }

                        let pid = u32::from_le_bytes([
                            request_data[0],
                            request_data[1],
                            request_data[2],
                            request_data[3],
                        ]);

                        let mut client_id_end = 4;
                        while client_id_end < request_data.len() && request_data[client_id_end] != 0
                        {
                            client_id_end += 1;
                        }

                        let client_id =
                            match String::from_utf8(request_data[4..client_id_end].to_vec()) {
                                Ok(id) => id,
                                Err(_) => {
                                    let _ = response_tx.send(None);
                                    continue;
                                }
                            };

                        // Create a response publisher for this client
                        let response_topic =
                            topic_formatter.format_response_topic("connection", &client_id);
                        let response_publisher = match node
                            .service_builder(&response_topic.as_str().try_into().unwrap())
                            .publish_subscribe::<[u8]>()
                            .history_size(16)
                            .subscriber_max_buffer_size(config.buffer_size)
                            .open_or_create()
                        {
                            Ok(port_factory) => match port_factory.publisher_builder().create() {
                                Ok(publisher) => publisher,
                                Err(e) => {
                                    error!("Failed to create response publisher: {}", e);
                                    let _ = response_tx.send(None);
                                    continue;
                                }
                            },
                            Err(e) => {
                                error!("Failed to create response publisher factory: {}", e);
                                let _ = response_tx.send(None);
                                continue;
                            }
                        };

                        // Store the client connection
                        client_connections
                            .insert(client_id.clone(), (Instant::now(), response_publisher));

                        // Return the new connection
                        let _ = response_tx.send(Some(ClientConnection {
                            client_id,
                            pid,
                            connected_at: Instant::now().into_std(),
                            transport_type: TransportType::Iceoryx,
                        }));
                    } else {
                        // No new connections
                        let _ = response_tx.send(None);
                    }
                }

                IceoryxCommand::SendResponse {
                    client_id,
                    payload,
                    response_tx,
                } => {
                    // Find the publisher for this client
                    let result = if let Some((_, publisher)) = client_connections.get(&client_id) {
                        // Send the response
                        match publisher.loan_slice_uninit(payload.len()) {
                            Ok(sample) => {
                                match sample.write_from_slice(payload.as_slice()).send() {
                                    Ok(_) => {
                                        debug!("Sent response to client: {}", client_id);
                                        Ok(())
                                    }
                                    Err(e) => Err(format!("Failed to send response to client {}: {}", client_id, e))
                                }
                            }
                            Err(e) => Err(format!("Failed to loan data slice for response to client {}: {}", client_id, e))
                        }
                    } else {
                        Err(format!("Client not found: {}", client_id))
                    };

                    let _ = response_tx.send(result);
                }

                IceoryxCommand::Shutdown => {
                    info!("Shutting down iceoryx worker thread");
                    break;
                }
            }
        }

        // Clean up resources
        publishers.clear();
        client_connections.clear();

        info!("Iceoryx worker thread stopped");
    }
}

#[async_trait]
impl Transport for IceoryxTransport {
    async fn initialize(&mut self) -> Result<(), CommunicationError> {
        info!("Initializing iceoryx2 transport");
        self.active = true;
        info!("Iceoryx2 transport initialized successfully");
        Ok(())
    }

    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<(), CommunicationError> {
        if !self.active {
            return Err(CommunicationError::Transport(
                "iceoryx2 transport is not active".into(),
            ));
        }

        // Create a oneshot channel for the response
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();

        // Send command to worker thread
        if let Err(e) = self
            .command_tx
            .send(IceoryxCommand::Publish {
                topic: topic.to_string(),
                payload: payload.to_vec(),
                response_tx,
            })
        {
            return Err(CommunicationError::Transport(format!(
                "Failed to send publish command: {}",
                e
            )));
        }

        // Wait for response
        match response_rx.await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(CommunicationError::Transport(e)),
            Err(e) => Err(CommunicationError::Transport(format!(
                "Failed to receive publish response: {}",
                e
            ))),
        }
    }

    async fn listen_for_connections(&self) -> Result<Option<ClientConnection>, CommunicationError> {
        if !self.active {
            return Err(CommunicationError::Transport(
                "iceoryx2 transport is not active".into(),
            ));
        }

        // Create a oneshot channel for the response
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();

        // Send command to worker thread
        if let Err(e) = self
            .command_tx
            .send(IceoryxCommand::CheckConnections { response_tx })
        {
            return Err(CommunicationError::Transport(format!(
                "Failed to send check connections command: {}",
                e
            )));
        }

        // Wait for response
        match response_rx.await {
            Ok(connection) => Ok(connection),
            Err(e) => Err(CommunicationError::Transport(format!(
                "Failed to receive connection check response: {}",
                e
            ))),
        }
    }

    async fn send_response(
        &self,
        client_id: &str,
        response: &[u8],
    ) -> Result<(), CommunicationError> {
        if !self.active {
            return Err(CommunicationError::Transport(
                "iceoryx2 transport is not active".into(),
            ));
        }

        // Create a oneshot channel for the response
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();

        // Send command to worker thread
        if let Err(e) = self
            .command_tx
            .send(IceoryxCommand::SendResponse {
                client_id: client_id.to_string(),
                payload: response.to_vec(),
                response_tx,
            })
        {
            return Err(CommunicationError::Transport(format!(
                "Failed to send response command: {}",
                e
            )));
        }

        // Wait for worker response
        match response_rx.await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(CommunicationError::Transport(e)),
            Err(e) => Err(CommunicationError::Transport(format!(
                "Failed to receive send response result: {}",
                e
            ))),
        }
    }

    fn name(&self) -> &str {
        "iceoryx"
    }

    fn is_active(&self) -> bool {
        self.active
    }
}

impl Drop for IceoryxTransport {
    fn drop(&mut self) {
        info!("Shutting down iceoryx2 transport");

        if let Some(handle) = self.worker_handle.take() {
            // Send shutdown command
            if let Err(e) = self.command_tx.send(IceoryxCommand::Shutdown) {
                error!("Failed to send shutdown command: {}", e);
            }

            handle.join().unwrap();
        }

        self.active = false;
    }
}
