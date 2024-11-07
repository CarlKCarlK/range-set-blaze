# cmk delete this (merge into useful.md)
cargo bench overflow
target\criterion\overflow\report\index.html

# test native
cargo test
# check and test WASM
cargo check --target wasm32-unknown-unknown --features alloc --no-default-features
wasm-pack test --chrome --headless --features alloc --no-default-features
# check embedded
cargo check --target thumbv7m-none-eabi --features alloc --no-default-features