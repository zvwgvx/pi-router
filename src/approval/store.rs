use crate::approval::DeviceInfo;
use crate::error::RouterError;
use std::collections::HashMap;
use tracing::{debug, warn};

const BACKUP_SUFFIX: &str = ".bak";

/// Load devices from a JSON file. Returns empty map on any error.
pub fn load(path: &str) -> Option<HashMap<String, DeviceInfo>> {
    let raw = std::fs::read_to_string(path).ok()?;
    match serde_json::from_str(&raw) {
        Ok(map) => {
            debug!(path, "Loaded device registry");
            Some(map)
        }
        Err(e) => {
            warn!(path, err = %e, "Failed to parse device store — starting fresh");
            None
        }
    }
}

/// Persist devices to a JSON file (atomic write via temp-then-rename).
pub fn save(
    path: &str,
    devices: &HashMap<String, DeviceInfo>,
) -> Result<(), RouterError> {
    let json = serde_json::to_string_pretty(devices)
        .map_err(|e| RouterError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    // Write to backup first
    let backup = format!("{path}{BACKUP_SUFFIX}");
    std::fs::write(&backup, &json)?;
    std::fs::rename(&backup, path)?;
    debug!(path, "Saved device registry ({} devices)", devices.len());
    Ok(())
}
