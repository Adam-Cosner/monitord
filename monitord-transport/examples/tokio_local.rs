use monitord_protocols::monitord::*;
use monitord_transport::config::TransportConfig;
use monitord_transport::TransportManager;
use tracing::info;

async fn cpu(mut transport: TransportManager) -> Result<(), anyhow::Error> {
    let cpu = transport.receive::<CpuInfo>("cpu").await?.unwrap();
    info!("Received cpu snapshot");

    let mut output = String::new();
    output.push_str("Cpu Snapshot:\n");
    output.push_str(format!("  Model: {}\n", cpu.model_name).as_str());
    output.push_str(format!("  Physical Cores: {}\n", cpu.physical_cores).as_str());
    output.push_str(format!("  Logical Cores: {}\n", cpu.logical_cores).as_str());
    output.push_str(format!("  Global Utilization: {}\n", cpu.global_utilization_percent).as_str());
    output.push_str("  Cores:\n");
    for core in cpu.core_info.iter() {
        output.push_str(
            format!(
                "    Core {}: {}MHz {}% {}°C [{}MHz {}MHz]\n",
                core.core_id,
                core.frequency_mhz,
                core.utilization_percent,
                core.temperature_celsius,
                core.min_frequency_mhz(),
                core.max_frequency_mhz(),
            )
            .as_str(),
        );
    }
    output.push_str(format!("  Governor: {}\n", cpu.scaling_governor()).as_str());
    output.push_str(format!("  Architecture: {}\n", cpu.architecture).as_str());
    output.push_str(format!("  Flags: {:?}", cpu.cpu_flags).as_str());

    println!("{}", output);

    Ok(())
}

async fn memory(mut transport: TransportManager) -> Result<(), anyhow::Error> {
    let memory = transport.receive::<MemoryInfo>("memory").await?.unwrap();
    info!("Received memory snapshot");

    let mut output = String::new();
    output.push_str("Memory Snapshot:\n");
    output.push_str(format!("  Total Memory: {} bytes\n", memory.total_memory_bytes).as_str());
    output.push_str(format!("  Used Memory: {} bytes\n", memory.used_memory_bytes).as_str());
    output.push_str(format!("  Free Memory: {} bytes\n", memory.free_memory_bytes).as_str());
    output.push_str(format!("  Avail Memory: {} bytes\n", memory.available_memory_bytes).as_str());
    output.push_str(format!("  Total Swap: {} bytes\n", memory.swap_total_bytes).as_str());
    output.push_str(format!("  Used Swap: {} bytes\n", memory.swap_used_bytes).as_str());
    output.push_str(format!("  Free Swap: {} bytes\n", memory.swap_free_bytes).as_str());
    if let Some(dram_info) = &memory.dram_info {
        output.push_str("  DRAM Information:\n");
        output.push_str(format!("    Frequency: {}MHz\n", dram_info.frequency_mhz).as_str());
        output.push_str(format!("    Memory Type: {}\n", dram_info.memory_type).as_str());
        output.push_str(format!("    Total Slots:{}\n", dram_info.slots_total).as_str());
        output.push_str(format!("    Used Slots: {}\n", dram_info.slots_used).as_str());
        output.push_str(format!("    Manufacturer: {}\n", dram_info.manufacturer()).as_str());
        output.push_str(format!("    Part Number: {}\n", dram_info.part_number()).as_str());
    }
    output.push_str(format!("  Cached Memory: {} bytes\n", memory.cached_memory_bytes).as_str());
    output.push_str(format!("  Shared Memory: {} bytes\n", memory.shared_memory_bytes).as_str());
    output.push_str(format!("  Memory Load: {}%\n", memory.memory_load_percent).as_str());

    println!("{}", output);

    Ok(())
}

async fn system(mut transport: TransportManager) -> Result<(), anyhow::Error> {
    let system = transport.receive::<SystemInfo>("system").await?.unwrap();
    info!("Received system snapshot");

    let mut output = String::new();
    output.push_str("System Snapshot:\n");
    output.push_str(format!("  Hostname: {}\n", system.hostname).as_str());
    output.push_str(format!("  OS: {} {}\n", system.os_name, system.os_version).as_str());
    output.push_str(format!("  Kernel: {}\n", system.kernel_version).as_str());
    output.push_str(format!("  Architecture: {}\n", system.architecture).as_str());
    output.push_str(format!("  Process Count: {}\n", system.process_count).as_str());
    output.push_str(format!("  Thread Count: {}\n", system.thread_count).as_str());
    output.push_str(format!("  Open Files: {}\n", system.open_file_count).as_str());
    output.push_str(format!("  Uptime: {} seconds\n", system.uptime_seconds).as_str());
    output.push_str(format!("  Load Average: {}m, {}m, {}m\n", 
        system.load_average_1m, system.load_average_5m, system.load_average_15m).as_str());
    output.push_str(format!("  Boot Time: {}\n", system.boot_time).as_str());
    if let Some(vendor) = &system.vendor {
        output.push_str(format!("  Vendor: {}\n", vendor).as_str());
    }
    if let Some(virtualization) = &system.virtualization {
        output.push_str(format!("  Virtualization: {}\n", virtualization).as_str());
    }
    if !system.security_features.is_empty() {
        output.push_str(format!("  Security Features: {:?}\n", system.security_features).as_str());
    }

    println!("{}", output);

    Ok(())
}

async fn storage(mut transport: TransportManager) -> Result<(), anyhow::Error> {
    let storage_list = transport.receive::<StorageList>("storage").await?.unwrap();
    info!("Received storage snapshot");

    let mut output = String::new();
    output.push_str("Storage Devices:\n");
    
    for storage in &storage_list.storages {
        output.push_str(format!("  Device: {}\n", storage.device_name).as_str());
        output.push_str(format!("    Type: {}\n", storage.device_type).as_str());
        output.push_str(format!("    Model: {}\n", storage.model).as_str());
        output.push_str(format!("    Filesystem: {}\n", storage.filesystem_type).as_str());
        output.push_str(format!("    Mount Point: {}\n", storage.mount_point).as_str());
        output.push_str(format!("    Total Space: {} bytes\n", storage.total_space_bytes).as_str());
        output.push_str(format!("    Used Space: {} bytes\n", storage.used_space_bytes).as_str());
        output.push_str(format!("    Available Space: {} bytes\n", storage.available_space_bytes).as_str());
        output.push_str(format!("    Read Speed: {} bytes/sec\n", storage.read_bytes_per_sec).as_str());
        output.push_str(format!("    Write Speed: {} bytes/sec\n", storage.write_bytes_per_sec).as_str());
        output.push_str(format!("    IO Time: {}ms\n", storage.io_time_ms).as_str());
        
        if let Some(temp) = storage.temperature_celsius {
            output.push_str(format!("    Temperature: {}°C\n", temp).as_str());
        }
        if let Some(lifetime_writes) = storage.lifetime_writes_bytes {
            output.push_str(format!("    Lifetime Writes: {} bytes\n", lifetime_writes).as_str());
        }
        if let Some(serial) = &storage.serial_number {
            output.push_str(format!("    Serial Number: {}\n", serial).as_str());
        }
        if let Some(label) = &storage.partition_label {
            output.push_str(format!("    Partition Label: {}\n", label).as_str());
        }
        
        if let Some(smart) = &storage.smart_data {
            output.push_str("    SMART Data:\n");
            output.push_str(format!("      Health Status: {}\n", smart.health_status).as_str());
            if let Some(hours) = smart.power_on_hours {
                output.push_str(format!("      Power On Hours: {}\n", hours).as_str());
            }
            if let Some(cycles) = smart.power_cycle_count {
                output.push_str(format!("      Power Cycles: {}\n", cycles).as_str());
            }
            if let Some(sectors) = smart.reallocated_sectors {
                output.push_str(format!("      Reallocated Sectors: {}\n", sectors).as_str());
            }
            if let Some(life) = smart.remaining_life_percent {
                output.push_str(format!("      Remaining Life: {}%\n", life).as_str());
            }
        }
        
        output.push_str("\n");
    }

    println!("{}", output);

    Ok(())
}

async fn network(mut transport: TransportManager) -> Result<(), anyhow::Error> {
    let network_list = transport.receive::<NetworkList>("network").await?.unwrap();
    info!("Received network snapshot");

    let mut output = String::new();
    output.push_str("Network Interfaces:\n");
    
    for net in &network_list.nets {
        output.push_str(format!("  Interface: {}\n", net.interface_name).as_str());
        output.push_str(format!("    Driver: {}\n", net.driver).as_str());
        output.push_str(format!("    MAC: {}\n", net.mac_address).as_str());
        output.push_str(format!("    IP Addresses: {:?}\n", net.ip_addresses).as_str());
        output.push_str(format!("    Status: {}\n", if net.is_up { "Up" } else { "Down" }).as_str());
        output.push_str(format!("    MTU: {}\n", net.mtu).as_str());
        output.push_str(format!("    Max Bandwidth: {} bytes/sec\n", net.max_bandwidth_bytes_per_sec).as_str());
        output.push_str(format!("    RX Speed: {} bytes/sec\n", net.rx_bytes_per_sec).as_str());
        output.push_str(format!("    TX Speed: {} bytes/sec\n", net.tx_bytes_per_sec).as_str());
        output.push_str(format!("    RX Packets: {}/sec\n", net.rx_packets_per_sec).as_str());
        output.push_str(format!("    TX Packets: {}/sec\n", net.tx_packets_per_sec).as_str());
        output.push_str(format!("    RX Errors: {}\n", net.rx_errors).as_str());
        output.push_str(format!("    TX Errors: {}\n", net.tx_errors).as_str());
        output.push_str(format!("    RX Total: {} bytes\n", net.rx_bytes_total).as_str());
        output.push_str(format!("    TX Total: {} bytes\n", net.tx_bytes_total).as_str());
        
        if !net.dns_servers.is_empty() {
            output.push_str(format!("    DNS Servers: {:?}\n", net.dns_servers).as_str());
        }
        if let Some(speed) = net.link_speed_mbps {
            output.push_str(format!("    Link Speed: {} Mbps\n", speed).as_str());
        }
        
        output.push_str("\n");
    }

    println!("{}", output);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let mut transport = TransportManager::new(TransportConfig::default())?;
    let mut joinset = tokio::task::JoinSet::new();

    transport.initialize().await?;
    {
        let transport_clone = transport.clone();
        joinset.spawn(tokio::task::spawn(cpu(transport_clone)));
    }
    {
        let transport_clone = transport.clone();
        joinset.spawn(tokio::task::spawn(memory(transport_clone)));
    }
    {
        let transport_clone = transport.clone();
        joinset.spawn(tokio::task::spawn(system(transport_clone)));
    }
    {
        let transport_clone = transport.clone();
        joinset.spawn(tokio::task::spawn(storage(transport_clone)));
    }
    {
        let transport_clone = transport.clone();
        joinset.spawn(tokio::task::spawn(network(transport_clone)));
    }

    while let Some(result) = joinset.join_next().await {
        match result {
            Ok(Ok(_)) => println!("Task completed successfully"),
            Ok(Err(e)) => println!("Task returned error: {}", e),
            Err(e) => println!("Task panicked: {}", e),
        }
    }

    Ok(())
}
