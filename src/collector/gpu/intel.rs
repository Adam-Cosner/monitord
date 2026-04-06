/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
use std::path::Path;

pub(super) struct Collector {}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector {
    pub fn new() -> Self {
        tracing::debug!("initializing Intel GPU collector");
        Collector {}
    }

    pub fn collect(&mut self, path: &Path, _config: &super::Config) -> anyhow::Result<super::Gpu> {
        let driver_path = path.join("device/driver");
        if let Ok(driver_link) = std::fs::read_link(driver_path) {
            if driver_link.file_name().is_some_and(|name| name == "i915") {
                self.collect_i915(path)
            } else if driver_link.file_name().is_some_and(|name| name == "xe") {
                self.collect_xe(path)
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

    fn collect_i915(&mut self, path: &Path) -> anyhow::Result<super::Gpu> {
        tracing::trace!("collecting metrics for i915 device {}", path.display());
        Err(anyhow::anyhow!("i915 not yet implemented"))
    }

    fn collect_xe(&mut self, path: &Path) -> anyhow::Result<super::Gpu> {
        tracing::trace!("collecting metrics for xe device {}", path.display());
        Err(anyhow::anyhow!("xe not yet implemented"))
    }
}
