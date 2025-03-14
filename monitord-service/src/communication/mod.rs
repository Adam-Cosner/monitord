//! Communication module for monitord
//!
//! This module handles communication between the monitord service
//! and client applications.

pub(crate) mod config;
pub(crate) mod error;
mod manager;
mod registry;

mod core;
pub (crate) mod handlers;
mod transports;
mod subscription;
mod tasks;

// Re-export the primary public interfaces
pub use manager::CommunicationManager;

// Re-export core traits for extensibility
pub use core::traits::Transport;

// Export transport implementations

// Export subscription management
