use futures::StreamExt;
use monitord_client::{MonitordClient, ProcessFilter};
use std::time::Duration;

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
        println!("Uptime: {} seconds", system_info.uptime_seconds);
    }
    
    // Print CPU information
    if let Some(cpu_info) = &snapshot.cpu_info {
        println!("\nCPU: {}", cpu_info.model_name);
        println!("Cores: {} physical, {} logical", cpu_info.physical_cores, cpu_info.logical_cores);
        println!("Utilization: {:.1}%", cpu_info.global_utilization_percent);
    }
    
    // Print memory information
    if let Some(memory_info) = &snapshot.memory_info {
        let total_gb = memory_info.total_memory_bytes as f64 / 1_073_741_824.0;
        let used_gb = memory_info.used_memory_bytes as f64 / 1_073_741_824.0;
        println!("\nMemory: {:.1} GB used of {:.1} GB total ({:.1}%)",
                used_gb,
                total_gb,
                memory_info.memory_load_percent);
    }
    
    // Stream CPU information for 5 updates
    println!("\nStreaming CPU information (5 updates)...");
    let mut cpu_stream = client.stream_cpu_info(1000).await?;
    let mut count = 0;
    
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
        
        count += 1;
        if count >= 5 {
            break;
        }
        
        // Small delay between prints
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // Stream process information with filtering
    println!("\nTop 5 CPU-intensive processes:");
    let filter = ProcessFilter {
        sort_by_cpu: true,
        limit: 5,
        ..Default::default()
    };
    
    let mut process_stream = client.stream_process_info(1000, filter).await?;
    
    if let Some(processes) = process_stream.next().await {
        match processes {
            Ok(process) => {
                println!("PID: {}, Name: {}, CPU: {:.1}%, Memory: {} MB",
                         process.pid,
                         process.name,
                         process.cpu_usage_percent,
                         process.physical_memory_bytes / 1_048_576);
            },
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    
    println!("\nDone!");
    Ok(())
}