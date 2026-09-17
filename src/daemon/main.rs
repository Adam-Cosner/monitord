/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

mod config;
mod server;
mod worker;

pub use monitord::collector;
pub use monitord::metrics;
use prost::bytes::Bytes;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let (worker_tx, _) = tokio::sync::broadcast::channel::<Bytes>(1);

    tracing::info!("initializing monitord");

    let config = config::Config::init()?;

    // note, when we get to the control component, the stop signal should be sent from the server component if no clients are connected for config's seconds
    tokio::select! {
        res = worker::run(worker_tx.clone(), &config) => {
            tracing::info!("worker exited: {:?}", res);
        }
        res = server::run(worker_tx.clone(), &config) => {
            tracing::info!("server exited: {:?}", res);
        }
    }

    tracing::info!("shutdown complete");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;

    #[tokio::test]
    async fn test_runtime() {
        tracing_subscriber::fmt::init();
        let (snap_tx, mut snap_rx) = tokio::sync::broadcast::channel(1);
        let config = config::Config::default();

        tokio::select! {
            // runtime
            _ = worker::run(snap_tx, &config) => {}
            // dummy server
            _ = async move {
                while let Ok(snap) = snap_rx.recv().await {
                    if let Ok(formatted) = format_snapshot(snap) {
                        tracing::info!("received snapshot: \n{}", formatted);
                    } else {
                        tracing::warn!("failed to format snapshot");
                    }
                }
            } => {}
            _ = async move {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            } => {}
        }
    }

    fn format_snapshot(bytes: Bytes) -> anyhow::Result<String> {
        let mut output = String::new();

        let snap = metrics::Snapshot::decode(bytes)?;

        use std::fmt::Write;

        snap.cpu
            .and_then(|s| writeln!(output, "cpu: logical cpus: {}", s.logical.len()).ok());
        snap.memory.and_then(|s| {
            writeln!(
                output,
                "memory: total: {} bytes",
                s.logical.as_ref().map_or(0, |s| s.capacity)
            )
            .ok()
        });
        snap.gpu
            .and_then(|s| writeln!(output, "gpu: {} devices", s.gpus.len()).ok());
        snap.network
            .and_then(|s| writeln!(output, "network: {} interfaces", s.adapters.len()).ok());
        snap.storage
            .and_then(|s| writeln!(output, "storage: {} devices", s.devices.len()).ok());
        snap.process
            .and_then(|s| writeln!(output, "process: {} running", s.processes.len()).ok());

        Ok(output)
    }
}
