#[derive(Debug, Clone)]
pub struct SubscriptionConfig {
    pub max_subscriptions_per_client: usize,
    pub default_timeout_seconds: u64,
    pub require_authentication: bool,
}
