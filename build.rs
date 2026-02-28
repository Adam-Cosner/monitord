/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "metrics")]
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(
            &[
                "proto/metrics/v1/metrics.proto",
                "proto/metrics/v1/cpu.proto",
                "proto/metrics/v1/gpu.proto",
                "proto/metrics/v1/memory.proto",
                "proto/metrics/v1/network.proto",
                "proto/metrics/v1/process.proto",
                "proto/metrics/v1/storage.proto",
            ],
            &["proto/"],
        )?;

    #[cfg(feature = "daemon")]
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(&["proto/service/v1/service.proto"], &["proto/"])?;

    #[cfg(feature = "control")]
    tonic_prost_build::configure()
        .build_server(false)
        .build_client(true)
        .compile_protos(&["proto/service/v1/service.proto"], &["proto/"])?;

    Ok(())
}
