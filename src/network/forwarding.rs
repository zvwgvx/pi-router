use crate::error::RouterError;
use std::fs;
use tracing::info;

const IP_FORWARD_PATH: &str = "/proc/sys/net/ipv4/ip_forward";

/// Enable kernel IP forwarding (required for routing between interfaces).
#[cfg(not(target_os = "macos"))]
pub fn enable() -> Result<(), RouterError> {
    info!("Enabling IPv4 forwarding");
    fs::write(IP_FORWARD_PATH, b"1\n").map_err(|e| {
        RouterError::Network(format!("cannot enable ip_forward: {e}"))
    })
}

#[cfg(not(target_os = "macos"))]
pub fn disable() -> Result<(), RouterError> {
    info!("Disabling IPv4 forwarding");
    fs::write(IP_FORWARD_PATH, b"0\n").map_err(|e| {
        RouterError::Network(format!("cannot disable ip_forward: {e}"))
    })
}

#[cfg(not(target_os = "macos"))]
pub fn is_enabled() -> bool {
    fs::read_to_string(IP_FORWARD_PATH)
        .map(|v| v.trim() == "1")
        .unwrap_or(false)
}

#[cfg(target_os = "macos")]
pub fn enable() -> Result<(), RouterError> {
    tracing::warn!("MacOS detected. Mocking IPv4 forwarding enable.");
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn disable() -> Result<(), RouterError> {
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn is_enabled() -> bool {
    true
}
