//! Communication module for monitord
//!
//! This module handles communication between the monitord service
//! and client applications.

// Public and crate-public modules
pub(crate) mod config;
pub(crate) mod error;
pub(crate) mod handlers;

// Internal modules
mod core;
mod manager;
mod registry;
mod subscription;
mod tasks;
mod transports;

// Re-export the primary public interfaces
pub use manager::CommunicationManager;

// Re-export core traits for extensibility
pub use core::traits::Transport;
