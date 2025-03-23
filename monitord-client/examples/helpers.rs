use monitord_protocols::monitord::{CpuInfo, GpuInfo, MemoryInfo, SystemSnapshot};

/// Format CPU information into a human-readable string
pub fn format_cpu_info(cpu: &CpuInfo) -> String {
    let mut output = format!(
        "CPU: {} ({} physical cores, {} logical cores)\n",
        cpu.model_name, cpu.physical_cores, cpu.logical_cores
    );
    
    output.push_str(&format!("Total Utilization: {:.1}%\n", cpu.global_utilization_percent));
    
    if !cpu.core_info.is_empty() {
        output.push_str("\nPer-core utilization:\n");
        for core in &cpu.core_info {
            output.push_str(&format!(
                "  Core {}: {:.1}% @ {:.0} MHz",
                core.core_id, core.utilization_percent, core.frequency_mhz
            ));
            
            if let Some(temp) = core.temperature_celsius.ne(&0.0).then_some(core.temperature_celsius) {
                output.push_str(&format!(", {:.1}°C", temp));
            }
            
            output.push('\n');
        }
    }
    
    output
}

/// Format memory information into a human-readable string
pub fn format_memory_info(mem: &MemoryInfo) -> String {
    let total_gb = mem.total_memory_bytes as f64 / 1_073_741_824.0;
    let used_gb = mem.used_memory_bytes as f64 / 1_073_741_824.0;
    let free_gb = mem.free_memory_bytes as f64 / 1_073_741_824.0;
    
    let mut output = format!(
        "Memory: {:.2} GB used / {:.2} GB total ({:.1}%)\n",
        used_gb, total_gb, mem.memory_load_percent
    );
    
    output.push_str(&format!("Free: {:.2} GB\n", free_gb));
    
    if mem.swap_total_bytes > 0 {
        let swap_total_gb = mem.swap_total_bytes as f64 / 1_073_741_824.0;
        let swap_used_gb = mem.swap_used_bytes as f64 / 1_073_741_824.0;
        let swap_free_gb = mem.swap_free_bytes as f64 / 1_073_741_824.0;
        let swap_percent = (mem.swap_used_bytes as f64 / mem.swap_total_bytes as f64) * 100.0;
        
        output.push_str(&format!(
            "\nSwap: {:.2} GB used / {:.2} GB total ({:.1}%)\n",
            swap_used_gb, swap_total_gb, swap_percent
        ));
        output.push_str(&format!("Swap Free: {:.2} GB\n", swap_free_gb));
    }
    
    output
}

/// Format GPU information into a human-readable string
pub fn format_gpu_info(gpu: &GpuInfo) -> String {
    let mut output = format!("GPU: {} ({})\n", gpu.name, gpu.vendor);
    
    if gpu.vram_total_bytes > 0 {
        let vram_total_gb = gpu.vram_total_bytes as f64 / 1_073_741_824.0;
        let vram_used_gb = gpu.vram_used_bytes as f64 / 1_073_741_824.0;
        let vram_free_gb = (gpu.vram_total_bytes - gpu.vram_used_bytes) as f64 / 1_073_741_824.0;
        
        output.push_str(&format!(
            "VRAM: {:.2} GB used / {:.2} GB total ({:.1}%)\n",
            vram_used_gb, vram_total_gb, gpu.memory_utilization_percent
        ));
        output.push_str(&format!("VRAM Free: {:.2} GB\n", vram_free_gb));
    }
    
    output.push_str(&format!("GPU Utilization: {:.1}%\n", gpu.core_utilization_percent));
    output.push_str(&format!("Temperature: {:.1}°C\n", gpu.temperature_celsius));
    
    if let Some(power) = gpu.power_usage_watts {
        output.push_str(&format!("Power Usage: {:.1} W\n", power));
    }
    
    if let Some(freq) = gpu.core_frequency_mhz {
        output.push_str(&format!("Core Frequency: {:.0} MHz\n", freq));
    }
    
    if let Some(mem_freq) = gpu.memory_frequency_mhz {
        output.push_str(&format!("Memory Frequency: {:.0} MHz\n", mem_freq));
    }
    
    output
}

/// Format a system snapshot into a summary string
pub fn format_system_summary(snapshot: &SystemSnapshot) -> String {
    let mut output = String::new();
    
    // System information
    if let Some(system) = &snapshot.system_info {
        output.push_str(&format!("System: {} ({} {})\n", 
                               system.hostname, 
                               system.os_name,
                               system.os_version));
        
        output.push_str(&format!("Kernel: {}\n", system.kernel_version));
        output.push_str(&format!("Uptime: {} hours\n", system.uptime_seconds / 3600));
        output.push_str(&format!("Load Average: {:.2}, {:.2}, {:.2}\n", 
                               system.load_average_1m,
                               system.load_average_5m,
                               system.load_average_15m));
        output.push('\n');
    }
    
    // CPU information
    if let Some(cpu) = &snapshot.cpu_info {
        output.push_str(&format_cpu_info(cpu));
        output.push('\n');
    }
    
    // Memory information
    if let Some(memory) = &snapshot.memory_info {
        output.push_str(&format_memory_info(memory));
        output.push('\n');
    }
    
    // GPU information
    if let Some(gpu_list) = &snapshot.gpu_info {
        for gpu in &gpu_list.gpus {
            output.push_str(&format_gpu_info(gpu));
            output.push('\n');
        }
    }
    
    output
}