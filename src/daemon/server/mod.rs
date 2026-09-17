/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Monitord snapshot reporting gRPC service implementation

mod local;
mod remote;
pub mod v1 {
    tonic::include_proto!("service.v1");
}
use prost::bytes::Bytes;
pub use v1::*;

use tokio::sync::broadcast;

pub async fn run(
    snap_tx: broadcast::Sender<Bytes>,
    _config: &crate::config::Config,
) -> anyhow::Result<()> {
    // todo: for now will just spawn the local server, once the control component is ready, implement the remote server
    // also todo: implement auto-shutdown logic and connection tracking in this component, not the subcomponents
    local::run(snap_tx).await?;

    tracing::info!("server stopped");

    Ok(())
}
