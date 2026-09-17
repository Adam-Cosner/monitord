/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Daemon configuration
//!
//! Initialization priority is as follows:
//! ENV -> Default filepath (/etc/monitord/config.toml) -> Default config

use monitord::helpers::io;
use monitord::metrics;
use rustix::fd::{AsFd, BorrowedFd};
use rustix::fs::{Mode, OFlags};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub core: CoreConfig,
    pub metrics: metrics::Config,
}

#[derive(Serialize, Deserialize)]
pub struct CoreConfig {
    pub interval_ms: u16,
    pub max_tries: u32,
    pub auto_shutdown_seconds: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            core: CoreConfig::default(),
            metrics: metrics::Config {
                cpu: Some(metrics::cpu::Config {
                    topology: true,
                    hwid: true,
                    drivers: true,
                }),
                memory: Some(metrics::memory::Config { dimms: true }),
                gpu: Some(metrics::gpu::Config {
                    drivers: true,
                    engines: true,
                    clocks: true,
                    memory: true,
                    power: true,
                    thermals: true,
                    processes: true,
                }),
                network: Some(metrics::network::Config {
                    addresses: true,
                    wifi_info: true,
                }),
                storage: Some(metrics::storage::Config { usage: true }),
                process: Some(metrics::process::Config {
                    identity: true,
                    status: true,
                    start_time: true,
                    cpu_usage: true,
                    memory_usage: true,
                    gpu_usage: true,
                    disk_usage: true,
                    net_usage: true,
                }),
            },
        }
    }
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            interval_ms: 1000,
            max_tries: 5,
            auto_shutdown_seconds: 10,
        }
    }
}

impl Config {
    pub fn init() -> anyhow::Result<Config> {
        let mut path_str = String::from("/etc/monitord/config.toml");
        if let Ok(env_path) = std::env::var("MONITORD_CONFIG") {
            path_str = env_path;
        }

        let config_path = std::path::Path::new(&path_str);
        let config_parent = config_path.parent().unwrap();
        std::fs::create_dir_all(config_parent)?;

        let config_fd = rustix::fs::open(
            config_path,
            OFlags::CLOEXEC | OFlags::RDWR | OFlags::CREATE,
            Mode::from(0o664),
        )?;

        let contents = io::read_string(config_fd.as_fd()).unwrap_or_default();

        // If the config file is empty
        if contents.len() == 0 {
            tracing::info!("config.toml file empty, creating new");
            Self::new_default(config_fd.as_fd())
        } else {
            // Otherwise read it
            match toml::from_str::<Self>(contents.as_str()) {
                Ok(config) => Ok(config),
                // Config file malformed, move old one to temp file and create new one
                Err(e) => {
                    tracing::warn!("config.toml file contents malformed: {e}");
                    tracing::info!(
                        "moving old config to config.toml.old and generating new config"
                    );
                    rustix::fs::rename(config_path, config_path.with_added_extension("old"))
                        .expect("config.toml file was moved or deleted mid operation!");
                    let new_config_fd = rustix::fs::open(
                        config_path,
                        OFlags::CLOEXEC | OFlags::RDWR | OFlags::CREATE,
                        Mode::from(0o664),
                    )?;
                    Self::new_default(new_config_fd.as_fd())
                }
            }
        }
    }

    fn new_default(config_fd: BorrowedFd) -> anyhow::Result<Self> {
        let config = Self::default();
        let Ok(config_toml) = toml::to_string(&config) else {
            anyhow::bail!("failed to serialize config!")
        };

        io::write_string(config_fd.as_fd(), &config_toml)?;
        Ok(config)
    }
}
