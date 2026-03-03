use crate::error::RouterError;
use std::process::Command;
use tracing::{debug, info};

/// Flush all IP addresses from an interface.
pub fn flush_ip(iface: &str) -> Result<(), RouterError> {
    info!(iface, "Flushing existing IP addresses");
    run_cmd("ip", &["addr", "flush", "dev", iface])
}

/// Bring an interface UP (link layer).
pub fn set_link_up(iface: &str) -> Result<(), RouterError> {
    info!(iface, "Bringing interface UP");
    run_cmd("ip", &["link", "set", iface, "up"])
}

/// Bring an interface DOWN.
pub fn set_link_down(iface: &str) -> Result<(), RouterError> {
    info!(iface, "Bringing interface DOWN");
    run_cmd("ip", &["link", "set", iface, "down"])
}

/// Assign a static IP address to an interface.
///
/// Equivalent to: `ip addr add <ip>/<prefix_len> dev <iface>`
pub fn assign_ip(iface: &str, ip: &str, prefix_len: u8) -> Result<(), RouterError> {
    let cidr = format!("{ip}/{prefix_len}");
    info!(iface, cidr, "Assigning IP address");
    run_cmd("ip", &["addr", "add", &cidr, "dev", iface])
}

// ─── Internal helper ─────────────────────────────────────────────────────────

#[cfg(not(target_os = "macos"))]
fn run_cmd(prog: &str, args: &[&str]) -> Result<(), RouterError> {
    debug!(cmd = prog, ?args, "Running command");
    let output = Command::new(prog)
        .args(args)
        .output()
        .map_err(|e| RouterError::Network(format!("failed to execute `{prog}`: {e}")))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(RouterError::Network(format!(
            "`{prog} {}` failed ({}): {stderr}",
            args.join(" "),
            output.status
        )))
    }
}

#[cfg(target_os = "macos")]
fn run_cmd(prog: &str, args: &[&str]) -> Result<(), RouterError> {
    tracing::warn!("MacOS detected. Mocking command: {prog} {}", args.join(" "));
    Ok(())
}
