/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Contains the runtime manager for the collectors
//!
//! Todo list:
//! - Error handling
//!     - Retry counters
//!     - Printing vs stopping handling

pub async fn runtime(
    snap_tx: tokio::sync::mpsc::Sender<crate::metrics::Snapshot>,
    stop_rx: tokio::sync::oneshot::Receiver<()>,
    config: crate::metrics::Config,
) -> anyhow::Result<()> {
    tokio::select! {
        _ = stop_rx => {
            tracing::info!("received stop signal");
            return Ok(());
        }
        res =
            run_collectors(snap_tx, config)
         => { return res; }
    }
}

async fn run_collectors(
    snap_tx: tokio::sync::mpsc::Sender<crate::metrics::Snapshot>,
    config: crate::metrics::Config,
) -> anyhow::Result<()> {
    use crate::collector::*;
    let mut cpu_collector = cpu::Collector::new();
    let mut memory_collector = mem::Collector::new();
    let mut gpu_collector = gpu::Collector::new();
    let mut network_collector = net::Collector::new();
    let mut storage_collector = storage::Collector::new();
    let mut process_collector = process::Collector::new();

    loop {
        // Collect
        let (
            cpu_snapshot,
            memory_snapshot,
            mut gpu_snapshot,
            network_snapshot,
            storage_snapshot,
            mut process_snapshot,
        ) = tokio::join!(
            async {
                cpu_collector
                    .collect(&config)
                    .inspect_err(|e| tracing::error!("cpu collector failed: {}", e))
            },
            async {
                memory_collector
                    .collect(&config)
                    .inspect_err(|e| tracing::error!("memory collector failed: {}", e))
            },
            async {
                gpu_collector
                    .collect(&config)
                    .inspect_err(|e| tracing::error!("gpu collector failed: {}", e))
            },
            async {
                network_collector
                    .collect(&config)
                    .inspect_err(|e| tracing::error!("network collector failed: {}", e))
            },
            async {
                storage_collector
                    .collect(&config)
                    .inspect_err(|e| tracing::error!("storage collector failed: {}", e))
            },
            async {
                process_collector
                    .collect(&config)
                    .inspect_err(|e| tracing::error!("process collector failed: {}", e))
            },
        );

        // Resolve
        if let Ok(proc) = process_snapshot.as_mut()
            && let Ok(gpu) = gpu_snapshot.as_mut()
        {
            process_collector.resolve(&gpu, proc)?;
        }
        if let Ok(gpu) = gpu_snapshot.as_mut()
            && let Ok(proc) = process_snapshot.as_mut()
        {
            gpu_collector.resolve(&proc, gpu)?;
        }

        let snapshot = crate::metrics::Snapshot {
            cpu: cpu_snapshot.ok(),
            memory: memory_snapshot.ok(),
            gpu: gpu_snapshot.ok(),
            network: network_snapshot.ok(),
            storage: storage_snapshot.ok(),
            process: process_snapshot.ok(),
        };

        snap_tx.send(snapshot).await?;
        // TODO: Variable interval
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
