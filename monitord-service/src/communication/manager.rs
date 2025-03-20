use crate::config::CommunicationConfig;
use crate::error::CommunicationError;
use futures::channel::mpsc::Receiver;
use futures::StreamExt;
use monitord_protocols::monitord::*;
use monitord_transport::TransportManager;
use tokio::task::JoinSet;
use tracing::{error, info, warn};

pub struct CommunicationManager {
    transport: TransportManager,
}

impl CommunicationManager {
    pub fn new(config: CommunicationConfig) -> Result<Self, CommunicationError> {
        let transport = TransportManager::new(config.transport_config)
            .map_err(CommunicationError::Transport)?;

        Ok(Self { transport })
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

        info!("Initializing transport manager");
        let mut transport = self.transport.clone();
        transport.initialize().await?;
        info!("Transport manager initialized");

        // CPU task
        {
            let mut transport_clone = transport.clone();
            tasks.spawn(async move {
                info!("Starting CPU data publisher");
                while let Some(cpu_info) = cpu_rx.next().await {
                    transport_clone.publish("cpu", cpu_info).await?;
                }
                Ok::<(), CommunicationError>(())
            });
        }

        // Memory task
        {
            let mut transport_clone = transport.clone();
            tasks.spawn(async move {
                info!("Starting Memory data publisher");
                while let Some(memory_info) = memory_rx.next().await {
                    transport_clone.publish("memory", memory_info).await?;
                }
                Ok::<(), CommunicationError>(())
            });
        }

        // GPU task
        {
            let mut transport_clone = transport.clone();
            tasks.spawn(async move {
                info!("Starting GPU data publisher");
                while let Some(gpu_info) = gpu_rx.next().await {
                    let gpu_info = GpuList { gpus: gpu_info };
                    transport_clone.publish("gpu", gpu_info).await?;
                }
                Ok::<(), CommunicationError>(())
            });
        }

        // Network task
        {
            let mut transport_clone = transport.clone();
            tasks.spawn(async move {
                info!("Starting Network data publisher");
                while let Some(net_info) = net_rx.next().await {
                    let net_info = NetworkList { nets: net_info };
                    transport_clone.publish("network", net_info).await?;
                }
                Ok::<(), CommunicationError>(())
            });
        }

        // Process task
        {
            let mut transport_clone = transport.clone();
            tasks.spawn(async move {
                info!("Starting Process data publisher");
                while let Some(proc_info) = proc_rx.next().await {
                    let proc_info = ProcessList {
                        processes: proc_info,
                    };
                    transport_clone.publish("process", proc_info).await?;
                }
                Ok::<(), CommunicationError>(())
            });
        }

        // Storage task
        {
            let mut transport_clone = transport.clone();
            tasks.spawn(async move {
                info!("Starting Storage data publisher");
                while let Some(storage_info) = storage_rx.next().await {
                    let storage_info = StorageList {
                        storages: storage_info,
                    };
                    transport_clone.publish("storage", storage_info).await?;
                }
                Ok::<(), CommunicationError>(())
            });
        }

        // System task
        {
            let mut transport_clone = transport.clone();
            tasks.spawn(async move {
                info!("Starting System data publisher");
                while let Some(system_info) = system_rx.next().await {
                    transport_clone.publish("system", system_info).await?;
                }
                Ok::<(), CommunicationError>(())
            });
        }

        // Snapshot task - combines data from all collectors into a single snapshot
        // This would be more efficiently implemented with shared state, but for simplicity
        // we'll leave this for a future enhancement

        // Wait for any task to complete and return its result
        if let Some(result) = tasks.join_next().await {
            // If any task completes, it's because of an error
            match result {
                Ok(Ok(())) => {
                    warn!("A publisher task completed unexpectedly but without error");
                    Ok(())
                }
                Ok(Err(e)) => {
                    error!("A publisher task failed: {}", e);
                    Err(e)
                }
                Err(e) => {
                    error!("Failed to join publisher task: {}", e);
                    Err(CommunicationError::TaskJoin(e.to_string()))
                }
            }
        } else {
            // All tasks completed successfully
            info!("All publisher tasks completed successfully");
            Ok(())
        }
    }
}
