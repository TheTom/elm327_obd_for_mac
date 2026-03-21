use std::os::fd::{AsRawFd, OwnedFd};
use std::path::PathBuf;

use nix::sys::termios;

use crate::error::{BridgeError, Result};

/// A PTY pair for bridging Wine COM port to serial device.
/// - `controller`: the bridge reads/writes on this end
/// - `device_fd`: Wine connects to this end (via symlink)
/// - `device_path`: filesystem path to the device end (e.g., /dev/ttys003)
pub struct PtyPair {
    pub controller: OwnedFd,
    pub device_fd: OwnedFd,
    pub device_path: PathBuf,
}

impl PtyPair {
    /// Create a new PTY pair using openpty.
    ///
    /// # Example
    /// ```no_run
    /// use elm327_core::pty::PtyPair;
    /// let pair = PtyPair::create().expect("failed to create PTY pair");
    /// println!("device at: {}", pair.device_path().display());
    /// ```
    pub fn create() -> Result<Self> {
        let pty = nix::pty::openpty(None, None)
            .map_err(BridgeError::PtyCreation)?;

        // Set raw mode so data passes through without line-discipline buffering.
        // TODO: Consider configuring baud rate for ELM327 compatibility
        let mut attrs = termios::tcgetattr(&pty.slave)
            .map_err(BridgeError::PtyCreation)?;
        termios::cfmakeraw(&mut attrs);
        termios::tcsetattr(&pty.slave, termios::SetArg::TCSANOW, &attrs)
            .map_err(BridgeError::PtyCreation)?;

        let device_path = nix::unistd::ttyname(&pty.slave)
            .map_err(BridgeError::PtyCreation)?;

        log::debug!(
            "PTY pair created: controller fd={}, device={}",
            pty.master.as_raw_fd(),
            device_path.display()
        );

        Ok(Self {
            controller: pty.master,
            device_fd: pty.slave,
            device_path,
        })
    }

    /// Get the device path for Wine COM symlink.
    pub fn device_path(&self) -> &std::path::Path {
        &self.device_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nix::poll::{PollFd, PollFlags, PollTimeout};
    use std::os::fd::BorrowedFd;

    /// Helper: poll an fd for readability with a timeout in ms.
    fn poll_readable(fd: &OwnedFd, timeout_ms: i32) -> bool {
        let borrowed = unsafe { BorrowedFd::borrow_raw(fd.as_raw_fd()) };
        let mut fds = [PollFd::new(borrowed, PollFlags::POLLIN)];
        let n = nix::poll::poll(&mut fds, PollTimeout::from(timeout_ms as u16))
            .expect("poll failed");
        n > 0
    }

    #[test]
    fn test_pty_create() {
        let pair = PtyPair::create().expect("failed to create PTY pair");
        assert!(
            pair.device_path().starts_with("/dev/"),
            "device_path should start with /dev/, got: {}",
            pair.device_path().display()
        );
    }

    #[test]
    fn test_pty_roundtrip() {
        let pair = PtyPair::create().expect("failed to create PTY pair");

        let msg = b"hello";
        nix::unistd::write(&pair.controller, msg).expect("write to controller failed");

        assert!(poll_readable(&pair.device_fd, 1000), "device_fd not readable");

        let mut buf = [0u8; 64];
        let n = nix::unistd::read(pair.device_fd.as_raw_fd(), &mut buf)
            .expect("read from device_fd failed");

        assert_eq!(&buf[..n], msg);
    }

    #[test]
    fn test_pty_reverse() {
        let pair = PtyPair::create().expect("failed to create PTY pair");

        let msg = b"world";
        nix::unistd::write(&pair.device_fd, msg).expect("write to device_fd failed");

        assert!(poll_readable(&pair.controller, 1000), "controller not readable");

        let mut buf = [0u8; 64];
        let n = nix::unistd::read(pair.controller.as_raw_fd(), &mut buf)
            .expect("read from controller failed");

        assert_eq!(&buf[..n], msg);
    }

    #[test]
    fn test_pty_latency() {
        let pair = PtyPair::create().expect("failed to create PTY pair");

        let msg = [0xAA_u8; 64];
        let start = std::time::Instant::now();

        nix::unistd::write(&pair.controller, &msg).expect("write failed");
        assert!(poll_readable(&pair.device_fd, 1000), "device_fd not readable");

        let mut buf = [0u8; 64];
        let n = nix::unistd::read(pair.device_fd.as_raw_fd(), &mut buf)
            .expect("read failed");
        let elapsed = start.elapsed();

        assert_eq!(n, 64);
        assert_eq!(&buf[..n], &msg[..]);
        assert!(
            elapsed < std::time::Duration::from_millis(5),
            "64-byte roundtrip took {:?}, expected < 5ms",
            elapsed
        );
    }
}
