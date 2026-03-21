# PRD: Native macOS Ford Diagnostic Tool (Apple Silicon)

## 1. Overview

**Goal:**
Build a native macOS diagnostic tool for Ford vehicles that communicates directly with ELM327 USB adapters — no Wine, no Windows, no bullshit.

**Core Idea:**
A Rust CLI (with future SwiftUI GUI) that talks to Ford vehicle modules over CAN bus via ELM327, supporting OBD-II diagnostics, Ford-specific module scanning, DTC management, and eventually As-Built configuration.

---

## 2. Problem Statement

FORScan is the gold standard for Ford diagnostics but:
- Windows only (Wine/CrossOver is broken on Apple Silicon)
- Closed source, no API
- Requires license for extended features

macOS users currently have no native option for Ford-specific diagnostics beyond basic OBD-II readers.

---

## 3. Objectives

### Primary
- Direct serial communication with ELM327 USB adapters on macOS
- Standard OBD-II: read/clear DTCs, live PID monitoring
- Ford module discovery (scan all known module addresses)
- Ford-specific DTC reading per module

### Secondary
- As-Built data reading (Ford configuration blocks)
- MS-CAN support (body control modules via Protocol B)
- Live data dashboard (RPM, speed, temps, pressures)
- Export diagnostics to JSON/CSV

### Phase 3+
- As-Built data writing (configuration changes)
- "Hidden features" activation (Bambi mode, global windows, etc.)
- SwiftUI GUI
- Homebrew installable

### Non-Goals
- ECU reflashing/reprogramming
- Safety-critical writes without explicit confirmation
- Support for non-Ford vehicles (initially)

---

## 4. System Architecture

```
CLI / SwiftUI GUI
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

---

## 5. Components

### 5.1 Serial Layer (existing)
- `serial.rs` — open/read/write with timeouts, 38400 8N1
- `detect.rs` — auto-detect adapters, baud rate probing

### 5.2 ELM327 Protocol Layer (new)
- Send AT commands, parse responses
- Wait for `>` prompt between commands
- Handle error responses (NO DATA, UNABLE TO CONNECT, etc.)
- Manage adapter state (echo, headers, protocol, timing)

### 5.3 OBD-II Layer (new)
- Standard services: Mode 01 (live data), Mode 03 (DTCs), Mode 04 (clear), Mode 09 (VIN)
- PID decoding (RPM, speed, temps, fuel, etc.)
- DTC decoding (P/C/B/U codes)

### 5.4 UDS Layer (new)
- ISO 14229 Unified Diagnostic Services over CAN
- DiagnosticSessionControl (0x10)
- ReadDTCInformation (0x19)
- ReadDataByIdentifier (0x22)
- TesterPresent (0x3E)
- ClearDTCs (0x14)

### 5.5 Ford Module Layer (new)
- Known module address database (CAN request/response ID pairs)
- Module scanning (send TesterPresent, collect responses)
- Per-module DTC reading
- As-Built data reading (Ford-specific DIDs)

### 5.6 CLI Interface (new)
- `ford-diag scan` — discover connected modules
- `ford-diag dtc` — read all DTCs across all modules
- `ford-diag dtc clear` — clear DTCs
- `ford-diag live` — real-time PID monitoring
- `ford-diag info` — vehicle info (VIN, calibration IDs)
- `ford-diag asbuilt read <module>` — dump As-Built data
- `ford-diag raw <cmd>` — send raw AT/OBD command

---

## 6. Target Vehicle

**2017 Ford F-150 EcoBoost 3.5L V6 Twin Turbo**
- 22 modules on HS-CAN and MS-CAN
- PCM ID: 0x2DF7 (FORScan internal), CAN 0x7E0/0x7E8
- ELM327 v1.5 (PIC18F25K80) at 38400 baud
- Device: /dev/cu.usbserial-110

---

## 7. Implementation Phases

### Phase 1: Talk to the Truck
- ELM327 protocol layer (init, AT commands, prompt handling)
- Connect to adapter, configure for Ford HS-CAN
- Read VIN (Mode 09)
- Read basic PIDs (RPM, speed, coolant temp)
- Read OBD-II DTCs (Mode 03)
- Clear DTCs (Mode 04)

### Phase 2: Ford Module Scanning
- Module address database
- Scan all known Ford module addresses via TesterPresent
- Read DTCs per module (UDS 0x19)
- Report module firmware versions

### Phase 3: Deep Diagnostics
- As-Built data reading
- MS-CAN support (Protocol B, 125 kbps)
- Full PID database with units/scaling
- Export to JSON/CSV

### Phase 4: Configuration & GUI
- As-Built data writing (with safety confirmations)
- SwiftUI desktop app
- Homebrew formula

---

## 8. Success Criteria

Phase 1:
- Read VIN from truck
- Display live RPM and coolant temp
- Read and display DTCs
- Clear DTCs

Phase 2:
- Discover all 22 modules from the FORScan profile
- Read DTCs from individual modules (not just OBD-II broadcast)

---

## 9. Hardware

- **Adapter**: ELM327 USB with MS-CAN/HS-CAN toggle (CH340T, PIC18F25K80)
- **Device path**: `/dev/cu.usbserial-110`
- **Baud rate**: 38400 (factory default, confirmed)
- **Vehicle**: 2017 Ford F-150 EcoBoost 3.5L

---

## 10. Existing Assets

- `elm327-core` crate: serial, PTY, bridge, detect, config, error handling
- `elm327-simulator`: full AT command state machine for testing
- ELM327 Technical Reference: complete AT command set + timing
- FORScan profile: all 22 module addresses + firmware versions
- 78 passing tests including end-to-end simulator integration
