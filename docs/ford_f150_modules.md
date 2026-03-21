# 2017 Ford F-150 3.5L EcoBoost — Module Reference

**Source:** FORScan profile `Ford F-150 EcoBoost Gasoline Turbocharged Direct Injection 3_5L 2017`
**VIN:** 1FT*********84222
**Profile Version:** 2

---

## Module Summary

| FORScan ID | Bus | Module Name | Description | UDS Request/Response IDs |
|------------|-----|-------------|-------------|--------------------------|
| `0004` | HS-CAN (0x20) | OBD-II | Generic OBD-II / Emissions | 7DF / broadcast |
| `1847` | HS-CAN (0x130) | APIM | Accessory Protocol Interface Module (Sync 3) | 7D0 / 7D8 |
| `1FA9` | HS-CAN (0x130) | ABS | Anti-Lock Brake System / ESC | 760 / 768 |
| `2502` | HS-CAN (0x130) | BCM | Body Control Module | 726 / 72E |
| `25AC` | HS-CAN (0x130) | PSCM | Parking Aid / Sonar Module (GPSM) | 727 / 72F |
| `25AD` | HS-CAN (0x130) | ACM | Audio Control Module | 754 / 75C |
| `2795` | MS-CAN (0x120) | HVAC | HVAC / Climate Control Module | 733 / 73B |
| `2B17` | HS-CAN (0x130) | SCCM | Steering Column Control Module | 724 / 72C |
| `2B9B` | HS-CAN (0x130) | PSCM/EPAS | Power Steering Control Module (EPS) | 730 / 738 |
| `2C3A` | MS-CAN (0x120) | TPMS | Tire Pressure Monitoring System | 741 / 749 |
| `2DC4` | HS-CAN (0x130) | DDM | Driver Door Module | 740 / 748 |
| `2DD6` | HS-CAN (0x130) | TCU | Telematic Control Unit (modem) | 754 / 75C |
| `2DF6` | HS-CAN (0x130) | TCM | Transmission Control Module | 7E1 / 7E9 |
| `2DF7` | HS-CAN (0x130) | PCM | Powertrain Control Module | 7E0 / 7E8 |
| `2F6C` | HS-CAN (0x130) | IPMA | Image Processing Module A (camera) | 706 / 70E |
| `2FAB` | HS-CAN (0x130) | EPAS | Electronic Power Assist Steering Rack | 730 / 738 |
| `3038` | HS-CAN (0x130) | GWM | Gateway Module | 716 / 71E |
| `3134` | HS-CAN (0x130) | IPC | Instrument Panel Cluster | 720 / 728 |
| `34ED` | HS-CAN (0x130) | FCIM | Front Controls Interface Module (BECM) | 7A7 / 7AF |
| `365C` | HS-CAN (0x130) | TBM | Turbo Boost Module / Wastegate Control | 7E0 / 7E8 (sub) |
| `37ED` | HS-CAN (0x130) | IPMB | Image Processing Module B (rear camera) | 706 / 70E (sub) |
| `389D` | HS-CAN (0x130) | PDM | Passenger Door Module | 742 / 74A |

---

## Bus Types

| Bus Code | Bus Name | Description |
|----------|----------|-------------|
| `0x20` (32) | HS-CAN | High-Speed CAN (500 kbps) — OBD-II generic |
| `0x130` (304) | HS-CAN | High-Speed CAN (500 kbps) — Ford extended diagnostics |
| `0x120` (288) | MS-CAN | Medium-Speed CAN (125 kbps) — Body/comfort modules |

> **Note:** FORScan bus codes `0x130` and `0x20` both ride on the HS-CAN physical bus but use different diagnostic protocols. `0x130` = Ford proprietary UDS, `0x20` = standard OBD-II. `0x120` = MS-CAN bus.

---

## Firmware / Part Numbers (from MODNS lines)

Each MODNS entry follows the pattern:
`Software_PN ; Calibration_PN ; Calibration2_PN ; Serial/Strategy ; ... ; VIN ; Config_PN`

### PCM — `2DF7` (Powertrain Control Module)

| Field | Value |
|-------|-------|
| Software P/N | HL3A-12A650-BBB |
| Calibration P/N | HL3A-14C204-BMN |
| Strategy/Serial | 784301297030 |
| VIN | 1FTEW1EGXHKC84222 |
| Config P/N | HL3A-12B684-AYA |

### TCM — `2DF6` (Transmission Control Module)

| Field | Value |
|-------|-------|
| Software P/N | HL3A-12B565-GB |
| Calibration P/N | HL3A-14C337-EP |
| Strategy/Serial | YC03217UTC00020A |
| VIN | 1FTEW1EGXHKC84222 |
| Config P/N | HL3A-14F106-FA |

### APIM — `1847` (Sync 3 Infotainment)

| Field | Value |
|-------|-------|
| Software P/N | HP5T-14G371-CCA |
| Calibration P/N | GB5T-14G374-CB |
| Calibration 2 P/N | GB5T-14G375-CA |
| Extra | DemoAudioProfile |
| VIN | 1FTEW1EGXHKC84222 |
| Config P/N | HP5T-14G380-BA |

### ABS — `1FA9` (Anti-Lock Brakes / ESC)

| Field | Value |
|-------|-------|
| Software P/N | HL3T-3F944-AC |
| Calibration P/N | HC3T-14C579-AC |
| Serial | 0123456789ABCDEF |
| Config P/N | HL3T-14F078-AC |

### BCM — `2502` (Body Control Module)

| Field | Value |
|-------|-------|
| Software P/N | HL3T-14B321-AC |
| Calibration P/N | GR3T-14C028-AA |
| Calibration 2 P/N | HL3T-14C098-AC |
| Serial | 3053277237300000 |
| VIN | 1FTEW1EGXHKC84222 |
| Config P/N | HL3T-14F136-AC |

### PSCM/GPSM — `25AC` (Parking Aid / Sonar)

| Field | Value |
|-------|-------|
| Software P/N | GD9T-15K619-AB |
| Calibration P/N | GD9T-14G090-AB |
| Serial | 5100170000785697 |
| Config P/N | GB5T-14G096-AA |

### ACM — `25AD` (Audio Control Module)

| Field | Value |
|-------|-------|
| Software P/N | HL3T-18E245-ABC |
| Calibration P/N | HL3T-14G121-AB |
| Calibration 2 P/N | HL3T-14G122-AA |
| Serial | 14053604943884 |
| Extra | METHOD-3--NOT-SUPPORTED |
| VIN | 1FTEW1EGXHKC84222 |
| Config P/N | GL3T-14G127-AA |

### HVAC — `2795` (Climate Control)

| Field | Value |
|-------|-------|
| Software P/N | DS7T-14C708-BA |
| Calibration P/N | DS7T-14C030-BA |
| Date | 2013-02-15 |
| Config P/N | DS7T-14C249-BA |

### SCCM — `2B17` (Steering Column Control)

| Field | Value |
|-------|-------|
| Software P/N | DG9T-14B526-MA |
| Calibration P/N | DG9T-14G155-BD |
| Serial | 1652660074 |
| Config P/N | FL3T-14G161-AA |

### PSCM/EPAS — `2B9B` (Power Steering)

| Field | Value |
|-------|-------|
| Software P/N | HL34-3F964-DG |
| Calibration P/N | HR3C-14D003-AA |
| Calibration 2 P/N | HL34-14D004-DF |
| Serial | 0070044587 |
| Extra | HL34-14D007-GG |
| VIN | 1FTEW1EGXHKC84222 |
| Config P/N | HL34-14F079-AA |

### TPMS — `2C3A` (Tire Pressure Monitoring)

| Field | Value |
|-------|-------|
| Software P/N | GU5T-14B663-BA |
| Calibration P/N | GU5T-14C144-BA |
| Serial | 8TPG9 |
| Version | v12 |
| Config P/N | 9U5T-14C255-AA |

### DDM — `2DC4` (Driver Door Module)

| Field | Value |
|-------|-------|
| Software P/N | DG9T-14B533-EB |
| Calibration P/N | DG9T-14C108-CB |
| Serial | 201607211115 |
| Extra | FL3T-14C639-AS |
| VIN | 1FTEW1EGXHKC84222 |
| Config P/N | DG9T-14F144-AE |

### TCU — `2DD6` (Telematics Control Unit)

| Field | Value |
|-------|-------|
| Software P/N | HL3B-604A17-AA |
| Calibration P/N | HL3B-14G309-AA |
| Calibration 2 P/N | HL3B-14G310-AA |
| Serial | 3BBX285406967015 |
| Config P/N | FL3B-14G315-AB |

### IPMA — `2F6C` (Front Camera / Image Processing)

| Field | Value |
|-------|-------|
| Software P/N | HL3T-19C107-BC |
| Calibration P/N | HL3T-14D099-BC |
| Calibration 2 P/N | GL3T-14D100-BB |
| Extra | HL3T-14D100-BC |
| Config P/N | HL3T-14F188-BA |

### EPAS — `2FAB` (Electronic Power Steering Rack)

| Field | Value |
|-------|-------|
| Software P/N | HL34-2C219-AF |
| Calibration P/N | HL34-2D053-AF |
| Serial | 051704713864 |
| Extra | HL34-14C602-CCF |
| VIN | 1FTEW1EGXHKC84222 |
| Config P/N | GL34-14F065-AB |

### GWM — `3038` (Gateway Module)

| Field | Value |
|-------|-------|
| Software P/N | GL3T-14F642-AA |
| Calibration P/N | HL3T-14F530-AC |
| Serial | 2333440279 |
| Extra | HL3T-14F535-AC |
| Config P/N | EB3T-14F536-AA |

### IPC — `3134` (Instrument Panel Cluster)

| Field | Value |
|-------|-------|
| Software P/N | HL3T-10849-CJE |
| Calibration P/N | HL3T-14C026-CE |
| Calibration 2 P/N | HL3T-14C088-BD |
| Serial | PHCJE23560 |
| Extra | HL3T-14C088-CD |
| Config P/N | GL3T-14F094-CA |

### FCIM — `34ED` (Front Controls Interface / BECM)

| Field | Value |
|-------|-------|
| Software P/N | HU5T-14B476-AAJ |
| Calibration P/N | HU5T-14C184-AAJ |
| Serial | 17024193155 |
| Extra | HL3T-14C636-AD |
| Additional | HU5T-14F391-AAA; HL3T-14F390-AAB; HL3T-14F389-BA; HL3T-14G163-CAB; HL3T-14G162-AD |
| VIN | 1FTEW1EGXHKC84222 |
| Config P/N | HU5T-14F141-AAC |

### TBM — `365C` (Turbo Boost / Wastegate)

| Field | Value |
|-------|-------|
| Software P/N | HL3A-7H417-GC |
| Calibration P/N | HL3A-14C366-GD |
| Calibration 2 P/N | HL3A-14C367-GD |
| Serial | 38130170 |
| Config P/N | HL3A-14C365-GC |

### IPMB — `37ED` (Rear Camera / Image Processing B)

| Field | Value |
|-------|-------|
| Software P/N | FL3T-19H517-AH |
| Calibration P/N | FL3T-14C280-AG |
| Calibration 2 P/N | FL3T-14C281-AH |
| Serial | AU0R0NA000000000 |
| Config P/N | CK4T-14F121-AC |

### PDM — `389D` (Passenger Door Module)

| Field | Value |
|-------|-------|
| Software P/N | DG9T-14B531-EB |
| Calibration P/N | DG9T-14C064-CB |
| Serial | 201607211112 |
| Extra | FL3T-14C637-AS |
| VIN | 1FTEW1EGXHKC84222 |
| Config P/N | DG9T-14F142-AE |

---

## Ford Part Number Decoding

Ford part numbers follow the pattern: `XXYY-ZZZZZ-SS`

| Prefix | Meaning |
|--------|---------|
| First letter | Decade (D=2010s, E=2020s, F=2010s, G=2010s, H=2017+) |
| Second letter | Vehicle line (L3=F-150, U5=Explorer, etc.) |
| Third+Fourth | Engineering responsible (T=Transmission, A=Engine, etc.) |
| Middle digits | Base part number (functional group) |
| Suffix | Revision level |

**Common functional group prefixes:**
- `12A650` / `12B565` = Powertrain / Transmission controllers
- `14B321` = Body control
- `14G371` = Infotainment/APIM
- `3F944` / `3F964` = Braking / Steering
- `10849` = Instrument cluster
- `19C107` / `19H517` = Camera modules
- `604A17` = Telematics

---

## Additional Profile Metadata

| Key | Value |
|-----|-------|
| `PCMID` | 2df7 |
| `INFLAGS` | 1847; 2df6; 2df7 |
| `CHECKSUM` | AEB705C4 |
| `DB_VERSION` | 4 |
| `VEHID` | e7a28566 |
| `ADQS` | 11080113; 11390118 |

> **INFLAGS** likely indicates modules that require initialization/flashing procedures: APIM (1847), TCM (2df6), PCM (2df7).

---

## Notes

- FORScan module IDs are **not** standard 11-bit CAN IDs. They are FORScan's internal identifiers that map to Ford's diagnostic addressing scheme.
- The UDS Request/Response ID pairs listed above are approximate standard mappings. Actual CAN arbitration IDs may vary — use FORScan's live scan to confirm.
- Bus code `0x130` appears to indicate Ford-proprietary extended diagnostics over HS-CAN, while `0x20` is standard OBD-II.
- The `PIDS*` lines contain hashed/encrypted PID definitions specific to FORScan's database — not directly human-readable.
- DDM (`2DC4`) and PDM (`389D`) share the same base part number family (`DG9T-14B53x`) — they're mirror modules for driver/passenger doors.
