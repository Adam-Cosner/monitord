//! Type definitions for the monitord client
//!
//! This module re-exports the protocol types from the monitord-protocols crate
//! with additional client-specific functionality.

pub use monitord_protocols::monitord::{
    CpuInfo, GpuInfo, MemoryInfo, NetworkInfo, ProcessInfo, StorageInfo, SystemInfo, SystemSnapshot,
};
