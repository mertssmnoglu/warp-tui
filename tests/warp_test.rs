use warp_tui::warp::client::WarpClient;
use warp_tui::warp::types::{WarpMode, WarpStatus};

#[tokio::test]
async fn test_client_creation() {
    // Test that clients can be created with default and custom timeouts
    let client = WarpClient::new();
    let client_with_timeout = WarpClient::with_timeout(60);

    // Just verify the clients can be created - we'll test timeout behavior in integration tests
    assert!(matches!(client, WarpClient { .. }));
    assert!(matches!(client_with_timeout, WarpClient { .. }));
}

#[test]
fn test_status_parsing() {
    let client = WarpClient::new();
    // set mode to warp+dot for testing
    client.set_mode_sync("warp+doh").unwrap();

    // Test connected status with new format
    let output = "Status update: Connected\nMode: Warp+DoH\nAccount type: Free";
    let info = client.parse_status_output(output).unwrap();
    assert_eq!(info.status, WarpStatus::Connected);
    assert_eq!(info.mode, Some(WarpMode::WarpDoH));
    assert_eq!(info.account_type, Some("Free".to_string()));

    // Test disconnected status with new format
    let output = "Status update: Disconnected\nReason: Settings Changed";
    let info = client.parse_status_output(output).unwrap();
    assert_eq!(info.status, WarpStatus::Disconnected);

    // Test connecting status
    let output = "Status update: Connecting\nReason: Checking for a route to the DNS endpoint";
    let info = client.parse_status_output(output).unwrap();
    assert_eq!(info.status, WarpStatus::Connecting);

    // Test backwards compatibility with old format
    let output = "Status: Connected\nMode: Warp+DoH";
    let info = client.parse_status_output(output).unwrap();
    assert_eq!(info.status, WarpStatus::Connected);
}

#[test]
fn test_status_line_parsing() {
    let client = WarpClient::new();

    // Test new format
    assert_eq!(
        client.parse_status_line("Status update: Connected"),
        WarpStatus::Connected
    );
    assert_eq!(
        client.parse_status_line("Status update: Disconnected"),
        WarpStatus::Disconnected
    );
    assert_eq!(
        client.parse_status_line("Status update: Connecting"),
        WarpStatus::Connecting
    );
    assert_eq!(
        client.parse_status_line("Status update: Disconnecting"),
        WarpStatus::Disconnecting
    );

    // Test case insensitive
    assert_eq!(
        client.parse_status_line("status update: connected"),
        WarpStatus::Connected
    );
    assert_eq!(
        client.parse_status_line("STATUS UPDATE: DISCONNECTED"),
        WarpStatus::Disconnected
    );

    // Test backwards compatibility with old format
    assert_eq!(
        client.parse_status_line("Status: Connected"),
        WarpStatus::Connected
    );
    assert_eq!(
        client.parse_status_line("Status: Disconnected"),
        WarpStatus::Disconnected
    );

    // Test unknown status
    assert_eq!(
        client.parse_status_line("Status update: Unknown"),
        WarpStatus::Unknown
    );
    assert_eq!(
        client.parse_status_line("Some other text"),
        WarpStatus::Unknown
    );
}
