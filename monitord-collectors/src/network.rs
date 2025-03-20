use crate::config::NetworkCollectorConfig;
use crate::error::{CollectorError, Result};
use crate::traits::Collector;
use monitord_protocols::monitord::{NetworkInfo, NetworkList};
use std::collections::HashMap;
use sysinfo::Networks;
use tracing::{debug, info};

pub struct NetworkCollector {
    config: NetworkCollectorConfig,
    networks: Networks,
    // Store previous values to calculate rates
    previous_rx: HashMap<String, u64>,
    previous_tx: HashMap<String, u64>,
    previous_rx_packets: HashMap<String, u64>,
    previous_tx_packets: HashMap<String, u64>,
    previous_time: std::time::Instant,
}

impl Collector for NetworkCollector {
    type Data = NetworkList;
    type Config = NetworkCollectorConfig;

    fn new(config: Self::Config) -> Result<Self> {
        debug!("Initializing Network collector with config: {:?}", config);

        if !config.enabled {
            info!("Network collector is disabled");
            return Err(CollectorError::ConfigurationError(
                "Network collector is disabled".into(),
            ));
        }

        let networks = Networks::new_with_refreshed_list();

        // Initialize previous values
        let previous_rx = HashMap::new();
        let previous_tx = HashMap::new();
        let previous_rx_packets = HashMap::new();
        let previous_tx_packets = HashMap::new();

        info!("Network collector initialized");
        Ok(Self {
            config,
            networks,
            previous_rx,
            previous_tx,
            previous_rx_packets,
            previous_tx_packets,
            previous_time: std::time::Instant::now(),
        })
    }

    fn collect(&mut self) -> Result<Self::Data> {
        debug!("Collecting network information");

        // Refresh network information
        self.networks.refresh(true);

        // Calculate time elapsed since last collection
        let now = std::time::Instant::now();
        let elapsed_secs = now.duration_since(self.previous_time).as_secs_f64();
        self.previous_time = now;

        let mut net_infos = Vec::new();

        for (interface_name, network) in self.networks.iter() {
            // Get current values
            let rx_bytes = network.total_received();
            let tx_bytes = network.total_transmitted();
            let rx_packets = network.total_packets_received();
            let tx_packets = network.total_packets_transmitted();

            // Calculate rates
            let rx_bytes_per_sec = if let Some(&prev_rx) = self.previous_rx.get(interface_name) {
                if elapsed_secs > 0.0 {
                    ((rx_bytes - prev_rx) as f64 / elapsed_secs) as u64
                } else {
                    0
                }
            } else {
                0
            };

            let tx_bytes_per_sec = if let Some(&prev_tx) = self.previous_tx.get(interface_name) {
                if elapsed_secs > 0.0 {
                    ((tx_bytes - prev_tx) as f64 / elapsed_secs) as u64
                } else {
                    0
                }
            } else {
                0
            };

            // Calculate packet rates if collection is enabled
            let (rx_packets_per_sec, tx_packets_per_sec) = if self.config.collect_packets {
                let rx_rate = if let Some(&prev_rx) = self.previous_rx_packets.get(interface_name) {
                    if elapsed_secs > 0.0 {
                        ((rx_packets - prev_rx) as f64 / elapsed_secs) as u64
                    } else {
                        0
                    }
                } else {
                    0
                };

                let tx_rate = if let Some(&prev_tx) = self.previous_tx_packets.get(interface_name) {
                    if elapsed_secs > 0.0 {
                        ((tx_packets - prev_tx) as f64 / elapsed_secs) as u64
                    } else {
                        0
                    }
                } else {
                    0
                };

                (rx_rate, tx_rate)
            } else {
                (0, 0)
            };

            // Error statistics - not directly available from sysinfo
            // Would need a platform-specific implementation
            let (rx_errors, tx_errors) = if self.config.collect_errors {
                (0, 0) // Placeholder values
            } else {
                (0, 0)
            };

            // Create NetworkInfo object
            let net_info = NetworkInfo {
                interface_name: interface_name.to_string(),
                driver: "Unknown".to_string(), // Not available from sysinfo
                mac_address: "00:00:00:00:00:00".to_string(), // Not available from sysinfo
                ip_addresses: Vec::new(),      // Not available from sysinfo
                max_bandwidth_bytes_per_sec: 0, // Not available from sysinfo
                rx_bytes_per_sec,
                tx_bytes_per_sec,
                rx_packets_per_sec,
                tx_packets_per_sec,
                rx_errors,
                tx_errors,
                rx_bytes_total: rx_bytes,
                tx_bytes_total: tx_bytes,
                is_up: true,             // Not available from sysinfo
                mtu: 0,                  // Not available from sysinfo
                dns_servers: Vec::new(), // Not available from sysinfo
                link_speed_mbps: None,   // Not available from sysinfo
            };

            net_infos.push(net_info);

            // Update previous values
            self.previous_rx
                .insert(interface_name.to_string(), rx_bytes);
            self.previous_tx
                .insert(interface_name.to_string(), tx_bytes);
            self.previous_rx_packets
                .insert(interface_name.to_string(), rx_packets);
            self.previous_tx_packets
                .insert(interface_name.to_string(), tx_packets);
        }

        debug!(
            "Network information collected for {} interface(s)",
            net_infos.len()
        );
        Ok(NetworkList { nets: net_infos })
    }
}
