use crate::approval::{DeviceState, SharedRegistry};
use crate::config::RouterConfig;
use axum::{
    extract::{Path, State},
    http::{HeaderValue, Method, StatusCode},
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

// ─── Shared app state ─────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct AppState {
    pub registry: SharedRegistry,
    pub config: std::sync::Arc<std::sync::Mutex<crate::config::RouterConfig>>,
    pub start_time: std::time::Instant,
    pub sys_monitor: std::sync::Arc<std::sync::Mutex<crate::sys_stats::SystemMonitor>>,
}

// ─── Router ───────────────────────────────────────────────────────────────────

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers(Any)
        .allow_origin(Any);

    Router::new()
        // System
        .route("/api/status",                  get(get_status))
        // Devices
        .route("/api/devices",                 get(list_devices))
        .route("/api/devices/:mac/approve",    post(approve_device))
        .route("/api/devices/:mac/deny",       post(deny_device))
        .route("/api/devices/:mac",            delete(delete_device))
        // Config
        .route("/api/config",                  get(get_config))
        .route("/api/config",                  put(put_config))
        // Firewall
        .route("/api/firewall",                get(get_firewall))
        .route("/api/firewall",                post(add_firewall_rule))
        .route("/api/firewall",                delete(del_firewall_rule))
        // NAT
        .route("/api/nat",                     get(get_nat))
        .route("/api/nat",                     post(add_nat_rule))
        .route("/api/nat",                     delete(del_nat_rule))
        // System / Stats
        .route("/api/interfaces",              get(get_interfaces))
        .route("/api/system",                  post(post_system))
        .layer(cors)
        .with_state(state)
}

/// Start the HTTP server on `addr` (e.g. "0.0.0.0:8080").
pub async fn serve(state: AppState, addr: &str) -> Result<(), crate::error::RouterError> {
    let app    = build_router(state);
    let listener = tokio::net::TcpListener::bind(addr).await
        .map_err(|e| crate::error::RouterError::Daemon(format!("HTTP bind {addr}: {e}")))?;
    info!(addr, "Web API listening");
    axum::serve(listener, app).await
        .map_err(|e| crate::error::RouterError::Daemon(format!("HTTP serve: {e}")))
}

// ─── Handlers: Status ─────────────────────────────────────────────────────────

async fn get_status(State(s): State<AppState>) -> Json<Value> {
    let uptime = s.start_time.elapsed().as_secs();
    let cfg = s.config.lock().unwrap();
    let reg = s.registry.lock().unwrap();

    let total    = reg.devices.len();
    let pending  = reg.devices.values().filter(|d| d.state == DeviceState::Pending).count();
    let approved = reg.devices.values().filter(|d| d.state == DeviceState::Approved).count();
    let denied   = reg.devices.values().filter(|d| d.state == DeviceState::Denied).count();

    let req_wan = cfg.wan.interface.clone();
    let stats = {
        let mut mon = s.sys_monitor.lock().unwrap();
        mon.get_stats(&req_wan)
    };

    Json(json!({
        "version":  env!("CARGO_PKG_VERSION"),
        "uptime":   uptime,
        "wan":      cfg.wan.interface,
        "ap":       cfg.ap.interface,
        "ssid":     cfg.ap.ssid,
        "ap_ip":    cfg.dhcp.ap_ip,
        "devices":  { "total": total, "pending": pending, "approved": approved, "denied": denied },
        "system":   stats
    }))
}

// ─── Handlers: Devices ────────────────────────────────────────────────────────

async fn list_devices(State(s): State<AppState>) -> Json<Value> {
    let reg   = s.registry.lock().unwrap();
    let list  = reg.list();
    Json(json!({ "ok": true, "devices": list }))
}

async fn approve_device(
    State(s): State<AppState>,
    Path(mac): Path<String>,
) -> (StatusCode, Json<Value>) {
    let cfg = s.config.lock().unwrap();
    let wan = cfg.wan.interface.clone();
    let ap  = cfg.ap.interface.clone();
    drop(cfg);

    match s.registry.lock().unwrap().approve(&mac, &wan, &ap) {
        Ok(_)  => (StatusCode::OK, Json(json!({ "ok": true, "message": format!("{mac} approved") }))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({ "ok": false, "message": e.to_string() }))),
    }
}

async fn deny_device(
    State(s): State<AppState>,
    Path(mac): Path<String>,
) -> (StatusCode, Json<Value>) {
    let cfg = s.config.lock().unwrap();
    let wan = cfg.wan.interface.clone();
    let ap  = cfg.ap.interface.clone();
    drop(cfg);

    match s.registry.lock().unwrap().deny(&mac, &wan, &ap) {
        Ok(_)  => (StatusCode::OK, Json(json!({ "ok": true, "message": format!("{mac} denied") }))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({ "ok": false, "message": e.to_string() }))),
    }
}

async fn delete_device(
    State(s): State<AppState>,
    Path(mac): Path<String>,
) -> (StatusCode, Json<Value>) {
    let mut reg = s.registry.lock().unwrap();
    if reg.devices.remove(&mac).is_some() {
        (StatusCode::OK, Json(json!({ "ok": true })))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({ "ok": false, "message": "device not found" })))
    }
}

// ─── Handlers: Config ─────────────────────────────────────────────────────────

async fn get_config(State(s): State<AppState>) -> Json<Value> {
    let cfg = s.config.lock().unwrap().clone();
    Json(serde_json::to_value(&cfg).unwrap_or(json!({})))
}

async fn get_interfaces(State(s): State<AppState>) -> Json<Vec<String>> {
    let mut mon = s.sys_monitor.lock().unwrap();
    let mut interfaces = mon.get_interfaces();
    interfaces.sort();
    Json(interfaces)
}

#[derive(Deserialize)]
struct SystemAction {
    action: String,
}

async fn post_system(
    State(_s): State<AppState>,
    Json(payload): Json<SystemAction>,
) -> (StatusCode, Json<Value>) {
    match payload.action.as_str() {
        "restart_service" => {
            tracing::warn!("Restart daemon requested via API");
            // Spawn a task to exit after a short delay so the response can be sent
            tokio::spawn(async {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                std::process::exit(0);
            });
            (StatusCode::OK, Json(json!({ "ok": true, "message": "restarting" })))
        }
        _ => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "message": "unknown action" })),
        ),
    }
}

#[derive(Deserialize)]
struct ConfigUpdate {
    ssid:        Option<String>,
    password:    Option<String>,
    channel:     Option<u8>,
    hw_mode:     Option<String>,
    country_code:Option<String>,
    log_level:   Option<String>,
}

async fn put_config(
    State(s): State<AppState>,
    Json(body): Json<ConfigUpdate>,
) -> (StatusCode, Json<Value>) {
    let mut cfg = s.config.lock().unwrap();
    if let Some(v) = body.ssid        { cfg.ap.ssid        = v; }
    if let Some(v) = body.password    { cfg.ap.password     = v; }
    if let Some(v) = body.channel     { cfg.ap.channel      = v; }
    if let Some(v) = body.hw_mode     { cfg.ap.hw_mode      = v; }
    if let Some(v) = body.country_code{ cfg.ap.country_code = v; }
    if let Some(v) = body.log_level   { cfg.log_level       = v; }
    (StatusCode::OK, Json(json!({ "ok": true, "message": "Config updated (restart daemons to apply)" })))
}

// ─── Handlers: Firewall ───────────────────────────────────────────────────────

async fn get_firewall() -> Json<Value> {
    Json(json!({ "ok": true, "rules": parse_iptables_chain("FORWARD") }))
}

#[derive(Deserialize)]
struct FirewallRule {
    iface_in:  Option<String>,
    iface_out: Option<String>,
    mac:       Option<String>,
    target:    String,   // ACCEPT | DROP | REJECT
}

async fn add_firewall_rule(
    State(s): State<AppState>,
    Json(r): Json<FirewallRule>,
) -> (StatusCode, Json<Value>) {
    let mut args = vec!["-A".to_string(), "FORWARD".to_string()];
    if let Some(i) = &r.iface_in  { args.extend(["-i".into(), i.clone()]); }
    if let Some(o) = &r.iface_out { args.extend(["-o".into(), o.clone()]); }
    if let Some(m) = &r.mac       { args.extend(["-m".into(), "mac".into(), "--mac-source".into(), m.clone()]); }
    args.extend(["-j".to_string(), r.target.clone()]);

    match run_iptables(&args) {
        Ok(_)  => (StatusCode::OK,          Json(json!({ "ok": true }))),
        Err(e) => (StatusCode::BAD_REQUEST,  Json(json!({ "ok": false, "message": e }))),
    }
}

#[derive(Deserialize)]
struct RuleDelete { rule_num: u32 }

async fn del_firewall_rule(Json(r): Json<RuleDelete>) -> (StatusCode, Json<Value>) {
    match run_iptables(&["-D", "FORWARD", &r.rule_num.to_string()]) {
        Ok(_)  => (StatusCode::OK,         Json(json!({ "ok": true }))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({ "ok": false, "message": e }))),
    }
}

// ─── Handlers: NAT ────────────────────────────────────────────────────────────

async fn get_nat() -> Json<Value> {
    Json(json!({ "ok": true, "rules": parse_iptables_nat() }))
}

#[derive(Deserialize)]
struct NatRule {
    iface_out: Option<String>,
    target:    String,        // MASQUERADE | SNAT
    to_source: Option<String>,// for SNAT
}

async fn add_nat_rule(Json(r): Json<NatRule>) -> (StatusCode, Json<Value>) {
    let mut args = vec!["-t".to_string(), "nat".to_string(), "-A".to_string(), "POSTROUTING".to_string()];
    if let Some(o) = &r.iface_out { args.extend(["-o".into(), o.clone()]); }
    args.extend(["-j".to_string(), r.target.clone()]);
    if let Some(src) = &r.to_source { args.extend(["--to-source".into(), src.clone()]); }

    match run_iptables(&args) {
        Ok(_)  => (StatusCode::OK,         Json(json!({ "ok": true }))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({ "ok": false, "message": e }))),
    }
}

async fn del_nat_rule(Json(r): Json<RuleDelete>) -> (StatusCode, Json<Value>) {
    match run_iptables(&["-t", "nat", "-D", "POSTROUTING", &r.rule_num.to_string()]) {
        Ok(_)  => (StatusCode::OK,         Json(json!({ "ok": true }))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({ "ok": false, "message": e }))),
    }
}

// ─── iptables helpers ─────────────────────────────────────────────────────────

#[cfg(not(target_os = "macos"))]
fn parse_iptables_chain(chain: &str) -> Vec<Value> {
    let out = std::process::Command::new("iptables")
        .args(["-L", chain, "-n", "--line-numbers", "-v"])
        .output();
    match out {
        Err(_) => vec![],
        Ok(o)  => String::from_utf8_lossy(&o.stdout)
            .lines()
            .skip(2) // skip header lines
            .filter(|l| !l.trim().is_empty())
            .map(|l| json!({ "raw": l.trim() }))
            .collect(),
    }
}

#[cfg(target_os = "macos")]
fn parse_iptables_chain(_chain: &str) -> Vec<Value> { vec![] }

#[cfg(not(target_os = "macos"))]
fn parse_iptables_nat() -> Vec<Value> {
    let out = std::process::Command::new("iptables")
        .args(["-t", "nat", "-L", "POSTROUTING", "-n", "--line-numbers", "-v"])
        .output();
    match out {
        Err(_) => vec![],
        Ok(o)  => String::from_utf8_lossy(&o.stdout)
            .lines()
            .skip(2)
            .filter(|l| !l.trim().is_empty())
            .map(|l| json!({ "raw": l.trim() }))
            .collect(),
    }
}

#[cfg(target_os = "macos")]
fn parse_iptables_nat() -> Vec<Value> { vec![] }

#[cfg(not(target_os = "macos"))]
fn run_iptables(args: &[impl AsRef<str>]) -> Result<(), String> {
    let args: Vec<&str> = args.iter().map(|a| a.as_ref()).collect();
    let out = std::process::Command::new("iptables")
        .args(&args)
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).to_string())
    }
}

#[cfg(target_os = "macos")]
fn run_iptables(_args: &[impl AsRef<str>]) -> Result<(), String> { Ok(()) }
