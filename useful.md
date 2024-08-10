# Useful commands

## Tests

```bash
cargo test
cargo test --target wasm32-wasip1
cargo test --target wasm32-unknown-unknown
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
