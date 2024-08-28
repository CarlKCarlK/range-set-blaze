# Useful Commands

## rust flags

```bash
set RUSTFLAGS=
set RUSTFLAGS=-C target-feature=+avx2
set RUSTFLAGS=-C target-feature=+avx512f
set RUSTFLAGS=-C target-cpu=native
set BUILDFEATURES=from_slice

rustup override set nightly
```

## Run criterion-means

```bash
cargo install cargo-criterion-means --version 0.1.0-beta.3
set SIMD_LANES=64
set SIMD_INTEGER=i16
set RUSTFLAGS=-C target-feature=+avx512f
cargo criterion-means ..\..\.. > delme.csv
```

## run packages

```bash
cargo run --package criterion-means ..\..\..
```

## tests

```bash
cargo test range_set_int_slice_constructor -- --nocapture
cargo test --doc intersection_dyn
cargo test coverage -- --nocapture
cargo test test_rog_functionality -- --nocapture
cargo test --features rog-experimental
```

## examples

```bash
cargo run --example targets
cargo run --example parity
cargo run --example missing
```

## Docs

```bash
cargo doc --no-deps --all-features --open
cargo doc --no-deps --features rog-experimental --open & cargo test --features rog-experimental --doc
cargo test --all-features --doc
```

## coverage

```bash
cargo llvm-cov --open
target\llvm-cov\html\index.html
```

## bench

```bash
bench.bat ingest_clumps_iter_v_slice
bench.bat ingest_clumps_base
cargo bench worst & target\criterion\report\index.html
cargo bench overflow & target\criterion\overflow\report\index.html 
python benches\summary.py > benches\summary_r.tsv
cargo bench ingest_roaring_data & target\criterion\ingest_roaring_data\report\index.html 
```

## publish

```bash
cargo publish --all-features --dry-run
# set version  = "1.0.0-beta.2"
cargo check --no-default-features
```

## test `alloc`

```bash
cargo test --features alloc --no-default-features
```

## test wasm

```bash
wasm-pack test --chrome --headless
```

See: <https://docs.rust-embedded.org/book/start/qemu.html>

```bash
set PATH="C:\Program Files\qemu\";%PATH%
rustup target add thumbv7m-none-eabi
rustup override set nightly
cargo build 
qemu-system-arm -cpu cortex-m3 -machine lm3s6965evb -nographic -semihosting-config enable=on,target=native -kernel ..\..\target\thumbv7m-none-eabi\debug\app


cargo run --example read_roaring_data

set TRYBUILD=overwrite
```

## Bench in-context from_slice

```bash
set RUSTFLAGS=
set RUSTFLAGS=-C target-feature=+avx2
set RUSTFLAGS=-C target-feature=+avx512f
set RUSTFLAGS=-C target-cpu=native
set BUILDFEATURES=from_slice

rustup override set nightly

# check that still around 90 µs
bench.bat ingest_clumps_iter_v_slice

bench.bat ingest_clumps_integers

set RUSTFLAGS=-C target-feature=+avx512f
bench.bat worst
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
