//! Error types for subscription management

use thiserror::Error;

/// Error types for subscription management
#[derive(Error, Debug)]
pub enum SubscriptionError {
    #[error("Subscription not found: {0}")]
    NotFound(String),

    #[error("Subscription already exists")]
    AlreadyExists,

    #[error("Maximum subscriptions per client exceeded")]
    TooManySubscriptions,

    #[error("Invalid subscription request: {0}")]
    InvalidRequest(String),

    #[error("Invalid filter: {0}")]
    InvalidFilter(String),

    #[error("Invalid subscription type: {0}")]
    InvalidType(String),

    #[error("Invalid interval: {0}")]
    InvalidInterval(String),

    #[error("Lock acquisition failed: {0}")]
    LockError(String),
}
