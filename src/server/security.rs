//! server/security.rs — lapisan keamanan HTTP:
//!
//!   * `headers`   — Cache-Control `private, no-store` untuk halaman pribadi
//!                   (/u/*: QR & nama tamu, /kelola/*: daftar tamu, /admin/*),
//!                   CSP ber-nonce untuk HTML, dan header pengaman lain.
//!   * `csrf`      — tolak POST lintas situs (Origin ≠ host). Cookie admin juga
//!                   SameSite=Strict; ini lapis kedua.
//!   * `RateLimit` — batasi tebak sandi /admin/masuk & /admin/setup, serta
//!                   tulis publik (RSVP, tanda kasih, pembuatan undangan).
//!
//! CSP memakai nonce acak per request: middleware menaruh `CspNonce` di
//! extensions request, `shell()` (web/app.rs) membacanya lewat `Parts` lalu
//! `provide_context(Nonce)` → skrip hydration Leptos & skrip global ikut
//! ber-nonce. 'wasm-unsafe-eval' wajib agar WASM Leptos bisa dikompilasi.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::extract::Request;
use axum::http::{header, HeaderValue, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

#[derive(Clone)]
pub struct CspNonce(pub String);

/// Mode dev: izinkan websocket hot-reload cargo-leptos (port 3601).
#[derive(Clone, Copy)]
pub struct DevMode(pub bool);

/// Pratinjau bergulir kartu katalog: undangan DEMO dengan `?pv=1` (lihat
/// global.js). Isinya publik & sama untuk semua orang → boleh di-cache browser
/// agar prefetch katalog membuat iframe tampil seketika.
pub fn is_demo_preview(path: &str, query: &str) -> bool {
    let demo = crate::web::themes::DEMO_SLUG;
    path.strip_prefix("/u/").is_some_and(|r| r == demo || r.starts_with(&format!("{demo}/")))
        && query.split('&').any(|kv| kv == "pv=1")
}

/// Tautan demo lama `/u|/kelola/{OLD_DEMO_SLUG}…` → alamat demo baru (query ikut).
fn legacy_demo_target(path: &str, query: Option<&str>) -> Option<String> {
    use crate::web::themes::{DEMO_SLUG, OLD_DEMO_SLUG};
    ["/u/", "/kelola/"].into_iter().find_map(|base| {
        let rest = path.strip_prefix(base)?.strip_prefix(OLD_DEMO_SLUG)?;
        (rest.is_empty() || rest.starts_with('/')).then(|| format!("{base}{DEMO_SLUG}{rest}{}", query.map(|q| format!("?{q}")).unwrap_or_default()))
    })
}

/// `www.domain` → `https://domain` (path & query ikut). Satu host saja di
/// mata Google — tanpa ini www & apex sama-sama 200 (konten ganda).
fn www_target(host: &str, path_and_query: &str) -> Option<String> {
    let apex = host.strip_prefix("www.")?;
    (!apex.is_empty()).then(|| format!("https://{apex}{path_and_query}"))
}

/// Middleware: www → apex, dan demo lama yang sudah tersebar (WA, banner)
/// → demo baru. Keduanya 301.
pub async fn legacy_demo(req: Request, next: Next) -> Response {
    let pq = req.uri().path_and_query().map(|p| p.as_str()).unwrap_or("/");
    if let Some(to) = request_host(&req).and_then(|h| www_target(&h, pq)) {
        return (StatusCode::MOVED_PERMANENTLY, [(header::LOCATION, to)]).into_response();
    }
    match legacy_demo_target(req.uri().path(), req.uri().query()) {
        Some(to) => (StatusCode::MOVED_PERMANENTLY, [(header::LOCATION, to)]).into_response(),
        None => next.run(req).await,
    }
}

fn is_private(path: &str) -> bool {
    path.starts_with("/u/") || path.starts_with("/kelola/") || path == "/admin" || path.starts_with("/admin/") || path == "/buat"
}

pub fn csp(nonce: &str, dev: bool) -> String {
    format!(
        "default-src 'self'; \
         script-src 'self' 'nonce-{nonce}' 'wasm-unsafe-eval'; \
         style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; \
         font-src 'self' https://fonts.gstatic.com; \
         img-src 'self' data: blob: https:; \
         media-src 'self' blob: https:; \
         connect-src 'self'{}; \
         frame-src 'self' https://www.google.com https://maps.google.com; \
         form-action 'self' https://wa.me https://api.whatsapp.com; \
         frame-ancestors 'self'; base-uri 'self'; object-src 'none'",
        if dev { " ws: http://localhost:3601 http://127.0.0.1:3601" } else { "" }
    )
}

pub async fn headers(mut req: Request, next: Next) -> Response {
    let nonce = super::auth::random_hex(16);
    let dev = req.extensions().get::<DevMode>().map(|d| d.0).unwrap_or(false);
    req.extensions_mut().insert(CspNonce(nonce.clone()));
    let path = req.uri().path().to_string();
    let private = is_private(&path);
    let demo_pv = is_demo_preview(&path, req.uri().query().unwrap_or(""));
    let mut res = next.run(req).await;
    let ok = res.status().is_success();
    let h = res.headers_mut();
    if private && demo_pv && ok {
        h.insert(header::CACHE_CONTROL, HeaderValue::from_static("private, max-age=600"));
        h.insert("x-robots-tag", HeaderValue::from_static("noindex, nofollow"));
    } else if private {
        h.insert(header::CACHE_CONTROL, HeaderValue::from_static("private, no-store"));
        // Undangan & dashboard tak boleh masuk mesin pencari walau tautannya tersebar.
        h.insert("x-robots-tag", HeaderValue::from_static("noindex, nofollow"));
    } else if ok && (path.starts_with("/img/") || path.starts_with("/music/")) && !h.contains_key(header::CACHE_CONTROL) {
        // Foto & lagu bawaan: jarang berubah → cache 7 hari.
        h.insert(header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=604800"));
    }
    let is_html = h.get(header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).is_some_and(|v| v.starts_with("text/html"));
    if is_html {
        if let Ok(v) = HeaderValue::from_str(&csp(&nonce, dev)) {
            h.insert(header::CONTENT_SECURITY_POLICY, v);
        }
    }
    h.insert("x-content-type-options", HeaderValue::from_static("nosniff"));
    // same-origin: permintaan ke situs LAIN tanpa Referer sama sekali —
    //  * tautan undangan (/u/slug?g=KODE) tak bocor ke Maps, wa.me, dll.;
    //  * lagu & foto di RustFS (image.ulalaapi.store, di balik proxy hotlink
    //    yang menolak Referer domain tak terdaftar tapi mengizinkan Referer
    //    kosong) tetap termuat dari domain mana pun, termasuk localhost.
    // JANGAN "no-referrer": itu membuat header Origin form POST menjadi "null"
    // sehingga csrf() di bawah menolak RSVP & server fn lain.
    h.insert("referrer-policy", HeaderValue::from_static("same-origin"));
    // Kamera hanya untuk halaman ini sendiri (pemindai QR check-in).
    h.insert("permissions-policy", HeaderValue::from_static("camera=(self), microphone=(), geolocation=()"));
    res
}

/// Host yang dilihat klien (di balik proxy: X-Forwarded-Host).
fn request_host(req: &Request) -> Option<String> {
    req.headers()
        .get("x-forwarded-host")
        .or_else(|| req.headers().get(header::HOST))
        .and_then(|v| v.to_str().ok())
        .map(|h| h.split(',').next().unwrap_or(h).trim().to_ascii_lowercase())
}

/// POST dengan Origin dari situs lain → 403. Tanpa Origin (klien lama/non-
/// browser) dibiarkan: browser modern selalu mengirim Origin pada POST.
pub async fn csrf(req: Request, next: Next) -> Result<Response, StatusCode> {
    if req.method() == Method::POST {
        if let Some(origin) = req.headers().get(header::ORIGIN).and_then(|v| v.to_str().ok()) {
            let origin_host = origin.split("://").nth(1).map(|h| h.split('/').next().unwrap_or(h).to_ascii_lowercase());
            if origin == "null" || origin_host.is_none() || origin_host != request_host(&req) {
                tracing::warn!(origin, path = %req.uri().path(), "csrf: POST lintas situs ditolak");
                return Err(StatusCode::FORBIDDEN);
            }
        }
    }
    Ok(next.run(req).await)
}

/// Pembatas percobaan per kunci (username / IP), jendela tetap sederhana.
/// Disimpan di memori PROSES: aplikasi ini sengaja single-instance (satu
/// container). Restart = hitungan mulai ulang — dapat diterima karena login
/// juga diperlambat 800 ms per kegagalan dan sandi di-hash argon2id. Bila
/// kelak dijalankan >1 replika, pindahkan ke tabel Postgres.
/// Memori terbatas: entri kedaluwarsa dibuang paling lambat tiap satu jendela,
/// dan peta tak pernah melebihi `CAP` kunci (kunci tertua dibuang dulu).
pub struct RateLimit {
    max: u32,
    window: Duration,
    inner: Mutex<Hits>,
}

struct Hits {
    map: HashMap<String, (u32, Instant)>,
    swept: Instant,
}

const CAP: usize = 20_000;

impl RateLimit {
    pub fn new(max: u32, window: Duration) -> Self {
        Self { max, window, inner: Mutex::new(Hits { map: HashMap::new(), swept: Instant::now() }) }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Hits> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn remaining(&self, e: &(u32, Instant), now: Instant) -> Option<u64> {
        self.remaining_max(e, now, self.max)
    }

    fn remaining_max(&self, e: &(u32, Instant), now: Instant, max: u32) -> Option<u64> {
        let age = now.duration_since(e.1);
        (e.0 >= max && age < self.window).then(|| (self.window - age).as_secs() + 1)
    }

    /// Sisa detik blokir bila salah satu kunci sudah melewati batas.
    pub fn blocked(&self, keys: &[&str]) -> Option<u64> {
        let now = Instant::now();
        let h = self.lock();
        keys.iter().filter_map(|k| h.map.get(*k)).filter_map(|e| self.remaining(e, now)).max()
    }

    fn record(&self, h: &mut Hits, keys: &[&str], now: Instant) {
        if now.duration_since(h.swept) >= self.window || h.map.len() >= CAP {
            let w = self.window;
            h.map.retain(|_, (_, start)| now.duration_since(*start) < w);
            h.swept = now;
            if h.map.len() >= CAP {
                // Serangan dengan banyak kunci unik: buang separuh tertua.
                let mut ages: Vec<Instant> = h.map.values().map(|v| v.1).collect();
                ages.sort_unstable();
                let cut = ages[ages.len() / 2];
                h.map.retain(|_, (_, start)| *start > cut);
            }
        }
        for k in keys {
            let e = h.map.entry((*k).to_string()).or_insert((0, now));
            if now.duration_since(e.1) >= self.window {
                *e = (0, now);
            }
            e.0 += 1;
        }
    }

    pub fn fail(&self, keys: &[&str]) {
        let now = Instant::now();
        let mut h = self.lock();
        self.record(&mut h, keys, now);
    }

    /// Catat satu percobaan untuk SEMUA permintaan (bukan hanya yang gagal):
    /// `Err(detik)` bila kunci sudah mencapai batas dalam jendela. Satu kunci
    /// mutex untuk cek + catat (tak ada celah balapan di antaranya).
    pub fn hit(&self, key: &str) -> Result<(), u64> {
        self.hit_max(key, self.max)
    }

    /// Seperti `hit`, dengan batas khusus kunci ini (satu pembatas melayani
    /// beberapa jenis kuota dalam jendela yang sama — lihat AppState::cap_limit).
    pub fn hit_max(&self, key: &str, max: u32) -> Result<(), u64> {
        let now = Instant::now();
        let mut h = self.lock();
        if let Some(secs) = h.map.get(key).and_then(|e| self.remaining_max(e, now, max)) {
            return Err(secs);
        }
        self.record(&mut h, &[key], now);
        Ok(())
    }

    pub fn clear(&self, key: &str) {
        self.lock().map.remove(key);
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.lock().map.len()
    }
}

/// Aset statis & cek kesehatan: murah (disajikan dari disk/memori, tanpa DB)
/// — tak ikut kuota request dinamis.
fn statis(path: &str) -> bool {
    ["/pkg/", "/img/", "/music/", "/video/"].iter().any(|p| path.starts_with(p))
        || path.starts_with("/gaya/")
        || matches!(path, "/healthz" | "/readyz" | "/favicon.svg" | "/robots.txt" | "/tata.js" | "/tata.css" | "/app.js" | "/tema.css" | "/sitemap.xml")
}

/// Middleware tahan banjir request untuk rute DINAMIS:
///  1. kuota per IP per menit (REQ_PER_MIN) → 429 + Retry-After;
///  2. batas request yang diproses bersamaan (MAX_INFLIGHT) → 503 seketika
///     tanpa antre — saat diserang, memori & pool DB tetap terkendali dan
///     pengunjung lain tetap dilayani begitu beban turun.
/// Pembatas di memori proses (single instance) dengan jumlah kunci terbatas
/// (RateLimit::CAP) — jutaan IP palsu tak membuat memori tumbuh tanpa batas.
pub async fn pelindung(req: Request, next: Next) -> Response {
    let path = req.uri().path();
    if statis(path) {
        return next.run(req).await;
    }
    let Some(st) = req.extensions().get::<std::sync::Arc<super::state::AppState>>().cloned() else {
        return next.run(req).await;
    };
    let ip = client_ip(req.headers());
    if let Err(secs) = st.req_limit.hit(&format!("req:{ip}")) {
        return (StatusCode::TOO_MANY_REQUESTS, [(header::RETRY_AFTER, secs.to_string())], "Terlalu banyak permintaan. Coba lagi sebentar.").into_response();
    }
    let Ok(_izin) = st.inflight.clone().try_acquire_owned() else {
        tracing::warn!(ip = %ip, path = %path, "pelindung: server penuh — 503");
        return (StatusCode::SERVICE_UNAVAILABLE, [(header::RETRY_AFTER, "3".to_string())], "Server sedang ramai. Muat ulang beberapa detik lagi.").into_response();
    };
    next.run(req).await
}

/// IP klien untuk kunci pembatas: X-Real-IP / X-Forwarded-For dari proxy,
/// atau "langsung". (Bisa dipalsukan bila tak di balik proxy — batas per
/// username tetap berlaku.)
pub fn client_ip(headers: &axum::http::HeaderMap) -> String {
    headers
        .get("x-real-ip")
        .or_else(|| headers.get("x-forwarded-for"))
        .and_then(|v| v.to_str().ok())
        .map(|v| v.split(',').next().unwrap_or(v).trim().to_string())
        .unwrap_or_else(|| "langsung".into())
}

pub fn too_many(secs: u64) -> Response {
    let menit = secs.div_ceil(60);
    axum::response::Redirect::to(&format!(
        "/admin?galat={}",
        crate::web::fmt::url_encode(&format!("Terlalu banyak percobaan gagal. Coba lagi dalam {menit} menit."))
    ))
    .into_response()
}

#[cfg(test)]
mod tests {

    #[test]
    fn batas_kiriman_publik() {
        let rl = RateLimit::new(3, std::time::Duration::from_secs(60));
        assert!(rl.hit("rsvp:ani:1.2.3.4").is_ok() && rl.hit("rsvp:ani:1.2.3.4").is_ok() && rl.hit("rsvp:ani:1.2.3.4").is_ok());
        assert!(rl.hit("rsvp:ani:1.2.3.4").is_err_and(|s| s > 0 && s <= 61), "kiriman ke-4 ditahan");
        // Undangan lain / IP lain tak ikut tertahan.
        assert!(rl.hit("rsvp:budi:1.2.3.4").is_ok() && rl.hit("rsvp:ani:5.6.7.8").is_ok());
    }

    use super::*;

    #[test]
    fn pembatas() {
        let rl = RateLimit::new(3, Duration::from_secs(300));
        assert!(rl.blocked(&["u:rina"]).is_none());
        for _ in 0..3 {
            rl.fail(&["u:rina", "ip:1.2.3.4"]);
        }
        assert!(rl.blocked(&["u:rina"]).is_some());
        assert!(rl.blocked(&["ip:1.2.3.4"]).is_some());
        assert!(rl.blocked(&["u:sekar"]).is_none());
        rl.clear("u:rina");
        assert!(rl.blocked(&["u:rina"]).is_none());
    }

    #[test]
    fn memori_pembatas_terbatas() {
        let rl = RateLimit::new(5, Duration::from_millis(30));
        for i in 0..1000 {
            rl.fail(&[&format!("ip:{i}")]);
        }
        assert_eq!(rl.len(), 1000);
        std::thread::sleep(Duration::from_millis(40));
        rl.fail(&["ip:baru"]);
        assert_eq!(rl.len(), 1, "entri kedaluwarsa dibuang");
    }

    #[test]
    fn www_ke_apex() {
        assert_eq!(www_target("www.ilyvowcraft.online", "/paket?x=1").as_deref(), Some("https://ilyvowcraft.online/paket?x=1"));
        assert!(www_target("ilyvowcraft.online", "/").is_none());
        assert!(www_target("www.", "/").is_none());
    }

    #[test]
    fn csp_memuat_nonce() {
        let c = csp("abc123", false);
        assert!(c.contains("'nonce-abc123'") && c.contains("'wasm-unsafe-eval'") && c.contains("frame-ancestors 'self'"));
        assert!(!c.contains("ws:"));
        assert!(csp("x", true).contains("ws:"));
        assert!(is_demo_preview("/u/yona-doni", "tema=jawa&pv=1") && is_demo_preview("/u/yona-doni/acara", "pv=1"));
        assert!(!is_demo_preview("/u/yona-doni", "tema=jawa") && !is_demo_preview("/u/budi-ani-x1", "pv=1") && !is_demo_preview("/u/yona-doni-x", "pv=1"));
        assert_eq!(legacy_demo_target("/u/anindita-raditya/acara", Some("tema=x")).as_deref(), Some("/u/yona-doni/acara?tema=x"));
        assert_eq!(legacy_demo_target("/kelola/anindita-raditya", None).as_deref(), Some("/kelola/yona-doni"));
        assert!(legacy_demo_target("/u/anindita-raditya-k7f3", None).is_none() && legacy_demo_target("/tema", None).is_none());
        assert!(is_private("/u/ani?g=AB12") && is_private("/kelola/ani") && is_private("/admin/akun") && !is_private("/paket"));
    }
}
