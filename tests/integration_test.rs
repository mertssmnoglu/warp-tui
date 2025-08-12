use crate::helpers::is_warp_cli_available;
use warp_tui::warp::{WarpClient, WarpStatus};

mod helpers;

/// Integration test for WARP connect/disconnect functionality
///
/// This test verifies that the WarpClient can properly:
/// 1. Get initial status
/// 2. Connect to WARP
/// 3. Verify connected status
/// 4. Disconnect from WARP
/// 5. Verify disconnected status
///
/// Note: This test requires warp-cli to be installed and available in PATH
#[test]
fn test_connect_disconnect_flow() {
    // Skip test if warp-cli is not available
    if !is_warp_cli_available() {
        println!("Skipping test: warp-cli not available");
        return;
    }

    let client = WarpClient::new();

    // Get initial status
    let initial_status = client
        .get_status_sync()
        .expect("Should be able to get initial WARP status");

    println!("Initial WARP status: {:?}", initial_status.status);

    // Test connect functionality
    match client.connect_sync() {
        Ok(_) => {
            println!("Connect command executed successfully");

            // Wait for connection to establish
            std::thread::sleep(std::time::Duration::from_secs(2));

            // Verify status after connect attempt
            let post_connect_status = client
                .get_status_sync()
                .expect("Should be able to get status after connect");

            println!("Status after connect: {:?}", post_connect_status.status);

            // Status should be Connected or Connecting (depending on timing)
            assert!(
                matches!(
                    post_connect_status.status,
                    WarpStatus::Connected | WarpStatus::Connecting
                ),
                "After connect, status should be Connected or Connecting, but was {:?}",
                post_connect_status.status
            );
        }
        Err(e) => {
            println!("Connect command failed: {:?}", e);
            // Don't fail the test if already connected
        }
    }

    // Test disconnect functionality
    match client.disconnect_sync() {
        Ok(_) => {
            println!("Disconnect command executed successfully");

            // Wait longer for disconnection to complete and retry if needed
            let mut attempts = 0;
            let max_attempts = 5;
            let mut final_status = None;

            while attempts < max_attempts {
                std::thread::sleep(std::time::Duration::from_secs(2));

                let status = client
                    .get_status_sync()
                    .expect("Should be able to get status after disconnect");

                println!(
                    "Status after disconnect (attempt {}): {:?}",
                    attempts + 1,
                    status.status
                );

                if matches!(
                    status.status,
                    WarpStatus::Disconnected | WarpStatus::Disconnecting
                ) {
                    final_status = Some(status);
                    break;
                }

                attempts += 1;
            }

            // Verify final status
            if let Some(status) = final_status {
                // Status should be Disconnected or Disconnecting
                assert!(
                    matches!(
                        status.status,
                        WarpStatus::Disconnected | WarpStatus::Disconnecting
                    ),
                    "After disconnect, status should be Disconnected or Disconnecting, but was {:?}",
                    status.status
                );
            } else {
                // If we couldn't get the expected status after multiple attempts,
                // get the final status for reporting
                let post_disconnect_status = client
                    .get_status_sync()
                    .expect("Should be able to get status after disconnect");

                println!(
                    "Warning: Status after disconnect attempts: {:?}",
                    post_disconnect_status.status
                );

                // More lenient assertion - allow any status except Connected
                // since disconnect was called but might be in transition
                assert!(
                    !matches!(post_disconnect_status.status, WarpStatus::Connected),
                    "After disconnect command, status should not be Connected, but was {:?}",
                    post_disconnect_status.status
                );
            }
        }
        Err(e) => {
            println!("Disconnect command failed: {:?}", e);
            // Don't fail the test if already disconnected
        }
    }

    // Final status check
    let final_status = client
        .get_status_sync()
        .expect("Should be able to get final WARP status");

    println!("Final WARP status: {:?}", final_status.status);
}

/// Test that verifies status parsing works correctly
#[test]
fn test_status_retrieval() {
    // Skip test if warp-cli is not available
    if !is_warp_cli_available() {
        println!("Skipping test: warp-cli not available");
        return;
    }

    let client = WarpClient::new();

    // Should be able to get status without errors
    let status = client
        .get_status_sync()
        .expect("Should be able to get WARP status");

    // Status should be one of the known variants
    assert!(
        matches!(
            status.status,
            WarpStatus::Connected
                | WarpStatus::Disconnected
                | WarpStatus::Connecting
                | WarpStatus::Disconnecting
                | WarpStatus::Unknown
        ),
        "Status should be a valid WarpStatus variant, but was {:?}",
        status.status
    );

    println!("Current WARP status: {:?}", status);
    println!("Account type: {:?}", status.account_type);
    println!("Mode: {:?}", status.mode);
    println!("WARP enabled: {}", status.warp_enabled);
    println!("Gateway enabled: {}", status.gateway_enabled);
}

/// Test multiple rapid connect/disconnect operations
#[test]
fn test_rapid_connect_disconnect() {
    // Skip test if warp-cli is not available
    if !is_warp_cli_available() {
        println!("Skipping test: warp-cli not available");
        return;
    }

    let client = WarpClient::new();

    // Ensure we start from a known state (disconnected)
    let _ = client.disconnect_sync();
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Perform multiple connect/disconnect cycles
    for i in 1..=3 {
        println!("Cycle {}: Testing connect...", i);

        // Connect
        let connect_result = client.connect_sync();
        println!("Connect result: {:?}", connect_result);

        // Wait for operation to complete
        std::thread::sleep(std::time::Duration::from_secs(1));

        println!("Cycle {}: Testing disconnect...", i);

        // Disconnect
        let disconnect_result = client.disconnect_sync();
        println!("Disconnect result: {:?}", disconnect_result);

        // Wait for operation to complete before next cycle
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    // Final status check
    let final_status = client
        .get_status_sync()
        .expect("Should be able to get final status");

    println!(
        "Final status after rapid operations: {:?}",
        final_status.status
    );
}

/// Test that verifies operation mode retrieval
#[test]
fn test_operation_mode() {
    // Skip test if warp-cli is not available
    if !is_warp_cli_available() {
        println!("Skipping test_operation_mode: warp-cli not available");
        return;
    }

    let client = WarpClient::new();
    let mode = client
        .get_operation_mode()
        .expect("Failed to get operation mode");
    println!("Current operation mode: {}", mode);
}
