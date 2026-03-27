/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Store holds the collected snapshots for one collection cycle.
//! Will cause a panic if reused across multiple collection cycles.
//! Does not need to be passed in as a mutable reference to [`Collector::collect`] due to [`OnceLock`]'s interior mutability.
//! This allows the store to be Send + Sync.
//!
//! Example usage:
//! ```no_run
//! let mut store = Store::new();
//! // does not need to be mutable due to OnceLock's interior mutability
//! collector.collect(&store);
//!
//! ```

use super::*;
use std::sync::OnceLock;

/// Store holds the collected snapshots for one collection cycle.
/// Should not be reused across multiple collection cycles.
pub struct Store {
    pub cpu: OnceLock<cpu::Snapshot>,
    pub mem: OnceLock<mem::Snapshot>,
    pub gpu: OnceLock<gpu::Snapshot>,
    pub net: OnceLock<net::Snapshot>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            cpu: OnceLock::new(),
            mem: OnceLock::new(),
            gpu: OnceLock::new(),
            net: OnceLock::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Demonstrates that the store is Send + Sync and that it can be mutated in tasks without needing mut reference or external locks.
    #[tokio::test]
    async fn store() {
        use std::sync::Arc;

        let store = Arc::new(Store::new());

        assert!(store.cpu.get().is_none());
        assert!(store.mem.get().is_none());
        assert!(store.gpu.get().is_none());
        assert!(store.net.get().is_none());

        let store_async = store.clone();

        // Mutate in task
        tokio::task::spawn(async move {
            store_async
                .cpu
                .set(cpu::Snapshot::default())
                .expect("Failed to set cpu snapshot");
            store_async
                .mem
                .set(mem::Snapshot::default())
                .expect("Failed to set mem snapshot");
            store_async
                .gpu
                .set(gpu::Snapshot::default())
                .expect("Failed to set gpu snapshot");
            store_async
                .net
                .set(net::Snapshot::default())
                .expect("Failed to set net snapshot");
        })
        .await
        .expect("Task panicked");

        assert!(store.cpu.get().is_some());
        assert!(store.mem.get().is_some());
        assert!(store.gpu.get().is_some());
        assert!(store.net.get().is_some());
    }
}
