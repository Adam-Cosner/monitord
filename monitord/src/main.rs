/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

mod config;
mod error;
mod server;
mod transport {
    mod control {
        tonic::include_proto!("control");
    }
    mod client {
        tonic::include_proto!("monitord");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("Monitord daemon executed, does nothing for now");

    // Load config from file
    let config = config::load_config_from_file("/etc/monitord.toml");

    // Run the server
    let mut server = server::Server::new(&config);

    server.run().await?;

    Ok(())
}
