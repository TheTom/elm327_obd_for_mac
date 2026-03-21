use std::io::{Read, Write};
use std::time::Duration;

use crate::error::{BridgeError, Result};

/// Serial port configuration for ELM327 communication.
///
/// # Example
/// ```
/// use elm327_core::serial::SerialConfig;
/// use std::time::Duration;
///
/// let config = SerialConfig {
///     device: "/dev/ttyUSB0".to_string(),
///     baud_rate: 115200,
///     timeout: Duration::from_millis(500),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct SerialConfig {
    pub device: String,
    pub baud_rate: u32,
    pub timeout: Duration,
}

impl Default for SerialConfig {
    fn default() -> Self {
        Self {
            device: String::new(),
            baud_rate: 38400, // ELM327 factory default
            timeout: Duration::from_secs(1),
        }
    }
}

/// Wrapper around a native serial port connection (TTYPort on Unix).
///
/// All reads/writes have timeouts — never blocks forever.
/// All I/O is logged at debug level.
///
/// Uses `open_native()` instead of `open()` so we get a concrete `TTYPort`
/// that implements `AsRawFd`, which we need for poll()-based I/O.
// TODO: Add Windows support via COMPort when needed
#[cfg(unix)]
pub struct SerialConnection {
    port: serialport::TTYPort,
    device: String,
}

#[cfg(unix)]
impl SerialConnection {
    /// Open a serial port with 8N1, no flow control.
    ///
    /// Uses `open_native()` to get a `TTYPort` directly, giving us
    /// access to `AsRawFd` for poll()-based event loops.
    pub fn open(config: &SerialConfig) -> Result<Self> {
        log::info!(
            "Opening serial port: {} at {} baud",
            config.device,
            config.baud_rate
        );

        let port = serialport::new(&config.device, config.baud_rate)
            .data_bits(serialport::DataBits::Eight)
            .parity(serialport::Parity::None)
            .stop_bits(serialport::StopBits::One)
            .flow_control(serialport::FlowControl::None)
            .timeout(config.timeout)
            .open_native()
            .map_err(BridgeError::Serial)?;

        log::debug!("Serial port opened successfully: {}", config.device);

        Ok(Self {
            port,
            device: config.device.clone(),
        })
    }

    /// Read bytes with timeout. Returns number of bytes read.
    /// Timeout is NOT an error — returns `Ok(0)`.
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self.port.read(buf) {
            Ok(n) => {
                if n > 0 {
                    log::debug!("Serial RX [{}]: {:?}", self.device, &buf[..n]);
                }
                Ok(n)
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(0),
            Err(e) => {
                log::error!("Serial read error [{}]: {}", self.device, e);
                Err(BridgeError::Io(e))
            }
        }
    }

    /// Write bytes. Logs data at debug level before sending.
    pub fn write(&mut self, data: &[u8]) -> Result<usize> {
        log::debug!("Serial TX [{}]: {:?}", self.device, data);
        let n = self.port.write(data).map_err(|e| {
            log::error!("Serial write error [{}]: {}", self.device, e);
            BridgeError::Io(e)
        })?;
        Ok(n)
    }

    /// Flush output buffer.
    pub fn flush(&mut self) -> Result<()> {
        self.port.flush().map_err(BridgeError::Io)
    }

    /// Get the raw file descriptor for use with poll().
    pub fn as_raw_fd(&self) -> std::os::unix::io::RawFd {
        use std::os::unix::io::AsRawFd;
        self.port.as_raw_fd()
    }

    /// Get device name.
    pub fn device(&self) -> &str {
        &self.device
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serial_config_defaults() {
        let config = SerialConfig::default();
        assert_eq!(config.baud_rate, 38400);
        assert_eq!(config.timeout, Duration::from_secs(1));
        assert!(config.device.is_empty());
    }

    #[test]
    fn test_serial_open_nonexistent() {
        let config = SerialConfig {
            device: "/dev/nonexistent".to_string(),
            ..Default::default()
        };
        let result = SerialConnection::open(&config);
        assert!(result.is_err(), "Opening a nonexistent device should fail");
    }
}
