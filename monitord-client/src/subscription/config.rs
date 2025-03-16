//! Configuration for subscription behavior

/// Configuration for subscription behavior
#[derive(Debug, Clone, Default)]
pub struct SubscriptionConfig {
    /// Default update interval for subscriptions in milliseconds (default: 1000)
    pub default_interval_ms: u32,
}

impl SubscriptionConfig {
    /// Creates a new subscription configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the default interval for subscriptions in milliseconds
    pub fn with_default_interval(mut self, interval_ms: u32) -> Self {
        self.default_interval_ms = interval_ms;
        self
    }
}

// Default implementation
impl SubscriptionConfig {
    fn default() -> Self {
        Self {
            default_interval_ms: 1000,
        }
    }
}
