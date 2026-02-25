/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use tonic_prost_build::configure;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    configure().compile_protos(
        &[
            "../proto/metrics/metrics.proto",
            "../proto/metrics/cpu.proto",
            "../proto/metrics/gpu.proto",
            "../proto/metrics/memory.proto",
            "../proto/metrics/network.proto",
            "../proto/metrics/storage.proto",
            "../proto/metrics/process.proto",
        ],
        &["../proto"],
    )?;
    Ok(())
}
