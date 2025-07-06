use warp_tui::warp::WarpClient;
use warp_tui::warp::manager::{WarpManager, WarpMessage};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Cloudflare WARP CLI Library Demo");
    println!("================================");

    // Create a new warp client
    let client = WarpClient::new();

    // Check if warp-cli is available
    if !client.is_available().await {
        println!("❌ warp-cli is not available on this system");
        println!("Please install Cloudflare WARP and ensure warp-cli is in your PATH");
        return Ok(());
    }

    println!("✅ warp-cli is available");

    // Get current status
    match client.get_status().await {
        Ok(info) => {
            println!("\n📊 Current WARP Status:");
            println!("  Status: {}", info.status);
            if let Some(mode) = &info.mode {
                println!("  Mode: {}", mode);
            }
            if let Some(account_type) = &info.account_type {
                println!("  Account Type: {}", account_type);
            }
            println!("  WARP Enabled: {}", info.warp_enabled);
            println!("  Gateway Enabled: {}", info.gateway_enabled);
        }
        Err(e) => {
            println!("❌ Failed to get status: {}", e);
        }
    }

    // Demo the manager for background operations
    println!("\n🔄 Starting background manager demo...");
    let manager = WarpManager::new();
    let sender = manager.get_sender();

    // Start background tasks
    manager.start_background_tasks().await;

    // Send a refresh command
    sender.send(WarpMessage::Refresh)?;

    // Process a few messages
    tokio::select! {
        _ = manager.process_messages() => {},
        _ = tokio::time::sleep(std::time::Duration::from_secs(2)) => {
            println!("✅ Manager demo completed");
        }
    }

    println!("\n🎯 Available Commands:");
    println!("  - client.get_status() - Get current status");
    println!("  - client.connect() - Connect to WARP");
    println!("  - client.disconnect() - Disconnect from WARP");
    println!("  - client.create_registration() - Create new registration");
    println!("  - client.delete_registration() - Delete registration");
    println!("  - client.set_mode(mode) - Set DNS mode");

    println!("\n📚 Integration Example:");
    println!("  Use WarpClient for direct command execution");
    println!("  Use WarpManager for background status monitoring");
    println!("  All methods return proper Result types for error handling");

    Ok(())
}
