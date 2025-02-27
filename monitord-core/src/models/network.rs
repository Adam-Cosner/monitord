/// Network interface model
use super::Model;
use crate::error::ModelError;
use monitord_protocols::monitord::NetworkInfo as ProtoNetworkInfo;

#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub interface_name: String,
    pub driver: String,
    pub mac_address: String,
    pub ip_addresses: Vec<String>,
    pub max_bandwidth_bytes_per_sec: u64,
    pub rx_bytes_per_sec: u64,
    pub tx_bytes_per_sec: u64,
    pub rx_packets_per_sec: u64,
    pub tx_packets_per_sec: u64,
    pub rx_errors: u64,
    pub tx_errors: u64,
    pub rx_bytes_total: u64,
    pub tx_bytes_total: u64,
    
    // Additional fields not in proto
    pub is_up: bool,
    pub mtu: u32,
    pub dns_servers: Vec<String>,
    pub link_speed_mbps: Option<u32>,
}

impl Model for NetworkInfo {
    type ProtoType = ProtoNetworkInfo;

    fn into_proto(self) -> Self::ProtoType {
        ProtoNetworkInfo {
            interface_name: self.interface_name,
            driver: self.driver,
            mac_address: self.mac_address,
            ip_addresses: self.ip_addresses,
            max_bandwidth_bytes_per_sec: self.max_bandwidth_bytes_per_sec,
            rx_bytes_per_sec: self.rx_bytes_per_sec,
            tx_bytes_per_sec: self.tx_bytes_per_sec,
            rx_packets_per_sec: self.rx_packets_per_sec,
            tx_packets_per_sec: self.tx_packets_per_sec,
            rx_errors: self.rx_errors,
            tx_errors: self.tx_errors,
            rx_bytes_total: self.rx_bytes_total,
            tx_bytes_total: self.tx_bytes_total,
        }
    }

    fn from_proto(proto: Self::ProtoType) -> Self {
        Self {
            interface_name: proto.interface_name,
            driver: proto.driver,
            mac_address: proto.mac_address,
            ip_addresses: proto.ip_addresses,
            max_bandwidth_bytes_per_sec: proto.max_bandwidth_bytes_per_sec,
            rx_bytes_per_sec: proto.rx_bytes_per_sec,
            tx_bytes_per_sec: proto.tx_bytes_per_sec,
            rx_packets_per_sec: proto.rx_packets_per_sec,
            tx_packets_per_sec: proto.tx_packets_per_sec,
            rx_errors: proto.rx_errors,
            tx_errors: proto.tx_errors,
            rx_bytes_total: proto.rx_bytes_total,
            tx_bytes_total: proto.tx_bytes_total,
            
            // Initialize additional fields
            is_up: true,
            mtu: 0,
            dns_servers: Vec::new(),
            link_speed_mbps: None,
        }
    }

    fn validate(&self) -> Result<(), ModelError> {
        // Basic mac address format validation (extremely simple check)
        if !self.mac_address.is_empty() && !self.mac_address.contains(':') {
            return Err(ModelError::Validation(
                "Invalid MAC address format".to_owned(),
            ));
        }
        
        // Validate IP addresses (very basic check)
        for ip in &self.ip_addresses {
            if !ip.contains('.') && !ip.contains(':') {
                return Err(ModelError::Validation(
                    format!("Invalid IP address format: {}", ip),
                ));
            }
        }

        Ok(())
    }
}

// Additional helper methods
impl NetworkInfo {
    /// Calculate total bandwidth usage as a percentage of max bandwidth
    pub fn bandwidth_utilization_percent(&self) -> f64 {
        if self.max_bandwidth_bytes_per_sec == 0 {
            return 0.0;
        }
        
        let total_usage = self.rx_bytes_per_sec + self.tx_bytes_per_sec;
        (total_usage as f64 / self.max_bandwidth_bytes_per_sec as f64) * 100.0
    }
    
    /// Check if the interface is experiencing high errors
    pub fn has_high_error_rate(&self) -> bool {
        let total_packets = self.rx_packets_per_sec + self.tx_packets_per_sec;
        if total_packets == 0 {
            return false;
        }
        
        let total_errors = self.rx_errors + self.tx_errors;
        (total_errors as f64 / total_packets as f64) > 0.01 // Error rate above 1%
    }
    
    /// Get the interface type based on its name
    pub fn interface_type(&self) -> &'static str {
        if self.interface_name.starts_with("wl") {
            "wireless"
        } else if self.interface_name.starts_with("en") || self.interface_name.starts_with("eth") {
            "ethernet"
        } else if self.interface_name.starts_with("ww") {
            "wwan"
        } else if self.interface_name.starts_with("lo") {
            "loopback"
        } else if self.interface_name.starts_with("br") {
            "bridge"
        } else if self.interface_name.starts_with("tun") || self.interface_name.starts_with("tap") {
            "virtual"
        } else {
            "unknown"
        }
    }
}
