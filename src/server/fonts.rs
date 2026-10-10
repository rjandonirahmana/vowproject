//! server/fonts.rs — Google Fonts disajikan dari domain sendiri.
//!
//! Halaman merujuk `/fonts.css?family=…` (web::fmt::font_css). Server
//! mengambil CSS dari Google SEKALI (di-cache di memori), mengganti URL
//! berkas ke `/fonts/g/…`, lalu berkas .woff2 juga diambil sekali & disajikan
//! dengan cache browser 1 tahun. Hasilnya: tak ada koneksi ke dua domain
//! Google (DNS + TLS + CSS yang memblokir render) — semua lewat koneksi
//! HTTP/2 yang sama dengan halaman. Google gagal → CSS dialihkan ke Google
//! (CSP masih mengizinkannya), jadi huruf tak pernah hilang.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use axum::body::Bytes;
use axum::extract::Request;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};

const CSS_UP: &str = "https://fonts.googleapis.com/css2?";
const FILE_UP: &str = "https://fonts.gstatic.com/";
const FILE_LOCAL: &str = "/fonts/g/";
/// Batas cache (CSS + berkas). Lewat → dikosongkan (diisi ulang dari Google).
const CACHE_MAX_BYTES: usize = 48 * 1024 * 1024;
/// Agen Chrome modern → Google mengirim woff2 (terkecil).
const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0 Safari/537.36";

struct Cache {
    items: HashMap<String, (Bytes, &'static str)>,
    bytes: usize,
}

fn cache() -> &'static Mutex<Cache> {
    static C: OnceLock<Mutex<Cache>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(Cache { items: HashMap::new(), bytes: 0 }))
}

fn client() -> Option<&'static reqwest::Client> {
    static C: OnceLock<Option<reqwest::Client>> = OnceLock::new();
    C.get_or_init(|| reqwest::Client::builder().timeout(Duration::from_secs(10)).user_agent(UA).build().ok()).as_ref()
}

fn get_cached(key: &str) -> Option<(Bytes, &'static str)> {
    cache().lock().ok()?.items.get(key).cloned()
}

fn put_cached(key: String, body: Bytes, ctype: &'static str) {
    if let Ok(mut c) = cache().lock() {
        if c.bytes + body.len() > CACHE_MAX_BYTES {
            c.items.clear();
            c.bytes = 0;
        }
        c.bytes += body.len();
        c.items.insert(key, (body, ctype));
    }
}

/// Query css2 yang sah: hanya karakter yang dipakai nama/sumbu font Google.
fn query_ok(q: &str) -> bool {
    q.starts_with("family=") && q.len() <= 4000 && q.bytes().all(|b| b.is_ascii_alphanumeric() || b"+:,;@.&=_-%".contains(&b))
}

/// Jalur berkas Google yang sah: `s/<keluarga>/<versi>/<berkas>.woff2`, atau
/// `l/font?kit=…` (subset ikon Material Symbols) — bukan proksi terbuka.
fn file_ok(path: &str, query: Option<&str>) -> bool {
    let safe = |s: &str| s.bytes().all(|b| b.is_ascii_alphanumeric() || b"_-.".contains(&b));
    match path.split('/').collect::<Vec<_>>().as_slice() {
        ["s", fam, ver, file] => safe(fam) && safe(ver) && file.ends_with(".woff2") && safe(file) && query.is_none(),
        ["l", "font"] => query.is_some_and(|q| q.len() <= 600 && q.bytes().all(|b| b.is_ascii_alphanumeric() || b"=&_-.%".contains(&b))),
        _ => false,
    }
}

async fn fetch(url: &str) -> Option<(Bytes, String)> {
    let r = client()?.get(url).send().await.ok()?;
    if !r.status().is_success() {
        return None;
    }
    let ctype = r.headers().get(header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
    Some((r.bytes().await.ok()?, ctype))
}

/// GET /fonts.css?family=…&display=swap
pub async fn css(req: Request) -> Response {
    let q = req.uri().query().unwrap_or("").to_string();
    if !query_ok(&q) {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let key = format!("css:{q}");
    let body = match get_cached(&key) {
        Some((b, _)) => b,
        None => match fetch(&format!("{CSS_UP}{q}")).await {
            Some((b, _)) => {
                let css = String::from_utf8_lossy(&b).replace(FILE_UP, FILE_LOCAL);
                let b = Bytes::from(css);
                put_cached(key, b.clone(), "text/css");
                b
            }
            None => {
                tracing::warn!(query = %q, "fonts: Google CSS gagal — dialihkan");
                return Redirect::temporary(&format!("{CSS_UP}{q}")).into_response();
            }
        },
    };
    // Isi CSS bisa berubah (versi font baru) — 1 hari, berkasnya 1 tahun.
    ([(header::CONTENT_TYPE, "text/css; charset=utf-8"), (header::CACHE_CONTROL, "public, max-age=86400")], body).into_response()
}

/// GET /fonts/g/{*path} — berkas font (URL ber-versi → immutable).
pub async fn file(req: Request) -> Response {
    let path = req.uri().path().strip_prefix(FILE_LOCAL).unwrap_or("").to_string();
    let query = req.uri().query().map(str::to_string);
    if !file_ok(&path, query.as_deref()) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let up = match &query {
        Some(q) => format!("{FILE_UP}{path}?{q}"),
        None => format!("{FILE_UP}{path}"),
    };
    let (body, ctype) = match get_cached(&up) {
        Some(v) => v,
        None => match fetch(&up).await {
            Some((b, ct)) => {
                let ct = if ct.contains("woff2") { "font/woff2" } else if ct.contains("woff") { "font/woff" } else { "font/ttf" };
                put_cached(up, b.clone(), ct);
                (b, ct)
            }
            None => return StatusCode::BAD_GATEWAY.into_response(),
        },
    };
    ([(header::CONTENT_TYPE, ctype), (header::CACHE_CONTROL, "public, max-age=31536000, immutable")], body).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hanya_jalur_font_google() {
        assert!(query_ok("family=Playfair+Display:ital,wght@0,500;1,500&family=Great+Vibes&display=swap"));
        assert!(!query_ok("family=x\"><script>"));
        assert!(!query_ok("url=http://evil"));
        assert!(file_ok("s/greatvibes/v21/RWmMoKWR9v4ksMfaWd_JN9XFiaQoDmlr.woff2", None));
        assert!(file_ok("l/font", Some("kit=abc-_123&skey=9f&v=v1")));
        assert!(!file_ok("s/../../etc/passwd.woff2", None));
        assert!(!file_ok("s/a/b/c.js", None));
        assert!(!file_ok("l/font", Some("kit=a&next=http://x/y")));
        assert!(!file_ok("x/y", None));
    }
}
