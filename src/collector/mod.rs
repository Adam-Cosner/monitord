/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
pub mod cpu;
pub mod gpu;
pub mod mem;
pub mod net;
pub mod process;
pub mod storage;

/// Trait for independent data collection
pub trait Collector {
    /// The data type produced by this collector
    type Output: Send;

    /// The name of the collector
    fn name() -> &'static str;

    /// Collect any independent data and return it
    fn collect(&mut self, config: &crate::metrics::Config) -> anyhow::Result<Self::Output>;
}

/// Trait for dependent data resolution after collection
pub trait Resolver: Collector {
    type Input: Send;

    /// Resolves the snapshot using any data that another collector generated.
    /// Current usage is for sharing GPU Snapshot processes and Process Snapshot GPU statistics
    fn resolve(&mut self, input: &Self::Input, output: &mut Self::Output) -> anyhow::Result<()>;
}

pub(crate) mod helpers;
