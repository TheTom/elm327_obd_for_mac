use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;

use elm327_core::bridge::Bridge;
use elm327_core::config::Config;
use elm327_core::detect::detect_devices;
use elm327_core::error::BridgeError;
use elm327_core::pty::PtyPair;
use elm327_core::serial::{SerialConfig, SerialConnection};
use elm327_core::wine;

/// ELM327 USB-to-Wine bridge — forwards serial traffic through a PTY
/// so FORScan (running under Wine) can talk to a real OBD adapter.
#[derive(Parser, Debug)]
#[command(name = "elm327-bridge", version, about)]
struct Cli {
    /// Path to config file
    #[arg(long, default_value = "config.yml")]
    config: PathBuf,

    /// Override device path (e.g., /dev/cu.wchusbserial14340)
    #[arg(long)]
    device: Option<String>,

    /// Override baud rate
    #[arg(long)]
    baud: Option<u32>,

    /// Override Wine COM port (e.g., COM3)
    #[arg(long)]
    com_port: Option<String>,

    /// Override Wine prefix path
    #[arg(long)]
    wine_prefix: Option<String>,

    /// List detected OBD devices and exit
    #[arg(long)]
    detect: bool,

    /// Detect devices, probe each with ATZ, and exit
    #[arg(long)]
    probe: bool,

    /// Enable debug logging
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();

    // --- Init logging ---
    init_logging(cli.verbose);

    // --- Detect mode ---
    if cli.detect {
        run_detect();
        return;
    }

    // --- Probe mode ---
    if cli.probe {
        run_probe();
        return;
    }

    // --- Bridge mode ---
    if let Err(e) = run_bridge(cli) {
        log::error!("{}", e);
        eprintln!("Fatal: {e}");
        std::process::exit(1);
    }
}

/// Set up env_logger. --verbose sets RUST_LOG=debug if not already set.
fn init_logging(verbose: bool) {
    if verbose && std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "debug");
    }
    env_logger::init();
}

/// Print all detected OBD-capable serial devices.
fn run_detect() {
    let devices = detect_devices();
    if devices.is_empty() {
        println!("No OBD devices detected.");
    } else {
        println!("Detected devices:");
        for dev in &devices {
            println!("  {dev}");
        }
    }
}

/// Detect devices, open each one, send ATZ, print the response.
fn run_probe() {
    let devices = detect_devices();
    if devices.is_empty() {
        println!("No OBD devices detected.");
        return;
    }

    for dev in &devices {
        println!("Probing {} ...", dev);
        let cfg = SerialConfig {
            device: dev.path.to_string_lossy().to_string(),
            baud_rate: 38400,
            timeout: Duration::from_secs(2),
        };

        match SerialConnection::open(&cfg) {
            Ok(mut conn) => {
                // Send ATZ (reset) command
                if let Err(e) = conn.write(b"ATZ\r") {
                    println!("  Write error: {e}");
                    continue;
                }
                if let Err(e) = conn.flush() {
                    println!("  Flush error: {e}");
                    continue;
                }

                // Wait a bit for the device to respond
                std::thread::sleep(Duration::from_millis(500));

                let mut buf = [0u8; 256];
                match conn.read(&mut buf) {
                    Ok(0) => println!("  No response (timeout)"),
                    Ok(n) => {
                        let response = String::from_utf8_lossy(&buf[..n]);
                        println!("  Response: {}", response.trim());
                    }
                    Err(e) => println!("  Read error: {e}"),
                }
            }
            Err(e) => println!("  Failed to open: {e}"),
        }
    }
}

/// Main bridge flow: load config, open serial, create PTY, symlink, run loop.
fn run_bridge(cli: Cli) -> Result<(), BridgeError> {
    // --- Load config ---
    let mut config = if cli.config.exists() {
        log::info!("Loading config from {}", cli.config.display());
        Config::load(&cli.config)?
    } else {
        log::info!(
            "Config file {} not found, using defaults",
            cli.config.display()
        );
        Config::default()
    };

    // --- Apply CLI overrides ---
    if let Some(device) = cli.device {
        config.device = device;
    }
    if let Some(baud) = cli.baud {
        config.baud_rate = baud;
    }
    if let Some(com_port) = cli.com_port {
        config.wine_com_port = com_port;
    }
    if let Some(prefix) = cli.wine_prefix {
        config.wine_prefix = Some(prefix);
    }

    // --- Auto-detect device if needed ---
    if config.device == "auto" {
        log::info!("Auto-detecting OBD device...");
        let devices = detect_devices();
        if devices.is_empty() {
            return Err(BridgeError::DeviceNotFound(
                "No OBD devices detected. Plug in your ELM327 adapter and try again.".to_string(),
            ));
        }
        config.device = devices[0].path.to_string_lossy().to_string();
        log::info!("Auto-detected: {}", config.device);
    }

    // --- Open serial connection ---
    let serial_cfg = SerialConfig {
        device: config.device.clone(),
        baud_rate: config.baud_rate,
        timeout: Duration::from_secs(1),
    };
    let serial = SerialConnection::open(&serial_cfg)?;
    let serial_fd = serial.as_raw_fd();

    // --- Create PTY pair ---
    let pty = PtyPair::create()?;
    let pty_fd = {
        use std::os::fd::AsRawFd;
        pty.controller.as_raw_fd()
    };

    // --- Create Wine COM symlink ---
    let wine_prefix = config.wine_prefix_path();
    let com_symlink = if wine_prefix.join("dosdevices").exists() {
        match wine::create_com_symlink(&wine_prefix, &config.wine_com_port, pty.device_path()) {
            Ok(link) => {
                log::info!("COM symlink created: {}", link.display());
                Some(link)
            }
            Err(e) => {
                log::warn!("Failed to create COM symlink: {e}");
                None
            }
        }
    } else {
        log::warn!(
            "Wine prefix not found at {} — skipping COM symlink",
            wine_prefix.display()
        );
        None
    };

    // --- Print status ---
    println!(
        "Bridge active: {} <-> {} (PTY: {})",
        config.device,
        config.wine_com_port,
        pty.device_path().display()
    );
    println!("Press Ctrl+C to stop.");

    // --- Install Ctrl+C handler ---
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_hook = shutdown.clone();
    ctrlc::set_handler(move || {
        println!("\nShutting down...");
        shutdown_hook.store(true, Ordering::SeqCst);
    })
    .expect("Failed to install Ctrl+C handler");

    // --- Run bridge loop ---
    // NOTE: `serial` and `pty` must stay alive — their FDs are borrowed by Bridge.
    let mut bridge = Bridge::new(pty_fd, serial_fd);
    let result = bridge.run(&shutdown);

    // --- Shutdown: print stats ---
    let stats = bridge.stats();
    println!(
        "\nBridge stats: {} bytes PTY->Serial, {} bytes Serial->PTY, {} forwards, {} errors",
        stats.bytes_pty_to_serial,
        stats.bytes_serial_to_pty,
        stats.forward_count,
        stats.errors
    );

    // --- Cleanup: remove COM symlink ---
    if com_symlink.is_some() {
        if let Err(e) = wine::remove_com_symlink(&wine_prefix, &config.wine_com_port) {
            log::warn!("Failed to remove COM symlink: {e}");
        } else {
            log::info!("COM symlink removed");
        }
    }

    // Keep serial and pty alive until here
    drop(serial);
    drop(pty);

    result
}
