use crate::config::NngConfig;
use crate::core::traits::Transport;
use crate::error::TransportError;
use futures::lock::Mutex;
use nng::options::Options;
use std::collections::HashMap;
use tracing::info;

pub struct NngTransport {
    active: bool,
    config: NngConfig,
    publishers: Mutex<HashMap<String, nng::Socket>>,
    subscribers: Mutex<HashMap<String, nng::Socket>>,
}

impl NngTransport {
    pub fn new(config: NngConfig) -> Result<Self, TransportError> {
        Ok(Self {
            active: false,
            config,
            publishers: Mutex::new(HashMap::new()),
            subscribers: Mutex::new(HashMap::new()),
        })
    }

    #[cfg(unix)]
    fn create_path(&self) -> Result<(), TransportError> {
        // Ensure the path exists if it's ipc
        if self.config.transport.as_str() == "ipc" {
            let path = std::path::Path::new(self.config.url.as_str());
            if !path.exists() {
                info!("Creating nng directory");
                std::fs::create_dir_all(path).map_err(|e| {
                    TransportError::Initialize(format!("Failed to create directory: {}", e))
                })?;
            }
        }
        Ok(())
    }

    async fn create_publisher(&self, topic: &str) -> Result<nng::Socket, TransportError> {
        // Create a socket with pub pattern
        let socket = nng::Socket::new(nng::Protocol::Pub0).map_err(|e| {
            TransportError::Initialize(format!("Failed to create NNG pub socket: {}", e))
        })?;

        #[cfg(unix)]
        self.create_path()?;

        // Construct the URL - this will differ based on platform
        #[cfg(unix)]
        let url = format!(
            "{}://{}/{}.ipc",
            self.config.transport, self.config.url, topic
        );
        #[cfg(windows)]
        let url = format!("{}/{}", self.url_base, topic);

        // Bind the socket to the address
        socket
            .listen(&url)
            .map_err(|e| TransportError::Initialize(format!("Failed to bind NNG socket: {}", e)))?;

        info!("Created publisher with URL: {}", url);

        // Return the configured socket
        Ok(socket)
    }

    async fn create_subscriber(&self, topic: &str) -> Result<nng::Socket, TransportError> {
        // Create socket with sub pattern
        let socket = nng::Socket::new(nng::Protocol::Sub0).map_err(|e| {
            TransportError::Initialize(format!("Failed to create NNG socket: {}", e))
        })?;

        #[cfg(unix)]
        self.create_path()?;

        // Construct the URL - this will differ based on platform
        #[cfg(unix)]
        let url = format!(
            "{}://{}/{}.ipc",
            self.config.transport, self.config.url, topic
        );
        #[cfg(windows)]
        let url = format!("{}/{}", self.url_base, topic);

        // Bind socket to address
        socket.dial(&url).map_err(|e| {
            TransportError::Initialize(format!("Failed to connect to NNG socket: {}", e))
        })?;

        info!("Created subscriber with URL: {}", url);

        socket
            .set_opt::<nng::options::protocol::pubsub::Subscribe>(vec![])
            .map_err(|e| TransportError::Initialize(e.to_string()))?;

        // Return configured socket
        Ok(socket)
    }
}

impl Transport for NngTransport {
    async fn initialize(&mut self) -> Result<(), TransportError> {
        self.active = true;
        Ok(())
    }

    async fn publish(&self, topic: &str, message: &[u8]) -> Result<(), TransportError> {
        if !self.active {
            return Err(TransportError::Publish(
                "NNG transport is not active".to_owned(),
            ));
        }

        // Get or create publisher for this topic
        let socket = {
            let mut publishers = self.publishers.lock().await;
            if !publishers.contains_key(topic) {
                let new_socket = self.create_publisher(topic).await?;
                publishers.insert(topic.to_string(), new_socket);
            }
            publishers.get(topic).unwrap().clone()
        };

        // With NNG pub/sub, we need to prepend the topic to the message
        // Create a new buffer with topic prefix + message
        let mut data = Vec::with_capacity(topic.len() + 1 + message.len());
        data.extend_from_slice(topic.as_bytes());
        data.push(b':'); // Use a separator between topic and payload
        data.extend_from_slice(message);

        // Send the data through the socket
        socket.send(&data).map_err(|e| {
            TransportError::Publish(format!("Failed to publish message: {}", e.1))
        })?;

        Ok(())
    }

    async fn receive(&self, topic: &str) -> Result<Option<Vec<u8>>, TransportError> {
        if !self.active {
            return Err(TransportError::Receive(
                "NNG transport is not active".to_owned(),
            ));
        }

        // Get or create subscriber for this topic
        let socket = {
            let mut subscribers = self.subscribers.lock().await;
            if !subscribers.contains_key(topic) {
                let new_socket = self.create_subscriber(topic).await?;
                subscribers.insert(topic.to_string(), new_socket);
            }
            subscribers.get(topic).unwrap().clone()
        };

        let message = match socket.recv() {
            Ok(msg) => Ok(Some(msg)),
            Err(nng::Error::Closed) => Ok(None), // Socket was closed, no message available
            Err(nng::Error::TimedOut) => Ok(None), // No message within timeout period
            Err(e) => Err(TransportError::Receive(e.to_string())),
        }
        .map_err(|e| TransportError::Receive(format!("Task join error: {}", e)))?;

        // If we got a message, we need to strip the topic prefix
        if let Some(data) = message {
            // Format is "topic:payload", so we need to find the payload part
            if let Some(pos) = data.iter().position(|&b| b == b':') {
                // Return everything after the topic prefix and separator
                return Ok(Some(data[pos + 1..].to_vec()));
            } else {
                // No separator found - either malformed message or empty payload
                return Ok(Some(data.to_vec()));
            }
        }

        Ok(None)
    }

    fn name(&self) -> &str {
        "nng"
    }

    fn is_active(&self) -> bool {
        self.active
    }
}
