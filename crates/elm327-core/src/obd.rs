//! OBD-II protocol layer — decoding standard OBD-II data (PIDs, DTCs, VIN).
//!
//! # Usage
//! ```
//! use elm327_core::obd::*;
//!
//! // Build a command string
//! let cmd = obd_command(0x01, 0x0C); // "010C" for RPM
//!
//! // Parse a hex response from the ELM327
//! let bytes = parse_hex_response("7E8 04 41 0C 1A F8");
//! // bytes = [0x41, 0x0C, 0x1A, 0xF8]
//!
//! // Decode a DTC
//! let dtc = decode_dtc(0x03, 0x00);
//! assert_eq!(dtc.code, "P0300");
//! ```

/// Standard OBD-II PID definition
#[derive(Debug, Clone)]
pub struct PidDef {
    pub mode: u8,
    pub pid: u8,
    pub name: &'static str,
    pub unit: &'static str,
    pub min: f64,
    pub max: f64,
    /// Decode raw bytes into a value
    pub decode: fn(&[u8]) -> f64,
}

/// A decoded PID value
#[derive(Debug, Clone)]
pub struct PidValue {
    pub pid: u8,
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub raw: Vec<u8>,
}

/// A decoded DTC (Diagnostic Trouble Code)
#[derive(Debug, Clone, PartialEq)]
pub struct Dtc {
    pub code: String,
    pub category: DtcCategory,
}

/// DTC category derived from the first two bits
#[derive(Debug, Clone, PartialEq)]
pub enum DtcCategory {
    Powertrain, // P — bits 00
    Chassis,    // C — bits 01
    Body,       // B — bits 10
    Network,    // U — bits 11
}

// ---------------------------------------------------------------------------
// PID definitions table
// ---------------------------------------------------------------------------

/// Common Mode 01 PID definitions.
///
/// TODO: expand with Mode 02 (freeze frame) and Mode 09 (vehicle info) PIDs
pub static PIDS: &[PidDef] = &[
    PidDef {
        mode: 0x01,
        pid: 0x00,
        name: "Supported PIDs 01-20",
        unit: "bitmap",
        min: 0.0,
        max: 4_294_967_295.0,
        decode: |d| {
            // 4 bytes → u32 bitmap
            let a = *d.first().unwrap_or(&0) as f64;
            let b = *d.get(1).unwrap_or(&0) as f64;
            let c = *d.get(2).unwrap_or(&0) as f64;
            let dd = *d.get(3).unwrap_or(&0) as f64;
            a * 16_777_216.0 + b * 65_536.0 + c * 256.0 + dd
        },
    },
    PidDef {
        mode: 0x01,
        pid: 0x04,
        name: "Engine Load",
        unit: "%",
        min: 0.0,
        max: 100.0,
        decode: |d| *d.first().unwrap_or(&0) as f64 * 100.0 / 255.0,
    },
    PidDef {
        mode: 0x01,
        pid: 0x05,
        name: "Coolant Temp",
        unit: "°C",
        min: -40.0,
        max: 215.0,
        decode: |d| *d.first().unwrap_or(&0) as f64 - 40.0,
    },
    PidDef {
        mode: 0x01,
        pid: 0x0B,
        name: "Intake Manifold Pressure",
        unit: "kPa",
        min: 0.0,
        max: 255.0,
        decode: |d| *d.first().unwrap_or(&0) as f64,
    },
    PidDef {
        mode: 0x01,
        pid: 0x0C,
        name: "Engine RPM",
        unit: "rpm",
        min: 0.0,
        max: 16383.75,
        decode: |d| {
            let a = *d.first().unwrap_or(&0) as f64;
            let b = *d.get(1).unwrap_or(&0) as f64;
            (a * 256.0 + b) / 4.0
        },
    },
    PidDef {
        mode: 0x01,
        pid: 0x0D,
        name: "Vehicle Speed",
        unit: "km/h",
        min: 0.0,
        max: 255.0,
        decode: |d| *d.first().unwrap_or(&0) as f64,
    },
    PidDef {
        mode: 0x01,
        pid: 0x0F,
        name: "Intake Air Temp",
        unit: "°C",
        min: -40.0,
        max: 215.0,
        decode: |d| *d.first().unwrap_or(&0) as f64 - 40.0,
    },
    PidDef {
        mode: 0x01,
        pid: 0x10,
        name: "MAF Rate",
        unit: "g/s",
        min: 0.0,
        max: 655.35,
        decode: |d| {
            let a = *d.first().unwrap_or(&0) as f64;
            let b = *d.get(1).unwrap_or(&0) as f64;
            (a * 256.0 + b) / 100.0
        },
    },
    PidDef {
        mode: 0x01,
        pid: 0x11,
        name: "Throttle Position",
        unit: "%",
        min: 0.0,
        max: 100.0,
        decode: |d| *d.first().unwrap_or(&0) as f64 * 100.0 / 255.0,
    },
    PidDef {
        mode: 0x01,
        pid: 0x1F,
        name: "Run Time",
        unit: "sec",
        min: 0.0,
        max: 65535.0,
        decode: |d| {
            let a = *d.first().unwrap_or(&0) as f64;
            let b = *d.get(1).unwrap_or(&0) as f64;
            a * 256.0 + b
        },
    },
    PidDef {
        mode: 0x01,
        pid: 0x2F,
        name: "Fuel Level",
        unit: "%",
        min: 0.0,
        max: 100.0,
        decode: |d| *d.first().unwrap_or(&0) as f64 * 100.0 / 255.0,
    },
    PidDef {
        mode: 0x01,
        pid: 0x42,
        name: "Control Module Voltage",
        unit: "V",
        min: 0.0,
        max: 65.535,
        decode: |d| {
            let a = *d.first().unwrap_or(&0) as f64;
            let b = *d.get(1).unwrap_or(&0) as f64;
            (a * 256.0 + b) / 1000.0
        },
    },
    PidDef {
        mode: 0x01,
        pid: 0x46,
        name: "Ambient Air Temp",
        unit: "°C",
        min: -40.0,
        max: 215.0,
        decode: |d| *d.first().unwrap_or(&0) as f64 - 40.0,
    },
    PidDef {
        mode: 0x01,
        pid: 0x5C,
        name: "Oil Temp",
        unit: "°C",
        min: -40.0,
        max: 215.0,
        decode: |d| *d.first().unwrap_or(&0) as f64 - 40.0,
    },
];

/// Look up a PID definition by mode and pid number.
pub fn lookup_pid(mode: u8, pid: u8) -> Option<&'static PidDef> {
    PIDS.iter().find(|p| p.mode == mode && p.pid == pid)
}

/// Alias for `lookup_pid` — find a PID definition by mode and pid number.
pub fn find_pid(mode: u8, pid: u8) -> Option<&'static PidDef> {
    lookup_pid(mode, pid)
}

// ---------------------------------------------------------------------------
// DTC decoding
// ---------------------------------------------------------------------------

/// Decode a single DTC from two raw bytes.
///
/// Two bytes encode a DTC as follows:
/// - Bits 15-14 → category (00=P, 01=C, 10=B, 11=U)
/// - Bits 13-12 → second character (0-3)
/// - Bits 11-8  → third character (0-F)
/// - Bits 7-4   → fourth character (0-F)
/// - Bits 3-0   → fifth character (0-F)
///
/// Example: bytes `0x01, 0x00` → `P0100`
pub fn decode_dtc(byte1: u8, byte2: u8) -> Dtc {
    let category_bits = (byte1 >> 6) & 0x03;
    let (category, prefix) = match category_bits {
        0b00 => (DtcCategory::Powertrain, 'P'),
        0b01 => (DtcCategory::Chassis, 'C'),
        0b10 => (DtcCategory::Body, 'B'),
        _ => (DtcCategory::Network, 'U'),
    };

    let second = (byte1 >> 4) & 0x03;
    let third = byte1 & 0x0F;
    let fourth = (byte2 >> 4) & 0x0F;
    let fifth = byte2 & 0x0F;

    let code = format!(
        "{}{}{:X}{:X}{:X}",
        prefix, second, third, fourth, fifth
    );

    Dtc { code, category }
}

/// Decode multiple DTCs from a Mode 03 response payload.
///
/// The `data` slice should contain the raw data bytes *after* stripping the
/// service ID byte (0x43). Each DTC is 2 bytes, so the slice length should
/// be even. Zero-valued pairs (0x00, 0x00) are padding and skipped.
pub fn decode_dtcs(data: &[u8]) -> Vec<Dtc> {
    let mut dtcs = Vec::new();
    for pair in data.chunks_exact(2) {
        // Skip padding (0x0000 means "no DTC")
        if pair[0] == 0 && pair[1] == 0 {
            continue;
        }
        dtcs.push(decode_dtc(pair[0], pair[1]));
    }
    dtcs
}

// ---------------------------------------------------------------------------
// Hex response parsing
// ---------------------------------------------------------------------------

/// Parse a hex string response from the ELM327 into raw bytes.
///
/// Handles responses with or without spaces, and strips the 3-character CAN
/// header (e.g. `7E8`) plus the data-length byte when present.
///
/// # Examples
/// ```
/// use elm327_core::obd::parse_hex_response;
///
/// // With CAN header + spaces
/// let bytes = parse_hex_response("7E8 06 41 00 BE 3E B8 13");
/// assert_eq!(bytes, vec![0x41, 0x00, 0xBE, 0x3E, 0xB8, 0x13]);
///
/// // Without spaces, with CAN header
/// let bytes = parse_hex_response("7E804410C1AF8");
/// assert_eq!(bytes, vec![0x41, 0x0C, 0x1A, 0xF8]);
/// ```
pub fn parse_hex_response(resp: &str) -> Vec<u8> {
    // Strip whitespace and collapse into a single hex string
    let hex: String = resp.chars().filter(|c| !c.is_whitespace()).collect();

    // Parse all nibble pairs into bytes
    let all_bytes: Vec<u8> = (0..hex.len() / 2)
        .filter_map(|i| {
            let pair = &hex[i * 2..i * 2 + 2];
            match u8::from_str_radix(pair, 16) {
                Ok(b) => Some(b),
                Err(_) => {
                    log::warn!("Invalid hex pair in response: {:?}", pair);
                    None
                }
            }
        })
        .collect();

    if all_bytes.is_empty() {
        return Vec::new();
    }

    // Detect CAN header: first byte typically 0x7E0-0x7EF range, or generally
    // the first 3 hex chars form a CAN ID (e.g. "7E8"). We check if the first
    // byte's high nibble is 0x7 and the second nibble is 0xE — covering the
    // standard 7E8/7E0 CAN response IDs.
    //
    // When a CAN header is present the layout is:
    //   [CAN_ID_HI] [CAN_ID_LO (4 bits) + LEN_HI (4 bits)] [data...]
    // But since CAN IDs are 3 hex chars (1.5 bytes), the actual layout in the
    // hex string is: 3 hex chars for ID + 2 hex chars for length + data.
    // That means byte index 0 = first 2 chars of ID, byte index 1 = last char
    // of ID + first char of length, byte index 2 = second char of length... nah.
    //
    // Actually: the hex string "7E80641..." breaks down as:
    //   "7E8" = CAN ID (3 hex chars)
    //   "06"  = data length byte (2 hex chars)
    //   "41..." = payload
    //
    // So we need to handle the 3-char CAN ID by looking at the raw hex string.
    let has_can_header = hex.len() >= 6 && {
        // Check if first 3 hex chars look like a CAN ID (7Ex or similar)
        let id_str = &hex[0..3];
        // Common CAN response IDs: 7E0-7EF, 7E8 is most common
        u16::from_str_radix(id_str, 16)
            .map(|id| (0x700..=0x7FF).contains(&id))
            .unwrap_or(false)
    };

    if has_can_header {
        // Skip 3 hex chars (CAN ID) + 2 hex chars (length byte) = 5 hex chars
        // That's bytes starting at hex index 5
        let data_hex = &hex[5..];
        (0..data_hex.len() / 2)
            .filter_map(|i| {
                let pair = &data_hex[i * 2..i * 2 + 2];
                match u8::from_str_radix(pair, 16) {
                    Ok(b) => Some(b),
                    Err(_) => {
                        log::warn!("Invalid hex pair in response: {:?}", pair);
                        None
                    }
                }
            })
            .collect()
    } else {
        all_bytes
    }
}

// ---------------------------------------------------------------------------
// VIN decoding
// ---------------------------------------------------------------------------

/// Decode VIN from Mode 09, PID 02 multi-frame response.
///
/// The `frames` parameter should contain the data bytes from each ISO-TP
/// frame (with CAN headers and PCI bytes already stripped). The VIN is 17
/// ASCII characters. Leading padding bytes (e.g. 0x01 count byte, 0x00
/// padding in first frame) are skipped.
///
/// # Example
/// ```
/// use elm327_core::obd::decode_vin;
///
/// // Simulated multi-frame data (already stripped of CAN ID + PCI)
/// let frame1 = vec![0x49, 0x02, 0x01, 0x31, 0x46, 0x41]; // I..\x01 "1FA"
/// let frame2 = vec![0x44, 0x50, 0x33, 0x46, 0x32, 0x38, 0x55]; // "DP3F28U"
/// let frame3 = vec![0x35, 0x33, 0x32, 0x31, 0x30, 0x30, 0x30]; // "5321000"
/// let vin = decode_vin(&[frame1, frame2, frame3]);
/// assert_eq!(vin, Some("1FADP3F28U5321000".to_string()));
/// ```
pub fn decode_vin(frames: &[Vec<u8>]) -> Option<String> {
    // Concatenate all frame data
    let mut all_bytes: Vec<u8> = Vec::new();
    for frame in frames {
        all_bytes.extend_from_slice(frame);
    }

    // Find the start of the VIN data. The response to Mode 09 PID 02 starts
    // with service byte 0x49, pid 0x02, then a count byte (usually 0x01),
    // followed by 17 ASCII bytes of VIN.
    let vin_start = all_bytes
        .windows(2)
        .position(|w| w[0] == 0x49 && w[1] == 0x02)?;

    // Skip: 0x49, 0x02, count byte (0x01) = 3 bytes after vin_start
    let data_start = vin_start + 3;
    if all_bytes.len() < data_start + 17 {
        return None;
    }

    let vin_bytes = &all_bytes[data_start..data_start + 17];

    // Verify all bytes are printable ASCII
    if vin_bytes.iter().all(|&b| b.is_ascii_alphanumeric()) {
        Some(String::from_utf8_lossy(vin_bytes).to_string())
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Command formatting
// ---------------------------------------------------------------------------

/// Format a mode-only OBD command (e.g., Mode 03 for DTCs).
///
/// Some OBD modes (03 = stored DTCs, 04 = clear DTCs) don't take a PID —
/// they're single-byte commands.
///
/// # Example
/// ```
/// use elm327_core::obd::obd_mode_command;
/// assert_eq!(obd_mode_command(0x03), "03");
/// assert_eq!(obd_mode_command(0x04), "04");
/// ```
pub fn obd_mode_command(mode: u8) -> String {
    format!("{:02X}", mode)
}

/// Format a mode+PID OBD command (e.g., Mode 01 PID 0C for RPM).
///
/// # Example
/// ```
/// use elm327_core::obd::obd_command;
/// assert_eq!(obd_command(0x01, 0x0C), "010C");
/// assert_eq!(obd_command(0x01, 0x00), "0100");
/// ```
pub fn obd_command(mode: u8, pid: u8) -> String {
    format!("{:02X}{:02X}", mode, pid)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_dtc_p0300() {
        let dtc = decode_dtc(0x03, 0x00);
        assert_eq!(dtc.code, "P0300");
        assert_eq!(dtc.category, DtcCategory::Powertrain);
    }

    #[test]
    fn test_decode_dtc_c0100() {
        // C0100: category=01 (Chassis), second=0, third=1, fourth=0, fifth=0
        // byte1: 01_00_0001 = 0x41, byte2: 0000_0000 = 0x00
        let dtc = decode_dtc(0x41, 0x00);
        assert_eq!(dtc.code, "C0100");
        assert_eq!(dtc.category, DtcCategory::Chassis);
    }

    #[test]
    fn test_decode_dtc_b0001() {
        // B0001: category=10 (Body), second=0, third=0, fourth=0, fifth=1
        // byte1: 10_00_0000 = 0x80, byte2: 0000_0001 = 0x01
        let dtc = decode_dtc(0x80, 0x01);
        assert_eq!(dtc.code, "B0001");
        assert_eq!(dtc.category, DtcCategory::Body);
    }

    #[test]
    fn test_decode_dtc_u0100() {
        // U0100: category=11 (Network), second=0, third=1, fourth=0, fifth=0
        // byte1: 11_00_0001 = 0xC1, byte2: 0000_0000 = 0x00
        let dtc = decode_dtc(0xC1, 0x00);
        assert_eq!(dtc.code, "U0100");
        assert_eq!(dtc.category, DtcCategory::Network);
    }

    #[test]
    fn test_decode_dtcs_with_padding() {
        let data = vec![0x03, 0x00, 0x00, 0x00, 0xC1, 0x00];
        let dtcs = decode_dtcs(&data);
        assert_eq!(dtcs.len(), 2);
        assert_eq!(dtcs[0].code, "P0300");
        assert_eq!(dtcs[1].code, "U0100");
    }

    #[test]
    fn test_decode_rpm() {
        let pid = lookup_pid(0x01, 0x0C).expect("RPM PID should exist");
        // (0x1A * 256 + 0xF8) / 4 = (26*256 + 248) / 4 = 6904 / 4 = 1726
        let rpm = (pid.decode)(&[0x1A, 0xF8]);
        assert!((rpm - 1726.0).abs() < 0.01);
    }

    #[test]
    fn test_decode_rpm_idle() {
        let pid = lookup_pid(0x01, 0x0C).unwrap();
        // ~800 RPM: 800 * 4 = 3200 = 0x0C80 → A=0x0C, B=0x80
        let rpm = (pid.decode)(&[0x0C, 0x80]);
        assert!((rpm - 800.0).abs() < 0.01);
    }

    #[test]
    fn test_decode_coolant_temp() {
        let pid = lookup_pid(0x01, 0x05).expect("Coolant Temp PID should exist");
        // A=130 → 130-40 = 90°C (normal operating temp)
        let temp = (pid.decode)(&[130]);
        assert!((temp - 90.0).abs() < 0.01);
    }

    #[test]
    fn test_decode_coolant_temp_cold() {
        let pid = lookup_pid(0x01, 0x05).unwrap();
        // A=40 → 40-40 = 0°C
        assert!(((pid.decode)(&[40]) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_decode_coolant_temp_below_zero() {
        let pid = lookup_pid(0x01, 0x05).unwrap();
        // A=0 → 0-40 = -40°C
        assert!(((pid.decode)(&[0]) - (-40.0)).abs() < 0.01);
    }

    #[test]
    fn test_parse_hex_with_spaces_and_header() {
        let bytes = parse_hex_response("7E8 06 41 00 BE 3E B8 13");
        assert_eq!(bytes, vec![0x41, 0x00, 0xBE, 0x3E, 0xB8, 0x13]);
    }

    #[test]
    fn test_parse_hex_no_spaces_with_header() {
        // "7E8" + "04" + "41" "0C" "1A" "F8"
        let bytes = parse_hex_response("7E804410C1AF8");
        assert_eq!(bytes, vec![0x41, 0x0C, 0x1A, 0xF8]);
    }

    #[test]
    fn test_parse_hex_no_header() {
        // Raw response without CAN header (e.g. headers off)
        let bytes = parse_hex_response("41 0C 1A F8");
        assert_eq!(bytes, vec![0x41, 0x0C, 0x1A, 0xF8]);
    }

    #[test]
    fn test_decode_vin() {
        // Simulated multi-frame VIN response (stripped of CAN ID + PCI bytes)
        let frame1 = vec![0x49, 0x02, 0x01, 0x31, 0x46, 0x41]; // 0x49=service, 0x02=pid, 0x01=count, "1FA"
        let frame2 = vec![0x44, 0x50, 0x33, 0x46, 0x32, 0x38, 0x55]; // "DP3F28U"
        let frame3 = vec![0x35, 0x33, 0x32, 0x31, 0x30, 0x30, 0x30]; // "5321000"

        let vin = decode_vin(&[frame1, frame2, frame3]);
        assert_eq!(vin, Some("1FADP3F28U5321000".to_string()));
    }

    #[test]
    fn test_decode_vin_too_short() {
        let frame = vec![0x49, 0x02, 0x01, 0x31, 0x46];
        assert_eq!(decode_vin(&[frame]), None);
    }

    #[test]
    fn test_obd_command() {
        assert_eq!(obd_command(0x01, 0x0C), "010C");
        assert_eq!(obd_command(0x01, 0x00), "0100");
        assert_eq!(obd_command(0x09, 0x02), "0902");
    }

    #[test]
    fn test_obd_mode_command() {
        assert_eq!(obd_mode_command(0x03), "03");
        assert_eq!(obd_mode_command(0x04), "04");
    }

    #[test]
    fn test_obd_command_formatting() {
        // Ensure single-digit modes/pids get zero-padded
        assert_eq!(obd_command(0x01, 0x04), "0104");
        assert_eq!(obd_command(0x01, 0x05), "0105");
    }

    #[test]
    fn test_lookup_pid_exists() {
        assert!(lookup_pid(0x01, 0x0C).is_some());
        assert!(lookup_pid(0x01, 0x05).is_some());
        assert!(lookup_pid(0x01, 0x42).is_some());
    }

    #[test]
    fn test_lookup_pid_not_found() {
        assert!(lookup_pid(0x01, 0xFF).is_none());
        assert!(lookup_pid(0x02, 0x0C).is_none());
    }

    #[test]
    fn test_decode_throttle_position() {
        let pid = lookup_pid(0x01, 0x11).unwrap();
        // A=255 → 100%
        assert!(((pid.decode)(&[255]) - 100.0).abs() < 0.01);
        // A=0 → 0%
        assert!(((pid.decode)(&[0]) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_decode_control_module_voltage() {
        let pid = lookup_pid(0x01, 0x42).unwrap();
        // 14.5V: 14500 = 0x38A4 → A=0x38, B=0xA4
        let v = (pid.decode)(&[0x38, 0xA4]);
        assert!((v - 14.5).abs() < 0.01);
    }

    // ── 2017 F-150 3.5L EcoBoost ignition coil DTC tests ─────────────────

    #[test]
    fn test_decode_dtc_ignition_coil_p0351() {
        let dtc = decode_dtc(0x03, 0x51);
        assert_eq!(dtc.code, "P0351");
        assert_eq!(dtc.category, DtcCategory::Powertrain);
    }

    #[test]
    fn test_decode_dtc_ignition_coil_all_cylinders() {
        // 3.5L EcoBoost has 6 cylinders = coils A-F (P0351-P0356)
        let expected = vec!["P0351", "P0352", "P0353", "P0354", "P0355", "P0356"];
        for (i, code) in expected.iter().enumerate() {
            let pid = 0x51 + i as u8;
            let dtc = decode_dtc(0x03, pid);
            assert_eq!(dtc.code, *code, "Coil {} DTC mismatch", i + 1);
        }
    }

    #[test]
    fn test_decode_dtc_misfire_codes() {
        // Random misfire
        let dtc = decode_dtc(0x03, 0x00);
        assert_eq!(dtc.code, "P0300");

        // Per-cylinder misfires (6 cylinders for EcoBoost V6)
        for cyl in 1..=6u8 {
            let dtc = decode_dtc(0x03, cyl);
            assert_eq!(dtc.code, format!("P030{}", cyl));
        }
    }

    #[test]
    fn test_decode_dtc_startup_misfire() {
        // P0316 — misfire detected on startup (first 1000 revolutions)
        let dtc = decode_dtc(0x03, 0x16);
        assert_eq!(dtc.code, "P0316");
    }

    // ── EcoBoost PID decoding tests ───────────────────────────────────────

    #[test]
    fn test_decode_rpm_ecoboost_idle() {
        // Typical EcoBoost idle: ~680 RPM
        // Formula: (A*256 + B) / 4
        // 680 RPM = 2720 raw = 0x0AA0
        let pid = find_pid(0x01, 0x0C).unwrap();
        assert_eq!((pid.decode)(&[0x0A, 0xA0]), 680.0);
    }

    #[test]
    fn test_decode_rpm_cruising() {
        // 2000 RPM = 8000 raw = 0x1F40
        let pid = find_pid(0x01, 0x0C).unwrap();
        assert_eq!((pid.decode)(&[0x1F, 0x40]), 2000.0);
    }

    #[test]
    fn test_decode_coolant_temp_operating() {
        // Normal operating temp: 90°C, raw = 130 (90 + 40)
        let pid = find_pid(0x01, 0x05).unwrap();
        assert_eq!((pid.decode)(&[130]), 90.0);
    }

    #[test]
    fn test_decode_coolant_temp_cold_start() {
        // Cold start: 20°C, raw = 60 (20 + 40)
        let pid = find_pid(0x01, 0x05).unwrap();
        assert_eq!((pid.decode)(&[60]), 20.0);
    }

    #[test]
    fn test_decode_engine_load_full() {
        // Full load: 100%, raw = 255
        let pid = find_pid(0x01, 0x04).unwrap();
        let val = (pid.decode)(&[255]);
        assert!((val - 100.0).abs() < 0.5);
    }

    #[test]
    fn test_decode_vehicle_speed_highway() {
        // 70 mph ~ 113 km/h
        let pid = find_pid(0x01, 0x0D).unwrap();
        assert_eq!((pid.decode)(&[113]), 113.0);
    }

    // ── Hex response parsing — real ELM327 formats ────────────────────────

    #[test]
    fn test_parse_response_with_header_no_spaces_supported_pids() {
        // Headers on, spaces off: "7E8064100BE3EB813"
        // 7E8 = CAN ID (3 hex chars), 06 = length (2 hex chars),
        // 4100BE3EB813 = data (12 hex chars = 6 bytes)
        let bytes = parse_hex_response("7E8064100BE3EB813");
        assert_eq!(bytes, vec![0x41, 0x00, 0xBE, 0x3E, 0xB8, 0x13]);
    }

    #[test]
    fn test_parse_response_no_header_with_spaces() {
        // Headers off: "41 0C 0A A0"
        let bytes = parse_hex_response("41 0C 0A A0");
        assert_eq!(bytes, vec![0x41, 0x0C, 0x0A, 0xA0]);
    }

    #[test]
    fn test_parse_dtc_response() {
        // Mode 03 response: "43 01 03 00 00 00 00"
        // 43 = response to mode 03, then DTC data
        let bytes = parse_hex_response("43 01 03 00 00 00 00");
        assert_eq!(bytes[0], 0x43); // Mode 03 response
        let dtc = decode_dtc(bytes[2], bytes[3]);
        assert_eq!(dtc.code, "P0300");
    }

    #[test]
    fn test_parse_multiple_dtcs() {
        // Multiple DTCs: "43 02 03 01 03 51 00 00"
        // 43 = response, 02 = count
        // 03 01 = P0301 (cylinder 1 misfire)
        // 03 51 = P0351 (ignition coil A)
        let bytes = parse_hex_response("43 02 03 01 03 51 00 00");
        let dtcs = decode_dtcs(&bytes[2..]); // skip 43 and count
        assert_eq!(dtcs.len(), 2);
        assert_eq!(dtcs[0].code, "P0301");
        assert_eq!(dtcs[1].code, "P0351");
    }

    // ── VIN decoding for F-150 ────────────────────────────────────────────

    #[test]
    fn test_decode_vin_f150() {
        // VIN: 1FTEW1EGXHKC84222 (2017 F-150 format)
        // Mode 09 PID 02 returns VIN as ASCII in multi-frame
        // First frame has 0x49 0x02 0x01 prefix, then 17 ASCII bytes
        let vin_str = "1FTEW1EGXHKC84222";
        let vin_bytes: Vec<u8> = vin_str.bytes().collect();
        // Build frames: service byte + pid + count + VIN data
        let mut frame1 = vec![0x49, 0x02, 0x01];
        frame1.extend_from_slice(&vin_bytes[..4]); // "1FTE"
        let frame2 = vin_bytes[4..11].to_vec();     // "W1EGXHK"
        let frame3 = vin_bytes[11..].to_vec();       // "C84222"
        // Pad frame3 to make VIN extraction work (need 17 bytes total after header)
        let vin = decode_vin(&[frame1, frame2, frame3]);
        assert_eq!(vin, Some("1FTEW1EGXHKC84222".to_string()));
    }

    // ── find_pid alias works ──────────────────────────────────────────────

    #[test]
    fn test_find_pid_alias() {
        assert!(find_pid(0x01, 0x0C).is_some());
        assert!(find_pid(0x01, 0xFF).is_none());
        // Should return same result as lookup_pid
        let a = find_pid(0x01, 0x05).unwrap();
        let b = lookup_pid(0x01, 0x05).unwrap();
        assert_eq!(a.name, b.name);
    }
}
