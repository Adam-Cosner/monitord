/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use tonic_prost_build::configure;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    configure().compile_protos(&["../proto/metrics.proto"], &["../proto"])?;
    Ok(())
}
