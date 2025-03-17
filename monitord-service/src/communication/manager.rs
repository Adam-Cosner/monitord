use crate::config::CommunicationConfig;
use crate::error::CommunicationError;
use monitord_protocols::monitord::*;
use monitord_transport::TransportManager;
use tokio::sync::broadcast::Receiver;

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
        let mut tasks = tokio::task::JoinSet::new();

        let mut transport = self.transport.clone();
        transport.initialize().await?;

        // CPU task
        {
            let mut transport_clone = transport.clone();
            tasks.spawn(async move {
                while let Ok(cpu_info) = cpu_rx.recv().await {
                    transport_clone.publish("monitord/cpu", cpu_info).await?;
                }
                Ok::<(), CommunicationError>(())
            });
        }
        
        // Memory task
        {
            let mut transport_clone = transport.clone();
            tasks.spawn(async move {
                while let Ok(memory_info) = memory_rx.recv().await {
                    transport_clone.publish("monitord/memory", memory_info).await?;
                }
                Ok::<(), CommunicationError>(())
            });
        }
        
        // GPU task
        {
            let mut transport_clone = transport.clone();
            tasks.spawn(async move {
                while let Ok(gpu_info) = gpu_rx.recv().await {
                    let gpu_info = GpuList { gpus: prost::alloc::vec::Vec::from(gpu_info) };
                    transport_clone.publish("monitord/gpu", gpu_info).await?;
                }
                Ok::<(), CommunicationError>(())
            });
        }
        
        // Network task
        {
            let mut transport_clone = transport.clone();
            tasks.spawn(async move {
                while let Ok(net_info) = net_rx.recv().await {
                    let net_info = NetworkList { nets: prost::alloc::vec::Vec::from(net_info) };
                    transport_clone.publish("monitord/network", net_info).await?;
                }
                Ok::<(), CommunicationError>(())
            });
        }
        
        // Process task
        {
            let mut transport_clone = transport.clone();
            tasks.spawn(async move {
                while let Ok(proc_info) = proc_rx.recv().await {
                    let proc_info = ProcessList { processes: prost::alloc::vec::Vec::from(proc_info) };
                    transport_clone.publish("monitord/process", proc_info).await?;
                }
                Ok::<(), CommunicationError>(())
            });
        }
        
        // Storage task
        {
            let mut transport_clone = transport.clone();
            tasks.spawn(async move {
                while let Ok(storage_info) = storage_rx.recv().await {
                    let storage_info = StorageList { storages: prost::alloc::vec::Vec::from(storage_info) };
                    transport_clone.publish("monitord/storage", storage_info).await?;
                }
                Ok::<(), CommunicationError>(())
            });
        }
        
        // System task
        {
            let mut transport_clone = transport.clone();
            tasks.spawn(async move {
                while let Ok(system_info) = system_rx.recv().await {
                    transport_clone.publish("monitord/system", system_info).await?;
                }
                Ok::<(), CommunicationError>(())
            });
        }
        
        // Wait for any task to complete and return its result
        if let Some(result) = tasks.join_next().await {
            // If any task completes, it's because of an error
            result.unwrap_or_else(|join_err| Err(CommunicationError::TaskJoin(join_err.to_string())))
        } else {
            // All tasks completed successfully
            Ok(())
        }
    }
}
