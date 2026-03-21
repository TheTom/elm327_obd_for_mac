# Ford F-150 Diagnostic Reference

Reference data for building diagnostic features targeting the Ford F-150, primarily the 3.5L EcoBoost V6.

## Cylinder Layout & Firing Order

**Firing Order:** 1-4-2-5-3-6

| Side       | Cylinders |
|------------|-----------|
| Passenger  | 1, 2, 3   |
| Driver     | 4, 5, 6   |

## Ignition Coil DTC Mapping

| DTC   | Coil | Cylinder |
|-------|------|----------|
| P0351 | A    | 1        |
| P0352 | B    | 2        |
| P0353 | C    | 3        |
| P0354 | D    | 4        |
| P0355 | E    | 5        |
| P0356 | F    | 6        |

## Misfire DTCs

| DTC   | Description                                    |
|-------|------------------------------------------------|
| P0300 | Random/multiple cylinder misfire detected       |
| P0301 | Cylinder 1 misfire detected                     |
| P0302 | Cylinder 2 misfire detected                     |
| P0303 | Cylinder 3 misfire detected                     |
| P0304 | Cylinder 4 misfire detected                     |
| P0305 | Cylinder 5 misfire detected                     |
| P0306 | Cylinder 6 misfire detected                     |
| P0316 | Misfire detected on startup (first 1000 revs)   |

## EcoBoost-Specific DTCs

| DTC   | Description                          |
|-------|--------------------------------------|
| P0299 | Turbocharger underboost condition    |
| P00B7 | Engine coolant flow low/performance  |
| P0234 | Turbocharger overboost condition     |

## CRITICAL WARNING: PCM Coil Driver Damage

**Affected Vehicles:** 2015-2017 F-150 3.5L EcoBoost

On these model years, repeatedly commanding ignition coil outputs or running certain actuator tests through the PCM can permanently damage the internal coil driver circuits. **Do NOT continuously cycle coil outputs via diagnostic commands.** This is a known hardware vulnerability in the PCM. Replacement PCMs are expensive and require As-Built programming.

## Ford Mode 22 PIDs (UDS ReadDataByIdentifier)

These are manufacturer-specific extended PIDs accessed via UDS service 0x22 with 2-byte DIDs. They are **not** standard OBD-II.

### PCM PIDs

| DID    | Name                    | Unit  | Notes                          |
|--------|-------------------------|-------|--------------------------------|
| 0x033E | Desired Boost Pressure  | PSI   | Turbo target boost             |
| 0x0462 | Wastegate Duty Cycle    | %     | Turbo wastegate actuator       |
| 0x0461 | Intercooler Temp        | °F    | Charge air cooler temperature  |
| 0x03EC | Knock Retard            | °     | Timing pulled for knock        |
| 0x03E8 | Learned Octane Ratio    | ratio | Fuel quality adaptation        |
| 0xF40F | IAT2 (Post-Intercooler) | °F    | Intake air temp after IC       |
| 0xD137 | DTC Count               | count | Number of stored DTCs          |

### BCM PIDs (Tire Pressure)

| DID    | Name              | Unit |
|--------|-------------------|------|
| 0x2813 | LF Tire Pressure  | PSI  |
| 0x2814 | RF Tire Pressure  | PSI  |
| 0x2815 | RR Tire Pressure  | PSI  |
| 0x2816 | LR Tire Pressure  | PSI  |

### TCM PIDs

| DID    | Name             | Unit |
|--------|------------------|------|
| 0x1E12 | Current Gear     | gear |
| 0x1E1C | Trans Fluid Temp | °F   |

## Mode 06 Misfire Monitoring

Misfire counts per cylinder via Mode 06:

- **Test ID (TID):** `$A6`
- **Component IDs (CIDs):**

| CID  | Cylinder |
|------|----------|
| $01  | 1        |
| $02  | 2        |
| $03  | 3        |
| $04  | 4        |
| $05  | 5        |
| $06  | 6        |

## Mode 02 Freeze Frame

Mode 02 returns freeze frame data captured at the time the first emission-related DTC was set. The format mirrors Mode 01 PIDs but uses service ID `0x02` and includes a frame number byte. Use `0200` to request supported freeze frame PIDs, then `02XX00` for each PID (where XX is the PID number and the trailing `00` is frame number).

## As-Built DID Range

Ford As-Built configuration data is stored in the DID range:

- **Range:** `0xDE00` - `0xDEFF`
- Access via UDS ReadDataByIdentifier (service 0x22)
- Used for module configuration, feature enable/disable, calibration settings
- **Caution:** Writing incorrect As-Built data can disable vehicle functions

## UDS Commands for Extended Diagnostics

| Command        | Description                                  |
|----------------|----------------------------------------------|
| `10 01`        | DiagnosticSessionControl — Default session   |
| `10 03`        | DiagnosticSessionControl — Extended session  |
| `19 02 FF`     | ReadDTCInformation — All DTCs by status mask |
| `22 F190`      | ReadDataByIdentifier — VIN                   |
| `22 F111`      | ReadDataByIdentifier — ECU Hardware Number   |
| `22 F188`      | ReadDataByIdentifier — ECU Software Number   |
| `14 FF FF FF`  | ClearDiagnosticInformation — Clear all DTCs  |
| `27 01`        | SecurityAccess — Request seed               |
| `27 02 XX...`  | SecurityAccess — Send key                   |
| `31 01 FF 00`  | RoutineControl — Start routine              |
| `3E 00`        | TesterPresent — Keep session alive           |
