use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::error::RouterError;

// ─── Top-level config ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    pub wan: WanConfig,
    pub ap: ApConfig,
    pub dhcp: DhcpConfig,
    pub monitor: MonitorConfig,
    #[serde(default)]
    pub approval: ApprovalConfig,
    #[serde(default)]
    pub http_api: HttpApiConfig,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_log_level() -> String {
    "info".to_string()
}

// ─── WAN interface ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WanConfig {
    /// Source interface for internet traffic (e.g. "eth0", "usb0", "wlan0")
    pub interface: String,
}

// ─── Access Point ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApConfig {
    /// Interface to broadcast WiFi on (e.g. "wlan1")
    pub interface: String,
    /// SSID to broadcast
    pub ssid: String,
    /// WPA2 pre-shared key (min 8 chars)
    pub password: String,
    /// WiFi channel (1-14 for 2.4 GHz, 36-165 for 5 GHz)
    #[serde(default = "default_channel")]
    pub channel: u8,
    /// hostapd hw_mode: "g" = 2.4 GHz 802.11g, "a" = 5 GHz, "n" = HT
    #[serde(default = "default_hw_mode")]
    pub hw_mode: String,
    /// ISO 3166-1 alpha-2 country code for regulatory compliance
    #[serde(default = "default_country")]
    pub country_code: String,
    /// Enable 802.11n (HT) extensions
    #[serde(default = "default_true")]
    pub ieee80211n: bool,
    /// Enable 802.11ax (Wi-Fi 6) extensions
    #[serde(default = "default_false")]
    pub ieee80211ax: bool,
    /// Hide SSID broadcast
    #[serde(default = "default_false")]
    pub ignore_broadcast_ssid: bool,
    /// Enable WMM (QoS for multimedia)
    #[serde(default = "default_true")]
    pub wmm_enabled: bool,
}

fn default_channel() -> u8 { 6 }
fn default_hw_mode() -> String { "g".to_string() }
fn default_country() -> String { "US".to_string() }
fn default_true() -> bool { true }
fn default_false() -> bool { false }

// ─── DHCP / DNS ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhcpConfig {
    /// IP to assign to the AP interface itself (gateway for clients)
    pub ap_ip: String,
    /// Subnet mask (e.g. "255.255.255.0")
    pub netmask: String,
    /// CIDR prefix length (e.g. 24 for /24)
    pub prefix_len: u8,
    /// Start of DHCP lease pool
    pub range_start: String,
    /// End of DHCP lease pool
    pub range_end: String,
    /// DHCP lease duration (e.g. "12h", "infinite")
    #[serde(default = "default_lease")]
    pub lease_time: String,
    /// DNS servers pushed to clients
    #[serde(default = "default_dns")]
    pub dns_servers: Vec<String>,
}

fn default_lease() -> String { "12h".to_string() }
fn default_dns() -> Vec<String> { vec!["8.8.8.8".to_string(), "1.1.1.1".to_string()] }

// ─── Monitor ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    /// Seconds between health-check ticks
    #[serde(default = "default_interval")]
    pub check_interval_secs: u64,
    /// How many consecutive restart failures before giving up
    #[serde(default = "default_restarts")]
    pub max_restart_attempts: u32,
}

fn default_interval() -> u64 { 5 }
fn default_restarts() -> u32 { 5 }

// ─── Approval ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalConfig {
    /// Path to the JSON file storing device state across restarts
    #[serde(default = "default_devices_store")]
    pub devices_store: String,
}

impl Default for ApprovalConfig {
    fn default() -> Self {
        Self { devices_store: default_devices_store() }
    }
}

fn default_devices_store() -> String {
    "/etc/pi-router/devices.json".to_string()
}

// ─── HTTP API ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpApiConfig {
    /// Address the HTTP REST API listens on
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,
}

impl Default for HttpApiConfig {
    fn default() -> Self {
        Self { listen_addr: default_listen_addr() }
    }
}

fn default_listen_addr() -> String {
    "0.0.0.0:8080".to_string()
}

// ─── Loading + Validation ─────────────────────────────────────────────────────

impl RouterConfig {
    /// Load and parse config.json from `path`.
    pub fn load(path: &Path) -> Result<Self, RouterError> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| RouterError::Config(format!("cannot read {}: {e}", path.display())))?;
        let cfg: RouterConfig = serde_json::from_str(&raw)
            .map_err(|e| RouterError::Config(format!("invalid JSON in {}: {e}", path.display())))?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// Validate semantic constraints on the configuration.
    pub fn validate(&self) -> Result<(), RouterError> {
        if self.wan.interface.is_empty() {
            return Err(RouterError::Config("wan.interface must not be empty".into()));
        }
        if self.ap.interface.is_empty() {
            return Err(RouterError::Config("ap.interface must not be empty".into()));
        }
        if self.wan.interface == self.ap.interface {
            return Err(RouterError::Config(
                "wan.interface and ap.interface must be different".into(),
            ));
        }
        if self.ap.ssid.is_empty() || self.ap.ssid.len() > 32 {
            return Err(RouterError::Config(
                "ap.ssid must be 1–32 characters".into(),
            ));
        }
        if self.ap.password.len() < 8 {
            return Err(RouterError::Config(
                "ap.password must be at least 8 characters (WPA2 requirement)".into(),
            ));
        }
        if self.dhcp.dns_servers.is_empty() {
            return Err(RouterError::Config(
                "dhcp.dns_servers must contain at least one entry".into(),
            ));
        }
        Ok(())
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn valid_json() -> &'static str {
        r#"{
          "wan":  { "interface": "eth0" },
          "ap":   { "interface": "wlan1", "ssid": "TestNet", "password": "password123" },
          "dhcp": {
            "ap_ip": "192.168.100.1", "netmask": "255.255.255.0", "prefix_len": 24,
            "range_start": "192.168.100.10", "range_end": "192.168.100.200"
          },
          "monitor": {}
        }"#
    }

    #[test]
    fn parses_valid_config() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(valid_json().as_bytes()).unwrap();
        let cfg = RouterConfig::load(f.path()).unwrap();
        assert_eq!(cfg.wan.interface, "eth0");
        assert_eq!(cfg.ap.ssid, "TestNet");
        assert_eq!(cfg.dhcp.prefix_len, 24);
    }

    #[test]
    fn rejects_short_password() {
        let json = r#"{
          "wan":  { "interface": "eth0" },
          "ap":   { "interface": "wlan1", "ssid": "Net", "password": "short" },
          "dhcp": {
            "ap_ip": "192.168.100.1", "netmask": "255.255.255.0", "prefix_len": 24,
            "range_start": "192.168.100.10", "range_end": "192.168.100.200"
          },
          "monitor": {}
        }"#;
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(json.as_bytes()).unwrap();
        assert!(RouterConfig::load(f.path()).is_err());
    }

    #[test]
    fn rejects_same_interface() {
        let json = r#"{
          "wan":  { "interface": "wlan0" },
          "ap":   { "interface": "wlan0", "ssid": "Net", "password": "password123" },
          "dhcp": {
            "ap_ip": "192.168.100.1", "netmask": "255.255.255.0", "prefix_len": 24,
            "range_start": "192.168.100.10", "range_end": "192.168.100.200"
          },
          "monitor": {}
        }"#;
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(json.as_bytes()).unwrap();
        assert!(RouterConfig::load(f.path()).is_err());
    }
}
