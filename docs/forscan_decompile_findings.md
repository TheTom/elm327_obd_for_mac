# FORScan Decompilation Findings — MS-CAN Protocol B

## Source
- Memory dump of FORScan.exe (v2.3.70) running on Windows
- 108MB .DMP file analyzed with strings extraction
- 804,502 strings extracted from runtime memory

## Critical Findings

### How FORScan Handles Different Adapters

FORScan has separate code paths for different adapter types:

| Adapter Type | Protocol B Command | Baud Rate Command |
|---|---|---|
| **ELM327 clone** | `ATTPB` | Uses stored PP 2C/2D values |
| **OBDKey** | `ATCSP%06X` (6-digit hex CAN speed) | Via ATCSP |
| **STN1170** | `STPBR%i` (e.g., STPBR125000) | Direct baud |
| **vLinker** | `VTPBR%i` (e.g., VTPBR125000) | Direct baud |
| **ForDiag** | `ATTPB` then `AT@2` | Via PPs |

### ELM327 MS-CAN Init Sequence (from memory strings)

For ELM327 adapters, FORScan's Protocol B sequence is:
1. `ATTPB` — Try Protocol B (uses PP 2C/2D stored values)
2. NO `ATPB xxxx` command is sent — it relies entirely on stored PPs
3. The PPs must already be configured for 125 kbps

### Key AT Commands Found

```
ATSP%X      — Set Protocol (hex digit, e.g., ATSP6)
ATTP%X      — Try Protocol (hex digit, e.g., ATTP6, ATTPB)
ATP%X       — Short protocol command
ATCSP%06X   — CAN Speed Protocol (6-digit hex, OBDKey-specific)
ATTPB       — Try Protocol B (ELM327 MS-CAN)
ATSH%06X    — Set Header (6 hex chars, e.g., ATSH0007E0)
ATCRA%03X   — CAN Receive Address filter
ATCF700     — CAN filter
ATCMF00     — CAN mask
ATCEA%02X   — CAN Extended Address
ATCT%03X    — Unknown CAN command (timeout?)
ATAT%i      — Adaptive Timing
ATBI        — Bypass Init
ATAL        — Allow Long messages
ATBRD       — Baud Rate Divisor
ATCAF       — CAN Auto Formatting
ATIB%02X    — ISO Baud rate
```

### STN-Specific Commands (STN1170 / OBDLink)

```
STPBR%i     — Set Protocol B Baud Rate (e.g., STPBR125000)
STBR%i      — Set Baud Rate
STCAFCP%03X — CAN Auto Format with CAN Protocol
STCCFCP     — CAN Conformance Check FCP
STCSTM0     — CAN Silent Monitoring OFF (critical for transmit!)
STCSTM0.13  — STCSTM with sub-parameter
STCSTM0.01  — STCSTM with sub-parameter
STPTOT%i    — Protocol Timeout
STP53       — Set Protocol 53 (?)
```

### vLinker-Specific Commands

```
VTPBR%i     — Set Protocol B Baud Rate
VTP253      — Set Protocol 253 (?)
VTPMQE1,%03X,%02X%s,%i,11 — Complex multi-param command
VTTOST REQ:%i — Timeout request
```

### MS-CAN Support Detection

FORScan checks for MS-CAN capability via:
- `MSCAN_SUPPORT_ELM327` — flag for ELM327 toggle switch adapters
- `MSCAN_SUPPORT_J2534` — flag for J2534 passthrough adapters
- Adapter types: "Ford MS CAN", "Ford MS CAN (STN1170)", "Ford MS CAN (vLinker FD)"
- Also: "Ford MS CAN GW" variants (gateway modules)

### Protocol Detection Strings

```
Ford HS CAN
Ford HS CAN (STN1170)
Ford HS CAN (vLinker FD)
Ford MS CAN
Ford MS CAN (STN1170)
Ford MS CAN (vLinker FD)
Ford MS CAN GW
Ford MS CAN GW (STN1170)
Ford MS CAN GW(vLinker FD)
HS CAN
HSCAN2, HSCAN3, HSCAN4, HSCAN5
MSCAN, MSCAN2
```

## Root Cause of Our CAN ERROR

Our adapter's PPs are:
- PP 2C = 0x81 (Protocol B type: CAN 11-bit with auto-format)
- PP 2D = 0x04 (Baud divisor: 500k/4 = 125 kbps)

FORScan sends `ATTPB` which uses these stored PP values.
We also sent `ATTPB` and got CAN ERROR.

**Conclusion:** The CAN ERROR is NOT a software/command issue.
FORScan uses the exact same command we tried (`ATTPB`).
The issue is hardware: our PIC18F25K80 clone cannot transmit
on Protocol B due to `ATCSM0` not being supported (silent
monitoring stuck ON). FORScan would also fail on MS-CAN
with this specific adapter.

**Tom's claim that FORScan works on MS-CAN with this adapter
needs to be re-verified with the truck connected.** The adapter
may work differently when there are active CAN nodes providing
ACK on the bus (vs. desktop testing with no bus).

## Next Steps

1. Test `ATTPB` with the truck connected (bus has active nodes to ACK)
2. If still fails, the adapter genuinely cannot transmit on Protocol B
3. For guaranteed MS-CAN: need STN1170-based adapter (OBDLink EX, vLinker FS)
