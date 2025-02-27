use thiserror::Error;

/// Top-level error type for the monitord core functionality
#[derive(Error, Debug)]
pub enum Error {
    #[error("Hardware collection error: {0}")]
    Collection(#[from] CollectionError),

    #[error("Data model error: {0}")]
    Model(#[from] ModelError),

    //#[error("Communication error: {0}")]
    //Communication(#[from] CommunicationError),

    //#[error("Subscription error: {0}")]
    //Subscription(#[from] SubscriptionError),

    //#[error("System error: {0}")]
    //System(#[from] SystemError),

    //#[error("Configuration error: {0}")]
    //Config(#[from] ConfigError),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Errors that can occur during hardware data collection
#[derive(Error, Debug)]
pub enum CollectionError {
    #[error("Device not available: {device}")]
    DeviceNotAvailable { device: String },

    #[error("Failed to access hardware: {message}")]
    AccessDenied { message: String },

    #[error("Driver error: {driver} - {message}")]
    DriverError { driver: String, message: String },

    #[error("Timeout collecting from {component} after {seconds}s")]
    Timeout { component: String, seconds: u64 },

    #[error("Device returned invalid data: {0}")]
    InvalidData(String),
}

/// Errors related to data models and conversion
#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Failed to convert data: {0}")]
    Conversion(String),

    #[error("Data validation failed: {0}")]
    Validation(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Data out of valid range: {field} ({value}) not in {min}..{max}")]
    OutOfRange {
        field: String,
        value: String,
        min: String,
        max: String,
    },
}
