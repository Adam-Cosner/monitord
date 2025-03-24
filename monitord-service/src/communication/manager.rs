use crate::config::CommunicationConfig;
use crate::error::CommunicationError;
use futures::channel::mpsc::Receiver;
use monitord_protocols::monitord::monitord_service_server::{
    MonitordService, MonitordServiceServer,
};
use monitord_protocols::monitord::*;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc::{self as tokio_mpsc};
use tokio::sync::RwLock;
use tokio::task::JoinSet;
use tokio_stream::Stream;
use tonic::{transport::Server, Response};
use tracing::{error, info, warn};

// Shared state for the gRPC service
#[derive(Debug, Default)]
struct SharedState {
    cpu_data: Option<CpuInfo>,
    memory_data: Option<MemoryInfo>,
    gpu_data: Option<GpuList>,
    network_data: Option<NetworkList>,
    process_data: Option<ProcessList>,
    storage_data: Option<StorageList>,
    system_data: Option<SystemInfo>,
}

// Our gRPC service implementation
#[derive(Debug)]
pub struct MonitordServiceImpl {
    state: Arc<RwLock<SharedState>>,
}

#[tonic::async_trait]
impl MonitordService for MonitordServiceImpl {
    async fn get_system_snapshot(
        &self,
        _request: tonic::Request<SnapshotRequest>,
    ) -> Result<tonic::Response<SystemSnapshot>, tonic::Status> {
        let state = self.state.read().await;

        // Create a snapshot from our current state
        let snapshot = SystemSnapshot {
            timestamp: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
            system_info: state.system_data.clone(),
            cpu_info: state.cpu_data.clone(),
            memory_info: state.memory_data.clone(),
            gpu_info: state.gpu_data.clone(),
            network_info: state.network_data.clone(),
            processes: state.process_data.clone(),
            storage_devices: state.storage_data.clone(),
        };

        Ok(Response::new(snapshot))
    }

    type StreamSystemSnapshotsStream =
        Pin<Box<dyn Stream<Item = Result<SystemSnapshot, tonic::Status>> + Send + 'static>>;

    async fn stream_system_snapshots(
        &self,
        request: tonic::Request<SnapshotRequest>,
    ) -> Result<tonic::Response<Self::StreamSystemSnapshotsStream>, tonic::Status> {
        let interval_ms = request.into_inner().interval_ms;
        let state_clone = self.state.clone();

        // Create a channel for our stream
        let (tx, rx) = tokio_mpsc::channel(128);

        // Spawn a task to send snapshots at the requested interval
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_millis(interval_ms as u64));

            loop {
                interval.tick().await;
                let state = state_clone.read().await;

                // Create a snapshot from our current state
                let snapshot = SystemSnapshot {
                    timestamp: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
                    system_info: state.system_data.clone(),
                    cpu_info: state.cpu_data.clone(),
                    memory_info: state.memory_data.clone(),
                    gpu_info: state.gpu_data.clone(),
                    network_info: state.network_data.clone(),
                    processes: state.process_data.clone(),
                    storage_devices: state.storage_data.clone(),
                };

                if tx.send(Ok(snapshot)).await.is_err() {
                    // Client disconnected
                    break;
                }
            }
        });

        // Convert our channel into a stream
        let output_stream =
            futures::StreamExt::boxed(tokio_stream::wrappers::ReceiverStream::new(rx));

        Ok(Response::new(output_stream))
    }

    type StreamCpuInfoStream =
        Pin<Box<dyn Stream<Item = Result<CpuInfo, tonic::Status>> + Send + 'static>>;

    async fn stream_cpu_info(
        &self,
        request: tonic::Request<SnapshotRequest>,
    ) -> Result<tonic::Response<Self::StreamCpuInfoStream>, tonic::Status> {
        let interval_ms = request.into_inner().interval_ms;
        let state_clone = self.state.clone();

        let (tx, rx) = tokio_mpsc::channel(128);

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_millis(interval_ms as u64));

            loop {
                interval.tick().await;
                let state = state_clone.read().await;

                if let Some(cpu_info) = &state.cpu_data {
                    if tx.send(Ok(cpu_info.clone())).await.is_err() {
                        break;
                    }
                }
            }
        });

        let output_stream =
            futures::StreamExt::boxed(tokio_stream::wrappers::ReceiverStream::new(rx));

        Ok(Response::new(output_stream))
    }

    type StreamMemoryInfoStream =
        Pin<Box<dyn Stream<Item = Result<MemoryInfo, tonic::Status>> + Send + 'static>>;

    async fn stream_memory_info(
        &self,
        request: tonic::Request<SnapshotRequest>,
    ) -> Result<tonic::Response<Self::StreamMemoryInfoStream>, tonic::Status> {
        let interval_ms = request.into_inner().interval_ms;
        let state_clone = self.state.clone();

        let (tx, rx) = tokio_mpsc::channel(128);

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_millis(interval_ms as u64));

            loop {
                interval.tick().await;
                let state = state_clone.read().await;

                if let Some(memory_info) = &state.memory_data {
                    if tx.send(Ok(memory_info.clone())).await.is_err() {
                        break;
                    }
                }
            }
        });

        let output_stream =
            futures::StreamExt::boxed(tokio_stream::wrappers::ReceiverStream::new(rx));

        Ok(Response::new(output_stream))
    }

    type StreamGpuInfoStream =
        Pin<Box<dyn Stream<Item = Result<GpuList, tonic::Status>> + Send + 'static>>;

    async fn stream_gpu_info(
        &self,
        request: tonic::Request<SnapshotRequest>,
    ) -> Result<tonic::Response<Self::StreamGpuInfoStream>, tonic::Status> {
        let interval_ms = request.into_inner().interval_ms;
        let state_clone = self.state.clone();

        let (tx, rx) = tokio_mpsc::channel(128);

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_millis(interval_ms as u64));

            loop {
                interval.tick().await;
                let state = state_clone.read().await;
                if let Some(gpu_list) = &state.gpu_data {
                    if tx.send(Ok(gpu_list.clone())).await.is_err() {
                        return;
                    }
                }
            }
        });

        let output_stream =
            futures::StreamExt::boxed(tokio_stream::wrappers::ReceiverStream::new(rx));

        Ok(Response::new(output_stream))
    }

    type StreamNetworkInfoStream =
        Pin<Box<dyn Stream<Item = Result<NetworkList, tonic::Status>> + Send + 'static>>;

    async fn stream_network_info(
        &self,
        request: tonic::Request<SnapshotRequest>,
    ) -> Result<tonic::Response<Self::StreamNetworkInfoStream>, tonic::Status> {
        let interval_ms = request.into_inner().interval_ms;
        let state_clone = self.state.clone();

        let (tx, rx) = tokio_mpsc::channel(128);

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_millis(interval_ms as u64));

            loop {
                interval.tick().await;
                let state = state_clone.read().await;

                if let Some(network_list) = &state.network_data {
                    if tx.send(Ok(network_list.clone())).await.is_err() {
                        return;
                    }
                }
            }
        });

        let output_stream =
            futures::StreamExt::boxed(tokio_stream::wrappers::ReceiverStream::new(rx));

        Ok(Response::new(output_stream))
    }

    type StreamStorageInfoStream =
        Pin<Box<dyn Stream<Item = Result<StorageList, tonic::Status>> + Send + 'static>>;

    async fn stream_storage_info(
        &self,
        request: tonic::Request<SnapshotRequest>,
    ) -> Result<tonic::Response<Self::StreamStorageInfoStream>, tonic::Status> {
        let interval_ms = request.into_inner().interval_ms;
        let state_clone = self.state.clone();

        let (tx, rx) = tokio_mpsc::channel(128);

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_millis(interval_ms as u64));

            loop {
                interval.tick().await;
                let state = state_clone.read().await;

                if let Some(storage_list) = &state.storage_data {
                    if tx.send(Ok(storage_list.clone())).await.is_err() {
                        return;
                    }
                }
            }
        });

        let output_stream =
            futures::StreamExt::boxed(tokio_stream::wrappers::ReceiverStream::new(rx));

        Ok(Response::new(output_stream))
    }

    type StreamProcessInfoStream =
        Pin<Box<dyn Stream<Item = Result<ProcessList, tonic::Status>> + Send + 'static>>;

    async fn stream_process_info(
        &self,
        request: tonic::Request<ProcessInfoRequest>,
    ) -> Result<tonic::Response<Self::StreamProcessInfoStream>, tonic::Status> {
        let req = request.into_inner();
        let interval_ms = req.interval_ms;
        let username_filter = req.username_filter;
        let pid_filter = req.pid_filter;
        let name_filter = req.name_filter;
        let sort_by_cpu = req.sort_by_cpu;
        let sort_by_memory = req.sort_by_memory;
        let limit = req.limit;

        let state_clone = self.state.clone();

        let (tx, rx) = tokio_mpsc::channel(128);

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_millis(interval_ms as u64));

            loop {
                interval.tick().await;
                let state = state_clone.read().await;

                if let Some(process_list) = &state.process_data {
                    // Apply filters
                    let mut filtered: Vec<ProcessInfo> = process_list
                        .processes
                        .iter()
                        .filter(|p| {
                            let username_match = username_filter
                                .as_ref()
                                .map(|u| p.username.contains(u))
                                .unwrap_or(true);

                            let pid_match = pid_filter.map(|pid| p.pid == pid).unwrap_or(true);

                            let name_match = name_filter
                                .as_ref()
                                .map(|n| p.name.contains(n))
                                .unwrap_or(true);

                            username_match && pid_match && name_match
                        })
                        .cloned()
                        .collect();

                    // Apply sorting
                    if sort_by_cpu {
                        filtered.sort_by(|a, b| {
                            b.cpu_usage_percent
                                .partial_cmp(&a.cpu_usage_percent)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        });
                    } else if sort_by_memory {
                        filtered
                            .sort_by(|a, b| b.physical_memory_bytes.cmp(&a.physical_memory_bytes));
                    }

                    // Apply limit
                    if limit > 0 && filtered.len() > limit as usize {
                        filtered.truncate(limit as usize);
                    }

                    // Send filtered processes
                    if tx
                        .send(Ok(ProcessList {
                            processes: filtered,
                        }))
                        .await
                        .is_err()
                    {
                        return;
                    }
                }
            }
        });

        let output_stream =
            futures::StreamExt::boxed(tokio_stream::wrappers::ReceiverStream::new(rx));

        Ok(Response::new(output_stream))
    }
}

// Our communication manager that will run the gRPC server and update the shared state
pub struct CommunicationManager {
    config: CommunicationConfig,
    state: Arc<RwLock<SharedState>>,
}

impl CommunicationManager {
    pub fn new(config: CommunicationConfig) -> Result<Self, CommunicationError> {
        Ok(Self {
            config,
            state: Arc::new(RwLock::new(SharedState::default())),
        })
    }

    pub async fn run(
        &self,
        mut cpu_rx: Receiver<CpuInfo>,
        mut memory_rx: Receiver<MemoryInfo>,
        mut gpu_rx: Receiver<Vec<GpuInfo>>,
        mut net_rx: Receiver<Vec<NetworkInfo>>,
        mut proc_rx: Receiver<Vec<ProcessInfo>>,
        mut storage_rx: Receiver<Vec<StorageInfo>>,
        mut system_rx: Receiver<SystemInfo>,
    ) -> Result<(), CommunicationError> {
        let mut tasks = JoinSet::new();
        let state = self.state.clone();

        // Start the gRPC server
        let server_addr = self
            .config
            .grpc_config
            .server_address
            .parse()
            .map_err(|e| CommunicationError::ServerStartup(format!("Invalid address: {}", e)))?;

        let service = MonitordServiceImpl {
            state: state.clone(),
        };

        // Spawn the gRPC server task
        let server_future = Server::builder()
            .add_service(MonitordServiceServer::new(service))
            .serve(server_addr);

        tasks.spawn(async move {
            info!("Starting gRPC server on {}", server_addr);
            if let Err(e) = server_future.await {
                error!("gRPC server error: {}", e);
                return Err(CommunicationError::ServerStartup(e.to_string()));
            }
            Ok(())
        });

        // Spawn tasks to update the shared state from collector channels

        // CPU task
        {
            let state_clone = state.clone();
            tasks.spawn(async move {
                info!("Starting CPU data collector");
                while let Some(cpu_info) = futures::StreamExt::next(&mut cpu_rx).await {
                    let mut state = state_clone.write().await;
                    state.cpu_data = Some(cpu_info);
                }
                Ok::<(), CommunicationError>(())
            });
        }

        // Memory task
        {
            let state_clone = state.clone();
            tasks.spawn(async move {
                info!("Starting Memory data collector");
                while let Some(memory_info) = futures::StreamExt::next(&mut memory_rx).await {
                    let mut state = state_clone.write().await;
                    state.memory_data = Some(memory_info);
                }
                Ok::<(), CommunicationError>(())
            });
        }

        // GPU task
        {
            let state_clone = state.clone();
            tasks.spawn(async move {
                info!("Starting GPU data collector");
                while let Some(gpu_info) = futures::StreamExt::next(&mut gpu_rx).await {
                    let mut state = state_clone.write().await;
                    state.gpu_data = Some(GpuList {
                        gpus: gpu_info.clone(),
                    });

                    // Iterate over gpu processes
                    for gpu in gpu_info.iter() {
                        for gpu_process in gpu.process_info.iter() {
                            if let Some(ref mut process_data) = state.process_data {
                                if let Some(process) = process_data
                                    .processes
                                    .iter_mut()
                                    .find(|proc| proc.pid == gpu_process.pid)
                                {
                                    process.gpu_usage = Some(gpu_process.clone())
                                }
                            }
                        }
                    }
                }
                Ok::<(), CommunicationError>(())
            });
        }

        // Network task
        {
            let state_clone = state.clone();
            tasks.spawn(async move {
                info!("Starting Network data collector");
                while let Some(net_info) = futures::StreamExt::next(&mut net_rx).await {
                    let mut state = state_clone.write().await;
                    state.network_data = Some(NetworkList { nets: net_info });
                }
                Ok::<(), CommunicationError>(())
            });
        }

        // Process task
        {
            let state_clone = state.clone();
            tasks.spawn(async move {
                info!("Starting Process data collector");
                while let Some(proc_info) = futures::StreamExt::next(&mut proc_rx).await {
                    let mut state = state_clone.write().await;
                    state.process_data = Some(ProcessList {
                        processes: proc_info,
                    });
                }
                Ok::<(), CommunicationError>(())
            });
        }

        // Storage task
        {
            let state_clone = state.clone();
            tasks.spawn(async move {
                info!("Starting Storage data collector");
                while let Some(storage_info) = futures::StreamExt::next(&mut storage_rx).await {
                    let mut state = state_clone.write().await;
                    state.storage_data = Some(StorageList {
                        storages: storage_info,
                    });
                }
                Ok::<(), CommunicationError>(())
            });
        }

        // System task
        {
            let state_clone = state.clone();
            tasks.spawn(async move {
                info!("Starting System data collector");
                while let Some(system_info) = futures::StreamExt::next(&mut system_rx).await {
                    let mut state = state_clone.write().await;
                    state.system_data = Some(system_info);
                }
                Ok::<(), CommunicationError>(())
            });
        }

        // Wait for any task to complete and return its result
        if let Some(result) = tasks.join_next().await {
            // If any task completes, it's because of an error
            match result {
                Ok(Ok(())) => {
                    warn!("A task completed unexpectedly but without error");
                    Ok(())
                }
                Ok(Err(e)) => {
                    error!("A task failed: {}", e);
                    Err(e)
                }
                Err(e) => {
                    error!("Failed to join task: {}", e);
                    Err(CommunicationError::TaskJoin(e.to_string()))
                }
            }
        } else {
            // All tasks completed successfully
            info!("All tasks completed successfully");
            Ok(())
        }
    }
}
