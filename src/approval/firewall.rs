use crate::error::RouterError;
use std::process::Command;
use tracing::{info, warn, debug};

/// Install a default-deny rule for all AP clients trying to reach the WAN.
/// This is inserted as the LAST rule in FORWARD for the AP→WAN flow,
/// after any per-MAC ACCEPT rules that get inserted at the front.
pub fn install_default_deny(wan: &str, ap: &str) -> Result<(), RouterError> {
    info!(wan, ap, "Installing default-deny for unapproved AP clients");
    ipt(&[
        "-A", "FORWARD",
        "-i", ap,
        "-o", wan,
        "-j", "DROP",
    ])
}

/// Remove the default-deny rule (shutdown cleanup).
pub fn remove_default_deny(wan: &str, ap: &str) {
    let _ = ipt(&[
        "-D", "FORWARD",
        "-i", ap,
        "-o", wan,
        "-j", "DROP",
    ]);
}

/// Insert a per-MAC ACCEPT rule at the top of FORWARD.
/// Because rules are evaluated in order, this fires before the default-deny.
pub fn allow(mac: &str, wan: &str, ap: &str) -> Result<(), RouterError> {
    info!(mac, wan, ap, "Allowing device");
    ipt(&[
        "-I", "FORWARD", "1",
        "-i", ap,
        "-o", wan,
        "-m", "mac",
        "--mac-source", mac,
        "-j", "ACCEPT",
    ])
}

/// Remove the per-MAC ACCEPT rule (deny or revoke).
pub fn revoke(mac: &str, wan: &str, ap: &str) {
    info!(mac, "Revoking device access");
    let _ = ipt(&[
        "-D", "FORWARD",
        "-i", ap,
        "-o", wan,
        "-m", "mac",
        "--mac-source", mac,
        "-j", "ACCEPT",
    ]);
}

/// Remove ALL per-MAC ACCEPT rules that pi-router added (shutdown cleanup).
pub fn revoke_all(macs: &[String], wan: &str, ap: &str) {
    for mac in macs {
        revoke(mac, wan, ap);
    }
    remove_default_deny(wan, ap);
}

// ─── Internal ─────────────────────────────────────────────────────────────────

#[cfg(not(target_os = "macos"))]
fn ipt(args: &[&str]) -> Result<(), RouterError> {
    debug!(?args, "iptables");
    let out = Command::new("iptables")
        .args(args)
        .output()
        .map_err(|e| RouterError::Network(format!("iptables: {e}")))?;

    if out.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr);
        // Silently ignore "does not exist" on -D (already removed)
        if args.contains(&"-D") && (stderr.contains("does a rule exist") || stderr.contains("No chain/target")) {
            warn!("iptables -D: rule already gone — {stderr}");
            return Ok(());
        }
        Err(RouterError::Network(format!(
            "iptables {} failed: {stderr}",
            args.join(" ")
        )))
    }
}

#[cfg(target_os = "macos")]
fn ipt(args: &[&str]) -> Result<(), RouterError> {
    tracing::warn!("MacOS detected. Mocking iptables: {}", args.join(" "));
    Ok(())
}
