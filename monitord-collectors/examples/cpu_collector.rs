use futures::StreamExt;
use monitord_collectors::cpu::{CpuCollector};
use monitord_collectors::config::{CpuCollectorConfig};
use monitord_collectors::traits::Collector;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();
    
    // Create a config with custom settings
    let config = CpuCollectorConfig {
        enabled: true,
        interval_ms: 1000,
        collect_per_core: true,
        collect_cache_info: true,
        collect_temperature: true,
        collect_frequency: true,
    };
    
    // Create a CPU collector
    let cpu_collector = CpuCollector::new(config)?;
    
    // Set the collection interval to 2 seconds
    let interval = Duration::from_secs(2);
    
    // Create a stream that will emit CPU info every 2 seconds
    let mut cpu_stream = cpu_collector.stream(interval);
    
    println!("Starting CPU collector stream, press Ctrl+C to exit");
    println!("==================================================");
    
    // Collect 5 samples
    for _i in 0..5 {
        // Wait for the next data point
        if let Some(cpu_info) = cpu_stream.next().await {
            match cpu_info {
                Ok(info) => {
                    println!("CPU Model: {}", info.model_name);
                    println!("Physical Cores: {}", info.physical_cores);
                    println!("Logical Cores: {}", info.logical_cores);
                    println!("Global Utilization: {:.2}%", info.global_utilization_percent);
                    
                    // Print per-core info
                    for core in &info.core_info {
                        println!(
                            "Core {}: {:.2}% @ {:.0} MHz", 
                            core.core_id, 
                            core.utilization_percent, 
                            core.frequency_mhz
                        );
                    }
                    
                    println!("==================================================");
                },
                Err(e) => eprintln!("Error collecting CPU info: {}", e),
            }
        }
    }
    
    Ok(())
}