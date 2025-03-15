use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::RwLock;
use tokio::task::{JoinHandle, JoinSet};
use tracing::{debug, error, info, warn};

use crate::communication::core::traits::{MessageHandler, Transport};
use crate::communication::core::models::DataType;
use crate::communication::core::traits::MessageType;
use crate::communication::error::CommunicationError;
use crate::communication::config::CommunicationConfig;
use crate::communication::subscription::manager::SubscriptionManager;
use crate::communication::subscription::config::SubscriptionConfig;
use crate::communication::tasks::connection;
use crate::communication::tasks::data::{self, DataTask};
use crate::communication::transports;

use monitord_protocols::monitord::{
    CpuInfo, MemoryInfo, GpuInfo, NetworkInfo, ProcessInfo,
    StorageInfo, SystemInfo
};

/// Main communication manager that coordinates transports and message handling
pub struct CommunicationManager {
    /// Available transport mechanisms
    transports: Vec<Arc<dyn Transport>>,
    /// Message serialization/deserialization handler
    message_handler: Arc<dyn MessageHandler>,
    /// Subscription manager
    subscription_manager: Arc<SubscriptionManager>,
    /// Task handles for background operations
    task_handles: RwLock<Vec<JoinHandle<Result<(), CommunicationError>>>>,
    /// Shutdown signal broadcaster
    shutdown_sender: Sender<()>,
    /// Configuration settings
    config: CommunicationConfig,
}

impl CommunicationManager {
    /// Create a new communication manager
    pub fn new(
        config: CommunicationConfig,
        message_handler: Arc<dyn MessageHandler>,
    ) -> Result<Self, CommunicationError> {
        // Initialize transports
        let transports = transports::create_transports(&config)?;
        
        

        if transports.is_empty() {
            return Err(CommunicationError::Init("No transport mechanisms could be initialized".into()));
        }

        // Initialize the subscription manager
        let subscription_manager = match SubscriptionManager::new(config.subscription.clone()) {
            Ok(manager) => Arc::new(manager),
            Err(e) => return Err(CommunicationError::Subscription(e)),
        };

        // Create shutdown channel
        let (shutdown_sender, _) = tokio::sync::broadcast::channel(1);

        Ok(Self {
            transports,
            message_handler: Arc::clone(&message_handler),
            subscription_manager,
            task_handles: RwLock::new(Vec::new()),
            shutdown_sender,
            config,
        })
    }

    /// Map DataType to MessageType
    fn data_type_to_message_type(data_type: DataType) -> MessageType {
        match data_type {
            DataType::System => MessageType::SystemInfo,
            DataType::Cpu => MessageType::CpuInfo,
            DataType::Memory => MessageType::MemoryInfo,
            DataType::Gpu => MessageType::GpuInfo,
            DataType::Network => MessageType::NetworkInfo,
            DataType::Process => MessageType::ProcessInfo,
            DataType::Storage => MessageType::StorageInfo,
        }
    }

    /// Run the communication manager
    pub async fn run(
        self,
        cpu_rx: Receiver<CpuInfo>,
        memory_rx: Receiver<MemoryInfo>,
        gpu_rx: Receiver<Vec<GpuInfo>>,
        network_rx: Receiver<Vec<NetworkInfo>>,
        process_rx: Receiver<Vec<ProcessInfo>>,
        storage_rx: Receiver<Vec<StorageInfo>>,
        system_rx: Receiver<SystemInfo>,
    ) -> Result<(), CommunicationError> {
        info!("Starting communication manager");

        // Create Arc references to share with tasks
        let transports_arc = self.transports.clone();
        let subscription_manager_arc = self.subscription_manager.clone();
        let message_handler_arc = self.message_handler.clone();

        // Create a new JoinSet to hold all our tasks
        let mut task_set = JoinSet::new();

        // Spawn connection handler
        task_set.spawn(connection::spawn_connection_handler(
            connection::ConnectionTask {
                frequency: self.config.connection_frequency,
                transports: transports_arc.clone(),
                subscription_manager: subscription_manager_arc.clone(),
                message_handler: message_handler_arc.clone(),
                shutdown: self.shutdown_sender.subscribe(),
            }
        ));

        // Spawn data handlers
        let data_handlers = data::create_data_handlers(
            cpu_rx,
            memory_rx,
            gpu_rx,
            network_rx,
            process_rx,
            storage_rx,
            system_rx,
            transports_arc.clone(),
            subscription_manager_arc,
            message_handler_arc,
            &self.shutdown_sender,
        );

        // Add all data handlers to the task set
        for handler in data_handlers {
            task_set.spawn(async move { handler.await });
        }

        // Wait for tasks to complete or for an error
        while let Some(result) = task_set.join_next().await {
            match result {
                Ok(Ok(_)) => {
                    // Task completed successfully
                    debug!("Task completed successfully");
                },
                Ok(Err(e)) => {
                    // Task returned an error
                    error!("Task error: {}", e);
                    // Signal all other tasks to shut down
                    let _ = self.shutdown_sender.send(());
                    return Err(CommunicationError::Task(e.to_string()));
                },
                Err(e) => {
                    // Task panicked or was cancelled
                    error!("Task join error: {}", e);
                    // Signal all other tasks to shut down
                    let _ = self.shutdown_sender.send(());
                    return Err(CommunicationError::Task(e.to_string()));
                }
            }
        }

        info!("Communication manager tasks completed");
        Ok(())
    }

    /// Create a subscription task
    async fn create_subscription_tasks(&self) -> Result<Vec<JoinHandle<Result<(), CommunicationError>>>, CommunicationError> {
        let mut tasks = Vec::new();

        // Here we would typically set up background tasks to handle subscription operations
        // This might include tasks that poll for subscription changes and update routing tables
        // For now, we'll leave this as a placeholder

        Ok(tasks)
    }

    /// Shutdown the communication manager
    pub async fn shutdown(self) -> Result<(), CommunicationError> {
        info!("Shutting down communication manager");

        // Signal all tasks to stop
        let _ = self.shutdown_sender.send(());

        // Wait for all tasks to complete
        let mut task_handles = self.task_handles.write().await;
        for handle in task_handles.drain(..) {
            if let Err(e) = handle.await {
                warn!("Task error during shutdown: {}", e);
            }
        }

        info!("Communication manager shutdown complete");
        Ok(())
    }

    /// Check health of the communication manager
    pub async fn check_health(&self) -> Result<(), CommunicationError> {
        // Check if all transports are active
        for transport in &self.transports {
            if !transport.is_active() {
                return Err(CommunicationError::Transport(
                    format!("Transport {} is not active", transport.name())
                ));
            }
        }

        // Check if any tasks have failed
        let task_handles = self.task_handles.read().await;
        for (i, handle) in task_handles.iter().enumerate() {
            if handle.is_finished() {
                return Err(CommunicationError::Task(
                    format!("Task {} has stopped unexpectedly", i)
                ));
            }
        }

        Ok(())
    }

    /// Get the number of active transports
    pub fn transport_count(&self) -> usize {
        self.transports.len()
    }

    /// Get the names of active transports
    pub fn transport_names(&self) -> Vec<String> {
        self.transports.iter().map(|t| t.name().to_string()).collect()
    }
}

impl Drop for CommunicationManager {
    fn drop(&mut self) {
        // Ensure we signal all tasks to shut down when the manager is dropped
        let _ = self.shutdown_sender.send(());
    }
}