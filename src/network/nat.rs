use crate::error::RouterError;
use std::process::Command;
use tracing::{debug, info, warn};

/// Set up iptables rules to NAT traffic from `ap_iface` out through `wan_iface`.
///
/// Rules added:
///  - `POSTROUTING MASQUERADE` on wan (NAT)
///  - `FORWARD` accept from ap → wan
///  - `FORWARD` accept from wan → ap (ESTABLISHED/RELATED)
pub fn setup(wan_iface: &str, ap_iface: &str) -> Result<(), RouterError> {
    info!(wan = wan_iface, ap = ap_iface, "Setting up NAT/FORWARD rules");

    // MASQUERADE outbound traffic on WAN interface
    ipt(&[
        "-t", "nat", "-A", "POSTROUTING",
        "-o", wan_iface,
        "-j", "MASQUERADE",
    ])?;

    // Allow forwarding from AP → WAN
    ipt(&[
        "-A", "FORWARD",
        "-i", ap_iface,
        "-o", wan_iface,
        "-j", "ACCEPT",
    ])?;

    // Allow established/related traffic returning from WAN → AP
    ipt(&[
        "-A", "FORWARD",
        "-i", wan_iface,
        "-o", ap_iface,
        "-m", "state",
        "--state", "RELATED,ESTABLISHED",
        "-j", "ACCEPT",
    ])?;

    Ok(())
}

/// Remove the iptables rules created by `setup`.
pub fn teardown(wan_iface: &str, ap_iface: &str) {
    info!(wan = wan_iface, ap = ap_iface, "Tearing down NAT/FORWARD rules");

    let _ = ipt(&[
        "-t", "nat", "-D", "POSTROUTING",
        "-o", wan_iface,
        "-j", "MASQUERADE",
    ]);

    let _ = ipt(&[
        "-D", "FORWARD",
        "-i", ap_iface,
        "-o", wan_iface,
        "-j", "ACCEPT",
    ]);

    let _ = ipt(&[
        "-D", "FORWARD",
        "-i", wan_iface,
        "-o", ap_iface,
        "-m", "state",
        "--state", "RELATED,ESTABLISHED",
        "-j", "ACCEPT",
    ]);
}

// ─── Internal ─────────────────────────────────────────────────────────────────

#[cfg(not(target_os = "macos"))]
fn ipt(args: &[&str]) -> Result<(), RouterError> {
    debug!(?args, "iptables");
    let output = Command::new("iptables")
        .args(args)
        .output()
        .map_err(|e| RouterError::Network(format!("iptables exec failed: {e}")))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Non-fatal on Delete (-D) when rule doesn't exist
        if args.contains(&"-D") {
            warn!("iptables -D may have had nothing to delete: {stderr}");
            Ok(())
        } else {
            Err(RouterError::Network(format!(
                "iptables {} failed: {stderr}",
                args.join(" ")
            )))
        }
    }
}

#[cfg(target_os = "macos")]
fn ipt(args: &[&str]) -> Result<(), RouterError> {
    tracing::warn!("MacOS detected. Mocking iptables: {}", args.join(" "));
    Ok(())
}
