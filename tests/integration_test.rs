//! End-to-end integration tests: FORScan (simulated) -> Bridge -> ELM327 Simulator
//!
//! Architecture:
//! ```text
//! Wine PTY pair:    wine_controller <-> wine_device
//!                        |
//!                     Bridge
//!                        |
//! Sim PTY pair:     sim_device <-> sim_controller
//!                                       |
//!                                   Simulator
//! ```
//!
//! - Bridge reads/writes wine_controller (pty_fd) and sim_device (serial_fd)
//! - Simulator reads/writes sim_controller (via std::fs::File)
//! - Test reads/writes wine_device (simulating FORScan)

use std::os::fd::{AsRawFd, BorrowedFd, FromRawFd, RawFd};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use elm327_core::bridge::Bridge;
use elm327_core::pty::PtyPair;
use elm327_simulator::elm327_sim::Elm327Simulator;
use nix::poll::{poll, PollFd, PollFlags, PollTimeout};

/// Max time to wait for a single command response.
const CMD_TIMEOUT_MS: i32 = 5000;

/// Max time for an entire test before we panic.
const TEST_TIMEOUT: Duration = Duration::from_secs(10);

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Poll an fd for readability with a timeout in ms.
fn poll_readable(fd: RawFd, timeout_ms: i32) -> bool {
    let borrowed = unsafe { BorrowedFd::borrow_raw(fd) };
    let mut fds = [PollFd::new(borrowed, PollFlags::POLLIN)];
    // Clamp timeout to u16 range for PollTimeout, or use raw poll for larger values
    let n = if timeout_ms <= u16::MAX as i32 {
        poll(&mut fds, PollTimeout::from(timeout_ms as u16)).expect("poll failed")
    } else {
        poll(&mut fds, PollTimeout::from(u16::MAX)).expect("poll failed")
    };
    n > 0
}

/// Send a command (appending \r) and read back the full response until the '>' prompt.
/// Returns the raw response string including echo and prompt.
fn send_and_receive(write_fd: RawFd, read_fd: RawFd, cmd: &str, timeout_ms: i32) -> String {
    // Write command + \r
    let mut payload = cmd.as_bytes().to_vec();
    payload.push(b'\r');
    let borrowed_w = unsafe { BorrowedFd::borrow_raw(write_fd) };
    nix::unistd::write(borrowed_w, &payload).expect("write command failed");

    // Read until we see '>' prompt
    let mut response = Vec::new();
    let deadline = Instant::now() + Duration::from_millis(timeout_ms as u64);

    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            panic!(
                "Timeout waiting for '>' prompt after sending {:?}. Got so far: {:?}",
                cmd,
                String::from_utf8_lossy(&response)
            );
        }

        let remaining_ms = remaining.as_millis().min(u16::MAX as u128) as i32;
        if !poll_readable(read_fd, remaining_ms) {
            continue;
        }

        let mut buf = [0u8; 1024];
        match nix::unistd::read(read_fd, &mut buf) {
            Ok(0) => panic!("EOF while waiting for response to {:?}", cmd),
            Ok(n) => {
                response.extend_from_slice(&buf[..n]);
                // Check if we have the prompt
                if response.contains(&b'>') {
                    break;
                }
            }
            Err(nix::errno::Errno::EAGAIN | nix::errno::Errno::EINTR) => continue,
            Err(e) => panic!("read error: {}", e),
        }
    }

    String::from_utf8_lossy(&response).to_string()
}

/// Strip echo and trailing prompt from a response.
/// After ATE0 the echo won't be present, but we handle both cases.
fn strip_response(resp: &str) -> String {
    // Remove trailing '>' and whitespace
    let s = resp.trim_end_matches('>').trim();
    s.to_string()
}

/// All-in-one test harness: creates PTY pairs, starts simulator + bridge threads,
/// returns the FDs for the test to read/write on, plus shutdown handle.
struct TestHarness {
    /// FD the test writes commands to and reads responses from (wine device side).
    forscan_fd: RawFd,
    shutdown: Arc<AtomicBool>,
    // Hold ownership so FDs stay alive for the duration of the test.
    // The bridge and simulator threads borrow raw FDs from these.
    _wine_pty: PtyPair,
    _sim_pty: PtyPair,
    _sim_thread: Option<thread::JoinHandle<()>>,
    _bridge_thread: Option<thread::JoinHandle<()>>,
}

impl TestHarness {
    fn new() -> Self {
        let wine_pty = PtyPair::create().expect("failed to create Wine PTY pair");
        let sim_pty = PtyPair::create().expect("failed to create Sim PTY pair");

        let shutdown = Arc::new(AtomicBool::new(false));

        // Grab raw FDs before we move anything
        let forscan_fd = wine_pty.device_fd.as_raw_fd();
        let bridge_pty_fd = wine_pty.controller.as_raw_fd(); // wine controller
        let bridge_serial_fd = sim_pty.device_fd.as_raw_fd(); // sim device

        // The simulator needs a File wrapping the sim controller fd.
        // We dup() it so the OwnedFd in sim_pty stays valid and the File gets its own fd.
        // Set to non-blocking so the simulator can check the shutdown flag between reads.
        let sim_controller_raw = sim_pty.controller.as_raw_fd();
        let sim_file_fd = nix::unistd::dup(sim_controller_raw).expect("dup sim controller failed");
        // Set non-blocking mode on the dup'd fd
        {
            use nix::fcntl::{fcntl, FcntlArg, OFlag};
            let flags = fcntl(sim_file_fd, FcntlArg::F_GETFL).expect("F_GETFL");
            let mut oflags = OFlag::from_bits_truncate(flags);
            oflags.insert(OFlag::O_NONBLOCK);
            fcntl(sim_file_fd, FcntlArg::F_SETFL(oflags)).expect("F_SETFL");
        }
        let sim_file = unsafe { std::fs::File::from_raw_fd(sim_file_fd) };

        // Start simulator thread
        let sim_shutdown = shutdown.clone();
        let sim_thread = thread::spawn(move || {
            let mut sim = Elm327Simulator::new();
            let mut file = sim_file;
            if let Err(e) = sim.run(&mut file, &sim_shutdown) {
                // BrokenPipe is expected on shutdown
                if e.kind() != std::io::ErrorKind::BrokenPipe {
                    eprintln!("Simulator error: {}", e);
                }
            }
        });

        // Start bridge thread
        let bridge_shutdown = shutdown.clone();
        let bridge_thread = thread::spawn(move || {
            let mut bridge = Bridge::new(bridge_pty_fd, bridge_serial_fd);
            if let Err(e) = bridge.run(&bridge_shutdown) {
                // BrokenPipe is expected on shutdown
                let msg = format!("{}", e);
                if !msg.contains("hangup") && !msg.contains("Broken pipe") {
                    eprintln!("Bridge error: {}", e);
                }
            }
        });

        // Give the threads a moment to spin up
        thread::sleep(Duration::from_millis(50));

        Self {
            forscan_fd,
            shutdown,
            _wine_pty: wine_pty,
            _sim_pty: sim_pty,
            _sim_thread: Some(sim_thread),
            _bridge_thread: Some(bridge_thread),
        }
    }

    /// Send a command and get the response.
    fn send(&self, cmd: &str) -> String {
        send_and_receive(self.forscan_fd, self.forscan_fd, cmd, CMD_TIMEOUT_MS)
    }

    /// Shut down the simulator and bridge cleanly.
    fn shutdown(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
    }
}

impl Drop for TestHarness {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        // Wait briefly for threads to finish — don't block forever on drop
        if let Some(h) = self._bridge_thread.take() {
            let _ = h.join();
        }
        if let Some(h) = self._sim_thread.take() {
            let _ = h.join();
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_e2e_atz_through_bridge() {
    let start = Instant::now();
    let harness = TestHarness::new();

    let resp = harness.send("ATZ");
    assert!(
        start.elapsed() < TEST_TIMEOUT,
        "Test exceeded timeout"
    );

    let text = strip_response(&resp);
    assert!(
        text.contains("ELM327 v1.5"),
        "ATZ response should contain 'ELM327 v1.5', got: {:?}",
        text
    );

    harness.shutdown();
}

#[test]
fn test_e2e_forscan_init_sequence() {
    let start = Instant::now();
    let harness = TestHarness::new();

    // ATZ — reset, expect version string
    let resp = harness.send("ATZ");
    let text = strip_response(&resp);
    assert!(
        text.contains("ELM327 v1.5"),
        "ATZ failed: {:?}",
        text
    );

    // ATE0 — echo off (response still echoes this one)
    let resp = harness.send("ATE0");
    let text = strip_response(&resp);
    assert!(text.contains("OK"), "ATE0 failed: {:?}", text);

    // ATL0 — linefeed off (no echo from here on)
    let resp = harness.send("ATL0");
    let text = strip_response(&resp);
    assert!(text.contains("OK"), "ATL0 failed: {:?}", text);

    // ATH1 — headers on
    let resp = harness.send("ATH1");
    let text = strip_response(&resp);
    assert!(text.contains("OK"), "ATH1 failed: {:?}", text);

    // ATS0 — spaces off
    let resp = harness.send("ATS0");
    let text = strip_response(&resp);
    assert!(text.contains("OK"), "ATS0 failed: {:?}", text);

    // ATAT1 — adaptive timing
    let resp = harness.send("ATAT1");
    let text = strip_response(&resp);
    assert!(text.contains("OK"), "ATAT1 failed: {:?}", text);

    // ATSP6 — set protocol to CAN 11/500
    let resp = harness.send("ATSP6");
    let text = strip_response(&resp);
    assert!(text.contains("OK"), "ATSP6 failed: {:?}", text);

    assert!(
        start.elapsed() < TEST_TIMEOUT,
        "Test exceeded timeout"
    );

    harness.shutdown();
}

#[test]
fn test_e2e_obd_pid_through_bridge() {
    let start = Instant::now();
    let harness = TestHarness::new();

    // Disable echo first so responses are cleaner
    let _ = harness.send("ATZ");
    let _ = harness.send("ATE0");

    // Send OBD PID 0100 — supported PIDs
    let resp = harness.send("0100");
    let text = strip_response(&resp);
    assert!(
        text.contains("41 00"),
        "0100 response should contain '41 00', got: {:?}",
        text
    );

    assert!(
        start.elapsed() < TEST_TIMEOUT,
        "Test exceeded timeout"
    );

    harness.shutdown();
}

#[test]
fn test_e2e_roundtrip_integrity() {
    let start = Instant::now();
    let harness = TestHarness::new();

    // Init: reset + echo off for cleaner parsing
    let _ = harness.send("ATZ");
    let _ = harness.send("ATE0");

    // Send 100 commands and verify no data corruption
    // Mix of AT commands and OBD PIDs
    let commands: Vec<(&str, &str)> = vec![
        ("ATI", "ELM327 v1.5"),
        ("ATRV", "12.6V"),
        ("0100", "41 00"),
        ("010C", "41 0C"),
        ("010D", "41 0D"),
    ];

    for i in 0..100 {
        let (cmd, expected) = commands[i % commands.len()];
        let resp = harness.send(cmd);
        let text = strip_response(&resp);
        assert!(
            text.contains(expected),
            "Command #{} {:?}: expected {:?} in response, got: {:?}",
            i,
            cmd,
            expected,
            text
        );

        // Bail if we're taking too long
        assert!(
            start.elapsed() < TEST_TIMEOUT,
            "Test exceeded timeout at iteration {}",
            i
        );
    }

    harness.shutdown();
}
