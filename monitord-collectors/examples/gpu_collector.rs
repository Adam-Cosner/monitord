use monitord_collectors::config::GpuCollectorConfig;
use monitord_collectors::gpu::GpuCollector;
use monitord_collectors::traits::Collector;
use std::error::Error;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize the tracing subscriber
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Create a configuration for the GPU collector
    // By default, all GPU types are enabled
    let config = GpuCollectorConfig {
        enabled: true,
        interval_ms: 1000,
        collect_nvidia: true,
        collect_amd: true,
        collect_intel: true,
        collect_processes: true,
    };

    // Initialize the GPU collector
    let mut collector = match GpuCollector::new(config) {
        Ok(collector) => collector,
        Err(e) => {
            println!("Failed to initialize GPU collector: {}", e);
            // Create a disabled configuration instead
            let disabled_config = GpuCollectorConfig {
                enabled: false,
                ..GpuCollectorConfig::default()
            };
            GpuCollector::new(disabled_config)?
        }
    };

    // Collect GPU information
    match collector.collect() {
        Ok(gpu_list) => {
            println!("Collected information for {} GPU(s)", gpu_list.gpus.len());
            
            // Print information about each GPU
            for (i, gpu) in gpu_list.gpus.iter().enumerate() {
                println!("\nGPU {}: {} ({})", i, gpu.name, gpu.vendor);
                println!("  VRAM: {:.2} GB total, {:.2} GB used",
                         gpu.vram_total_bytes as f64 / 1_073_741_824.0,
                         gpu.vram_used_bytes as f64 / 1_073_741_824.0);
                println!("  GPU Utilization: {:.1}%", gpu.core_utilization_percent);
                println!("  Memory Utilization: {:.1}%", gpu.memory_utilization_percent);
                println!("  Temperature: {:.1}°C", gpu.temperature_celsius);
                
                if let Some(power) = gpu.power_usage_watts {
                    println!("  Power Usage: {:.1} W", power);
                }
                
                if let Some(freq) = gpu.core_frequency_mhz {
                    println!("  Core Frequency: {:.0} MHz", freq);
                }
                
                if let Some(freq) = gpu.memory_frequency_mhz {
                    println!("  Memory Frequency: {:.0} MHz", freq);
                }
                
                if let Some(driver) = &gpu.driver_info {
                    println!("  Driver: {} {}", driver.kernel_driver, driver.driver_version);
                }
                
                if let Some(encoder) = &gpu.encoder_info {
                    println!("  Encoder Utilization: {:.1}%", encoder.video_encode_utilization_percent);
                    println!("  Decoder Utilization: {:.1}%", encoder.video_decode_utilization_percent);
                }
                
                if !gpu.process_info.is_empty() {
                    println!("  Processes:");
                    for proc in &gpu.process_info {
                        println!("    PID {}: {:.1}% GPU, {:.2} GB VRAM",
                                 proc.pid,
                                 proc.gpu_utilization_percent,
                                 proc.vram_bytes as f64 / 1_073_741_824.0);
                    }
                }
            }
        }
        Err(e) => println!("Failed to collect GPU information: {}", e),
    }

    Ok(())
}