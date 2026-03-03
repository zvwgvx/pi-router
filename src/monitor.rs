use crate::{
    config::MonitorConfig,
    daemon::{hostapd::HostapdManager, dnsmasq::DnsmasqManager},
    error::RouterError,
};
use std::sync::{Arc, Mutex};
use tokio::time::{interval, Duration};
use tracing::{debug, error, warn};

/// Async health-monitor: periodically checks whether `hostapd` and `dnsmasq`
/// are alive; restarts them if they have crashed. Gives up after
/// `max_restart_attempts` consecutive failures per daemon.
pub async fn run(
    hostapd:   Arc<Mutex<HostapdManager>>,
    dnsmasq:   Arc<Mutex<DnsmasqManager>>,
    mon_cfg:   MonitorConfig,
) -> Result<(), RouterError> {
    let mut tick = interval(Duration::from_secs(mon_cfg.check_interval_secs));
    tick.tick().await; // skip first immediate tick

    loop {
        tick.tick().await;

        // ── hostapd ──────────────────────────────────────────────────────────
        {
            let mut hp = hostapd.lock().unwrap();
            if !hp.is_alive() {
                if hp.restart_count() >= mon_cfg.max_restart_attempts {
                    error!(
                        daemon = "hostapd",
                        restarts = hp.restart_count(),
                        "Exceeded max restart attempts — aborting"
                    );
                    return Err(RouterError::Monitor(
                        "hostapd exceeded max restart attempts".into(),
                    ));
                }
                warn!(daemon = "hostapd", "Detected crash — restarting");
                hp.restart()?;
            } else {
                debug!(daemon = "hostapd", "OK");
            }
        }

        // ── dnsmasq ──────────────────────────────────────────────────────────
        {
            let mut dm = dnsmasq.lock().unwrap();
            if !dm.is_alive() {
                if dm.restart_count() >= mon_cfg.max_restart_attempts {
                    error!(
                        daemon = "dnsmasq",
                        restarts = dm.restart_count(),
                        "Exceeded max restart attempts — aborting"
                    );
                    return Err(RouterError::Monitor(
                        "dnsmasq exceeded max restart attempts".into(),
                    ));
                }
                warn!(daemon = "dnsmasq", "Detected crash — restarting");
                dm.restart()?;
            } else {
                debug!(daemon = "dnsmasq", "OK");
            }
        }
    }
}
