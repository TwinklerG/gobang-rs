<h1 align="center">Gobang-rs</h1>

## Quick Start

### Desktop

```shell
cargo run
```

### WebAssembly

```shell
cargo build --target wasm32-unknown-unknown --release # rustup target add wasm32-unknown-unknown
wasm-bindgen ./target/wasm32-unknown-unknown/release/gobang-rs.wasm  --out-dir ./web --target web # cargo install wasm-bindgen-cli
python3 -m http.server -d ./web
```
