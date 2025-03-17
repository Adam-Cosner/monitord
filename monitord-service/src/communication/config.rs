use monitord_transport::config::TransportConfig;

#[derive(Debug, Clone, Default)]
pub struct CommunicationConfig {
    pub transport_config: TransportConfig,
}