<h1 align="center">Gobang-rs</h1>

## Quick Start

### Desktop

```shell
cargo run
```

### WebAssembly

```shell
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen ./target/wasm32-unknown-unknown/release/gobang-rs.wasm --out-dir ./web
python3 -m http.server -d ./web
```
