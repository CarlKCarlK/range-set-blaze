# Useful commands

## Test

```bash
cargo test
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
