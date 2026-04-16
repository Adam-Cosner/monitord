/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Network collector module.
//! Collects network adapter information and Wi-Fi information (if using wireless).
//!
//! # Example
//!
//! ```no_run
//!
//! ```
mod wifi;

use super::{
    helpers::{
        discovery::Discovery,
        sampler::{Differential, Sampler},
        *,
    },
    staging,
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

impl super::Collector for Collector {
    type Output = Snapshot;

    fn name(&self) -> &'static str {
        "net"
    }

    /// Collects one full snapshot of network adapters and emplaces it into the associated Store slot.
    /// If collection fails critically, the store slot is not modified and an error is returned.
    /// On non-critical errors, the store slot is emplaced with empty data and a warning is logged.
    fn collect(&mut self, config: &crate::metrics::Config) -> anyhow::Result<Self::Output> {
        self.collect_adapters(config.network.as_ref())
            .inspect_err(|e| tracing::error!("collector failed: {e}"))
    }
}

impl Collector {
    pub fn new() -> Self {
        Self {
            counters: std::collections::HashMap::new(),
            wifi_reader: Discovery::default(),
        }
    }

    fn collect_adapters(&mut self, config: Option<&Config>) -> anyhow::Result<Snapshot> {
        let Some(config) = config else {
            anyhow::bail!("no config supplied to collector")
        };
        let addresses = get_addresses()?;
        match std::fs::read_dir("/sys/class/net") {
            Ok(dir) => {
                let mut adapters = Vec::new();

                for interface in dir.flatten() {
                    let interface_name = interface.file_name().to_string_lossy().into_owned();
                    let interface_path = interface.path();
                    adapters.push(self.build_adapter(
                        config,
                        &interface_name,
                        &interface_path,
                        &addresses,
                    ));
                }

                Ok(Snapshot { adapters })
            }
            Err(e) => {
                tracing::warn!("unable to read /sys/class/net: {}", e);
                Ok(Snapshot::default())
            }
        }
    }

    fn build_adapter(
        &mut self,
        config: &Config,
        name: &str,
        path: &Path,
        addresses: &[IfAddr],
    ) -> Adapter {
        let ipv4_addresses = config
            .addresses
            .then(|| get_ipv4_addresses(addresses, name))
            .unwrap_or_default();
        let ipv6_addresses = config
            .addresses
            .then(|| get_ipv6_addresses(addresses, name))
            .unwrap_or_default();
        let adapter_type = classify_adapter(path);
        let is_up = sysfs::read_string(&path.join("operstate"))
            .map(|s| s == "up")
            .unwrap_or(false);
        let packet_counters = Counters::read(path);
        let counter_delta = self
            .counters
            .entry(name.to_string())
            .or_insert_with(Sampler::new)
            .push(packet_counters.clone());
        let wifi = config
            .wifi_info
            .then(|| self.read_wifi(adapter_type, is_up, name))
            .flatten();
        Adapter {
            interface_name: name.to_string(),
            mac_address: sysfs::read_string(&path.join("address")).unwrap_or_default(),
            ipv4_addresses,
            ipv6_addresses,
            adapter_type: adapter_type as i32,
            mtu: sysfs::read_u32(&path.join("mtu")).unwrap_or_default(),
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
                .map(|delta| (delta.change.rx_bytes as f64 / delta.interval.as_secs_f64()) as u64)
                .unwrap_or_default(),
            tx_bytes_per_second: counter_delta
                .map(|delta| (delta.change.tx_bytes as f64 / delta.interval.as_secs_f64()) as u64)
                .unwrap_or_default(),
            wifi_info: wifi,
        }
    }

    fn read_wifi(
        &mut self,
        adapter_type: adapter::AdapterType,
        is_up: bool,
        name: &str,
    ) -> Option<WifiInfo> {
        if adapter_type == adapter::AdapterType::Wifi && is_up {
            self.wifi_reader
                .probe_mut(wifi::WifiReader::new)
                .and_then(|reader| match reader.read(name) {
                    Ok(wifi_info) => Some(wifi_info),
                    Err(e) => {
                        tracing::warn!("failed to read wifi info for {}: {}", name, e);
                        None
                    }
                })
        } else {
            None
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
            rx_bytes: self.rx_bytes.wrapping_sub(previous.rx_bytes),
            tx_bytes: self.tx_bytes.wrapping_sub(previous.tx_bytes),
        }
    }
}

#[derive(Debug)]
struct CounterDelta {
    rx_bytes: u64,
    tx_bytes: u64,
}

const ARPHRD_ETHER: u32 = 1;
const ARPHRD_TUNNEL: u32 = 768;
const ARPHRD_LOOPBACK: u32 = 772;
const ARPHRD_SIT: u32 = 776;
const ARPHRD_NONE: u32 = 65534;

fn classify_adapter(path: &Path) -> adapter::AdapterType {
    if path.join("wireless").exists() || path.join("phy80211").exists() {
        adapter::AdapterType::Wifi
    } else if path.join("bridge").exists() {
        adapter::AdapterType::Bridge
    } else {
        match sysfs::read_u32(&path.join("type")) {
            Some(ARPHRD_LOOPBACK) => adapter::AdapterType::Loopback,
            Some(ARPHRD_ETHER) => adapter::AdapterType::Ethernet,
            Some(ARPHRD_NONE) | Some(ARPHRD_TUNNEL) | Some(ARPHRD_SIT) => {
                adapter::AdapterType::Virtual
            }
            _ => adapter::AdapterType::Unknown,
        }
    }
}

fn get_ipv4_addresses(addresses: &[IfAddr], name: &str) -> Vec<String> {
    addresses
        .iter()
        .filter(|a| a.addr.is_ipv4() && a.name == name)
        .map(|a| format!("{}/{}", a.addr, a.prefix_len))
        .collect::<Vec<_>>()
}

fn get_ipv6_addresses(addresses: &[IfAddr], name: &str) -> Vec<String> {
    addresses
        .iter()
        .filter(|a| a.addr.is_ipv6() && a.name == name)
        .map(|a| format!("{}/{}", a.addr, a.prefix_len))
        .collect::<Vec<_>>()
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
        let _ = tracing_subscriber::fmt::try_init();
        let mut collector = super::Collector::new();
        let mut config = crate::metrics::Config::default();
        config.network = Some(Config {
            addresses: true,
            wifi_info: true,
        });
        let _ = collector.collect(&config)?;
        std::thread::sleep(std::time::Duration::from_secs(1));
        let snapshot = collector.collect(&config)?;
        println!("{:#?}", snapshot);
        Ok(())
    }
}
