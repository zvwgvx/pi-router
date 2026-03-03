# pi-router

> A professional, production-grade **WiFi router daemon** written in Rust.  
> Turns any Linux device (Raspberry Pi, laptop, embedded board) into a full WiFi Access Point by bridging a WAN interface to a WiFi AP interface.

---

## Features

- ⚡ **Zero-copy config driven** — single `config.json` controls everything
- 📶 **WPA2 WiFi hotspot** via `hostapd` (2.4 GHz / 5 GHz)
- 🏠 **DHCP + DNS server** via `dnsmasq`
- 🔀 **NAT masquerade** via `iptables`
- 🛡️ **Health monitor** — auto-restarts crashed daemons
- 🧹 **Graceful shutdown** — cleans up iptables rules and processes on Ctrl-C / SIGTERM
- 📋 **Structured logging** via `tracing` (configurable level)

## Requirements

```bash
# Debian/Ubuntu/Raspbian
sudo apt install hostapd dnsmasq iptables iproute2

# Arch
sudo pacman -S hostapd dnsmasq iptables iproute2
```

Build requires Rust 1.75+:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Configuration

Edit `config.json` before running:

```json
{
  "wan": {
    "interface": "eth0"           // Source internet interface
  },
  "ap": {
    "interface": "wlan1",         // WiFi interface to broadcast on
    "ssid": "Pi-Router",          // Network name
    "password": "supersecret",    // WPA2 key (min 8 chars)
    "channel": 6,                 // 1-13 for 2.4 GHz, 36-165 for 5 GHz
    "hw_mode": "g",               // "g" = 2.4 GHz, "a" = 5 GHz
    "country_code": "VN",         // ISO country code (regulatory)
    "ieee80211n": true,
    "wmm_enabled": true
  },
  "dhcp": {
    "ap_ip": "192.168.100.1",     // Gateway IP assigned to AP interface
    "netmask": "255.255.255.0",
    "prefix_len": 24,
    "range_start": "192.168.100.10",
    "range_end": "192.168.100.200",
    "lease_time": "12h",
    "dns_servers": ["8.8.8.8", "1.1.1.1"]
  },
  "monitor": {
    "check_interval_secs": 5,     // How often to health-check daemons
    "max_restart_attempts": 5     // Max restarts before fatal exit
  },
  "log_level": "info"             // trace | debug | info | warn | error
}
```

## Building

```bash
# Debug
cargo build

# Optimized release binary
cargo build --release
```

## Running

> **Requires root** — needs to configure interfaces, iptables, and spawn privileged daemons.

```bash
sudo ./target/release/pi-router --config config.json
```

Or with verbose logging:

```bash
sudo RUST_LOG=debug ./target/release/pi-router
```

## How It Works

```
 Internet
    │
    ▼
 WAN interface (eth0)
    │
    │  iptables MASQUERADE + FORWARD
    │
    ▼
 AP interface (wlan1) ←→ hostapd (WiFi AP, WPA2)
    │
    ▼
 dnsmasq (DHCP: 192.168.100.10–200 / DNS)
    │
    ▼
 Client devices (phones, laptops, etc.)
```

### Startup sequence

| Step | Action |
|------|--------|
| 1 | Read & validate `config.json` |
| 2 | Assign AP IP, bring interface UP |
| 3 | Enable `net.ipv4.ip_forward` |
| 4 | Install iptables NAT/FORWARD rules |
| 5 | Start `hostapd` (WiFi AP) |
| 6 | Start `dnsmasq` (DHCP/DNS) |
| 7 | Launch async health monitor |

### Shutdown sequence (Ctrl-C / SIGTERM)

| Step | Action |
|------|--------|
| 1 | Abort monitor task |
| 2 | Stop `dnsmasq` (SIGTERM → SIGKILL) |
| 3 | Stop `hostapd` (SIGTERM → SIGKILL) |
| 4 | Remove iptables rules |
| 5 | Flush AP interface IP |

## Running Tests

```bash
cargo test
```

## Project Structure

```
src/
├── main.rs              # Entry point, CLI, orchestration, signal handling
├── config.rs            # config.json schema + validation
├── error.rs             # RouterError unified error type
├── monitor.rs           # Async health-check loop with auto-restart
├── network/
│   ├── interface.rs     # ip addr / ip link commands
│   ├── forwarding.rs    # /proc/sys/net/ipv4/ip_forward
│   └── nat.rs           # iptables NAT/MASQUERADE rules
└── daemon/
    ├── process.rs       # Generic supervised child-process manager
    ├── hostapd.rs       # hostapd.conf generator + process manager
    └── dnsmasq.rs       # dnsmasq.conf generator + process manager
```

## Troubleshooting

| Problem | Likely cause | Fix |
|---------|-------------|-----|
| `hostapd` fails to start | Interface doesn't support AP mode | Check with `iw list` → `Supported interface modes` must include `AP` |
| `dnsmasq` port 53 conflict | System dnsmasq/systemd-resolved running | `sudo systemctl stop systemd-resolved` |
| Clients get IP but no internet | `ip_forward` not active | Check `cat /proc/sys/net/ipv4/ip_forward` = 1 |
| `iptables` errors | Missing kernel modules | `sudo modprobe iptable_nat ip_conntrack` |

## License

GPL-3.0
