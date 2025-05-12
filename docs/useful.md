# Useful commands for this project

## coverage

```cmd
set BUILDFEATURES=from_slice
rustup override set nightly
cargo llvm-cov --tests --all-features --open
cargo llvm-cov --tests --all-features --open --release

```

---------

## Benchmark Related

### Benchmarking (but not SIMD)

### Turn off stuff

- Backblaze, etc

```cmd
sudo sc config "WSearch" start= disabled
sudo net stop "WSearch"
# later
sudo sc config "WSearch" start= delayed-auto
sudo net start "WSearch"
```

### Set up means

```cmd
cargo install criterion-means
set BUILDFEATURES=from_slice
set RUSTFLAGS=-C target-cpu=native
rustup override set nightly
```

### bench

```cmd
cls & bench map_insert_speed & cargo criterion-means | findstr map_insert_speed
```

Look at `benchmarksApril2025.xlsx'

Slice

```cmd
rustup override set nightly
set BUILDFEATURES=from_slice
bench.bat
cargo criterion-means > delme3.csv
```

Non-Slice

```cmd
rustup override set stable
set RUSTFLAGS=-C target-cpu=native
set BUILDFEATURES=
bench.bat
cargo criterion-means > delme4.csv
```

Map

```cmd
rustup override set stable
set RUSTFLAGS=-C target-cpu=native
set BUILDFEATURES=
bench.bat map_
cargo criterion-means > delme5.csv
```

### Run criterion-means

```cmd
cargo install cargo-criterion-means --version 0.1.0-beta.3
set SIMD_LANES=64
set SIMD_INTEGER=i16
set RUSTFLAGS=-C target-feature=+avx512f
cargo criterion-means ..\..\.. > delme.csv
```

### run packages

```cmd
cargo run --package criterion-means ..\..\..
```

### check that still around 90 µs

```cmd
bench.bat ingest_clumps_iter_v_slice

bench.bat ingest_clumps_integers

set RUSTFLAGS=-C target-feature=+avx512f
bench.bat worst
```

---------

## rust flags

```cmd
set RUSTFLAGS=
set RUSTFLAGS=-C target-feature=+avx2
set RUSTFLAGS=-C target-feature=+avx512f
set RUSTFLAGS=-C target-cpu=native
set BUILDFEATURES=from_slice

rustup override set nightly
```

## tests

```cmd
cargo test range_set_int_slice_constructor -- --nocapture
cargo test --doc intersection_dyn
cargo test coverage -- --nocapture
cargo test test_rog_functionality -- --nocapture
cargo test --features rog-experimental
```

## examples

```cmd
cargo run --example targets
cargo run --example parity
cargo run --example missing
```

## publish

```cmd
cargo publish --all-features --dry-run
# set version  = "1.0.0-beta.2"


cargo check --no-default-features
```

## test `alloc`

```cmd
cargo test --features --no-default-features
```

## test wasm

```cmd
wasm-pack test --chrome --headless
```

## running on embedded -- see useful.md

```cmd
cargo run --example read_roaring_data

set TRYBUILD=overwrite
```

## Testing

```cmd
cargo test --all-features
cargo testnc map
cargo testnc --test test/map_test
cargo test --target wasm32-wasip1 --all-features
cargo test --target wasm32-unknown-unknown
```

## Coverage

```cmd
cargo llvm-cov --open --all-features
```

## Clippy

```cmd
cargo clippy --all-targets --all-features
```

## Wasm

```cmd
cargo check --target wasm32-unknown-unknown --features alloc --no-default-features
cargo install wasm-bindgen-cli --force
wasm-pack test --firefox --headless --features alloc --no-default-features
cargo test --target wasm32-wasip1 --all-features
```

## Docs

```cmd
# 1. Generate nightly-only docs and open
rustup override set nightly
cargo doc --no-deps --all-features --open

# 2. Switch back to stable, run with experimental feature
rustup override set stable
cargo doc --no-deps --features rog-experimental --open &
cargo test --features rog-experimental --doc

# 3. Run doc tests on all features
cargo test --all-features --doc

# 4. Clear screen, regenerate docs, and check for broken links
cls & cargo doc --no-deps --all-features & cargo deadlinks --dir target/doc
```

## Embedded

Checking and Testing

```cmd
cargo test --features alloc --no-default-features
cargo check --target thumbv7m-none-eabi --features alloc --no-default-features
```

## Running

See: <https://docs.rust-embedded.org/book/start/qemu.html>

```cmd
cd O:\programs\range-set-blaze\tests\embedded
set PATH="C:\Program Files\qemu\";%PATH%
rustup target add thumbv7m-none-eabi
rustup override set nightly
cargo run
```

Behind the scenes it uses its own `.cargo/config.toml` to

```cmd
cargo build --target thumbv7m-none-eabi
qemu-system-arm -cpu cortex-m3 -machine lm3s6965evb -nographic -semihosting-config enable=on,target=native -kernel ..\..\target\thumbv7m-none-eabi\debug\embedded
```

## WASM

Testing

```cmd
# cargo install -f wasm-bindgen-cli --version 0.2.99
set WASM_BINDGEN_TEST_TIMEOUT=60
cargo test --target wasm32-unknown-unknown --all-features
```

Example

```cmd
cd O:\programs\range-set-blaze\tests\wasm-led
wasm-pack build --target web
```

In VS Code, load O:\programs\range-set-blaze\tests\wasm-demo\index.html
Start the Microsoft Live Preview with cntl-shift-P Live Preview ...

## Publish

Set version, e.g., 0.2.0-alpha3

In main directory

```cmd
cargo publish --dry-run
```

## Linux

```bash
cargo bench overflow
target\criterion\overflow\report\index.html

# test native
cargo test
# check and test WASM
cargo check --target wasm32-unknown-unknown --features alloc --no-default-features
wasm-pack test --chrome --headless --features alloc --no-default-features
# check embedded
cargo check --target thumbv7m-none-eabi --features alloc --no-default-features
```
