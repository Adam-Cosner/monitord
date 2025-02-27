//! # monitord_core
//!
//! The core monitoring engine for the monitord system monitoring service.
//!
//! This crate provides functionality for collecting system information from
//! various hardware components (CPU, memory, GPU, network, storage), managing
//! client subscriptions, and distributing this information via IPC using iceoryx2.
//!
//! ## Architecture
//!
//! The architecture includes these main components:
//! - **Collectors**: Hardware-specific data collection modules
//! - **Models**: Data structures that mirror Protocol Buffer definitions
//! - **Subscription**: Management of client subscriptions for different data types
//! - **IPC**: Inter-process communication using iceoryx2

// Modules defined in their own files
pub mod collectors;
pub mod error;
pub mod ipc;
pub mod models;
pub mod subscription;
pub mod utils;
