# monitord-client

A Rust client library for interacting with the monitord service. This library provides a convenient interface for fetching system monitoring information from a running monitord service.

## Features

- Connect to a monitord service using gRPC
- Fetch single system snapshots
- Stream continuous system data
- Access individual subsystem metrics (CPU, memory, GPU, network, processes, storage)
- Filter process information

## Usage

```rust
use futures::StreamExt;
use monitord_client::{MonitordClient, ProcessFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the monitord service
    let client = MonitordClient::connect("http://localhost:50051").await?;
    
    // Get a one-time system snapshot
    println!("Fetching a system snapshot...");
    let snapshot = client.get_system_snapshot().await?;
    
    // Print some basic system information
    if let Some(system_info) = &snapshot.system_info {
        println!("System: {} ({})", system_info.hostname, system_info.os_name);
        println!("Kernel: {}", system_info.kernel_version);
        println!("Architecture: {}", system_info.architecture);
    }
    
    // Stream CPU information for 5 updates
    println!("\nStreaming CPU information...");
    let mut cpu_stream = client.stream_cpu_info(1000).await?; // 1000ms interval
    
    while let Some(cpu_info) = cpu_stream.next().await {
        match cpu_info {
            Ok(info) => {
                println!("CPU utilization: {:.1}%", info.global_utilization_percent);
                
                // Print per-core utilization
                for core in &info.core_info {
                    println!("  Core {}: {:.1}% @ {:.0} MHz", 
                             core.core_id, 
                             core.utilization_percent,
                             core.frequency_mhz);
                }
            },
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    
    // Stream process information with filtering
    println!("\nTop 5 CPU-intensive processes:");
    let filter = ProcessFilter {
        sort_by_cpu: true,
        limit: 5,
        ..Default::default()
    };
    
    let mut process_stream = client.stream_process_info(1000, filter).await?;
    
    while let Some(process) = process_stream.next().await {
        match process {
            Ok(info) => {
                println!("PID: {}, Name: {}, CPU: {:.1}%, Memory: {} MB",
                         info.pid,
                         info.name,
                         info.cpu_usage_percent,
                         info.physical_memory_bytes / 1_048_576);
            },
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    
    Ok(())
}
```

## Examples

For more examples, check the `examples` directory:

- `basic_client.rs`: Simple client showing basic usage
- `simple_monitor.rs`: Periodic monitoring of key system metrics
- `system_dashboard.rs`: More complex example showing how to build a simple system dashboard

## License

MIT