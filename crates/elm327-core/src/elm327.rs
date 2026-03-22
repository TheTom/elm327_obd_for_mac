use crate::error::{BridgeError, Result};
use crate::serial::{SerialConfig, SerialConnection};
use std::time::{Duration, Instant};

/// Default command timeout (5 seconds).
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// ATZ reset needs extra time for the adapter to reinitialize.
const RESET_TIMEOUT: Duration = Duration::from_secs(2);

/// Known ELM327 error responses.
/// These show up as response lines when something goes wrong on the bus or adapter.
const ERROR_RESPONSES: &[&str] = &[
    "NO DATA",
    "UNABLE TO CONNECT",
    "BUS INIT: ...ERROR",
    "BUFFER FULL",
    "CAN ERROR",
    "DATA ERROR",
    "ERR",
    "FB ERROR",
    "LV RESET",
    "BUS BUSY",
    "ACT ALERT",
];

/// ELM327 adapter state.
///
/// Wraps a `SerialConnection` and tracks adapter configuration (echo, headers,
/// spaces, protocol). Use `open()` then `init()` to get a ready-to-use adapter.
///
/// # Example
/// ```no_run
/// use elm327_core::serial::SerialConfig;
/// use elm327_core::elm327::Elm327;
/// use std::time::Duration;
///
/// let config = SerialConfig {
///     device: "/dev/cu.wchusbserial14340".to_string(),
///     baud_rate: 38400,
///     timeout: Duration::from_millis(500),
/// };
/// let mut elm = Elm327::open(&config).unwrap();
/// let version = elm.init().unwrap();
/// println!("Adapter: {}", version);
/// ```
pub struct Elm327 {
    conn: SerialConnection,
    echo: bool,
    headers: bool,
    spaces: bool,
    protocol: u8,
    version: Option<String>,
}

/// Response from the adapter after a command.
///
/// Contains the parsed response lines (excluding echo and prompt) and
/// error detection results. For OBD responses with headers on and spaces off,
/// a line like `"7E8064100BE3EB813"` means:
/// - `7E8` = responder CAN ID
/// - `06` = data length
/// - `41 00` = service 01 response, PID 00
/// - `BE 3E B8 13` = data bytes
#[derive(Debug, Clone)]
pub struct Response {
    /// Raw response lines (excluding echo and prompt).
    pub lines: Vec<String>,
    /// Whether the response indicates an error.
    pub is_error: bool,
    /// Error message if any (e.g., "NO DATA", "UNABLE TO CONNECT").
    pub error: Option<String>,
}

impl Response {
    /// Check if any line in the response matches a known ELM327 error string.
    fn detect_error(lines: &[String]) -> (bool, Option<String>) {
        for line in lines {
            let upper = line.trim().to_uppercase();
            for &err in ERROR_RESPONSES {
                if upper.contains(err) {
                    return (true, Some(line.trim().to_string()));
                }
            }
            // "?" by itself is also an error (unrecognized command)
            if upper == "?" {
                return (true, Some("Unrecognized command".to_string()));
            }
        }
        (false, None)
    }

    /// Build a Response from raw lines, running error detection.
    fn from_lines(lines: Vec<String>) -> Self {
        let (is_error, error) = Self::detect_error(&lines);
        Self {
            lines,
            is_error,
            error,
        }
    }
}

impl Elm327 {
    /// Open a connection to an ELM327 adapter.
    ///
    /// Does NOT initialize — call `init()` after opening.
    pub fn open(config: &SerialConfig) -> Result<Self> {
        let conn = SerialConnection::open(config)?;
        log::info!("ELM327 connection opened on {}", config.device);

        Ok(Self {
            conn,
            echo: true,     // factory default: echo on
            headers: false, // factory default: headers off
            spaces: true,   // factory default: spaces on
            protocol: 0,    // factory default: auto
            version: None,
        })
    }

    /// Initialize the adapter with the standard FORScan-like sequence.
    ///
    /// Sends: ATZ, ATE0, ATL0, ATH1, ATS0, ATAT1
    /// Returns the adapter version string (e.g., "ELM327 v1.5").
    pub fn init(&mut self) -> Result<String> {
        // ATZ — full reset, parse version from response
        let resp = self.send_timeout("ATZ", RESET_TIMEOUT)?;
        let version = resp
            .lines
            .iter()
            .find(|l| l.contains("ELM327") || l.contains("ELM") || l.contains("v"))
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string())
            .trim()
            .to_string();
        self.version = Some(version.clone());
        log::info!("ELM327 adapter version: {}", version);

        // ATE0 — echo off (reduces noise in response parsing)
        self.send("ATE0")?;
        self.echo = false;

        // ATL0 — linefeed off (responses terminated with \r only)
        self.send("ATL0")?;

        // ATH1 — headers on (we need CAN IDs in responses)
        self.send("ATH1")?;
        self.headers = true;

        // ATS0 — spaces off (compact hex output)
        self.send("ATS0")?;
        self.spaces = false;

        // ATAT1 — adaptive timing on (auto-adjusts OBD response timeout)
        self.send("ATAT1")?;

        log::info!("ELM327 initialization complete");
        Ok(version)
    }

    /// Send a raw command and wait for the response with default timeout (5s).
    ///
    /// Appends `\r` if not already present. Waits for the `>` prompt.
    pub fn send(&mut self, cmd: &str) -> Result<Response> {
        self.send_timeout(cmd, DEFAULT_TIMEOUT)
    }

    /// Send a raw command with a custom timeout.
    ///
    /// This is the core send method. It:
    /// 1. Flushes any stale data in the serial buffer
    /// 2. Writes `cmd\r`
    /// 3. Reads byte-by-byte until `>` prompt or timeout
    /// 4. Strips echo line if echo is on
    /// 5. Splits by `\r`, filters empty lines
    /// 6. Checks for known error responses
    pub fn send_timeout(&mut self, cmd: &str, timeout: Duration) -> Result<Response> {
        // Flush stale data
        self.drain_buffer();

        // Append \r if not present
        let cmd_bytes = if cmd.ends_with('\r') {
            cmd.as_bytes().to_vec()
        } else {
            format!("{}\r", cmd).into_bytes()
        };

        log::debug!("ELM327 TX: {:?}", cmd);
        self.conn.write_all(&cmd_bytes)?;

        // Read until '>' prompt or timeout
        let start = Instant::now();
        let mut buf = [0u8; 1];
        let mut accumulator = Vec::with_capacity(256);

        loop {
            if start.elapsed() > timeout {
                log::warn!("ELM327 command timed out after {:?}: {}", timeout, cmd);
                return Err(BridgeError::Timeout(timeout));
            }

            match self.conn.read(&mut buf)? {
                0 => {
                    // Timeout on read — keep trying until overall timeout
                    continue;
                }
                _ => {
                    let byte = buf[0];
                    if byte == b'>' {
                        break;
                    }
                    accumulator.push(byte);
                }
            }
        }

        let raw = String::from_utf8_lossy(&accumulator).to_string();
        log::debug!("ELM327 RX raw: {:?}", raw);

        // Split by \r, filter empty/whitespace-only lines
        let mut lines: Vec<String> = raw
            .split('\r')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // Strip echo line if echo is on (first line is the echoed command)
        if self.echo && !lines.is_empty() {
            let echo_line = lines[0].to_uppercase();
            let cmd_upper = cmd.trim_end_matches('\r').to_uppercase();
            if echo_line.contains(&cmd_upper) {
                lines.remove(0);
            }
        }

        let response = Response::from_lines(lines);
        if response.is_error {
            log::warn!("ELM327 error response for '{}': {:?}", cmd, response.error);
        }

        Ok(response)
    }

    /// Set OBD protocol.
    ///
    /// For Ford HS-CAN: protocol 6 (ISO 15765-4 CAN, 11-bit, 500 kbps).
    /// For Ford MS-CAN: use protocol B with ATPB for 125 kbps.
    pub fn set_protocol(&mut self, protocol: u8) -> Result<()> {
        let cmd = format!("ATSP{:X}", protocol);
        let resp = self.send(&cmd)?;
        if resp.is_error {
            return Err(BridgeError::Config(format!(
                "Failed to set protocol {}: {:?}",
                protocol, resp.error
            )));
        }
        self.protocol = protocol;
        log::info!("ELM327 protocol set to {}", protocol);
        Ok(())
    }

    /// Set CAN header for addressing specific modules.
    ///
    /// # Example
    /// ```no_run
    /// # use elm327_core::elm327::Elm327;
    /// # let mut elm: Elm327 = todo!();
    /// elm.set_header("7E0").unwrap(); // PCM
    /// elm.set_header("7E2").unwrap(); // TCM
    /// ```
    pub fn set_header(&mut self, header: &str) -> Result<()> {
        let cmd = format!("ATSH{}", header);
        let resp = self.send(&cmd)?;
        if resp.is_error {
            return Err(BridgeError::Config(format!(
                "Failed to set header '{}': {:?}",
                header, resp.error
            )));
        }
        log::debug!("ELM327 CAN header set to {}", header);
        Ok(())
    }

    /// Set CAN receive address filter.
    ///
    /// # Example
    /// ```no_run
    /// # use elm327_core::elm327::Elm327;
    /// # let mut elm: Elm327 = todo!();
    /// elm.set_receive_filter("7E8").unwrap(); // PCM response
    /// ```
    pub fn set_receive_filter(&mut self, filter: &str) -> Result<()> {
        let cmd = format!("ATCRA{}", filter);
        let resp = self.send(&cmd)?;
        if resp.is_error {
            return Err(BridgeError::Config(format!(
                "Failed to set receive filter '{}': {:?}",
                filter, resp.error
            )));
        }
        log::debug!("ELM327 CAN receive filter set to {}", filter);
        Ok(())
    }

    /// Clear receive address filter (receive from all).
    pub fn clear_receive_filter(&mut self) -> Result<()> {
        let resp = self.send("ATCRA")?;
        if resp.is_error {
            return Err(BridgeError::Config(format!(
                "Failed to clear receive filter: {:?}",
                resp.error
            )));
        }
        log::debug!("ELM327 CAN receive filter cleared");
        Ok(())
    }

    /// Send an OBD/UDS command (hex bytes) and return the parsed response.
    ///
    /// This is for vehicle commands, not AT commands. The hex string is sent
    /// directly to the bus.
    ///
    /// # Example
    /// ```no_run
    /// # use elm327_core::elm327::Elm327;
    /// # let mut elm: Elm327 = todo!();
    /// let resp = elm.send_obd("0100").unwrap(); // Supported PIDs (service 01, PID 00)
    /// ```
    // TODO: Add multi-frame (ISO-TP) response reassembly for long responses
    pub fn send_obd(&mut self, hex_cmd: &str) -> Result<Response> {
        self.send(hex_cmd)
    }

    /// Read the adapter's input voltage (OBD port voltage).
    ///
    /// Returns voltage as f32 (e.g., 12.4 for 12.4V).
    pub fn read_voltage(&mut self) -> Result<f32> {
        let resp = self.send("ATRV")?;
        if resp.is_error {
            return Err(BridgeError::Config(format!(
                "Failed to read voltage: {:?}",
                resp.error
            )));
        }

        // Response is like "12.4V" or "12.4" — strip trailing 'V' and parse
        let voltage_str = resp
            .lines
            .first()
            .ok_or_else(|| BridgeError::Config("No voltage response".to_string()))?
            .trim()
            .trim_end_matches('V')
            .trim_end_matches('v')
            .trim();

        voltage_str.parse::<f32>().map_err(|e| {
            BridgeError::Config(format!("Failed to parse voltage '{}': {}", voltage_str, e))
        })
    }

    /// Get the adapter version string (available after `init()`).
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    /// Check if the adapter is connected and responding.
    ///
    /// Sends `ATI` and checks for a valid response.
    pub fn ping(&mut self) -> Result<bool> {
        match self.send("ATI") {
            Ok(resp) => Ok(!resp.is_error && !resp.lines.is_empty()),
            Err(_) => Ok(false),
        }
    }

    /// Drain any stale bytes from the serial buffer.
    /// Reads until nothing comes back (timeout on each read).
    fn drain_buffer(&mut self) {
        let mut junk = [0u8; 64];
        loop {
            match self.conn.read(&mut junk) {
                Ok(0) => break,
                Ok(n) => {
                    log::debug!("ELM327 drained {} stale bytes", n);
                }
                Err(_) => break,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_error_detection() {
        // Known error strings should be detected
        let cases = vec![
            ("NO DATA", true),
            ("UNABLE TO CONNECT", true),
            ("BUS INIT: ...ERROR", true),
            ("BUFFER FULL", true),
            ("CAN ERROR", true),
            ("DATA ERROR", true),
            ("ERR", true),
            ("FB ERROR", true),
            ("LV RESET", true),
            ("BUS BUSY", true),
            ("ACT ALERT", true),
            ("?", true),
            // Normal responses should NOT be errors
            ("OK", false),
            ("41 00 BE 3E B8 13", false),
            ("7E8064100BE3EB813", false),
            ("ELM327 v1.5", false),
        ];

        for (input, expected_error) in cases {
            let lines = vec![input.to_string()];
            let resp = Response::from_lines(lines);
            assert_eq!(
                resp.is_error, expected_error,
                "Expected is_error={} for '{}'",
                expected_error, input
            );
        }
    }

    #[test]
    fn test_response_parsing_multiline() {
        // Simulate a multi-line OBD response (headers on, spaces off)
        // e.g., querying multiple ECUs for supported PIDs
        let lines = vec![
            "7E8064100BE3EB813".to_string(), // PCM response
            "7EA064100401000C1".to_string(), // TCM response
        ];
        let resp = Response::from_lines(lines.clone());

        assert!(!resp.is_error);
        assert!(resp.error.is_none());
        assert_eq!(resp.lines.len(), 2);
        assert_eq!(resp.lines[0], "7E8064100BE3EB813");
        assert_eq!(resp.lines[1], "7EA064100401000C1");
    }

    #[test]
    fn test_response_error_message_preserved() {
        let lines = vec!["NO DATA".to_string()];
        let resp = Response::from_lines(lines);
        assert!(resp.is_error);
        assert_eq!(resp.error.as_deref(), Some("NO DATA"));
    }

    #[test]
    fn test_response_empty_lines() {
        let lines: Vec<String> = vec![];
        let resp = Response::from_lines(lines);
        assert!(!resp.is_error);
        assert!(resp.error.is_none());
        assert!(resp.lines.is_empty());
    }

    #[test]
    fn test_response_error_case_insensitive() {
        // Error detection should be case-insensitive
        let lines = vec!["no data".to_string()];
        let resp = Response::from_lines(lines);
        assert!(resp.is_error);
    }
}
