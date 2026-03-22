use std::io::Write;
use std::time::{Duration, Instant};

use clap::{Parser, Subcommand};

use elm327_core::detect::{detect_devices, probe_baud_rate};
use elm327_core::obd::{decode_dtcs, parse_hex_response};
use elm327_core::serial::{SerialConfig, SerialConnection};

#[derive(Parser)]
#[command(name = "ford-diag")]
#[command(about = "Native macOS Ford diagnostic tool")]
struct Cli {
    /// Serial device path (auto-detect if not specified)
    #[arg(short, long)]
    device: Option<String>,

    /// Baud rate (default: 38400)
    #[arg(short, long, default_value = "38400")]
    baud: u32,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Detect connected adapters
    Detect,

    /// Read vehicle info (VIN, calibration IDs)
    Info,

    /// Scan for connected Ford modules
    Scan,

    /// Read diagnostic trouble codes
    Dtc {
        /// Clear DTCs instead of reading
        #[arg(long)]
        clear: bool,
        /// Specific module to read DTCs from (e.g., PCM, BCM)
        #[arg(short, long)]
        module: Option<String>,
    },

    /// Monitor live PIDs
    Live {
        /// PIDs to monitor (e.g., rpm,speed,coolant)
        #[arg(short, long, value_delimiter = ',')]
        pids: Option<Vec<String>>,
    },

    /// Send raw AT or OBD command
    Raw {
        /// Command to send (e.g., "ATI", "0100")
        command: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let log_level = if cli.verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();

    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> elm327_core::error::Result<()> {
    match cli.command {
        Commands::Detect => cmd_detect(),
        Commands::Info => cmd_info(),
        Commands::Scan => cmd_scan(),
        Commands::Dtc { clear, ref module } => cmd_dtc(clear, module.clone(), &cli),
        Commands::Live { pids } => cmd_live(pids),
        Commands::Raw { ref command } => cmd_raw(&cli, command),
    }
}

/// Detect connected ELM327 adapters and probe their baud rates.
fn cmd_detect() -> elm327_core::error::Result<()> {
    println!("Scanning for ELM327 adapters...\n");

    let devices = detect_devices();
    if devices.is_empty() {
        println!("No USB-serial devices found in /dev/cu.*");
        println!("Make sure your ELM327 adapter is plugged in.");
        return Ok(());
    }

    println!("Found {} device(s):\n", devices.len());

    for dev in &devices {
        println!("  {} ({:?})", dev.path.display(), dev.device_type);

        // Try to probe baud rate
        match probe_baud_rate(
            &dev.path.to_string_lossy(),
            Duration::from_secs(2),
        ) {
            Ok(result) => {
                println!("    -> {} @ {} baud", result.version, result.baud_rate);
            }
            Err(e) => {
                println!("    -> probe failed: {}", e);
            }
        }
        println!();
    }

    Ok(())
}

/// Read vehicle info (VIN, calibration IDs).
// TODO: Implement once OBD/UDS service layer is built (Phase 1)
fn cmd_info() -> elm327_core::error::Result<()> {
    println!("Not yet implemented — coming in Phase 1/2");
    Ok(())
}

/// Scan for connected Ford modules.
// TODO: Implement once Ford module scanning is built (Phase 2)
fn cmd_scan() -> elm327_core::error::Result<()> {
    println!("Not yet implemented — coming in Phase 1/2");
    Ok(())
}

/// Read or clear diagnostic trouble codes.
fn cmd_dtc(clear: bool, _module: Option<String>, cli: &Cli) -> elm327_core::error::Result<()> {
    let device_path = resolve_device(cli)?;
    let config = SerialConfig {
        device: device_path.clone(),
        baud_rate: cli.baud,
        timeout: Duration::from_millis(200),
    };
    let mut conn = SerialConnection::open(&config)?;

    // Init adapter for Ford HS-CAN
    send_and_wait(&mut conn, "ATZ", 1500)?;
    send_and_wait(&mut conn, "ATE0", 500)?;
    send_and_wait(&mut conn, "ATH1", 500)?;
    send_and_wait(&mut conn, "ATS0", 500)?;
    send_and_wait(&mut conn, "ATTP6", 500)?;
    send_and_wait(&mut conn, "ATSH7DF", 500)?;

    if clear {
        // --- CLEAR DTCs ---
        // Step 1: Read current DTCs first so user knows what they're clearing
        println!("Reading current DTCs before clearing...\n");
        let dtc_resp = send_and_wait(&mut conn, "03", 3000)?;
        let dtcs = parse_dtc_response(&dtc_resp);

        if dtcs.is_empty() {
            println!("No DTCs found — nothing to clear.");
            return Ok(());
        }

        println!("Current DTCs:");
        for dtc in &dtcs {
            println!("  {} ({})", dtc.code, match dtc.category {
                elm327_core::obd::DtcCategory::Powertrain => "Powertrain",
                elm327_core::obd::DtcCategory::Chassis => "Chassis",
                elm327_core::obd::DtcCategory::Body => "Body",
                elm327_core::obd::DtcCategory::Network => "Network",
            });
        }

        // Step 2: Confirmation prompt
        println!("\n⚠️  WARNING: This will clear ALL DTCs and freeze frame data.");
        println!("   The check engine light will turn off.");
        println!("   DTCs will return if the underlying problem still exists.");
        print!("\n   Type 'CLEAR' to confirm: ");
        std::io::stdout().flush().ok();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();

        if input.trim() != "CLEAR" {
            println!("Cancelled. No DTCs were cleared.");
            return Ok(());
        }

        // Step 3: Send Mode 04 (Clear DTCs)
        println!("\nClearing DTCs...");
        let clear_resp = send_and_wait(&mut conn, "04", 3000)?;

        if clear_resp.contains("44") {
            println!("✓ DTCs cleared successfully.");
        } else {
            println!("Clear response: {}", clear_resp);
            println!("⚠️  Unexpected response — verify DTCs were cleared.");
        }

        // Step 4: Verify by re-reading
        println!("\nVerifying...");
        let verify_resp = send_and_wait(&mut conn, "03", 3000)?;
        let remaining = parse_dtc_response(&verify_resp);

        if remaining.is_empty() {
            println!("✓ Confirmed: no DTCs remaining.");
        } else {
            println!("⚠️  {} DTC(s) still present (may be current/active faults):", remaining.len());
            for dtc in &remaining {
                println!("  {}", dtc.code);
            }
        }
    } else {
        // --- READ DTCs ---
        println!("Reading diagnostic trouble codes...\n");
        let dtc_resp = send_and_wait(&mut conn, "03", 3000)?;
        let dtcs = parse_dtc_response(&dtc_resp);

        if dtcs.is_empty() {
            println!("No DTCs found. ✓");
        } else {
            println!("Found {} DTC(s):\n", dtcs.len());
            for dtc in &dtcs {
                let category = match dtc.category {
                    elm327_core::obd::DtcCategory::Powertrain => "Powertrain",
                    elm327_core::obd::DtcCategory::Chassis => "Chassis",
                    elm327_core::obd::DtcCategory::Body => "Body",
                    elm327_core::obd::DtcCategory::Network => "Network",
                };
                println!("  {} — {}", dtc.code, category);
            }
            println!("\nTo clear these DTCs, run: ford-diag dtc --clear");
        }

        // Also read pending DTCs (Mode 07)
        let pending_resp = send_and_wait(&mut conn, "07", 3000)?;
        let pending = parse_dtc_response(&pending_resp);
        if !pending.is_empty() {
            println!("\nPending DTCs ({}):", pending.len());
            for dtc in &pending {
                println!("  {} (pending)", dtc.code);
            }
        }
    }

    Ok(())
}

/// Parse DTC response from Mode 03/07 hex output
fn parse_dtc_response(resp: &str) -> Vec<elm327_core::obd::Dtc> {
    let mut all_dtcs = Vec::new();

    for line in resp.split('\r') {
        let line = line.trim();
        if line.is_empty() || line == ">" || line.contains("NO DATA") {
            continue;
        }

        let bytes = parse_hex_response(line);

        // Mode 03 response starts with 0x43, Mode 07 with 0x47
        if bytes.is_empty() || (bytes[0] != 0x43 && bytes[0] != 0x47) {
            continue;
        }

        // Skip response byte and count, decode DTC pairs
        let dtc_data = if bytes.len() > 2 { &bytes[2..] } else { continue };
        let dtcs = decode_dtcs(dtc_data);
        all_dtcs.extend(dtcs);
    }

    all_dtcs
}

/// Send an AT/OBD command and wait for the '>' prompt response.
fn send_and_wait(conn: &mut SerialConnection, cmd: &str, timeout_ms: u64) -> elm327_core::error::Result<String> {
    let cmd_bytes = format!("{}\r", cmd);
    conn.write_all(cmd_bytes.as_bytes())?;

    let mut response = Vec::with_capacity(1024);
    let mut buf = [0u8; 256];
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);

    while Instant::now() < deadline {
        match conn.read(&mut buf)? {
            0 => {
                if !response.is_empty() {
                    let text = String::from_utf8_lossy(&response);
                    if text.contains('>') {
                        break;
                    }
                }
            }
            n => {
                response.extend_from_slice(&buf[..n]);
                let text = String::from_utf8_lossy(&response);
                if text.contains('>') {
                    break;
                }
            }
        }
    }

    Ok(String::from_utf8_lossy(&response).to_string())
}

/// Monitor live PIDs.
// TODO: Implement once PID monitoring is built (Phase 2)
fn cmd_live(_pids: Option<Vec<String>>) -> elm327_core::error::Result<()> {
    println!("Not yet implemented — coming in Phase 1/2");
    Ok(())
}

/// Send a raw AT or OBD command to the adapter.
///
/// Auto-detects the device if --device is not specified, opens a serial
/// connection, sends the command, and prints the response.
fn cmd_raw(cli: &Cli, command: &str) -> elm327_core::error::Result<()> {
    let device_path = resolve_device(cli)?;

    log::info!("Using device: {} @ {} baud", device_path, cli.baud);

    let config = SerialConfig {
        device: device_path.clone(),
        baud_rate: cli.baud,
        timeout: Duration::from_millis(200),
    };

    let mut conn = SerialConnection::open(&config)?;

    // Send command with carriage return terminator
    let cmd = format!("{}\r", command);
    conn.write_all(cmd.as_bytes())?;

    // Read response until we see the '>' prompt or timeout
    let mut response = Vec::with_capacity(1024);
    let mut buf = [0u8; 256];
    let deadline = Instant::now() + Duration::from_secs(5);

    while Instant::now() < deadline {
        match conn.read(&mut buf)? {
            0 => {
                // Timeout on this read — check if we got a complete response
                if !response.is_empty() {
                    let text = String::from_utf8_lossy(&response);
                    if text.contains('>') {
                        break;
                    }
                }
            }
            n => {
                response.extend_from_slice(&buf[..n]);
                let text = String::from_utf8_lossy(&response);
                if text.contains('>') {
                    break;
                }
            }
        }
    }

    let text = String::from_utf8_lossy(&response);

    // ELM327 uses CR (\r) as line terminator, not LF
    if !text.contains('>') {
        eprintln!("Warning: response incomplete (no '>' prompt received)");
        // Still print what we got
        if !text.is_empty() {
            eprintln!("Partial response: {}", text.trim());
        }
        return Err(elm327_core::error::BridgeError::Timeout(Duration::from_secs(5)));
    }

    // Clean up and print the response
    for line in text.split('\r') {
        let line = line.trim();
        if line.is_empty() || line == ">" {
            continue;
        }
        println!("{}", line);
    }

    Ok(())
}

/// Resolve the device path: use --device if given, otherwise auto-detect.
fn resolve_device(cli: &Cli) -> elm327_core::error::Result<String> {
    if let Some(ref dev) = cli.device {
        return Ok(dev.clone());
    }

    log::info!("No --device specified, auto-detecting...");
    let devices = detect_devices();
    if devices.is_empty() {
        return Err(elm327_core::error::BridgeError::DeviceNotFound(
            "No USB-serial devices found. Use --device to specify manually.".to_string(),
        ));
    }

    let path = devices[0].path.to_string_lossy().to_string();
    log::info!("Auto-detected device: {}", path);
    Ok(path)
}
