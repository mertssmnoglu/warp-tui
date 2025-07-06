use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarpStatus {
    Connected,
    Disconnected,
    Connecting,
    Disconnecting,
    Unknown,
}

impl std::fmt::Display for WarpStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WarpStatus::Connected => write!(f, "Connected"),
            WarpStatus::Disconnected => write!(f, "Disconnected"),
            WarpStatus::Connecting => write!(f, "Connecting"),
            WarpStatus::Disconnecting => write!(f, "Disconnecting"),
            WarpStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarpMode {
    DoH,     // DNS over HTTPS
    DoT,     // DNS over TLS
    WarpDoH, // Warp + DNS over HTTPS
    WarpDoT, // Warp + DNS over TLS
    Unknown,
}

impl std::fmt::Display for WarpMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WarpMode::DoH => write!(f, "DoH"),
            WarpMode::DoT => write!(f, "DoT"),
            WarpMode::WarpDoH => write!(f, "Warp+DoH"),
            WarpMode::WarpDoT => write!(f, "Warp+DoT"),
            WarpMode::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarpInfo {
    pub status: WarpStatus,
    pub mode: Option<WarpMode>,
    pub account_type: Option<String>,
    pub warp_enabled: bool,
    pub gateway_enabled: bool,
    pub connected_networks: Vec<String>,
}

impl Default for WarpInfo {
    fn default() -> Self {
        Self {
            status: WarpStatus::Unknown,
            mode: None,
            account_type: None,
            warp_enabled: false,
            gateway_enabled: false,
            connected_networks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationInfo {
    pub device_id: Option<String>,
    pub organization: Option<String>,
    pub account_type: Option<String>,
    pub license_key: Option<String>,
}
