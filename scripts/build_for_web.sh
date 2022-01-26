#!/usr/bin/env bash
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/release/rust_roguelike.wasm --out-dir web --no-modules --no-typescript
