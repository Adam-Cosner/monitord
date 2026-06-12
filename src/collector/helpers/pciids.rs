/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Helper data type for parsing the pci.ids file.

use std::collections::HashMap;

pub struct PciIds {
    entries: HashMap<String, String>,
}

impl PciIds {
    /// Takes in a string containing the contents of the pci.ids file and parses it into a [`PciIds`] struct.
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        let mut entries = HashMap::new();
        let mut cur_vendor = String::new();
        let mut cur_device = String::new();

        for entry in s.lines() {
            // Skipping lines that starts with C because those are device class blocks
            if entry.is_empty() || entry.starts_with('#') || entry.starts_with('C') {
                continue;
            }
            let num_tabs = entry.chars().take_while(|c| *c == '\t').count();
            let text = entry.trim_start();
            let mut tokens = text.split_whitespace();

            match num_tabs {
                // 0 tabs means it's a vendor entry
                0 => {
                    if let Some(id) = tokens.next() {
                        cur_vendor = id.to_string();
                        entries.insert(
                            cur_vendor.clone(),
                            text[id.len()..].trim_start().to_string(),
                        );
                    }
                }
                // 1 tab means it's a device entry
                1 => {
                    if let Some(id) = tokens.next() {
                        cur_device = id.to_string();
                        entries.insert(
                            format!("{}:{}", cur_vendor, cur_device),
                            text[id.len()..].trim_start().to_string(),
                        );
                    }
                }
                // 2 tabs means it's a subdevice entry
                2 => {
                    if let (Some(subven), Some(subdev)) = (tokens.next(), tokens.next()) {
                        let offset = text.find(subdev).unwrap_or(0) + subdev.len();
                        entries.insert(
                            format!("{}:{}:{}:{}", cur_vendor, cur_device, subven, subdev),
                            text[offset..].trim_start().to_string(),
                        );
                    }
                }
                _ => {}
            }
        }

        tracing::debug!("entries: {:?}", entries);
        Ok(Self { entries })
    }

    pub fn lookup(
        &self,
        vendor: &str,
        device: &str,
        subvendor: Option<&str>,
        subdevice: Option<&str>,
    ) -> Option<&str> {
        // vendor:device:subvendor:subdevice
        if let (Some(subvendor), Some(subdevice)) = (subvendor, subdevice) {
            let key = format!("{}:{}:{}:{}", vendor, device, subvendor, subdevice);
            if let Some(name) = self.entries.get(&key) {
                return Some(name);
            }
        }

        // vendor:device
        if let Some(name) = self.entries.get(&format!("{}:{}", vendor, device)) {
            return Some(name);
        }

        // vendor (fallback)
        if let Some(name) = self.entries.get(vendor) {
            return Some(name);
        }

        None
    }
}
