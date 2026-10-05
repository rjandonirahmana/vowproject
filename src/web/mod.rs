//! web — halaman Leptos (SSR + hydration) dalam binary yang sama.

/// Nama situs/merek — SATU-SATUNYA tempat untuk menggantinya. Berupa makro
/// agar bisa dipakai di `concat!` (teks statis) maupun `format!`.
#[macro_export]
macro_rules! brand {
    () => {
        "ilyvowcraft"
    };
}
/// Monogram merek di header situs & panel admin.
pub const BRAND_MONOGRAM: &str = "I&V";

pub mod anim;
pub mod api;
pub mod app;
pub mod components;
pub mod fmt;
pub mod icons;
pub mod konten;
pub mod layanan;
pub mod model;
pub mod ornamen;
pub mod pages;
pub mod seo;
pub mod skin;
pub mod themes;
