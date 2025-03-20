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
    output.push_str(format!("  Free Swap: {} bytes\n", memory.swap_used_bytes).as_str());
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

    for results in joinset.join_all().await {
        results?;
    }

    Ok(())
}
