#!/bin/bash

set -euo pipefail

cargo build --release --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/release/random-grid.wasm --out-dir wasm --no-modules --no-typescript
cd wasm
python -m http.server