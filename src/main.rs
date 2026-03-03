mod config;
mod error;
mod network;
mod daemon;
mod monitor;
mod approval;
mod watcher;
mod api;
mod http_api;
mod sys_stats;

use config::RouterConfig;
use approval::{DeviceRegistry, SharedRegistry};
use daemon::{hostapd::HostapdManager, dnsmasq::DnsmasqManager};
use error::RouterError;
use std::{path::PathBuf, sync::{Arc, Mutex}};
use clap::Parser;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt};

// ─── CLI ──────────────────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(
    name    = "pi-router",
    version,
    about   = "Professional Rust WiFi router daemon with device approval",
)]
struct Cli {
    /// Path to config JSON file
    #[arg(short, long, default_value = "config.json")]
    config: PathBuf,
}

// ─── Main ─────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config_path_str = cli.config.to_string_lossy().to_string();

    let cfg = match RouterConfig::load(&cli.config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[pi-router] FATAL: {e}");
            std::process::exit(1);
        }
    };

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&cfg.log_level));
    fmt().with_env_filter(filter).with_target(true).compact().init();

    info!(
        version = env!("CARGO_PKG_VERSION"),
        config  = %cli.config.display(),
        wan     = %cfg.wan.interface,
        ap      = %cfg.ap.interface,
        ssid    = %cfg.ap.ssid,
        "pi-router starting"
    );

    if let Err(e) = run(cfg, config_path_str).await {
        error!("Fatal error: {e}");
        std::process::exit(1);
    }
}

// ─── Orchestration ────────────────────────────────────────────────────────────

async fn run(cfg: RouterConfig, config_path: String) -> Result<(), RouterError> {
    let cfg = Arc::new(cfg);
    // Wrap config in Mutex for mutable HTTP API access
    let cfg_mutex: Arc<Mutex<RouterConfig>> = Arc::new(Mutex::new((*cfg).clone()));

    // 1. Network setup ────────────────────────────────────────────────────────
    info!("── Step 1/5: Configuring AP interface ──");
    network::interface::flush_ip(&cfg.ap.interface)?;
    network::interface::set_link_up(&cfg.ap.interface)?;
    network::interface::assign_ip(
        &cfg.ap.interface,
        &cfg.dhcp.ap_ip,
        cfg.dhcp.prefix_len,
    )?;

    // 2. IP forwarding ────────────────────────────────────────────────────────
    info!("── Step 2/5: Enabling IP forwarding ──");
    network::forwarding::enable()?;

    // 3. NAT rules + default-deny per AP client ───────────────────────────────
    info!("── Step 3/5: Installing NAT + default-deny rules ──");
    network::nat::setup(&cfg.wan.interface, &cfg.ap.interface)?;
    // Default-deny: new clients get WiFi + DHCP but NO internet until approved
    approval::firewall::install_default_deny(&cfg.wan.interface, &cfg.ap.interface)?;

    // 4. Device registry ───────────────────────────────────────────────────────
    info!("── Step 4/5: Loading device registry ({}) ──", cfg.approval.devices_store);
    let registry: SharedRegistry = Arc::new(Mutex::new(
        DeviceRegistry::new(&cfg.approval.devices_store)
    ));
    // Restore iptables ACCEPT rules for previously-approved devices
    {
        let reg = registry.lock().unwrap();
        reg.restore_firewall(&cfg.wan.interface, &cfg.ap.interface);
        info!(
            approved = reg.devices.values().filter(|d| d.state == approval::DeviceState::Approved).count(),
            "Restored approved devices"
        );
    }

    // 5. Daemons ───────────────────────────────────────────────────────────────
    info!("── Step 5/5: Starting hostapd + dnsmasq ──");

    let hostapd = Arc::new(Mutex::new(HostapdManager::new()));
    let dnsmasq = Arc::new(Mutex::new(DnsmasqManager::new()));

    { hostapd.lock().unwrap().start(&cfg)?; }
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    { dnsmasq.lock().unwrap().start(&cfg)?; }

    info!(
        ssid   = %cfg.ap.ssid,
        ap_ip  = %cfg.dhcp.ap_ip,
        "✓ Router is UP — new clients need approval via `pictl approve <MAC>` or the Web UI"
    );

    // ── HTTP API server ───────────────────────────────────────────────────────
    let sys_monitor = Arc::new(Mutex::new(sys_stats::SystemMonitor::new()));
    let http_state = http_api::AppState {
        registry:    Arc::clone(&registry),
        config:      Arc::clone(&cfg_mutex),
        config_path,
        start_time:  std::time::Instant::now(),
        sys_monitor,
    };
    let http_addr  = cfg.http_api.listen_addr.clone();
    let _http = tokio::spawn(async move {
        if let Err(e) = http_api::serve(http_state, &http_addr).await {
            error!("HTTP API: {e}");
        }
    });

    // ── Background tasks ──────────────────────────────────────────────────────

    // Health monitor
    let mon_hp  = Arc::clone(&hostapd);
    let mon_dm  = Arc::clone(&dnsmasq);
    let mon_cfg = cfg.monitor.clone();
    let _monitor = tokio::spawn(async move {
        if let Err(e) = monitor::run(mon_hp, mon_dm, mon_cfg).await {
            error!("Monitor: {e}");
        }
    });

    // Lease watcher — detects new clients → marks Pending
    let watch_reg       = Arc::clone(&registry);
    let watch_secs      = cfg.monitor.check_interval_secs;
    let watch_require   = cfg.approval.require_approval;
    let watch_wan       = cfg.wan.interface.clone();
    let watch_ap        = cfg.ap.interface.clone();
    let _watcher = tokio::spawn(async move {
        watcher::run(watch_reg, watch_secs, watch_require, watch_wan, watch_ap).await;
    });

    // API server — Unix socket for pictl commands
    let api_reg = Arc::clone(&registry);
    let api_cfg = Arc::clone(&cfg);
    let _api = tokio::spawn(async move {
        if let Err(e) = api::run(api_reg, api_cfg).await {
            error!("API: {e}");
        }
    });

    // ── Graceful shutdown ─────────────────────────────────────────────────────
    shutdown_signal().await;
    info!("Shutdown signal received — cleaning up…");

    // Stop daemons
    { dnsmasq.lock().unwrap().stop(); }
    { hostapd.lock().unwrap().stop(); }

    // Revoke all approved device rules + default-deny
    {
        let reg = registry.lock().unwrap();
        let approved_macs: Vec<String> = reg.devices.values()
            .filter(|d| d.state == approval::DeviceState::Approved)
            .map(|d| d.mac.clone())
            .collect();
        approval::firewall::revoke_all(&approved_macs, &cfg.wan.interface, &cfg.ap.interface);
    }

    network::nat::teardown(&cfg.wan.interface, &cfg.ap.interface);
    let _ = network::interface::flush_ip(&cfg.ap.interface);
    let _ = std::fs::remove_file(api::SOCKET_PATH);

    info!("pi-router stopped cleanly");
    Ok(())
}

// ─── Signal helpers ───────────────────────────────────────────────────────────

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm = signal(SignalKind::terminate())
            .expect("failed to register SIGTERM handler");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => { info!("Received SIGINT (Ctrl-C)"); }
            _ = sigterm.recv()          => { info!("Received SIGTERM"); }
        }
    }
    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await.ok();
        info!("Received Ctrl-C");
    }
}
