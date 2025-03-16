//! Common utilities for transport implementations

use crate::communication::core::models::DataType;

/// Utility for formatting topic names consistently
#[derive(Debug, Clone)]
pub struct TopicFormatter {
    service_prefix: String,
}

impl TopicFormatter {
    /// Create a new topic formatter with service prefix
    pub fn new(service_name: &str) -> Self {
        Self {
            service_prefix: service_name.to_string(),
        }
    }

    /// Format a topic for a data type and subscription ID
    pub fn format_data_topic(&self, data_type: DataType, subscription_id: &str) -> String {
        format!("{}/{}/{}", self.service_prefix, data_type, subscription_id)
    }

    /// Format a topic for client responses
    pub fn format_response_topic(&self, response_type: &str, client_id: &str) -> String {
        format!("{}/{}/{}", self.service_prefix, response_type, client_id)
    }

    /// Format a connection request topic
    pub fn format_connection_topic(&self, direction: &str) -> String {
        format!("{}/connection/{}", self.service_prefix, direction)
    }
}

/// Common utilities for transports
pub struct TransportUtils;

impl TransportUtils {
    /// Extract client ID from a subscription ID
    pub fn extract_client_id(subscription_id: &str) -> Option<&str> {
        subscription_id.split('-').next()
    }

    /// Generate a subscription ID from client ID and a unique identifier
    pub fn generate_subscription_id(client_id: &str, unique_id: &str) -> String {
        format!("{}-{}", client_id, unique_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_formatter() {
        let formatter = TopicFormatter::new("monitord");

        assert_eq!(
            formatter.format_data_topic(DataType::Cpu, "client1-sub1"),
            "monitord/cpu/client1-sub1"
        );

        assert_eq!(
            formatter.format_response_topic("subscribe", "client1"),
            "monitord/subscribe/client1"
        );

        assert_eq!(
            formatter.format_connection_topic("incoming"),
            "monitord/connection/incoming"
        );
    }

    #[test]
    fn test_transport_utils() {
        assert_eq!(
            TransportUtils::extract_client_id("client1-sub1"),
            Some("client1")
        );

        assert_eq!(
            TransportUtils::generate_subscription_id("client1", "sub1"),
            "client1-sub1"
        );
    }
}
