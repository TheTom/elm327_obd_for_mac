# ELM327 USB OBD2 Adapter Technical Reference
## CH340-Based USB Adapter with MS-CAN/HS-CAN Switch for Ford Vehicles

---

## 1. CH340 Driver on macOS (Apple Silicon)

### Driver Source
- **Official driver**: [WCH ch34xser_macos](https://github.com/WCHSoftGroup/ch34xser_macos)
- Supports: CH340, CH341, CH342, CH343, CH344, CH346, CH9101, CH9102, CH9103, CH9104, CH9143, CH339
- macOS 10.9 through current (Big Sur, Monterey, Ventura, Sonoma, Sequoia)

### Apple Silicon Compatibility
- For macOS 11.0+ **without Rosetta**: install the **DMG format** (`CH34xVCPDriver.dmg`), not the PKG
- For macOS 11.0+: after install, launch the app via LaunchPad and click "Install"
- Must allow the system extension in **System Settings > Privacy & Security**
- **No SIP disable required** — current drivers are properly signed by WCH

### Device Path
```
/dev/cu.wchusbserial<N>     # e.g., /dev/cu.wchusbserial14340
/dev/tty.wchusbserial<N>    # blocking variant
```
- The numeric suffix varies per device instance
- Use `cu.*` for outgoing/non-blocking connections (preferred for serial comms)
- Use `tty.*` for incoming/blocking connections
- Verify with: `ls /dev/cu.wchusbserial*`

### USB VID/PID
| Chip | Vendor ID | Product ID |
|------|-----------|------------|
| CH340/CH340T/CH341 | `0x1A86` | `0x7523` |

- Verify with: `system_profiler SPUSBDataType` — look for "QinHeng Electronics"

### Known Issues
- macOS 10.12+ (High Sierra, Mojave): some users report unexpected restarts on first connect — uninstall old drivers first
- macOS Ventura+: some users report the built-in CDC driver may conflict with the WCH driver. If the device appears as `/dev/cu.usbserial-*` instead of `/dev/cu.wchusbserial*`, the built-in driver is handling it (this is fine, it still works)
- **macOS may include built-in CDC drivers** for CH340 on recent versions — third-party install may not be necessary. Test first by plugging in and checking `ls /dev/cu.*`
- USB power management: macOS may put USB devices to sleep. Use `pmset` or keep the adapter active

---

## 2. ELM327 Serial Communication Protocol

### Default Settings (Factory)
| Parameter | Value |
|-----------|-------|
| **Baud Rate** | 38400 bps |
| **Data Bits** | 8 |
| **Parity** | None |
| **Stop Bits** | 1 |
| **Flow Control** | None |

### Connection String (for `screen`, `minicom`, `pyserial`, etc.)
```
38400 8N1, no flow control
```

### pyserial Example
```python
import serial
ser = serial.Serial(
    port='/dev/cu.wchusbserial14340',
    baudrate=38400,
    bytesize=serial.EIGHTBITS,
    parity=serial.PARITY_NONE,
    stopbits=serial.STOPBITS_ONE,
    timeout=10,           # 10 seconds for init
    xonxoff=False,        # no software flow control
    rtscts=False,         # no hardware flow control
    dsrdtr=False          # no DSR/DTR flow control
)
```

### macOS `screen` Example
```bash
screen /dev/cu.wchusbserial14340 38400
```

### Changing the Baud Rate
The serial baud rate can be changed via Programmable Parameter 0C:
```
ATPP 0C SV <hex_divisor>    # Set baud rate divisor
ATPP 0C ON                   # Enable the parameter
ATZ                          # Reset to apply
```
Baud rate divisors (PP 0C values):
| PP 0C Value | Baud Rate |
|-------------|-----------|
| `00` | 9600 (if pin 6 = 0V at boot) |
| `00` (default) | 38400 |
| `23` | 9600 |
| `08` | 57600 |
| `06` | 115200 |
| `03` | 230400 |
| `01` | 500000 |

### Alternative Baud Rate Selection (Runtime)
```
AT BRD <divisor>    # Try baud rate divisor (runtime, not saved)
AT BRT <timeout>    # Set baud rate handshake timeout
```
After `AT BRD`, the ELM327 responds "OK" at the OLD baud rate, then switches. You must also switch your terminal to the new rate and send a CR to confirm.

---

## 3. AT Command Set (ELM327 v1.5)

### Command Format
- All commands start with `AT` (case-insensitive)
- Terminated with carriage return (`\r`, `0x0D`)
- Spaces are ignored (i.e., `AT SP 0` == `ATSP0`)
- `#` in syntax means 0 or 1

### General Commands
| Command | Description |
|---------|-------------|
| `ATZ` | Reset all (full reset, returns ID string) |
| `ATWS` | Warm start (soft reset, faster than ATZ) |
| `ATD` | Set all to defaults |
| `ATI` | Print version ID (e.g., "ELM327 v1.5") |
| `AT@1` | Display device description |
| `AT@2` | Display device identifier |
| `AT@3 <string>` | Store device identifier |
| `ATE0` / `ATE1` | Echo off / on |
| `ATL0` / `ATL1` | Linefeed off / on |
| `ATM0` / `ATM1` | Memory off / on |
| `ATLP` | Go to low power mode |
| `ATRD` | Read stored data |
| `ATSD <hh>` | Store data byte |
| `ATFE` | Forget events |
| `ATIGN` | Read ignition monitor input level |

### OBD Protocol Commands
| Command | Description |
|---------|-------------|
| `ATSP <h>` | Set protocol to `h` (saved to memory) |
| `ATSP A<h>` | Set protocol `h` with auto-search fallback |
| `ATSP 00` | Reset to auto-detect |
| `ATTP <h>` | Try protocol `h` (not saved) |
| `ATTP A<h>` | Try protocol `h` with auto-search |
| `ATDP` | Describe current protocol (text) |
| `ATDPN` | Describe current protocol (number) |
| `ATPC` | Protocol close (end session) |
| `ATBI` | Bypass initialization sequence |
| `ATSS` | Set standard (SAE J1978) search order |

### Protocol Numbers (ATSP / ATTP values)
| Protocol # | Description | Speed |
|------------|-------------|-------|
| `0` | Automatic | Auto |
| `1` | SAE J1850 PWM | 41.6 kbaud |
| `2` | SAE J1850 VPW | 10.4 kbaud |
| `3` | ISO 9141-2 | 5 baud init, 10.4 kbaud |
| `4` | ISO 14230-4 KWP2000 | 5 baud init, 10.4 kbaud |
| `5` | ISO 14230-4 KWP2000 | fast init, 10.4 kbaud |
| `6` | ISO 15765-4 CAN | 11-bit ID, 500 kbaud |
| `7` | ISO 15765-4 CAN | 29-bit ID, 500 kbaud |
| `8` | ISO 15765-4 CAN | 11-bit ID, 250 kbaud |
| `9` | ISO 15765-4 CAN | 29-bit ID, 250 kbaud |
| `A` | SAE J1939 CAN | 29-bit ID, 250 kbaud |
| `B` | User-defined CAN | 11-bit ID (configurable via ATPB) |
| `C` | User-defined CAN | 29-bit ID (configurable via ATPB) |

### Response & Formatting Commands
| Command | Description |
|---------|-------------|
| `ATH0` / `ATH1` | Headers off / on |
| `ATS0` / `ATS1` | Spaces off / on (in hex output) |
| `ATR0` / `ATR1` | Responses off / on |
| `ATAL` | Allow long (>7 byte) messages |
| `ATNL` | Normal length (7 byte) messages only |
| `ATBD` | Buffer dump |
| `ATD0` / `ATD1` | DLC display off / on |
| `ATV0` / `ATV1` | Variable DLC off / on |
| `ATCAF0` / `ATCAF1` | CAN auto formatting off / on |

### Timing Commands
| Command | Description |
|---------|-------------|
| `ATST <hh>` | Set timeout (hh x 4 ms, max 0xFF = 1020 ms) |
| `ATAT0` | Adaptive timing off |
| `ATAT1` | Adaptive timing auto1 (default) |
| `ATAT2` | Adaptive timing auto2 (more aggressive) |

### Header & Address Commands
| Command | Description |
|---------|-------------|
| `ATSH <xx yy zz>` | Set header bytes (3 bytes for CAN 11-bit) |
| `ATSH <xx yy zz aa bb>` | Set header (5 bytes for CAN 29-bit) |
| `ATAR` | Automatic receive |
| `ATRA <hh>` | Set receive address |
| `ATSR <hh>` | Set receive address (alternate) |
| `ATTA <hh>` | Set tester address |

### CAN-Specific Commands
| Command | Description |
|---------|-------------|
| `ATCF <xxx>` | Set CAN ID filter (11-bit) |
| `ATCF <xxxxxxxx>` | Set CAN ID filter (29-bit) |
| `ATCM <xxx>` | Set CAN ID mask (11-bit) |
| `ATCM <xxxxxxxx>` | Set CAN ID mask (29-bit) |
| `ATCP <hh>` | Set CAN priority bits |
| `ATCRA <xxx>` | Set CAN receive address filter |
| `ATCRA` | Reset CAN receive address filter |
| `ATCEA <hh>` | Set CAN extended address |
| `ATCEA` | Turn off CAN extended address |
| `ATCSM0` / `ATCSM1` | CAN silent monitoring off / on |
| `ATCFC0` / `ATCFC1` | CAN flow control off / on |
| `ATPB <xx yy>` | Set Protocol B parameters (user CAN protocol) |

### CAN Flow Control Commands
| Command | Description |
|---------|-------------|
| `ATFC SH <xxx>` | Flow control set header |
| `ATFC SD <data>` | Flow control set data |
| `ATFC SM <h>` | Flow control set mode (0=defaults, 1=use SH/SD, 2=no FCs) |

### Monitor Commands
| Command | Description |
|---------|-------------|
| `ATMA` | Monitor all (raw bus data) |
| `ATMR <hh>` | Monitor for receiver = hh |
| `ATMT <hh>` | Monitor for transmitter = hh |

### ISO-Specific Commands
| Command | Description |
|---------|-------------|
| `ATFI` | Perform fast initiation |
| `ATSI` | Perform slow initiation |
| `ATIB 10` | Set ISO baud to 10400 |
| `ATIB 12` | Set ISO baud to 12500 (v1.2+) |
| `ATIB 15` | Set ISO baud to 15625 (v1.2+) |
| `ATIB 48` | Set ISO baud to 4800 |
| `ATIB 96` | Set ISO baud to 9600 |
| `ATKW` | Display key words |
| `ATKW0` / `ATKW1` | Key word checking off / on |
| `ATSW <hh>` | Set wakeup interval (hh x 20 ms) |
| `ATSW 00` | Disable periodic wakeups |
| `ATWM <data>` | Set wakeup message |
| `ATIIA <hh>` | Set ISO init address |

### J1939 Commands
| Command | Description |
|---------|-------------|
| `ATJE` | J1939 Elm data format |
| `ATJHF0` / `ATJHF1` | J1939 header formatting off / on |
| `ATJS` | J1939 SAE data format |
| `ATJTM1` / `ATJTM5` | J1939 timer multiplier x1 / x5 |
| `ATDM1` | Monitor for DM1 messages |
| `ATMP <xxxx>` | Monitor for PGN (4 hex digits) |
| `ATMP <xxxx n>` | Monitor for PGN, get n messages |

### Voltage Commands
| Command | Description |
|---------|-------------|
| `ATRV` | Read input voltage |
| `ATCV <dddd>` | Calibrate voltage to dd.dd volts |
| `ATCV 0000` | Restore factory voltage calibration |

### Programmable Parameters
| Command | Description |
|---------|-------------|
| `ATPPS` | Print PP summary |
| `ATPP <xx> ON` | Enable PP xx |
| `ATPP <xx> OFF` | Disable PP xx |
| `ATPP <xx> SV <yy>` | Set PP xx value to yy |
| `ATPP FF ON` | Enable all PPs |
| `ATPP FF OFF` | Disable all PPs |

### Key Programmable Parameters
| PP | Default | Description |
|----|---------|-------------|
| `00` | `FF` | Command character ('AT' alternative) |
| `01` | `00` | Header bytes 1, 2, 3 defaults |
| `02` | `FF` | Function control flags |
| `03` | `32` | Response timeout (x4 ms = 200ms default) |
| `04`-`05` | | CAN auto-detect order |
| `06` | `F1` | Tester/source address (ISO) |
| `0C` | `00` | Baud rate divisor |
| `0E` | `9A` | CAN filter/mask defaults |

---

## 4. MS-CAN vs HS-CAN (Ford Vehicles)

### Bus Specifications
| Bus | OBD Pins | Speed | Usage |
|-----|----------|-------|-------|
| **HS-CAN** (High Speed) | Pin 6 (CAN-H), Pin 14 (CAN-L) | 500 kbps | Powertrain, ABS, airbags, standard OBD-II |
| **MS-CAN** (Medium Speed) | Pin 3 (CAN-H), Pin 11 (CAN-L) | 125 kbps | Body control, HVAC, instrument cluster, doors, windows, seats |

### Physical Toggle Switch
The DPDT (Dual-Pole, Dual-Throw) switch is **purely hardware**:
- **HS position**: connects ELM327's CAN-H/CAN-L inputs to OBD pins 6 and 14
- **MS position**: connects ELM327's CAN-H/CAN-L inputs to OBD pins 3 and 11
- Switch type: 6-pin mini ON-ON switch (e.g., MTS-202-A2)

### What Changes at Software Level
When the physical switch changes the bus, FORScan must also change the **CAN protocol speed**:
- HS-CAN: `ATSP 6` (CAN 11-bit, 500 kbps) — standard protocol 6
- MS-CAN: Uses **Protocol B** (`ATSP B`) with custom baud rate of 125 kbps via `ATPB` command

FORScan handles this automatically — "there is no necessity to adjust any ELM327 parameters for MS CAN, FORScan makes all the necessary changes in an automated mode."

### CRITICAL WARNINGS
1. **NEVER toggle the switch while FORScan is actively communicating** — risk of corrupting module data
2. **NEVER mix up CAN-H and CAN-L** — reversed polarity can cause bus errors or ECU damage
3. **Only toggle when FORScan prompts you** — it will display a popup asking you to switch
4. **Do not hot-swap** during writes/programming operations

### Auto-Switch Alternatives
- **OBDLink EX** (STN2230): electronic switching, no manual toggle needed
- **ELS27** (STN1170): built-in MS-CAN support
- **vLinker FS**: electronic MS/HS switching
- These adapters switch via software commands (STN-specific extended commands, not standard ELM327)

---

## 5. OBD2 Protocols Supported

### Protocol Details

#### SAE J1850 PWM (Protocol 1)
- Used by: Ford (older, pre-CAN vehicles, roughly pre-2008)
- Speed: 41.6 kbaud
- Pins: 2 (Bus+) and 10 (Bus-)
- Pulse Width Modulation signaling

#### SAE J1850 VPW (Protocol 2)
- Used by: GM (older vehicles)
- Speed: 10.4 kbaud
- Pin: 2 (Bus+)
- Variable Pulse Width signaling

#### ISO 9141-2 (Protocol 3)
- Used by: European vehicles (older)
- Speed: 10.4 kbaud, 5 baud initialization
- Pin: 7 (K-line), optionally pin 15 (L-line)

#### ISO 14230-4 KWP2000 (Protocols 4 & 5)
- Used by: European and Asian vehicles (older)
- Speed: 10.4 kbaud
- Protocol 4: 5 baud initialization
- Protocol 5: fast initialization
- Pin: 7 (K-line)

#### ISO 15765-4 CAN (Protocols 6, 7, 8, 9)
- Used by: All vehicles 2008+ (US mandate), many 2003+
- Protocol 6: 11-bit ID, 500 kbps (most common for modern vehicles)
- Protocol 7: 29-bit ID, 500 kbps
- Protocol 8: 11-bit ID, 250 kbps
- Protocol 9: 29-bit ID, 250 kbps
- Pins: 6 (CAN-H) and 14 (CAN-L)

#### SAE J1939 (Protocol A)
- Used by: Heavy-duty trucks/diesel
- 29-bit ID, 250 kbps

#### User CAN Protocols (B and C)
- Protocol B: User-defined CAN, 11-bit ID
- Protocol C: User-defined CAN, 29-bit ID
- Configured via `ATPB <xx yy>` command
- **MS-CAN at 125 kbps uses Protocol B**

### Protocol Selection via AT Commands
```
ATSP 0       # Auto-detect (tries all protocols)
ATSP 6       # Explicitly select CAN 11-bit 500k (Ford HS-CAN)
ATSP B       # User-defined CAN (used for MS-CAN at 125k)
ATPB C0 29   # Configure Protocol B for 125kbps, 11-bit
ATTP 6       # Try protocol 6 without saving
```

### Auto-Detection Order
When `ATSP 0` is used, the ELM327 tries protocols in this order:
6, 8, 1, 7, 9, 2, 3, 4, 5, A

---

## 6. FORScan-Specific Behavior

### Platform Support
- **Windows**: Native (primary platform)
- **macOS**: Via CrossOver, Wine/Wineskin, or VirtualBox/Parallels
  - CrossOver: Run FORScan installer inside a Windows bottle
  - USB passthrough required for VM approaches
  - CH340 driver must be installed in host macOS AND guest Windows (for VM)
- **iOS/Android**: FORScan Lite (Bluetooth/WiFi adapters only)

### What FORScan Expects from the Adapter
1. **Full ELM327 v1.0 command set compatibility** — FORScan uses service-level protocols, not just basic OBD-II
2. **Support for commands longer than 2 bytes** — clone adapters that truncate to 2 bytes will fail
3. **ATPPS support** — FORScan checks programmable parameters
4. **CAN filter/mask commands** — `ATCF`, `ATCM`, `ATCRA` must work
5. **Protocol B support** — needed for MS-CAN
6. **Proper response to all standard AT commands** — `ATS0`, `ATAT1`, `ATST`, `ATR1`

### FORScan Initialization Sequence (Observed)
1. Adapter detection (scans COM ports / serial devices)
2. `ATZ` — reset
3. `ATI` — identify firmware version (checks for clone indicators)
4. `ATE0` — disable echo
5. `ATH1` — enable headers
6. `ATL0` — disable linefeeds
7. `ATS0` — disable spaces (for faster parsing)
8. `ATAT1` — adaptive timing on
9. `ATSP 6` — set protocol to CAN 500k 11-bit (Ford default)
10. Module scan — sends diagnostic requests to known Ford module addresses
11. If MS-CAN modules needed: prompts user to toggle switch, then reconfigures for Protocol B at 125k

### Clone Detection
FORScan actively detects "bad clones" and will warn:
- "Bad clone" warning if critical commands fail
- ~80% of adapters sold as "ELM327" are clones
- ~80% of those clones are "bad clones" incompatible with FORScan

### Recommended Adapters for FORScan
| Adapter | Chip | MS-CAN | Interface | Price |
|---------|------|--------|-----------|-------|
| OBDLink EX | STN2230 | Electronic auto-switch | USB | ~$70 |
| OBDLink MX+ | STN2255 | Electronic auto-switch | Bluetooth | ~$100 |
| vLinker FS | STN1170 | Electronic auto-switch | USB/BT/WiFi | ~$30-40 |
| ELS27 | STN1170/2120 | Built-in | USB | ~$40-60 |
| ELM327 USB (PIC18F25K80) | PIC clone | Manual toggle | USB | ~$10-20 |

---

## 7. ELM327 v1.5 vs v2.1 — Real vs Clone

### The Truth About Version Numbers
- **Elm Electronics (genuine)** released: v1.0, v1.2, v1.3, v1.3a, v1.4, v1.4b, v2.0, v2.1, v2.2, v2.3
- **Genuine chips are not available for retail purchase** — they're sold to OEM manufacturers only
- **ALL consumer "v1.5" and most "v2.1" adapters are clones** — Elm Electronics never released v1.5

### Clone Generations

#### "v1.5" PIC-based clones (BETTER)
- Chip: **PIC18F25K80** microcontroller with 4 MHz crystal
- These are copies of the original PIC-based ELM327 firmware
- Support most/all of the ELM327 v1.0 command set
- **Work with FORScan** (if using PIC18F25K80)
- Respond correctly to `ATPPS` with a list of hex values
- Support CAN filters/masks, long commands, Protocol B
- **This is what you want for FORScan**

#### "v2.1" ARM-based clones (WORSE)
- Chip: cheap ARM microcontroller (often STM32 or similar)
- Incomplete re-implementation of ELM327 firmware
- **Only reads first 2 bytes of commands** — anything longer is silently truncated
- Missing many AT commands: `ATPPS`, `ATR1`, `ATS0`, `ATAT1`, `ATCF`, `ATCM`
- Optimized only for basic OBD-II PIDs (Mode 01/02)
- **Does NOT work with FORScan** for advanced diagnostics
- May return fake/static data for some PIDs
- Some units only support specific vehicle brands (e.g., Hyundai only)

### How to Identify Your Clone
```
# Connect and send:
ATI          # Should return "ELM327 v1.5" or "ELM327 v2.1"
ATPPS        # Good clone: returns hex table. Bad clone: "?" or no response
ATCRA 7E8    # Good clone: "OK". Bad clone: "?" or error
ATS0         # Good clone: "OK". Bad clone: "?"
```

### CH340-Specific Clone Quirks
- CH340 is just the USB-to-UART bridge — it doesn't affect OBD functionality
- CH340 reliability is generally fine; the issues are in the ELM327 firmware clone quality
- CH340T vs CH340G: electrically equivalent for this purpose, same VID/PID
- Some cheap boards have poor power regulation causing intermittent disconnects
- USB cable quality matters — long/thin cables cause voltage drop

---

## 8. Serial Communication Timing

### Command Termination
- **Send**: terminate all commands with CR (`\r`, `0x0D`)
- **Receive**: ELM327 terminates lines with CR only (default) or CR+LF (if linefeed enabled via `ATL1` or hardware pin)
- **No need to send LF** — ELM327 ignores LF on input

### Prompt Character
- `>` (0x3E) — indicates ELM327 is idle and ready for next command
- Always appears after a response is complete
- **Wait for `>` before sending next command**

### Response Format
```
[echo of command if ATE1]\r
[response line 1]\r
[response line 2]\r
...
[response line N]\r
\r
>
```
Example (with echo on, headers off):
```
> 0100\r
0100\r
41 00 BE 3E B8 13\r
\r
>
```

### Timeouts
| Timeout | Value | Notes |
|---------|-------|-------|
| Command input timeout | ~20 seconds | If no CR received, aborts with `?` |
| OBD response timeout (default) | 200 ms | PP 03 default = 0x32, each unit = 4.096 ms |
| ATST range | 4 ms to 1020 ms | `ATST 01` = 4ms, `ATST FF` = 1020ms |
| Initialization timeout | 1-5 seconds | ATZ reset takes ~1 second |
| Protocol auto-detect | 5-15 seconds | ATSP 0 then 0100 can be slow |

### Adaptive Timing (ATAT)
- `ATAT0`: disabled — always uses full ATST timeout
- `ATAT1`: auto1 (default) — adjusts timeout based on responses
- `ATAT2`: auto2 — more aggressive timing optimization

### Inter-Command Delay
- Minimum: none needed if you wait for `>` prompt
- Recommended: 50-100 ms between rapid commands for clone stability
- After ATZ: wait at least 1 second before next command

### Error Responses
| Response | Meaning |
|----------|---------|
| `?` | Unrecognized command |
| `NO DATA` | No response from vehicle within timeout |
| `UNABLE TO CONNECT` | Protocol detection failed |
| `BUS INIT: ...ERROR` | Protocol initialization failed |
| `BUFFER FULL` | CAN receive buffer overflow (common on clones) |
| `CAN ERROR` | CAN bus error detected |
| `DATA ERROR` | Checksum error in received data |
| `<DATA ERROR` | CAN data error with partial data |
| `ERR` | General error |
| `FB ERROR` | Feedback error |
| `LV RESET` | Low voltage reset occurred |
| `STOPPED` | Interrupted by received character |
| `BUS BUSY` | Bus is busy |
| `ACT ALERT` | Activity monitor alert |
| `LP ALERT` | Low power alert |

---

## 9. USB-Serial Specifics

### CH340T USB Details
| Property | Value |
|----------|-------|
| USB Vendor ID | `0x1A86` (QinHeng Electronics) |
| USB Product ID | `0x7523` |
| USB Class | `0xFF` (Vendor Specific) |
| Driver name | `ch34xser` (macOS), `ch341` (Linux) |
| Max baud rate | 2,000,000 bps |
| Crystal | 12 MHz (external) or internal oscillator |

### macOS Enumeration
1. Plug in adapter → macOS detects USB device
2. If WCH driver installed: creates `/dev/cu.wchusbserial<N>` and `/dev/tty.wchusbserial<N>`
3. If built-in CDC driver handles it: creates `/dev/cu.usbserial-<ID>` or `/dev/cu.usbmodem<N>`
4. Check with: `ls /dev/cu.*` or `ioreg -p IOUSB -l` for detailed USB tree

### Power Management Concerns
- macOS USB power management may suspend the device after inactivity
- Workaround: send periodic `AT` commands (returns `OK`) as keepalive
- Some cheap adapters draw power from OBD port (pins 16=+12V, 4/5=GND) and don't need USB power
- If adapter has both USB power and OBD power, the CH340 runs on USB power regardless

### Multiple Adapters
- Each CH340 adapter gets a unique serial number suffix
- Multiple adapters can coexist: `/dev/cu.wchusbserial14340`, `/dev/cu.wchusbserial14310`, etc.
- The suffix may change if plugged into a different USB port

---

## 10. Quick Start: Typical Initialization Sequence

```python
# Recommended initialization for Ford vehicle with ELM327 USB adapter

# 1. Open serial connection
# Port: /dev/cu.wchusbserial<N>
# Settings: 38400 8N1, no flow control, timeout=10s

# 2. Reset
send("ATZ\r")          # wait 1-2 seconds, expect "ELM327 v1.5"
wait_for_prompt()      # wait for ">"

# 3. Configure
send("ATE0\r")         # echo off → "OK"
wait_for_prompt()
send("ATL0\r")         # linefeed off → "OK"
wait_for_prompt()
send("ATH1\r")         # headers on → "OK"
wait_for_prompt()
send("ATS0\r")         # spaces off → "OK" (faster parsing)
wait_for_prompt()
send("ATAT1\r")        # adaptive timing on → "OK"
wait_for_prompt()

# 4. Check voltage (verify car is on)
send("ATRV\r")         # expect "12.3V" or similar
wait_for_prompt()

# 5. Set protocol (Ford HS-CAN)
send("ATSP6\r")        # CAN 11-bit, 500 kbps → "OK"
wait_for_prompt()

# 6. Test communication
send("0100\r")          # Request supported PIDs
wait_for_prompt()       # expect "41 00 XX XX XX XX"

# 7. For MS-CAN (after physical switch toggle):
send("ATSP B\r")        # Switch to user CAN protocol → "OK"
send("ATPB C0 29\r")    # Set 125 kbps → "OK"  # TODO: verify exact ATPB values for 125k
```

---

## Sources

- [WCH CH34x macOS Driver (GitHub)](https://github.com/WCHSoftGroup/ch34xser_macos)
- [ELM327 Datasheet (SparkFun mirror)](https://cdn.sparkfun.com/assets/learn_tutorials/8/3/ELM327DS.pdf)
- [ELM327 AT Commands Reference (SparkFun)](https://cdn.sparkfun.com/assets/4/e/5/0/2/ELM327_AT_Commands.pdf)
- [ELM327 AT Command Set (libreXC wiki)](https://github.com/deshi-basara/libreXC/wiki/ELM327-AT-Command-Set)
- [ELM327 Wikipedia](https://en.wikipedia.org/wiki/ELM327)
- [can327 Linux kernel driver docs](https://docs.kernel.org/networking/device_drivers/can/can327.html)
- [python-OBD elm327.py (initialization sequence)](https://github.com/brendan-w/python-OBD/blob/master/obd/elm327.py)
- [FORScan Forum: Known problems with China clones](https://forscan.org/forum/viewtopic.php?t=1575)
- [FORScan Forum: ELM327 with toggle switch](https://forscan.org/forum/viewtopic.php?t=3395)
- [FORScan Forum: How to choose adapters](https://forscan.org/forum/viewtopic.php?t=6142)
- [FORScan Forum: MS-CAN access](https://forscan.org/forum/viewtopic.php?t=4)
- [FORScan Documentation](https://forscan.org/documentation_13.html)
- [ELM327 MS/HS Switch Modification Guide](https://blog.uobdii.com/how-to-modify-elm327-to-add-hsms-can-switch/)
- [WindowsForum: ELM327 MS/HS Switch Analysis](https://windowsforum.com/threads/elm327-with-ms-hs-switch-for-forscan-pros-risks-and-alternatives.403830/)
- [OBDTester: ELM-USB Commands](https://www.obdtester.com/elm-usb-commands)
- [SparkFun: CH340 Driver Installation](https://learn.sparkfun.com/tutorials/how-to-install-ch340-drivers/all)
- [ScanTool.net: Switching Baud Rate](https://www.scantool.net/blog/switching-communication-baud-rate/)
- [FORScan macOS via CrossOver](https://forscan.org/forum/viewtopic.php?t=12572)
- [FORScan macOS Monterey Setup](https://forscan.org/forum/viewtopic.php?t=20840)
- [Counterfeit ELM327 Dissection](https://timyouard.wordpress.com/2015/09/02/disection-of-a-counterfeit-elm327-obdii-adapter-from-china/)
- [CVTz50: How to buy proper ELM327](http://cvtz50.info/en/elm327/)
- [Device Hunt: CH340 VID/PID](https://devicehunt.com/view/type/usb/vendor/1A86/device/7523)
- [CH340 Drivers Guide](https://sparks.gogo.co.nz/ch340.html)
