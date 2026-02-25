/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
pub mod cpu;
pub mod gpu;
pub mod memory;

pub(crate) mod metrics {
    pub mod cpu {
        tonic::include_proto!("metrics.cpu");
    }
    pub mod gpu {
        tonic::include_proto!("metrics.gpu");
    }
    pub mod memory {
        tonic::include_proto!("metrics.memory");
    }
    pub mod network {
        tonic::include_proto!("metrics.network");
    }
    pub mod storage {
        tonic::include_proto!("metrics.storage");
    }
    pub mod process {
        tonic::include_proto!("metrics.process");
    }
    tonic::include_proto!("metrics");
}
