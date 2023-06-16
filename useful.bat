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


cargo check --no-default-features

# test `alloc`
cargo test --no-default-features --features alloc

# cmk test wasm
wasm-pack test --chrome --headless