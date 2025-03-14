//! Task-based concurrency for communication

mod connection;
mod data;

pub use connection::{spawn_connection_handler, ConnectionTask};