use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("PTY creation failed: {0}")]
    PtyCreation(#[source] nix::Error),

    #[error("Serial port error: {0}")]
    Serial(#[source] serialport::Error),

    #[error("I/O error: {0}")]
    Io(#[source] std::io::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Wine prefix not found: {path}")]
    WinePrefix { path: PathBuf },

    #[error("Timeout after {0:?}")]
    Timeout(Duration),

    #[error("Bridge shutdown")]
    Shutdown,
}

pub type Result<T> = std::result::Result<T, BridgeError>;

impl From<nix::Error> for BridgeError {
    fn from(err: nix::Error) -> Self {
        BridgeError::PtyCreation(err)
    }
}

impl From<std::io::Error> for BridgeError {
    fn from(err: std::io::Error) -> Self {
        BridgeError::Io(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::Duration;

    #[test]
    fn display_pty_creation() {
        let err = BridgeError::PtyCreation(nix::Error::ENOENT);
        assert!(err.to_string().starts_with("PTY creation failed: ENOENT"));
    }

    #[test]
    fn display_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file gone");
        let err = BridgeError::Io(io_err);
        assert_eq!(err.to_string(), "I/O error: file gone");
    }

    #[test]
    fn display_config_error() {
        let err = BridgeError::Config("bad yaml".to_string());
        assert_eq!(err.to_string(), "Config error: bad yaml");
    }

    #[test]
    fn display_device_not_found() {
        let err = BridgeError::DeviceNotFound("/dev/ttyUSB0".to_string());
        assert_eq!(err.to_string(), "Device not found: /dev/ttyUSB0");
    }

    #[test]
    fn display_wine_prefix() {
        let err = BridgeError::WinePrefix {
            path: PathBuf::from("/home/user/.wine"),
        };
        assert_eq!(err.to_string(), "Wine prefix not found: /home/user/.wine");
    }

    #[test]
    fn display_timeout() {
        let err = BridgeError::Timeout(Duration::from_secs(5));
        assert_eq!(err.to_string(), "Timeout after 5s");
    }

    #[test]
    fn display_shutdown() {
        let err = BridgeError::Shutdown;
        assert_eq!(err.to_string(), "Bridge shutdown");
    }

    #[test]
    fn from_nix_error() {
        let nix_err = nix::Error::EPERM;
        let bridge_err: BridgeError = nix_err.into();
        assert!(matches!(bridge_err, BridgeError::PtyCreation(_)));
        assert!(bridge_err
            .to_string()
            .starts_with("PTY creation failed: EPERM"));
    }

    #[test]
    fn from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe broke");
        let bridge_err: BridgeError = io_err.into();
        assert!(matches!(bridge_err, BridgeError::Io(_)));
        assert_eq!(bridge_err.to_string(), "I/O error: pipe broke");
    }
}
