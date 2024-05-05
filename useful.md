# Useful commands for this project

## Docs

cargo doc --no-deps --all-features --open
cargo doc --no-deps --features rog-experimental --open & cargo test --features rog-experimental --doc
cargo test --all-features --doc


## Benchmarking

```cmd
cargo bench

bench every_op_blaze

@'rem cargo install cargo-criterion-means 
cargo criterion-means > o:\Projects\Science\rangemapblaze\unitvalue\results.2.csv
```

## Testing

```cmd
cargo testnc map
cargo testnc --test test/map_test

## Embedded

Checking and Testing

```cmd
cargo test --features alloc --no-default-features
cargo check --target thumbv7m-none-eabi --features alloc --no-default-features
```

Running

See: <https://docs.rust-embedded.org/book/start/qemu.html>

```cmd
cd O:\programs\range-set-blaze\tests\embedded
set PATH="C:\Program Files\qemu\";%PATH%
rustup target add thumbv7m-none-eabi
rustup override set nightly
cargo build
qemu-system-arm -cpu cortex-m3 -machine lm3s6965evb -nographic -semihosting-config enable=on,target=native -kernel ..\..\target\thumbv7m-none-eabi\debug\embedded
```

## WASM

```cmd
cd O:\programs\range-set-blaze\tests\wasm-led
wasm-pack build --target web
```

In VS Code, load O:\programs\range-set-blaze\tests\wasm-demo\index.html
Start the Microsoft Live Preview with cntl-shift-P Live Preview ...

## Publish

Set version, e.g., 0.1.16-alpha1

In main directory

```cmd
cargo publish --dry-run
```
