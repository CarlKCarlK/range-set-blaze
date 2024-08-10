# Useful commands

## Tests

```bash
cargo test
cargo test --target wasm32-wasip1
cargo test --target wasm32-unknown-unknown

# Just checking
cargo check --target thumbv7m-none-eabi --features alloc --no-default-features
```

## Benchmarking example

```bash
cargo bench overflow
target\criterion\overflow\report\index.html
```

## Strict Linting

```bash
cargo clippy --verbose --all-targets --all-features -- -D warnings
```
