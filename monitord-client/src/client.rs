use crate::{ClientError, ProcessFilter, Result};
use futures::Stream;
use monitord_protocols::monitord::{
    monitord_service_client::MonitordServiceClient, CpuInfo, GpuInfo, MemoryInfo, NetworkInfo,
    ProcessInfo, ProcessInfoRequest, SnapshotRequest, SystemSnapshot,
};
use tonic::transport::Channel;

/// Client for interacting with the monitord service
#[derive(Debug, Clone)]
pub struct MonitordClient {
    client: MonitordServiceClient<Channel>,
}

impl MonitordClient {
    /// Connect to a monitord service at the specified address.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address of the monitord service (e.g., "http://localhost:50051")
    ///
    /// # Returns
    ///
    /// A new `MonitordClient` or a connection error
    pub async fn connect(addr: impl AsRef<str>) -> Result<Self> {
        let client = MonitordServiceClient::connect(addr.as_ref().to_string()).await?;
        Ok(Self { client })
    }

    /// Get a single system snapshot
    pub async fn get_system_snapshot(&self) -> Result<SystemSnapshot> {
        let request = SnapshotRequest { interval_ms: 0 };
        let response = self.client.clone().get_system_snapshot(request).await?;
        Ok(response.into_inner())
    }

    /// Stream system snapshots at the specified interval
    ///
    /// # Arguments
    ///
    /// * `interval_ms` - The interval between snapshots in milliseconds
    ///
    /// # Returns
    ///
    /// A stream of system snapshots
    pub async fn stream_system_snapshots(
        &self,
        interval_ms: u32,
    ) -> Result<impl Stream<Item = Result<SystemSnapshot>>> {
        let request = SnapshotRequest { interval_ms };
        let stream = self
            .client
            .clone()
            .stream_system_snapshots(request)
            .await?
            .into_inner();

        Ok(Box::pin(futures::stream::unfold(stream, |mut stream| async move {
            match stream.message().await {
                Ok(Some(item)) => Some((Ok(item), stream)),
                Ok(None) => None,
                Err(e) => Some((Err(ClientError::from(e)), stream)),
            }
        })))
    }

    /// Stream CPU information at the specified interval
    ///
    /// # Arguments
    ///
    /// * `interval_ms` - The interval between updates in milliseconds
    ///
    /// # Returns
    ///
    /// A stream of CPU information
    pub async fn stream_cpu_info(
        &self,
        interval_ms: u32,
    ) -> Result<impl Stream<Item = Result<CpuInfo>>> {
        let request = SnapshotRequest { interval_ms };
        let stream = self
            .client
            .clone()
            .stream_cpu_info(request)
            .await?
            .into_inner();

        Ok(Box::pin(futures::stream::unfold(stream, |mut stream| async move {
            match stream.message().await {
                Ok(Some(item)) => Some((Ok(item), stream)),
                Ok(None) => None,
                Err(e) => Some((Err(ClientError::from(e)), stream)),
            }
        })))
    }

    /// Stream memory information at the specified interval
    ///
    /// # Arguments
    ///
    /// * `interval_ms` - The interval between updates in milliseconds
    ///
    /// # Returns
    ///
    /// A stream of memory information
    pub async fn stream_memory_info(
        &self,
        interval_ms: u32,
    ) -> Result<impl Stream<Item = Result<MemoryInfo>>> {
        let request = SnapshotRequest { interval_ms };
        let stream = self
            .client
            .clone()
            .stream_memory_info(request)
            .await?
            .into_inner();

        Ok(Box::pin(futures::stream::unfold(stream, |mut stream| async move {
            match stream.message().await {
                Ok(Some(item)) => Some((Ok(item), stream)),
                Ok(None) => None,
                Err(e) => Some((Err(ClientError::from(e)), stream)),
            }
        })))
    }

    /// Stream GPU information at the specified interval
    ///
    /// # Arguments
    ///
    /// * `interval_ms` - The interval between updates in milliseconds
    ///
    /// # Returns
    ///
    /// A stream of GPU information
    pub async fn stream_gpu_info(
        &self,
        interval_ms: u32,
    ) -> Result<impl Stream<Item = Result<GpuInfo>>> {
        let request = SnapshotRequest { interval_ms };
        let stream = self
            .client
            .clone()
            .stream_gpu_info(request)
            .await?
            .into_inner();

        Ok(Box::pin(futures::stream::unfold(stream, |mut stream| async move {
            match stream.message().await {
                Ok(Some(item)) => Some((Ok(item), stream)),
                Ok(None) => None,
                Err(e) => Some((Err(ClientError::from(e)), stream)),
            }
        })))
    }

    /// Stream network information at the specified interval
    ///
    /// # Arguments
    ///
    /// * `interval_ms` - The interval between updates in milliseconds
    ///
    /// # Returns
    ///
    /// A stream of network information
    pub async fn stream_network_info(
        &self,
        interval_ms: u32,
    ) -> Result<impl Stream<Item = Result<NetworkInfo>>> {
        let request = SnapshotRequest { interval_ms };
        let stream = self
            .client
            .clone()
            .stream_network_info(request)
            .await?
            .into_inner();

        Ok(Box::pin(futures::stream::unfold(stream, |mut stream| async move {
            match stream.message().await {
                Ok(Some(item)) => Some((Ok(item), stream)),
                Ok(None) => None,
                Err(e) => Some((Err(ClientError::from(e)), stream)),
            }
        })))
    }

    /// Stream process information with optional filtering
    ///
    /// # Arguments
    ///
    /// * `interval_ms` - The interval between updates in milliseconds
    /// * `filter` - Filter options for the process information
    ///
    /// # Returns
    ///
    /// A stream of process information
    pub async fn stream_process_info(
        &self,
        interval_ms: u32,
        filter: ProcessFilter,
    ) -> Result<impl Stream<Item = Result<ProcessInfo>>> {
        let request = ProcessInfoRequest {
            interval_ms,
            username_filter: filter.username_filter,
            pid_filter: filter.pid_filter,
            name_filter: filter.name_filter,
            sort_by_cpu: filter.sort_by_cpu,
            sort_by_memory: filter.sort_by_memory,
            limit: filter.limit,
        };

        let stream = self
            .client
            .clone()
            .stream_process_info(request)
            .await?
            .into_inner();

        Ok(Box::pin(futures::stream::unfold(stream, |mut stream| async move {
            match stream.message().await {
                Ok(Some(item)) => Some((Ok(item), stream)),
                Ok(None) => None,
                Err(e) => Some((Err(ClientError::from(e)), stream)),
            }
        })))
    }
}