# Contributing to range-set-blaze

Thank you for your interest in contributing! This document provides information for developers working on the project.

## Development Prerequisites

- **Rust Stable**: For main development
- **Rust Nightly**: For `from_slice` SIMD feature
- **Just**: Task runner for local CI checks - Install: `cargo install just`
- **Optional Tools** (installed automatically by relevant commands):
  - `cargo-deadlinks`: For documentation link checking
  - `cargo-audit`: For security audits
  - `cargo-deny`: For license compliance

## Quick Start

```bash
# Clone the repository
git clone https://github.com/CarlKCarlK/range-set-blaze.git
cd range-set-blaze

# Install just (if not already installed)
cargo install just

# See all available commands
just --list

# Run all checks before pushing (recommended!)
just check-all
```

## Local Testing with Just

We use **[Just](https://github.com/casey/just)** for local CI testing. This ensures your changes pass CI checks before pushing.

### Most Common Commands

```bash
# Before pushing - run ALL checks (includes nightly tests)
just check-all

# Quick check before committing (stable only, faster)
just pre-commit

# Run only stable CI checks (what CI runs on stable)
just ci

# Run clippy with exact CI settings
just clippy

# Run all stable tests
just test-stable

# Run nightly tests (includes from_slice feature)
just test-nightly
```

### SIMD Feature Commands

The `from_slice` feature requires nightly Rust and uses SIMD:

```bash
# Build with SIMD feature
just build-simd

# Test with SIMD feature
just test-simd
```

### Quality Checks

```bash
# Check documentation for broken links
just doc-links

# Audit dependencies for security & licenses
just audit

# Test publishing (dry run)
just publish-dry-all

# Full CI simulation (everything except WASM/embedded)
just ci-full
```

### Utilities

```bash
# Format code
just fmt

# Check formatting
just fmt-check

# Clean build artifacts
just clean
```

## Testing Locally Before CI

**Always run `just check-all` or at minimum `just ci` before pushing!**

This catches:
- Clippy lints that fail CI with `-D clippy::all`
- Test failures across different configurations
- Formatting issues
- Documentation problems

### Why Local Testing Matters

CI runs strict checks that regular `cargo test` doesn't:
- **Clippy errors**: `cargo clippy -- -D clippy::all` treats ALL clippy warnings as errors
- **Multiple configurations**: Tests with different feature combinations
- **Release mode**: Tests in both debug and release
- **Documentation**: Checks for broken links and missing docs

Running `just check-all` replicates these checks locally, saving you from CI failures.

## Understanding the CI Pipeline

Our CI (`.github/workflows/ci.yml`) tests:

1. **3 OS platforms**: Ubuntu, macOS, Windows
2. **Multiple architectures**: 64-bit and 32-bit
3. **WASM targets**: wasm32-unknown-unknown and wasm32-wasip1
4. **Embedded**: thumbv7m-none-eabi (ARM Cortex-M)
5. **Both toolchains**: Stable and Nightly
6. **Security**: Dependency audits with cargo-audit and cargo-deny

You can't replicate everything locally (WASM, embedded, multiple OSes), but `just check-all` covers the main issues.

## Pull Request Process

1. **Create a feature branch**: `git checkout -b feature/my-feature`
2. **Make your changes**
3. **Run local checks**: `just check-all`
4. **Fix any issues** and commit
5. **Push and create PR**: `git push origin feature/my-feature`
6. **Wait for CI** to pass on GitHub
7. **Address review feedback** if any
8. **Merge** once approved and CI passes

## Code Style

- Run `just fmt` before committing
- Follow existing code patterns
- Add documentation for public APIs
- Write tests for new functionality

## Feature Flags

The project has several optional features:

- `std` (default): Standard library support
- `from_slice`: SIMD-accelerated slice ingestion (nightly only)
- `rog_experimental`: Experimental ROG (Range-of-Gaps) feature
- `test_util`: Testing utilities (dev only)

## Release Process

(For maintainers)

See the internal release notes, but in brief:

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Run `just ci-full` locally
4. Run `just publish-dry-all`
5. Create and merge PR
6. Tag release: `git tag v0.x.y`
7. Push tag: `git push origin v0.x.y`
8. Publish: `cargo publish`

## Getting Help

- **Issues**: [GitHub Issues](https://github.com/CarlKCarlK/range-set-blaze/issues)
- **Discussions**: [GitHub Discussions](https://github.com/CarlKCarlK/range-set-blaze/discussions)
- **Documentation**: [docs.rs](https://docs.rs/range-set-blaze)

## License

By contributing, you agree that your contributions will be dual-licensed under MIT OR Apache-2.0.
