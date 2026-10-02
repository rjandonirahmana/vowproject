.PHONY: dev run build check test clean

## Dev dengan WASM hydration + hot reload (http://localhost:3600)
dev:
	cargo leptos watch

## SSR saja (tanpa WASM)
run:
	cargo run

## Build produksi — jalankan sebelum push (recursion_limit hanya teruji di release)
build:
	cargo leptos build --release --precompress

check:
	cargo check --features ssr
	cargo check --lib --no-default-features --features hydrate --target wasm32-unknown-unknown

test:
	cargo test --lib

clean:
	cargo clean
