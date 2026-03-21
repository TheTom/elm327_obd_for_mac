# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Native macOS Ford Diagnostic Tool — a Rust CLI that communicates directly with ELM327 USB adapters for Ford-specific OBD-II and UDS diagnostics on Apple Silicon. No Wine, no Windows. See `PRD.md` for full product spec.

## Development Commands

### **Quick Start**
```bash
make smoke          # Validate environment (Rust, serial device)
make build          # Build all crates
make clean          # Cargo clean
```

### **Testing**
```bash
make test           # Run all tests (single-threaded for PTY safety)
make test-unit      # Unit tests only (no hardware required)
make test-pty       # PTY creation + bidirectional data flow
make test-serial    # Serial device communication (requires adapter)
make test-bridge    # Bridge integration (PTY ↔ serial forwarding)
make test-e2e       # End-to-end: CLI → ELM327 → simulator
make lint           # Clippy with -D warnings
make fmt            # Format all code
```

### **CLI Tool**
```bash
cargo run --bin ford-diag -- detect          # Find OBD adapters
cargo run --bin ford-diag -- raw "ATZ"       # Send raw command
cargo run --bin ford-diag -- info            # Read VIN (Phase 1)
cargo run --bin ford-diag -- scan            # Scan Ford modules (Phase 2)
cargo run --bin ford-diag -- dtc             # Read DTCs
cargo run --bin ford-diag -- dtc --clear     # Clear DTCs
cargo run --bin ford-diag -- live            # Monitor live PIDs
```

### **Device Utilities**
```bash
make detect         # Auto-detect OBD adapters on /dev/cu.*
make probe          # Send ATZ to detected device, print response
make list-ports     # List all /dev/cu.* devices
```

## Architecture

### **Data Flow**
```
ford-diag CLI → Diagnostic Engine → ELM327 Protocol → Serial → /dev/cu.* → Adapter → CAN Bus → Ford Modules
```

### **Crate Structure**
- `crates/elm327-core/` — Core library
  - `serial.rs` — macOS serial port (38400 8N1, TTYPort with AsRawFd)
  - `detect.rs` — Device enumeration, baud rate auto-detection
  - `elm327.rs` — ELM327 protocol (init, send/receive, prompt handling)
  - `obd.rs` — OBD-II (PID decoding, DTC parsing, VIN reading)
  - `ford.rs` — Ford module database (CAN address pairs, bus mapping)
  - `pty.rs` — PTY pair creation (used by simulator)
  - `bridge.rs` — Byte forwarding (used by simulator tests)
  - `config.rs` — YAML config loading
  - `error.rs` — Unified BridgeError type
  - `wine.rs` — Wine COM symlink management (legacy, may remove)
- `crates/ford-diag/` — CLI binary (clap subcommands)
- `crates/elm327-bridge/` — Bridge CLI (legacy Wine approach)
- `crates/elm327-simulator/` — Fake ELM327 for testing without hardware

### **Configuration**
All settings via `config.yml`:
- **Device**: `device` (default: auto-detect), `baud_rate` (default: 38400)
- **Behavior**: `logging`, `log_level`

## Testing Protocol

### **Test Hierarchy (strict)**
Every PR must pass tests in this order. A failure at any level blocks the next.

1. **Unit tests** — no I/O, no hardware. Pure logic (PID decoding, DTC parsing, config).
2. **PTY tests** — PTY pair creation + bidirectional data flow.
3. **Simulator tests** — full ELM327 command/response through simulator.
4. **Bridge tests** — PTY ↔ serial forwarding via bridge.
5. **Integration tests** — end-to-end: CLI → bridge → simulator pipeline.
6. **Hardware tests** — real adapter communication. **Skip if no adapter** (`SKIP_HARDWARE=1`).

### **Test Rules**
- Tests that require hardware MUST be skippable via `SKIP_HARDWARE=1`
- All tests MUST have timeouts (max 10s for unit, 30s for integration)
- Serial tests MUST clean up device handles on exit
- PTY tests MUST clean up file descriptors on exit
- No test may leave orphan processes
- Use `--test-threads=1` to prevent PTY fd exhaustion (ERANGE)

### **Review Process (mandatory)**
- Every module gets: build → test → clippy → **codex review** → commit → push
- Codex findings are evaluated by Claude — accept real bugs, reject style nits
- All accepted findings must be fixed before committing
- GitHub issues track work by phase

### **What "passes" means**
- `ATZ` → response contains `ELM327`
- PID decode: 0x0BB8 → 748 RPM (formula: (A*256+B)/4)
- DTC decode: 0x0300 → "P0300"
- VIN decode: multi-frame hex → 17-char ASCII string
- PTY round-trip latency < 5ms for 64-byte payload
- Bridge forwarding: zero data loss over 1000 round-trips

## Development Philosophy

### **Phase-gated work**
This project follows strict phases (see PRD.md §7). Do NOT skip ahead:
1. **Phase 1 (Talk to the Truck)**: ELM327 protocol + OBD-II basics (VIN, PIDs, DTCs)
2. **Phase 2 (Ford Modules)**: Module scanning, per-module DTCs, firmware versions
3. **Phase 3 (Deep Diagnostics)**: As-Built reading, MS-CAN, full PID database
4. **Phase 4 (Config & GUI)**: As-Built writing, SwiftUI app, Homebrew

### **Code Rules**
- **Language**: Rust. No Python in production code.
- **No kernel extensions**: Everything runs in user space.
- **No unsafe unless justified**: If you write `unsafe`, add a comment explaining why.
- **Logging**: All I/O operations MUST be loggable at debug level.
- **Error handling**: Never silently swallow serial errors. Log + propagate.
- **Baud rate**: Always configurable, never hardcoded (default 38400).
- **Timeouts**: Every serial read/write MUST have a timeout. No blocking forever.

### **Hardware Safety**
- **Never send raw bytes to the adapter without logging them first.**
- Do not attempt ECU writes or flash operations without explicit user confirmation.
- Default to read-only diagnostic commands (AT*, Mode 01/03/09).
- If the adapter stops responding, back off — do not spam retries.
- **NEVER toggle MS/HS-CAN switch while actively communicating.**

## Target Vehicle

**2017 Ford F-150 EcoBoost 3.5L V6 Twin Turbo**
- 22 modules (20 HS-CAN, 2 MS-CAN)
- PCM: HL3A-12A650-BBB / HL3A-12B565-GB
- FORScan profile with full module/firmware mapping in `data/`

## Verified Hardware (2026-03-21)

```
/dev/cu.usbserial-110    # macOS built-in CDC driver (Apple Silicon, no WCH driver needed)
```

- **Adapter**: ELM327 USB with MS-CAN/HS-CAN toggle (CH340T, PIC18F25K80)
- **Baud rate**: 38400 (factory default, confirmed)
- **Version**: ELM327 v1.5 (good PIC clone, full AT command set)
- **ATPPS**: Full table returned (not a bad ARM clone)

## Common ELM327 AT Commands (Reference)

```
ATZ     → Reset, returns adapter ID (e.g., "ELM327 v1.5")
ATI     → Adapter version info
ATE0    → Echo off
ATL0    → Linefeeds off
ATH1    → Headers on (show CAN IDs in responses)
ATS0    → Spaces off
ATAT1   → Adaptive timing on
ATSP6   → Set protocol: CAN 11-bit 500kbps (Ford HS-CAN)
ATSPB   → Set protocol: User CAN (for MS-CAN at 125kbps)
ATSH7E0 → Set header to PCM request address
ATCRA7E8 → Filter responses to PCM only
0100    → Supported PIDs [01-20]
010C    → Engine RPM
010D    → Vehicle speed
03      → Read DTCs
04      → Clear DTCs
0902    → Read VIN
```
