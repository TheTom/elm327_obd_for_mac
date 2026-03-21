/// ELM327 v1.5 AT command state machine simulator.
///
/// Responds realistically to the FORScan initialization sequence and common
/// AT commands. Useful for integration testing the bridge without real hardware.
///
/// # Example
/// ```no_run
/// use elm327_simulator::elm327_sim::Elm327Simulator;
/// let mut sim = Elm327Simulator::new();
/// let resp = sim.process_command("ATZ");
/// assert!(String::from_utf8_lossy(&resp).contains("ELM327 v1.5"));
/// ```
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Fake programmable parameter table matching a PIC18F25K80 clone.
const PPS_TABLE: &str = "\
00:FF F  01:00 F  02:FF F  03:32 F\r\
04:01 F  05:01 F  06:F1 F  07:09 F\r\
08:FF F  09:00 F  0A:0A F  0B:FF F\r\
0C:00 F  0D:0D F  0E:9A F  0F:FF F\r\
10:0D F  11:00 F  12:FF F  13:F4 F\r\
14:FF F  15:FF F  16:FF F  17:92 F\r\
18:00 F  19:28 F  1A:FF F  1B:FF F\r\
1C:FF F  1D:FF F  1E:FF F  1F:FF F\r\
20:FF F  21:FF F  22:FF F  23:FF F\r\
24:00 F  25:00 F  26:00 F  27:FF F\r\
28:FF F  29:FF F  2A:00 N";

/// Protocol names indexed by protocol number (0-C hex).
const PROTOCOL_NAMES: &[&str] = &[
    "AUTO",                     // 0
    "SAE J1850 PWM",            // 1
    "SAE J1850 VPW",            // 2
    "ISO 9141-2",               // 3
    "ISO 14230-4 (KWP 5BAUD)",  // 4
    "ISO 14230-4 (KWP FAST)",   // 5
    "ISO 15765-4 (CAN 11/500)", // 6
    "ISO 15765-4 (CAN 29/500)", // 7
    "ISO 15765-4 (CAN 11/250)", // 8
    "ISO 15765-4 (CAN 29/250)", // 9
    "SAE J1939 (CAN 29/250)",   // A
    "USER1 CAN (11/125)",       // B
    "USER2 CAN (11/50)",        // C
];

pub struct Elm327Simulator {
    echo: bool,
    linefeed: bool,
    spaces: bool,
    headers: bool,
    protocol: u8,
    adaptive_timing: u8,
    version: String,
    voltage: String,
    /// Injected DTCs for Mode 03 simulation. Each tuple is (byte1, byte2).
    dtcs: Vec<(u8, u8)>,
}

impl Elm327Simulator {
    pub fn new() -> Self {
        Self {
            echo: true,
            linefeed: true,
            spaces: true,
            headers: false,
            protocol: 0,
            adaptive_timing: 1,
            version: "ELM327 v1.5".to_string(),
            voltage: "12.6".to_string(),
            dtcs: Vec::new(),
        }
    }

    /// Inject a DTC for Mode 03 simulation. Each DTC is two raw bytes.
    ///
    /// # Example
    /// ```
    /// use elm327_simulator::elm327_sim::Elm327Simulator;
    /// let mut sim = Elm327Simulator::new();
    /// sim.add_dtc(0x03, 0x01); // P0301 — cylinder 1 misfire
    /// sim.add_dtc(0x03, 0x51); // P0351 — ignition coil A
    /// ```
    #[allow(dead_code)] // Used by integration tests via lib re-export
    pub fn add_dtc(&mut self, byte1: u8, byte2: u8) {
        self.dtcs.push((byte1, byte2));
    }

    /// Clear all injected DTCs (simulates Mode 04 clear).
    #[allow(dead_code)] // Used by integration tests via lib re-export
    pub fn clear_dtcs(&mut self) {
        self.dtcs.clear();
    }

    /// Reset to factory defaults (ATD command).
    fn reset_defaults(&mut self) {
        self.echo = true;
        self.linefeed = true;
        self.spaces = true;
        self.headers = false;
        self.protocol = 0;
        self.adaptive_timing = 1;
    }

    /// Process a single command string (without the trailing \r).
    /// Returns the response bytes to send back.
    pub fn process_command(&mut self, cmd: &str) -> Vec<u8> {
        let trimmed = cmd.trim();
        if trimmed.is_empty() {
            return self.format_response("");
        }

        let upper = trimmed.to_uppercase();

        // Check if it's an AT command
        if let Some(at_cmd) = upper.strip_prefix("AT") {
            self.process_at_command(at_cmd, trimmed)
        } else {
            // Treat as OBD PID request
            self.process_obd_command(&upper, trimmed)
        }
    }

    /// Process an AT command (with the "AT" prefix already stripped).
    fn process_at_command(&mut self, at_cmd: &str, original: &str) -> Vec<u8> {
        // ATZ — full reset with delay
        if at_cmd == "Z" {
            thread::sleep(Duration::from_millis(500));
            let mut resp = Vec::new();
            if self.echo {
                resp.extend_from_slice(original.as_bytes());
                resp.push(b'\r');
            }
            // Reset to defaults
            self.reset_defaults();
            let nl = if self.linefeed { "\r\n" } else { "\r" };
            resp.extend_from_slice(nl.as_bytes());
            resp.extend_from_slice(nl.as_bytes());
            resp.extend_from_slice(self.version.as_bytes());
            resp.extend_from_slice(nl.as_bytes());
            resp.extend_from_slice(nl.as_bytes());
            resp.push(b'>');
            return resp;
        }

        // ATI — version info
        if at_cmd == "I" {
            return self.build_response(original, &self.version.clone());
        }

        // ATE0/ATE1 — echo control
        if at_cmd == "E0" {
            let resp = self.build_response(original, "OK");
            self.echo = false;
            return resp;
        }
        if at_cmd == "E1" {
            self.echo = true;
            return self.build_response(original, "OK");
        }

        // ATL0/ATL1 — linefeed control
        if at_cmd == "L0" {
            self.linefeed = false;
            return self.build_response(original, "OK");
        }
        if at_cmd == "L1" {
            self.linefeed = true;
            return self.build_response(original, "OK");
        }

        // ATS0/ATS1 — spaces control
        if at_cmd == "S0" {
            self.spaces = false;
            return self.build_response(original, "OK");
        }
        if at_cmd == "S1" {
            self.spaces = true;
            return self.build_response(original, "OK");
        }

        // ATH0/ATH1 — headers control
        if at_cmd == "H0" {
            self.headers = false;
            return self.build_response(original, "OK");
        }
        if at_cmd == "H1" {
            self.headers = true;
            return self.build_response(original, "OK");
        }

        // ATAT0/ATAT1/ATAT2 — adaptive timing
        if at_cmd == "AT0" {
            self.adaptive_timing = 0;
            return self.build_response(original, "OK");
        }
        if at_cmd == "AT1" {
            self.adaptive_timing = 1;
            return self.build_response(original, "OK");
        }
        if at_cmd == "AT2" {
            self.adaptive_timing = 2;
            return self.build_response(original, "OK");
        }

        // ATSP <h> — set protocol
        if let Some(proto_str) = at_cmd.strip_prefix("SP") {
            let proto_str = proto_str.trim();
            if let Ok(p) = u8::from_str_radix(proto_str, 16) {
                self.protocol = p;
                return self.build_response(original, "OK");
            }
            return self.build_response(original, "?");
        }

        // ATDP — describe protocol
        if at_cmd == "DP" {
            let name = PROTOCOL_NAMES
                .get(self.protocol as usize)
                .unwrap_or(&"UNKNOWN");
            return self.build_response(original, name);
        }

        // ATDPN — describe protocol number
        if at_cmd == "DPN" {
            let num = format!("{:X}", self.protocol);
            return self.build_response(original, &num);
        }

        // ATRV — read voltage
        if at_cmd == "RV" {
            let v = format!("{}V", self.voltage);
            return self.build_response(original, &v);
        }

        // ATPPS — programmable parameter summary
        if at_cmd == "PPS" {
            return self.build_response(original, PPS_TABLE);
        }

        // ATCRA xxx — set CAN receive address filter
        if at_cmd.starts_with("CRA") {
            return self.build_response(original, "OK");
        }

        // ATCF xxx — set CAN ID filter
        if at_cmd.starts_with("CF") {
            return self.build_response(original, "OK");
        }

        // ATCM xxx — set CAN ID mask
        if at_cmd.starts_with("CM") {
            return self.build_response(original, "OK");
        }

        // ATPB xx yy — set protocol B parameters
        if at_cmd.starts_with("PB") {
            return self.build_response(original, "OK");
        }

        // ATD — set defaults
        if at_cmd == "D" {
            self.reset_defaults();
            return self.build_response(original, "OK");
        }

        // ATWS — warm start
        if at_cmd == "WS" {
            let version = self.version.clone();
            return self.build_response(original, &version);
        }

        // ATPC — protocol close
        if at_cmd == "PC" {
            return self.build_response(original, "OK");
        }

        // Unknown AT command
        self.build_response(original, "?")
    }

    /// Process a non-AT (OBD PID) command.
    fn process_obd_command(&mut self, upper: &str, original: &str) -> Vec<u8> {
        // Mode 03 — read stored DTCs
        if upper == "03" {
            return self.process_mode_03(original);
        }

        // Mode 04 — clear DTCs
        if upper == "04" {
            return self.process_mode_04(original);
        }

        // TODO: add more realistic PID responses as needed
        let response = match upper {
            "0100" => {
                if self.spaces {
                    "41 00 BE 3E B8 13"
                } else {
                    "4100BE3EB813"
                }
            }
            "010C" => {
                if self.spaces {
                    "41 0C 0B B8"
                } else {
                    "410C0BB8"
                }
            }
            "010D" => {
                if self.spaces {
                    "41 0D 00"
                } else {
                    "410D00"
                }
            }
            _ => "NO DATA",
        };
        self.build_response(original, response)
    }

    /// Handle Mode 03 — read stored DTCs.
    ///
    /// Returns "NO DATA" if no DTCs are injected, otherwise builds a proper
    /// Mode 03 response: "43 XX DD DD DD DD ..." where XX = count, DD DD = DTC bytes.
    fn process_mode_03(&self, original: &str) -> Vec<u8> {
        if self.dtcs.is_empty() {
            return self.build_response(original, "NO DATA");
        }

        // Build response: 43 (count) (dtc pairs...) padded to 6 DTC bytes (3 DTCs max per frame)
        let count = self.dtcs.len() as u8;
        let mut bytes = vec![0x43, count];
        for &(b1, b2) in &self.dtcs {
            bytes.push(b1);
            bytes.push(b2);
        }
        // Pad to 3 DTC pairs (6 bytes of DTC data) with 00 00
        while bytes.len() < 8 {
            bytes.push(0x00);
        }

        // Format as hex string
        let hex_str = if self.spaces {
            bytes
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            bytes.iter().map(|b| format!("{:02X}", b)).collect()
        };

        self.build_response(original, &hex_str)
    }

    /// Handle Mode 04 — clear stored DTCs and freeze frame data.
    fn process_mode_04(&mut self, original: &str) -> Vec<u8> {
        self.dtcs.clear();
        // Mode 04 positive response = 0x44
        self.build_response(original, "44")
    }

    /// Build a full response with optional echo prefix and proper line endings.
    fn build_response(&self, original_cmd: &str, text: &str) -> Vec<u8> {
        let mut resp = Vec::new();
        if self.echo {
            resp.extend_from_slice(original_cmd.as_bytes());
            resp.push(b'\r');
        }
        resp.extend_from_slice(&self.format_response(text));
        resp
    }

    /// Format a response body according to current linefeed settings.
    /// Appends the trailing prompt (> character).
    fn format_response(&self, text: &str) -> Vec<u8> {
        let nl = if self.linefeed { "\r\n" } else { "\r" };
        let mut resp = Vec::new();
        resp.extend_from_slice(text.as_bytes());
        resp.extend_from_slice(nl.as_bytes());
        resp.extend_from_slice(nl.as_bytes());
        resp.push(b'>');
        resp
    }

    /// Run the simulator on a readable/writable stream.
    ///
    /// Reads commands byte-by-byte until \r, processes them, writes responses.
    /// Stops when the shutdown flag is set or the stream closes.
    pub fn run<F: Read + Write>(
        &mut self,
        stream: &mut F,
        shutdown: &Arc<AtomicBool>,
    ) -> std::io::Result<()> {
        let mut buf = [0u8; 1];
        let mut cmd_buf = Vec::with_capacity(256);

        loop {
            if shutdown.load(Ordering::Relaxed) {
                log::info!("Shutdown signal received");
                break;
            }

            match stream.read(&mut buf) {
                Ok(0) => {
                    // EOF — stream closed
                    log::debug!("Stream closed (EOF)");
                    break;
                }
                Ok(_) => {
                    let byte = buf[0];
                    if byte == b'\r' {
                        let cmd = String::from_utf8_lossy(&cmd_buf).to_string();
                        log::debug!("Received command: {:?}", cmd);

                        let response = self.process_command(&cmd);
                        log::debug!("Sending response: {:?}", String::from_utf8_lossy(&response));

                        stream.write_all(&response)?;
                        stream.flush()?;
                        cmd_buf.clear();
                    } else if byte == b'\n' {
                        // Ignore newlines (some clients send \r\n)
                    } else {
                        cmd_buf.push(byte);
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::Interrupted {
                        continue;
                    }
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        // Non-blocking mode: no data available, sleep briefly
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        continue;
                    }
                    return Err(e);
                }
            }
        }

        Ok(())
    }
}

impl Default for Elm327Simulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: extract the text portion from a response (strip echo + prompt).
    fn response_text(resp: &[u8]) -> String {
        String::from_utf8_lossy(resp).to_string()
    }

    #[test]
    fn test_reset() {
        let mut sim = Elm327Simulator::new();
        sim.echo = false; // disable echo for cleaner assertions
        let resp = sim.process_command("ATZ");
        let text = response_text(&resp);
        assert!(
            text.contains("ELM327 v1.5"),
            "ATZ should return version string, got: {:?}",
            text
        );
        assert!(text.ends_with('>'), "Response should end with prompt");
    }

    #[test]
    fn test_echo_off() {
        let mut sim = Elm327Simulator::new();
        assert!(sim.echo, "Echo should start ON");

        // ATE0 response should still echo (echo is on when command is received)
        let resp = sim.process_command("ATE0");
        let text = response_text(&resp);
        assert!(text.contains("ATE0"), "ATE0 itself should be echoed");
        assert!(text.contains("OK"));
        assert!(!sim.echo, "Echo should now be OFF");

        // Next command should NOT be echoed
        let resp2 = sim.process_command("ATI");
        let text2 = response_text(&resp2);
        assert!(
            !text2.contains("ATI"),
            "Command should not be echoed when echo is off"
        );
        assert!(text2.contains("ELM327 v1.5"));
    }

    #[test]
    fn test_linefeed() {
        let mut sim = Elm327Simulator::new();
        sim.echo = false;

        // Default: linefeed ON => \r\n
        let resp = sim.process_command("ATI");
        assert!(
            resp.windows(2).any(|w| w == b"\r\n"),
            "With linefeed ON, should contain \\r\\n"
        );

        // Turn linefeed off
        sim.process_command("ATL0");
        assert!(!sim.linefeed);

        let resp2 = sim.process_command("ATI");
        let text = response_text(&resp2);
        // Should use \r only, not \r\n
        assert!(
            !text.contains("\r\n"),
            "With linefeed OFF, should not contain \\r\\n"
        );
        assert!(text.contains('\r'), "Should still contain \\r");
    }

    #[test]
    fn test_protocol() {
        let mut sim = Elm327Simulator::new();
        sim.echo = false;

        let resp = sim.process_command("ATSP6");
        let text = response_text(&resp);
        assert!(text.contains("OK"), "ATSP6 should return OK");
        assert_eq!(sim.protocol, 6);

        let resp2 = sim.process_command("ATDPN");
        let text2 = response_text(&resp2);
        assert!(
            text2.contains('6'),
            "ATDPN should return '6', got: {:?}",
            text2
        );
    }

    #[test]
    fn test_voltage() {
        let mut sim = Elm327Simulator::new();
        sim.echo = false;

        let resp = sim.process_command("ATRV");
        let text = response_text(&resp);
        assert!(
            text.contains("12.6V"),
            "ATRV should return voltage, got: {:?}",
            text
        );
    }

    #[test]
    fn test_pps() {
        let mut sim = Elm327Simulator::new();
        sim.echo = false;

        let resp = sim.process_command("ATPPS");
        let text = response_text(&resp);
        assert!(text.contains("00:FF F"), "PPS should contain first entry");
        assert!(text.contains("2A:00 N"), "PPS should contain last entry");
    }

    #[test]
    fn test_unknown_command() {
        let mut sim = Elm327Simulator::new();
        sim.echo = false;

        let resp = sim.process_command("ATXYZ");
        let text = response_text(&resp);
        assert!(
            text.contains('?'),
            "Unknown AT command should return '?', got: {:?}",
            text
        );
    }

    #[test]
    fn test_obd_pid() {
        let mut sim = Elm327Simulator::new();
        sim.echo = false;

        let resp = sim.process_command("0100");
        let text = response_text(&resp);
        assert!(
            text.contains("41 00 BE 3E B8 13"),
            "0100 should return supported PIDs, got: {:?}",
            text
        );

        // Unknown PID
        let resp2 = sim.process_command("01FF");
        let text2 = response_text(&resp2);
        assert!(
            text2.contains("NO DATA"),
            "Unknown PID should return NO DATA, got: {:?}",
            text2
        );
    }

    #[test]
    fn test_obd_pid_no_spaces() {
        let mut sim = Elm327Simulator::new();
        sim.echo = false;
        sim.process_command("ATS0");

        let resp = sim.process_command("0100");
        let text = response_text(&resp);
        assert!(
            text.contains("4100BE3EB813"),
            "With spaces off, should return compact hex, got: {:?}",
            text
        );
    }

    #[test]
    fn test_forscan_init_sequence() {
        // Simulate the typical FORScan initialization sequence:
        // ATZ -> ATE0 -> ATL0 -> ATH1 -> ATS0 -> ATAT1 -> ATSP6
        let mut sim = Elm327Simulator::new();

        // ATZ — reset
        let resp = sim.process_command("ATZ");
        let text = response_text(&resp);
        assert!(text.contains("ELM327 v1.5"), "ATZ failed: {:?}", text);

        // ATE0 — echo off
        let resp = sim.process_command("ATE0");
        let text = response_text(&resp);
        assert!(text.contains("OK"), "ATE0 failed: {:?}", text);
        assert!(!sim.echo);

        // ATL0 — linefeed off
        let resp = sim.process_command("ATL0");
        let text = response_text(&resp);
        assert!(text.contains("OK"), "ATL0 failed: {:?}", text);
        assert!(!sim.linefeed);

        // ATH1 — headers on
        let resp = sim.process_command("ATH1");
        let text = response_text(&resp);
        assert!(text.contains("OK"), "ATH1 failed: {:?}", text);
        assert!(sim.headers);

        // ATS0 — spaces off
        let resp = sim.process_command("ATS0");
        let text = response_text(&resp);
        assert!(text.contains("OK"), "ATS0 failed: {:?}", text);
        assert!(!sim.spaces);

        // ATAT1 — adaptive timing 1
        let resp = sim.process_command("ATAT1");
        let text = response_text(&resp);
        assert!(text.contains("OK"), "ATAT1 failed: {:?}", text);
        assert_eq!(sim.adaptive_timing, 1);

        // ATSP6 — set protocol to ISO 15765-4 CAN 11/500
        let resp = sim.process_command("ATSP6");
        let text = response_text(&resp);
        assert!(text.contains("OK"), "ATSP6 failed: {:?}", text);
        assert_eq!(sim.protocol, 6);

        // Verify final state: echo off, no linefeed, headers on, no spaces
        assert!(!sim.echo);
        assert!(!sim.linefeed);
        assert!(sim.headers);
        assert!(!sim.spaces);
    }

    // ── DTC injection / Mode 03 / Mode 04 tests ─────────────────────────

    #[test]
    fn test_sim_dtc_read_no_dtcs() {
        let mut sim = Elm327Simulator::new();
        sim.echo = false;
        sim.linefeed = false;

        // Mode 03 with no injected DTCs should return NO DATA
        let resp = sim.process_command("03");
        let text = response_text(&resp);
        assert!(
            text.contains("NO DATA"),
            "Mode 03 with no DTCs should return NO DATA, got: {:?}",
            text
        );
    }

    #[test]
    fn test_sim_with_injected_misfire_dtc() {
        let mut sim = Elm327Simulator::new();
        sim.echo = false;
        sim.linefeed = false;
        sim.add_dtc(0x03, 0x01); // P0301 — cylinder 1 misfire
        sim.add_dtc(0x03, 0x51); // P0351 — ignition coil A

        let resp = sim.process_command("03");
        let text = response_text(&resp);
        assert!(
            text.contains("43"),
            "Should return Mode 03 response (0x43), got: {:?}",
            text
        );
        // With spaces on (default), response should be "43 02 03 01 03 51 00 00"
        assert!(
            text.contains("03 01"),
            "Should contain P0301 bytes, got: {:?}",
            text
        );
        assert!(
            text.contains("03 51"),
            "Should contain P0351 bytes, got: {:?}",
            text
        );
    }

    #[test]
    fn test_sim_dtc_response_no_spaces() {
        let mut sim = Elm327Simulator::new();
        sim.echo = false;
        sim.linefeed = false;
        sim.spaces = false;
        sim.add_dtc(0x03, 0x01); // P0301

        let resp = sim.process_command("03");
        let text = response_text(&resp);
        // No spaces: "4301030100000000" or similar
        assert!(
            text.contains("43"),
            "Should contain 43 response byte, got: {:?}",
            text
        );
        assert!(
            !text.contains(" 03"),
            "Should NOT have spaces in response, got: {:?}",
            text
        );
    }

    #[test]
    fn test_sim_clear_dtcs() {
        let mut sim = Elm327Simulator::new();
        sim.echo = false;
        sim.linefeed = false;
        sim.add_dtc(0x03, 0x01); // P0301

        // Mode 04 — clear DTCs
        let resp = sim.process_command("04");
        let text = response_text(&resp);
        assert!(
            text.contains("44"),
            "Mode 04 should return 0x44 positive response, got: {:?}",
            text
        );

        // Verify DTCs are cleared — Mode 03 should return NO DATA
        let resp2 = sim.process_command("03");
        let text2 = response_text(&resp2);
        assert!(
            text2.contains("NO DATA"),
            "DTCs should be cleared after Mode 04, got: {:?}",
            text2
        );
    }

    #[test]
    fn test_sim_dtc_parsed_by_obd_decoder() {
        use elm327_core::obd::{decode_dtc, decode_dtcs, parse_hex_response};

        let mut sim = Elm327Simulator::new();
        sim.echo = false;
        sim.linefeed = false;
        sim.add_dtc(0x03, 0x01); // P0301
        sim.add_dtc(0x03, 0x51); // P0351

        let resp = sim.process_command("03");
        let text = response_text(&resp);
        // Strip the trailing prompt and whitespace to get just the hex
        let hex_part = text.trim().trim_end_matches('>').trim();
        let bytes = parse_hex_response(hex_part);

        // bytes[0] = 0x43 (Mode 03 response)
        // bytes[1] = count
        // bytes[2..] = DTC pairs
        assert_eq!(bytes[0], 0x43);
        let dtcs = decode_dtcs(&bytes[2..]);
        assert_eq!(dtcs.len(), 2);
        assert_eq!(dtcs[0].code, "P0301");
        assert_eq!(dtcs[1].code, "P0351");
    }

    #[test]
    fn test_run_with_stream() {
        use std::io::Cursor;

        // Simulate sending "ATI\r" via a buffer
        let input = b"ATI\r";
        let mut stream = Cursor::new(Vec::new());
        stream.get_mut().extend_from_slice(input);
        stream.set_position(0);

        let mut sim = Elm327Simulator::new();
        sim.echo = false;
        sim.linefeed = false;

        let shutdown = Arc::new(AtomicBool::new(false));

        // Run will read until EOF (cursor exhausted) and return
        let result = sim.run(&mut stream, &shutdown);
        assert!(result.is_ok());

        // Check what was written — Cursor writes at current position
        let written = stream.into_inner();
        // The response is written after the input bytes
        let resp_part = &written[input.len()..];
        let text = String::from_utf8_lossy(resp_part);
        assert!(
            text.contains("ELM327 v1.5"),
            "Stream should contain version response, got: {:?}",
            text
        );
    }
}
