//! Protobuf implementation of the MessageHandler trait

use prost::Message;
use crate::communication::core::traits::{MessageHandler, MessageType};
use crate::communication::error::CommunicationError;

/// Handler for Protocol Buffers serialization/deserialization
// In handlers/protobuf.rs
#[derive(Debug, Clone, Default)]
pub struct ProtobufHandler;

impl MessageHandler for ProtobufHandler {
    fn serialize_bytes(&self, message_type: MessageType, message_bytes: Vec<u8>)
                       -> Result<Vec<u8>, CommunicationError> {
        // For Protobuf, we can just return the bytes directly
        // In a more complex implementation, we might add headers or other metadata
        Ok(message_bytes)
    }

    fn deserialize_bytes(&self, message_type: MessageType, data: &[u8])
                         -> Result<Vec<u8>, CommunicationError> {
        // For Protobuf, we can just return the bytes directly
        Ok(data.to_vec())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use monitord_protocols::monitord::{CpuInfo, CoreInfo};
    use crate::communication::core::traits::message_utils;

    #[test]
    fn test_protobuf_serialization() {
        let handler = ProtobufHandler;

        // Create a sample CPU info
        let cpu_info = CpuInfo {
            model_name: "Test CPU".to_string(),
            physical_cores: 4,
            logical_cores: 8,
            global_utilization_percent: 25.5,
            core_info: vec![
                CoreInfo {
                    core_id: 0,
                    frequency_mhz: 3600.0,
                    utilization_percent: 30.0,
                    temperature_celsius: 45.0,
                    min_frequency_mhz: Some(1200.0),
                    max_frequency_mhz: Some(4000.0),
                }
            ],
            cache_info: None,
            scaling_governor: None,
            architecture: "x86_64".to_string(),
            cpu_flags: vec!["sse".to_string(), "avx".to_string()],
        };

        // Serialize
        let bytes = message_utils::serialize(&handler, MessageType::CpuInfo, &cpu_info).unwrap();

        // Deserialize
        let deserialized: CpuInfo = message_utils::deserialize(&handler, MessageType::CpuInfo, &bytes).unwrap();

        // Verify
        assert_eq!(deserialized.model_name, "Test CPU");
        assert_eq!(deserialized.physical_cores, 4);
        assert_eq!(deserialized.logical_cores, 8);
        assert_eq!(deserialized.global_utilization_percent, 25.5);
        assert_eq!(deserialized.core_info.len(), 1);
        assert_eq!(deserialized.core_info[0].core_id, 0);
    }
}