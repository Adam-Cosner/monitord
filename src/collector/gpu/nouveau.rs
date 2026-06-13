/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use rustix::fd::{AsFd, OwnedFd};
use std::path::PathBuf;

use crate::{collector::helpers::sysfs, metrics::gpu::*};

pub struct Card {
    card_fd: OwnedFd,
    primary_node: PathBuf,
    render_node: PathBuf,
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
        let mut render_node = PathBuf::new();
        for entry in rustix::fs::Dir::read_from(&drm_root)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("card") {
                primary_node = PathBuf::from(format!("/dev/dri/{}", name));
            } else if name.starts_with("renderD") {
                render_node = PathBuf::from(format!("/dev/dri/{}", name));
            }
        }
        Ok(Self {
            card_fd: fd,
            primary_node,
            render_node,
        })
    }
}

impl super::Card for Card {
    fn identify(&self) -> (String, String, Option<String>, Option<String>) {
        (
            sysfs::readat_string(self.card_fd.as_fd(), "device/vendor")
                .and_then(|v| v.strip_prefix("0x").map(|v| v.to_string()))
                .map(String::from)
                .unwrap_or_default(),
            sysfs::readat_string(self.card_fd.as_fd(), "device/device")
                .and_then(|d| d.strip_prefix("0x").map(|d| d.to_string()))
                .map(String::from)
                .unwrap_or_default(),
            sysfs::readat_string(self.card_fd.as_fd(), "device/subsystem_vendor")
                .and_then(|sv| sv.strip_prefix("0x").map(|sv| sv.to_string()))
                .map(String::from),
            sysfs::readat_string(self.card_fd.as_fd(), "device/subsystem_device")
                .and_then(|sd| sd.strip_prefix("0x").map(|sd| sd.to_string()))
                .map(String::from),
        )
    }

    fn collect(&mut self, config: &super::Config) -> anyhow::Result<Gpu> {
        let mut gpu = super::Gpu::default();
        gpu.primary_node = self.primary_node.to_string_lossy().to_string();
        gpu.render_node = self.render_node.to_string_lossy().to_string();
        gpu.pci_id = rustix::fs::readlinkat(self.card_fd.as_fd(), "device", [])
            .ok()
            .and_then(|p| {
                PathBuf::from(p.to_string_lossy().to_string())
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
            })
            .unwrap_or_default();
        gpu.drivers = config.drivers.then(|| Drivers {
            kernel: Some(KernelDriver {
                name: "nouveau".to_string(),
                version: None,
            }),
            opengl: None,
            vulkan: None,
        });
        Ok(gpu)
    }

    fn resolve(
        &mut self,
        _input: &super::process::Snapshot,
        _output: &mut Gpu,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn primary_node(&self) -> String {
        self.primary_node.to_string_lossy().to_string()
    }
}
