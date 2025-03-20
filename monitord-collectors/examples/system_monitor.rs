use futures::{stream, StreamExt};
use monitord_collectors::{
    cpu::{CpuCollector},
    memory::{MemoryCollector},
    config::{CpuCollectorConfig, MemoryCollectorConfig},
    traits::Collector,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();
    
    // Create CPU collector with config
    let cpu_config = CpuCollectorConfig {
        enabled: true,
        interval_ms: 1000,
        collect_per_core: true,
        collect_cache_info: true,
        collect_temperature: true,
        collect_frequency: true,
    };
    let cpu_collector = CpuCollector::new(cpu_config)?;
    
    // Create Memory collector with config
    let memory_config = MemoryCollectorConfig {
        enabled: true,
        interval_ms: 1000,
        collect_dram_info: true,
        collect_swap_info: true,
    };
    let memory_collector = MemoryCollector::new(memory_config)?;
    
    // Create streams with different intervals
    let cpu_interval = Duration::from_secs(2);
    let memory_interval = Duration::from_secs(3);
    
    let mut cpu_stream = cpu_collector.stream(cpu_interval)
        .map(|result| {
            match result {
                Ok(info) => format!(
                    "CPU: {:.2}% utilization, {} cores", 
                    info.global_utilization_percent, 
                    info.logical_cores
                ),
                Err(e) => format!("CPU error: {}", e),
            }
        });
    
    let mut memory_stream = memory_collector.stream(memory_interval)
        .map(|result| {
            match result {
                Ok(info) => {
                    let total_gb = info.total_memory_bytes as f64 / 1_073_741_824.0;
                    let used_gb = info.used_memory_bytes as f64 / 1_073_741_824.0;
                    let memory_percent = info.memory_load_percent;
                    
                    format!(
                        "Memory: {:.2}% used ({:.2} GB / {:.2} GB)", 
                        memory_percent,
                        used_gb,
                        total_gb
                    )
                },
                Err(e) => format!("Memory error: {}", e),
            }
        });
    
    // Merge the streams
    let mut combined_stream = stream::select(cpu_stream, memory_stream);
    
    println!("Starting system monitor, press Ctrl+C to exit");
    println!("=============================================");
    
    // Display the first 10 metrics from either stream
    for _ in 0..10 {
        if let Some(message) = combined_stream.next().await {
            println!("{}", message);
        }
    }
    
    Ok(())
}