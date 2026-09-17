/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub use crate::metrics;
use prost::Message;
use tokio_stream::StreamExt;

pub mod service {
    pub mod v1 {
        tonic::include_proto!("service.v1");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn client() {
        tracing_subscriber::fmt::init();
        let mut client =
            service::v1::monitord_client::MonitordClient::connect("http://[::1]:50051")
                .await
                .unwrap();

        let stream = client.report(()).await.unwrap().into_inner();

        let mut stream = stream.take(10);
        while let Some(payload) = stream.next().await.map(|r| r.ok()).flatten() {
            let response = service::v1::ReportResponse::decode(payload.data.as_slice()).unwrap();
            let Some(report) = response.report else {
                tracing::warn!("No report in response");
                continue;
            };
            tracing::info!("Received metrics report: {:#?}", report)
        }
    }
}
