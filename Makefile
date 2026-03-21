.PHONY: build rebuild clean smoke bridge dev
.PHONY: test test-unit test-pty test-serial test-bridge test-wine test-e2e
.PHONY: detect probe list-ports lint fmt

# Build
build:
	cargo build

rebuild:
	cargo clean && cargo build

clean:
	cargo clean

# Run
bridge:
	cargo run --bin elm327-bridge -- --config config.yml

dev:
	RUST_LOG=debug cargo run --bin elm327-bridge -- --config config.yml

smoke:
	@echo "=== Smoke Test ==="
	@echo "Checking Rust toolchain..."
	@rustc --version
	@cargo --version
	@echo "Building..."
	@cargo build 2>&1
	@echo "Checking for serial devices..."
	@ls /dev/cu.* 2>/dev/null || echo "No serial devices found (OK for dev)"
	@echo "Checking for Wine..."
	@which wine64 2>/dev/null || which wine 2>/dev/null || echo "Wine not found (OK for dev)"
	@echo "=== Smoke test passed ==="

# Testing (strict hierarchy - each level gates the next)
test:
	cargo test --workspace

test-unit:
	cargo test --workspace --lib

test-pty:
	cargo test --workspace pty_

test-serial:
	@if [ "$$SKIP_HARDWARE" = "1" ]; then echo "Skipping serial tests (SKIP_HARDWARE=1)"; exit 0; fi
	cargo test --workspace serial_

test-bridge:
	cargo test --workspace bridge_

test-wine:
	@if [ "$$SKIP_WINE" = "1" ]; then echo "Skipping wine tests (SKIP_WINE=1)"; exit 0; fi
	cargo test --workspace wine_

test-e2e:
	@echo "E2E tests require manual verification. See CLAUDE.md."
	@echo "Run: cargo test --workspace e2e_ -- --ignored"

# Device utilities
detect:
	cargo run --bin elm327-bridge -- --detect

probe:
	cargo run --bin elm327-bridge -- --probe

list-ports:
	@ls -la /dev/cu.* 2>/dev/null || echo "No serial devices found"

# Code quality
lint:
	cargo clippy --workspace -- -D warnings

fmt:
	cargo fmt --all
