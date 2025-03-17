use std::collections::HashMap;
use std::thread::JoinHandle;
use std::sync::{mpsc};
use futures::channel::oneshot;
use tracing::{debug, error, info};
use crate::config::IceoryxConfig;
use crate::core::traits::Transport;
use crate::error::TransportError;


/// Commands to send to the worker thread
enum IceoryxCommand {
    Publish {
        topic: String,
        payload: Vec<u8>,
        response_tx: oneshot::Sender<Result<(), TransportError>>,
    },
    Receive {
        topic: String,
        response_tx: oneshot::Sender<Result<Option<Vec<u8>>, TransportError>>,
    },
}
pub struct IceoryxTransport {
    active: bool,
    command_tx: mpsc::Sender<IceoryxCommand>,
    worker_handle: Option<JoinHandle<()>>,
}

impl IceoryxTransport {
    pub fn new(config: IceoryxConfig) -> Result<Self, TransportError> {
        let (command_tx, command_rx) = mpsc::channel::<IceoryxCommand>();

        let worker_handle = std::thread::spawn(move || Self::run_worker(config, command_rx));
        Ok(Self {
            active: false,
            command_tx,
            worker_handle: Some(worker_handle),
        })
    }

    /// The worker function that runs in its own single thread
    fn run_worker(config: IceoryxConfig, command_rx: mpsc::Receiver<IceoryxCommand>) {
        use iceoryx2::prelude::*;

        // Initialize iceoryx2 in this thread
        let node = match NodeBuilder::new()
            .name(&config.service_name.as_str().try_into().unwrap())
            .create::<ipc::Service>() {
            Ok(node) => node,
            Err(e) => {
                error!("Failed to create iceoryx2 node: {e}");
                return;
            }
        };
        info!("Iceoryx worker thread started");

        let mut publishers = HashMap::new();
        let mut subscribers = HashMap::new();

        while let Some(cmd) = command_rx.recv().ok() {
            match cmd {
                IceoryxCommand::Publish { topic, payload, response_tx } => {
                    let publisher = if let Some(publisher) = publishers.get(&topic) {
                        publisher
                    } else {
                        match node.service_builder(&topic.as_str().try_into().unwrap())
                            .publish_subscribe::<[u8]>()
                            .history_size(16)
                            .open_or_create()
                        {
                            Ok(port_factory) => {
                                match port_factory.publisher_builder().initial_max_slice_len(config.buffer_size).create() {
                                    Ok(publisher) => {
                                        publishers.insert(topic.clone(), publisher);
                                        publishers.get(&topic).unwrap()
                                    }
                                    Err(e) => {
                                        let _ = response_tx.send(Err(TransportError::Publish(e.to_string())));
                                        continue;
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = response_tx.send(Err(TransportError::Publish(e.to_string())));
                                continue;
                            }
                        }
                    };

                    let result = match publisher.loan_slice_uninit(payload.len()) {
                        Ok(sample) => {
                            match sample.write_from_slice(payload.as_slice()).send() {
                                Ok(count) => {
                                    debug!("Published data to topic {topic} and was received by {count} remote subscribers");
                                    Ok(())
                                }
                                Err(e) => Err(TransportError::Publish(e.to_string())),
                            }
                        }
                        Err(e) => Err(TransportError::Publish(e.to_string())),
                    };

                    let _ = response_tx.send(result);
                }
                IceoryxCommand::Receive { topic, response_tx } => {
                    let subscriber = if let Some(subscriber) = subscribers.get(&topic) {
                        subscriber
                    } else {
                        match node.service_builder(&topic.as_str().try_into().unwrap())
                            .publish_subscribe::<[u8]>()
                            .history_size(16)
                            .open_or_create() { // open_or_create because remote may not have created the response node yet
                            Ok(port_factory) => {
                                match port_factory.subscriber_builder().buffer_size(config.buffer_size).create() {
                                    Ok(subscriber) => {
                                        subscribers.insert(topic.clone(), subscriber);
                                        subscribers.get(&topic).unwrap()
                                    }
                                    Err(e) => {
                                        let _ = response_tx.send(Err(TransportError::Receive(e.to_string())));
                                        continue;
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = response_tx.send(Err(TransportError::Publish(e.to_string())));
                                continue;
                            }
                        }
                    };

                    match subscriber.receive() {
                        Ok(sample_opt) => {
                            let received_data = sample_opt.map(|sample| sample.payload().to_vec());
                            let _ = response_tx.send(Ok(received_data));
                        }
                        Err(e) => {
                            let _ = response_tx.send(Err(TransportError::Publish(e.to_string())));
                            continue;
                        }
                    }
                }
            }

        }
    }
}

impl Transport for IceoryxTransport {
    async fn initialize(&mut self) -> Result<(), TransportError> {
        self.active = true;
        Ok(())
    }

    async fn publish(&self, topic: &str, message: &[u8]) -> Result<(), TransportError> {
        if !self.active {
            return Err(TransportError::Publish("iceoryx2 transport is not active".to_owned()));
        }

        // Create a oneshot channel for the response
        let (response_tx, response_rx) = oneshot::channel();

        if let Err(e) = self.command_tx.send(IceoryxCommand::Publish {
            topic: topic.to_string(),
            payload: message.to_vec(),
            response_tx,
        }) {
            return Err(TransportError::Publish(format!("Failed to send publish command: {e}")));
        }

        // Wait for response
        match response_rx.await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(TransportError::Publish(e.to_string())),
            Err(e) => Err(TransportError::Publish(e.to_string())),
        }
    }

    async fn receive(&self, topic: &str) -> Result<Option<Vec<u8>>, TransportError> {
        if !self.active {
            return Err(TransportError::Receive("iceoryx2 transport is not active".to_owned()));
        }

        let (response_tx, response_rx) = oneshot::channel();

        if let Err(e) = self.command_tx.send(IceoryxCommand::Receive {
            topic: topic.to_string(),
            response_tx,
        }) {
            return Err(TransportError::Receive(format!("Failed to send receive command: {e}")));
        }

        // Wait for response
        match response_rx.await {
            Ok(Ok(payload)) => Ok(payload),
            Ok(Err(e)) => Err(TransportError::Receive(e.to_string())),
            Err(e) => Err(TransportError::Receive(e.to_string())),
        }
    }

    fn name(&self) -> &str {
        "iceoryx2"
    }

    fn is_active(&self) -> bool {
        self.active
    }
}