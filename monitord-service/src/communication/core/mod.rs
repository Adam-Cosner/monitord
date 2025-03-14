//! Core abstractions for the communication module

pub mod traits;
pub mod models;

pub use traits::{Transport, MessageHandler};
pub use models::{ClientConnection, TransportType, DataType};