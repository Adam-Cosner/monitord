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
use rustix::fd::{AsFd, BorrowedFd};
use rustix::fs::{Mode, OFlags};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub core: CoreConfig,
    pub metrics: MetricsConfig,
}

#[derive(Serialize, Deserialize)]
pub struct CoreConfig {
    pub interval_ms: u16,
}

#[derive(Serialize, Deserialize)]
// uses lists of strings for whichever config fields it has enabled
pub struct MetricsConfig {
    pub cpu: Vec<String>,
    pub mem: Vec<String>,
    pub gpu: Vec<String>,
    pub net: Vec<String>,
    pub process: Vec<String>,
    pub storage: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            core: CoreConfig::default(),
            metrics: Default::default(),
        }
    }
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self { interval_ms: 1000 }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            cpu: vec![
                "topology".to_string(),
                "hwid".to_string(),
                "drivers".to_string(),
            ],
            mem: vec!["dimms".to_string()],
            gpu: vec![
                "drivers".to_string(),
                "engines".to_string(),
                "clocks".to_string(),
                "memory".to_string(),
                "power".to_string(),
                "thermals".to_string(),
                "processes".to_string(),
            ],
            net: vec!["addresses".to_string(), "wifi_info".to_string()],
            storage: vec!["usage".to_string()],
            process: vec![
                "identity".to_string(),
                "status".to_string(),
                "start_time".to_string(),
                "cpu_usage".to_string(),
                "memory_usage".to_string(),
                "gpu_usage".to_string(),
                "disk_usage".to_string(),
                "net_usage".to_string(),
            ],
        }
    }
}

impl Config {
    pub fn init() -> anyhow::Result<Config> {
        let mut config_path = String::from("/etc/monitord/config.toml");
        if let Ok(env_path) = std::env::var("MONITORD_CONFIG") {
            config_path = env_path;
        }
        let config_fd = rustix::fs::open(
            &config_path,
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
                    rustix::fs::rename(&config_path, config_path.clone() + ".old")
                        .expect("config.toml file was moved or deleted mid operation!");
                    let new_config_fd = rustix::fs::open(
                        &config_path,
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
