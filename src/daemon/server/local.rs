/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Local machine snapshot reporting service

use prost::bytes::Bytes;
use tokio::sync::broadcast;
use tokio_stream::wrappers::ReceiverStream;

pub struct Service {
    pub tx: broadcast::Sender<Bytes>,
}

impl Service {
    pub fn new(tx: broadcast::Sender<Bytes>) -> anyhow::Result<Self> {
        Ok(Self { tx })
    }
}

#[tonic::async_trait]
impl super::monitord_server::Monitord for Service {
    type ReportStream = ReceiverStream<Result<super::ReportPayload, tonic::Status>>;

    async fn report(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<Self::ReportStream>, tonic::Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        let mut snap_rx = self.tx.subscribe();

        tracing::info!("report request received");

        tokio::spawn(async move {
            loop {
                let snap = snap_rx.recv().await;
                tracing::debug!("snapshot received");

                match snap {
                    Ok(snap) => {
                        match tx
                            .send(Ok(super::ReportPayload {
                                data: snap.to_vec(),
                            }))
                            .await
                        {
                            Ok(_) => {
                                tracing::debug!("snapshot sent");
                            }
                            Err(_) => {
                                tracing::info!("report channel closed");
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(tonic::Status::aborted(format!(
                            "snapshot broadcaster aborted: {}",
                            e
                        ))));
                        break;
                    }
                }
            }
        });

        tracing::info!("report request completed, sending stream");
        Ok(tonic::Response::new(ReceiverStream::new(rx)))
    }
}

pub async fn run(tx: broadcast::Sender<Bytes>) -> anyhow::Result<()> {
    let service = Service::new(tx)?;

    let root = std::env::var("MONITORD_ROOT").unwrap_or_else(|_| "/etc/monitord".to_string());

    let sock_path = std::path::PathBuf::from(&root).join("local.sock");

    let uds = tokio::net::UnixListener::bind(&sock_path)?;
    let uds_stream = tokio_stream::wrappers::UnixListenerStream::new(uds);

    tonic::transport::Server::builder()
        .add_service(super::monitord_server::MonitordServer::new(service))
        .serve_with_incoming(uds_stream)
        .await?;

    Ok(())
}
