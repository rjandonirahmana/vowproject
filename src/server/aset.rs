//! server/aset.rs — aset statis situs (public/img, public/video) di RustFS.
//!
//! `scripts/unggah-aset.sh` mengunggah berkas ke bucket di bawah kunci
//! `aset/…` dan mencatat URL-nya di tabel `aset` (migrasi 038). Saat start
//! tabel itu dimuat ke PETA (jalur situs → URL RustFS), lalu:
//!   * CSS tema (server/state.rs) & HTML halaman (`layanan`) ditulis ulang:
//!     "/img/x.svg" → "https://image…/undangan/aset/img/x.svg" — browser
//!     langsung mengambil dari RustFS, server app tak menyajikan byte-nya;
//!   * rujukan yang tak tertulis ulang (main.css, render WASM) dialihkan 301.
//! Jalur yang tak ada di peta tetap disajikan dari public/. Peta kosong
//! (tabel belum ada / ASET_RUSTFS=false) = perilaku lama, tanpa biaya.

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use axum::body::{Body, Bytes};
use axum::extract::Request;
use axum::http::{header, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use futures_util::StreamExt;

/// Awalan kunci objek aset bersama. storage.rs menolak menghapus/menimpa
/// kunci berawalan ini — URL aset bisa ikut tersimpan di data undangan
/// (form sunting membawa nilai yang sudah ditulis ulang).
pub const AWALAN_KUNCI: &str = "aset/";

const AWALAN_JALUR: [&[u8]; 2] = [b"/img/", b"/video/"];

type Peta = HashMap<String, String>;

static PETA: RwLock<Option<Arc<Peta>>> = RwLock::new(None);

/// Pasang peta baru (kosong = nonaktif).
pub fn pasang(m: Peta) {
    let v = (!m.is_empty()).then(|| Arc::new(m));
    match PETA.write() {
        Ok(mut g) => *g = v,
        Err(e) => *e.into_inner() = v,
    }
}

fn peta() -> Option<Arc<Peta>> {
    PETA.read().map(|g| g.clone()).unwrap_or_else(|e| e.into_inner().clone())
}

/// Muat tabel `aset`. Galat (tabel belum ada) → peta kosong, dicatat saja.
pub async fn muat(pool: &deadpool_postgres::Pool) {
    match super::repo::aset(pool).await {
        Ok(rows) => {
            let n = rows.len();
            pasang(rows.into_iter().collect());
            if n > 0 {
                tracing::info!(n, "aset statis dilayani dari RustFS");
            } else {
                tracing::info!("tabel aset kosong — /img & /video dari public/ (jalankan scripts/unggah-aset.sh)");
            }
        }
        Err(e) => tracing::warn!(error = %format!("{e:#}"), "tabel aset belum ada (migration/038_aset.sql) — /img & /video dari public/"),
    }
}

fn path_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'/' | b'.' | b'_' | b'-' | b'~' | b'%' | b'+' | b'@')
}

/// Ganti setiap "/img/…" / "/video/…" yang ada di peta dengan URL RustFS-nya.
/// Hanya jalur yang BERDIRI SENDIRI (didahului kutip, kurung, spasi, `=`, `;`
/// dari &quot;, dll.) — "https://situs-lain/img/x" tak disentuh. Akhiran
/// ?v= / #pos= tetap dipertahankan.
pub fn tulis_ulang(s: &str) -> Cow<'_, str> {
    match peta() {
        Some(m) => tulis_ulang_dengan(s, &m),
        None => Cow::Borrowed(s),
    }
}

fn tulis_ulang_dengan<'a>(s: &'a str, m: &Peta) -> Cow<'a, str> {
    let b = s.as_bytes();
    let mut out = String::new();
    let mut last = 0;
    let mut i = 0;
    while let Some(off) = b[i..].iter().position(|&c| c == b'/') {
        let at = i + off;
        let berdiri = at == 0 || !(path_char(b[at - 1]) || b[at - 1] == b':');
        if berdiri && AWALAN_JALUR.iter().any(|p| b[at..].starts_with(p)) {
            let end = at + b[at..].iter().position(|&c| !path_char(c)).unwrap_or(b.len() - at);
            if let Some(url) = m.get(&s[at..end]) {
                out.push_str(&s[last..at]);
                out.push_str(url);
                last = end;
                i = end;
                continue;
            }
        }
        i = at + 1;
    }
    if last == 0 {
        Cow::Borrowed(s)
    } else {
        out.push_str(&s[last..]);
        Cow::Owned(out)
    }
}

fn tulis_ulang_bytes(v: Vec<u8>, m: &Peta) -> Vec<u8> {
    match String::from_utf8(v) {
        Ok(s) => match tulis_ulang_dengan(&s, m) {
            Cow::Borrowed(_) => s.into_bytes(),
            Cow::Owned(o) => o.into_bytes(),
        },
        Err(e) => e.into_bytes(),
    }
}

/// Middleware: GET /img/… & /video/… yang ada di peta → 301 ke RustFS;
/// respons HTML ditulis ulang SAMBIL streaming (SSR out-of-order tetap
/// mengalir). Dipasang di DALAM kompresi (body masih polos).
pub async fn layanan(req: Request, next: Next) -> Response {
    let Some(m) = peta() else { return next.run(req).await };
    if req.method() != Method::GET && req.method() != Method::HEAD {
        return next.run(req).await;
    }
    let path = req.uri().path();
    if AWALAN_JALUR.iter().any(|p| path.as_bytes().starts_with(p)) {
        return match m.get(path) {
            Some(url) => (
                StatusCode::MOVED_PERMANENTLY,
                // 301 di-cache browser; dibatasi sehari agar bisa dicabut.
                [(header::LOCATION, url.as_str()), (header::CACHE_CONTROL, "public, max-age=86400")],
            )
                .into_response(),
            None => next.run(req).await,
        };
    }
    let res = next.run(req).await;
    let html = res
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("text/html"));
    if !html || res.headers().contains_key(header::CONTENT_ENCODING) {
        return res;
    }
    let (mut parts, body) = res.into_parts();
    parts.headers.remove(header::CONTENT_LENGTH);
    Response::from_parts(parts, Body::from_stream(alir(body, m)))
}

/// Tulis ulang body potong demi potong. Tiap potongan dipotong setelah `>`
/// terakhir: jalur tak pernah memuat `>`, jadi tak ada jalur yang terbelah
/// antar potongan; sisanya ditahan sampai potongan berikut.
fn alir(body: Body, m: Arc<Peta>) -> impl futures_util::Stream<Item = Result<Bytes, axum::Error>> {
    struct St {
        inner: axum::body::BodyDataStream,
        sisa: Vec<u8>,
        m: Arc<Peta>,
        selesai: bool,
    }
    const TAHAN_MAKS: usize = 256 * 1024;
    let st = St { inner: body.into_data_stream(), sisa: Vec::new(), m, selesai: false };
    // fuse(): kompresi/hyper bisa mem-poll SEKALI LAGI setelah body habis;
    // `unfold` polos panic ("must not be polled after … Ready(None)").
    futures_util::stream::unfold(st, |mut st| async move {
        loop {
            if st.selesai {
                return None;
            }
            match st.inner.next().await {
                Some(Ok(chunk)) => {
                    st.sisa.extend_from_slice(&chunk);
                    let potong = match st.sisa.iter().rposition(|&c| c == b'>') {
                        Some(p) => p + 1,
                        None if st.sisa.len() > TAHAN_MAKS => st.sisa.len(),
                        None => continue,
                    };
                    let ekor = st.sisa.split_off(potong);
                    let kepala = std::mem::replace(&mut st.sisa, ekor);
                    let out = tulis_ulang_bytes(kepala, &st.m);
                    return Some((Ok(Bytes::from(out)), st));
                }
                Some(Err(e)) => {
                    st.selesai = true;
                    return Some((Err(e), st));
                }
                None => {
                    st.selesai = true;
                    if st.sisa.is_empty() {
                        return None;
                    }
                    let kepala = std::mem::take(&mut st.sisa);
                    let out = tulis_ulang_bytes(kepala, &st.m);
                    return Some((Ok(Bytes::from(out)), st));
                }
            }
        }
    })
    .fuse()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn peta_uji() -> Peta {
        [
            ("/img/tema/a.svg", "https://cdn/u/aset/img/tema/a.svg"),
            ("/video/b.mp4", "https://cdn/u/aset/video/b.mp4"),
        ]
        .into_iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect()
    }

    #[test]
    fn jalur_berdiri_sendiri_saja() {
        let m = peta_uji();
        let t = |s: &str| tulis_ulang_dengan(s, &m).into_owned();
        assert_eq!(t(r#"<img src="/img/tema/a.svg">"#), r#"<img src="https://cdn/u/aset/img/tema/a.svg">"#);
        assert_eq!(t("mask: url(/img/tema/a.svg) center"), "mask: url(https://cdn/u/aset/img/tema/a.svg) center");
        assert_eq!(t("url(&quot;/img/tema/a.svg&quot;)"), "url(&quot;https://cdn/u/aset/img/tema/a.svg&quot;)");
        assert_eq!(t(r#"<video src="/video/b.mp4#t=2">"#), r#"<video src="https://cdn/u/aset/video/b.mp4#t=2">"#);
        assert_eq!(t("/img/tema/a.svg?v=3"), "https://cdn/u/aset/img/tema/a.svg?v=3");
        // Situs lain, jalur lebih panjang, dan jalur tak terdaftar tak disentuh.
        for s in ["https://x.id/img/tema/a.svg", "/img/tema/a.svg.bak", "/u/img/tema/a.svg", "/img/lain.svg", "tanpa jalur"] {
            assert!(matches!(tulis_ulang_dengan(s, &m), Cow::Borrowed(_)), "{s}");
        }
    }

    #[tokio::test]
    async fn streaming_tidak_membelah_jalur() {
        let m = Arc::new(peta_uji());
        let potongan: Vec<Result<Bytes, std::io::Error>> =
            ["<p>a</p><img src=\"/im", "g/tema/a.svg\"><b>", "x</b> /video/b.mp4"].into_iter().map(|s| Ok(Bytes::from(s))).collect();
        let body = Body::from_stream(futures_util::stream::iter(potongan));
        let mut s = Box::pin(alir(body, m));
        let mut out = Vec::new();
        while let Some(r) = s.next().await {
            out.extend_from_slice(&r.unwrap());
        }
        // Di-poll lagi setelah habis (perilaku kompresi/hyper) → tetap None, tak panic.
        assert!(s.next().await.is_none());
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "<p>a</p><img src=\"https://cdn/u/aset/img/tema/a.svg\"><b>x</b> https://cdn/u/aset/video/b.mp4"
        );
    }
}
