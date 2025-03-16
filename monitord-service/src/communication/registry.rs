//! Registry for dynamic loading of transport providers

use crate::communication::config::CommunicationConfig;
use crate::communication::core::traits::Transport;
use crate::communication::error::CommunicationError;
use std::collections::HashMap;
use std::sync::RwLock;

/// Factory for creating transport instances
pub trait TransportFactory: Send + Sync + 'static {
    /// Create a new transport instance
    fn create_transport(
        &self,
        config: &CommunicationConfig,
    ) -> Result<Box<dyn Transport>, CommunicationError>;

    /// Get the name of the transport type
    fn name(&self) -> &str;
}

/// Registry for transport factories
pub struct TransportRegistry {
    factories: RwLock<HashMap<String, Box<dyn TransportFactory>>>,
}

impl TransportRegistry {
    /// Create a new transport registry
    pub fn new() -> Self {
        Self {
            factories: RwLock::new(HashMap::new()),
        }
    }

    /// Register a transport factory
    pub fn register<F>(&self, factory: F) -> Result<(), CommunicationError>
    where
        F: TransportFactory + 'static,
    {
        let name = factory.name().to_string();
        let mut factories = self.factories.write().map_err(|e| {
            CommunicationError::Registry(format!("Failed to acquire write lock: {}", e))
        })?;

        factories.insert(name.clone(), Box::new(factory));
        Ok(())
    }

    /// Create transports from configuration
    pub fn create_transports(
        &self,
        config: &CommunicationConfig,
    ) -> Result<Vec<Box<dyn Transport>>, CommunicationError> {
        let factories = self.factories.read().map_err(|e| {
            CommunicationError::Registry(format!("Failed to acquire read lock: {}", e))
        })?;

        let mut transports = Vec::new();

        // Create transports from each registered factory
        for factory in factories.values() {
            match factory.create_transport(config) {
                Ok(transport) => transports.push(transport),
                Err(e) => {
                    // Log the error but continue with other transports
                    eprintln!("Failed to create transport {}: {}", factory.name(), e);
                }
            }
        }

        if transports.is_empty() {
            return Err(CommunicationError::Registry(
                "No transports could be created".into(),
            ));
        }

        Ok(transports)
    }
}

impl Default for TransportRegistry {
    fn default() -> Self {
        Self::new()
    }
}
