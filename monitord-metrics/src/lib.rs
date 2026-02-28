/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
pub mod cpu;
pub mod gpu;
pub mod memory;

pub(crate) mod metrics {
    pub mod v1 {
        pub mod cpu {
            tonic::include_proto!("metrics.v1.cpu");
        }
        pub mod gpu {
            tonic::include_proto!("metrics.v1.gpu");
        }
        pub mod memory {
            tonic::include_proto!("metrics.v1.memory");
        }
        pub mod network {
            tonic::include_proto!("metrics.v1.network");
        }
        pub mod storage {
            tonic::include_proto!("metrics.v1.storage");
        }
        pub mod process {
            tonic::include_proto!("metrics.v1.process");
        }
        tonic::include_proto!("metrics.v1");
    }
    //#[cfg(feature = "protov1")]
    pub use v1::*;
}
