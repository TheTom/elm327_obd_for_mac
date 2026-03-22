# FORScan Complete Adapter Support Database

Extracted from FORScan.exe v2.3.70 runtime memory dump (108MB).

---

## Supported Adapter Types

| Adapter | Detection String | Interface | MS-CAN Method |
|---------|-----------------|-----------|---------------|
| ELM327 (PIC clone) | `ELM327v` | USB/BT/WiFi | `ATTPB` (manual switch) |
| OBDLink EX | `OBDLink` / STN2230/2231 | USB | `STPBR%i` (electronic) |
| OBDLink MX+ | `OBDLink` / STN2255 | Bluetooth | `STPBR%i` (electronic) |
| vLinker FS | `vLinker` / MIC3422/3322 | USB/BT | `VTPBR%i` (electronic) |
| ELS27 | STN1170/2120 | USB | `STPBR%i` (electronic) |
| ForDiag | `ForDiag Adapter` | USB | `ATTPB` + `AT@2` |
| OBDKey | `OBDKey Adapter` | USB | `ATCSP%06X` |
| AGV4000 | `OBD-Diag AGV4000B` | USB | `ATCA` + `ATOHS` |
| CANtieCAR | `CANtieCAR (FORScan mode)` | USB | J2534 |
| OpenPort 2.0 | `OpenPort 2.0 J2534 ISO/CAN/VPW/PWM` | USB | J2534 |
| Ford VCM-II | `Ford-VCM-II` | USB | J2534 |
| Mongoose-Plus | `Mongoose-Plus Ford2` | USB | J2534 |
| VXDIAG | `VXDIAG` | USB | J2534 |
| PLXKiwi | `PLXKiwi` | WiFi/BT | ELM327 |
| CHIPSOFT | `CHIPSOFT J2534 Mid ISO/CAN/SWCAN/GMUART` | USB | J2534 |
| UCDS-J2534 | `UCDS-J2534` | USB | J2534 |
| SM2 USB | `SM2 USB` | USB | J2534 |

## Adapter Detection Order

FORScan detects adapters by checking: `STN` → `vLinker` → `ELM327v` → fallback

---

## ELM327 AT Commands (Complete)

### Protocol & Init
| Command | Description |
|---------|-------------|
| `ATZ` | Full reset |
| `ATWS` | Warm start (soft reset) |
| `ATI` | Print version ID |
| `AT@2` | Device identifier |
| `ATE` | Echo on/off |
| `ATL` | Linefeed on/off |
| `ATH` | Headers on/off |
| `ATR` | Responses on/off |
| `ATS` | Spaces on/off (`ATS0` = off) |
| `ATAL` | Allow Long messages |
| `ATNL` | Normal Length messages |
| `ATBI` | Bypass Initialization |
| `ATPC` | Protocol Close |
| `ATSI` | Slow Init (ISO) |

### Protocol Selection
| Command | Description |
|---------|-------------|
| `ATSP%X` | Set Protocol (saved) |
| `ATTP%X` | Try Protocol (not saved) |
| `ATTP3` | Try Protocol 3 (ISO 9141) |
| `ATTPB` | Try Protocol B (MS-CAN 125k) |
| `ATP%X` | Short protocol command |

### CAN Configuration
| Command | Description |
|---------|-------------|
| `ATSH%06X` | Set Header (6 hex chars, e.g., `ATSH0007E0`) |
| `ATCRA%03X` | Set CAN Receive Address filter |
| `ATCR709-7EF` | CAN receive range (custom?) |
| `ATCF700` | CAN filter |
| `ATCMF00` | CAN mask |
| `ATCAF` | CAN Auto Formatting |
| `ATCEA%02X` | CAN Extended Address |
| `ATCEA` | Clear CAN Extended Address |
| `ATCSP%06X` | CAN Speed Protocol (OBDKey-specific, 6-digit hex) |
| `ATCT%03X` | CAN timing parameter |
| `ATAR` | Auto Receive |

### Timing
| Command | Description |
|---------|-------------|
| `ATST%02X` | Set Timeout (hex, × 4ms) |
| `ATAT%i` | Adaptive Timing (0=off, 1=auto1, 2=auto2) |
| `ATAT1` | Adaptive Timing ON |
| `ATTA30` | Unknown timing command |

### Programmable Parameters (Protocol B / MS-CAN)
| Command | Description |
|---------|-------------|
| `ATPP2ASV38` | Set PP 2A = 0x38 (CAN config byte) |
| `ATPP2AON` | Enable PP 2A |
| `ATPP2AOFF` | Disable PP 2A (STN adapters only) |
| `ATPP2CSV81` | Set PP 2C = 0x81 (Protocol B: CAN 11-bit) |
| `ATPP2CON` | Enable PP 2C |
| `ATPP2DSV04` | Set PP 2D = 0x04 (baud divisor: 500k/4 = 125k) |
| `ATPP2DON` | Enable PP 2D |
| `ATPPS` | Print PP summary |
| `ATPP%02XSV%02X` | Generic: set PP XX to value YY |
| `ATPP%02XON` | Generic: enable PP XX |

### Baud Rate
| Command | Description |
|---------|-------------|
| `ATBRD` | Baud Rate Divisor |
| `ATIB%02X` | ISO Baud rate |

### Other
| Command | Description |
|---------|-------------|
| `ATRV` | Read Voltage |
| `ATMC3B` | Monitor CAN address 3B |
| `ATMC6E` | Monitor CAN address 6E |
| `ATONI` | Unknown (OBDKey/AGV specific?) |
| `ATOHS` | Unknown (AGV specific?) |
| `ATCA` | Unknown (AGV specific?) |
| `?ATRD` | Read stored data (with error check) |

---

## STN Commands (OBDLink / ELS27)

| Command | Description |
|---------|-------------|
| `STDI` | Device Info |
| `STSN` | Serial Number |
| `STBR%i` | Set Baud Rate (serial) |
| `STPBR%i` | Set Protocol B Baud Rate (e.g., `STPBR125000`) |
| `STP53` | Set Protocol 53 |
| `STPTOT%i` | Protocol Timeout |
| `STCSTM0` | CAN Silent Monitoring OFF (enable transmit) |
| `STCSTM0.01` | STCSTM variant |
| `STCSTM0.1` | STCSTM variant |
| `STCSTM0.13` | STCSTM variant |
| `STCAFCP%03X,%03X` | CAN Auto Format with CAN Protocol |
| `STCCFCP` | CAN Conformance Check |
| `STCSEGR1` | CAN Segment Register 1 |
| `STPPMC` | PP Memory Clear |
| `STPPMA2000,7DF,3E80` | PP Multi-Address (broadcast keepalive) |
| `STPPMA%i,%03X,%02X%s` | PP Multi-Address (generic) |
| `STPFEPS` | Flash EEPROM Programming Supply |
| `STPXl:%u` | Protocol X length |
| `STPXl:%u,r:1` | Protocol X length with response |
| `STPXd:` | Protocol X data |
| `STGPOW34:0` | GPIO pin 34 power off |
| `STGPOW34:1` | GPIO pin 34 power on |
| `STGPC34:O` | GPIO pin 34 configure output |
| `STSLVL off,off` | Signal Level off |
| `ATPP2AOFF` | Disable PP 2A (done for STN adapters) |

---

## vLinker Commands

| Command | Description |
|---------|-------------|
| `VTVERS` | Version info |
| `VTPBR%i` | Set Protocol B Baud Rate (e.g., `VTPBR125000`) |
| `VTP253` | Set Protocol 253 |
| `VTSET_HD%03X,%03X` | Set Header (request, response) |
| `VTPMQE1,%03X,%02X%s,%i,11` | Multi-query (complex) |
| `VTPMQEFF` | Multi-query flush |
| `VTSWGPBOOST0` | Software GP boost off |
| `VTSWGPBOOST1` | Software GP boost on |
| `VTSWGPGR1` | Software GP group 1 |
| `VTTOST REQ:%i` | Timeout request |
| `VTTOSTCFST:20` | Timeout config 20ms |
| `VTTOSTCFST:120` | Timeout config 120ms |
| `VTTOSTCFST:140` | Timeout config 140ms |
| `VTFEPS` | Flash EEPROM Programming Supply |
| `VTFullyRequest` | Full request mode |
| `VTFullyRequestCk%04X%s%04X` | Full request with checksum |
| `VTSDST00` | SD status 00 |
| `VTDLDT` | Download data |
| `VTDLED` | Download LED |
| `VTDL` | Download |
| `VTUART_BUAD_SET %i,100` | UART baud rate set |

---

## Firmware Update Files (OBDLink)

| File | Adapter |
|------|---------|
| `obdlink_mxp-5.8.1-stn2255.bin` | OBDLink MX+ |
| `obdlink_ex-5.8.1-stn2231.bin` | OBDLink EX (v1) |
| `obdlink_ex-5.8.1-stn2230.bin` | OBDLink EX (v2) |
| `stn2120-5.10.1.bin` | ELS27 / STN2120 |
| `vLinker_FS_v2.3.04.txt` | vLinker FS USB |
| `vLinker_FS_BT_v2.3.04.txt` | vLinker FS Bluetooth |

---

## Protocol Types

| Protocol | Description |
|----------|-------------|
| Ford HS CAN | High Speed CAN 500k (primary) |
| Ford HS CAN (STN1170) | HS-CAN via STN adapter |
| Ford HS CAN (vLinker FD) | HS-CAN via vLinker |
| Ford MS CAN | Medium Speed CAN 125k (body) |
| Ford MS CAN (STN1170) | MS-CAN via STN adapter |
| Ford MS CAN (vLinker FD) | MS-CAN via vLinker |
| Ford MS CAN GW | MS-CAN via Gateway |
| Ford MS CAN GW (STN1170) | MS-CAN GW via STN |
| Ford MS CAN GW(vLinker FD) | MS-CAN GW via vLinker |
| Ford CAN (Full) | Full CAN (both HS+MS) |
| Ford ISO | ISO 9141/KWP2000 |
| Ford SCP | Standard Corporate Protocol (J1850 PWM) |
| HSCAN2-5 | Additional HS-CAN buses |
| MSCAN2 | Additional MS-CAN bus |

---

## As-Built Data

### DID Format
- `22DE%02X` — Read As-Built block (e.g., `22DE01`, `22DE02`)
- `2EDE%02X` — Write As-Built block
- Response: `62DE%02X` + data bytes

### Error Responses
- `7F2200000012` — General read error
- `7F22xxxx0012` — Specific DID not supported (0x12 = subFunctionNotSupported)

---

## Security Access

### From Memory Dump
- Request seed: `2701` → Response: `67010037` (seed = 0x0037)
- Send key: `27021EB9` → Response: `7F27021EB933` (NRC 0x33 = securityAccessDenied)

---

## Error Strings

| Error | Description |
|-------|-------------|
| `CAN ERROR` | CAN bus transmission error (no ACK) |
| `BUS ERROR` | Protocol bus error |
| `DATA ERROR` | Data checksum/format error |
| `NO DATA` | No response within timeout |
| `BUFFER FULL` | CAN receive buffer overflow |
| `STOPPED` | Command interrupted |
| `RX ERROR` | Receive error |
| `FB ERROR` | Feedback error |
| `UNKNOWN COMMAND` | Unrecognized AT command |
| `BUS INIT:...` | Bus initialization sequence |
| `Bad adapter - not compatible with ELM327` | Clone detection failed |

---

## Settings Keys (from preferences)

| Key | Description |
|-----|-------------|
| `BAUD_RATE_PREF` | Preferred serial baud rate |
| `BAUD_RATE_SEL` | Selected baud rate |
| `COM_PORT_PREF` | Preferred COM port |
| `COM_PORT_SEL` | Selected COM port |
| `TRANSPORT_PREF` | Transport type (COM/FTDI/BT/WiFi/J2534) |
| `FTDI_PORT_SEL` | FTDI port selection |
| `AUTO_CONNECT` | Auto-connect on startup |
