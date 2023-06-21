# tests
cargo test --doc intersection_dyn
cargo test coverage -- --nocapture

# examples
cargo run --example parity
cargo run --example missing

# Docs
cargo doc --no-deps --open & cargo test --doc

# coverage
cargo llvm-cov --open
target\llvm-cov\html\index.html

# bench
cargo bench worst & target\criterion\report\index.html
cargo bench overflow & target\criterion\overflow\report\index.html 
python benches\summary.py > benches\summary.tsv

# publish
cargo publish --dry-run


cargo check

# test `alloc`
cargo test --features alloc

# test wasm
wasm-pack test --chrome --headless --features alloc

See: https://docs.rust-embedded.org/book/start/qemu.html
set PATH="C:\Program Files\qemu\";%PATH%
rustup target add thumbv7m-none-eabi
cargo check --target thumbv7m-none-eabi
rustup override set nightly
cargo build
qemu-system-arm -cpu cortex-m3 -machine lm3s6965evb -nographic -semihosting-config enable=on,target=native -kernel ..\..\target\thumbv7m-none-eabi\debug\app
