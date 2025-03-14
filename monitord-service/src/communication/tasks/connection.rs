//! Connection handling tasks

use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio::time::Duration;
use crate::communication::core::traits::Transport;
use crate::communication::subscription::manager::SubscriptionManager;
use crate::communication::error::CommunicationError;
use crate::communication::core::traits::MessageHandler;

/// Connection task parameters
pub struct ConnectionTask {
    /// Connection listening frequency
    pub frequency: Duration,
    /// Available transports
    pub transports: Vec<Arc<dyn Transport>>,
    /// Subscription manager
    pub subscription_manager: Arc<SubscriptionManager>,
    /// Message handler
    pub message_handler: Arc<dyn MessageHandler>,
    /// Channel for shutdown signals
    pub shutdown: tokio::sync::broadcast::Receiver<()>,
}

/// Spawn a task to handle client connections
pub fn spawn_connection_handler(task: ConnectionTask) -> JoinHandle<Result<(), CommunicationError>> {
    tokio::spawn(async move {
        let ConnectionTask {
            frequency,
            transports,
            subscription_manager,
            message_handler,
            mut shutdown,
        } = task;

        loop {
            // Check for shutdown signal
            if shutdown.try_recv().is_ok() {
                break;
            }

            // Check each transport for new connections
            for transport in &transports {
                match transport.listen_for_connections().await {
                    Ok(Some(connection)) => {
                        // Process new connection
                        // This would typically involve adding the client to the registry
                        // and preparing for subscription requests
                        tracing::info!(
                            "New client connection: {} (pid: {}) via {}",
                            connection.client_id,
                            connection.pid,
                            transport.name()
                        );
                    }
                    Ok(None) => {
                        // No new connections
                    }
                    Err(e) => {
                        tracing::error!(
                            "Error listening for connections on {}: {}",
                            transport.name(),
                            e
                        );
                    }
                }
            }

            // Wait before checking again
            tokio::time::sleep(frequency).await;
        }

        Ok(())
    })
}