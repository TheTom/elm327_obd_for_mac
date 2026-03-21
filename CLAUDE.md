# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

FORScan macOS Compatibility Layer — a user-space serial bridge that enables the Windows-only FORScan OBD diagnostic tool to communicate with USB OBD adapters (ELM327) on macOS Apple Silicon via Wine/CrossOver. See `PRD.md` for full product spec.

## Development Commands

### **Quick Start**
```bash
make smoke          # Validate environment (Wine, PTY, serial device)
make bridge         # Start the serial bridge service
make clean          # Kill bridge, remove PTY artifacts
```

### **Testing**
```bash
make test           # Run all tests
make test-unit      # Unit tests only (no hardware required)
make test-pty       # PTY creation + bidirectional data flow
make test-serial    # Serial device detection + communication (requires adapter)
make test-bridge    # Full bridge integration (PTY ↔ serial forwarding)
make test-wine      # Wine COM port mapping verification
make test-e2e       # End-to-end: FORScan → COM → PTY → bridge → adapter
```

### **Development**
```bash
make build          # Build bridge binary
make rebuild        # Clean + build
make lint           # Run linter
make fmt            # Format code
make dev            # Build + start bridge with debug logging
```

### **Device Utilities**
```bash
make detect         # Auto-detect OBD adapters on /dev/cu.*
make probe          # Send ATZ to detected device, print response
make list-ports     # List all /dev/cu.* devices
```

## Architecture Notes

### **Data Flow**
```
FORScan (Wine) → COM3 (dosdevices symlink) → PTY-A ↔ PTY-B → Bridge Service → /dev/cu.* → OBD Adapter
```

### **Core Components**
- `src/bridge/` - Main bridge service: PTY ↔ serial byte forwarding
- `src/pty/` - PTY pair creation and management
- `src/serial/` - macOS serial device open/read/write, baud rate, flow control
- `src/detect/` - Device enumeration and filtering (`wchusbserial`, `usbserial`, `SLAB_USBtoUART`)
- `src/config/` - YAML config loading and validation
- `src/wine/` - Wine dosdevices COM port symlink management

### **Configuration**
All settings via `config.yml`:
- **Device**: `device`, `baud_rate` (default: 115200)
- **Wine**: `wine_com_port` (default: COM3)
- **Behavior**: `auto_reconnect`, `logging`, `log_level`

## Testing Protocol

### **Test Hierarchy (strict)**
Every PR must pass tests in this order. A failure at any level blocks the next.

1. **Unit tests** — no I/O, no hardware, no Wine. Pure logic.
2. **PTY tests** — create PTY pair, write bytes A→B and B→A, verify integrity.
3. **Serial tests** — open real `/dev/cu.*`, send `ATZ`, expect `ELM327` response. **Skip if no adapter connected** (CI-safe).
4. **Bridge tests** — full PTY ↔ serial forwarding. Write to PTY-A, verify it arrives on serial. Write from serial, verify it arrives on PTY-A.
5. **Wine tests** — COM symlink exists, Wine can open the mapped port. **Skip if Wine not installed.**
6. **E2E tests** — FORScan detects COM port, sends AT command, gets response. **Manual gate.**

### **Test Rules**
- Tests that require hardware MUST be skippable via env flag (`SKIP_HARDWARE=1`)
- Tests that require Wine MUST be skippable via env flag (`SKIP_WINE=1`)
- All tests MUST have timeouts (max 10s for unit, 30s for integration)
- Serial tests MUST clean up device handles on exit
- PTY tests MUST clean up file descriptors on exit
- No test may leave orphan processes

### **What "passes" means**
- `ATZ` → response contains `ELM327` (or adapter identifier)
- `ATI` → response is non-empty
- PTY round-trip latency < 5ms for 64-byte payload
- Bridge forwarding: zero data loss over 1000 round-trips
- COM symlink resolves to valid PTY device

## Development Philosophy

### **Phase-gated work**
This project follows strict phases (see PRD.md §8). Do NOT skip ahead:
1. **Phase 1 (POC)**: PTY pair + byte bridge + Wine COM mapping. No config, no reconnect, no polish.
2. **Phase 2 (Integration)**: FORScan actually talks through the bridge. Debug handshake/timing.
3. **Phase 3 (Stability)**: Reconnect, logging, config system.
4. **Phase 4 (Packaging)**: CLI tool, Homebrew formula.

### **Code Rules**
- **Language**: Rust preferred. Python acceptable for prototyping only.
- **No kernel extensions**: Everything runs in user space.
- **No unsafe unless justified**: If you write `unsafe`, add a comment explaining why.
- **Logging**: All I/O operations MUST be loggable at debug level.
- **Error handling**: Never silently swallow serial errors. Log + propagate.
- **Baud rate**: Always configurable, never hardcoded.
- **Timeouts**: Every serial read/write MUST have a timeout. No blocking forever.

### **Hardware Safety**
- **Never send raw bytes to the adapter without logging them first.**
- Do not attempt ECU writes or flash operations.
- Default to read-only diagnostic commands (AT*, 01xx PIDs).
- If the adapter stops responding, back off — do not spam retries.

## Performance Notes

- **PTY forwarding**: Target < 1ms overhead per byte-forward operation
- **Serial I/O**: Non-blocking, buffered. Use `poll`/`select`/`epoll` (or `kqueue` on macOS)
- **Bridge throughput**: Must sustain 38400 baud (factory default) without data loss
- **Reconnect**: Detect device disconnect within 2s, attempt reconnect within 5s

## Known Device Patterns

```
/dev/cu.usbserial-*      # macOS built-in CDC driver (confirmed on Apple Silicon)
/dev/cu.wchusbserial*    # CH340-based with WCH driver installed
/dev/cu.SLAB_USBtoUART*  # Silicon Labs CP210x adapters
```

### Verified Hardware (2026-03-21)
- **Device**: ELM327 USB with MS-CAN/HS-CAN toggle (CH340T, PIC18F25K80)
- **macOS path**: `/dev/cu.usbserial-110` (built-in CDC driver, no WCH driver needed)
- **Baud rate**: 38400 (factory default, confirmed)
- **Version**: ELM327 v1.5 (good PIC clone, full AT command set)
- **ATPPS**: Full table returned (not a bad ARM clone)
- **PP 0C**: 0x68 (non-default baud divisor stored but running at 38400)

## Common ELM327 AT Commands (Reference)

```
ATZ     → Reset, returns adapter ID (e.g., "ELM327 v1.5")
ATI     → Adapter version info
ATE0    → Echo off
ATL0    → Linefeeds off
ATS0    → Spaces off
ATSP0   → Auto-detect OBD protocol
0100    → Supported PIDs [01-20]
010C    → Engine RPM
010D    → Vehicle speed
```
