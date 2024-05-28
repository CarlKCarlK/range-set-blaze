# Useful Commands

```cmd
cargo test

rustup target add wasm32-unknown-unknown

cargo check --target wasm32-unknown-unknown
wasm-pack test --firefox --headless 

doesn't work: cargo test --target wasm32-unknown-unknown 
cargo test --target wasm32-wasip1
```

cmk Weird Work Arounds

* Chrome didn't work so used Firefox
* Got WASM test macro bug. It went away when I deleted Cargo.lock

cmk

* Can we share tests?
* Can we test with regular `cargo test`?
* Should we use node rather than headless firefox.
