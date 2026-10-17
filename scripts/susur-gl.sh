#!/usr/bin/env bash
# Bangun perender WebGL2 tema SUSUR 3D (crate susur-gl/) → src/web/susur/gl/
# (di-embed binary server: /susur/gl.js & /susur/gl.wasm, ?v= hash isi).
# Jalankan ulang setiap susur-gl/ atau src/web/susur/dunia.rs berubah, lalu
# build server seperti biasa. Butuh: target wasm32-unknown-unknown, CLI
# wasm-bindgen 0.2.122 (sama dengan Cargo.toml), wasm-opt (binaryen).
set -euo pipefail
cd "$(dirname "$0")/.."
T="${CARGO_TARGET_DIR:-target}/susur-gl"
OUT=src/web/susur/gl
cargo build --manifest-path susur-gl/Cargo.toml --release --target wasm32-unknown-unknown --target-dir "$T"
wasm-bindgen --target web --no-typescript --out-dir "$OUT" --out-name susur_gl "$T/wasm32-unknown-unknown/release/susur_gl.wasm"
# Fitur yang memang dipancarkan rustc (Safari 15+/Chrome 91+ mendukung).
wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext --enable-mutable-globals --enable-reference-types --enable-multivalue "$OUT/susur_gl_bg.wasm" -o "$OUT/susur_gl_bg.wasm"
ls -l "$OUT"
