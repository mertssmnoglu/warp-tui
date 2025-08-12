use std::process::Command;

/// Helper function to check if warp-cli is available
pub fn is_warp_cli_available() -> bool {
    Command::new("warp-cli")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
