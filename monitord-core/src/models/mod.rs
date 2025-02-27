//! The model trait is for the representation of data outside of its serialized form
use crate::error::ModelError;

pub trait Model {
    type ProtoType;

    /// Convert this model into its protocol buffer representation
    fn into_proto(self) -> Self::ProtoType;

    /// Convert a model from its protobuf representation into the model
    fn from_proto(proto: Self::ProtoType) -> Self;

    /// Validate that the model contains sensible values
    fn validate(&self) -> Result<(), ModelError>;
}

pub trait CollectorData {
    /// The corresponding model type
    type ModelType: Model;

    fn into_model(self) -> Self::ModelType;
}

pub mod cpu;
pub mod gpu;
pub mod memory;
pub mod network;
pub mod process;
pub mod storage;
pub mod system;
