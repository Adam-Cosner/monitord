# Monitord Collectors

This crate provides a set of asynchronous collectors for system metrics. Each collector implements the `Collector` trait, which allows it to be used as a futures `Stream` to produce a continuous feed of metrics.

## Features

- Async-first design with Futures Stream API
- Configurable collection intervals for each metric type
- Runtime-agnostic (works with tokio, async-std, etc.)
- Modular architecture allowing use of individual collectors
- Low overhead and efficient resource utilization
- Comprehensive system metrics collection

## Supported Metrics

- **CPU**: Utilization, frequencies, core-specific metrics, cache info
- **Memory**: RAM usage, swap, detailed memory stats
- **GPU**: NVIDIA support with detailed GPU metrics (AMD and Intel planned)
- **Network**: Interface statistics, bandwidth, packets
- **Storage**: Disk usage, I/O statistics, filesystem info
- **Process**: Per-process resource usage, command line, environment
- **System**: General system information, load averages, uptime

## Usage

### Basic Example

```rust
use futures::StreamExt;
use monitord_collectors::{
    cpu::CpuCollector,
    config::CpuCollectorConfig,
    traits::Collector,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a collector with default configuration
    let config = CpuCollectorConfig::default();
    let cpu_collector = CpuCollector::new(config)?;
    
    // Create a stream that produces values every 2 seconds
    let mut cpu_stream = cpu_collector.stream(Duration::from_secs(2));
    
    // Consume 5 values from the stream
    for _ in 0..5 {
        if let Some(result) = cpu_stream.next().await {
            match result {
                Ok(cpu_info) => println!("CPU utilization: {:.2}%", cpu_info.global_utilization_percent),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    }
    
    Ok(())
}
```

### Using Multiple Collectors

```rust
use futures::{stream, StreamExt};
use monitord_collectors::{
    cpu::CpuCollector, 
    memory::MemoryCollector,
    config::{CpuCollectorConfig, MemoryCollectorConfig},
    traits::Collector,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create collectors with different configurations
    let cpu_collector = CpuCollector::new(CpuCollectorConfig::default())?;
    let memory_collector = MemoryCollector::new(MemoryCollectorConfig::default())?;
    
    // Create streams with different intervals
    let cpu_stream = cpu_collector.stream(Duration::from_secs(2))
        .map(|r| r.map(|i| format!("CPU: {:.2}%", i.global_utilization_percent)).unwrap_or_else(|e| e.to_string()));
    
    let memory_stream = memory_collector.stream(Duration::from_secs(3))
        .map(|r| r.map(|i| format!("Memory: {:.2}%", i.memory_load_percent)).unwrap_or_else(|e| e.to_string()));
    
    // Merge the streams
    let mut combined_stream = stream::select(cpu_stream, memory_stream);
    
    // Consume from the combined stream
    while let Some(message) = combined_stream.next().await {
        println!("{}", message);
    }
    
    Ok(())
}
```

### Creating a Full System Snapshot

See the `full_snapshot.rs` example for how to collect a complete system snapshot with all metrics.

## Configuration

Each collector has its own configuration type that allows customizing:

- Collection interval
- Which specific metrics to collect
- Collection thresholds and limits
- Feature toggles

## Examples

The `examples/` directory contains several usage examples:

- `cpu_collector.rs`: Basic CPU metrics collection
- `system_monitor.rs`: Combined CPU and memory monitoring
- `full_snapshot.rs`: Complete system snapshot with all metrics

Run an example with:

```
cargo run --example cpu_collector
```