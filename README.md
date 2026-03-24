# ford-diag — Native macOS Ford Diagnostic Tool

A native macOS CLI for Ford vehicle diagnostics via ELM327 USB adapters. Read and clear diagnostic trouble codes, monitor real-time OBD-II data, access Ford-specific extended diagnostics, and scan Ford modules — all from your terminal.

**No Wine. No Windows. No VM. Runs natively on Apple Silicon.**

## Features

- **Diagnostic Trouble Codes** — Read and clear DTCs (check engine light, ABS, airbag, etc.)
- **Real-time PID monitoring** — RPM, speed, coolant temp, intake temp, throttle position, boost pressure
- **Ford-specific extended diagnostics** — Mode 22 PIDs: knock retard, wastegate duty cycle, octane adjustment ratio, individual tire pressures (TPMS)
- **Ford module scanning** — Discover and query all 22 known modules on 2017 F-150 (PCM, BCM, ABS, APIM, IPC, TCM, and more)
- **VIN reading** — Multi-frame VIN decoding and vehicle identification
- **ELM327 simulator** — Full AT command state machine for development and testing without hardware
- **Raw command mode** — Send arbitrary AT and OBD-II commands directly

## Tested Hardware

| Component | Details |
|-----------|---------|
| **Adapter** | [ELM327 USB with MS-CAN/HS-CAN toggle](https://amzn.to/4takI1e) (CH340T + PIC18F25K80) |
| **Device path** | `/dev/cu.usbserial-*` (macOS built-in CDC driver, no extra drivers needed) |
| **Baud rate** | 38400 (factory default, auto-detected) |
| **Chip** | ELM327 v1.5 (good PIC clone, full AT command set, ATPPS supported) |

## Supported Vehicles

### Full Ford-specific diagnostics (module scanning, extended PIDs, TPMS)

- **Ford** (1996–2024): F-150, F-250/F-350, Explorer, Edge, Escape, Bronco, Mustang, Ranger, Expedition, Transit, Fusion, Focus
- **Lincoln**: Navigator, Aviator, Continental, MKZ, MKC, Corsair, Nautilus
- **Mercury**: All models (1996–2011)
- **Mazda**: All models through 6G generation (1996–2022, shared Ford platform)

### Basic OBD-II (read/clear codes, live PIDs)

Works with **any OBD-II compliant vehicle**:

- 1996+ gasoline vehicles, 2008+ diesel vehicles (US market)
- GM, Toyota, Honda, Stellantis, Volkswagen, BMW, Hyundai/Kia, Subaru, and more
- ~275 million vehicles currently on US roads

## Prerequisites

- **macOS** (Apple Silicon or Intel)
- **Rust toolchain** — install from [rustup.rs](https://rustup.rs)
- **ELM327 USB adapter** — for vehicle communication
- For development without hardware: **no adapter needed** (simulator included)

## Installation

```bash
git clone https://github.com/TheTom/elm327_obd_for_mac.git
cd elm327_obd_for_mac
cargo build --release
```

The binary will be at `target/release/ford-diag`.

## Usage

```bash
# Detect connected adapters
ford-diag detect

# Read diagnostic trouble codes
ford-diag dtc

# Clear trouble codes (requires confirmation)
ford-diag dtc --clear

# Read vehicle info (VIN)
ford-diag info

# Monitor live PIDs
ford-diag live

# Scan Ford modules
ford-diag scan

# Send raw AT/OBD commands
ford-diag raw "ATZ"          # Reset adapter
ford-diag raw "0100"         # Query supported PIDs
ford-diag raw "03"           # Read DTCs (raw)
ford-diag raw "ATRV"         # Read battery voltage

# Specify a device manually
ford-diag --device /dev/cu.usbserial-110 raw "ATZ"

# Verbose logging
ford-diag -v detect
```

## Project Structure

```
crates/
├── elm327-core/          # Core library
│   ├── serial.rs         #   macOS serial port (38400 8N1)
│   ├── detect.rs         #   Device enumeration, baud rate auto-detection
│   ├── elm327.rs         #   ELM327 protocol (init, AT commands, prompt handling)
│   ├── obd.rs            #   OBD-II (PID decoding, DTC parsing, VIN reading)
│   ├── ford.rs           #   Ford module database (CAN address pairs, bus mapping)
│   ├── pty.rs            #   PTY pair creation (used by simulator)
│   ├── bridge.rs         #   Byte forwarding (PTY ↔ serial)
│   ├── config.rs         #   YAML config loading
│   └── error.rs          #   Unified BridgeError type
├── ford-diag/            # CLI binary (clap)
├── elm327-bridge/        # Legacy Wine bridge (deprecated)
└── elm327-simulator/     # Fake ELM327 for testing without hardware
```

### Data Flow

```
ford-diag CLI
    ↓
Diagnostic Engine (Rust)
    ↓
ELM327 Protocol Layer
    ↓
Serial Port (/dev/cu.usbserial-*)
    ↓
ELM327 USB Adapter
    ↓
Vehicle CAN Bus (HS-CAN / MS-CAN)
    ↓
Ford Modules (PCM, BCM, ABS, APIM, IPC, ...)
```

## Development

```bash
make build          # Build all crates
make test           # Run all tests (single-threaded for PTY safety)
make lint           # Clippy with -D warnings
make fmt            # Format all code
make smoke          # Validate environment (Rust toolchain, serial devices)
make detect         # Auto-detect connected adapters
make probe          # Send ATZ to detected device
make list-ports     # List all /dev/cu.* devices
```

## Testing

164 tests covering:

- OBD-II protocol parsing and PID formula calculations
- DTC decoding (P/C/B/U codes) with 100% coverage
- VIN multi-frame decoding
- Ford module database and CAN address pairs
- PTY pair creation and bidirectional data flow
- ELM327 simulator AT command state machine
- End-to-end CLI → simulator integration

All tests run without hardware. The included ELM327 simulator provides a full fake adapter for development. Hardware-dependent tests are skippable:

```bash
SKIP_HARDWARE=1 make test
```

## Verified Test Vehicle

| Detail | Value |
|--------|-------|
| **Vehicle** | 2017 Ford F-150 SuperCrew 4x4 |
| **Engine** | 3.5L V6 EcoBoost (Twin Turbo), 365 HP |
| **Verified** | P0303 (cylinder 3 misfire), freeze frame data, VIN decode, Ford extended PIDs, 5 HS-CAN modules responding |

## Roadmap

- [ ] Bluetooth adapter support (OBDLink MX+, vLinker FS BT)
- [ ] Ford module scanning via TesterPresent
- [ ] Live PID monitoring dashboard
- [ ] VIN auto-decode with vehicle info display
- [ ] MS-CAN support (requires STN-based adapter for transmit)
- [ ] As-Built data reading
- [ ] Multi-OEM module databases (GM, Stellantis, Toyota)
- [ ] JSON/CSV diagnostic export
- [ ] SwiftUI desktop GUI
- [ ] Homebrew formula

## Contributing

Contributions are welcome! This project uses a strict test-driven workflow:

1. **Create a GitHub Issue** describing the work
2. **Write failing tests first**
3. **Implement the minimal solution**
4. **Verify:** `cargo build && cargo test && cargo clippy -- -D warnings`
5. **Submit a PR**

See [CLAUDE.md](CLAUDE.md) for the full development workflow, coding standards, and architecture details.

## License

This project is licensed under the [Apache License 2.0](LICENSE).

## Disclaimer

This tool is for **diagnostic purposes only**. Read operations are safe and non-destructive. The "clear DTC" command (Mode 04) resets the check engine light — codes will return if the underlying problem persists. No write operations to vehicle ECUs or modules are supported. **Use at your own risk.**
