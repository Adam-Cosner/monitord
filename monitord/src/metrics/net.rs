use crate::error::Result;

pub struct NetMetricCollector {
    nets: sysinfo::Networks,
}

impl NetMetricCollector {
    pub fn new() -> Result<Self> {
        Ok(Self {
            nets: sysinfo::Networks::new_with_refreshed_list(),
        })
    }

    pub fn collect(
        &mut self,
        request: &monitord_types::service::NetworkRequest,
    ) -> Result<Vec<monitord_types::service::NetworkResponse>> {
        self.nets.refresh(true);
        let mut net_metrics = Vec::new();

        // exclude the loopback device
        for (name, network) in self.nets.list().iter().filter(|net| net.0 != "lo") {
            let received = if request.received {
                network.total_received()
            } else {
                0
            };

            let receiving = if request.receiving {
                network.received()
            } else {
                0
            };

            let sent = if request.sent {
                network.total_transmitted()
            } else {
                0
            };

            let sending = if request.sending {
                network.transmitted()
            } else {
                0
            };

            let ipv4_address = if request.ipv4_address {
                network
                    .ip_networks()
                    .iter()
                    .filter(|net| net.addr.is_ipv4())
                    .next()
                    .map(|net| {
                        if let std::net::IpAddr::V4(ipv4) = net.addr {
                            ipv4.to_string()
                        } else {
                            unreachable!()
                        }
                    })
                    .unwrap_or_default()
            } else {
                "".to_string()
            };

            let signal_strength = 0.0;
            tracing::debug!("Signal strength not yet implemented");

            net_metrics.push(monitord_types::service::NetworkResponse {
                name: name.clone(),
                received,
                receiving,
                sent,
                sending,
                ipv4_address,
                signal_strength,
            })
        }

        Ok(net_metrics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_net_metrics() -> Result<()> {
        let request = monitord_types::service::NetworkRequest {
            received: true,
            receiving: true,
            sent: true,
            sending: true,
            ipv4_address: true,
            signal_strength: true,
        };

        let mut metric_cache = NetMetricCollector::new()?;
        let _ = metric_cache.collect(&request)?;
        // pause to allow second capture for accurate rates
        std::thread::sleep(std::time::Duration::from_secs(1));
        let net_metrics = metric_cache.collect(&request)?;

        println!("{:?}", net_metrics);

        Ok(())
    }
}
