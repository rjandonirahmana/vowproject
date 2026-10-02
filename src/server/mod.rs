//! server — kode khusus native (Postgres, RustFS, handler form multipart).
//! Tidak pernah dikompilasi ke wasm32.

pub mod auth;
pub mod cleanup;
pub mod config;
pub mod db;
pub mod form;
pub mod gambar;
pub mod handlers;
pub mod migrate;
pub mod owner;
pub mod repo;
pub mod security;
pub mod state;
pub mod storage;
pub mod util;
