//! pictl — command-line control tool for pi-router
//!
//! Usage:
//!   pictl list
//!   pictl approve <MAC>
//!   pictl deny    <MAC>
//!   pictl status

use clap::{Parser, Subcommand};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

const SOCKET: &str = "/tmp/pi-router.sock";

#[derive(Parser)]
#[command(
    name        = "pictl",
    version,
    about       = "Control tool for the pi-router daemon",
    long_about  = "Manage device approval on the pi-router WiFi AP.\n\
                   Communicates with the running pi-router daemon via a Unix socket."
)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// List all connected / known devices and their approval state
    List,
    /// Approve a device (grant internet access)
    Approve {
        /// MAC address (e.g. aa:bb:cc:dd:ee:ff)
        mac: String,
    },
    /// Deny / revoke a device's internet access
    Deny {
        /// MAC address (e.g. aa:bb:cc:dd:ee:ff)
        mac: String,
    },
    /// Show daemon status
    Status,
}

fn main() {
    let cli = Cli::parse();

    let payload = match &cli.command {
        Cmd::List              => r#"{"cmd":"list"}"#.to_string(),
        Cmd::Approve { mac }   => format!(r#"{{"cmd":"approve","mac":"{mac}"}}"#),
        Cmd::Deny    { mac }   => format!(r#"{{"cmd":"deny","mac":"{mac}"}}"#),
        Cmd::Status            => r#"{"cmd":"status"}"#.to_string(),
    };

    let mut stream = UnixStream::connect(SOCKET).unwrap_or_else(|e| {
        eprintln!("error: cannot connect to pi-router daemon ({SOCKET}): {e}");
        eprintln!("       Is pi-router running with root privileges?");
        std::process::exit(1);
    });

    // Send command
    writeln!(stream, "{payload}").unwrap_or_else(|e| {
        eprintln!("error: write failed: {e}");
        std::process::exit(1);
    });

    // Read single-line JSON response
    let mut reader = BufReader::new(&stream);
    let mut response = String::new();
    reader.read_line(&mut response).unwrap_or_else(|e| {
        eprintln!("error: read failed: {e}");
        std::process::exit(1);
    });

    let v: serde_json::Value = serde_json::from_str(response.trim()).unwrap_or_else(|e| {
        eprintln!("error: bad response ({e}): {response}");
        std::process::exit(1);
    });

    // Pretty-print depending on command
    match cli.command {
        Cmd::List => print_device_table(&v),
        _         => print_simple(&v),
    }
}

// ─── Formatters ───────────────────────────────────────────────────────────────

fn print_device_table(v: &serde_json::Value) {
    if !v["ok"].as_bool().unwrap_or(false) {
        eprintln!("error: {}", v["message"].as_str().unwrap_or("unknown"));
        std::process::exit(1);
    }

    let devices = match v["devices"].as_array() {
        Some(d) => d,
        None    => { println!("(no devices)"); return; }
    };

    if devices.is_empty() {
        println!("No devices seen yet.");
        return;
    }

    println!(
        "{:<20}  {:<18}  {:<20}  {:<10}",
        "MAC", "IP", "HOSTNAME", "STATE"
    );
    println!("{}", "─".repeat(74));

    for d in devices {
        let state  = d["state"].as_str().unwrap_or("?");
        let symbol = match state {
            "approved" => "✓",
            "denied"   => "✗",
            _          => "⏳",
        };
        println!(
            "{:<20}  {:<18}  {:<20}  {} {}",
            d["mac"].as_str().unwrap_or("-"),
            d["ip"].as_str().unwrap_or("-"),
            d["hostname"].as_str().unwrap_or("-"),
            symbol,
            state,
        );
    }
}

fn print_simple(v: &serde_json::Value) {
    if v["ok"].as_bool().unwrap_or(false) {
        let msg = v["message"].as_str()
            .or_else(|| v["version"].as_str())
            .unwrap_or("ok");
        println!("✓ {msg}");
    } else {
        eprintln!("✗ {}", v["message"].as_str().unwrap_or("unknown error"));
        std::process::exit(1);
    }
}
