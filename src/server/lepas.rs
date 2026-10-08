//! server/lepas.rs — memutus kebocoran memori SSR Leptos per request.
//!
//! Diukur 8 Okt 2026 (alat `leaks` macOS): SETIAP render SSR membocorkan
//! ±13–20 KB — 100 rb kunjungan ≈ 1,5 GB, container 512 MB OOM dalam hitungan
//! jam saat ramai/diserang bot. Penyebab di leptos_router 0.8.15–0.8.17
//! (nested_router: `top_level_outlet` & `Outlet`): closure efek render
//! menyimpan salinan KUAT `Owner` pemiliknya, dan closure itu tersimpan di
//! arena milik owner yang sama → siklus Arc; root owner per request (dibuat
//! leptos_axum) tak pernah di-drop.
//!
//! Perbaikan tanpa mengubah framework: middleware `lepas` menitipkan
//! `OwnerSlot` di extensions request; `web::app::shell` (berjalan di root
//! owner untuk SEMUA render — rute & halaman 404) menyimpan
//! `Owner::current()` ke slot; body respons dibungkus, dan saat body di-drop
//! (selesai terkirim / koneksi putus) `Owner::cleanup()` menghapus node arena
//! beserta closure-nya → siklus putus, memori kembali.

use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use axum::body::{Body, Bytes};
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use leptos::prelude::Owner;

/// Wadah root owner render SSR untuk satu request.
#[derive(Clone, Default)]
pub struct OwnerSlot(Arc<Mutex<Option<Owner>>>);

impl OwnerSlot {
    /// Dipanggil `shell()` — owner pertama yang tercatat dipakai.
    pub fn set(&self, owner: Owner) {
        let mut g = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if g.is_none() {
            *g = Some(owner);
        }
    }

    fn lepas(&self) {
        let owner = self.0.lock().unwrap_or_else(|e| e.into_inner()).take();
        if let Some(o) = owner {
            o.cleanup();
        }
    }
}

pub async fn lepas(mut req: Request, next: Next) -> Response {
    let slot = OwnerSlot::default();
    req.extensions_mut().insert(slot.clone());
    let res = next.run(req).await;
    let (parts, body) = res.into_parts();
    Response::from_parts(parts, Body::new(BodyLepas { inner: body, slot }))
}

/// Body pembungkus: meneruskan frame apa adanya; saat di-drop, root owner
/// render dibersihkan (render streaming sudah pasti selesai/dibatalkan).
struct BodyLepas {
    inner: Body,
    slot: OwnerSlot,
}

impl Drop for BodyLepas {
    fn drop(&mut self) {
        self.slot.lepas();
    }
}

impl http_body::Body for BodyLepas {
    type Data = Bytes;
    type Error = axum::Error;

    fn poll_frame(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        Pin::new(&mut self.inner).poll_frame(cx)
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> http_body::SizeHint {
        self.inner.size_hint()
    }
}
