# pi-router

A WiFi router daemon written in Rust. Turns any Linux device (Raspberry Pi, laptop, embedded board) into a full WiFi Access Point by routing traffic from a WAN interface through a WiFi AP interface.

---

## Features

- Single `config.json` controls all runtime parameters
- WPA2 WiFi hotspot via `hostapd` (2.4 GHz or 5 GHz)
- DHCP and DNS server via `dnsmasq`
- NAT masquerade via `iptables`
- Per-device approval: new clients are held pending until approved via CLI or Web UI
- Daemon health monitor with automatic restart on crash
- Graceful shutdown: cleans up iptables rules and terminates child processes on Ctrl-C or SIGTERM
- Structured logging via `tracing`, configurable log level
- Web dashboard for real-time monitoring and configuration

---

## Requirements

### System Dependencies

Debian / Ubuntu / Raspbian:

```
sudo apt install hostapd dnsmasq iptables iproute2
```

Arch Linux:

```
sudo pacman -S hostapd dnsmasq iptables iproute2
```

### Rust Toolchain

Requires Rust 1.75 or later:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

## Configuration

All runtime settings are controlled by `config.json`. A minimal example:

```json
{
  "wan": {
    "interface": "eth0"
  },
  "ap": {
    "interface": "wlan1",
    "ssid": "Pi-Router",
    "password": "supersecret",
    "channel": 6,
    "hw_mode": "g",
    "country_code": "VN",
    "ieee80211n": true,
    "ieee80211ax": false,
    "ignore_broadcast_ssid": false,
    "wmm_enabled": true
  },
  "dhcp": {
    "ap_ip": "192.168.100.1",
    "netmask": "255.255.255.0",
    "prefix_len": 24,
    "range_start": "192.168.100.10",
    "range_end": "192.168.100.200",
    "lease_time": "12h",
    "dns_servers": ["8.8.8.8", "1.1.1.1"]
  },
  "monitor": {
    "check_interval_secs": 5,
    "max_restart_attempts": 5
  },
  "log_level": "info"
}
```

### Key Parameters

| Field | Description |
|---|---|
| `wan.interface` | Network interface connected to the internet (e.g. `eth0`, `usb0`) |
| `ap.interface` | WiFi interface to broadcast the hotspot on (e.g. `wlan1`) |
| `ap.hw_mode` | `g` for 2.4 GHz, `a` for 5 GHz |
| `ap.ieee80211n` | Enable Wi-Fi 4 (802.11n) HT extensions |
| `ap.ieee80211ax` | Enable Wi-Fi 6 (802.11ax) HE extensions |
| `ap.ignore_broadcast_ssid` | Set to `true` to hide the SSID from scan results |
| `ap.country_code` | ISO 3166-1 alpha-2 country code for regulatory compliance |
| `monitor.check_interval_secs` | How often to health-check hostapd and dnsmasq |
| `monitor.max_restart_attempts` | Maximum restart attempts before fatal exit |
| `log_level` | One of: `trace`, `debug`, `info`, `warn`, `error` |

---

## Building

Debug build:

```
cargo build
```

Optimized release binary:

```
cargo build --release
```

---

## Running

Requires root privileges to configure network interfaces, install iptables rules, and spawn system daemons.

```
sudo ./target/release/pi-router --config config.json
```

With verbose logging:

```
sudo RUST_LOG=debug ./target/release/pi-router
```

### Web Dashboard

The web dashboard runs on port 3000 (frontend) and port 8080 (API):

```
cd web && npm install && npm run dev
```

Open `http://<device-ip>:3000` in a browser. The dashboard provides:

- Real-time system stats: CPU, RAM, temperature, disk, WAN bandwidth
- Device approval queue with allow/block actions
- Full configuration editor with live validation
- Firewall and NAT rule management

---

## How It Works

```
Internet
  |
  | WAN interface (eth0)
  |
  | iptables NAT MASQUERADE + FORWARD
  |
  | AP interface (wlan1) <-> hostapd (WPA2 access point)
  |
  | dnsmasq (DHCP pool + DNS resolver)
  |
  | Client devices
```

### Startup Sequence

| Step | Action |
|---|---|
| 1 | Read and validate `config.json` |
| 2 | Assign AP IP address, bring interface up |
| 3 | Enable `net.ipv4.ip_forward` |
| 4 | Install iptables NAT and FORWARD rules |
| 5 | Install default-deny rule for unapproved clients |
| 6 | Load device approval registry |
| 7 | Start `hostapd` |
| 8 | Start `dnsmasq` |
| 9 | Launch async health monitor and device watcher |

### Shutdown Sequence (Ctrl-C or SIGTERM)

| Step | Action |
|---|---|
| 1 | Stop monitor and watcher tasks |
| 2 | Terminate `dnsmasq` (SIGTERM then SIGKILL) |
| 3 | Terminate `hostapd` (SIGTERM then SIGKILL) |
| 4 | Remove iptables rules |
| 5 | Flush AP interface IP address |

---

## Device Approval

By default, all newly connected clients are blocked until explicitly approved. Use the `pictl` CLI or the Web UI to manage devices.

```
# Approve a device by MAC address
sudo pictl approve AA:BB:CC:DD:EE:FF

# Block a device
sudo pictl deny AA:BB:CC:DD:EE:FF

# List all known devices
sudo pictl list
```

---

## Project Structure

```
src/
  main.rs              Entry point, orchestration, signal handling
  config.rs            config.json schema and validation
  error.rs             RouterError unified error type
  sys_stats.rs         System metrics collection (CPU, RAM, temp, disk, network)
  monitor.rs           Async health-check loop with auto-restart
  watcher.rs           Device registry watcher
  api/
    mod.rs             Unix socket IPC API (used by pictl)
  http_api/
    mod.rs             HTTP REST API (used by web dashboard)
  approval/
    mod.rs             Per-device approval state and iptables rule management
    firewall.rs        Default-deny firewall rule management
    store.rs           Persistent device registry (JSON)
  network/
    interface.rs       ip addr and ip link commands
    forwarding.rs      IPv4 forwarding via /proc/sys/net/ipv4/ip_forward
    nat.rs             iptables NAT and MASQUERADE rules
  daemon/
    process.rs         Generic supervised child-process manager
    hostapd.rs         hostapd.conf generator and process manager
    dnsmasq.rs         dnsmasq.conf generator and process manager

web/                   Next.js web dashboard
  app/
    page.tsx           Overview and device management
    config/page.tsx    Configuration editor
    firewall/page.tsx  Firewall rule editor
    nat/page.tsx       NAT rule editor
    devices/page.tsx   Device management
  lib/api.ts           API client
```

---

## Troubleshooting

| Problem | Likely Cause | Fix |
|---|---|---|
| `hostapd` fails to start | Interface does not support AP mode | Run `iw list` and verify `AP` appears under `Supported interface modes` |
| `dnsmasq` port 53 conflict | System resolver is using port 53 | `sudo systemctl stop systemd-resolved` |
| Clients get an IP but no internet | IPv4 forwarding is not active | `cat /proc/sys/net/ipv4/ip_forward` must output `1` |
| `iptables` errors on startup | Missing kernel modules | `sudo modprobe iptable_nat ip_conntrack` |
| Client stuck in Pending state | Default-deny rule is active | Approve via `pictl approve <MAC>` or the Web UI |

---

## License

GPL-3.0
