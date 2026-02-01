# Just commands for local CI testing
# Install just: cargo install just
# Run: just --list (to see all commands)
# Run: just check-all (recommended before pushing)

# ============================================================================
# Main Commands (use these most often)
# ============================================================================

# Check everything locally before pushing (recommended!)
check-all: clippy test-stable test-nightly fmt-check

# Run all stable CI checks locally
ci: clippy test-stable

# Run all nightly CI checks locally
ci-nightly: test-nightly

# ============================================================================
# Individual Check Commands
# ============================================================================

# Run clippy with CI settings (matches CI exactly)
clippy:
    cargo clippy --verbose --all-targets --features "std rog_experimental" -- -D clippy::all -A deprecated

# Run all stable tests (matches CI)
test-stable:
    cargo test --verbose
    cargo test --verbose --release
    cargo test --verbose --no-default-features --features "rog_experimental"

# Run nightly tests with from_slice feature
test-nightly:
    rustup override set nightly
    cargo test --verbose --features "rog_experimental from_slice"
    cargo test --verbose --all-features
    rustup override set stable

# Quick check before commit (clippy + basic tests)
pre-commit: clippy
    cargo test --verbose

# ============================================================================
# Quality & Publishing Checks
# ============================================================================

# Check documentation for dead links (requires cargo-deadlinks)
doc-links:
    cargo install cargo-deadlinks
    cargo doc --no-deps --all-features
    cargo deadlinks --dir target/doc | grep -vE '(help\.html|settings\.html)'

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
    cargo publish --all-features --dry-run

# Full local CI simulation (stable only, no WASM/embedded)
ci-full: clippy test-stable doc-links audit publish-dry-all

# ============================================================================
# Utilities
# ============================================================================

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
# SIMD Feature (from_slice) - Requires Nightly
# ============================================================================

# Build with from_slice feature
build-simd:
    rustup override set nightly
    cargo build --features from_slice
    rustup override set stable

# Test with from_slice feature
test-simd:
    rustup override set nightly
    cargo test --features from_slice
    rustup override set stable
