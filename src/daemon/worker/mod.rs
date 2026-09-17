/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Contains the runtime manager for the collectors

use prost::{Message, bytes::Bytes};

pub async fn run(
    snap_tx: tokio::sync::broadcast::Sender<Bytes>,
    config: &crate::config::Config,
) -> anyhow::Result<()> {
    use crate::collector::*;
    let mut cpu_collector = CollectorWrapper::new(cpu::Collector::new());
    let mut mem_collector = CollectorWrapper::new(mem::Collector::new());
    let mut gpu_collector = CollectorWrapper::new(gpu::Collector::new());
    let mut net_collector = CollectorWrapper::new(net::Collector::new());
    let mut stor_collector = CollectorWrapper::new(storage::Collector::new());
    let mut proc_collector = CollectorWrapper::new(process::Collector::new());

    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(
        config.core.interval_ms as u64,
    ));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        interval.tick().await;
        // Collect
        let (
            cpu_snapshot,
            memory_snapshot,
            mut gpu_snapshot,
            network_snapshot,
            storage_snapshot,
            mut process_snapshot,
        ) = tokio::join!(
            async { cpu_collector.try_collect(&config) },
            async { mem_collector.try_collect(&config) },
            async { gpu_collector.try_collect(&config) },
            async { net_collector.try_collect(&config) },
            async { stor_collector.try_collect(&config) },
            async { proc_collector.try_collect(&config) },
        );

        // Resolve
        if let Some(proc) = process_snapshot.as_mut()
            && let Some(gpu) = gpu_snapshot.as_mut()
        {
            proc_collector.collector.resolve(&gpu, proc)?;
        }
        if let Some(gpu) = gpu_snapshot.as_mut()
            && let Some(proc) = process_snapshot.as_mut()
        {
            gpu_collector.collector.resolve(&proc, gpu)?;
        }

        let snapshot = crate::metrics::Snapshot {
            cpu: cpu_snapshot,
            memory: memory_snapshot,
            gpu: gpu_snapshot,
            network: network_snapshot,
            storage: storage_snapshot,
            process: process_snapshot,
        };

        let response = crate::server::ReportResponse {
            timestamp: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
            report: Some(snapshot),
        };

        let _ = snap_tx.send(response.encode_to_vec().into());
    }
}

struct CollectorWrapper<C: crate::collector::Collector> {
    try_count: u32,
    pub collector: C,
}

impl<C: crate::collector::Collector> CollectorWrapper<C> {
    fn new(c: C) -> Self {
        Self {
            try_count: 0,
            collector: c,
        }
    }

    fn try_collect(&mut self, config: &crate::config::Config) -> Option<C::Output> {
        if self.try_count < config.core.max_tries {
            self.collector
                .collect(&config.metrics)
                .inspect_err(|e| {
                    tracing::error!("{} collector failed: {e}", C::name());
                    self.try_count += 1;
                })
                .ok()
        } else {
            tracing::warn!("no {} data collected due to too many fails!", C::name());
            None
        }
    }
}
