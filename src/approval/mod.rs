pub mod store;
pub mod firewall;

use crate::error::RouterError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceState {
    /// Seen in DHCP leases — waiting for admin decision.
    Pending,
    /// Admin approved: iptables ACCEPT rule is active.
    Approved,
    /// Admin denied: traffic continues to be dropped.
    Denied,
}

/// Information about a client that has connected to the AP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// MAC address (lower-case, colon-separated: "aa:bb:cc:dd:ee:ff")
    pub mac: String,
    /// Last known IP from DHCP lease
    pub ip: String,
    /// Hostname reported by the client (may be empty)
    pub hostname: String,
    /// Current approval state
    pub state: DeviceState,
    /// Unix timestamp of first-seen
    pub first_seen: u64,
    /// Unix timestamp of last DHCP lease renewal
    pub last_seen: u64,
}

// ─── Registry ─────────────────────────────────────────────────────────────────

/// Thread-safe registry of all clients ever seen on the AP.
pub type SharedRegistry = Arc<Mutex<DeviceRegistry>>;

pub struct DeviceRegistry {
    /// MAC → DeviceInfo
    pub devices: HashMap<String, DeviceInfo>,
    /// Path to persist state
    pub store_path: String,
}

impl DeviceRegistry {
    pub fn new(store_path: &str) -> Self {
        let devices = store::load(store_path).unwrap_or_default();
        Self { devices, store_path: store_path.to_string() }
    }

    /// Upsert a device seen in DHCP leases.
    /// Returns true if this is a brand-new device (just became Pending).
    pub fn upsert(&mut self, mac: &str, ip: &str, hostname: &str) -> bool {
        let now = now_secs();
        if let Some(dev) = self.devices.get_mut(mac) {
            dev.ip        = ip.to_string();
            dev.hostname  = hostname.to_string();
            dev.last_seen = now;
            false
        } else {
            self.devices.insert(mac.to_string(), DeviceInfo {
                mac:        mac.to_string(),
                ip:         ip.to_string(),
                hostname:   hostname.to_string(),
                state:      DeviceState::Pending,
                first_seen: now,
                last_seen:  now,
            });
            self.persist();
            true
        }
    }

    /// Approve a device by MAC — stores state and applies iptables rule.
    pub fn approve(
        &mut self,
        mac: &str,
        wan: &str,
        ap: &str,
    ) -> Result<(), RouterError> {
        let dev = self.devices.get_mut(mac)
            .ok_or_else(|| RouterError::Config(format!("unknown device: {mac}")))?;

        if dev.state == DeviceState::Approved {
            return Ok(()); // already approved
        }
        firewall::allow(mac, wan, ap)?;
        dev.state = DeviceState::Approved;
        self.persist();
        Ok(())
    }

    /// Deny a device by MAC — revokes iptables rule if it had been approved.
    pub fn deny(
        &mut self,
        mac: &str,
        wan: &str,
        ap: &str,
    ) -> Result<(), RouterError> {
        let dev = self.devices.get_mut(mac)
            .ok_or_else(|| RouterError::Config(format!("unknown device: {mac}")))?;

        if dev.state == DeviceState::Approved {
            firewall::revoke(mac, wan, ap);
        }
        dev.state = DeviceState::Denied;
        self.persist();
        Ok(())
    }

    /// List all devices (cloned, safe to send across threads).
    pub fn list(&self) -> Vec<DeviceInfo> {
        let mut v: Vec<DeviceInfo> = self.devices.values().cloned().collect();
        v.sort_by(|a, b| a.first_seen.cmp(&b.first_seen));
        v
    }

    /// Restore approved devices' iptables rules after a restart.
    pub fn restore_firewall(&self, wan: &str, ap: &str) {
        for dev in self.devices.values() {
            if dev.state == DeviceState::Approved {
                let _ = firewall::allow(&dev.mac, wan, ap);
            }
        }
    }

    fn persist(&self) {
        let _ = store::save(&self.store_path, &self.devices);
    }
}

fn now_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}
