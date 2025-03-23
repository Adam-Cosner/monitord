mod helpers;

use futures::StreamExt;
use helpers::format_system_summary;
use monitord_client::MonitordClient;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to monitord service
    let client = match MonitordClient::connect("http://localhost:50051").await {
        Ok(client) => {
            println!("Connected to monitord service");
            client
        },
        Err(e) => {
            eprintln!("Failed to connect to monitord service: {}", e);
            eprintln!("Make sure the monitord service is running on localhost:50051");
            std::process::exit(1);
        }
    };
    
    // Get a single system snapshot
    println!("Fetching initial system snapshot...");
    match client.get_system_snapshot().await {
        Ok(snapshot) => {
            println!("{}", format_system_summary(&snapshot));
        },
        Err(e) => {
            eprintln!("Failed to get system snapshot: {}", e);
        }
    }
    
    // Stream system snapshots
    println!("Starting to stream system snapshots (press Ctrl+C to exit)...");
    println!("Updates will arrive every 5 seconds.");
    
    let mut stream = client.stream_system_snapshots(5000).await?;
    
    while let Some(result) = stream.next().await {
        match result {
            Ok(snapshot) => {
                // Clear the screen
                print!("\x1B[2J\x1B[1;1H");
                
                // Print timestamp
                let now = chrono::Local::now();
                println!("=== System Update: {} ===\n", now.format("%Y-%m-%d %H:%M:%S"));
                
                // Print system summary
                println!("{}", format_system_summary(&snapshot));
                
                // Small sleep to avoid flooding the terminal
                tokio::time::sleep(Duration::from_millis(100)).await;
            },
            Err(e) => {
                eprintln!("Error receiving system snapshot: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}