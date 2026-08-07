/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod v1 {
    pub mod cpu {
        tonic::include_proto!("metrics.v1.cpu");
        impl Config {
            pub fn from_strings(fields: &[String]) -> Self {
                let mut config = Self::default();
                for field in fields {
                    match field.as_str() {
                        "topology" => config.topology = true,
                        "hwid" => config.hwid = true,
                        "drivers" => config.drivers = true,
                        _ => tracing::warn!("ignoring unknown cpu config field {field}"),
                    }
                }
                config
            }
        }
    }
    pub mod gpu {
        tonic::include_proto!("metrics.v1.gpu");
        impl Config {
            pub fn from_strings(fields: &[String]) -> Self {
                let mut config = Self::default();
                for field in fields {
                    match field.as_str() {
                        "drivers" => config.drivers = true,
                        "engines" => config.engines = true,
                        "clocks" => config.clocks = true,
                        "memory" => config.memory = true,
                        "power" => config.power = true,
                        "thermals" => config.thermals = true,
                        "processes" => config.processes = true,
                        _ => tracing::warn!("ignoring unknown gpu config field {field}"),
                    }
                }
                config
            }
        }
    }
    pub mod memory {
        tonic::include_proto!("metrics.v1.memory");
        impl Config {
            pub fn from_strings(fields: &[String]) -> Self {
                let mut config = Self::default();
                for field in fields {
                    match field.as_str() {
                        "dimms" => config.dimms = true,
                        _ => tracing::warn!("ignoring unknown memory config field {field}"),
                    }
                }
                config
            }
        }
    }
    pub mod network {
        tonic::include_proto!("metrics.v1.network");
        impl Config {
            pub fn from_strings(fields: &[String]) -> Self {
                let mut config = Self::default();
                for field in fields {
                    match field.as_str() {
                        "addresses" => config.addresses = true,
                        "wifi_info" => config.wifi_info = true,
                        _ => tracing::warn!("ignoring unknown network config field {field}"),
                    }
                }
                config
            }
        }
    }
    pub mod storage {
        tonic::include_proto!("metrics.v1.storage");
        impl Config {
            pub fn from_strings(fields: &[String]) -> Self {
                let mut config = Self::default();
                for field in fields {
                    match field.as_str() {
                        "usage" => config.usage = true,
                        _ => tracing::warn!("ignoring unknown storage config field {field}"),
                    }
                }
                config
            }
        }
    }
    pub mod process {
        tonic::include_proto!("metrics.v1.process");
        impl Config {
            pub fn from_strings(fields: &[String]) -> Self {
                let mut config = Self::default();
                for field in fields {
                    match field.as_str() {
                        "identity" => config.identity = true,
                        "status" => config.status = true,
                        "start_time" => config.start_time = true,
                        "cpu_usage" => config.cpu_usage = true,
                        "memory_usage" => config.memory_usage = true,
                        "gpu_usage" => config.gpu_usage = true,
                        "disk_usage" => config.disk_usage = true,
                        "net_usage" => config.net_usage = true,
                        _ => tracing::warn!("ignoring unknown process config field {field}"),
                    }
                }
                config
            }
        }
    }
    tonic::include_proto!("metrics.v1");
}

pub use v1::*;
