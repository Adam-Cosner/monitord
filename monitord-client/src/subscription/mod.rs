//! Subscription management for monitord data streams

pub mod config;

use crate::error::ClientError;
use crate::Result;

/// Types of data that can be subscribed to
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubscriptionType {
    /// Complete system information
    System,

    /// CPU information only
    Cpu,

    /// Memory information only
    Memory,

    /// GPU information only
    Gpu,

    /// Network information only
    Network,

    /// Storage information only
    Storage,

    /// Process information only
    Process,
}

/// A subscription to monitord data
#[derive(Debug)]
pub struct Subscription {
    /// Unique identifier for this subscription
    pub id: String,

    /// The type of data this subscription provides
    pub subscription_type: SubscriptionType,

    /// How often updates are received (in milliseconds)
    pub interval_ms: u32,

    // Internal subscription state
    #[doc(hidden)]
    pub(crate) active: bool,
}

impl Subscription {
    /// Creates a new subscription builder
    pub fn builder() -> SubscriptionBuilder {
        SubscriptionBuilder::new()
    }

    /// Checks if the subscription is currently active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Returns the subscription type
    pub fn subscription_type(&self) -> SubscriptionType {
        self.subscription_type
    }

    /// Returns the update interval in milliseconds
    pub fn interval_ms(&self) -> u32 {
        self.interval_ms
    }
}

/// Builder for creating subscriptions
#[derive(Debug, Default, Clone)]
pub struct SubscriptionBuilder {
    subscription_type: Option<SubscriptionType>,
    interval_ms: Option<u32>,
    process_filter: Option<ProcessFilter>,
    gpu_filter: Option<GpuFilter>,
    network_filter: Option<NetworkFilter>,
    storage_filter: Option<StorageFilter>,
}

/// Filter for process subscriptions
#[derive(Debug, Default, Clone)]
pub struct ProcessFilter {
    pub pids: Vec<u32>,
    pub names: Vec<String>,
    pub usernames: Vec<String>,
    pub top_by_cpu: Option<u32>,
    pub top_by_memory: Option<u32>,
    pub top_by_disk: Option<u32>,
}

/// Filter for GPU subscriptions
#[derive(Debug, Default, Clone)]
pub struct GpuFilter {
    pub names: Vec<String>,
    pub vendors: Vec<String>,
    pub include_processes: bool,
}

/// Filter for network subscriptions
#[derive(Debug, Default, Clone)]
pub struct NetworkFilter {
    pub interface_names: Vec<String>,
}

/// Filter for storage subscriptions
#[derive(Debug, Default, Clone)]
pub struct StorageFilter {
    pub device_names: Vec<String>,
    pub mount_points: Vec<String>,
}

impl SubscriptionBuilder {
    /// Creates a new subscription builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the subscription type
    pub fn subscription_type(mut self, subscription_type: SubscriptionType) -> Self {
        self.subscription_type = Some(subscription_type);
        self
    }

    /// Sets the update interval in milliseconds
    pub fn interval_ms(mut self, interval_ms: u32) -> Self {
        self.interval_ms = Some(interval_ms);
        self
    }

    /// Sets a process filter for the subscription
    pub fn process_filter(mut self, filter: ProcessFilter) -> Self {
        self.process_filter = Some(filter);
        self
    }

    /// Sets a GPU filter for the subscription
    pub fn gpu_filter(mut self, filter: GpuFilter) -> Self {
        self.gpu_filter = Some(filter);
        self
    }

    /// Sets a network filter for the subscription
    pub fn network_filter(mut self, filter: NetworkFilter) -> Self {
        self.network_filter = Some(filter);
        self
    }

    /// Sets a storage filter for the subscription
    pub fn storage_filter(mut self, filter: StorageFilter) -> Self {
        self.storage_filter = Some(filter);
        self
    }

    /// Validates and builds the subscription
    pub fn validate(&self) -> Result<()> {
        if self.subscription_type.is_none() {
            return Err(ClientError::SubscriptionError(
                "Subscription type is required".to_string(),
            ));
        }

        // TODO: Implement additional subscription validation
        // - Check interval_ms is within valid range
        // - Validate filters are appropriate for the subscription type
        // - Check for any invalid combinations of filters

        Ok(())
    }
}
