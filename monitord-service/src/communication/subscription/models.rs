//! Data models for subscription management

use std::time::Instant;
use crate::communication::core::models::TransportType;
use crate::communication::core::models::DataType;
use monitord_protocols::subscription::{
    subscription_request::Filter,
    SubscriptionType,
};

/// Represents a client subscription
#[derive(Debug, Clone)]
pub struct Subscription {
    /// Unique ID for this subscription
    pub id: String,

    /// The type of data being subscribed to
    pub subscription_type: SubscriptionType,

    /// The interval in milliseconds to receive updates
    pub interval_ms: u32,

    /// The ID of the client that requested the subscription
    pub client_id: String,

    /// The transport strategy to use
    pub transport: TransportType,

    /// The time the subscription was created
    pub created_at: Instant,

    /// The last time the client received data
    pub last_received_at: Instant,

    /// The optional filter for this subscription
    pub filter: Option<Filter>,
}

impl Subscription {
    /// Create a new subscription
    pub fn new(
        subscription_type: SubscriptionType,
        client_id: String,
        interval_ms: u32,
        transport: TransportType,
        filter: Option<Filter>,
    ) -> Self {
        let now = Instant::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            subscription_type,
            interval_ms,
            client_id,
            transport,
            created_at: now,
            last_received_at: now,
            filter,
        }
    }

    /// Check if it's time to update this subscription
    pub fn should_update(&self) -> bool {
        self.last_received_at.elapsed().as_millis() >= self.interval_ms as u128
    }

    /// Convert subscription type to data type
    pub fn to_data_type(&self) -> DataType {
        match self.subscription_type {
            SubscriptionType::All => DataType::System, // Special case
            SubscriptionType::System => DataType::System,
            SubscriptionType::Cpu => DataType::Cpu,
            SubscriptionType::Memory => DataType::Memory,
            SubscriptionType::Gpu => DataType::Gpu,
            SubscriptionType::Network => DataType::Network,
            SubscriptionType::Process => DataType::Process,
            SubscriptionType::Storage => DataType::Storage,
        }
    }
}

/// Represents a filter for subscription data
#[derive(Debug, Clone)]
pub enum SubscriptionFilter {
    /// Filter for process subscriptions
    Process {
        /// Specific process IDs to include
        pids: Vec<u32>,
        /// Process names to include
        names: Vec<String>,
        /// Usernames to include
        usernames: Vec<String>,
        /// Top N processes by CPU usage
        top_by_cpu: Option<u32>,
        /// Top N processes by memory usage
        top_by_memory: Option<u32>,
    },

    /// Filter for GPU subscriptions
    Gpu {
        /// GPU names to include
        names: Vec<String>,
        /// GPU vendors to include
        vendors: Vec<String>,
        /// Whether to include process information
        include_processes: bool,
    },

    /// Filter for network subscriptions
    Network {
        /// Network interface names to include
        interface_names: Vec<String>,
    },

    /// Filter for storage subscriptions
    Storage {
        /// Storage device names to include
        device_names: Vec<String>,
        /// Mount points to include
        mount_points: Vec<String>,
    },
}