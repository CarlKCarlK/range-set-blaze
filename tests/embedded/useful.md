# Useful Commands

Be sure QEMU is on path with `qemu-system-arm --version`.

```bash
cd tests/embedded
rustup override set nightly # to support #![feature(alloc_error_handler)]
cargo run
```
