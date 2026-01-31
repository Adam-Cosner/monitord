pub mod cpu;
pub mod memory;

mod metrics {
    tonic::include_proto!("metrics");
}
