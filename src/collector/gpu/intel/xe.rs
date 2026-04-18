/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
use crate::metrics::gpu::*;
use std::os::fd::BorrowedFd;
use std::path::Path;

pub fn collect(path: &Path, fd: BorrowedFd, config: &Config) -> anyhow::Result<Gpu> {
    tracing::trace!("collecting metrics for xe device {}", path.display());

    let brand_name = String::from("todo");

    let drivers = config.drivers.then(|| todo!()).unwrap_or_default();
    let engines = config.engines.then(|| todo!()).unwrap_or_default();
    let clocks = config.clocks.then(|| todo!()).unwrap_or_default();
    let memory = config.memory.then(|| todo!()).unwrap_or_default();
    let power = config.power.then(|| todo!()).unwrap_or_default();
    let thermals = config.thermals.then(|| todo!()).unwrap_or_default();
    let processes = config.processes.then(|| todo!()).unwrap_or_default();

    Ok(Gpu {
        brand_name,
        drivers,
        engines,
        clocks,
        memory,
        power,
        thermals,
        processes,
    })
}
