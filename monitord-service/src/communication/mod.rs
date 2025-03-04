pub mod config;
pub mod error;
mod grpc;
mod iceoryx;
mod manager;
mod subscription;

pub use manager::CommunicationManager;
