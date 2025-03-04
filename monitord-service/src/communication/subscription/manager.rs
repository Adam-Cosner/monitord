use crate::config::SubscriptionConfig;
use crate::error::SubscriptionError;
use monitord_protocols::subscription::subscription_request::Filter;
use monitord_protocols::subscription::{
    active_subscription, modify_subscription_request, ActiveSubscription, GpuFilter,
    ListSubscriptionsRequest, ListSubscriptionsResponse, ModifySubscriptionRequest, NetworkFilter,
    ProcessFilter, StorageFilter, SubscriptionRequest, SubscriptionResponse, SubscriptionStatus,
    SubscriptionType, TransportType, UnsubscribeRequest, UnsubscribeResponse,
};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tokio::time::Instant;
use tracing::info;
use uuid::{uuid, Uuid};

#[derive(Clone, Debug)]
pub struct Subscription {
    // The ID of the subscription
    pub id: String,

    // The type of data being subscribed to
    pub subscription_type: SubscriptionType,

    // The interval in milliseconds to receive updates
    pub interval_ms: u32,

    // The ID of the client that requested the subscription
    pub client_id: String,

    // The transport strategy to use
    pub transport: TransportType,

    // The time the subscription was created
    pub created_at: Instant,

    // The last time the client received data
    pub last_received_at: Instant,

    // The optional filter for this subscription
    pub filter: Option<Filter>,
}

impl Subscription {
    pub fn new(
        subscription_type: SubscriptionType,
        client_id: String,
        interval_ms: u32,
        transport: TransportType,
        filter: Option<Filter>,
    ) -> Self {
        let now = Instant::now();
        Self {
            id: Uuid::new_v4().to_string(),
            subscription_type,
            interval_ms,
            client_id,
            transport,
            created_at: now,
            last_received_at: now,
            filter,
        }
    }

    pub fn should_update(&self) -> bool {
        self.last_received_at.elapsed().as_millis() >= self.interval_ms as u128
    }
}

pub struct SubscriptionManager {
    subscriptions: RwLock<HashMap<String, Subscription>>,
    client_to_subscriptions: RwLock<HashMap<String, Vec<String>>>,
    config: SubscriptionConfig,
}

impl SubscriptionManager {
    pub fn init(config: SubscriptionConfig) -> Result<Self, super::error::SubscriptionError> {
        info!("Initializing subscription manager");
        Ok(Self {
            subscriptions: RwLock::new(HashMap::new()),
            client_to_subscriptions: RwLock::new(HashMap::new()),
            config,
        })
    }

    pub async fn create_subscription(
        &self,
        client_id: String,
        request: SubscriptionRequest,
        transport_type: TransportType,
    ) -> Result<SubscriptionResponse, SubscriptionError> {
        // Validate subscription request
        self.validate_subscription_request(client_id.as_str(), &request)
            .await?;

        let subscription = Subscription::new(
            request.r#type(),
            client_id,
            request.interval_ms,
            transport_type,
            request.filter,
        );

        let subscription_id = subscription.id.clone();
        let client_id = subscription.client_id.clone();

        self.subscriptions
            .write()
            .await
            .insert(subscription.id.clone(), subscription);
        self.client_to_subscriptions
            .write()
            .await
            .insert(client_id.clone(), vec![subscription_id.clone()]);
        info!("Added subscription: {}", subscription_id);
        Ok(SubscriptionResponse {
            subscription_id,
            status: SubscriptionStatus::Success.into(),
            error_message: "".to_string(),
        })
    }

    pub async fn modify_subscription(
        &self,
        request: ModifySubscriptionRequest,
    ) -> Result<SubscriptionResponse, SubscriptionError> {
        // Validate modify request
        self.validate_modify_subscription_request(&request).await?;

        let mut subscriptions = self.subscriptions.write().await;
        let subscription = subscriptions
            .get_mut(&request.subscription_id)
            .ok_or_else(|| {
                SubscriptionError::NotFound(format!(
                    "Subscription not found: {}",
                    request.subscription_id
                ))
            })?;

        subscription.interval_ms = request.interval_ms;
        subscription.filter = match request.filter {
            Some(filter) => match filter {
                modify_subscription_request::Filter::ProcessFilter(filter) => {
                    Some(Filter::ProcessFilter(filter))
                }
                modify_subscription_request::Filter::GpuFilter(filter) => {
                    Some(Filter::GpuFilter(filter))
                }
                modify_subscription_request::Filter::NetworkFilter(filter) => {
                    Some(Filter::NetworkFilter(filter))
                }
                modify_subscription_request::Filter::StorageFilter(filter) => {
                    Some(Filter::StorageFilter(filter))
                }
            },
            None => None,
        };

        Ok(SubscriptionResponse {
            subscription_id: request.subscription_id,
            status: SubscriptionStatus::Success.into(),
            error_message: "".to_string(),
        })
    }

    pub async fn cancel_subscription(
        &mut self,
        client_id: String,
        request: UnsubscribeRequest,
    ) -> Result<UnsubscribeResponse, SubscriptionError> {
        // Validate the unsubscribe request
        self.validate_unsubscribe_request(&request).await?;
        self.subscriptions
            .write()
            .await
            .remove(&request.subscription_id);

        self.client_to_subscriptions
            .write()
            .await
            .get_mut(&client_id)
            .map(|subscriptions| {
                subscriptions.retain(|s| s != &request.subscription_id);
            });

        Ok(UnsubscribeResponse {
            success: true,
            error_message: "".to_string(),
        })
    }

    pub async fn list_subscriptions(&self) -> Result<ListSubscriptionsResponse, SubscriptionError> {
        let subscriptions = self.subscriptions.read().await;
        Ok(ListSubscriptionsResponse {
            subscriptions: subscriptions
                .iter()
                .map(|(subscription_id, s)| ActiveSubscription {
                    subscription_id: subscription_id.clone(),
                    r#type: s.subscription_type.into(),
                    transport_type: s.transport.into(),
                    interval_ms: s.interval_ms,
                    created_at: chrono::DateTime::<chrono::Utc>::from(
                        std::time::SystemTime::now() - s.created_at.elapsed(),
                    )
                    .to_rfc3339(),
                    filter: match &s.filter {
                        Some(filter) => match filter {
                            Filter::ProcessFilter(filter) => {
                                Some(active_subscription::Filter::ProcessFilter(filter.clone()))
                            }
                            Filter::GpuFilter(filter) => {
                                Some(active_subscription::Filter::GpuFilter(filter.clone()))
                            }
                            Filter::NetworkFilter(filter) => {
                                Some(active_subscription::Filter::NetworkFilter(filter.clone()))
                            }
                            Filter::StorageFilter(filter) => {
                                Some(active_subscription::Filter::StorageFilter(filter.clone()))
                            }
                        },
                        None => None,
                    },
                })
                .collect(),
        })
    }

    pub async fn list_subscriptions_by_client(
        &self,
        client: &str,
    ) -> Result<ListSubscriptionsResponse, SubscriptionError> {
        let subscriptions = self.subscriptions.read().await;

        Ok(ListSubscriptionsResponse {
            subscriptions: subscriptions
                .iter()
                .filter(|(_, s)| s.client_id == client)
                .map(|(subscription_id, s)| ActiveSubscription {
                    subscription_id: subscription_id.clone(),
                    r#type: s.subscription_type.into(),
                    transport_type: s.transport.into(),
                    interval_ms: s.interval_ms,
                    created_at: chrono::DateTime::<chrono::Utc>::from(
                        std::time::SystemTime::now() - s.created_at.elapsed(),
                    )
                    .to_rfc3339(),
                    filter: match &s.filter {
                        Some(filter) => match filter {
                            Filter::ProcessFilter(filter) => {
                                Some(active_subscription::Filter::ProcessFilter(filter.clone()))
                            }
                            Filter::GpuFilter(filter) => {
                                Some(active_subscription::Filter::GpuFilter(filter.clone()))
                            }
                            Filter::NetworkFilter(filter) => {
                                Some(active_subscription::Filter::NetworkFilter(filter.clone()))
                            }
                            Filter::StorageFilter(filter) => {
                                Some(active_subscription::Filter::StorageFilter(filter.clone()))
                            }
                        },
                        None => None,
                    },
                })
                .collect(),
        })
    }

    async fn validate_subscription_request(
        &self,
        client_id: &str,
        request: &SubscriptionRequest,
    ) -> Result<(), SubscriptionError> {
        if let Some(client_subscriptions) = self
            .client_to_subscriptions
            .read()
            .await
            .get(&client_id.to_string())
        {
            if client_subscriptions.len() == self.config.max_subscriptions_per_client {
                return Err(SubscriptionError::TooManySubscriptions);
            }
        }
        Ok(())
    }

    async fn validate_modify_subscription_request(
        &self,
        request: &ModifySubscriptionRequest,
    ) -> Result<(), SubscriptionError> {
        if self
            .subscriptions
            .read()
            .await
            .contains_key(&request.subscription_id)
        {
            return Err(SubscriptionError::AlreadyExists);
        }
        Ok(())
    }

    async fn validate_unsubscribe_request(
        &self,
        request: &UnsubscribeRequest,
    ) -> Result<(), SubscriptionError> {
        todo!()
    }
}
