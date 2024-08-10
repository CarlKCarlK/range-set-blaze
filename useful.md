# Useful commands

## Tests

```bash
cargo test
cargo test --target wasm32-wasip1
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
