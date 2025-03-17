use monitord_transport::config::TransportType;

#[derive(Debug, Clone, Default)]
pub struct CommunicationConfig {
    pub transport_config: TransportType,
}