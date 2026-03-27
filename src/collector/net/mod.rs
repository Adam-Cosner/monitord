/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Network collector module.
//! Collects network adapter information and Wi-Fi information (if using wireless).
mod wifi;

use super::{
    helpers::{
        discovery::Discovery,
        sampler::{Differential, Sampler},
        *,
    },
    store,
};
use std::{net::IpAddr, path::Path};

#[doc(inline)]
pub use crate::metrics::network::*;

/// Network collector
pub struct Collector {
    /// Map of network adapter names to its tx/rx counters
    counters: std::collections::HashMap<String, Sampler<Counters>>,
    /// Wi-Fi reader wrapped in a `Discovery` lazy-init wrapper
    wifi_reader: Discovery<wifi::WifiReader>,
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}


impl Collector {
    pub fn new() -> Self {
        Self {
            counters: std::collections::HashMap::new(),
            wifi_reader: Discovery::default(),
        }
    }

    fn collect_adapters(&mut self) -> anyhow::Result<Snapshot> {
        match std::fs::read_dir("/sys/class/net") {
            Ok(dir) => {
                let mut adapters = Vec::new();

                for interface in dir.flatten() {
                    let interface_name = interface.file_name().to_string_lossy().into_owned();
                    let interface_path = interface.path();

                    let packet_counters = Counters::read(&interface_path);
                    let counter_delta = self
                        .counters
                        .entry(interface_name.clone())
                        .or_insert_with(Sampler::new)
                        .push(packet_counters.clone());
                    let is_up = sysfs::read_string(&interface_path.join("operstate"))
                        .map(|s| s == "up")
                        .unwrap_or(false);
                    let adapter_type = classify_adapter(&interface_path);
                    let wifi = if adapter_type == adapter::AdapterType::Wifi && is_up {
                        self.wifi_reader
                            .probe_mut(wifi::WifiReader::new)
                            .and_then(|reader| match reader.read(&interface_name) {
                                Ok(wifi_info) => Some(wifi_info),
                                Err(e) => {
                                    tracing::warn!(
                                        "[NET] Failed to read wifi info for {}: {}",
                                        interface_name,
                                        e
                                    );
                                    None
                                }
                            })
                    } else {
                        None
                    };

                    let addresses = get_addresses()?;
                    let ipv4_addresses = addresses
                        .iter()
                        .filter(|a| a.addr.is_ipv4() && a.name == interface_name)
                        .map(|a| format!("{}/{}", a.addr, a.prefix_len))
                        .collect::<Vec<_>>();
                    let ipv6_addresses = addresses
                        .iter()
                        .filter(|a| a.addr.is_ipv6() && a.name == interface_name)
                        .map(|a| format!("{}/{}", a.addr, a.prefix_len))
                        .collect::<Vec<_>>();

                    adapters.push(Adapter {
                        interface_name,
                        mac_address: sysfs::read_string(&interface_path.join("address"))
                            .unwrap_or_default(),
                        ipv4_addresses,
                        ipv6_addresses,
                        adapter_type: adapter_type as i32,
                        mtu: sysfs::read_u32(&interface_path.join("mtu")).unwrap_or_default(),
                        is_up,
                        rx_bytes_total: packet_counters.rx_bytes,
                        tx_bytes_total: packet_counters.tx_bytes,
                        rx_packets_total: packet_counters.rx_packets,
                        tx_packets_total: packet_counters.tx_packets,
                        rx_errors_total: packet_counters.rx_errors,
                        tx_errors_total: packet_counters.tx_errors,
                        rx_drops_total: packet_counters.rx_drops,
                        tx_drops_total: packet_counters.tx_drops,
                        rx_bytes_per_second: counter_delta
                            .as_ref()
                            .map(|delta| {
                                (delta.change.rx_bytes as f64 / delta.interval.as_secs_f64()) as u64
                            })
                            .unwrap_or_default(),
                        tx_bytes_per_second: counter_delta
                            .map(|delta| {
                                (delta.change.tx_bytes as f64 / delta.interval.as_secs_f64()) as u64
                            })
                            .unwrap_or_default(),
                        wifi_info: wifi,
                    });
                }

                Ok(Snapshot { adapters })
            }
            Err(e) => {
                use crate::collector::Collector;
                tracing::warn!("[{}] unable to read /sys/class/net: {}", self.name(), e);
                Ok(Snapshot::default())
            }
        }
    }
}

#[derive(Debug, Clone)]
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

impl Counters {
    fn read(path: &Path) -> Self {
        Self {
            rx_bytes: sysfs::read_u64(&path.join("statistics/rx_bytes")).unwrap_or_default(),
            tx_bytes: sysfs::read_u64(&path.join("statistics/tx_bytes")).unwrap_or_default(),
            rx_packets: sysfs::read_u64(&path.join("statistics/rx_packets")).unwrap_or_default(),
            tx_packets: sysfs::read_u64(&path.join("statistics/tx_packets")).unwrap_or_default(),
            rx_errors: sysfs::read_u64(&path.join("statistics/rx_errors")).unwrap_or_default(),
            tx_errors: sysfs::read_u64(&path.join("statistics/tx_errors")).unwrap_or_default(),
            rx_drops: sysfs::read_u64(&path.join("statistics/rx_dropped")).unwrap_or_default(),
            tx_drops: sysfs::read_u64(&path.join("statistics/tx_dropped")).unwrap_or_default(),
        }
    }
}

impl Differential for Counters {
    type Delta = CounterDelta;

    fn delta(&self, previous: &Self) -> Self::Delta {
        CounterDelta {
            rx_bytes: self.rx_bytes - previous.rx_bytes,
            tx_bytes: self.tx_bytes - previous.tx_bytes,
        }
    }
}

#[derive(Debug)]
struct CounterDelta {
    rx_bytes: u64,
    tx_bytes: u64,
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
    use crate::collector::Collector;

    #[test]
    fn network() -> anyhow::Result<()> {
        tracing_subscriber::fmt::init();
        let mut collector = super::Collector::new();
        let mut store = store::Store::new();
        collector.collect(&store)?;
        std::thread::sleep(std::time::Duration::from_secs(1));
        store = store::Store::new();
        collector.collect(&store)?;
        assert!(store.net.get().is_some_and(|n| !n.adapters.is_empty()));
        println!("{:#?}", store.net.get());

        Ok(())
    }
}
