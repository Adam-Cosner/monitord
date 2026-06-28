/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Storage device collector

use std::collections::HashMap;
use std::os::fd::AsRawFd;

use rustix::fd::AsFd;
use rustix::fs::{AtFlags, Mode, OFlags};

#[doc(inline)]
pub use crate::metrics::storage::*;

use super::helpers::*;

pub struct Collector {
    previous_samples: HashMap<String, (u64, u64)>,
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Collector for Collector {
    type Output = Snapshot;

    fn name(&self) -> &'static str {
        "storage"
    }

    fn collect(&mut self, config: &crate::metrics::Config) -> anyhow::Result<Self::Output> {
        let Some(config) = config.storage else {
            return Ok(Snapshot::default());
        };

        let mut devices = Vec::new();
        for entry in std::fs::read_dir("/sys/block")? {
            let Ok(entry) = entry else {
                continue;
            };
            // Open the directory so we don't have a TOCTOU race condition
            let Ok(dir_fd) = rustix::fs::open(
                &entry.path(),
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
                Mode::empty(),
            ) else {
                continue;
            };
            // Filter out virtual devices
            if !rustix::fs::statat(dir_fd.as_fd(), "device", AtFlags::empty()).is_ok() {
                continue;
            }

            let device_id = entry.file_name().to_string_lossy().to_string();

            // Now we can read the data
            let Some(name) = sysfs::readat_string(dir_fd.as_fd(), "device/model") else {
                continue;
            };

            // Calculate type
            let ty = if device_id.starts_with("mmcblk") {
                DeviceType::SdCard
            } else if device_id.starts_with("sr") {
                DeviceType::Optical
            } else if device_id.starts_with("nvme") {
                DeviceType::Nvme
            } else if let Ok(link) =
                std::fs::read_link(format!("/proc/self/fd/{}", dir_fd.as_raw_fd()))
                    .map(|path| path.to_string_lossy().to_string())
                && link.contains("/usb")
            {
                DeviceType::Usb
            } else if let Some(rotational) = sysfs::readat_u32(dir_fd.as_fd(), "queue/rotational") {
                if rotational == 1 {
                    DeviceType::Hdd
                } else {
                    DeviceType::Ssd
                }
            } else {
                DeviceType::Unknown
            } as i32;

            let Some(capacity) = sysfs::readat_u64(dir_fd.as_fd(), "size").map(|s| s * 512) else {
                continue;
            };

            let usage = config
                .usage
                .then(|| {
                    let Some(stat) = sysfs::readat_string(dir_fd.as_fd(), "stat") else {
                        return None;
                    };

                    let split: Vec<_> = stat.split_ascii_whitespace().collect();

                    let Some((total_read, total_write)) = split
                        .get(2)
                        .and_then(|s| s.parse::<u64>().ok().map(|r| r * 512))
                        .zip(
                            split
                                .get(6)
                                .and_then(|s| s.parse::<u64>().ok().map(|w| w * 512)),
                        )
                    else {
                        return None;
                    };

                    let Some(key) = sysfs::readat_string(dir_fd.as_fd(), "dev") else {
                        return None;
                    };
                    let Some(&(prev_read, prev_write)) = self.previous_samples.get(&key) else {
                        return Some(DiskUsage {
                            read: 0,
                            write: 0,
                            total_read,
                            total_write,
                        });
                    };

                    let read = total_read.saturating_sub(prev_read);
                    let write = total_write.saturating_sub(prev_write);

                    self.previous_samples
                        .insert(key.clone(), (total_read, total_write));

                    Some(DiskUsage {
                        read,
                        write,
                        total_read,
                        total_write,
                    })
                })
                .flatten();

            let writable = if let Some(ro) = sysfs::readat_u32(dir_fd.as_fd(), "ro") {
                ro == 0
            } else {
                false
            };

            let removable = if let Some(removable) = sysfs::readat_u32(dir_fd.as_fd(), "removable")
            {
                removable == 1
            } else {
                false
            };

            devices.push(Device {
                name,
                ty,
                capacity,
                usage,
                device_id,
                writable,
                removable,
            });
        }

        Ok(Snapshot { devices })
    }
}

impl Collector {
    pub fn new() -> Self {
        Self {
            previous_samples: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collector::Collector;

    #[tracing_test::traced_test]
    #[test]
    fn storage() -> anyhow::Result<()> {
        let mut collector = super::Collector::new();
        let mut config = crate::metrics::Config::default();
        config.storage = Some(Config { usage: true });

        let _ = collector.collect(&config)?;
        std::thread::sleep(std::time::Duration::from_secs(1));
        let snapshot = collector.collect(&config)?;
        assert!(!snapshot.devices.is_empty());
        println!("{:#?}", snapshot);
        Ok(())
    }
}
