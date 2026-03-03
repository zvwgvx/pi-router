use crate::approval::{DeviceState, SharedRegistry};
use crate::config::RouterConfig;
use crate::error::RouterError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tracing::{error, info, warn};

pub const SOCKET_PATH: &str = "/tmp/pi-router.sock";

// ─── Wire protocol (newline-delimited JSON) ────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(tag = "cmd", rename_all = "lowercase")]
pub enum Request {
    List,
    Approve { mac: String },
    Deny    { mac: String },
    Status,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Response {
    Ok    { ok: bool, message: String },
    Devices { ok: bool, devices: Vec<serde_json::Value> },
    StatusInfo { ok: bool, version: String, socket: String },
}

// ─── Server ───────────────────────────────────────────────────────────────────

/// Run a Unix-socket command server.
/// Accepts JSON commands from `pictl` and performs approve/deny actions.
pub async fn run(
    registry: SharedRegistry,
    cfg: Arc<RouterConfig>,
) -> Result<(), RouterError> {
    // Remove stale socket file from a previous run
    let _ = std::fs::remove_file(SOCKET_PATH);

    let listener = UnixListener::bind(SOCKET_PATH)
        .map_err(|e| RouterError::Daemon(format!("cannot bind socket {SOCKET_PATH}: {e}")))?;

    info!(socket = SOCKET_PATH, "API server listening");

    loop {
        let (stream, _) = match listener.accept().await {
            Ok(s) => s,
            Err(e) => { error!("accept: {e}"); continue; }
        };

        let reg = Arc::clone(&registry);
        let c   = Arc::clone(&cfg);

        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, reg, c).await {
                warn!("client error: {e}");
            }
        });
    }
}

async fn handle_client(
    stream: tokio::net::UnixStream,
    registry: SharedRegistry,
    cfg: Arc<RouterConfig>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    while let Some(line) = lines.next_line().await? {
        let line = line.trim().to_string();
        if line.is_empty() { continue; }

        let request: Request = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp = serde_json::json!({"ok": false, "message": format!("invalid command: {e}")});
                writer.write_all(format!("{resp}\n").as_bytes()).await?;
                continue;
            }
        };

        let response = dispatch(request, &registry, &cfg);
        writer.write_all(format!("{}\n", serde_json::to_string(&response)?).as_bytes()).await?;
    }
    Ok(())
}

fn dispatch(
    req: Request,
    registry: &SharedRegistry,
    cfg: &RouterConfig,
) -> serde_json::Value {
    let wan = &cfg.wan.interface;
    let ap  = &cfg.ap.interface;

    match req {
        Request::List => {
            let devices = registry.lock().unwrap().list();
            let items: Vec<serde_json::Value> = devices.iter().map(|d| serde_json::json!({
                "mac":       d.mac,
                "ip":        d.ip,
                "hostname":  d.hostname,
                "state":     d.state,
                "first_seen": d.first_seen,
                "last_seen":  d.last_seen,
            })).collect();
            serde_json::json!({ "ok": true, "devices": items })
        }

        Request::Approve { mac } => {
            let mac = mac.to_lowercase();
            let result = registry.lock().unwrap().approve(&mac, wan, ap);
            match result {
                Ok(())  => {
                    info!(mac, "✓ Device approved via API");
                    serde_json::json!({ "ok": true, "message": format!("{mac} approved") })
                }
                Err(e)  => serde_json::json!({ "ok": false, "message": e.to_string() }),
            }
        }

        Request::Deny { mac } => {
            let mac = mac.to_lowercase();
            let result = registry.lock().unwrap().deny(&mac, wan, ap);
            match result {
                Ok(())  => {
                    info!(mac, "✗ Device denied via API");
                    serde_json::json!({ "ok": true, "message": format!("{mac} denied") })
                }
                Err(e)  => serde_json::json!({ "ok": false, "message": e.to_string() }),
            }
        }

        Request::Status => {
            serde_json::json!({
                "ok": true,
                "version": env!("CARGO_PKG_VERSION"),
                "socket":  SOCKET_PATH,
            })
        }
    }
}
