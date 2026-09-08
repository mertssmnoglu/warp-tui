use serde::Deserialize;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use tokio::process::Command as AsyncCommand;
use tokio::time::timeout;

use crate::warp::error::{WarpError, WarpResult};
use crate::warp::setup::{default_cli_name, resolve_warp_cli};
use crate::warp::types::{NamedEntry, RegistrationInfo, WarpInfo, WarpMode, WarpStatus};

#[derive(Debug, Deserialize)]
struct StatusJson {
    status: String,
    #[serde(default)]
    reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WarpSettingsFile {
    settings: SettingsJson,
}

#[derive(Debug, Deserialize)]
struct SettingsJson {
    #[serde(default)]
    always_on: bool,
    #[serde(default)]
    switch_locked: bool,
    auto_connect_after_min: Option<u32>,
    operation_mode: Option<String>,
    split_tunnel_mode: Option<String>,
    #[serde(default)]
    split_tunnel_hosts: Vec<NamedValueJson>,
    #[serde(default)]
    split_tunnel_ips: Vec<NamedValueJson>,
    #[serde(default)]
    fallback_domains: Vec<FallbackDomainJson>,
    organization: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NamedValueJson {
    value: String,
    #[serde(default)]
    description: String,
}

#[derive(Debug, Deserialize)]
struct FallbackDomainJson {
    #[serde(alias = "value")]
    domain: String,
}

#[derive(Debug, Deserialize)]
struct RegistrationJson {
    device_id: Option<String>,
    public_key: Option<String>,
    account: Option<RegistrationAccountJson>,
}

#[derive(Debug, Deserialize)]
struct RegistrationAccountJson {
    #[serde(rename = "type")]
    account_type: Option<String>,
    id: Option<String>,
    organization: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VnetJson {
    active_vnet_id: Option<String>,
    #[serde(default)]
    virtual_networks: Vec<VnetEntryJson>,
}

#[derive(Debug, Deserialize)]
struct VnetEntryJson {
    id: String,
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default, rename = "default")]
    is_default: bool,
}

#[derive(Clone, Debug)]
pub struct WarpClient {
    #[allow(dead_code)] // Used for async operations which may be used in the future
    command_timeout: Duration,
    binary: PathBuf,
}

impl Default for WarpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl WarpClient {
    pub fn new() -> Self {
        Self::with_binary(resolve_warp_cli().unwrap_or_else(|| PathBuf::from(default_cli_name())))
    }

    pub fn with_binary(binary: PathBuf) -> Self {
        Self {
            command_timeout: Duration::from_secs(30),
            binary,
        }
    }

    #[allow(dead_code)] // May be used in future async implementations
    pub fn with_timeout(timeout_secs: u64) -> Self {
        Self {
            command_timeout: Duration::from_secs(timeout_secs),
            binary: resolve_warp_cli().unwrap_or_else(|| PathBuf::from(default_cli_name())),
        }
    }

    fn command(&self) -> Command {
        Command::new(&self.binary)
    }

    fn async_command(&self) -> AsyncCommand {
        AsyncCommand::new(&self.binary)
    }

    fn map_spawn_err(e: std::io::Error) -> WarpError {
        if e.kind() == std::io::ErrorKind::NotFound {
            WarpError::CommandNotFound
        } else {
            WarpError::IoError(e)
        }
    }

    fn run_ok(&self, args: &[&str]) -> WarpResult<String> {
        let output = self
            .command()
            .args(args)
            .output()
            .map_err(Self::map_spawn_err)?;
        if !output.status.success() {
            return Err(WarpError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Check if warp-cli is available in the system
    #[allow(dead_code)] // May be used in future async implementations
    pub async fn is_available(&self) -> bool {
        match self.async_command().arg("--version").output().await {
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

        let command_future = self.async_command().args(args).output();

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
        self.run_ok(&["mode", mode])?;
        Ok(())
    }

    /// Get the current operation mode from warp-cli settings
    pub fn get_operation_mode(&self) -> WarpResult<WarpMode> {
        let stdout = self.run_ok(&["--json", "settings"])?;
        let settings: WarpSettingsFile =
            serde_json::from_str(&stdout).map_err(|e| WarpError::ParseError(e.to_string()))?;
        Ok(WarpMode::from_label(
            settings
                .settings
                .operation_mode
                .as_deref()
                .unwrap_or_default(),
        ))
    }

    /// Get the current operation mode asynchronously
    pub async fn get_operation_mode_async(&self) -> WarpResult<WarpMode> {
        let output = timeout(
            self.command_timeout,
            self.async_command().args(["--json", "settings"]).output(),
        )
        .await
        .map_err(|e| WarpError::Timeout(e.to_string()))??;

        if !output.status.success() {
            return Err(WarpError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let settings: WarpSettingsFile = serde_json::from_slice(&output.stdout)
            .map_err(|e| WarpError::ParseError(e.to_string()))?;

        Ok(WarpMode::from_label(
            settings
                .settings
                .operation_mode
                .as_deref()
                .unwrap_or_default(),
        ))
    }

    /// Parse the status command output into WarpInfo struct
    pub fn parse_status_output(&self, output: &str) -> WarpResult<WarpInfo> {
        let mut info = WarpInfo::default();

        for line in output.lines() {
            let line = line.trim();

            if line.starts_with("Status update:") || line.contains("Status:") {
                info.status = self.parse_status_line(line);
            } else if line.contains("Account type:") {
                info.account_type = self.extract_value_after_colon(line);
            } else if line.starts_with("Reason:") {
                info.reason = self.extract_value_after_colon(line);
            } else if let Some(mode) = line
                .strip_prefix("Mode:")
                .or_else(|| line.split("Mode:").nth(1))
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                info.mode = Some(WarpMode::from_label(mode));
            } else if line.contains("Warp enabled:") {
                info.warp_enabled = line.contains("true");
            } else if line.contains("Gateway enabled:") {
                info.gateway_enabled = line.contains("true");
            }
        }

        Ok(info)
    }

    /// Parse status from a status line
    pub fn parse_status_line(&self, line: &str) -> WarpStatus {
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

    /// Synchronous snapshot for the TUI dashboard
    pub fn get_status_sync(&self) -> WarpResult<WarpInfo> {
        let mut info = match self.run_ok(&["--json", "status"]) {
            Ok(stdout) => self.parse_status_json(&stdout)?,
            Err(_) => {
                let stdout = self.run_ok(&["status"])?;
                self.parse_status_output(&stdout)?
            }
        };

        if let Ok(stdout) = self.run_ok(&["--json", "settings"]) {
            self.apply_settings_json(&mut info, &stdout)?;
        } else if let Ok(mode) = self.get_operation_mode() {
            info.mode = Some(mode);
        }

        if let Ok(stdout) = self.run_ok(&["--json", "registration", "show"]) {
            self.apply_registration_json(&mut info, &stdout);
        } else if info.organization.is_none() {
            info.organization = self.organization_sync();
        }

        if let Ok(stdout) = self.run_ok(&["--json", "vnet"]) {
            self.apply_vnet_json(&mut info, &stdout);
        }

        Ok(info)
    }

    fn parse_status_json(&self, stdout: &str) -> WarpResult<WarpInfo> {
        let parsed: StatusJson =
            serde_json::from_str(stdout).map_err(|e| WarpError::ParseError(e.to_string()))?;
        let mut info = WarpInfo {
            status: WarpStatus::from_label(&parsed.status),
            reason: parsed.reason.filter(|s| !s.is_empty()),
            ..Default::default()
        };
        if info.status == WarpStatus::Unknown {
            let text = self.run_ok(&["status"])?;
            info = self.parse_status_output(&text)?;
        }
        Ok(info)
    }

    fn apply_settings_json(&self, info: &mut WarpInfo, stdout: &str) -> WarpResult<()> {
        let parsed: WarpSettingsFile =
            serde_json::from_str(stdout).map_err(|e| WarpError::ParseError(e.to_string()))?;
        let settings = parsed.settings;
        info.always_on = settings.always_on;
        info.switch_locked = settings.switch_locked;
        info.auto_connect_after_min = settings.auto_connect_after_min;
        if let Some(mode) = settings.operation_mode {
            info.mode = Some(WarpMode::from_label(&mode));
        }
        info.split_tunnel_mode = settings.split_tunnel_mode;
        let mut tunnels = Vec::new();
        for entry in settings
            .split_tunnel_ips
            .into_iter()
            .chain(settings.split_tunnel_hosts)
        {
            tunnels.push(NamedEntry {
                value: entry.value,
                description: entry.description,
            });
        }
        info.split_tunnels = tunnels;
        info.fallback_domains = settings
            .fallback_domains
            .into_iter()
            .map(|d| d.domain)
            .filter(|d| !d.is_empty())
            .collect();
        if info.organization.is_none() {
            info.organization = settings.organization;
        }
        Ok(())
    }

    fn apply_registration_json(&self, info: &mut WarpInfo, stdout: &str) {
        let Ok(parsed) = serde_json::from_str::<RegistrationJson>(stdout) else {
            return;
        };
        info.device_id = parsed.device_id.filter(|s| !s.is_empty());
        info.public_key = parsed.public_key.filter(|s| !s.is_empty());
        if let Some(account) = parsed.account {
            if account.account_type.is_some() {
                info.account_type = account.account_type;
            }
            info.account_id = account.id.filter(|s| !s.is_empty());
            if account.organization.is_some() {
                info.organization = account.organization;
            }
        }
    }

    fn apply_vnet_json(&self, info: &mut WarpInfo, stdout: &str) {
        let Ok(parsed) = serde_json::from_str::<VnetJson>(stdout) else {
            return;
        };
        let active = parsed.active_vnet_id.as_deref();
        let selected = parsed
            .virtual_networks
            .iter()
            .find(|v| Some(v.id.as_str()) == active)
            .or_else(|| parsed.virtual_networks.iter().find(|v| v.is_default))
            .or_else(|| parsed.virtual_networks.first());
        if let Some(entry) = selected {
            info.vnet_name = Some(entry.name.clone());
            if !entry.description.is_empty() {
                info.vnet_description = Some(entry.description.clone());
            }
        }
    }

    pub fn organization_sync(&self) -> Option<String> {
        let Ok(output) = self.run_ok(&["registration", "organization"]) else {
            return None;
        };
        if output.is_empty() {
            None
        } else {
            Some(output)
        }
    }

    pub fn enroll_sync(&self, team: &str) -> WarpResult<String> {
        self.run_ok(&["--accept-tos", "registration", "new", team])
    }

    pub fn delete_registration_sync(&self) -> WarpResult<()> {
        self.run_ok(&["registration", "delete"])?;
        Ok(())
    }

    pub fn registration_token_sync(&self, token: &str) -> WarpResult<String> {
        self.run_ok(&["--accept-tos", "registration", "token", token])
    }

    /// Synchronous version of connect for non-async contexts
    pub fn connect_sync(&self) -> WarpResult<()> {
        match self.run_ok(&["connect"]) {
            Ok(_) => Ok(()),
            Err(WarpError::CommandFailed(msg)) if msg.contains("already connected") => Ok(()),
            Err(WarpError::CommandFailed(msg)) => Err(WarpError::ConnectionFailed(msg)),
            Err(e) => Err(e),
        }
    }

    /// Synchronous version of disconnect for non-async contexts
    pub fn disconnect_sync(&self) -> WarpResult<()> {
        match self.run_ok(&["disconnect"]) {
            Ok(_) => Ok(()),
            Err(WarpError::CommandFailed(msg)) if msg.contains("already disconnected") => Ok(()),
            Err(WarpError::CommandFailed(msg)) => Err(WarpError::DisconnectionFailed(msg)),
            Err(e) => Err(e),
        }
    }
}
