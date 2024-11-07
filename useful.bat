# cmk delete this (merge into useful.md)
# rust flags
set RUSTFLAGS=
set RUSTFLAGS=-C target-feature=+avx2
set RUSTFLAGS=-C target-feature=+avx512f
set RUSTFLAGS=-C target-cpu=native
set BUILDFEATURES=from_slice

rustup override set nightly

# Run criterion-means
cargo install cargo-criterion-means --version 0.1.0-beta.3
set SIMD_LANES=64
set SIMD_INTEGER=i16
set RUSTFLAGS=-C target-feature=+avx512f
cargo criterion-means ..\..\.. > delme.csv

# run packages
cargo run --package criterion-means ..\..\..

# tests
cargo test range_set_int_slice_constructor -- --nocapture
cargo test --doc intersection_dyn
cargo test coverage -- --nocapture
cargo test test_rog_functionality -- --nocapture
cargo test --features rog-experimental

# examples
cargo run --example targets
cargo run --example parity
cargo run --example missing

# Docs
cargo doc --no-deps --all-features --open
cargo doc --no-deps --features rog-experimental --open & cargo test --features rog-experimental --doc
cargo test --all-features --doc

# coverage
cargo llvm-cov --open
target\llvm-cov\html\index.html

# bench
bench.bat ingest_clumps_iter_v_slice
bench.bat ingest_clumps_base
cargo bench worst & target\criterion\report\index.html
cargo bench overflow & target\criterion\overflow\report\index.html 
python benches\summary.py > benches\summary_r.tsv
cargo bench ingest_roaring_data & target\criterion\ingest_roaring_data\report\index.html 

# publish
cargo publish --all-features --dry-run
# set version  = "1.0.0-beta.2"


cargo check --no-default-features

# test `alloc`
cargo test --features alloc --no-default-features

# test wasm
wasm-pack test --chrome --headless

# running on embedded -- see useful.md

cargo run --example read_roaring_data

set TRYBUILD=overwrite

# Bench in-context from_slice

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
