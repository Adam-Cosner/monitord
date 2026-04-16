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
//!
//! ```

use super::*;

/// A Staging struct holds the intermediate collection data from independent collection
/// It is then passed into collectors that implement Resolver for inter-dependent code
pub struct Staging {
    pub cpu: cpu::Snapshot,
    pub mem: mem::Snapshot,
    pub gpu: gpu::Snapshot,
    pub net: net::Snapshot,
}
