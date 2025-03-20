use futures::StreamExt;
use monitord_collectors::{
    config::{
        CollectorsConfig, CpuCollectorConfig, GpuCollectorConfig, MemoryCollectorConfig,
        NetworkCollectorConfig, ProcessCollectorConfig, StorageCollectorConfig,
        SystemCollectorConfig,
    },
    cpu::CpuCollector,
    gpu::GpuCollector,
    memory::MemoryCollector,
    network::NetworkCollector,
    process::ProcessCollector,
    storage::StorageCollector,
    system::SystemCollector,
    traits::Collector,
};
use monitord_protocols::monitord::SystemSnapshot;
use monitord_protocols::protocols::prost_types::Timestamp;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    println!("Initializing collectors...");

    // Create all collectors with their configs
    let cpu_config = CpuCollectorConfig::default();
    let memory_config = MemoryCollectorConfig::default();
    let gpu_config = GpuCollectorConfig::default();
    let network_config = NetworkCollectorConfig::default();
    let storage_config = StorageCollectorConfig::default();
    let process_config = ProcessCollectorConfig::default();
    let system_config = SystemCollectorConfig::default();

    // Create a single collector report
    let snapshot = tokio::try_join!(
        async {
            let mut collector = SystemCollector::new(system_config.clone())?;
            Ok::<_, Box<dyn std::error::Error>>(collector.collect()?)
        },
        async {
            let mut collector = CpuCollector::new(cpu_config.clone())?;
            Ok::<_, Box<dyn std::error::Error>>(collector.collect()?)
        },
        async {
            let mut collector = MemoryCollector::new(memory_config.clone())?;
            Ok::<_, Box<dyn std::error::Error>>(collector.collect()?)
        },
        async {
            let collector = GpuCollector::new(gpu_config.clone());
            if let Ok(mut collector) = collector {
                Ok(collector.collect().unwrap_or_default())
            } else {
                // Return empty GPU list if collector fails to initialize
                Ok(monitord_protocols::monitord::GpuList { gpus: vec![] })
            }
        },
        async {
            let mut collector = NetworkCollector::new(network_config.clone())?;
            Ok::<_, Box<dyn std::error::Error>>(collector.collect()?)
        },
        async {
            let mut collector = StorageCollector::new(storage_config.clone())?;
            Ok::<_, Box<dyn std::error::Error>>(collector.collect()?)
        },
        async {
            let mut collector = ProcessCollector::new(process_config.clone())?;
            Ok::<_, Box<dyn std::error::Error>>(collector.collect()?)
        },
    )?;

    // Create a system snapshot from all collected data
    let system_snapshot = SystemSnapshot {
        timestamp: None,
        system_info: Some(snapshot.0),
        cpu_info: Some(snapshot.1),
        memory_info: Some(snapshot.2),
        gpu_info: Some(snapshot.3),
        network_info: Some(snapshot.4),
        storage_devices: snapshot.5.storages,
        processes: snapshot.6.processes,
    };

    // Print a summary of the collected data
    println!("\nSystem Snapshot Summary");
    println!("======================");

    if let Some(sys_info) = &system_snapshot.system_info {
        println!("Hostname: {}", sys_info.hostname);
        println!("OS: {} {}", sys_info.os_name, sys_info.os_version);
        println!("Kernel: {}", sys_info.kernel_version);
        println!("Uptime: {} hours", sys_info.uptime_seconds / 3600);
        println!(
            "Load Avg: {:.2}, {:.2}, {:.2}",
            sys_info.load_average_1m, sys_info.load_average_5m, sys_info.load_average_15m
        );
    }

    if let Some(cpu_info) = &system_snapshot.cpu_info {
        println!("\nCPU: {}", cpu_info.model_name);
        println!(
            "Cores: {} physical, {} logical",
            cpu_info.physical_cores, cpu_info.logical_cores
        );
        println!("CPU Usage: {:.2}%", cpu_info.global_utilization_percent);
    }

    if let Some(mem_info) = &system_snapshot.memory_info {
        let total_gb = mem_info.total_memory_bytes as f64 / 1_073_741_824.0;
        let used_gb = mem_info.used_memory_bytes as f64 / 1_073_741_824.0;
        println!(
            "\nMemory: {:.2} GB / {:.2} GB ({:.2}%)",
            used_gb, total_gb, mem_info.memory_load_percent
        );

        let swap_total_gb = mem_info.swap_total_bytes as f64 / 1_073_741_824.0;
        let swap_used_gb = mem_info.swap_used_bytes as f64 / 1_073_741_824.0;
        println!("Swap: {:.2} GB / {:.2} GB", swap_used_gb, swap_total_gb);
    }

    if let Some(gpu_info) = &system_snapshot.gpu_info {
        println!("\nGPUs: {}", gpu_info.gpus.len());
        for (i, gpu) in gpu_info.gpus.iter().enumerate() {
            println!("  GPU {}: {}", i, gpu.name);
            if gpu.vram_total_bytes > 0 {
                let vram_total_gb = gpu.vram_total_bytes as f64 / 1_073_741_824.0;
                let vram_used_gb = gpu.vram_used_bytes as f64 / 1_073_741_824.0;
                println!("    VRAM: {:.2} GB / {:.2} GB", vram_used_gb, vram_total_gb);
                println!(
                    "    Utilization: {:.2}% core, {:.2}% memory",
                    gpu.core_utilization_percent, gpu.memory_utilization_percent
                );
                println!("    Temperature: {:.1}°C", gpu.temperature_celsius);
            }
        }
    }

    println!(
        "\nNetwork Interfaces: {}",
        system_snapshot
            .network_info
            .as_ref()
            .map_or(0, |n| n.nets.len())
    );
    if let Some(net_info) = &system_snapshot.network_info {
        for net in &net_info.nets {
            println!(
                "  {}: ↓ {:.2} MB/s, ↑ {:.2} MB/s",
                net.interface_name,
                net.rx_bytes_per_sec as f64 / 1_048_576.0,
                net.tx_bytes_per_sec as f64 / 1_048_576.0
            );
        }
    }

    println!(
        "\nStorage Devices: {}",
        system_snapshot.storage_devices.len()
    );
    for disk in &system_snapshot.storage_devices {
        let total_gb = disk.total_space_bytes as f64 / 1_073_741_824.0;
        let used_gb = disk.used_space_bytes as f64 / 1_073_741_824.0;
        let used_pct = if disk.total_space_bytes > 0 {
            (disk.used_space_bytes as f64 / disk.total_space_bytes as f64) * 100.0
        } else {
            0.0
        };
        println!(
            "  {} ({}): {:.2} GB / {:.2} GB ({:.1}%)",
            disk.device_name, disk.mount_point, used_gb, total_gb, used_pct
        );
    }

    println!("\nProcesses: {}", system_snapshot.processes.len());
    // Show the top 5 CPU-consuming processes
    let mut processes = system_snapshot.processes.clone();
    processes.sort_by(|a, b| {
        b.cpu_usage_percent
            .partial_cmp(&a.cpu_usage_percent)
            .unwrap()
    });
    println!("Top processes by CPU usage:");
    for (i, proc) in processes.iter().take(5).enumerate() {
        println!(
            "  {}. {} (PID {}): {:.2}% CPU, {:.2} MB RAM",
            i + 1,
            proc.name,
            proc.pid,
            proc.cpu_usage_percent,
            proc.physical_memory_bytes as f64 / 1_048_576.0
        );
    }

    println!("\nSnapshot complete!");

    Ok(())
}
