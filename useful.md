# Useful commands for this project

## Embedded

Checking and Testing

```cmd
cargo test --features alloc --no-default-features
cargo check --target thumbv7m-none-eabi --features alloc --no-default-features
```

Running

See: https://docs.rust-embedded.org/book/start/qemu.html

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
cd O:\programs\range-set-blaze\tests\wasm-demo
wasm-pack build --target web
```

In VS Code, load O:\programs\range-set-blaze\tests\wasm-demo\index.html
Start the Microsoft Live Preview with cntl-shift-P Live Preview ...