/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Contains the runtime manager for the collectors

pub async fn runtime(
    snap_tx: tokio::sync::mpsc::Sender<crate::metrics::Snapshot>,
    stop_rx: tokio::sync::oneshot::Receiver<()>,
    config: crate::metrics::Config,
) -> anyhow::Result<()> {
    tokio::select! {
        _ = stop_rx => {
            tracing::info!("received stop signal");
            Ok(())
        }
        res =
            run_collectors(snap_tx, config)
         => { res }
    }
}

async fn run_collectors(
    snap_tx: tokio::sync::mpsc::Sender<crate::metrics::Snapshot>,
    config: crate::metrics::Config,
) -> anyhow::Result<()> {
    use crate::collector::*;
    let mut cpu_collector = CollectorWrapper::new(cpu::Collector::new());
    let mut mem_collector = CollectorWrapper::new(mem::Collector::new());
    let mut gpu_collector = CollectorWrapper::new(gpu::Collector::new());
    let mut net_collector = CollectorWrapper::new(net::Collector::new());
    let mut stor_collector = CollectorWrapper::new(storage::Collector::new());
    let mut proc_collector = CollectorWrapper::new(process::Collector::new());

    // TODO: Daemon config interval
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(200));
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

        snap_tx.send(snapshot).await?;
    }
}

// TODO: Daemon config retry count
const MAX_TRIES: u32 = 5;

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

    fn try_collect(&mut self, config: &crate::metrics::Config) -> Option<C::Output> {
        if self.try_count < MAX_TRIES {
            self.collector
                .collect(config)
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
