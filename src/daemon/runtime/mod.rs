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
    let mut cpu_tries = 0;
    let mut memory_collector = mem::Collector::new();
    let mut memory_tries = 0;
    let mut gpu_collector = gpu::Collector::new();
    let mut gpu_tries = 0;
    let mut network_collector = net::Collector::new();
    let mut network_tries = 0;
    let mut storage_collector = storage::Collector::new();
    let mut storage_tries = 0;
    let mut process_collector = process::Collector::new();
    let mut process_tries = 0;

    // temporary, add to daemon config
    const MAX_TRIES: u32 = 5;

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
                if cpu_tries < MAX_TRIES {
                    cpu_collector
                        .collect(&config)
                        .inspect_err(|e| {
                            tracing::error!("cpu collector failed: {}", e);
                            cpu_tries += 1;
                        })
                        .ok()
                } else {
                    None
                }
            },
            async {
                if memory_tries < MAX_TRIES {
                    memory_collector
                        .collect(&config)
                        .inspect_err(|e| {
                            tracing::error!("memory collector failed: {}", e);
                            memory_tries += 1;
                        })
                        .ok()
                } else {
                    None
                }
            },
            async {
                if gpu_tries < MAX_TRIES {
                    gpu_collector
                        .collect(&config)
                        .inspect_err(|e| {
                            tracing::error!("gpu collector failed: {}", e);
                            gpu_tries += 1;
                        })
                        .ok()
                } else {
                    None
                }
            },
            async {
                if network_tries < MAX_TRIES {
                    network_collector
                        .collect(&config)
                        .inspect_err(|e| {
                            tracing::error!("network collector failed: {}", e);
                            network_tries += 1;
                        })
                        .ok()
                } else {
                    None
                }
            },
            async {
                if storage_tries < MAX_TRIES {
                    storage_collector
                        .collect(&config)
                        .inspect_err(|e| {
                            tracing::error!("storage collector failed: {}", e);
                            storage_tries += 1;
                        })
                        .ok()
                } else {
                    None
                }
            },
            async {
                if process_tries < MAX_TRIES {
                    process_collector
                        .collect(&config)
                        .inspect_err(|e| {
                            tracing::error!("process collector failed: {}", e);
                            process_tries += 1;
                        })
                        .ok()
                } else {
                    None
                }
            },
        );

        // Resolve
        if let Some(proc) = process_snapshot.as_mut()
            && let Some(gpu) = gpu_snapshot.as_mut()
        {
            process_collector.resolve(&gpu, proc)?;
        }
        if let Some(gpu) = gpu_snapshot.as_mut()
            && let Some(proc) = process_snapshot.as_mut()
        {
            gpu_collector.resolve(&proc, gpu)?;
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
        // TODO: Variable interval
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
