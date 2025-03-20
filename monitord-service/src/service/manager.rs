use crate::communication::CommunicationManager;
use crate::config::ServiceConfig;
use crate::error::ServiceError;
use futures::{channel::mpsc, SinkExt, Stream, StreamExt};
use monitord_collectors::{
    cpu::CpuCollector, gpu::GpuCollector, memory::MemoryCollector, network::NetworkCollector,
    process::ProcessCollector, storage::StorageCollector, system::SystemCollector,
    traits::Collector, CollectorConfig, CollectorError,
};
use monitord_protocols::monitord::*;
use std::time::Duration;
use tokio::task::JoinSet;
use tracing::{error, info, warn};

pub struct ServiceManager {
    communication_manager: CommunicationManager,
    config: ServiceConfig,
}

impl ServiceManager {
    pub fn init(config: ServiceConfig) -> Result<Self, ServiceError> {
        // Initialize communication manager
        let communication_manager =
            match CommunicationManager::new(config.communication_config.clone()) {
                Ok(manager) => manager,
                Err(e) => return Err(ServiceError::Communication(e)),
            };

        // We don't register with the platform automatically anymore
        // This is now done via the --register-service command-line flag

        Ok(Self {
            communication_manager,
            config,
        })
    }

    pub async fn run(self) -> Result<(), ServiceError> {
        info!("Starting monitord service");

        // Create channels for all collector data
        let (cpu_tx, cpu_rx) = mpsc::channel(16);
        let (memory_tx, memory_rx) = mpsc::channel(16);
        let (gpu_tx, gpu_rx) = mpsc::channel(16);
        let (network_tx, network_rx) = mpsc::channel(16);
        let (process_tx, process_rx) = mpsc::channel(16);
        let (storage_tx, storage_rx) = mpsc::channel(16);
        let (system_tx, system_rx) = mpsc::channel(16);

        // Create a join set for all collector tasks
        let mut collector_tasks = JoinSet::new();

        // Initialize collectors using the new API
        self.init_cpu_collector(&mut collector_tasks, cpu_tx)?;
        self.init_memory_collector(&mut collector_tasks, memory_tx)?;
        self.init_gpu_collector(&mut collector_tasks, gpu_tx)?;
        self.init_network_collector(&mut collector_tasks, network_tx)?;
        self.init_process_collector(&mut collector_tasks, process_tx)?;
        self.init_storage_collector(&mut collector_tasks, storage_tx)?;
        self.init_system_collector(&mut collector_tasks, system_tx)?;

        // Start the communication manager to publish data
        let comm_handle = tokio::spawn(async move {
            match self
                .communication_manager
                .run(
                    cpu_rx, memory_rx, gpu_rx, network_rx, process_rx, storage_rx, system_rx,
                )
                .await
            {
                Ok(_) => Ok(()),
                Err(e) => Err(ServiceError::Communication(e)),
            }
        });

        // Wait for any collector task to complete (usually means an error occurred)
        tokio::select! {
            Some(result) = collector_tasks.join_next() => {
                match result {
                    Ok(Ok(())) => {
                        warn!("A collector task completed unexpectedly but without error");
                        Ok(())
                    },
                    Ok(Err(e)) => {
                        error!("A collector task failed: {}", e);
                        Err(e)
                    },
                    Err(e) => {
                        error!("Failed to join collector task: {}", e);
                        Err(ServiceError::Collection(CollectorError::CollectionError(e.to_string())))
                    }
                }
            },
            result = comm_handle => {
                match result {
                    Ok(Ok(())) => {
                        info!("Communication manager completed successfully");
                        Ok(())
                    },
                    Ok(Err(e)) => {
                        error!("Communication manager failed: {}", e);
                        Err(e)
                    },
                    Err(e) => {
                        error!("Failed to join communication task: {}", e);
                        Err(ServiceError::Communication(e.to_string().into()))
                    }
                }
            }
        }
    }

    // Initialize CPU collector
    fn init_cpu_collector(
        &self,
        tasks: &mut JoinSet<Result<(), ServiceError>>,
        mut sender: mpsc::Sender<CpuInfo>,
    ) -> Result<(), ServiceError> {
        // Create CPU collector with config
        let cpu_config = self.config.collection_config.cpu.clone();
        if !cpu_config.is_enabled() {
            info!("CPU collector is disabled");
            return Ok(());
        }

        match CpuCollector::new(cpu_config.clone()) {
            Ok(collector) => {
                info!("CPU collector initialized");

                // Create a stream with the configured interval
                let interval = Duration::from_millis(cpu_config.interval_ms);
                let stream = collector.stream(interval);

                // Spawn a task to process the stream
                tasks.spawn(async move {
                    Self::process_stream("CPU", stream, &mut sender)
                        .await
                        .map_err(ServiceError::Collection)
                });

                Ok(())
            }
            Err(e) => {
                error!("Failed to initialize CPU collector: {}", e);
                Err(ServiceError::Collection(e))
            }
        }
    }

    // Initialize Memory collector
    fn init_memory_collector(
        &self,
        tasks: &mut JoinSet<Result<(), ServiceError>>,
        mut sender: mpsc::Sender<MemoryInfo>,
    ) -> Result<(), ServiceError> {
        // Create Memory collector with config
        let memory_config = self.config.collection_config.memory.clone();
        if !memory_config.is_enabled() {
            info!("Memory collector is disabled");
            return Ok(());
        }

        match MemoryCollector::new(memory_config.clone()) {
            Ok(collector) => {
                info!("Memory collector initialized");

                // Create a stream with the configured interval
                let interval = Duration::from_millis(memory_config.interval_ms);
                let stream = collector.stream(interval);

                // Spawn a task to process the stream
                tasks.spawn(async move {
                    Self::process_stream("Memory", stream, &mut sender)
                        .await
                        .map_err(ServiceError::Collection)
                });

                Ok(())
            }
            Err(e) => {
                error!("Failed to initialize Memory collector: {}", e);
                Err(ServiceError::Collection(e))
            }
        }
    }

    // Initialize GPU collector
    fn init_gpu_collector(
        &self,
        tasks: &mut JoinSet<Result<(), ServiceError>>,
        mut sender: mpsc::Sender<Vec<GpuInfo>>,
    ) -> Result<(), ServiceError> {
        // Create GPU collector with config
        let gpu_config = self.config.collection_config.gpu.clone();
        if !gpu_config.is_enabled() {
            info!("GPU collector is disabled");
            return Ok(());
        }

        // GPU collector may fail if no GPUs are available, which is not a fatal error
        let collector = match GpuCollector::new(gpu_config.clone()) {
            Ok(collector) => {
                info!("GPU collector initialized");
                collector
            }
            Err(e) => {
                warn!(
                    "Failed to initialize GPU collector: {}. GPU metrics will not be available.",
                    e
                );
                return Ok(());
            }
        };

        // Create a stream with the configured interval
        let interval = Duration::from_millis(gpu_config.interval_ms);
        let stream = collector.stream(interval);

        // Spawn a task to process the stream
        tasks.spawn(async move {
            let result = Self::process_gpu_stream("GPU", stream, &mut sender).await;
            if let Err(ref e) = result {
                warn!("GPU collection encountered an error: {}", e);
            }
            result.map_err(ServiceError::Collection)
        });

        Ok(())
    }

    // Initialize Network collector
    fn init_network_collector(
        &self,
        tasks: &mut JoinSet<Result<(), ServiceError>>,
        mut sender: mpsc::Sender<Vec<NetworkInfo>>,
    ) -> Result<(), ServiceError> {
        // Create Network collector with config
        let network_config = self.config.collection_config.network.clone();
        if !network_config.is_enabled() {
            info!("Network collector is disabled");
            return Ok(());
        }

        match NetworkCollector::new(network_config.clone()) {
            Ok(collector) => {
                info!("Network collector initialized");

                // Create a stream with the configured interval
                let interval = Duration::from_millis(network_config.interval_ms);
                let stream = collector.stream(interval);

                // Spawn a task to process the stream
                tasks.spawn(async move {
                    let result = Self::process_network_stream("Network", stream, &mut sender).await;
                    result.map_err(ServiceError::Collection)
                });

                Ok(())
            }
            Err(e) => {
                error!("Failed to initialize Network collector: {}", e);
                Err(ServiceError::Collection(e))
            }
        }
    }

    // Initialize Process collector
    fn init_process_collector(
        &self,
        tasks: &mut JoinSet<Result<(), ServiceError>>,
        mut sender: mpsc::Sender<Vec<ProcessInfo>>,
    ) -> Result<(), ServiceError> {
        // Create Process collector with config
        let process_config = self.config.collection_config.process.clone();
        if !process_config.is_enabled() {
            info!("Process collector is disabled");
            return Ok(());
        }

        match ProcessCollector::new(process_config.clone()) {
            Ok(collector) => {
                info!("Process collector initialized");

                // Create a stream with the configured interval
                let interval = Duration::from_millis(process_config.interval_ms);
                let stream = collector.stream(interval);

                // Spawn a task to process the stream
                tasks.spawn(async move {
                    let result = Self::process_process_stream("Process", stream, &mut sender).await;
                    result.map_err(ServiceError::Collection)
                });

                Ok(())
            }
            Err(e) => {
                error!("Failed to initialize Process collector: {}", e);
                Err(ServiceError::Collection(e))
            }
        }
    }

    // Initialize Storage collector
    fn init_storage_collector(
        &self,
        tasks: &mut JoinSet<Result<(), ServiceError>>,
        mut sender: mpsc::Sender<Vec<StorageInfo>>,
    ) -> Result<(), ServiceError> {
        // Create Storage collector with config
        let storage_config = self.config.collection_config.storage.clone();
        if !storage_config.is_enabled() {
            info!("Storage collector is disabled");
            return Ok(());
        }

        match StorageCollector::new(storage_config.clone()) {
            Ok(collector) => {
                info!("Storage collector initialized");

                // Create a stream with the configured interval
                let interval = Duration::from_millis(storage_config.interval_ms);
                let stream = collector.stream(interval);

                // Spawn a task to process the stream
                tasks.spawn(async move {
                    let result = Self::process_storage_stream("Storage", stream, &mut sender).await;
                    result.map_err(ServiceError::Collection)
                });

                Ok(())
            }
            Err(e) => {
                error!("Failed to initialize Storage collector: {}", e);
                Err(ServiceError::Collection(e))
            }
        }
    }

    // Initialize System collector
    fn init_system_collector(
        &self,
        tasks: &mut JoinSet<Result<(), ServiceError>>,
        mut sender: mpsc::Sender<SystemInfo>,
    ) -> Result<(), ServiceError> {
        // Create System collector with config
        let system_config = self.config.collection_config.system.clone();
        if !system_config.is_enabled() {
            info!("System collector is disabled");
            return Ok(());
        }

        match SystemCollector::new(system_config.clone()) {
            Ok(collector) => {
                info!("System collector initialized");

                // Create a stream with the configured interval
                let interval = Duration::from_millis(system_config.interval_ms);
                let stream = collector.stream(interval);

                // Spawn a task to process the stream
                tasks.spawn(async move {
                    Self::process_stream("System", stream, &mut sender)
                        .await
                        .map_err(ServiceError::Collection)
                });

                Ok(())
            }
            Err(e) => {
                error!("Failed to initialize System collector: {}", e);
                Err(ServiceError::Collection(e))
            }
        }
    }

    // Generic function to process a collector stream and send the results
    async fn process_stream<T, E>(
        name: &str,
        mut stream: impl Stream<Item = Result<T, E>> + Unpin,
        sender: &mut mpsc::Sender<T>,
    ) -> Result<(), E>
    where
        E: std::error::Error,
    {
        info!("{} collector stream started", name);

        while let Some(result) = stream.next().await {
            match result {
                Ok(data) => {
                    if sender.send(data).await.is_err() {
                        error!("{} collector channel closed, exiting", name);
                        break;
                    }
                }
                Err(e) => {
                    error!("{} collector error: {}", name, e);
                    return Err(e);
                }
            }
        }

        info!("{} collector stream completed", name);
        Ok(())
    }

    // Process GPU collector stream - specialized to extract the GpuInfo vec
    async fn process_gpu_stream<E>(
        name: &str,
        mut stream: impl Stream<Item = Result<GpuList, E>> + Unpin,
        sender: &mut mpsc::Sender<Vec<GpuInfo>>,
    ) -> Result<(), E>
    where
        E: std::error::Error,
    {
        info!("{} collector stream started", name);

        while let Some(result) = stream.next().await {
            match result {
                Ok(data) => {
                    if sender.send(data.gpus).await.is_err() {
                        error!("{} collector channel closed, exiting", name);
                        break;
                    }
                }
                Err(e) => {
                    error!("{} collector error: {}", name, e);
                    return Err(e);
                }
            }
        }

        info!("{} collector stream completed", name);
        Ok(())
    }

    // Process Network collector stream
    async fn process_network_stream<E>(
        name: &str,
        mut stream: impl Stream<Item = Result<NetworkList, E>> + Unpin,
        sender: &mut mpsc::Sender<Vec<NetworkInfo>>,
    ) -> Result<(), E>
    where
        E: std::error::Error,
    {
        info!("{} collector stream started", name);

        while let Some(result) = stream.next().await {
            match result {
                Ok(data) => {
                    if sender.send(data.nets).await.is_err() {
                        error!("{} collector channel closed, exiting", name);
                        break;
                    }
                }
                Err(e) => {
                    error!("{} collector error: {}", name, e);
                    return Err(e);
                }
            }
        }

        info!("{} collector stream completed", name);
        Ok(())
    }

    // Process Storage collector stream
    async fn process_storage_stream<E>(
        name: &str,
        mut stream: impl Stream<Item = Result<StorageList, E>> + Unpin,
        sender: &mut mpsc::Sender<Vec<StorageInfo>>,
    ) -> Result<(), E>
    where
        E: std::error::Error,
    {
        info!("{} collector stream started", name);

        while let Some(result) = stream.next().await {
            match result {
                Ok(data) => {
                    if sender.send(data.storages).await.is_err() {
                        error!("{} collector channel closed, exiting", name);
                        break;
                    }
                }
                Err(e) => {
                    error!("{} collector error: {}", name, e);
                    return Err(e);
                }
            }
        }

        info!("{} collector stream completed", name);
        Ok(())
    }

    // Process Process collector stream
    async fn process_process_stream<E>(
        name: &str,
        mut stream: impl Stream<Item = Result<ProcessList, E>> + Unpin,
        sender: &mut mpsc::Sender<Vec<ProcessInfo>>,
    ) -> Result<(), E>
    where
        E: std::error::Error,
    {
        info!("{} collector stream started", name);

        while let Some(result) = stream.next().await {
            match result {
                Ok(data) => {
                    if sender.send(data.processes).await.is_err() {
                        error!("{} collector channel closed, exiting", name);
                        break;
                    }
                }
                Err(e) => {
                    error!("{} collector error: {}", name, e);
                    return Err(e);
                }
            }
        }

        info!("{} collector stream completed", name);
        Ok(())
    }
}
