use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarpStatus {
    Connected,
    Disconnected,
    Connecting,
    Disconnecting,
    Unknown,
}

impl WarpStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Connected | Self::Connecting)
    }

    pub fn from_label(value: &str) -> Self {
        match value.trim().to_lowercase().as_str() {
            "connected" => Self::Connected,
            "disconnected" => Self::Disconnected,
            "connecting" => Self::Connecting,
            "disconnecting" => Self::Disconnecting,
            _ => Self::Unknown,
        }
    }
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
    Warp,
    DoH,
    DoT,
    WarpDoH,
    WarpDoT,
    Proxy,
    TunnelOnly,
    Unknown,
}

impl WarpMode {
    pub fn from_label(value: &str) -> Self {
        match value.to_lowercase().replace(' ', "").as_str() {
            "warp" => Self::Warp,
            "doh" => Self::DoH,
            "dot" => Self::DoT,
            "warp+doh" | "warpdoh" => Self::WarpDoH,
            "warp+dot" | "warpdot" => Self::WarpDoT,
            "proxy" => Self::Proxy,
            "tunnel_only" | "tunnelonly" => Self::TunnelOnly,
            _ => Self::Unknown,
        }
    }

    pub fn cli_name(&self) -> &'static str {
        match self {
            Self::Warp => "warp",
            Self::DoH => "doh",
            Self::DoT => "dot",
            Self::WarpDoH => "warp+doh",
            Self::WarpDoT => "warp+dot",
            Self::Proxy => "proxy",
            Self::TunnelOnly => "tunnel_only",
            Self::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for WarpMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WarpMode::Warp => write!(f, "WARP"),
            WarpMode::DoH => write!(f, "DoH"),
            WarpMode::DoT => write!(f, "DoT"),
            WarpMode::WarpDoH => write!(f, "Warp+DoH"),
            WarpMode::WarpDoT => write!(f, "Warp+DoT"),
            WarpMode::Proxy => write!(f, "Proxy"),
            WarpMode::TunnelOnly => write!(f, "Tunnel only"),
            WarpMode::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedEntry {
    pub value: String,
    pub description: String,
}

impl NamedEntry {
    pub fn display_label(&self) -> String {
        if self.description.is_empty() {
            self.value.clone()
        } else {
            format!("{}  {}", self.description, self.value)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarpInfo {
    pub status: WarpStatus,
    pub reason: Option<String>,
    pub mode: Option<WarpMode>,
    pub account_type: Option<String>,
    pub organization: Option<String>,
    pub device_id: Option<String>,
    pub account_id: Option<String>,
    pub public_key: Option<String>,
    pub vnet_name: Option<String>,
    pub vnet_description: Option<String>,
    pub always_on: bool,
    pub switch_locked: bool,
    pub auto_connect_after_min: Option<u32>,
    pub split_tunnel_mode: Option<String>,
    pub split_tunnels: Vec<NamedEntry>,
    pub fallback_domains: Vec<String>,
    pub warp_enabled: bool,
    pub gateway_enabled: bool,
    pub connected_networks: Vec<String>,
}

impl Default for WarpInfo {
    fn default() -> Self {
        Self {
            status: WarpStatus::Unknown,
            reason: None,
            mode: None,
            account_type: None,
            organization: None,
            device_id: None,
            account_id: None,
            public_key: None,
            vnet_name: None,
            vnet_description: None,
            always_on: false,
            switch_locked: false,
            auto_connect_after_min: None,
            split_tunnel_mode: None,
            split_tunnels: Vec::new(),
            fallback_domains: Vec::new(),
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
