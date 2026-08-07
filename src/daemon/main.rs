/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod service {
    pub mod v1 {
        tonic::include_proto!("service.v1");
    }
    pub use v1::*;
}

mod config;
mod runtime;

pub use monitord::collector;
pub use monitord::metrics;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let (snap_tx, _snap_rx) = tokio::sync::mpsc::channel(12);
    let (_stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();

    tracing::info!("initializing monitord");

    let config = config::Config::init()?;

    tokio::select! {
        _ = runtime::runtime(snap_tx, stop_rx, config) => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runtime() {
        tracing_subscriber::fmt::init();
        let (snap_tx, mut snap_rx) = tokio::sync::mpsc::channel(12);
        let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
        let config = config::Config::default();

        tokio::select! {
            // runtime
            _ = runtime::runtime(snap_tx, stop_rx, config) => {}
            // dummy server
            _ = async move {
                while let Some(snap) = snap_rx.recv().await {
                    if let Ok(formatted) = format_snapshot(snap) {
                        tracing::info!("received snapshot: \n{}", formatted);
                    } else {
                        tracing::warn!("failed to format snapshot");
                    }
                }
            } => {}
            _ = async move {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                stop_tx.send(()).ok();
            } => {}
        }
    }

    fn format_snapshot(snap: metrics::Snapshot) -> anyhow::Result<String> {
        let mut output = String::new();

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
