use futures::StreamExt;
use monitord_client::MonitordClient;
use monitord_protocols::monitord::{GpuInfo, NetworkInfo};
use std::time::Duration;
use tokio::time::interval;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the monitord service
    let client = MonitordClient::connect("http://localhost:50051").await?;
    
    println!("=== Monitord System Dashboard ===");
    println!("Press Ctrl+C to exit");
    
    // Create a refresh interval
    let mut refresh = interval(Duration::from_secs(1));
    
    loop {
        // Wait for the next interval tick
        refresh.tick().await;
        
        // Clear the terminal
        print!("\x1B[2J\x1B[1;1H");
        
        // Get a system snapshot
        match client.get_system_snapshot().await {
            Ok(snapshot) => {
                // System Information
                if let Some(system_info) = &snapshot.system_info {
                    println!("=== System Information ===");
                    println!("Hostname:  {}", system_info.hostname);
                    println!("OS:        {} {}", system_info.os_name, system_info.os_version);
                    println!("Kernel:    {}", system_info.kernel_version);
                    println!("Arch:      {}", system_info.architecture);
                    println!("Uptime:    {} hours", system_info.uptime_seconds / 3600);
                    println!("Load Avg:  {:.2}, {:.2}, {:.2}",
                             system_info.load_average_1m,
                             system_info.load_average_5m,
                             system_info.load_average_15m);
                    println!();
                }
                
                // CPU Information
                if let Some(cpu_info) = &snapshot.cpu_info {
                    println!("=== CPU Information ===");
                    println!("Model:      {}", cpu_info.model_name);
                    println!("Cores:      {} physical, {} logical",
                             cpu_info.physical_cores,
                             cpu_info.logical_cores);
                    println!("Usage:      {:.1}%", cpu_info.global_utilization_percent);
                    
                    // Show core information
                    if !cpu_info.core_info.is_empty() {
                        println!("\nCores:");
                        for (i, core) in cpu_info.core_info.iter().enumerate() {
                            if i % 4 == 0 && i > 0 {
                                println!();
                            }
                            print!("Core {}: {:.1}% @ {:.0}MHz  ",
                                  core.core_id,
                                  core.utilization_percent,
                                  core.frequency_mhz);
                        }
                        println!();
                    }
                    println!();
                }
                
                // Memory Information
                if let Some(memory_info) = &snapshot.memory_info {
                    println!("=== Memory Information ===");
                    let total_gb = memory_info.total_memory_bytes as f64 / 1_073_741_824.0;
                    let used_gb = memory_info.used_memory_bytes as f64 / 1_073_741_824.0;
                    let free_gb = memory_info.free_memory_bytes as f64 / 1_073_741_824.0;
                    
                    println!("Total:      {:.2} GB", total_gb);
                    println!("Used:       {:.2} GB ({:.1}%)", 
                             used_gb,
                             memory_info.memory_load_percent);
                    println!("Free:       {:.2} GB", free_gb);
                    
                    if memory_info.swap_total_bytes > 0 {
                        let swap_total_gb = memory_info.swap_total_bytes as f64 / 1_073_741_824.0;
                        let swap_used_gb = memory_info.swap_used_bytes as f64 / 1_073_741_824.0;
                        let swap_percent = if memory_info.swap_total_bytes > 0 {
                            (memory_info.swap_used_bytes as f64 / memory_info.swap_total_bytes as f64) * 100.0
                        } else {
                            0.0
                        };
                        
                        println!("Swap Total: {:.2} GB", swap_total_gb);
                        println!("Swap Used:  {:.2} GB ({:.1}%)", swap_used_gb, swap_percent);
                    }
                    println!();
                }
                
                // GPU Information
                if let Some(gpu_list) = &snapshot.gpu_info {
                    if !gpu_list.gpus.is_empty() {
                        println!("=== GPU Information ===");
                        for (i, gpu) in gpu_list.gpus.iter().enumerate() {
                            print_gpu_info(gpu, i);
                        }
                    }
                }
                
                // Network Information
                if let Some(network_list) = &snapshot.network_info {
                    if !network_list.nets.is_empty() {
                        println!("=== Network Information ===");
                        for net in &network_list.nets {
                            print_network_info(net);
                        }
                    }
                }
                
                // Top Processes
                if !snapshot.processes.is_empty() {
                    println!("=== Top Processes (by CPU) ===");
                    println!("{:<5} {:<20} {:<8} {:<10}", "PID", "NAME", "CPU%", "MEM (MB)");
                    
                    // Sort by CPU usage and take top 5
                    let mut processes = snapshot.processes.clone();
                    processes.sort_by(|a, b| {
                        b.cpu_usage_percent.partial_cmp(&a.cpu_usage_percent)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                    
                    for process in processes.iter().take(5) {
                        println!("{:<5} {:<20} {:<8.1} {:<10}",
                                 process.pid,
                                 truncate_str(&process.name, 20),
                                 process.cpu_usage_percent,
                                 process.physical_memory_bytes / 1_048_576);
                    }
                    println!();
                }
                
                // Timestamp
                let now = chrono::Local::now();
                println!("Last update: {}", now.format("%Y-%m-%d %H:%M:%S"));
            },
            Err(e) => {
                println!("Error fetching system data: {}", e);
            }
        }
    }
}

fn print_gpu_info(gpu: &GpuInfo, index: usize) {
    println!("GPU #{}: {} ({})", index, gpu.name, gpu.vendor);
    
    // Memory
    if gpu.vram_total_bytes > 0 {
        let vram_total_gb = gpu.vram_total_bytes as f64 / 1_073_741_824.0;
        let vram_used_gb = gpu.vram_used_bytes as f64 / 1_073_741_824.0;
        println!("Memory:     {:.2} GB / {:.2} GB ({:.1}%)",
                 vram_used_gb,
                 vram_total_gb,
                 gpu.memory_utilization_percent);
    }
    
    // Utilization and temperature
    println!("Utilization: {:.1}%", gpu.core_utilization_percent);
    println!("Temperature: {:.1}°C", gpu.temperature_celsius);
    
    // Power and frequency if available
    if let Some(power) = gpu.power_usage_watts {
        println!("Power:      {:.1} W", power);
    }
    
    if let Some(freq) = gpu.core_frequency_mhz {
        println!("Frequency:  {:.0} MHz", freq);
    }
    
    println!();
}

fn print_network_info(net: &NetworkInfo) {
    // Skip loopback or inactive interfaces
    if net.interface_name == "lo" || !net.is_up {
        return;
    }
    
    println!("Interface: {} ({})", net.interface_name, net.driver);
    
    // IP addresses
    if !net.ip_addresses.is_empty() {
        println!("IP:        {}", net.ip_addresses.join(", "));
    }
    
    // Traffic
    let rx_mbps = net.rx_bytes_per_sec as f64 / 131_072.0; // Convert to Mbps
    let tx_mbps = net.tx_bytes_per_sec as f64 / 131_072.0;
    println!("Traffic:   ↓ {:.2} Mbps  ↑ {:.2} Mbps", rx_mbps, tx_mbps);
    
    println!();
}

fn truncate_str(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        s
    } else {
        &s[..max_len]
    }
}