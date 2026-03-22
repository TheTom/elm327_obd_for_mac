# CLAUDE.md

# Ford Diagnostic Tool — Autonomous Development Rules

Native macOS Ford Diagnostic Tool — a Rust CLI that communicates directly with ELM327 USB adapters for Ford-specific OBD-II and UDS diagnostics on Apple Silicon. No Wine, no Windows. See `PRD.md` for full product spec.

Core thesis:

Most Ford owners can't run FORScan on macOS without painful Wine hacks.
Build a native Rust tool that talks directly to ELM327 adapters and the truck's CAN bus.

This repository is developed using **test-driven, autonomous execution with strict guardrails**.

---

# Core Operating Principles

1. **Tests before implementation — mandatory.**
2. **Every piece of work must correspond to a GitHub Issue.**
3. Implement the **smallest working solution**.
4. **No unsafe unless justified** — if you write `unsafe`, comment why.
5. Keep architecture **simple and extensible**.
6. Keep the repository **compilable at all times**.
7. Maintain **organized documentation, repository maps, and test maps**.
8. Follow the **issue → failing tests → implementation → review → commit → close issue loop**.
9. Focus on **serial communication, ELM327 protocol, OBD-II, and Ford-specific diagnostics**.
10. If uncertain about a requirement, create a **GitHub Issue** and continue with other unblocked work.

---

# GitHub Issue-Driven Development (MANDATORY)

This project uses **GitHub Issues as the bug/task system**, modeled after Google Buganizer.

Every unit of work is treated as an issue.

This includes:

- features
- bugs
- implementation tasks
- investigations
- product questions
- refactors
- architectural decisions
- hardware compatibility findings

---

# Issue Requirement (MANDATORY)

Before starting any work:

1. Ensure a **GitHub Issue exists**
2. Assign the correct priority label
3. Assign the correct type label
4. Assign the correct area label(s)
5. Write failing tests
6. Only then may implementation begin

No task may be worked on without a corresponding issue.

---

# Priority Labels (Buganizer Style)

Use the following priorities:

- **P0** — blocking / critical / hardware-damaging potential / correctness failure
- **P1** — core product work (current phase)
- **P2** — normal development task
- **P3** — improvement / cleanup
- **P4** — polish / optional

Guidelines:

- P0 should be rare — reserve for things that could brick an ECU or corrupt CAN bus
- most work should be P1 or P2
- cleanup tasks are P3
- ideas or polish are P4

---

# Issue Labels

Issues should use labels for classification.

Priority labels:

- `P0`
- `P1`
- `P2`
- `P3`
- `P4`

Type labels:

- `type:feature`
- `type:bug`
- `type:task`
- `type:investigation`
- `type:question`
- `type:refactor`
- `type:hardware` — hardware compatibility finding or issue

Area labels:

- `area:serial` — serial port, TTYPort, macOS device handling
- `area:obd` — OBD-II protocol, PID decoding, DTC parsing, VIN
- `area:ford` — Ford module database, CAN address pairs, MS-CAN/HS-CAN
- `area:elm327` — ELM327 AT commands, protocol init, prompt handling
- `area:cli` — CLI binary, clap subcommands, user-facing output
- `area:simulator` — ELM327 simulator, PTY-based testing
- `area:bridge` — bridge forwarding, PTY ↔ serial
- `area:config` — YAML config, device detection settings
- `area:docs` — documentation, CLAUDE.md, PRD, DEVLOG

---

# Development Workflow (MANDATORY)

All work must follow this exact sequence.

1. Create or identify GitHub Issue
2. Confirm labels and priority are correct
3. Write **failing tests first**
4. Confirm tests fail for the expected reason
5. Implement the minimal code required
6. Run tests: `make test`
7. Run linter: `make lint`
8. Ensure compilation: `make build`
9. Run `codex-review`
10. Run `@roast review this change`
11. Fix any issues
12. Rerun tests and lint if code changed
13. Commit changes
14. Close the issue with resolution comments

No feature is complete without tests, compilation, linting, and review.

---

# Autonomous Execution Loop

When operating autonomously:

1. Review open GitHub Issues
2. Select the highest-priority unblocked issue
3. Write failing tests first
4. Implement the minimal solution
5. Run tests: `make test`
6. Run linter: `make lint`
7. Ensure compilation: `make build`
8. Run `codex-review`
9. Fix review findings
10. Run `@roast review this change`
11. Fix roast findings
12. Rerun tests, lint, and build if code changed
13. Commit changes
14. Close issue with resolution comments
15. Move to next issue

Do not skip the **tests-first** step.

---

# Definition of Done (MANDATORY)

A task or feature is complete **only when all conditions are met**.

1. A GitHub Issue exists and is the source of the work
2. Tests were written first
3. Tests pass (`make test`)
4. Code coverage thresholds pass
5. Code compiles successfully (`make build`)
6. Linting passes (`make lint` — clippy with `-D warnings`)
7. Codex review completed using `codex-review`
8. Roast agent signoff completed using `@roast`
9. Documentation updated if behavior, architecture, or tests changed
10. Issue closed with resolution comments

If any step is missing, the task is **not complete**.

---

# Mandatory Review Workflow

Before marking any feature or task as complete, code must go through the required review workflow below.

This is mandatory.

### Codex Review (Mandatory)

Use the `codex-review` skill for normal code review.

Examples:

- new AT command handlers
- PID decoding logic
- DTC parsing
- serial port initialization
- Ford module database changes
- CLI subcommand additions
- simulator response handling
- refactors and bug fixes

Do not skip `codex-review` just because tests pass.

### Roast Agent Final Signoff (Mandatory)

Before a feature is marked done, run the roast agent for final review.

Invocation:

```text
@roast review this change
```

Purpose:

- identify weak assumptions
- identify sloppy implementation details
- identify missing edge cases (especially CAN bus edge cases)
- identify architecture mistakes
- challenge whether the feature is actually done

If roast finds issues:

1. fix them
2. rerun tests
3. rerun lint
4. rerun build
5. rerun roast if the code changed

No feature is complete without roast signoff.

---

# Coverage Requirements (MANDATORY)

Run code coverage regularly and enforce it before considering work complete.

## Core Module Coverage

Target:

- **90%+** overall coverage for core modules: `obd.rs`, `ford.rs`, `elm327.rs`, `serial.rs`
- **100%** coverage for DTC decoding and PID formula calculations — these are pure math with known correct answers
- **80%+** coverage for `pty.rs`, `bridge.rs`, `config.rs`, `detect.rs`

Recommended command:

```bash
cargo tarpaulin --out Html --skip-clean
```

## Coverage Rules

- do not duplicate tests unnecessarily
- prefer meaningful coverage over shallow line coverage
- critical logic (DTC decode, PID formulas, VIN parsing) must have direct, obvious test ownership
- if coverage drops below threshold, the task is not done
- hardware-dependent code paths may use simulator-based coverage

---

# Build Integrity (MANDATORY)

The repository must always remain compilable.

Before committing, all three must pass:

```bash
cargo build          # compilation
cargo clippy -- -D warnings  # linting
cargo test -- --test-threads=1  # tests (single-threaded for PTY safety)
```

Or equivalently:

```bash
make build && make lint && make test
```

If any of these fail:

1. stop work
2. fix errors
3. rerun all three

Never leave the repository in a broken state.

---

# Commit Discipline

Commit frequently.

Commit after:

- passing tests
- significant logic changes
- refactors
- issue completion

Commit format:

```
type(scope): description
```

Examples:

```
feat(obd): add Mode 09 VIN multi-frame decoding
test(ford): add HS-CAN module address pair coverage
fix(elm327): handle missing prompt in ATZ response
refactor(serial): extract baud rate detection to separate fn
docs(claude): update development workflow
chore(ci): add clippy to pre-commit checks
```

Scopes: `obd`, `ford`, `elm327`, `serial`, `cli`, `simulator`, `bridge`, `config`, `pty`, `ci`, `docs`

Commits should be small, focused, and readable.

---

# Error Handling Philosophy

Errors must not be silently swallowed.

Errors should:

- fail fast
- produce clear log output with `tracing` or `log`
- include actionable messages (what failed, what was expected, what to check)
- be testable — use the unified `BridgeError` type
- propagate via `Result<T, BridgeError>` — no `.unwrap()` in library code

Serial errors specifically:

- log the raw bytes sent and received at `debug` level
- include device path in error messages
- include timeout duration when timeouts occur

---

# Failure Handling

If a task fails after three attempts:

1. stop retrying
2. document failure in `DEVLOG.md`
3. create or update the GitHub Issue
4. continue with other unblocked work

Do not loop indefinitely.

---

# Scope Control — Phase-Gated Development

This project follows strict phases (see `PRD.md` §7). Do NOT skip ahead.

1. **Phase 1 (Talk to the Truck)**: ELM327 protocol + OBD-II basics (VIN, PIDs, DTCs)
2. **Phase 2 (Ford Modules)**: Module scanning, per-module DTCs, firmware versions
3. **Phase 3 (Deep Diagnostics)**: As-Built reading, MS-CAN, full PID database
4. **Phase 4 (Config & GUI)**: As-Built writing, SwiftUI app, Homebrew

Do not implement Phase N+1 features unless Phase N is complete and explicitly assigned.

Ignore for now unless explicitly tasked:

- SwiftUI GUI
- As-Built writing (read-only first)
- Homebrew distribution
- Third-party adapter support beyond ELM327

---

# DEVLOG (MANDATORY)

Maintain:

```
DEVLOG.md
```

Use DEVLOG.md to record:

- implementation decisions
- blockers
- architectural changes
- failed approaches
- hardware findings (adapter quirks, baud rate issues, clone detection)
- review outcomes
- known limitations

Record meaningful notes, not noise.

---

# Codebase Map (MANDATORY)

Maintain:

```
docs/CODEBASE_MAP.md
```

This document describes:

- repository structure
- crate architecture and dependencies
- data flow (CLI → Engine → ELM327 → Serial → CAN Bus)
- module responsibilities
- test structure

Before implementing new work:

1. read the codebase map
2. determine the correct crate/module
3. avoid creating duplicate systems

Update the map when architecture changes.

---

# Test Map (MANDATORY)

Maintain:

```
docs/TEST_MAP.md
```

The test map should document:

- major modules and their test files
- the test hierarchy (unit → PTY → simulator → bridge → integration → hardware)
- important fixtures and mock adapters
- critical end-to-end flows

Update the test map whenever:

- a new major module is added
- a new critical test is added
- an important fixture changes

Purpose:

- avoid duplicate tests
- make coverage discoverable
- make onboarding and review easier

---

# Architecture

### Data Flow
```
ford-diag CLI → Diagnostic Engine → ELM327 Protocol → Serial → /dev/cu.* → Adapter → CAN Bus → Ford Modules
```

### Crate Structure
- `crates/elm327-core/` — Core library
  - `serial.rs` — macOS serial port (38400 8N1, TTYPort with AsRawFd)
  - `detect.rs` — Device enumeration, baud rate auto-detection
  - `elm327.rs` — ELM327 protocol (init, send/receive, prompt handling)
  - `obd.rs` — OBD-II (PID decoding, DTC parsing, VIN reading)
  - `ford.rs` — Ford module database (CAN address pairs, bus mapping)
  - `pty.rs` — PTY pair creation (used by simulator)
  - `bridge.rs` — Byte forwarding (used by simulator tests)
  - `config.rs` — YAML config loading
  - `error.rs` — Unified BridgeError type
  - `wine.rs` — Wine COM symlink management (legacy, may remove)
- `crates/ford-diag/` — CLI binary (clap subcommands)
- `crates/elm327-bridge/` — Bridge CLI (legacy Wine approach)
- `crates/elm327-simulator/` — Fake ELM327 for testing without hardware

### Configuration
All settings via `config.yml`:
- **Device**: `device` (default: auto-detect), `baud_rate` (default: 38400)
- **Behavior**: `logging`, `log_level`

---

# Development Commands

### Quick Start
```bash
make smoke          # Validate environment (Rust, serial device)
make build          # Build all crates
make clean          # Cargo clean
```

### Testing
```bash
make test           # Run all tests (single-threaded for PTY safety)
make test-unit      # Unit tests only (no hardware required)
make test-pty       # PTY creation + bidirectional data flow
make test-serial    # Serial device communication (requires adapter)
make test-bridge    # Bridge integration (PTY ↔ serial forwarding)
make test-e2e       # End-to-end: CLI → ELM327 → simulator
make lint           # Clippy with -D warnings
make fmt            # Format all code
```

### CLI Tool
```bash
cargo run --bin ford-diag -- detect          # Find OBD adapters
cargo run --bin ford-diag -- raw "ATZ"       # Send raw command
cargo run --bin ford-diag -- info            # Read VIN (Phase 1)
cargo run --bin ford-diag -- scan            # Scan Ford modules (Phase 2)
cargo run --bin ford-diag -- dtc             # Read DTCs
cargo run --bin ford-diag -- dtc --clear     # Clear DTCs
cargo run --bin ford-diag -- live            # Monitor live PIDs
```

### Device Utilities
```bash
make detect         # Auto-detect OBD adapters on /dev/cu.*
make probe          # Send ATZ to detected device, print response
make list-ports     # List all /dev/cu.* devices
```

---

# Testing Protocol

### Test Hierarchy (strict)
Every PR must pass tests in this order. A failure at any level blocks the next.

1. **Unit tests** — no I/O, no hardware. Pure logic (PID decoding, DTC parsing, config).
2. **PTY tests** — PTY pair creation + bidirectional data flow.
3. **Simulator tests** — full ELM327 command/response through simulator.
4. **Bridge tests** — PTY ↔ serial forwarding via bridge.
5. **Integration tests** — end-to-end: CLI → bridge → simulator pipeline.
6. **Hardware tests** — real adapter communication. **Skip if no adapter** (`SKIP_HARDWARE=1`).

### Test Rules
- Tests that require hardware MUST be skippable via `SKIP_HARDWARE=1`
- All tests MUST have timeouts (max 10s for unit, 30s for integration)
- Serial tests MUST clean up device handles on exit
- PTY tests MUST clean up file descriptors on exit
- No test may leave orphan processes
- Use `--test-threads=1` to prevent PTY fd exhaustion (ERANGE)

### What "passes" means
- `ATZ` → response contains `ELM327`
- PID decode: 0x0BB8 → 748 RPM (formula: (A*256+B)/4)
- DTC decode: 0x0300 → "P0300"
- VIN decode: multi-frame hex → 17-char ASCII string
- PTY round-trip latency < 5ms for 64-byte payload
- Bridge forwarding: zero data loss over 1000 round-trips

---

# Code Rules

- **Language**: Rust. No Python in production code.
- **No kernel extensions**: Everything runs in user space.
- **No unsafe unless justified**: If you write `unsafe`, add a comment explaining why.
- **Logging**: All I/O operations MUST be loggable at debug level.
- **Error handling**: Never silently swallow serial errors. Log + propagate.
- **Baud rate**: Always configurable, never hardcoded (default 38400).
- **Timeouts**: Every serial read/write MUST have a timeout. No blocking forever.
- **No `.unwrap()` in library code**: Use `?` and proper error types.
- **Comments**: Include TODO comments for future improvements. Add inline docs for complex bit manipulation (PID formulas, DTC byte packing).

---

# Hardware Safety

- **Never send raw bytes to the adapter without logging them first.**
- Do not attempt ECU writes or flash operations without explicit user confirmation.
- Default to read-only diagnostic commands (AT*, Mode 01/03/09).
- If the adapter stops responding, back off — do not spam retries.
- **NEVER toggle MS/HS-CAN switch while actively communicating.**

---

# Target Vehicle

**2017 Ford F-150 EcoBoost 3.5L V6 Twin Turbo**
- 22 modules (20 HS-CAN, 2 MS-CAN)
- PCM: HL3A-12A650-BBB / HL3A-12B565-GB
- FORScan profile with full module/firmware mapping in `data/`

---

# Verified Hardware (2026-03-21)

```
/dev/cu.usbserial-110    # macOS built-in CDC driver (Apple Silicon, no WCH driver needed)
```

- **Adapter**: ELM327 USB with MS-CAN/HS-CAN toggle (CH340T, PIC18F25K80)
- **Baud rate**: 38400 (factory default, confirmed)
- **Version**: ELM327 v1.5 (good PIC clone, full AT command set)
- **ATPPS**: Full table returned (not a bad ARM clone)

---

# Common ELM327 AT Commands (Reference)

```
ATZ     → Reset, returns adapter ID (e.g., "ELM327 v1.5")
ATI     → Adapter version info
ATE0    → Echo off
ATL0    → Linefeeds off
ATH1    → Headers on (show CAN IDs in responses)
ATS0    → Spaces off
ATAT1   → Adaptive timing on
ATSP6   → Set protocol: CAN 11-bit 500kbps (Ford HS-CAN)
ATSPB   → Set protocol: User CAN (for MS-CAN at 125kbps)
ATSH7E0 → Set header to PCM request address
ATCRA7E8 → Filter responses to PCM only
0100    → Supported PIDs [01-20]
010C    → Engine RPM
010D    → Vehicle speed
03      → Read DTCs
04      → Clear DTCs
0902    → Read VIN
```

---

# Summary

The required development loop for this repository is:

```
create or identify GitHub Issue
↓
write failing tests
↓
implement minimal solution
↓
tests pass + clippy passes + cargo build passes
↓
codex-review
↓
@roast review this change
↓
commit (conventional format)
↓
close GitHub Issue with resolution comments
```

No shortcuts.
