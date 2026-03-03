use serde::Serialize;
use std::sync::Mutex;
use sysinfo::{Components, Disks, Networks, System};

#[derive(Serialize)]
pub struct SysStats {
    pub cpu_percent: f32,
    pub ram_total: u64,
    pub ram_used: u64,
    pub temp_c: f32,
    pub disk_total: u64,
    pub disk_used: u64,
    pub net_rx_bps: u64,
    pub net_tx_bps: u64,
}

pub struct SystemMonitor {
    sys: System,
    net: Networks,
    disks: Disks,
    comps: Components,
    last_update: std::time::Instant,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let mut sys = System::new();
        // Initial refresh
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        
        let net = Networks::new_with_refreshed_list();
        let disks = Disks::new_with_refreshed_list();
        let comps = Components::new_with_refreshed_list();

        Self {
            sys,
            net,
            disks,
            comps,
            last_update: std::time::Instant::now(),
        }
    }

    pub fn get_stats(&mut self, wan_iface: &str) -> SysStats {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f32().max(0.1);
        self.last_update = now;

        // Refresh required components
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();
        self.net.refresh(true); // refresh network interfaces and bytes
        self.disks.refresh(true);
        self.comps.refresh(true);

        // CPU: average across all cores
        let cpus = self.sys.cpus();
        let cpu_percent = if !cpus.is_empty() {
            cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / (cpus.len() as f32)
        } else {
            0.0
        };

        // Network: calculate bps based on bytes since last refresh.
        // Wait, sysinfo Networks already tracks received/transmitted since last refresh?
        // sysinfo 0.3x: `network.received()` returns bytes received since last refresh.
        let mut net_rx = 0;
        let mut net_tx = 0;
        if let Some(n) = self.net.iter().find(|(name, _)| *name == wan_iface) {
            net_rx = n.1.received();
            net_tx = n.1.transmitted();
        }

        // Disk: aggregate across all listed disks
        let mut disk_total = 0;
        let mut disk_used = 0;
        for d in &self.disks {
            disk_total += d.total_space();
            let used = d.total_space().saturating_sub(d.available_space());
            disk_used += used;
        }

        // Temp: attempt to find a sensible CPU temperature
        let mut temp_c = 0.0;
        let mut temp_count = 0;
        for c in &self.comps {
            if let Some(t) = c.temperature() {
                temp_c += t;
                temp_count += 1;
            }
        }
        if temp_count > 0 {
            temp_c /= temp_count as f32;
        }

        SysStats {
            cpu_percent,
            ram_total: self.sys.total_memory(),
            ram_used: self.sys.used_memory(),
            temp_c,
            disk_total,
            disk_used,
            net_rx_bps: (net_rx as f32 / elapsed) as u64,
            net_tx_bps: (net_tx as f32 / elapsed) as u64,
        }
    }

    pub fn get_interfaces(&mut self) -> Vec<String> {
        self.net.refresh(true);
        self.net.iter()
            .filter(|(name, data)| {
                let name = name.to_lowercase();
                // Allowlist common physical interface prefixes
                if !name.starts_with("en") && !name.starts_with("eth") && !name.starts_with("wlan") && !name.starts_with("usb") {
                    return false;
                }
                
                // Ensure the interface has a valid MAC address
                if data.mac_address() == sysinfo::MacAddr::UNSPECIFIED {
                    return false;
                }
                true
            })
            .map(|(name, _)| name.clone())
            .collect()
    }
}
