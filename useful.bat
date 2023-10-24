# tests
cargo test --doc intersection_dyn
cargo test coverage -- --nocapture
cargo test test_rog_functionality -- --nocapture
cargo test --features rog-experimental

# examples
cargo run --example parity
cargo run --example missing

# Docs
cargo doc --no-deps --features rog-experimental --open & cargo test --features rog-experimental --doc

# coverage
cargo llvm-cov --open
target\llvm-cov\html\index.html

# bench
cargo bench worst & target\criterion\report\index.html
cargo bench overflow & target\criterion\overflow\report\index.html 
python benches\summary.py > benches\summary_r.tsv
cargo bench ingest_roaring_data & target\criterion\ingest_roaring_data\report\index.html 

# publish
cargo publish --dry-run
# set version  = "1.0.0-alpha"


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