use tracing::debug;
use crate::error::CollectionError;
use monitord_protocols::monitord::NetworkInfo;

pub mod config;

pub struct NetworkCollector {
    config: config::NetworkCollectorConfig,
    nets: sysinfo::Networks,
}

impl NetworkCollector {
    pub fn new(config: config::NetworkCollectorConfig) -> Result<Self, CollectionError> {
        Ok(Self {
            config,
            nets: sysinfo::Networks::new_with_refreshed_list(),
        })
    }
}

impl super::Collector for NetworkCollector {
    type CollectedData = Vec<NetworkInfo>;
    type CollectorConfig = config::NetworkCollectorConfig;

    fn name(&self) -> &'static str {
        "network"
    }

    fn config(&self) -> &Self::CollectorConfig {
        &self.config
    }

    fn collect(&mut self) -> Result<Self::CollectedData, CollectionError> {
        if !self.config.enabled {
            return Err(CollectionError::Disabled);
        }
        debug!("Collecting network information");
        self.nets.refresh(true);

        let mut networks = Vec::new();
        for (interface_name, data) in self.nets.iter() {
            networks.push(NetworkInfo {
                interface_name: interface_name.clone(),
                driver: "".to_string(),
                mac_address: data.mac_address().to_string(),
                ip_addresses: data
                    .ip_networks()
                    .iter()
                    .map(|ip| ip.addr.to_string())
                    .collect(),
                max_bandwidth_bytes_per_sec: 0, // not provided by sysinfo
                rx_bytes_per_sec: data.received(),
                tx_bytes_per_sec: data.transmitted(),
                rx_packets_per_sec: data.packets_received(),
                tx_packets_per_sec: data.packets_transmitted(),
                rx_errors: data.errors_on_received(),
                tx_errors: data.errors_on_transmitted(),
                rx_bytes_total: data.total_received(),
                tx_bytes_total: data.total_transmitted(),
                is_up: true, // not provided by sysinfo
                mtu: data.mtu() as u32,
                dns_servers: vec![],   // not provided by sysinfo
                link_speed_mbps: None, // not provided by sysinfo
            })
        }
        Ok(networks)
    }
}
