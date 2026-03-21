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
const CMD_TIMEOUT_MS: u64 = 5000;

/// Max time for an entire test before we panic.
const TEST_TIMEOUT: Duration = Duration::from_secs(10);

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Poll an fd for readability with a timeout in ms.
fn poll_readable(fd: RawFd, timeout_ms: u16) -> bool {
    let borrowed = unsafe { BorrowedFd::borrow_raw(fd) };
    let mut fds = [PollFd::new(borrowed, PollFlags::POLLIN)];
    let n = poll(&mut fds, PollTimeout::from(timeout_ms)).expect("poll failed");
    n > 0
}

/// Send a command (appending \r) and read back the full response until the '>' prompt.
/// Returns the raw response string including echo and prompt.
fn send_and_receive(fd: RawFd, cmd: &str) -> String {
    // Write command + \r
    let mut payload = cmd.as_bytes().to_vec();
    payload.push(b'\r');
    let borrowed_w = unsafe { BorrowedFd::borrow_raw(fd) };
    nix::unistd::write(borrowed_w, &payload).expect("write command failed");

    // Read until we see '>' prompt
    let mut response = Vec::new();
    let deadline = Instant::now() + Duration::from_millis(CMD_TIMEOUT_MS);

    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            panic!(
                "Timeout waiting for '>' prompt after sending {:?}. Got so far: {:?}",
                cmd,
                String::from_utf8_lossy(&response)
            );
        }

        let remaining_ms = remaining.as_millis().min(u16::MAX as u128) as u16;
        if !poll_readable(fd, remaining_ms) {
            continue;
        }

        let mut buf = [0u8; 1024];
        match nix::unistd::read(fd, &mut buf) {
            Ok(0) => panic!("EOF while waiting for response to {:?}", cmd),
            Ok(n) => {
                response.extend_from_slice(&buf[..n]);
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
fn strip_response(resp: &str) -> String {
    resp.trim_end_matches('>').trim().to_string()
}

/// All-in-one test harness: creates PTY pairs, starts simulator + bridge threads,
/// provides an fd for the test to read/write on (simulating FORScan).
///
/// On drop, signals shutdown and closes the simulator's fd to unblock its
/// blocking read, then joins both threads.
struct TestHarness {
    /// FD the test writes commands to and reads responses from (wine device side).
    forscan_fd: RawFd,
    shutdown: Arc<AtomicBool>,
    // Hold ownership so FDs stay alive for the duration of the test.
    _wine_pty: PtyPair,
    _sim_pty: PtyPair,
    sim_thread: Option<thread::JoinHandle<()>>,
    bridge_thread: Option<thread::JoinHandle<()>>,
}

impl TestHarness {
    fn new() -> Self {
        let wine_pty = PtyPair::create().expect("failed to create Wine PTY pair");
        let sim_pty = PtyPair::create().expect("failed to create Sim PTY pair");

        let shutdown = Arc::new(AtomicBool::new(false));

        let forscan_fd = wine_pty.device_fd.as_raw_fd();
        let bridge_pty_fd = wine_pty.controller.as_raw_fd();
        let bridge_serial_fd = sim_pty.device_fd.as_raw_fd();

        // dup() the sim controller so the simulator gets its own fd via File.
        let sim_controller_raw = sim_pty.controller.as_raw_fd();
        let sim_file_fd = nix::unistd::dup(sim_controller_raw).expect("dup sim controller failed");
        let sim_file = unsafe { std::fs::File::from_raw_fd(sim_file_fd) };

        // Start simulator thread
        let sim_shutdown = shutdown.clone();
        let sim_thread = thread::spawn(move || {
            let mut sim = Elm327Simulator::new();
            let mut file = sim_file;
            // Simulator blocks on read() until data arrives or fd is closed.
            let _ = sim.run(&mut file, &sim_shutdown);
        });

        // Start bridge thread
        let bridge_shutdown = shutdown.clone();
        let bridge_thread = thread::spawn(move || {
            let mut bridge = Bridge::new(bridge_pty_fd, bridge_serial_fd);
            let _ = bridge.run(&bridge_shutdown);
        });

        // Let threads initialize
        thread::sleep(Duration::from_millis(50));

        Self {
            forscan_fd,
            shutdown,
            _wine_pty: wine_pty,
            _sim_pty: sim_pty,
            sim_thread: Some(sim_thread),
            bridge_thread: Some(bridge_thread),
        }
    }

    /// Send a command and get the response.
    fn send(&self, cmd: &str) -> String {
        send_and_receive(self.forscan_fd, cmd)
    }
}

impl Drop for TestHarness {
    fn drop(&mut self) {
        // Signal shutdown
        self.shutdown.store(true, Ordering::SeqCst);

        // The simulator is likely blocked on a read() call on the dup'd sim controller fd.
        // We send a dummy byte through the sim PTY's device (slave) side to unblock it.
        // The simulator will read the byte, loop back, check shutdown, and exit.
        // We write to the sim_pty device_fd, which feeds into the sim controller.
        let sim_device_fd = self._sim_pty.device_fd.as_raw_fd();
        let borrowed = unsafe { BorrowedFd::borrow_raw(sim_device_fd) };
        let _ = nix::unistd::write(borrowed, b"\r");

        // Join threads and check for panics.
        // If a background thread panicked but the test itself passed, propagate the panic.
        // Don't double-panic if we're already unwinding from a test assertion failure.
        if let Some(h) = self.bridge_thread.take() {
            if let Err(e) = h.join() {
                if !std::thread::panicking() {
                    std::panic::resume_unwind(e);
                }
            }
        }
        if let Some(h) = self.sim_thread.take() {
            if let Err(e) = h.join() {
                if !std::thread::panicking() {
                    std::panic::resume_unwind(e);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Send ATZ through bridge -> simulator, verify "ELM327 v1.5" response.
#[test]
fn test_e2e_atz_through_bridge() {
    let start = Instant::now();
    let _harness = TestHarness::new();

    let resp = _harness.send("ATZ");
    assert!(start.elapsed() < TEST_TIMEOUT, "Test exceeded timeout");

    let text = strip_response(&resp);
    assert!(
        text.contains("ELM327 v1.5"),
        "ATZ response should contain 'ELM327 v1.5', got: {:?}",
        text
    );
}

/// Run the full FORScan init: ATZ, ATE0, ATL0, ATH1, ATS0, ATAT1, ATSP6.
/// Verify each response is "OK" (except ATZ which returns version).
#[test]
fn test_e2e_forscan_init_sequence() {
    let start = Instant::now();
    let _harness = TestHarness::new();

    // ATZ -- reset, expect version string
    let resp = _harness.send("ATZ");
    let text = strip_response(&resp);
    assert!(text.contains("ELM327 v1.5"), "ATZ failed: {:?}", text);

    // ATE0 -- echo off (response still echoes this one)
    let resp = _harness.send("ATE0");
    let text = strip_response(&resp);
    assert!(text.contains("OK"), "ATE0 failed: {:?}", text);

    // ATL0 -- linefeed off (no echo from here on)
    let resp = _harness.send("ATL0");
    let text = strip_response(&resp);
    assert!(text.contains("OK"), "ATL0 failed: {:?}", text);

    // ATH1 -- headers on
    let resp = _harness.send("ATH1");
    let text = strip_response(&resp);
    assert!(text.contains("OK"), "ATH1 failed: {:?}", text);

    // ATS0 -- spaces off
    let resp = _harness.send("ATS0");
    let text = strip_response(&resp);
    assert!(text.contains("OK"), "ATS0 failed: {:?}", text);

    // ATAT1 -- adaptive timing
    let resp = _harness.send("ATAT1");
    let text = strip_response(&resp);
    assert!(text.contains("OK"), "ATAT1 failed: {:?}", text);

    // ATSP6 -- set protocol to CAN 11/500
    let resp = _harness.send("ATSP6");
    let text = strip_response(&resp);
    assert!(text.contains("OK"), "ATSP6 failed: {:?}", text);

    assert!(start.elapsed() < TEST_TIMEOUT, "Test exceeded timeout");
}

/// Send 0100 through bridge, verify response contains "41 00".
#[test]
fn test_e2e_obd_pid_through_bridge() {
    let start = Instant::now();
    let _harness = TestHarness::new();

    // Disable echo first so responses are cleaner
    let _ = _harness.send("ATZ");
    let _ = _harness.send("ATE0");

    // Send OBD PID 0100 -- supported PIDs
    let resp = _harness.send("0100");
    let text = strip_response(&resp);
    assert!(
        text.contains("41 00"),
        "0100 response should contain '41 00', got: {:?}",
        text
    );

    assert!(start.elapsed() < TEST_TIMEOUT, "Test exceeded timeout");
}

/// Send 100 commands, verify no data corruption across the full pipeline.
#[test]
fn test_e2e_roundtrip_integrity() {
    let start = Instant::now();
    let _harness = TestHarness::new();

    // Init: reset + echo off for cleaner parsing
    let _ = _harness.send("ATZ");
    let _ = _harness.send("ATE0");

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
        let resp = _harness.send(cmd);
        let text = strip_response(&resp);
        assert!(
            text.contains(expected),
            "Command #{} {:?}: expected {:?} in response, got: {:?}",
            i,
            cmd,
            expected,
            text
        );

        assert!(
            start.elapsed() < TEST_TIMEOUT,
            "Test exceeded timeout at iteration {}",
            i
        );
    }
}
