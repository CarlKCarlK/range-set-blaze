# Just commands for local CI testing
# Install just: cargo install just
# Run: just --list (to see all commands)
# Run: just check-all (recommended before pushing)

# ============================================================================
# Main Commands (use these most often)
# ============================================================================

# Check everything locally before pushing: delegates to `cargo check-all` (xtask)
check-all:
    cargo check-all
    just fmt-check
    @echo "For additional WASM coverage, run: just test-wasm"

# Run all stable CI checks locally
ci: clippy test-stable

# Run all nightly CI checks locally
ci-nightly: test-nightly

# ============================================================================
# Individual Check Commands
# ============================================================================

# Run clippy with CI settings (matches CI exactly, using the pinned toolchain in rust-toolchain.toml)
clippy:
    cargo clippy --verbose --all-targets --features std -- -D clippy::all -A deprecated

# Preview lints on the newest stable toolchain, ignoring the pinned CI toolchain.
# Run this deliberately when evaluating a Rust-toolchain update; it is not part of
# the normal pinned CI path.
clippy-latest:
    rustup update stable
    cargo +stable clippy --verbose --all-targets --features std -- -D clippy::all -A deprecated

# Run all stable tests (matches CI)
test-stable:
    cargo test --verbose
    cargo test --verbose --release
    cargo test --verbose --no-default-features
    cargo test --verbose --release --no-default-features

# Run nightly tests with from_slice feature
# Uses `cargo +nightly` (not `rustup override set`) so this is safe to run
# concurrently with stable steps in the same directory (see `cargo check-all`).
test-nightly:
    cargo +nightly check --tests --features "float_nightly_experimental"
    cargo +nightly test --verbose --features from_slice
    cargo +nightly test --verbose --features cursor_nightly_experimental
    cargo +nightly test --verbose --no-default-features --features cursor_nightly_experimental
    cargo +nightly test --verbose --features insert_nightly_experimental
    cargo +nightly test --verbose --no-default-features --features insert_nightly_experimental
    cargo +nightly clippy --verbose --all-targets --features insert_nightly_experimental -- -D clippy::all -A deprecated
    cargo +nightly test --verbose --all-features

# Quick check before commit (clippy + basic tests)
pre-commit: clippy
    cargo test --verbose

# ============================================================================
# Quality & Publishing Checks
# ============================================================================

# Build and open the docs as docs.rs publishes them: all optional APIs are
# included, and rustdoc labels feature-gated items with their required feature.
# Nightly is required by the optional SIMD, float, cursor, and `doc_cfg` features.
show-docs:
    RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --no-deps --open --all-features

# Rebuild the docs (same features as `show-docs`) without opening a browser
update-docs:
    RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --no-deps --all-features

# Check documentation for dead links (requires cargo-deadlinks)
doc-links:
    cargo install cargo-deadlinks
    RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --no-deps --all-features
    RUST_LOG=error cargo deadlinks --dir target/doc/range_set_blaze

# Audit dependencies for security and license issues (requires cargo-audit and cargo-deny)
audit:
    cargo install cargo-audit cargo-deny
    cargo audit
    cargo deny check

# Test publishing without actually publishing
publish-dry:
    cargo publish --dry-run

# Test publishing with all features
publish-dry-all:
    cargo +nightly publish --all-features --dry-run

# Full local CI simulation (no WASM/embedded)
ci-full: clippy test-stable doc-links audit publish-dry-all

# ============================================================================
# Utilities
# ============================================================================

# Run the float_maps example in release mode (it's too slow in debug)
run-float-maps:
    cargo run --release --example float_maps

# Clean build artifacts
clean:
    cargo clean

# Format code
fmt:
    cargo fmt --all

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# ============================================================================
# MSRV (matches `rust-version` in Cargo.toml — run `rustup toolchain install 1.87` once)
# ============================================================================

# Check the crate still compiles on the declared MSRV (1.87)
msrv-check:
    cargo +1.87 check --verbose
    cargo +1.87 check --verbose --no-default-features
    cargo +1.87 check --verbose --features std

# ============================================================================
# WASM (wasip1 via wasmtime — the browser/Chrome lane is CI-only)
# ============================================================================

# WASM tests via wasmtime/wasip1 (full test suite, not just wasm-bindgen-test); skips Chrome/browser lane, which is CI-only
test-wasm:
    rustup target add wasm32-wasip1
    CARGO_TARGET_WASM32_WASIP1_RUNNER='wasmtime run --dir .' cargo test --target wasm32-wasip1 --verbose
    CARGO_TARGET_WASM32_WASIP1_RUNNER='wasmtime run --dir .' cargo test --target wasm32-wasip1 --verbose --no-default-features

# Portable stable float tests for local WSL runs.
test-floats-portable:
    cargo test --verbose --features std
    cargo test --verbose --release --features std
    rustup target add wasm32-wasip1
    CARGO_TARGET_WASM32_WASIP1_RUNNER='wasmtime run --dir .' cargo test --target wasm32-wasip1 --verbose --features std
    wasm-pack test --node -- --features std --verbose

# ============================================================================
# SIMD Feature (from_slice) - Requires Nightly
# ============================================================================

# Build with from_slice feature
build-simd:
    cargo +nightly build --features from_slice

# Test with from_slice feature
test-simd:
    cargo +nightly test --features from_slice
