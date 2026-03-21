use std::os::fd::{BorrowedFd, RawFd};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use nix::poll::{poll, PollFd, PollFlags, PollTimeout};

use crate::error::{BridgeError, Result};

const BUF_SIZE: usize = 4096;
const POLL_TIMEOUT_MS: u16 = 100; // 100ms poll timeout, checks shutdown flag each iteration

/// Statistics for monitoring bridge health.
///
/// # Example
/// ```
/// use elm327_core::bridge::BridgeStats;
/// let stats = BridgeStats::default();
/// assert_eq!(stats.bytes_pty_to_serial, 0);
/// ```
#[derive(Debug, Default, Clone)]
pub struct BridgeStats {
    pub bytes_pty_to_serial: u64,
    pub bytes_serial_to_pty: u64,
    pub forward_count: u64,
    pub errors: u64,
}

/// Bidirectional byte-forwarding bridge between a PTY and a serial port.
///
/// Uses `nix::poll::poll()` (kqueue-backed on macOS) to multiplex two file
/// descriptors with a 100ms timeout, forwarding bytes in both directions.
///
/// The caller is responsible for keeping the underlying FD owners (OwnedFd,
/// serial port handle, etc.) alive for the duration of the bridge.
///
/// # Example
/// ```no_run
/// use std::sync::Arc;
/// use std::sync::atomic::AtomicBool;
/// use elm327_core::bridge::Bridge;
///
/// let shutdown = Arc::new(AtomicBool::new(false));
/// // pty_fd and serial_fd come from PtyPair and serial port setup
/// let mut bridge = Bridge::new(3, 4); // placeholder FDs
/// // bridge.run(&shutdown).unwrap();
/// ```
pub struct Bridge {
    pty_fd: RawFd,
    serial_fd: RawFd,
    stats: BridgeStats,
}

impl Bridge {
    /// Create a new bridge from raw file descriptors.
    ///
    /// # Safety contract
    /// The caller must ensure both FDs remain valid (owned elsewhere) for
    /// the lifetime of this Bridge and any call to `run()`.
    pub fn new(pty_fd: RawFd, serial_fd: RawFd) -> Self {
        Self {
            pty_fd,
            serial_fd,
            stats: BridgeStats::default(),
        }
    }

    /// Run the bidirectional forwarding loop.
    ///
    /// Blocks until the shutdown flag is set or an unrecoverable error occurs.
    /// Returns `Ok(())` on clean shutdown, `Err(BridgeError)` on I/O failure.
    pub fn run(&mut self, shutdown: &Arc<AtomicBool>) -> Result<()> {
        log::info!(
            "Bridge started: pty_fd={} <-> serial_fd={}",
            self.pty_fd,
            self.serial_fd
        );

        let mut buf = [0u8; BUF_SIZE];

        loop {
            if shutdown.load(Ordering::Relaxed) {
                log::info!("Bridge shutdown requested");
                return Ok(());
            }

            // Safety: we borrow raw FDs that the caller guarantees are alive.
            let pty_borrow = unsafe { BorrowedFd::borrow_raw(self.pty_fd) };
            let serial_borrow = unsafe { BorrowedFd::borrow_raw(self.serial_fd) };

            let mut poll_fds = [
                PollFd::new(pty_borrow, PollFlags::POLLIN),
                PollFd::new(serial_borrow, PollFlags::POLLIN),
            ];

            // Poll with 100ms timeout so we re-check the shutdown flag regularly
            let n = poll(&mut poll_fds, PollTimeout::from(POLL_TIMEOUT_MS))
                .map_err(|e| BridgeError::Io(std::io::Error::from(e)))?;

            if n == 0 {
                continue; // timeout — loop back and check shutdown
            }

            // --- PTY -> Serial ---
            if let Some(revents) = poll_fds[0].revents() {
                if revents.contains(PollFlags::POLLIN) {
                    self.forward_pty_to_serial(&mut buf)?;
                }
                // TODO: consider reconnect logic instead of hard error on hangup
                if revents.contains(PollFlags::POLLHUP) || revents.contains(PollFlags::POLLERR) {
                    log::warn!("PTY error/hangup detected");
                    return Err(BridgeError::Io(std::io::Error::new(
                        std::io::ErrorKind::BrokenPipe,
                        "PTY hangup",
                    )));
                }
            }

            // --- Serial -> PTY ---
            if let Some(revents) = poll_fds[1].revents() {
                if revents.contains(PollFlags::POLLIN) {
                    self.forward_serial_to_pty(&mut buf)?;
                }
                if revents.contains(PollFlags::POLLHUP) || revents.contains(PollFlags::POLLERR) {
                    log::warn!("Serial error/hangup detected");
                    return Err(BridgeError::Io(std::io::Error::new(
                        std::io::ErrorKind::BrokenPipe,
                        "Serial hangup",
                    )));
                }
            }
        }
    }

    /// Read from PTY, write all bytes to serial. Updates stats.
    fn forward_pty_to_serial(&mut self, buf: &mut [u8]) -> Result<()> {
        match nix::unistd::read(self.pty_fd, buf) {
            Ok(0) => {
                log::warn!("PTY closed (EOF)");
                Err(BridgeError::Io(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "PTY closed",
                )))
            }
            Ok(n) => {
                log::debug!("PTY -> Serial: {} bytes: {:02X?}", n, &buf[..n]);
                self.write_all_raw(self.serial_fd, &buf[..n], "serial")?;
                self.stats.bytes_pty_to_serial += n as u64;
                self.stats.forward_count += 1;
                Ok(())
            }
            Err(nix::errno::Errno::EAGAIN | nix::errno::Errno::EINTR) => Ok(()),
            Err(e) => {
                log::error!("PTY read error: {}", e);
                self.stats.errors += 1;
                Err(BridgeError::Io(std::io::Error::from(e)))
            }
        }
    }

    /// Read from serial, write all bytes to PTY. Updates stats.
    fn forward_serial_to_pty(&mut self, buf: &mut [u8]) -> Result<()> {
        match nix::unistd::read(self.serial_fd, buf) {
            Ok(0) => {
                log::warn!("Serial closed (EOF)");
                Err(BridgeError::Io(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Serial closed",
                )))
            }
            Ok(n) => {
                log::debug!("Serial -> PTY: {} bytes: {:02X?}", n, &buf[..n]);
                self.write_all_raw(self.pty_fd, &buf[..n], "PTY")?;
                self.stats.bytes_serial_to_pty += n as u64;
                self.stats.forward_count += 1;
                Ok(())
            }
            Err(nix::errno::Errno::EAGAIN | nix::errno::Errno::EINTR) => Ok(()),
            Err(e) => {
                log::error!("Serial read error: {}", e);
                self.stats.errors += 1;
                Err(BridgeError::Io(std::io::Error::from(e)))
            }
        }
    }

    /// Write all bytes to a raw fd, retrying on short writes.
    /// `nix::unistd::write` takes `impl AsFd`, so we borrow the raw fd.
    fn write_all_raw(&mut self, fd: RawFd, data: &[u8], label: &str) -> Result<()> {
        let mut written = 0;
        while written < data.len() {
            let borrowed = unsafe { BorrowedFd::borrow_raw(fd) };
            match nix::unistd::write(borrowed, &data[written..]) {
                Ok(w) => written += w,
                Err(nix::errno::Errno::EINTR) => continue, // interrupted, retry
                Err(e) => {
                    log::error!("{} write error: {}", label, e);
                    self.stats.errors += 1;
                    return Err(BridgeError::Io(std::io::Error::from(e)));
                }
            }
        }
        Ok(())
    }

    /// Get current bridge statistics.
    pub fn stats(&self) -> &BridgeStats {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pty::PtyPair;
    use std::os::fd::AsRawFd;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    /// Create two PTY pairs and bridge their controllers.
    /// Write to pair1.device -> bridge -> pair2.device (simulates Wine <-> serial).
    fn make_bridged_pairs() -> (PtyPair, PtyPair) {
        let pair1 = PtyPair::create().expect("failed to create PTY pair 1");
        let pair2 = PtyPair::create().expect("failed to create PTY pair 2");
        (pair1, pair2)
    }

    /// Helper: poll an fd for readability with a timeout in ms.
    fn poll_readable(fd: RawFd, timeout_ms: u16) -> bool {
        let borrowed = unsafe { BorrowedFd::borrow_raw(fd) };
        let mut fds = [PollFd::new(borrowed, PollFlags::POLLIN)];
        let n = poll(&mut fds, PollTimeout::from(timeout_ms)).expect("poll failed");
        n > 0
    }

    #[test]
    fn test_bridge_forward() {
        // pair1.device (Wine side) -> pair1.controller -> bridge -> pair2.controller -> pair2.device (serial side)
        let (pair1, pair2) = make_bridged_pairs();
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = shutdown.clone();

        let pty_fd = pair1.controller.as_raw_fd();
        let serial_fd = pair2.controller.as_raw_fd();

        let bridge_handle = thread::spawn(move || {
            let mut bridge = Bridge::new(pty_fd, serial_fd);
            bridge.run(&shutdown_clone)
        });

        // Write "ATZ\r" to pair1's device side (simulating Wine writing)
        let msg = b"ATZ\r";
        nix::unistd::write(&pair1.device_fd, msg).expect("write to pair1.device failed");

        // Read from pair2's device side (simulating serial device reading)
        assert!(
            poll_readable(pair2.device_fd.as_raw_fd(), 1000),
            "pair2.device not readable within 1s"
        );

        let mut buf = [0u8; 64];
        let n = nix::unistd::read(pair2.device_fd.as_raw_fd(), &mut buf)
            .expect("read from pair2.device failed");

        assert_eq!(&buf[..n], msg, "forwarded data mismatch");

        // Shut it down
        shutdown.store(true, Ordering::Relaxed);
        bridge_handle
            .join()
            .expect("bridge thread panicked")
            .expect("bridge returned error");
    }

    #[test]
    fn test_bridge_bidirectional() {
        let (pair1, pair2) = make_bridged_pairs();
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = shutdown.clone();

        let pty_fd = pair1.controller.as_raw_fd();
        let serial_fd = pair2.controller.as_raw_fd();

        let bridge_handle = thread::spawn(move || {
            let mut bridge = Bridge::new(pty_fd, serial_fd);
            bridge.run(&shutdown_clone)
        });

        // Direction 1: pair1.device -> pair2.device
        let msg1 = b"ATZ\r";
        nix::unistd::write(&pair1.device_fd, msg1).expect("write fwd failed");

        assert!(poll_readable(pair2.device_fd.as_raw_fd(), 1000));
        let mut buf = [0u8; 64];
        let n = nix::unistd::read(pair2.device_fd.as_raw_fd(), &mut buf).expect("read fwd failed");
        assert_eq!(&buf[..n], msg1);

        // Direction 2: pair2.device -> pair1.device
        let msg2 = b"ELM327 v1.5\r\n>";
        nix::unistd::write(&pair2.device_fd, msg2).expect("write rev failed");

        assert!(poll_readable(pair1.device_fd.as_raw_fd(), 1000));
        let n = nix::unistd::read(pair1.device_fd.as_raw_fd(), &mut buf).expect("read rev failed");
        assert_eq!(&buf[..n], msg2);

        shutdown.store(true, Ordering::Relaxed);
        bridge_handle
            .join()
            .expect("bridge thread panicked")
            .expect("bridge returned error");
    }

    #[test]
    fn test_bridge_shutdown() {
        let (pair1, pair2) = make_bridged_pairs();
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = shutdown.clone();

        let pty_fd = pair1.controller.as_raw_fd();
        let serial_fd = pair2.controller.as_raw_fd();

        let bridge_handle = thread::spawn(move || {
            let mut bridge = Bridge::new(pty_fd, serial_fd);
            bridge.run(&shutdown_clone)
        });

        // Set shutdown almost immediately
        thread::sleep(Duration::from_millis(50));
        shutdown.store(true, Ordering::Relaxed);

        // Bridge should exit cleanly within ~100ms (one poll timeout cycle)
        let result = bridge_handle.join().expect("bridge thread panicked");
        assert!(result.is_ok(), "bridge should return Ok on clean shutdown");
    }

    #[test]
    fn test_bridge_stats() {
        let (pair1, pair2) = make_bridged_pairs();
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = shutdown.clone();

        let pty_fd = pair1.controller.as_raw_fd();
        let serial_fd = pair2.controller.as_raw_fd();

        // We need stats from the bridge thread, so we use a channel or just
        // return them. The run() method mutates self, so we return stats after.
        let bridge_handle = thread::spawn(move || {
            let mut bridge = Bridge::new(pty_fd, serial_fd);
            let result = bridge.run(&shutdown_clone);
            (result, bridge.stats().clone())
        });

        // Send data in both directions
        let fwd_msg = b"ATZ\r";
        nix::unistd::write(&pair1.device_fd, fwd_msg).expect("write fwd");
        assert!(poll_readable(pair2.device_fd.as_raw_fd(), 1000));
        let mut buf = [0u8; 64];
        nix::unistd::read(pair2.device_fd.as_raw_fd(), &mut buf).expect("read fwd");

        let rev_msg = b"OK\r\n>";
        nix::unistd::write(&pair2.device_fd, rev_msg).expect("write rev");
        assert!(poll_readable(pair1.device_fd.as_raw_fd(), 1000));
        nix::unistd::read(pair1.device_fd.as_raw_fd(), &mut buf).expect("read rev");

        // Small delay to let bridge process everything before shutdown
        thread::sleep(Duration::from_millis(50));
        shutdown.store(true, Ordering::Relaxed);

        let (result, stats) = bridge_handle.join().expect("bridge thread panicked");
        result.expect("bridge returned error");

        assert_eq!(
            stats.bytes_pty_to_serial,
            fwd_msg.len() as u64,
            "pty->serial byte count wrong"
        );
        assert_eq!(
            stats.bytes_serial_to_pty,
            rev_msg.len() as u64,
            "serial->pty byte count wrong"
        );
        assert_eq!(stats.forward_count, 2, "expected 2 forwards");
        assert_eq!(stats.errors, 0, "expected no errors");
    }
}
