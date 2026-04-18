/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Intel GPU collection requires ioctl calls on render nodes

mod i915;
mod xe;

use std::collections::HashMap;
use std::os::fd::{AsFd, OwnedFd};
use std::path::{Path, PathBuf};

pub(super) struct Collector {
    // Held file descriptors for render nodes (for ioctl calls)
    fds: HashMap<String, OwnedFd>,
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector {
    pub fn new() -> Self {
        tracing::debug!("initializing Intel GPU collector");
        Collector {
            fds: HashMap::new(),
        }
    }

    pub fn collect(&mut self, path: &Path, config: &super::Config) -> anyhow::Result<super::Gpu> {
        let driver_path = path.join("device/driver");
        if let Ok(driver_link) = std::fs::read_link(driver_path) {
            // Get the file descriptor for the render node
            let fd = match self.fds.get(&path.to_string_lossy().to_string()) {
                Some(fd) => fd.as_fd(),
                None => {
                    let render_node = path
                        .file_name()
                        .and_then(|p| Some(p.to_string_lossy().to_string()))
                        .unwrap_or_default();
                    if render_node.is_empty() {
                        anyhow::bail!("gpu {path:?} could not be parsed?")
                    }
                    let Ok(render_node_file) =
                        std::fs::File::open(PathBuf::from(format!("/dev/dri/{}", render_node)))
                    else {
                        anyhow::bail!("gpu {path:?} does not have a render node file???")
                    };
                    let fd: OwnedFd = render_node_file.into();
                    self.fds.insert(path.to_string_lossy().to_string(), fd);
                    self.fds
                        .get(&path.to_string_lossy().to_string())
                        .unwrap()
                        .as_fd()
                }
            };
            if driver_link.file_name().is_some_and(|name| name == "i915") {
                i915::collect(path, fd, config)
            } else if driver_link.file_name().is_some_and(|name| name == "xe") {
                xe::collect(path, fd, config)
            } else {
                Err(anyhow::anyhow!(
                    "Unsupported driver: {}",
                    driver_link
                        .file_name()
                        .map(|name| name.to_string_lossy().to_string())
                        .unwrap_or_else(String::new)
                ))
            }
        } else {
            Err(anyhow::anyhow!(
                "failed to determine Intel GPU driver, this is a bug. Please report this issue."
            ))
        }
    }
}
