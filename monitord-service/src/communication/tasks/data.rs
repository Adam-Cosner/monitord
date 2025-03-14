//! Data handling tasks for different system metrics
//!
//! This module contains task implementations for handling the various types
//! of system metrics data collected by monitord. Each data type has a dedicated
//! handler function that processes incoming data and publishes it to subscribed clients
//! using the appropriate transport mechanisms.

use std::sync::Arc;
use tokio::sync::broadcast::Receiver;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};
use prost::Message;

use crate::communication::core::traits::{message_utils, MessageType, Transport};
use crate::communication::core::traits::MessageHandler;
use crate::communication::core::models::{DataType, TransportType};
use crate::communication::subscription::manager::SubscriptionManager;
use crate::communication::subscription::models::Subscription;
use crate::communication::error::CommunicationError;
use monitord_protocols::monitord::{
    CpuInfo, MemoryInfo, GpuInfo, NetworkInfo, ProcessInfo, StorageInfo, SystemInfo
};

/// Parameters for data handling tasks
pub struct DataTask {
    /// Type of system data being handled
    pub data_type: DataType,

    /// Corresponding message type for serialization
    pub message_type: MessageType,

    /// Available transport mechanisms
    pub transports: Vec<Arc<dyn Transport>>,

    /// Subscription manager
    pub subscription_manager: Arc<SubscriptionManager>,

    /// Message serialization/deserialization handler
    pub message_handler: Arc<dyn MessageHandler>,

    /// Channel for receiving shutdown signals
    pub shutdown: tokio::sync::broadcast::Receiver<()>,
}

/// Spawn a task to handle CPU data
pub fn spawn_cpu_data_handler(
    mut receiver: Receiver<CpuInfo>,
    task: DataTask,
) -> JoinHandle<Result<(), CommunicationError>> {
    tokio::spawn(async move {
        let DataTask {
            data_type,
            message_type,
            transports,
            subscription_manager,
            message_handler,
            mut shutdown,
        } = task;

        info!("Started CPU data handler task");

        loop {
            tokio::select! {
                // Check for shutdown signal
                _ = shutdown.recv() => {
                    info!("Shutting down CPU data handler task");
                    break;
                }

                // Process incoming CPU data
                result = receiver.recv() => {
                    match result {
                        Ok(data) => {
                            debug!("Received CPU data: utilization={}%", data.global_utilization_percent);
                            process_message(
                                &data,
                                data_type,
                                message_type,
                                &transports,
                                &subscription_manager,
                                &message_handler
                            ).await?;
                        }
                        Err(e) => {
                            error!("Failed to receive CPU data: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    })
}

/// Spawn a task to handle Memory data
pub fn spawn_memory_data_handler(
    mut receiver: Receiver<MemoryInfo>,
    task: DataTask,
) -> JoinHandle<Result<(), CommunicationError>> {
    tokio::spawn(async move {
        let DataTask {
            data_type,
            message_type,
            transports,
            subscription_manager,
            message_handler,
            mut shutdown,
        } = task;

        info!("Started Memory data handler task");

        loop {
            tokio::select! {
                // Check for shutdown signal
                _ = shutdown.recv() => {
                    info!("Shutting down Memory data handler task");
                    break;
                }

                // Process incoming Memory data
                result = receiver.recv() => {
                    match result {
                        Ok(data) => {
                            debug!("Received Memory data: used={}MB, free={}MB",
                                   data.used_memory_bytes / (1024 * 1024),
                                   data.free_memory_bytes / (1024 * 1024));
                            process_message(
                                &data,
                                data_type,
                                message_type,
                                &transports,
                                &subscription_manager,
                                &message_handler
                            ).await?;
                        }
                        Err(e) => {
                            error!("Failed to receive Memory data: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    })
}

/// Spawn a task to handle GPU data
pub fn spawn_gpu_data_handler(
    mut receiver: Receiver<Vec<GpuInfo>>,
    task: DataTask,
) -> JoinHandle<Result<(), CommunicationError>> {
    tokio::spawn(async move {
        let DataTask {
            data_type,
            message_type,
            transports,
            subscription_manager,
            message_handler,
            mut shutdown,
        } = task;

        info!("Started GPU data handler task");

        loop {
            tokio::select! {
                // Check for shutdown signal
                _ = shutdown.recv() => {
                    info!("Shutting down GPU data handler task");
                    break;
                }

                // Process incoming GPU data
                result = receiver.recv() => {
                    match result {
                        Ok(gpu_list) => {
                            debug!("Received GPU data for {} devices", gpu_list.len());

                            // Get all subscriptions
                            let subscriptions = match subscription_manager.get_subscriptions_for_type(data_type).await {
                                Ok(subs) => subs,
                                Err(e) => {
                                    error!("Failed to get GPU subscriptions: {}", e);
                                    continue;
                                }
                            };

                            if subscriptions.is_empty() {
                                continue;
                            }

                            // For each GPU, check if anyone is subscribed and publish individually
                            for gpu in &gpu_list {
                                process_gpu_info(
                                    gpu,
                                    &subscriptions,
                                    data_type,
                                    message_type,
                                    &transports,
                                    &message_handler
                                ).await?;
                            }
                        }
                        Err(e) => {
                            error!("Failed to receive GPU data: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    })
}

/// Spawn a task to handle Network data
pub fn spawn_network_data_handler(
    mut receiver: Receiver<Vec<NetworkInfo>>,
    task: DataTask,
) -> JoinHandle<Result<(), CommunicationError>> {
    tokio::spawn(async move {
        let DataTask {
            data_type,
            message_type,
            transports,
            subscription_manager,
            message_handler,
            mut shutdown,
        } = task;

        info!("Started Network data handler task");

        loop {
            tokio::select! {
                // Check for shutdown signal
                _ = shutdown.recv() => {
                    info!("Shutting down Network data handler task");
                    break;
                }

                // Process incoming Network data
                result = receiver.recv() => {
                    match result {
                        Ok(network_list) => {
                            debug!("Received Network data for {} interfaces", network_list.len());

                            // Get all subscriptions
                            let subscriptions = match subscription_manager.get_subscriptions_for_type(data_type).await {
                                Ok(subs) => subs,
                                Err(e) => {
                                    error!("Failed to get Network subscriptions: {}", e);
                                    continue;
                                }
                            };

                            if subscriptions.is_empty() {
                                continue;
                            }

                            // For each interface, check if anyone is subscribed and publish individually
                            for network in &network_list {
                                process_network_info(
                                    network,
                                    &subscriptions,
                                    data_type,
                                    message_type,
                                    &transports,
                                    &message_handler
                                ).await?;
                            }
                        }
                        Err(e) => {
                            error!("Failed to receive Network data: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    })
}

/// Spawn a task to handle Process data
pub fn spawn_process_data_handler(
    mut receiver: Receiver<Vec<ProcessInfo>>,
    task: DataTask,
) -> JoinHandle<Result<(), CommunicationError>> {
    tokio::spawn(async move {
        let DataTask {
            data_type,
            message_type,
            transports,
            subscription_manager,
            message_handler,
            mut shutdown,
        } = task;

        info!("Started Process data handler task");

        loop {
            tokio::select! {
                // Check for shutdown signal
                _ = shutdown.recv() => {
                    info!("Shutting down Process data handler task");
                    break;
                }

                // Process incoming Process data
                result = receiver.recv() => {
                    match result {
                        Ok(process_list) => {
                            debug!("Received Process data for {} processes", process_list.len());

                            // Get all subscriptions
                            let subscriptions = match subscription_manager.get_subscriptions_for_type(data_type).await {
                                Ok(subs) => subs,
                                Err(e) => {
                                    error!("Failed to get Process subscriptions: {}", e);
                                    continue;
                                }
                            };

                            if subscriptions.is_empty() {
                                continue;
                            }

                            // For each process, check if anyone is subscribed and publish individually
                            for process in &process_list {
                                process_process_info(
                                    process,
                                    &subscriptions,
                                    data_type,
                                    message_type,
                                    &transports,
                                    &message_handler
                                ).await?;
                            }
                        }
                        Err(e) => {
                            error!("Failed to receive Process data: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    })
}

/// Spawn a task to handle Storage data
pub fn spawn_storage_data_handler(
    mut receiver: Receiver<Vec<StorageInfo>>,
    task: DataTask,
) -> JoinHandle<Result<(), CommunicationError>> {
    tokio::spawn(async move {
        let DataTask {
            data_type,
            message_type,
            transports,
            subscription_manager,
            message_handler,
            mut shutdown,
        } = task;

        info!("Started Storage data handler task");

        loop {
            tokio::select! {
                // Check for shutdown signal
                _ = shutdown.recv() => {
                    info!("Shutting down Storage data handler task");
                    break;
                }

                // Process incoming Storage data
                result = receiver.recv() => {
                    match result {
                        Ok(storage_list) => {
                            debug!("Received Storage data for {} devices", storage_list.len());

                            // Get all subscriptions
                            let subscriptions = match subscription_manager.get_subscriptions_for_type(data_type).await {
                                Ok(subs) => subs,
                                Err(e) => {
                                    error!("Failed to get Storage subscriptions: {}", e);
                                    continue;
                                }
                            };

                            if subscriptions.is_empty() {
                                continue;
                            }

                            // For each storage device, check if anyone is subscribed and publish individually
                            for storage in &storage_list {
                                process_storage_info(
                                    storage,
                                    &subscriptions,
                                    data_type,
                                    message_type,
                                    &transports,
                                    &message_handler
                                ).await?;
                            }
                        }
                        Err(e) => {
                            error!("Failed to receive Storage data: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    })
}

/// Spawn a task to handle System data
pub fn spawn_system_data_handler(
    mut receiver: Receiver<SystemInfo>,
    task: DataTask,
) -> JoinHandle<Result<(), CommunicationError>> {
    tokio::spawn(async move {
        let DataTask {
            data_type,
            message_type,
            transports,
            subscription_manager,
            message_handler,
            mut shutdown,
        } = task;

        info!("Started System data handler task");

        loop {
            tokio::select! {
                // Check for shutdown signal
                _ = shutdown.recv() => {
                    info!("Shutting down System data handler task");
                    break;
                }

                // Process incoming System data
                result = receiver.recv() => {
                    match result {
                        Ok(data) => {
                            debug!("Received System data: hostname={}", data.hostname);
                            process_message(
                                &data,
                                data_type,
                                message_type,
                                &transports,
                                &subscription_manager,
                                &message_handler
                            ).await?;
                        }
                        Err(e) => {
                            error!("Failed to receive System data: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    })
}

/// Generic function to process a message of any type
async fn process_message<T: Message + Clone + Send + 'static>(
    data: &T,
    data_type: DataType,
    message_type: MessageType,
    transports: &[Arc<dyn Transport>],
    subscription_manager: &SubscriptionManager,
    message_handler: &Arc<dyn MessageHandler>,
) -> Result<(), CommunicationError> {
    // Get relevant subscriptions
    let subscriptions = match subscription_manager.get_subscriptions_for_type(data_type).await {
        Ok(subs) => subs,
        Err(e) => {
            error!("Failed to get subscriptions: {}", e);
            return Ok(());
        }
    };

    if subscriptions.is_empty() {
        return Ok(());
    }

    // Serialize the data once
    let payload = match message_utils::serialize(
        message_handler.as_ref(),
        message_type,
        data
    ) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to serialize data: {}", e);
            return Ok(());
        }
    };

    // For each subscription, find the appropriate transport and publish
    for subscription in subscriptions {
        // Find the right transport
        let transport = transports.iter().find(|t| {
            matches!(subscription.transport, TransportType::Iceoryx) && t.name() == "iceoryx" ||
                matches!(subscription.transport, TransportType::Grpc) && t.name() == "grpc"
        });

        if let Some(transport) = transport {
            // Format the topic name
            let topic = format!(
                "{}/{}",
                data_type,
                subscription.id
            );

            // Publish the data
            if let Err(e) = transport.publish(&topic, &payload).await {
                error!("Failed to publish to {}: {}", topic, e);
            } else {
                debug!("Published data to topic {}", topic);
            }
        } else {
            warn!("No matching transport found for subscription {}", subscription.id);
        }
    }

    Ok(())
}

/// Process GPU info with filtering based on subscriptions
async fn process_gpu_info(
    gpu: &GpuInfo,
    subscriptions: &[Subscription],
    data_type: DataType,
    message_type: MessageType,
    transports: &[Arc<dyn Transport>],
    message_handler: &Arc<dyn MessageHandler>,
) -> Result<(), CommunicationError> {
    // Serialize the data once
    let payload = match message_utils::serialize(
        message_handler.as_ref(),
        message_type,
        gpu
    ) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to serialize GPU data: {}", e);
            return Ok(());
        }
    };

    // For each subscription, check if this GPU matches the filter
    for subscription in subscriptions {
        // Check if this GPU matches the subscription filter
        if let Some(filter) = &subscription.filter {
            if let monitord_protocols::subscription::subscription_request::Filter::GpuFilter(gpu_filter) = filter {
                // Skip if GPU name or vendor doesn't match filter
                if !gpu_filter.name.is_empty() && !gpu_filter.name.contains(&gpu.name) {
                    continue;
                }
                if !gpu_filter.vendor.is_empty() && !gpu_filter.vendor.contains(&gpu.vendor) {
                    continue;
                }
            }
        }

        // Find the right transport
        let transport = transports.iter().find(|t| {
            matches!(subscription.transport, TransportType::Iceoryx) && t.name() == "iceoryx" ||
                matches!(subscription.transport, TransportType::Grpc) && t.name() == "grpc"
        });

        if let Some(transport) = transport {
            // Format the topic name
            let topic = format!(
                "{}/{}/{}",
                data_type,
                gpu.name.replace(" ", "_"),
                subscription.id
            );

            // Publish the data
            if let Err(e) = transport.publish(&topic, &payload).await {
                error!("Failed to publish GPU data to {}: {}", topic, e);
            } else {
                debug!("Published GPU data for {} to topic {}", gpu.name, topic);
            }
        }
    }

    Ok(())
}

/// Process Network info with filtering based on subscriptions
async fn process_network_info(
    network: &NetworkInfo,
    subscriptions: &[Subscription],
    data_type: DataType,
    message_type: MessageType,
    transports: &[Arc<dyn Transport>],
    message_handler: &Arc<dyn MessageHandler>,
) -> Result<(), CommunicationError> {
    // Serialize the data once
    let payload = match message_utils::serialize(
        message_handler.as_ref(),
        message_type,
        network
    ) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to serialize Network data: {}", e);
            return Ok(());
        }
    };

    // For each subscription, check if this network interface matches the filter
    for subscription in subscriptions {
        // Check if this interface matches the subscription filter
        if let Some(filter) = &subscription.filter {
            if let monitord_protocols::subscription::subscription_request::Filter::NetworkFilter(net_filter) = filter {
                // Skip if interface name doesn't match filter
                if !net_filter.interface_name.is_empty() && !net_filter.interface_name.contains(&network.interface_name) {
                    continue;
                }
            }
        }

        // Find the right transport
        let transport = transports.iter().find(|t| {
            matches!(subscription.transport, TransportType::Iceoryx) && t.name() == "iceoryx" ||
                matches!(subscription.transport, TransportType::Grpc) && t.name() == "grpc"
        });

        if let Some(transport) = transport {
            // Format the topic name
            let topic = format!(
                "{}/{}/{}",
                data_type,
                network.interface_name,
                subscription.id
            );

            // Publish the data
            if let Err(e) = transport.publish(&topic, &payload).await {
                error!("Failed to publish Network data to {}: {}", topic, e);
            } else {
                debug!("Published Network data for {} to topic {}", network.interface_name, topic);
            }
        }
    }

    Ok(())
}

/// Process Process info with filtering based on subscriptions
async fn process_process_info(
    process: &ProcessInfo,
    subscriptions: &[Subscription],
    data_type: DataType,
    message_type: MessageType,
    transports: &[Arc<dyn Transport>],
    message_handler: &Arc<dyn MessageHandler>,
) -> Result<(), CommunicationError> {
    // Serialize the data once
    let payload = match message_utils::serialize(
        message_handler.as_ref(),
        message_type,
        process
    ) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to serialize Process data: {}", e);
            return Ok(());
        }
    };

    // For each subscription, check if this process matches the filter
    for subscription in subscriptions {
        // Check if this process matches the subscription filter
        if let Some(filter) = &subscription.filter {
            if let monitord_protocols::subscription::subscription_request::Filter::ProcessFilter(proc_filter) = filter {
                // Skip if process doesn't match filter criteria
                if !proc_filter.pid.is_empty() && !proc_filter.pid.contains(&process.pid) {
                    continue;
                }
                if !proc_filter.name.is_empty() && !proc_filter.name.contains(&process.name) {
                    continue;
                }
                if !proc_filter.username.is_empty() && !proc_filter.username.contains(&process.username) {
                    continue;
                }

                // Skip if not in top processes by resource usage
                if proc_filter.top_by_cpu > 0 && process.cpu_usage_percent < proc_filter.top_by_cpu as f64 {
                    continue;
                }
                if proc_filter.top_by_memory > 0 && process.physical_memory_bytes < proc_filter.top_by_memory as u64 {
                    continue;
                }
                if proc_filter.top_by_disk > 0 && (process.disk_read_bytes_per_sec + process.disk_write_bytes_per_sec) < proc_filter.top_by_disk as u64 {
                    continue;
                }
            }
        }

        // Find the right transport
        let transport = transports.iter().find(|t| {
            matches!(subscription.transport, TransportType::Iceoryx) && t.name() == "iceoryx" ||
                matches!(subscription.transport, TransportType::Grpc) && t.name() == "grpc"
        });

        if let Some(transport) = transport {
            // Format the topic name
            let topic = format!(
                "{}/{}/{}",
                data_type,
                process.pid,
                subscription.id
            );

            // Publish the data
            if let Err(e) = transport.publish(&topic, &payload).await {
                error!("Failed to publish Process data to {}: {}", topic, e);
            } else {
                debug!("Published Process data for {} (pid {}) to topic {}", process.name, process.pid, topic);
            }
        }
    }

    Ok(())
}

/// Process Storage info with filtering based on subscriptions
async fn process_storage_info(
    storage: &StorageInfo,
    subscriptions: &[Subscription],
    data_type: DataType,
    message_type: MessageType,
    transports: &[Arc<dyn Transport>],
    message_handler: &Arc<dyn MessageHandler>,
) -> Result<(), CommunicationError> {
    // Serialize the data once
    let payload = match message_utils::serialize(
        message_handler.as_ref(),
        message_type,
        storage
    ) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to serialize Storage data: {}", e);
            return Ok(());
        }
    };

    // For each subscription, check if this storage device matches the filter
    for subscription in subscriptions {
        // Check if this storage device matches the subscription filter
        if let Some(filter) = &subscription.filter {
            if let monitord_protocols::subscription::subscription_request::Filter::StorageFilter(storage_filter) = filter {
                // Skip if device doesn't match filter criteria
                if !storage_filter.device_name.is_empty() && !storage_filter.device_name.contains(&storage.device_name) {
                    continue;
                }
                if !storage_filter.mount_point.is_empty() && !storage_filter.mount_point.contains(&storage.mount_point) {
                    continue;
                }
            }
        }

        // Find the right transport
        let transport = transports.iter().find(|t| {
            matches!(subscription.transport, TransportType::Iceoryx) && t.name() == "iceoryx" ||
                matches!(subscription.transport, TransportType::Grpc) && t.name() == "grpc"
        });

        if let Some(transport) = transport {
            // Format the topic name
            let topic = format!(
                "{}/{}/{}",
                data_type,
                storage.device_name.replace("/", "_"),
                subscription.id
            );

            // Publish the data
            if let Err(e) = transport.publish(&topic, &payload).await {
                error!("Failed to publish Storage data to {}: {}", topic, e);
            } else {
                debug!("Published Storage data for {} to topic {}", storage.device_name, topic);
            }
        }
    }

    Ok(())
}

/// Create all data handlers for the communication manager
pub fn create_data_handlers(
    cpu_rx: Receiver<CpuInfo>,
    memory_rx: Receiver<MemoryInfo>,
    gpu_rx: Receiver<Vec<GpuInfo>>,
    network_rx: Receiver<Vec<NetworkInfo>>,
    process_rx: Receiver<Vec<ProcessInfo>>,
    storage_rx: Receiver<Vec<StorageInfo>>,
    system_rx: Receiver<SystemInfo>,
    transports: Vec<Arc<dyn Transport>>,
    subscription_manager: Arc<SubscriptionManager>,
    message_handler: Arc<dyn MessageHandler>,
    shutdown_sender: &tokio::sync::broadcast::Sender<()>,
) -> Vec<JoinHandle<Result<(), CommunicationError>>> {
    let mut handlers = Vec::new();

    // Spawn CPU data handler
    handlers.push(spawn_cpu_data_handler(
        cpu_rx,
        DataTask {
            data_type: DataType::Cpu,
            message_type: MessageType::CpuInfo,
            transports: transports.clone(),
            subscription_manager: Arc::clone(&subscription_manager),
            message_handler: message_handler.clone(),
            shutdown: shutdown_sender.subscribe(),
        },
    ));

    // Spawn Memory data handler
    handlers.push(spawn_memory_data_handler(
        memory_rx,
        DataTask {
            data_type: DataType::Memory,
            message_type: MessageType::MemoryInfo,
            transports: transports.clone(),
            subscription_manager: Arc::clone(&subscription_manager),
            message_handler: message_handler.clone(),
            shutdown: shutdown_sender.subscribe(),
        },
    ));

    // Spawn GPU data handler
    handlers.push(spawn_gpu_data_handler(
        gpu_rx,
        DataTask {
            data_type: DataType::Gpu,
            message_type: MessageType::GpuInfo,
            transports: transports.clone(),
            subscription_manager: Arc::clone(&subscription_manager),
            message_handler: message_handler.clone(),
            shutdown: shutdown_sender.subscribe(),
        },
    ));

    // Spawn Network data handler
    handlers.push(spawn_network_data_handler(
        network_rx,
        DataTask {
            data_type: DataType::Network,
            message_type: MessageType::NetworkInfo,
            transports: transports.clone(),
            subscription_manager: Arc::clone(&subscription_manager),
            message_handler: message_handler.clone(),
            shutdown: shutdown_sender.subscribe(),
        },
    ));

    // Spawn Process data handler
    handlers.push(spawn_process_data_handler(
        process_rx,
        DataTask {
            data_type: DataType::Process,
            message_type: MessageType::ProcessInfo,
            transports: transports.clone(),
            subscription_manager: Arc::clone(&subscription_manager),
            message_handler: message_handler.clone(),
            shutdown: shutdown_sender.subscribe(),
        },
    ));

    // Spawn Storage data handler
    handlers.push(spawn_storage_data_handler(
        storage_rx,
        DataTask {
            data_type: DataType::Storage,
            message_type: MessageType::StorageInfo,
            transports: transports.clone(),
            subscription_manager: Arc::clone(&subscription_manager),
            message_handler: message_handler.clone(),
            shutdown: shutdown_sender.subscribe(),
        },
    ));

    // Spawn System data handler
    handlers.push(spawn_system_data_handler(
        system_rx,
        DataTask {
            data_type: DataType::System,
            message_type: MessageType::SystemInfo,
            transports: transports.clone(),
            subscription_manager: Arc::clone(&subscription_manager),
            message_handler: message_handler.clone(),
            shutdown: shutdown_sender.subscribe(),
        },
    ));

    handlers
}