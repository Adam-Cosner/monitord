# Monitord

A comprehensive system monitoring service for Linux, macOS, and Windows systems.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

Monitord is a high-performance system monitoring daemon that provides real-time system metrics and statistics. It collects information about CPU, memory, storage, network, and other system components, making this data available through various transport mechanisms.

Key features:
- Cross-platform support (Linux, macOS, Windows)
- Low overhead monitoring
- Multiple transport mechanisms (IPC, gRPC, etc.)
- Configurable collection intervals
- System service integration

## Architecture

Monitord consists of several components:

- **monitord-service**: The main daemon that runs as a system service
- **monitord-collectors**: Data collection modules for system metrics
- **monitord-transport**: Communication transport layer
- **monitord-protocols**: Protocol definitions and data structures

## Installation

### Prerequisites

- Rust toolchain (1.70+)
- Just command runner (`cargo install just`)
- System development libraries (varies by platform)

### Building from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/monitord.git
   cd monitord
   ```

2. Build the project:
   ```bash
   just build
   ```

### Installing as a System Service

```bash
just install
just register-service
```

This will install the monitord binary to `/usr/local/bin/monitord` and register it as a system service.

## Usage

### Service Management

Start the service:
```bash
just start
```

Check service status:
```bash
just status
```

Stop the service:
```bash
just stop
```

Restart the service:
```bash
just restart
```

### Development Mode

Run in development mode with standard logging:
```bash
just run-dev
```

Run with debug logging:
```bash
just run-debug
```

### Uninstalling

```bash
just uninstall
```

## Client Interface

### Connecting to Monitord

Monitord uses a transport layer that abstracts the communication details. Here's how to connect to the service:

```rust
use monitord_protocols::monitord::*;
use monitord_transport::config::TransportConfig;
use monitord_transport::TransportManager;

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the transport with default configuration
    let mut transport = TransportManager::new(TransportConfig::default())?;
    
    // Initialize the connection
    transport.initialize().await?;
    
    // Now you can receive data from various collectors
    if let Some(cpu_info) = transport.receive::<CpuInfo>("cpu").await? {
        println!("CPU Model: {}", cpu_info.model_name);
        println!("CPU Usage: {}%", cpu_info.global_utilization_percent);
        println!("Logical Cores: {}", cpu_info.logical_cores);
        // ... and much more
    }
    
    // Get memory information
    if let Some(memory_info) = transport.receive::<MemoryInfo>("memory").await? {
        let total_gb = memory_info.total_memory_bytes as f64 / 1_073_741_824.0;
        let used_gb = memory_info.used_memory_bytes as f64 / 1_073_741_824.0;
        
        println!("Memory Usage: {:.2}% ({:.2} GB / {:.2} GB)", 
                 memory_info.memory_load_percent,
                 used_gb,
                 total_gb);
    }
    
    // Get system information
    if let Some(system_info) = transport.receive::<SystemInfo>("system").await? {
        println!("Hostname: {}", system_info.hostname);
        println!("OS: {} {}", system_info.os_name, system_info.os_version);
        println!("Uptime: {} seconds", system_info.uptime_seconds);
    }
    
    Ok(())
}
```

### Creating Custom Collectors

You can also create custom collectors that integrate with the monitord framework:

```rust
use monitord_collectors::{
    config::CpuCollectorConfig,
    cpu::CpuCollector,
    traits::Collector,
};
use std::time::Duration;

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a CPU collector with custom configuration
    let cpu_config = CpuCollectorConfig {
        enabled: true,
        interval_ms: 1000,
        collect_per_core: true,
        collect_cache_info: true,
        collect_temperature: true,
        collect_frequency: true,
    };
    
    let cpu_collector = CpuCollector::new(cpu_config)?;
    
    // Create a stream that produces CPU metrics every 2 seconds
    let interval = Duration::from_secs(2);
    let mut cpu_stream = cpu_collector.stream(interval);
    
    // Process the stream
    while let Some(result) = cpu_stream.next().await {
        match result {
            Ok(info) => {
                println!("CPU: {:.2}% utilization, {} cores", 
                    info.global_utilization_percent, 
                    info.logical_cores);
            },
            Err(e) => println!("Error: {}", e),
        }
    }
    
    Ok(())
}
```

## Available Metrics

Monitord collects a wide range of system metrics:

### CPU
- Utilization (global and per-core)
- Frequency
- Temperature
- Cache information
- Model details
- CPU flags

### Memory
- Total, used, and available memory
- Swap usage
- Memory load percentage
- DRAM information (when available)

### Storage
- Disk usage
- Read/write speeds
- IO statistics
- SMART data (when available)
- Temperature

### Network
- Interface status
- Bandwidth usage
- Packet statistics
- Error counts
- IP/MAC addresses
- Driver information

### System
- OS details
- Kernel version
- Uptime
- Load averages
- Process/thread counts

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request