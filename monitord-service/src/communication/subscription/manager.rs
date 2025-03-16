use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::communication::core::models::{DataType, TransportType};
use crate::communication::subscription::error::SubscriptionError;
use crate::communication::subscription::models::Subscription;
use crate::communication::subscription::SubscriptionConfig;

use monitord_protocols::subscription::{
    active_subscription, modify_subscription_request, subscription_request::Filter,
    ActiveSubscription, SubscriptionRequest, SubscriptionResponse, SubscriptionStatus,
    TransportType as ProtoTransportType,
};
use monitord_protocols::subscription::{
    ListSubscriptionsRequest, ListSubscriptionsResponse, ModifySubscriptionRequest,
    UnsubscribeRequest, UnsubscribeResponse,
};

/// Manages client subscriptions to different data streams
pub struct SubscriptionManager {
    /// All active subscriptions indexed by subscription ID
    subscriptions: RwLock<HashMap<String, Subscription>>,

    /// Maps client IDs to their subscription IDs
    client_subscriptions: RwLock<HashMap<String, HashSet<String>>>,

    /// Maps data types to subscription IDs for efficient lookup
    data_type_subscriptions: RwLock<HashMap<DataType, HashSet<String>>>,

    /// Configuration settings
    config: SubscriptionConfig,

    /// Last cleanup time for stale subscriptions
    last_cleanup: RwLock<Instant>,
}

impl SubscriptionManager {
    /// Create a new subscription manager
    pub fn new(config: SubscriptionConfig) -> Result<Self, SubscriptionError> {
        Ok(Self {
            subscriptions: RwLock::new(HashMap::new()),
            client_subscriptions: RwLock::new(HashMap::new()),
            data_type_subscriptions: RwLock::new(HashMap::new()),
            config,
            last_cleanup: RwLock::new(Instant::now()),
        })
    }

    /// Create a new subscription
    pub async fn create_subscription(
        &self,
        client_id: String,
        request: SubscriptionRequest,
        transport_type: TransportType,
    ) -> Result<SubscriptionResponse, SubscriptionError> {
        // Convert protocol subscription type to internal type
        let subscription_type = request.r#type.try_into().map_err(|_| {
            SubscriptionError::InvalidType(format!(
                "Invalid subscription type: {}",
                request.r#type as i32
            ))
        })?;

        // Validate the interval (must be > 0)
        if request.interval_ms == 0 {
            return Err(SubscriptionError::InvalidInterval(
                "Interval must be greater than zero".to_string(),
            ));
        }

        // Validate the filter (if any)
        let filter = request.filter.clone();
        if let Some(ref filter) = filter {
            Self::validate_filter(filter, subscription_type)?;
        }

        // Check if the client has reached their subscription limit
        let mut client_subs = self.client_subscriptions.write().await;
        let client_sub_ids = client_subs
            .entry(client_id.clone())
            .or_insert_with(HashSet::new);

        if client_sub_ids.len() >= self.config.max_subscriptions_per_client {
            return Err(SubscriptionError::TooManySubscriptions);
        }

        // Create a new subscription
        let subscription = Subscription::new(
            subscription_type,
            client_id.clone(),
            request.interval_ms,
            transport_type,
            filter,
        );

        // Add the subscription to our maps
        let mut subscriptions = self.subscriptions.write().await;
        let mut data_type_subs = self.data_type_subscriptions.write().await;

        subscriptions.insert(subscription.id.clone(), subscription.clone());
        client_sub_ids.insert(subscription.id.clone());

        // Add to data type map for efficient lookup
        let data_type = subscription.to_data_type();
        data_type_subs
            .entry(data_type)
            .or_insert_with(HashSet::new)
            .insert(subscription.id.clone());

        // If it's ALL type, also add to each specific type
        if matches!(
            subscription_type,
            monitord_protocols::subscription::SubscriptionType::All
        ) {
            for data_type in [
                DataType::Cpu,
                DataType::Memory,
                DataType::Gpu,
                DataType::Network,
                DataType::Process,
                DataType::Storage,
                DataType::System,
            ] {
                data_type_subs
                    .entry(data_type)
                    .or_insert_with(HashSet::new)
                    .insert(subscription.id.clone());
            }
        }

        // Create response
        let response = SubscriptionResponse {
            subscription_id: subscription.id.clone(),
            status: SubscriptionStatus::Success as i32,
            error_message: String::new(),
        };

        info!(
            "Created subscription {} for client {}",
            subscription.id, client_id
        );
        Ok(response)
    }

    /// Modify an existing subscription
    pub async fn modify_subscription(
        &self,
        request: ModifySubscriptionRequest,
    ) -> Result<SubscriptionResponse, SubscriptionError> {
        let subscription_id = request.subscription_id.clone();

        // Find the subscription
        let mut subscriptions = self.subscriptions.write().await;
        let subscription = subscriptions
            .get_mut(&subscription_id)
            .ok_or_else(|| SubscriptionError::NotFound(subscription_id.clone()))?;

        // Update interval if provided
        if request.interval_ms > 0 {
            subscription.interval_ms = request.interval_ms;
        }

        // Update filter if provided
        if let Some(filter) = request.filter {
            // Validate the filter
            let filter = match &filter {
                modify_subscription_request::Filter::GpuFilter(filter) => {
                    Filter::GpuFilter(filter.clone())
                }
                modify_subscription_request::Filter::StorageFilter(filter) => {
                    Filter::StorageFilter(filter.clone())
                }
                modify_subscription_request::Filter::ProcessFilter(filter) => {
                    Filter::ProcessFilter(filter.clone())
                }
                modify_subscription_request::Filter::NetworkFilter(filter) => {
                    Filter::NetworkFilter(filter.clone())
                }
            };
            Self::validate_filter(&filter, subscription.subscription_type)?;
            subscription.filter = Some(filter);
        }

        // Create response
        let response = SubscriptionResponse {
            subscription_id,
            status: SubscriptionStatus::Success as i32,
            error_message: String::new(),
        };

        Ok(response)
    }

    /// Cancel a subscription
    pub async fn unsubscribe(
        &self,
        request: UnsubscribeRequest,
    ) -> Result<UnsubscribeResponse, SubscriptionError> {
        let subscription_id = request.subscription_id.clone();

        // Find and remove the subscription
        let mut subscriptions = self.subscriptions.write().await;
        let subscription = subscriptions
            .remove(&subscription_id)
            .ok_or_else(|| SubscriptionError::NotFound(subscription_id.clone()))?;

        // Update client subscriptions
        let mut client_subs = self.client_subscriptions.write().await;
        if let Some(client_sub_ids) = client_subs.get_mut(&subscription.client_id) {
            client_sub_ids.remove(&subscription_id);

            // Remove client entry if no subscriptions left
            if client_sub_ids.is_empty() {
                client_subs.remove(&subscription.client_id);
            }
        }

        // Update data type subscriptions
        let mut data_type_subs = self.data_type_subscriptions.write().await;
        let data_type = subscription.to_data_type();
        if let Some(type_subs) = data_type_subs.get_mut(&data_type) {
            type_subs.remove(&subscription_id);

            // Remove data type entry if no subscriptions left
            if type_subs.is_empty() {
                data_type_subs.remove(&data_type);
            }
        }

        // If it was ALL type, remove from each specific type
        if matches!(
            subscription.subscription_type,
            monitord_protocols::subscription::SubscriptionType::All
        ) {
            for data_type in [
                DataType::Cpu,
                DataType::Memory,
                DataType::Gpu,
                DataType::Network,
                DataType::Process,
                DataType::Storage,
                DataType::System,
            ] {
                if let Some(type_subs) = data_type_subs.get_mut(&data_type) {
                    type_subs.remove(&subscription_id);

                    // Remove data type entry if no subscriptions left
                    if type_subs.is_empty() {
                        data_type_subs.remove(&data_type);
                    }
                }
            }
        }

        // Create response
        let response = UnsubscribeResponse {
            success: true,
            error_message: String::new(),
        };

        info!(
            "Removed subscription {} for client {}",
            subscription_id, subscription.client_id
        );
        Ok(response)
    }

    /// List all active subscriptions
    pub async fn list_subscriptions(
        &self,
        _request: ListSubscriptionsRequest,
    ) -> Result<ListSubscriptionsResponse, SubscriptionError> {
        let subscriptions = self.subscriptions.read().await;

        let active_subscriptions = subscriptions
            .values()
            .map(|sub| ActiveSubscription {
                subscription_id: sub.id.clone(),
                r#type: sub.subscription_type as i32,
                transport_type: match sub.transport {
                    TransportType::Iceoryx => ProtoTransportType::Iceoryx as i32,
                    TransportType::Grpc => ProtoTransportType::Grpc as i32,
                },
                interval_ms: sub.interval_ms,
                created_at: format!("{:?}", sub.created_at),
                filter: match &sub.filter {
                    None => None,
                    Some(Filter::GpuFilter(filter)) => {
                        Some(active_subscription::Filter::GpuFilter(filter.clone()))
                    }
                    Some(Filter::NetworkFilter(filter)) => {
                        Some(active_subscription::Filter::NetworkFilter(filter.clone()))
                    }
                    Some(Filter::ProcessFilter(filter)) => {
                        Some(active_subscription::Filter::ProcessFilter(filter.clone()))
                    }
                    Some(Filter::StorageFilter(filter)) => {
                        Some(active_subscription::Filter::StorageFilter(filter.clone()))
                    }
                },
            })
            .collect();

        let response = ListSubscriptionsResponse {
            subscriptions: active_subscriptions,
        };

        Ok(response)
    }

    /// Get all subscriptions for a specific data type
    pub async fn get_subscriptions_for_type(
        &self,
        data_type: DataType,
    ) -> Result<Vec<Subscription>, SubscriptionError> {
        // Check for cleanup
        self.cleanup_stale_subscriptions().await?;

        let data_type_subs = self.data_type_subscriptions.read().await;
        let subscriptions = self.subscriptions.read().await;

        let sub_ids = match data_type_subs.get(&data_type) {
            Some(ids) => ids,
            None => return Ok(Vec::new()), // No subscriptions for this type
        };

        let result: Vec<Subscription> = sub_ids
            .iter()
            .filter_map(|id| subscriptions.get(id).cloned())
            .collect();

        Ok(result)
    }

    /// Validate a subscription filter
    fn validate_filter(
        filter: &Filter,
        subscription_type: monitord_protocols::subscription::SubscriptionType,
    ) -> Result<(), SubscriptionError> {
        // Check if the filter type matches the subscription type
        match (filter, subscription_type) {
            (
                Filter::ProcessFilter(_),
                monitord_protocols::subscription::SubscriptionType::Process,
            )
            | (Filter::ProcessFilter(_), monitord_protocols::subscription::SubscriptionType::All) => {
                Ok(())
            }

            (Filter::GpuFilter(_), monitord_protocols::subscription::SubscriptionType::Gpu)
            | (Filter::GpuFilter(_), monitord_protocols::subscription::SubscriptionType::All) => {
                Ok(())
            }

            (
                Filter::NetworkFilter(_),
                monitord_protocols::subscription::SubscriptionType::Network,
            )
            | (Filter::NetworkFilter(_), monitord_protocols::subscription::SubscriptionType::All) => {
                Ok(())
            }

            (
                Filter::StorageFilter(_),
                monitord_protocols::subscription::SubscriptionType::Storage,
            )
            | (Filter::StorageFilter(_), monitord_protocols::subscription::SubscriptionType::All) => {
                Ok(())
            }

            _ => Err(SubscriptionError::InvalidFilter(format!(
                "Filter type does not match subscription type: {:?}",
                subscription_type
            ))),
        }
    }

    /// Cleanup stale subscriptions (those that haven't been updated in a while)
    async fn cleanup_stale_subscriptions(&self) -> Result<(), SubscriptionError> {
        let mut last_cleanup = self.last_cleanup.write().await;
        let now = Instant::now();

        // Only cleanup periodically (every 60 seconds by default)
        if now.duration_since(*last_cleanup) < Duration::from_secs(60) {
            return Ok(());
        }

        // Update cleanup time
        *last_cleanup = now;

        let timeout = Duration::from_secs(self.config.default_timeout_seconds);
        let mut to_remove = Vec::new();

        // Find stale subscriptions
        let subscriptions = self.subscriptions.read().await;
        for (id, sub) in subscriptions.iter() {
            if now.duration_since(sub.last_received_at) > timeout {
                to_remove.push((id.clone(), sub.client_id.clone()));
            }
        }

        // Release read lock before acquiring write lock
        drop(subscriptions);

        // Remove stale subscriptions
        for (sub_id, client_id) in to_remove {
            let req = UnsubscribeRequest {
                subscription_id: sub_id.clone(),
            };

            match self.unsubscribe(req).await {
                Ok(_) => {
                    info!(
                        "Removed stale subscription {} for client {}",
                        sub_id, client_id
                    );
                }
                Err(e) => {
                    warn!("Error removing stale subscription {}: {}", sub_id, e);
                }
            }
        }

        Ok(())
    }

    /// Get all subscriptions for a client
    pub async fn get_client_subscriptions(
        &self,
        client_id: &str,
    ) -> Result<Vec<Subscription>, SubscriptionError> {
        let client_subs = self.client_subscriptions.read().await;
        let subscriptions = self.subscriptions.read().await;

        let sub_ids = match client_subs.get(client_id) {
            Some(ids) => ids,
            None => return Ok(Vec::new()), // No subscriptions for this client
        };

        let result: Vec<Subscription> = sub_ids
            .iter()
            .filter_map(|id| subscriptions.get(id).cloned())
            .collect();

        Ok(result)
    }

    /// Mark a subscription as having received data
    pub async fn mark_subscription_received(
        &self,
        subscription_id: &str,
    ) -> Result<(), SubscriptionError> {
        let mut subscriptions = self.subscriptions.write().await;

        if let Some(subscription) = subscriptions.get_mut(subscription_id) {
            subscription.last_received_at = Instant::now();
            Ok(())
        } else {
            Err(SubscriptionError::NotFound(subscription_id.to_string()))
        }
    }

    /// Check if any subscriptions exist for a data type
    pub async fn has_subscriptions_for_type(&self, data_type: DataType) -> bool {
        let data_type_subs = self.data_type_subscriptions.read().await;

        if let Some(subs) = data_type_subs.get(&data_type) {
            !subs.is_empty()
        } else {
            false
        }
    }

    /// Get subscription statistics
    pub async fn get_stats(&self) -> SubscriptionStats {
        let subscriptions = self.subscriptions.read().await;
        let client_subs = self.client_subscriptions.read().await;
        let data_type_subs = self.data_type_subscriptions.read().await;

        SubscriptionStats {
            total_subscriptions: subscriptions.len(),
            total_clients: client_subs.len(),
            subscriptions_by_type: data_type_subs
                .iter()
                .map(|(data_type, subs)| (data_type.clone(), subs.len()))
                .collect(),
        }
    }
}

/// Subscription statistics
#[derive(Debug, Clone)]
pub struct SubscriptionStats {
    /// Total number of active subscriptions
    pub total_subscriptions: usize,
    /// Total number of clients with subscriptions
    pub total_clients: usize,
    /// Number of subscriptions per data type
    pub subscriptions_by_type: HashMap<DataType, usize>,
}
