/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::{os::fd::OwnedFd, path::PathBuf};

pub struct Card {
    primary_node: PathBuf,
    render_node: OwnedFd,
}

impl Card {
    pub fn new(fd: OwnedFd) -> anyhow::Result<Self> {
        let drm_subsystem = rustix::fs::openat(
            &fd,
            "device/drm",
            rustix::fs::OFlags::RDONLY
                | rustix::fs::OFlags::DIRECTORY
                | rustix::fs::OFlags::CLOEXEC,
            rustix::fs::Mode::empty(),
        )?;
        let mut primary_node = PathBuf::new();
        let mut render_node = None;
        for entry in rustix::fs::Dir::read_from(&drm_subsystem)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("card") {
                primary_node = PathBuf::from(format!("/dev/dri/{}", name));
            } else if name.starts_with("renderD") {
                render_node = Some(rustix::fs::open(
                    format!("/dev/dri/{}", name),
                    rustix::fs::OFlags::RDWR
                        | rustix::fs::OFlags::CLOEXEC
                        | rustix::fs::OFlags::NONBLOCK,
                    rustix::fs::Mode::empty(),
                )?);
            }
        }
        Ok(Self {
            primary_node,
            render_node: render_node.ok_or_else(|| anyhow::anyhow!("render node not found"))?,
        })
    }
}

impl super::Card for Card {
    fn collect(&mut self, config: &super::Config) -> anyhow::Result<super::Gpu> {
        todo!()
    }

    fn resolve(
        &mut self,
        staging: &crate::collector::staging::Staging,
        output: super::Gpu,
    ) -> anyhow::Result<super::Gpu> {
        todo!()
    }

    fn primary_node(&self) -> String {
        self.primary_node.to_string_lossy().to_string()
    }
}
