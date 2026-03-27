/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
#[cfg(feature = "collector")]
pub mod cpu;
#[cfg(feature = "collector")]
pub mod gpu;
#[cfg(feature = "collector")]
pub mod mem;
#[cfg(feature = "collector")]
pub mod net;
#[cfg(feature = "collector")]
pub mod store;

#[cfg(feature = "collector")]
pub trait Collector: Send {
    /// The data type produced by this collector.
    type Output: Send;

    /// The name of the collector, for use in dependency ordering.
    fn name(&self) -> &'static str;

    /// The names of the collectors that this collector depends on.
    fn dependencies(&self) -> &[&'static str];

    /// Collect the data, using the store as a shared data source that dependent collectors can read from.
    /// Value is placed into the store slot associated with this collector.
    /// The reason it's emplaced into the parameter instead of return type is so that dependent collectors have access to the data without needing special function signatures.
    ///
    /// # Panics
    ///
    /// Panics if the associated snapshot is already set in the store.
    fn collect(&mut self, store: &store::Store) -> anyhow::Result<()>;
}

#[cfg(feature = "collector")]
pub(crate) mod helpers;
