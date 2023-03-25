# tests
cargo test --doc intersection_dyn
cargo test coverage -- --nocapture

# examples
cargo run --example parity

# Docs
cargo doc --no-deps --open & cargo test --doc

# coverage
cargo llvm-cov --open
target\llvm-cov\html\index.html

# bench
cargo bench worst & target\criterion\report\index.html