use thiserror::Error;

#[derive(Error, Debug)]
pub enum SubscriptionError {
    #[error("Subscription not found: {0}")]
    NotFound(String),

    #[error("Subscription already exists")]
    AlreadyExists,

    #[error("Maximum subscriptions per client exceeded")]
    TooManySubscriptions,
}
