use serde::Deserialize;
use std::process::Command;
use std::time::Duration;
use tokio::process::Command as AsyncCommand;
use tokio::time::timeout;

use crate::warp::error::{WarpError, WarpResult};
use crate::warp::types::{RegistrationInfo, WarpInfo, WarpMode, WarpStatus};

#[derive(Debug, Deserialize)]
struct WarpSettings {
    settings: Settings,
}

#[derive(Debug, Deserialize)]
struct Settings {
    operation_mode: String,
}

#[derive(Clone, Debug)]
pub struct WarpClient {
    #[allow(dead_code)] // Used for async operations which may be used in the future
    command_timeout: Duration,
}

impl Default for WarpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl WarpClient {
    pub fn new() -> Self {
        Self {
            command_timeout: Duration::from_secs(30),
        }
    }

    #[allow(dead_code)] // May be used in future async implementations
    pub fn with_timeout(timeout_secs: u64) -> Self {
        Self {
            command_timeout: Duration::from_secs(timeout_secs),
        }
    }

    /// Check if warp-cli is available in the system
    #[allow(dead_code)] // May be used in future async implementations
    pub async fn is_available(&self) -> bool {
        match AsyncCommand::new("warp-cli")
            .arg("--version")
            .output()
            .await
        {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    /// Execute a warp-cli command with arguments
    #[allow(dead_code)] // May be used in future async implementations
    async fn execute_command(&self, args: &[&str]) -> WarpResult<String> {
        if !self.is_available().await {
            return Err(WarpError::CommandNotFound);
        }

        let command_future = AsyncCommand::new("warp-cli").args(args).output();

        let output = timeout(self.command_timeout, command_future)
            .await
            .map_err(|_| WarpError::CommandFailed("Command timed out".to_string()))?
            .map_err(WarpError::IoError)?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(WarpError::CommandFailed(error_msg.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().to_string())
    }

    /// Get current warp status and information
    #[allow(dead_code)] // May be used in future async implementations
    pub async fn get_status(&self) -> WarpResult<WarpInfo> {
        let output = self.execute_command(&["status"]).await?;
        self.parse_status_output(&output)
    }

    /// Create a new registration
    #[allow(dead_code)] // May be used in future async implementations
    pub async fn create_registration(&self) -> WarpResult<RegistrationInfo> {
        let output = self.execute_command(&["registration", "new"]).await?;
        self.parse_registration_output(&output)
    }

    /// Delete the current registration
    #[allow(dead_code)] // May be used in future async implementations
    pub async fn delete_registration(&self) -> WarpResult<()> {
        self.execute_command(&["registration", "delete"]).await?;
        Ok(())
    }

    /// Connect to warp
    #[allow(dead_code)] // May be used in future async implementations
    pub async fn connect(&self) -> WarpResult<()> {
        match self.execute_command(&["connect"]).await {
            Ok(_) => Ok(()),
            Err(WarpError::CommandFailed(msg)) => {
                if msg.contains("already connected") {
                    Ok(()) // Already connected is not an error
                } else {
                    Err(WarpError::ConnectionFailed(msg))
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Disconnect from warp
    #[allow(dead_code)] // May be used in future async implementations
    pub async fn disconnect(&self) -> WarpResult<()> {
        match self.execute_command(&["disconnect"]).await {
            Ok(_) => Ok(()),
            Err(WarpError::CommandFailed(msg)) => {
                if msg.contains("already disconnected") {
                    Ok(()) // Already disconnected is not an error
                } else {
                    Err(WarpError::DisconnectionFailed(msg))
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Get warp settings
    #[allow(dead_code)] // May be used in future async implementations
    pub async fn get_settings(&self) -> WarpResult<String> {
        self.execute_command(&["settings"]).await
    }

    /// Set DNS mode
    #[allow(dead_code)] // May be used in future async implementations
    pub async fn set_mode(&self, mode: &str) -> WarpResult<()> {
        self.execute_command(&["set-mode", mode]).await?;
        Ok(())
    }

    /// Set mode synchronously
    pub fn set_mode_sync(&self, mode: &str) -> WarpResult<()> {
        let output = Command::new("warp-cli")
            .args(["mode", mode])
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    WarpError::CommandNotFound
                } else {
                    WarpError::IoError(e)
                }
            })?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(WarpError::CommandFailed(error_msg.to_string()));
        }

        Ok(())
    }

    /// Get the current operation mode from warp-cli settings
    pub fn get_operation_mode(&self) -> WarpResult<WarpMode> {
        let output = Command::new("warp-cli")
            .args(["--json", "settings"])
            .output()
            .map_err(|e| WarpError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(WarpError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let settings: WarpSettings = serde_json::from_slice(&output.stdout)
            .map_err(|e| WarpError::ParseError(e.to_string()))?;

        Ok(match settings.settings.operation_mode.as_str() {
            "warp+dot" => WarpMode::WarpDoT,
            "dot" => WarpMode::DoT,
            "doh" => WarpMode::DoH,
            "warp+doh" => WarpMode::WarpDoH,
            _ => WarpMode::Unknown,
        })
    }

    /// Get the current operation mode asynchronously
    pub async fn get_operation_mode_async(&self) -> WarpResult<WarpMode> {
        let output = timeout(
            self.command_timeout,
            AsyncCommand::new("warp-cli")
                .args(["--json", "settings"])
                .output(),
        )
        .await
        .map_err(|e| WarpError::Timeout(e.to_string()))??;

        if !output.status.success() {
            return Err(WarpError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let settings: WarpSettings = serde_json::from_slice(&output.stdout)
            .map_err(|e| WarpError::ParseError(e.to_string()))?;

        Ok(match settings.settings.operation_mode.as_str() {
            "warp+dot" => WarpMode::WarpDoT,
            "dot" => WarpMode::DoT,
            "doh" => WarpMode::DoH,
            "warp+doh" => WarpMode::WarpDoH,
            _ => WarpMode::Unknown,
        })
    }

    /// Parse the status command output into WarpInfo struct
    fn parse_status_output(&self, output: &str) -> WarpResult<WarpInfo> {
        let mode = Some(self.get_operation_mode()?);
        let mut info = WarpInfo {
            mode,
            ..Default::default()
        };

        for line in output.lines() {
            let line = line.trim();

            if line.starts_with("Status update:") || line.contains("Status:") {
                info.status = self.parse_status_line(line);
            } else if line.contains("Account type:") {
                info.account_type = self.extract_value_after_colon(line);
            } else if line.contains("Warp enabled:") {
                info.warp_enabled = line.contains("true");
            } else if line.contains("Gateway enabled:") {
                info.gateway_enabled = line.contains("true");
            }
        }

        Ok(info)
    }

    /// Parse status from a status line
    fn parse_status_line(&self, line: &str) -> WarpStatus {
        let line_lower = line.to_lowercase();

        // First try the new "Status update:" format
        if line_lower.starts_with("status update:") {
            let status_part = line_lower.strip_prefix("status update:").unwrap().trim();
            match status_part {
                "connected" => WarpStatus::Connected,
                "disconnected" => WarpStatus::Disconnected,
                "connecting" => WarpStatus::Connecting,
                "disconnecting" => WarpStatus::Disconnecting,
                _ => WarpStatus::Unknown,
            }
        } else if line_lower.contains("status:") {
            // Handle the old "Status:" format
            let status_part = line_lower.split("status:").nth(1).unwrap_or("").trim();
            if status_part.contains("connected") && !status_part.contains("disconnected") {
                WarpStatus::Connected
            } else if status_part.contains("disconnected") {
                WarpStatus::Disconnected
            } else if status_part.contains("connecting") {
                WarpStatus::Connecting
            } else if status_part.contains("disconnecting") {
                WarpStatus::Disconnecting
            } else {
                WarpStatus::Unknown
            }
        } else {
            WarpStatus::Unknown
        }
    }

    // Operation mode is now handled by get_operation_mode() which uses the JSON output

    /// Extract value after colon from a line
    fn extract_value_after_colon(&self, line: &str) -> Option<String> {
        line.split(':')
            .nth(1)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    /// Parse registration command output
    #[allow(dead_code)] // May be used in future async implementations
    fn parse_registration_output(&self, output: &str) -> WarpResult<RegistrationInfo> {
        let mut info = RegistrationInfo {
            device_id: None,
            organization: None,
            account_type: None,
            license_key: None,
        };

        for line in output.lines() {
            let line = line.trim();

            if line.contains("Device ID:") {
                info.device_id = self.extract_value_after_colon(line);
            } else if line.contains("Organization:") {
                info.organization = self.extract_value_after_colon(line);
            } else if line.contains("Account type:") {
                info.account_type = self.extract_value_after_colon(line);
            } else if line.contains("License key:") {
                info.license_key = self.extract_value_after_colon(line);
            }
        }

        Ok(info)
    }

    /// Synchronous version of get_status for non-async contexts
    pub fn get_status_sync(&self) -> WarpResult<WarpInfo> {
        let output = Command::new("warp-cli")
            .arg("status")
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    WarpError::CommandNotFound
                } else {
                    WarpError::IoError(e)
                }
            })?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(WarpError::CommandFailed(error_msg.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        self.parse_status_output(stdout.trim())
    }

    /// Synchronous version of connect for non-async contexts
    pub fn connect_sync(&self) -> WarpResult<()> {
        let output = Command::new("warp-cli")
            .arg("connect")
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    WarpError::CommandNotFound
                } else {
                    WarpError::IoError(e)
                }
            })?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            if error_msg.contains("already connected") {
                Ok(()) // Already connected is not an error
            } else {
                Err(WarpError::ConnectionFailed(error_msg.to_string()))
            }
        } else {
            Ok(())
        }
    }

    /// Synchronous version of disconnect for non-async contexts
    pub fn disconnect_sync(&self) -> WarpResult<()> {
        let output = Command::new("warp-cli")
            .arg("disconnect")
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    WarpError::CommandNotFound
                } else {
                    WarpError::IoError(e)
                }
            })?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            if error_msg.contains("already disconnected") {
                Ok(()) // Already disconnected is not an error
            } else {
                Err(WarpError::DisconnectionFailed(error_msg.to_string()))
            }
        } else {
            Ok(())
        }
    }

    // ...existing code...
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = WarpClient::new();
        assert_eq!(client.command_timeout, Duration::from_secs(30));

        let client_with_timeout = WarpClient::with_timeout(60);
        assert_eq!(client_with_timeout.command_timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_status_parsing() {
        let client = WarpClient::new();

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

    // Test for operation mode has been moved to integration tests since it requires the warp-cli command

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
}
