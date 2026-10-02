//! server/owner.rs — kunci Kelola pemilik undangan: tautan khusus → cookie.
//!
//! Pembeli tetap menerima tautan rahasia `/kelola/{slug}?key=…` (itulah cara
//! masuk dari perangkat mana pun). Tapi kunci tak dibiarkan hidup di URL
//! (riwayat browser, log proxy, tangkapan layar): middleware `exchange`
//! memvalidasinya SEKALI, menyimpannya di cookie HttpOnly `ily_k_{slug}`, lalu
//! 303 ke URL yang sama tanpa kunci. Server fn & ekspor CSV membaca cookie bila
//! kunci tak dikirim. Admin menerbitkan ulang kunci → hash berubah → cookie
//! lama otomatis tak berlaku.
//!
//! SameSite=Lax (bukan Strict): tautan dibuka dari WhatsApp = navigasi lintas
//! situs; dengan Strict, cookie tak ikut pada request hasil 303 itu sehingga
//! dashboard pertama gagal. POST tetap dijaga `security::csrf` (Origin).
//!
//! Juga berlaku untuk pratinjau pemilik `/u/{slug}?k=…` (undangan terkunci).

use std::sync::Arc;

use axum::extract::{Extension, Request};
use axum::http::{header, HeaderMap, HeaderValue, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use super::state::AppState;
use super::{auth, repo};

/// Masa berlaku cookie: cukup untuk persiapan sampai lewat hari H.
const MAX_AGE_DAYS: i64 = 180;

fn cookie_name(slug: &str) -> String {
    format!("ily_k_{slug}")
}

/// Slug hanya `[a-z0-9-]` (fmt::key + random_key) — aman jadi nama cookie.
fn valid_slug(s: &str) -> bool {
    !s.is_empty() && s.len() <= 80 && s.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// Kunci Kelola dari cookie request ini (belum diverifikasi).
pub fn key_from(headers: &HeaderMap, slug: &str) -> Option<String> {
    if !valid_slug(slug) {
        return None;
    }
    super::handlers::cookie_value(headers, &cookie_name(slug)).filter(|v| !v.is_empty()).map(str::to_string)
}

fn set_cookie(slug: &str, key: &str, headers: &HeaderMap) -> String {
    let https = headers.get("x-forwarded-proto").and_then(|v| v.to_str().ok()) == Some("https");
    format!(
        "{}={key}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}{}",
        cookie_name(slug),
        MAX_AGE_DAYS * 86_400,
        if https { "; Secure" } else { "" }
    )
}

/// `/kelola/{slug}…?key=` → ("slug", "key"); `/u/{slug}…?k=` → ("slug", "k").
fn target(path: &str) -> Option<(&str, &'static str)> {
    let (rest, param) = if let Some(r) = path.strip_prefix("/kelola/") {
        (r, "key")
    } else if let Some(r) = path.strip_prefix("/u/") {
        (r, "k")
    } else {
        return None;
    };
    let slug = rest.split('/').next().unwrap_or("");
    valid_slug(slug).then_some((slug, param))
}

/// Pisahkan parameter kunci dari query: (nilai kunci, sisa query).
fn split_key(query: &str, param: &str) -> (Option<String>, String) {
    let mut key = None;
    let rest: Vec<&str> = query
        .split('&')
        .filter(|kv| match kv.split_once('=').unwrap_or((kv, "")) {
            (k, v) if k == param => {
                key = Some(v.to_string());
                false
            }
            _ => !kv.is_empty(),
        })
        .collect();
    (key, rest.join("&"))
}

pub async fn exchange(Extension(state): Extension<Arc<AppState>>, req: Request, next: Next) -> Response {
    if req.method() != Method::GET {
        return next.run(req).await;
    }
    let path = req.uri().path().to_string();
    let Some((slug, param)) = target(&path) else { return next.run(req).await };
    let (Some(key), rest) = split_key(req.uri().query().unwrap_or(""), param) else { return next.run(req).await };

    let valid = match repo::invitation(&state.pool, slug).await {
        Ok(Some(row)) => !key.is_empty() && auth::same_hash(&auth::token_hash(key.trim()), &row.manage_key_hash),
        Ok(None) => false,
        // DB bermasalah: biarkan halaman menampilkan galatnya sendiri.
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "owner: cek kunci");
            return next.run(req).await;
        }
    };
    // Kunci salah pun dibuang dari URL — halaman lalu menampilkan "Kunci kelola
    // tidak valid" karena cookie tak dipasang.
    let to = if rest.is_empty() { path.clone() } else { format!("{path}?{rest}") };
    let mut res = (StatusCode::SEE_OTHER, [(header::LOCATION, to)]).into_response();
    if valid {
        if let Ok(v) = HeaderValue::from_str(&set_cookie(slug, key.trim(), req.headers())) {
            res.headers_mut().append(header::SET_COOKIE, v);
        }
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rute_dan_parameter() {
        assert_eq!(target("/kelola/ani-budi-k7f3x9m2"), Some(("ani-budi-k7f3x9m2", "key")));
        assert_eq!(target("/kelola/ani/scan"), Some(("ani", "key")));
        assert_eq!(target("/u/ani/acara"), Some(("ani", "k")));
        assert_eq!(target("/kelola/Ani%20B"), None);
        assert_eq!(target("/tema/jawa"), None);
    }

    #[test]
    fn kunci_dipisah_dari_query() {
        assert_eq!(split_key("key=abc&baru=1", "key"), (Some("abc".into()), "baru=1".into()));
        assert_eq!(split_key("tema=x&key=demo", "key"), (Some("demo".into()), "tema=x".into()));
        assert_eq!(split_key("g=AB12&k=zz", "k"), (Some("zz".into()), "g=AB12".into()));
        assert_eq!(split_key("tema=x", "key"), (None, "tema=x".into()));
    }

    #[test]
    fn cookie_dibaca_per_slug() {
        let mut h = HeaderMap::new();
        h.insert(header::COOKIE, HeaderValue::from_static("ily_k_ani=s3cret; ily_k_lain=x"));
        assert_eq!(key_from(&h, "ani").as_deref(), Some("s3cret"));
        assert_eq!(key_from(&h, "budi"), None);
        assert!(set_cookie("ani", "s3cret", &h).contains("HttpOnly; SameSite=Lax"));
    }
}
