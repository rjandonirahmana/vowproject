//! lib.rs — entry WASM (hydration) + modul bersama untuk server.
//!
//! `web` dikompilasi untuk KEDUA target (native SSR + wasm32); `server`
//! (Postgres, RustFS, handler multipart) hanya native.

#![recursion_limit = "512"]

#[cfg(feature = "ssr")]
pub mod server;
pub mod web;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    // Async: halaman yang dibuka langsung di rute terpisah (admin, Kelola, …)
    // menunggu potongan WASM-nya terunduh sebelum di-hydrate.
    leptos::mount::hydrate_lazy(web::app::App);
}
