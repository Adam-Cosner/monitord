/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#[allow(unused)]
fn daemon_protos() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(false)
        .type_attribute(
            ".metrics.v1.cpu.Config",
            "#[derive(serde::Deserialize, serde::Serialize)]",
        )
        .type_attribute(
            ".metrics.v1.gpu.Config",
            "#[derive(serde::Deserialize, serde::Serialize)]",
        )
        .type_attribute(
            ".metrics.v1.memory.Config",
            "#[derive(serde::Deserialize, serde::Serialize)]",
        )
        .type_attribute(
            ".metrics.v1.network.Config",
            "#[derive(serde::Deserialize, serde::Serialize)]",
        )
        .type_attribute(
            ".metrics.v1.process.Config",
            "#[derive(serde::Deserialize, serde::Serialize)]",
        )
        .type_attribute(
            ".metrics.v1.storage.Config",
            "#[derive(serde::Deserialize, serde::Serialize)]",
        )
        .type_attribute(
            ".metrics.v1.Config",
            "#[derive(serde::Deserialize, serde::Serialize)]",
        )
        .compile_protos(
            &[
                "proto/service/v1/service.proto",
                "proto/control/v1/control.proto",
            ],
            &["proto/"],
        )?;
    Ok(())
}

#[allow(unused)]
fn client_protos() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .build_server(false)
        .build_client(true)
        .compile_protos(
            &[
                "proto/metrics/v1/metrics.proto",
                "proto/service/v1/service.proto",
            ],
            &["proto/"],
        )?;
    Ok(())
}

#[allow(unused)]
fn control_protos() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .build_server(false)
        .build_client(true)
        .compile_protos(&["proto/control/v1/control.proto"], &["proto/"])?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(all(feature = "daemon", not(feature = "control"), not(feature = "client")))]
    daemon_protos()?;

    #[cfg(all(not(feature = "daemon"), feature = "control", not(feature = "client")))]
    control_protos()?;

    #[cfg(all(not(feature = "daemon"), not(feature = "control"), feature = "client"))]
    client_protos()?;

    Ok(())
}
