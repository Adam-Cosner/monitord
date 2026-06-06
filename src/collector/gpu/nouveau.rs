/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::{os::fd::OwnedFd, path::PathBuf};

use rustix::fs::{Mode, OFlags};

use crate::metrics::gpu::*;

pub struct Card {
    primary_node: PathBuf,
}

impl Card {
    pub fn new(fd: OwnedFd) -> anyhow::Result<Self> {
        let drm_root = rustix::fs::openat(
            &fd,
            "device/drm",
            rustix::fs::OFlags::DIRECTORY
                | rustix::fs::OFlags::RDONLY
                | rustix::fs::OFlags::CLOEXEC,
            rustix::fs::Mode::empty(),
        )?;
        let mut primary_node = PathBuf::new();
        for entry in rustix::fs::Dir::read_from(&drm_root)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("card") {
                primary_node = PathBuf::from(format!("/dev/dri/{}", name));
            }
        }
        Ok(Self { primary_node })
    }
}

impl super::Card for Card {
    fn collect(&mut self, config: &super::Config) -> anyhow::Result<Gpu> {
        let mut gpu = super::Gpu::default();

        gpu.drivers = config.drivers.then(|| Drivers {
            kernel: "nouveau".to_string(),
            opengl: "".to_string(),
            vulkan: "".to_string(),
        });

        Ok(gpu)
    }

    fn resolve(
        &mut self,
        staging: &crate::collector::staging::Staging,
        output: Gpu,
    ) -> anyhow::Result<Gpu> {
        todo!()
    }

    fn primary_node(&self) -> String {
        self.primary_node.to_string_lossy().to_string()
    }
}
