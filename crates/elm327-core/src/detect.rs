use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::error::{BridgeError, Result};
use crate::serial::{SerialConfig, SerialConnection};

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceType {
    WchUsbSerial,  // CH340-based (most common for ELM327)
    UsbSerial,     // Generic USB-serial
    SlabUsbToUart, // Silicon Labs CP210x
    Unknown,
}

#[derive(Debug, Clone)]
pub struct DetectedDevice {
    pub path: PathBuf,
    pub device_type: DeviceType,
}

impl std::fmt::Display for DetectedDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({:?})", self.path.display(), self.device_type)
    }
}

/// Result of a successful baud rate probe.
///
/// Contains the device path, working baud rate, and the ELM327 version
/// string extracted from the ATZ reset response.
///
/// # Example
/// ```
/// use elm327_core::detect::ProbeResult;
/// use std::path::PathBuf;
///
/// let result = ProbeResult {
///     device: PathBuf::from("/dev/cu.wchusbserial14340"),
///     baud_rate: 38400,
///     version: "ELM327 v1.5".to_string(),
/// };
/// assert_eq!(result.baud_rate, 38400);
/// ```
#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub device: PathBuf,
    pub baud_rate: u32,
    pub version: String,
}

/// Common ELM327 baud rates in order of likelihood.
/// 38400 is the factory default, so we try it first.
// TODO: Consider making this configurable or adding 500000 for some clones
const BAUD_RATES: &[u32] = &[38400, 115200, 57600, 9600, 230400];

/// Classify a device path into a DeviceType based on naming conventions.
///
/// # Examples
/// ```
/// use elm327_core::detect::{classify_device, DeviceType};
/// assert_eq!(classify_device("cu.wchusbserial14340"), DeviceType::WchUsbSerial);
/// ```
pub fn classify_device(path: &str) -> DeviceType {
    let name = path.to_lowercase();
    if name.contains("wchusbserial") {
        DeviceType::WchUsbSerial
    } else if name.contains("slab_usbtouart") || name.contains("cp210") {
        DeviceType::SlabUsbToUart
    } else if name.contains("usbserial") || name.contains("usbmodem") {
        DeviceType::UsbSerial
    } else {
        DeviceType::Unknown
    }
}

/// Enumerate /dev/cu.* and filter for likely OBD adapters.
/// Returns devices sorted by likelihood (WchUsbSerial first).
///
/// # Returns
/// A `Vec<DetectedDevice>` sorted with the most likely OBD adapter first.
/// Only includes devices matching known USB-serial chipset naming patterns.
pub fn detect_devices() -> Vec<DetectedDevice> {
    let mut devices = Vec::new();

    // Glob /dev/cu.*
    let entries = match std::fs::read_dir("/dev") {
        Ok(entries) => entries,
        Err(e) => {
            log::warn!("Failed to read /dev: {}", e);
            return devices;
        }
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("cu.") {
            continue;
        }

        let path = entry.path();
        let device_type = classify_device(&name);

        // Only include known OBD-likely devices
        if device_type != DeviceType::Unknown {
            devices.push(DetectedDevice { path, device_type });
        }
    }

    // Sort: WchUsbSerial first (most common ELM327), then others
    devices.sort_by_key(|d| match d.device_type {
        DeviceType::WchUsbSerial => 0,
        DeviceType::SlabUsbToUart => 1,
        DeviceType::UsbSerial => 2,
        DeviceType::Unknown => 3,
    });

    devices
}

/// Auto-detect the correct baud rate for an ELM327 device.
///
/// Tries each common baud rate (38400, 115200, 57600, 9600, 230400),
/// sends ATZ (reset), and checks if the response contains "ELM".
///
/// # Arguments
/// * `device` - Serial device path (e.g., "/dev/cu.wchusbserial14340")
/// * `timeout` - Per-attempt timeout for reading the ATZ response
///
/// # Returns
/// A `ProbeResult` with the working baud rate and version string, or
/// `BridgeError::DeviceNotFound` if no baud rate produces a valid response.
///
/// # Example
/// ```no_run
/// use elm327_core::detect::probe_baud_rate;
/// use std::time::Duration;
///
/// let result = probe_baud_rate("/dev/cu.wchusbserial14340", Duration::from_secs(2)).unwrap();
/// println!("Found {} at {} baud", result.version, result.baud_rate);
/// ```
pub fn probe_baud_rate(device: &str, timeout: Duration) -> Result<ProbeResult> {
    log::info!("Probing baud rate for device: {}", device);

    // Track error categories to differentiate "couldn't open port" from
    // "opened but no valid response" (Issue 4: don't collapse all errors
    // into DeviceNotFound).
    let mut last_error: Option<BridgeError> = None;
    let mut had_open_failure = false;
    let mut had_timeout = false;

    for &baud in BAUD_RATES {
        log::debug!("Trying {} baud on {}...", baud, device);

        match try_baud_rate(device, baud, timeout) {
            Ok(version) => {
                log::info!("Found ELM327 at {} baud: {}", baud, version);
                return Ok(ProbeResult {
                    device: PathBuf::from(device),
                    baud_rate: baud,
                    version,
                });
            }
            Err(e) => {
                log::debug!("Baud {} failed for {}: {}", baud, device, e);
                // Serial/Io errors mean we couldn't open the port (permission
                // denied, device busy, etc.). Timeout means the port opened
                // but we got no valid ELM response.
                let is_open_failure = matches!(e, BridgeError::Serial(_) | BridgeError::Io(_));
                if is_open_failure {
                    had_open_failure = true;
                } else {
                    had_timeout = true;
                }
                last_error = Some(e);
                continue;
            }
        }
    }

    // If we never successfully opened the port at any baud rate, surface the
    // underlying serial/IO error instead of a generic DeviceNotFound.
    match last_error {
        Some(err) if !had_timeout && had_open_failure => Err(err),
        _ => Err(BridgeError::DeviceNotFound(format!(
            "No ELM327 response on {} at any baud rate ({:?})",
            device, BAUD_RATES
        ))),
    }
}

/// Try a single baud rate: open port, send ATZ, read response, check for "ELM".
///
/// Returns the version string if successful (e.g., "ELM327 v1.5").
// TODO: Consider adding a small delay after open before sending ATZ —
//       some cheap clones need time to initialize after port open.
fn try_baud_rate(device: &str, baud_rate: u32, timeout: Duration) -> Result<String> {
    let config = SerialConfig {
        device: device.to_string(),
        baud_rate,
        timeout: Duration::from_millis(200), // short read timeout for individual reads
    };

    let mut conn = SerialConnection::open(&config)?;

    // Send ATZ (reset command) — write_all handles flush internally
    conn.write_all(b"ATZ\r")?;

    // Read response, accumulating until we hit timeout or get enough data
    let mut response = Vec::with_capacity(256);
    let mut buf = [0u8; 256];
    let deadline = Instant::now() + timeout;

    while Instant::now() < deadline {
        match conn.read(&mut buf) {
            Ok(0) => {
                // Timeout on this read cycle. ELM327 adapters emit ATZ
                // responses in chunks with gaps, so only stop reading if
                // we've received data AND the `>` prompt indicates the
                // response is complete. Otherwise keep reading until the
                // deadline expires.
                if !response.is_empty() {
                    let text = String::from_utf8_lossy(&response);
                    if text.contains('>') {
                        break;
                    }
                }
            }
            Ok(n) => {
                response.extend_from_slice(&buf[..n]);
                // Check early if we already have what we need
                let text = String::from_utf8_lossy(&response);
                if text.contains("ELM") {
                    return extract_version(&text);
                }
            }
            Err(e) => {
                log::debug!("Read error at {} baud: {}", baud_rate, e);
                return Err(e);
            }
        }
    }

    // Final check on whatever we accumulated
    let text = String::from_utf8_lossy(&response);
    if text.contains("ELM") {
        return extract_version(&text);
    }

    Err(BridgeError::Timeout(timeout))
}

/// Extract the ELM327 version string from an ATZ response.
///
/// Looks for a line containing "ELM" and returns it trimmed.
/// Typical response: "\r\r\nELM327 v1.5\r\n\r\n>"
fn extract_version(response: &str) -> Result<String> {
    for line in response.lines() {
        let trimmed = line.trim();
        if trimmed.contains("ELM") {
            return Ok(trimmed.to_string());
        }
    }

    // Shouldn't get here if caller checked contains("ELM"), but be safe
    Err(BridgeError::DeviceNotFound(
        "ELM version line not found in response".to_string(),
    ))
}

/// Probe a detected device: auto-detect baud rate and return the result.
///
/// This is a convenience wrapper that combines device detection with
/// baud rate probing. Uses the default 2-second timeout per baud rate attempt.
///
/// # Example
/// ```no_run
/// use elm327_core::detect::{DetectedDevice, DeviceType, probe_device};
/// use std::path::PathBuf;
///
/// let dev = DetectedDevice {
///     path: PathBuf::from("/dev/cu.wchusbserial14340"),
///     device_type: DeviceType::WchUsbSerial,
/// };
/// let result = probe_device(&dev).unwrap();
/// println!("Connected: {} @ {} baud", result.version, result.baud_rate);
/// ```
pub fn probe_device(device: &DetectedDevice) -> Result<ProbeResult> {
    let device_path = device.path.to_string_lossy();
    probe_baud_rate(&device_path, Duration::from_secs(2))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_wch_usb_serial() {
        assert_eq!(
            classify_device("cu.wchusbserial14340"),
            DeviceType::WchUsbSerial
        );
    }

    #[test]
    fn classify_generic_usb_serial() {
        assert_eq!(classify_device("cu.usbserial-1420"), DeviceType::UsbSerial);
    }

    #[test]
    fn classify_slab_usb_to_uart() {
        assert_eq!(
            classify_device("cu.SLAB_USBtoUART"),
            DeviceType::SlabUsbToUart
        );
    }

    #[test]
    fn classify_bluetooth_is_unknown() {
        assert_eq!(
            classify_device("cu.Bluetooth-Incoming-Port"),
            DeviceType::Unknown
        );
    }

    #[test]
    fn display_detected_device() {
        let dev = DetectedDevice {
            path: PathBuf::from("/dev/cu.wchusbserial14340"),
            device_type: DeviceType::WchUsbSerial,
        };
        assert_eq!(dev.to_string(), "/dev/cu.wchusbserial14340 (WchUsbSerial)");
    }

    #[test]
    fn test_baud_rates_order() {
        // 38400 is the ELM327 factory default — must be tried first
        assert_eq!(
            BAUD_RATES[0], 38400,
            "Factory default baud rate must be first"
        );
        assert_eq!(
            BAUD_RATES.len(),
            5,
            "Should have exactly 5 baud rates to try"
        );
        assert!(BAUD_RATES.contains(&115200));
        assert!(BAUD_RATES.contains(&57600));
        assert!(BAUD_RATES.contains(&9600));
        assert!(BAUD_RATES.contains(&230400));
    }

    #[test]
    fn test_probe_result_display() {
        let result = ProbeResult {
            device: PathBuf::from("/dev/cu.wchusbserial14340"),
            baud_rate: 38400,
            version: "ELM327 v1.5".to_string(),
        };
        let debug = format!("{:?}", result);
        assert!(
            debug.contains("38400"),
            "Debug output should contain baud rate"
        );
        assert!(
            debug.contains("ELM327 v1.5"),
            "Debug output should contain version"
        );
        assert!(
            debug.contains("wchusbserial14340"),
            "Debug output should contain device path"
        );
    }

    #[test]
    fn test_extract_version_typical() {
        let response = "\r\r\nELM327 v1.5\r\n\r\n>";
        let version = extract_version(response).unwrap();
        assert_eq!(version, "ELM327 v1.5");
    }

    #[test]
    fn test_extract_version_with_extra_whitespace() {
        let response = "  \r\n  ELM327 v2.1  \r\n>";
        let version = extract_version(response).unwrap();
        assert_eq!(version, "ELM327 v2.1");
    }

    #[test]
    fn test_extract_version_no_elm() {
        let response = "garbage\r\nno adapter here\r\n>";
        let result = extract_version(response);
        assert!(result.is_err(), "Should fail when no ELM in response");
    }

    #[test]
    fn test_probe_result_clone() {
        let result = ProbeResult {
            device: PathBuf::from("/dev/ttyUSB0"),
            baud_rate: 115200,
            version: "ELM327 v1.5".to_string(),
        };
        let cloned = result.clone();
        assert_eq!(cloned.baud_rate, result.baud_rate);
        assert_eq!(cloned.version, result.version);
        assert_eq!(cloned.device, result.device);
    }
}
