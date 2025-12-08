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

    pub fn collect(&mut self) -> Result<Vec<monitord_types::service::NetworkResponse>> {
        self.nets.refresh(true);
        let mut net_metrics = Vec::new();

        // exclude the loopback device
        for (name, network) in self.nets.list().iter().filter(|net| net.0 != "lo") {
            let received = network.total_received();
            let receiving = network.received();
            let sent = network.total_transmitted();
            let sending = network.transmitted();
            let ipv4_address = network
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
                .unwrap_or_default();
            // TODO: Signal strength
            let signal_strength = 0.0;

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
        let mut metric_cache = NetMetricCollector::new()?;
        let _ = metric_cache.collect()?;
        // pause to allow second capture for accurate rates
        std::thread::sleep(std::time::Duration::from_secs(1));
        let net_metrics = metric_cache.collect()?;

        println!("{:?}", net_metrics);

        Ok(())
    }
}
