/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

mod wifi;

use std::{net::IpAddr, path::Path};

use super::helpers::*;

use crate::collector::helpers::cached::Cached;
#[doc(inline)]
pub use crate::metrics::network::*;

pub struct Collector {
    counters: std::collections::HashMap<String, sample::Sample<Counters>>,
    wifi_reader: Cached<wifi::WifiReader>,
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct Counters {
    rx_bytes: u64,
    tx_bytes: u64,
    rx_packets: u64,
    tx_packets: u64,
    rx_errors: u64,
    tx_errors: u64,
    rx_drops: u64,
    tx_drops: u64,
}

#[derive(Debug)]
struct UsageDiff {
    rx_bytes: u64,
    tx_bytes: u64,
}

impl sample::Diffable for Counters {
    type Delta = UsageDiff;

    fn diff(&self, other: &Self) -> Self::Delta {
        UsageDiff {
            rx_bytes: self.rx_bytes - other.rx_bytes,
            tx_bytes: self.tx_bytes - other.tx_bytes,
        }
    }
}

impl Collector {
    pub fn new() -> Self {
        Self {
            counters: std::collections::HashMap::new(),
            wifi_reader: Cached::default(),
        }
    }

    pub fn collect(&mut self) -> anyhow::Result<Snapshot> {
        let mut adapters = Vec::new();

        for entry in std::fs::read_dir("/sys/class/net")?.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path();

            let counters = read_counters(&path);

            let usage = self.diff_rates(&name, &counters);

            let adapter_type = classify_adapter(&path);
            let wifi = if adapter_type == adapter::AdapterType::Wifi {
                self.wifi_reader
                    .get_or_try_mut(wifi::WifiReader::new)
                    .and_then(|r| {
                        let res = r.read(&name);
                        if let Err(e) = &res {
                            tracing::warn!("Failed to read wifi info for {}: {}", name, e);
                        }
                        res.ok()
                    })
            } else {
                None
            };

            let addresses = get_addresses()?;
            let ipv4_addresses = addresses
                .iter()
                .filter(|a| a.addr.is_ipv4() && a.name == name)
                .map(|a| format!("{}/{}", a.addr, a.prefix_len))
                .collect::<Vec<_>>();
            let ipv6_addresses = addresses
                .iter()
                .filter(|a| a.addr.is_ipv6() && a.name == name)
                .map(|a| format!("{}/{}", a.addr, a.prefix_len))
                .collect::<Vec<_>>();

            adapters.push(Adapter {
                interface_name: name.clone(),
                mac_address: sysfs::read_string(&path.join("address")).unwrap_or_default(),
                ipv4_addresses,
                ipv6_addresses,
                adapter_type: adapter_type as i32,
                mtu: sysfs::read_u32(&path.join("mtu")).unwrap_or_default(),
                is_up: sysfs::read_string(&path.join("operstate")).unwrap_or_default() == "up",
                rx_bytes_total: counters.value.rx_bytes,
                tx_bytes_total: counters.value.tx_bytes,
                rx_packets_total: counters.value.rx_packets,
                tx_packets_total: counters.value.tx_packets,
                rx_errors_total: counters.value.rx_errors,
                tx_errors_total: counters.value.tx_errors,
                rx_drops_total: counters.value.rx_drops,
                tx_drops_total: counters.value.tx_drops,
                rx_bytes_per_second: (usage.delta.rx_bytes as f64 / usage.elapsed.as_secs_f64())
                    as u64,
                tx_bytes_per_second: (usage.delta.tx_bytes as f64 / usage.elapsed.as_secs_f64())
                    as u64,
                wifi_info: wifi,
            });

            self.counters.insert(name, counters);
        }

        Ok(Snapshot { adapters })
    }

    fn diff_rates(&self, name: &str, new: &sample::Sample<Counters>) -> sample::Diff<UsageDiff> {
        if let Some(old) = self.counters.get(name) {
            new - old
        } else {
            sample::Diff {
                delta: UsageDiff {
                    rx_bytes: 0,
                    tx_bytes: 0,
                },
                elapsed: std::time::Duration::from_secs(0),
            }
        }
    }
}

fn read_counters(path: &Path) -> sample::Sample<Counters> {
    sample::Sample::new(Counters {
        rx_bytes: sysfs::read_u64(&path.join("statistics/rx_bytes")).unwrap_or_default(),
        tx_bytes: sysfs::read_u64(&path.join("statistics/tx_bytes")).unwrap_or_default(),
        rx_packets: sysfs::read_u64(&path.join("statistics/rx_packets")).unwrap_or_default(),
        tx_packets: sysfs::read_u64(&path.join("statistics/tx_packets")).unwrap_or_default(),
        rx_errors: sysfs::read_u64(&path.join("statistics/rx_errors")).unwrap_or_default(),
        tx_errors: sysfs::read_u64(&path.join("statistics/tx_errors")).unwrap_or_default(),
        rx_drops: sysfs::read_u64(&path.join("statistics/rx_dropped")).unwrap_or_default(),
        tx_drops: sysfs::read_u64(&path.join("statistics/tx_dropped")).unwrap_or_default(),
    })
}

fn classify_adapter(path: &Path) -> adapter::AdapterType {
    if path.join("wireless").exists() || path.join("phy80211").exists() {
        adapter::AdapterType::Wifi
    } else if path.join("bridge").exists() {
        adapter::AdapterType::Bridge
    } else {
        match sysfs::read_u32(&path.join("type")) {
            Some(772) => adapter::AdapterType::Loopback,
            Some(1) => adapter::AdapterType::Ethernet,
            Some(65534) | Some(768) | Some(776) => adapter::AdapterType::Virtual,
            _ => adapter::AdapterType::Unknown,
        }
    }
}

struct IfAddr {
    name: String,
    addr: IpAddr,
    prefix_len: u8,
}

fn get_addresses() -> anyhow::Result<Vec<IfAddr>> {
    let mut result = Vec::new();

    for ifa in nix::ifaddrs::getifaddrs()? {
        let Some(addr) = ifa.address else { continue };
        let netmask = ifa.netmask;

        match addr.as_sockaddr_in() {
            Some(v4) => {
                let ip = IpAddr::from(v4.ip());
                let prefix = netmask
                    .as_ref()
                    .and_then(|m| m.as_sockaddr_in())
                    .map(|m| u32::from(m.ip()).count_ones())
                    .unwrap_or(0);
                result.push(IfAddr {
                    name: ifa.interface_name,
                    addr: ip,
                    prefix_len: prefix as u8,
                });
            }
            None => {
                if let Some(v6) = addr.as_sockaddr_in6() {
                    let ip = IpAddr::from(v6.ip());
                    let prefix = netmask
                        .as_ref()
                        .and_then(|m| m.as_sockaddr_in6())
                        .map(|m| m.ip().octets().iter().map(|b| b.count_ones()).sum())
                        .unwrap_or(0);
                    result.push(IfAddr {
                        name: ifa.interface_name,
                        addr: ip,
                        prefix_len: prefix as u8,
                    });
                }
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network() -> anyhow::Result<()> {
        tracing_subscriber::fmt::init();
        let mut collector = Collector::new();
        let _ = collector.collect()?;
        std::thread::sleep(std::time::Duration::from_secs(1));
        let snapshot = collector.collect()?;
        assert!(!snapshot.adapters.is_empty());
        println!("{:#?}", snapshot);

        Ok(())
    }
}
