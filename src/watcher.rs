use crate::approval::{DeviceState, SharedRegistry};
use tokio::time::{interval, Duration};
use tracing::{debug, info, warn};

const LEASE_FILE: &str = "/tmp/pi-router-dnsmasq.leases";

/// Watches the dnsmasq lease file every `poll_secs` seconds.
/// On each tick, parses the current leases and syncs newly seen clients
/// into the registry as `Pending` (or auto-approves if require_approval is false).
///
/// dnsmasq lease file format (space-separated):
///   <expiry_timestamp> <mac> <ip> <hostname> <client-id>
pub async fn run(
    registry: SharedRegistry,
    poll_secs: u64,
    require_approval: bool,
    wan: String,
    ap: String,
) {
    let mut tick = interval(Duration::from_secs(poll_secs));
    tick.tick().await; // skip first immediate tick

    loop {
        tick.tick().await;

        let leases = match read_leases() {
            Some(l) => l,
            None => continue,
        };

        for (mac, ip, hostname) in leases {
            let is_new = {
                let mut reg = registry.lock().unwrap();
                reg.upsert(&mac, &ip, &hostname)
            };

            if is_new {
                if require_approval {
                    info!(mac, ip, hostname, "New device connected — PENDING approval");
                } else {
                    // Auto-approve immediately
                    let mut reg = registry.lock().unwrap();
                    let _ = reg.approve(&mac, &wan, &ap);
                    info!(mac, ip, hostname, "New device connected — auto-approved");
                }
            } else {
                debug!(mac, ip, "Lease refreshed");
            }
        }
    }
}

// ─── Lease parser ─────────────────────────────────────────────────────────────

fn read_leases() -> Option<Vec<(String, String, String)>> {
    let raw = match std::fs::read_to_string(LEASE_FILE) {
        Ok(r) => r,
        Err(e) => {
            // file may not exist yet if no clients have connected
            debug!(err = %e, "Cannot read lease file (no clients yet?)");
            return None;
        }
    };

    let mut leases = Vec::new();
    for line in raw.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        // expected: timestamp mac ip hostname client-id
        if parts.len() < 4 {
            warn!("Unexpected lease line: {line}");
            continue;
        }
        let mac      = parts[1].to_lowercase();
        let ip       = parts[2].to_string();
        let hostname = if parts[3] == "*" { String::new() } else { parts[3].to_string() };
        leases.push((mac, ip, hostname));
    }
    Some(leases)
}
