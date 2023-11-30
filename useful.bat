# rust flags
set RUSTFLAGS=
set RUSTFLAGS=-C target-cpu=native
set RUSTFLAGS=-C target-feature=+avx2
set RUSTFLAGS=-C target-feature=+avx512f
set BUILDFEATURES=from_slice

rustup override set nightly

# Run criterion-means
cargo install cargo-criterion-means --version 0.1.0-beta.1
set SIMD_LANES=32
set SIMD_INTEGER=i64
set RUSTFLAGS=-C target-feature=+avx512f
cargo criterion-means ..\..\..

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

See: https://docs.rust-embedded.org/book/start/qemu.html
set PATH="C:\Program Files\qemu\";%PATH%
rustup target add thumbv7m-none-eabi
rustup override set nightly
cargo build 
qemu-system-arm -cpu cortex-m3 -machine lm3s6965evb -nographic -semihosting-config enable=on,target=native -kernel ..\..\target\thumbv7m-none-eabi\debug\app


cargo run --example read_roaring_data

set TRYBUILD=overwrite

# Disassemble

cargo install cargo-binutils
rustup component add llvm-tools-preview
cargo objdump --example targets -- --disassemble --no-show-raw-insn --print-imm-hex

cargo objdump --release --example simdtest -- --disassemble --no-show-raw-insn --print-imm-hex > target\release\examples\simdtest.asm.txt
