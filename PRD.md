# PRD: FORScan Compatibility Layer for macOS (Apple Silicon)

## 1. Overview

**Goal:**
Enable the Windows-only FORScan application to run on macOS (Apple Silicon) using Wine/CrossOver by providing a functional bridge between macOS serial devices and Windows COM port expectations.

**Core Idea:**
Implement a user-space serial bridge + Wine COM port mapping layer that allows FORScan to communicate with OBD adapters (e.g., ELM327) connected to macOS as if they were native Windows COM devices.

---

## 2. Problem Statement

FORScan depends on:
- Windows COM ports (COM1, COM2, etc.)
- Windows kernel-level serial drivers
- Low-latency bidirectional communication

macOS:
- exposes devices as /dev/cu.*
- does not support Windows drivers
- cannot natively satisfy FORScan's expectations

Wine:
- emulates Windows APIs (user space)
- does not support kernel drivers
- provides limited COM port mapping

**Result:**
FORScan can run under Wine, but cannot communicate with hardware.

---

## 3. Objectives

### Primary
- Enable FORScan to:
  - detect a COM port
  - open/close it
  - send/receive serial data reliably

### Secondary
- Support common adapters:
  - ELM327 (CH340, FTDI)
- Maintain low-latency communication
- Avoid kernel extensions (DriverKit preferred)

### Non-Goals
- Full driver emulation
- Perfect compatibility with all adapters
- ECU flashing safety guarantees

---

## 4. System Architecture

### High-Level Flow

```
FORScan (Wine)
   ↓
COM Port (Wine mapping)
   ↓
PTY (pseudo-terminal)
   ↓
Bridge Service (macOS)
   ↓
/dev/cu.* (real USB serial device)
   ↓
OBD Adapter
```

---

## 5. Components

### 5.1 Wine Layer
- Runs FORScan
- Maps COM3 → PTY device
- Uses Wine's dosdevices mapping:

```bash
ln -s /path/to/pty ~/.wine/dosdevices/com3
```

### 5.2 PTY Layer
- Creates virtual serial endpoints
- Allows bidirectional communication

```
ptyA <-> ptyB
```

- Wine connects to ptyA
- Bridge connects to ptyB

### 5.3 Bridge Service (Core Component)

**Language:** Go / Rust / Python (initial), Rust preferred for performance

**Responsibilities:**
- Open real serial device (/dev/cu.wchusbserial*)
- Open PTY endpoint
- Forward data both directions:
  - PTY → Serial
  - Serial → PTY
- Handle:
  - baud rate
  - flow control
  - reconnect logic

**Features:**
- configurable baud rate
- logging/debug mode
- auto device detection
- error handling / retries

### 5.4 Device Detection Module
- Enumerate: `/dev/cu.*`
- Filter likely OBD devices:
  - wchusbserial
  - usbserial
  - SLAB_USBtoUART
- Optional:
  - probe device with ATZ command

### 5.5 Configuration Layer

Config file (YAML/JSON):

```yaml
device: /dev/cu.wchusbserial1410
baud_rate: 115200
wine_com_port: COM3
auto_reconnect: true
logging: true
```

---

## 6. Data Flow

Example interaction:

1. FORScan sends: `ATZ`
2. Wine → PTY
3. Bridge → macOS serial
4. Adapter responds: `ELM327 v1.5`
5. Response flows back to FORScan

---

## 7. Key Technical Challenges

### 7.1 Latency
- Serial communication must be low-latency
- Solution:
  - non-blocking I/O
  - buffered streams
  - minimal processing overhead

### 7.2 COM Port Behavior
FORScan may expect:
- specific timeouts
- control signals (RTS/CTS)

Mitigation:
- emulate minimal required behavior
- ignore unsupported signals initially

### 7.3 Wine Compatibility
- COM mapping inconsistencies
- need stable PTY mapping

### 7.4 Adapter Variability
- CH340 vs FTDI differences
- inconsistent firmware

Mitigation:
- test with multiple adapters
- fallback configurations

---

## 8. Implementation Phases

### Phase 1: Proof of Concept
- Create PTY pair
- Build simple byte-forwarding bridge
- Map Wine COM → PTY
- Verify: manual serial communication works

### Phase 2: FORScan Integration
- Launch FORScan via Wine
- Attempt connection to COM port
- Debug:
  - handshake issues
  - timing problems

### Phase 3: Stability
- reconnect handling
- logging
- configuration system

### Phase 4: Packaging
- CLI tool
- optional GUI wrapper
- Homebrew installable package

---

## 9. Success Criteria
- FORScan detects COM port
- Adapter responds to basic commands:
  - ATZ
  - ATI
- Stable connection for:
  - reading codes
  - basic diagnostics

---

## 10. Risks

**High Risk**
- FORScan uses unsupported Windows APIs
- timing mismatches cause instability

**Medium Risk**
- Wine COM behavior inconsistencies
- adapter-specific quirks

**Low Risk**
- basic serial bridging functionality

---

## 11. Future Enhancements
- GUI device selector
- multi-device support
- Bluetooth adapter support
- protocol-level optimizations

---

## 12. Alternatives Considered

| Approach | Reason Rejected |
|---|---|
| File-based I/O | Not real-time |
| macOS driver rewrite | Overkill |
| Full Windows VM | Already exists (Parallels) |

---

## 13. Open Questions
- Does FORScan require non-standard serial APIs?
- How strict are timing requirements?
- Can Wine fully emulate required COM semantics?

---

## Final Note

This is: **a fun engineering project with a non-zero chance of working**

It is not: **a reliable replacement for a $30 scanner or a proper adapter**

But if you pull this off, you get:
- native-ish Mac workflow
- no Windows dependency
- and the deeply satisfying feeling of beating a very dumb stack at its own game
