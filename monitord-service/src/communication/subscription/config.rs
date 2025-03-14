//! Configuration for subscription management

/// Configuration for subscription management
#[derive(Debug, Clone)]
pub struct SubscriptionConfig {
    /// Maximum number of subscriptions per client
    pub max_subscriptions_per_client: usize,
    /// Default timeout for subscriptions in seconds
    pub default_timeout_seconds: u64,
    /// Whether to require authentication for subscriptions
    pub require_authentication: bool,
}

impl Default for SubscriptionConfig {
    fn default() -> Self {
        Self {
            max_subscriptions_per_client: 100,
            default_timeout_seconds: 60,
            require_authentication: false,
        }
    }
}