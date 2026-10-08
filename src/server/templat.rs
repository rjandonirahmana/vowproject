//! server/templat.rs — tema TEMPLAT: HTML + CSS tema disimpan di database.
//!
//! Dua lapis (migrasi 029, hasil bedah everlove — semua tema mereka satu
//! templat induk yang di-clone, bedanya di aset/warna/font):
//!
//!   * `theme_templates` = STRUKTUR + GERAK. HTML adalah templat Jinja
//!     (minijinja, autoescape HTML) yang menerima data undangan (lihat
//!     `page_ctx`); gerak ditulis deklaratif (data-a / data-s / data-idle …)
//!     dan dijalankan SATU mesin `/tata.js` + pustaka `/tata.css`.
//!   * `themes.template` = KULIT. Tema memilih templat, lalu menimpa aset
//!     (`template_assets`) & CSS (`template_css`) sendiri.
//!
//! Middleware `serve` mencegat GET /u/{slug}: bila tema undangan memakai
//! templat, halaman dirender di sini (tanpa WASM); selain itu diteruskan ke
//! Leptos seperti biasa. Tab /u/{slug}/story & /kelola tetap Leptos.
//!
//! Keamanan: templat hanya bisa disunting peran ADMIN, tapi tetap dibatasi —
//! tanpa <script>/iframe/<style>/atribut on*/javascript:/referensi karakter
//! numerik (lagi pula CSP menolak skrip tanpa nonce); CSS lewat pemeriksa url()
//! yang sama dengan animasi; semua data tamu/undangan di-escape otomatis oleh
//! minijinja dan templat tak boleh mematikannya (`|safe`, `autoescape`).
//! Pemeriksaan ini daftar-tolak (bukan parser + daftar-izin): lapis utamanya
//! tetap CSP ber-nonce + peran Admin.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use axum::extract::{Extension, Path, Request};
use axum::http::{header, HeaderMap, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Redirect, Response};
use minijinja::{AutoEscape, Environment, UndefinedBehavior};
use serde::{Deserialize, Serialize};

use super::repo::{self, InvRow};
use super::state::AppState;
use crate::web::fmt;
use crate::web::model::*;

pub const TATA_JS: &str = include_str!("../../templat/tata.js");
pub const TATA_CSS: &str = include_str!("../../templat/tata.css");
/// Batas ukuran sumber templat (HTML/CSS) dari editor admin.
pub const HTML_MAX: usize = 200_000;
pub const CSS_MAX: usize = 120_000;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Templat {
    pub slug: String,
    pub name: String,
    pub html: String,
    pub css: String,
    pub fonts: String,
    pub assets: BTreeMap<String, String>,
    pub builtin: bool,
    pub edited: bool,
}

#[derive(Deserialize)]
struct Meta {
    name: String,
    #[serde(default)]
    fonts: String,
    #[serde(default)]
    assets: BTreeMap<String, String>,
}

fn bawaan(slug: &str, html: &str, css: &str, meta: &str) -> Templat {
    let m: Meta = serde_json::from_str(meta).expect("templat/*/meta.json tidak valid");
    Templat {
        slug: slug.into(),
        name: m.name,
        html: html.into(),
        css: css.into(),
        fonts: m.fonts,
        assets: m.assets,
        builtin: true,
        edited: false,
    }
}

/// Templat bawaan dari berkas repo (templat/<slug>/).
pub fn builtins() -> Vec<Templat> {
    vec![bawaan(
        "kusuma",
        include_str!("../../templat/kusuma/templat.html"),
        include_str!("../../templat/kusuma/gaya.css"),
        include_str!("../../templat/kusuma/meta.json"),
    )]
}

// ── Validasi (dipakai saat seed, simpan admin, dan uji) ───────────────────

pub fn check_html(src: &str) -> Result<(), String> {
    if src.len() > HTML_MAX {
        return Err(format!("HTML templat terlalu panjang (maks {HTML_MAX} karakter)."));
    }
    let low = src.to_ascii_lowercase();
    // <style> di HTML akan melewati check_css; &# / &colon; dipakai menyamarkan
    // "javascript:" di atribut (j&#97;vascript:) dari pencocokan teks di bawah.
    for bad in ["<script", "<iframe", "<object", "<embed", "<base", "<meta", "<link", "<style", "javascript:", "vbscript:", "srcdoc", "&#", "&colon;", "&tab;", "&newline;"] {
        if low.contains(bad) {
            return Err(format!("HTML templat tidak boleh memuat \"{bad}\" — perilaku ditulis lewat atribut data-* (lihat templat/README.md)."));
        }
    }
    // Data tamu wajib tetap di-escape: larang |safe dan blok autoescape.
    let compact: String = low.chars().filter(|c| !c.is_whitespace()).collect();
    if compact.contains("|safe") || compact.contains("autoescape") {
        return Err("HTML templat tidak boleh mematikan escape otomatis (|safe / autoescape).".into());
    }
    // Atribut event inline (onclick=, onload= …).
    let b = low.as_bytes();
    for i in 1..b.len().saturating_sub(3) {
        if b[i] == b'o' && b[i + 1] == b'n' && (b[i - 1].is_ascii_whitespace() || b[i - 1] == b'/') {
            let rest = &low[i + 2..];
            let name: String = rest.chars().take_while(|c| c.is_ascii_alphabetic()).collect();
            if !name.is_empty() && rest[name.len()..].trim_start().starts_with('=') {
                return Err(format!("HTML templat tidak boleh memuat atribut on{name}=."));
            }
        }
    }
    let mut env = Environment::new();
    configure(&mut env);
    env.template_from_str(src).map(|_| ()).map_err(|e| format!("Templat tidak bisa dibaca: {e}"))
}

pub fn check_css(css: &str) -> Result<(), String> {
    if css.len() > CSS_MAX {
        return Err(format!("CSS templat terlalu panjang (maks {CSS_MAX} karakter)."));
    }
    if css.contains('<') {
        return Err("CSS tidak boleh memuat karakter \"<\".".into());
    }
    let low = css.to_ascii_lowercase();
    for bad in ["@import", "expression(", "javascript:", "behavior:", "-moz-binding"] {
        if low.contains(bad) {
            return Err(format!("CSS tidak boleh memuat \"{bad}\"."));
        }
    }
    crate::web::anim::check_css_urls(css)
}

/// `family=` Google Fonts: huruf, angka, dan tanda yang dipakai sintaks css2.
pub fn check_fonts(f: &str) -> bool {
    f.len() <= 400 && f.chars().all(|c| c.is_ascii_alphanumeric() || "+:;@,.&=_-".contains(c))
}

/// URL aset: /lokal (bukan //) atau https.
pub fn check_asset(u: &str) -> bool {
    (u.starts_with('/') && !u.starts_with("//")) || u.starts_with("https://")
}

// ── Cache ──────────────────────────────────────────────────────────────────

pub struct TemplatSet {
    env: Environment<'static>,
    items: HashMap<String, Templat>,
}

fn configure(env: &mut Environment<'static>) {
    env.set_auto_escape_callback(|_| AutoEscape::Html);
    env.set_undefined_behavior(UndefinedBehavior::Chainable);
    env.set_recursion_limit(64);
    // URL foto tanpa fragmen pengaturan (#pos=… / #t=…).
    env.add_filter("foto", |u: String| u.split('#').next().unwrap_or("").to_string());
    // Gaya object-position dari fragmen #pos=x,y,zoom (fmt::photo_style).
    env.add_filter("pos", |u: String| fmt::photo_style(&u));
    env.add_filter("rupiah", |n: i64| fmt::rupiah(n));
    env.add_filter("inisial", |s: String| avatar_initials(&s));
}

impl TemplatSet {
    pub fn new(list: Vec<Templat>) -> Self {
        let mut env = Environment::new();
        configure(&mut env);
        let mut items = HashMap::new();
        for t in list {
            if let Err(e) = env.add_template_owned(t.slug.clone(), t.html.clone()) {
                tracing::warn!(slug = %t.slug, error = %e, "templat tak bisa dikompilasi — dilewati");
                continue;
            }
            items.insert(t.slug.clone(), t);
        }
        Self { env, items }
    }

    pub fn get(&self, slug: &str) -> Option<&Templat> {
        self.items.get(slug)
    }

    pub fn list(&self) -> Vec<&Templat> {
        let mut v: Vec<&Templat> = self.items.values().collect();
        v.sort_by(|a, b| a.slug.cmp(&b.slug));
        v
    }
}

pub fn asset_version() -> &'static str {
    static V: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    V.get_or_init(|| format!("{:08x}", super::util::fnv1a64(&format!("{TATA_JS}{TATA_CSS}")) as u32))
}

pub async fn tata_js() -> Response {
    ([(header::CONTENT_TYPE, "text/javascript; charset=utf-8"), (header::CACHE_CONTROL, "public, max-age=31536000, immutable")], TATA_JS).into_response()
}

pub async fn tata_css() -> Response {
    ([(header::CONTENT_TYPE, "text/css; charset=utf-8"), (header::CACHE_CONTROL, "public, max-age=31536000, immutable")], TATA_CSS).into_response()
}

// ── Data untuk templat ─────────────────────────────────────────────────────

#[derive(Serialize)]
struct Person {
    first: String,
    nick: String,
    name: String,
    degree: String,
    parents: String,
    ig: String,
    photo: String,
    initial: String,
}

#[derive(Serialize)]
struct Ev {
    title: String,
    kind: String,
    /// "Sabtu"
    day_name: String,
    /// "12"
    day: String,
    /// "September 2026"
    month_year: String,
    date_label: String,
    time_label: String,
    venue: String,
    address: String,
    maps: String,
    sessions: Vec<Session>,
}

const BULAN: [&str; 12] = ["Januari", "Februari", "Maret", "April", "Mei", "Juni", "Juli", "Agustus", "September", "Oktober", "November", "Desember"];

fn ev(e: &Event) -> Ev {
    let label = e.date_label();
    let (day, month_year) = match fmt::parse_date(&e.date) {
        Some((y, m, d)) => (d.to_string(), format!("{} {y}", BULAN.get((m - 1).clamp(0, 11) as usize).copied().unwrap_or(""))),
        None => (String::new(), String::new()),
    };
    Ev {
        title: e.title.clone(),
        kind: e.kind.clone(),
        day_name: label.split(',').next().filter(|_| label.contains(',')).unwrap_or("").trim().to_string(),
        day,
        month_year,
        date_label: label,
        time_label: e.time_label(),
        venue: e.venue.clone(),
        address: e.address.clone(),
        maps: e.maps_link(),
        sessions: e.sessions.clone(),
    }
}

fn person(nick: &str, name: &str, degree: &str, parents: &str, ig: &str, photo: &str) -> Person {
    let first = if nick.trim().is_empty() { name.split_whitespace().next().unwrap_or("").to_string() } else { nick.trim().to_string() };
    Person {
        initial: initial(&first),
        first,
        nick: nick.into(),
        name: name.into(),
        degree: degree.into(),
        parents: parents.into(),
        ig: ig.trim_start_matches('@').into(),
        photo: photo.into(),
    }
}

#[derive(Serialize)]
struct WishOut {
    name: String,
    message: String,
    status: String,
    ago: String,
    initials: String,
}

fn wish_out(w: &Wish) -> WishOut {
    WishOut {
        initials: avatar_initials(&w.name),
        name: w.name.clone(),
        message: w.message.clone(),
        status: rsvp_label(&w.status).to_string(),
        ago: w.ago.clone(),
    }
}

struct Page<'a> {
    row: &'a InvRow,
    guest: Option<GuestInfo>,
    to: String,
    qs: String,
    preview: bool,
    wishes: WishPage,
    assets: BTreeMap<String, String>,
    /// Hasil kiriman RSVP tanpa JS (?rsvp=ok|galat setelah 303).
    rsvp_flash: Option<bool>,
}

/// Pesan tetap untuk ?rsvp= — teks tak pernah diambil dari URL.
fn flash_msg(ok: bool) -> &'static str {
    if ok {
        "Terima kasih! Konfirmasi & ucapan Anda telah kami terima."
    } else {
        "Gagal mengirim — periksa isian (nama, status kehadiran, jumlah tamu) lalu coba lagi."
    }
}

fn page_ctx(p: &Page) -> minijinja::Value {
    let inv = &p.row.inv;
    let tamu = p.guest.as_ref().map(|g| g.name.clone()).filter(|n| !n.is_empty()).unwrap_or_else(|| if p.to.is_empty() { "Tamu Undangan".into() } else { p.to.clone() });
    let first = inv.first_event().map(ev);
    let date_num = inv
        .first_event()
        .and_then(|e| fmt::parse_date(&e.date))
        .map(|(y, m, d)| format!("{d:02} . {m:02} . {y}"))
        .unwrap_or_default();
    let mut sessions: Vec<String> = Vec::new();
    for e in &inv.events {
        if e.sessions.is_empty() {
            let t = e.time_start.replace(':', ".");
            sessions.push(if t.is_empty() { e.title.clone() } else { format!("{} ({t} WIB)", e.title) });
        } else {
            sessions.extend(e.sessions.iter().map(|s| format!("{} ({})", s.label, s.time)));
        }
    }
    let gallery: Vec<String> = inv.gallery.iter().filter(|g| !g.is_empty()).cloned().collect();
    minijinja::context! {
        brand => crate::brand!(),
        slug => inv.slug.clone(),
        qs => p.qs.clone(),
        is_demo => inv.is_demo,
        preview => p.preview,
        tamu => tamu,
        guest => p.guest.as_ref().map(|g| minijinja::context! {
            code => g.code.clone(),
            vip => matches!(g.category.as_str(), "vip" | "keluarga"),
            table_no => g.table_no.clone(),
        }),
        couple => inv.couple(),
        bride => person(&inv.bride_nick, &inv.bride_name, &inv.bride_degree, &inv.bride_parents, &inv.bride_ig, &inv.bride_photo),
        groom => person(&inv.groom_nick, &inv.groom_name, &inv.groom_degree, &inv.groom_parents, &inv.groom_ig, &inv.groom_photo),
        date_label => inv.date_label(),
        date_num => date_num,
        first => first,
        events => inv.events.iter().map(ev).collect::<Vec<_>>(),
        countdown_ms => inv.countdown_target_ms(),
        calendar_url => inv.calendar_link(),
        quote => minijinja::context! { text => inv.quote_text.clone(), source => inv.quote_source.clone() },
        love_story => inv.love_story.clone(),
        gallery => gallery,
        cover_photo => inv.cover_photo.clone(),
        video_url => inv.video_url.clone(),
        banks => inv.banks.clone(),
        gift_address => inv.gift_address.clone(),
        family_name => inv.family_name.clone(),
        dress_code => inv.dress_code.clone(),
        dress_colors => inv.dress_colors.clone(),
        live_url => inv.live_url.clone(),
        music => minijinja::context! { url => inv.music_url.clone(), label => inv.music_label(), autoplay => inv.music_autoplay },
        wishes => p.wishes.items.iter().map(wish_out).collect::<Vec<_>>(),
        wish_total => p.wishes.total,
        sessions => sessions,
        rsvp_action => format!("/u/{}/rsvp/kirim", inv.slug),
        story_url => format!("/u/{}/story{}", inv.slug, p.qs),
        prefill_name => if p.guest.is_some() || !p.to.is_empty() { tamu_name(p) } else { String::new() },
        guest_code => p.guest.as_ref().map(|g| g.code.clone()).unwrap_or_default(),
        rsvp_flash => p.rsvp_flash.map(|ok| minijinja::context! { ok => ok, msg => flash_msg(ok) }),
        a => p.assets.clone(),
    }
}

fn tamu_name(p: &Page) -> String {
    p.guest.as_ref().map(|g| g.name.clone()).filter(|n| !n.is_empty()).unwrap_or_else(|| p.to.clone())
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Ikon garis 24×24 untuk navigasi bawah (font Material Symbols tidak
/// dimuat di halaman templat).
const NAV_ICON: [(&str, &str); 5] = [
    ("sampul", "M12 20.5 4.2 13A4.6 4.6 0 0 1 12 6.6 4.6 4.6 0 0 1 19.8 13Z"),
    ("acara", "M5 6h14v14H5zM5 10h14M9 3v4M15 3v4M9 15l2 2 4-4"),
    ("rsvp", "M4 6h16v12H4zM4 7l8 6 8-6"),
    ("story", "M4 8h3l2-3h6l2 3h3v11H4zM12 16.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7"),
    ("kelola", "M12 3l7 3v5c0 4.5-3 8-7 10-4-2-7-5.5-7-10V6zM9 12l2 2 4-4"),
];

/// Navigasi bawah halaman templat — SAMA dengan tema komponen (Sampul ·
/// Acara · Doa & RSVP · Story · Kelola). Ditulis platform, bukan templat:
/// setiap templat otomatis punya, cukup sediakan id `sampul`/`acara`/`ucapan`.
fn bottom_nav(p: &Page, theme: &str) -> String {
    let inv = &p.row.inv;
    let mut items = vec![
        ("#sampul".to_string(), "Sampul", "sampul"),
        ("#acara".to_string(), "Acara", "acara"),
        ("#ucapan".to_string(), "Doa & RSVP", "rsvp"),
        (format!("/u/{}/story{}", inv.slug, p.qs), "Story", "story"),
    ];
    if inv.is_demo {
        items.push((format!("/kelola/{}?key=demo&tema={}", inv.slug, fmt::url_encode(theme)), "Kelola", "kelola"));
    } else if p.preview {
        // Pratinjau pemilik (belum dibayar): kembali ke dashboard Kelola.
        items.push((format!("/kelola/{}", inv.slug), "Kelola", "kelola"));
    }
    let links: String = items
        .iter()
        .map(|(href, label, icon)| {
            let d = NAV_ICON.iter().find(|(k, _)| k == icon).map(|(_, d)| *d).unwrap_or("");
            format!(
                "<a class=\"t-nav__item\" href=\"{}\"><svg viewBox=\"0 0 24 24\" aria-hidden=\"true\"><path d=\"{d}\"/></svg><span>{}</span></a>",
                esc(href),
                esc(label)
            )
        })
        .collect();
    format!("<nav class=\"t-nav\" aria-label=\"Navigasi undangan\">{links}</nav>\n")
}

/// Bungkus hasil templat menjadi dokumen utuh (head, CSS, data, mesin JS).
fn document(st: &AppState, t: &Templat, theme: &crate::web::skin::ThemeInfo, p: &Page, body: &str, nonce: &str) -> String {
    let inv = &p.row.inv;
    let desc = format!("{} — {}. Kami mengundang Anda untuk hadir dan berbagi doa restu.", inv.couple(), inv.date_label());
    let og_image = inv.cover_photo.split('#').next().unwrap_or("");
    let fonts = if t.fonts.is_empty() || !check_fonts(&t.fonts) {
        String::new()
    } else {
        format!(
            "<link rel=\"preconnect\" href=\"https://fonts.gstatic.com\" crossorigin>\n<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com/css2?family={}&display=swap\">\n",
            esc(&t.fonts)
        )
    };
    let data = serde_json::json!({ "slug": inv.slug, "gate_ms": 1500, "video_delay_ms": 700, "video_max_ms": 9000 }).to_string().replace("</", "<\\/");
    // CSS tema di atas CSS templat; keduanya sudah diperiksa saat disimpan,
    // diperiksa ULANG di sini (baris lama/DB disunting langsung).
    let css_ok = |c: &str| if check_css(c).is_ok() { c.to_string() } else { String::new() };
    format!(
        "<!DOCTYPE html>\n<html lang=\"id\">\n<head>\n<meta charset=\"utf-8\">\n\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1, viewport-fit=cover\">\n\
<title>{title}</title>\n<meta name=\"description\" content=\"{desc}\">\n<meta name=\"robots\" content=\"noindex, nofollow\">\n\
<meta property=\"og:title\" content=\"The Wedding of {couple}\">\n<meta property=\"og:description\" content=\"{desc}\">\n{og}\
<link rel=\"icon\" type=\"image/svg+xml\" href=\"/favicon.svg\">\n{fonts}\
<link rel=\"stylesheet\" href=\"/tema.css?v={tv}\">\n<link rel=\"stylesheet\" href=\"/tata.css?v={av}\">\n\
<style>\n{tcss}\n{xcss}\n</style>\n</head>\n<body class=\"t-{tslug} th-{theme} t-has-nav\">\n{body}\n{nav}\
<script type=\"application/json\" id=\"tata-data\">{data}</script>\n\
<script nonce=\"{nonce}\" src=\"/tata.js?v={av}\"></script>\n</body>\n</html>\n",
        title = esc(&format!("Undangan Pernikahan {}", inv.couple())),
        desc = esc(&desc),
        couple = esc(&inv.couple()),
        og = if og_image.is_empty() { String::new() } else { format!("<meta property=\"og:image\" content=\"{}\">\n", esc(og_image)) },
        tv = st.themes().version,
        av = asset_version(),
        tcss = css_ok(&t.css),
        xcss = css_ok(&theme.template_css),
        tslug = esc(&t.slug),
        nav = bottom_nav(p, &theme.slug),
        theme = esc(&theme.slug),
    )
}

/// Render /u/{slug} bila tema undangan memakai templat; None = serahkan ke Leptos.
async fn render(st: &AppState, slug: &str, query: &str, headers: &HeaderMap, nonce: &str) -> anyhow::Result<Option<String>> {
    let Some(mut row) = repo::invitation(&st.pool, slug).await? else { return Ok(None) };
    let q: HashMap<String, String> = url_query(query);
    // Demo "Coba Demo" dengan tema pilihan (sama seperti get_invitation).
    if row.inv.is_demo {
        if let Some(t) = q.get("tema").map(|t| t.trim()).filter(|t| st.themes().get(t).is_some()) {
            row.inv.theme = t.to_string();
        }
    }
    let cat = st.themes();
    let Some(theme) = cat.get(&row.inv.theme).filter(|t| !t.template.is_empty()) else { return Ok(None) };
    let set = st.templat();
    let Some(t) = set.get(&theme.template) else { return Ok(None) };
    // Belum dibayar: hanya pemilik (cookie Kelola) — selain itu Leptos
    // merender layar "Undangan belum aktif" (403).
    let preview = row.inv.is_locked();
    if preview {
        let key = super::owner::key_from(headers, slug).unwrap_or_default();
        if key.is_empty() || !super::auth::same_hash(&super::auth::token_hash(key.trim()), &row.manage_key_hash) {
            return Ok(None);
        }
    }
    let to: String = q.get("to").map(|t| t.chars().take(80).collect()).unwrap_or_default();
    let guest = match q.get("g").map(|g| g.trim()).filter(|g| !g.is_empty() && g.len() <= 12) {
        Some(code) if !preview => repo::open_guest(&st.pool, row.id, code).await?.map(|(_, g)| g),
        _ => None,
    };
    let mut parts = Vec::new();
    match (q.get("g"), to.is_empty()) {
        (Some(g), _) => parts.push(format!("g={}", fmt::url_encode(g))),
        (None, false) => parts.push(format!("to={}", fmt::url_encode(&to))),
        _ => {}
    }
    if row.inv.is_demo {
        parts.push(format!("tema={}", fmt::url_encode(&theme.slug)));
    }
    let qs = if parts.is_empty() { String::new() } else { format!("?{}", parts.join("&")) };
    let wishes = repo::wishes(&st.pool, row.id, 30).await.unwrap_or_default();
    let mut assets = t.assets.clone();
    assets.extend(theme.template_assets.iter().filter(|(_, v)| check_asset(v)).map(|(k, v)| (k.clone(), v.clone())));
    let rsvp_flash = match q.get("rsvp").map(String::as_str) {
        Some("ok") => Some(true),
        Some("galat") => Some(false),
        _ => None,
    };
    let page = Page { row: &row, guest, to, qs, preview, wishes, assets, rsvp_flash };
    let body = set.env.get_template(&t.slug)?.render(page_ctx(&page))?;
    Ok(Some(document(st, t, theme, &page, &body, nonce)))
}

fn url_query(q: &str) -> HashMap<String, String> {
    q.split('&')
        .filter_map(|kv| kv.split_once('=').or(Some((kv, ""))))
        .filter(|(k, _)| !k.is_empty())
        .map(|(k, v)| (url_decode(k), url_decode(v)))
        .collect()
}

fn url_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < b.len() => {
                match u8::from_str_radix(std::str::from_utf8(&b[i + 1..i + 3]).unwrap_or("zz"), 16) {
                    Ok(v) => {
                        out.push(v);
                        i += 2;
                    }
                    Err(_) => out.push(b'%'),
                }
            }
            c => out.push(c),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Middleware: GET /u/{slug} bertema templat → render di sini.
pub async fn serve(req: Request, next: Next) -> Response {
    if req.method() != Method::GET && req.method() != Method::HEAD {
        return next.run(req).await;
    }
    let path = req.uri().path();
    let Some(slug) = path.strip_prefix("/u/").map(|s| s.trim_end_matches('/')).filter(|s| !s.is_empty() && !s.contains('/')) else {
        return next.run(req).await;
    };
    let slug = slug.to_string();
    let Some(st) = req.extensions().get::<Arc<AppState>>().cloned() else { return next.run(req).await };
    let nonce = req.extensions().get::<super::security::CspNonce>().map(|n| n.0.clone()).unwrap_or_default();
    let query = req.uri().query().unwrap_or("").to_string();
    match render(&st, &slug, &query, req.headers(), &nonce).await {
        Ok(Some(html)) => ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], html).into_response(),
        Ok(None) => next.run(req).await,
        Err(e) => {
            tracing::error!(slug = %slug, error = %format!("{e:#}"), "render templat");
            next.run(req).await
        }
    }
}

/// POST /u/{slug}/rsvp/kirim — formulir RSVP tema templat. Dengan JS (Accept
/// JSON) → {ok, msg, wish, wish_total, baru}; tanpa JS → 303 kembali ke bagian
/// ucapan dengan ?rsvp=ok|galat (kode tamu ikut) agar tamu tahu hasilnya.
pub async fn rsvp_kirim(
    Extension(st): Extension<Arc<AppState>>,
    Path(slug): Path<String>,
    headers: HeaderMap,
    axum::Form(f): axum::Form<HashMap<String, String>>,
) -> Response {
    let json = headers.get(header::ACCEPT).and_then(|v| v.to_str().ok()).is_some_and(|v| v.contains("application/json"));
    let get = |k: &str| f.get(k).cloned().unwrap_or_default();
    let opt = |k: &str| f.get(k).cloned().filter(|v| !v.is_empty());
    let res = match repo::invitation(&st.pool, &slug).await {
        Ok(Some(row)) => {
            let inv_id = row.id;
            let ip = super::security::client_ip(&headers);
            let input = crate::web::api::RsvpInput {
                guest: opt("guest"),
                name: get("name"),
                phone: opt("phone"),
                status: get("status"),
                pax: opt("pax"),
                session: opt("session"),
                message: opt("message"),
            };
            match crate::web::api::rsvp_core(&st, &row, &ip, input).await {
                // Jumlah ucapan dari DB: kiriman ulang tamu terdaftar memperbarui
                // baris yang sama, jadi penghitung di layar tak boleh sekadar +1.
                Ok((msg, baru)) => Ok((msg, baru, repo::wish_total(&st.pool, inv_id).await.ok())),
                Err(e) => Err(e),
            }
        }
        Ok(None) => Err("Undangan tidak ditemukan.".into()),
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "rsvp templat");
            Err("Server sedang sibuk, coba lagi sebentar.".into())
        }
    };
    if !json {
        let g = get("guest");
        let g = g.trim();
        let g = if g.is_empty() || g.len() > 12 { String::new() } else { format!("g={}&", fmt::url_encode(g)) };
        let hasil = if res.is_ok() { "ok" } else { "galat" };
        return Redirect::to(&format!("/u/{}?{g}rsvp={hasil}#ucapan", fmt::url_encode(&slug))).into_response();
    }
    let body = match res {
        Ok((msg, baru, total)) => {
            let message = super::handlers::clean(&get("message"), 600);
            let wish = (!message.is_empty()).then(|| {
                wish_out(&Wish { name: super::handlers::clean(&get("name"), 80), status: get("status"), message, ago: "baru saja".into() })
            });
            serde_json::json!({ "ok": true, "msg": msg, "wish": wish, "wish_total": total, "baru": baru })
        }
        Err(msg) => serde_json::json!({ "ok": false, "msg": msg }),
    };
    let code = if body["ok"] == true { StatusCode::OK } else { StatusCode::UNPROCESSABLE_ENTITY };
    (code, axum::Json(body)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn templat_bawaan_lolos_validasi() {
        for t in builtins() {
            check_html(&t.html).unwrap_or_else(|e| panic!("{}: {e}", t.slug));
            check_css(&t.css).unwrap_or_else(|e| panic!("{}: {e}", t.slug));
            assert!(check_fonts(&t.fonts), "{}: fonts", t.slug);
            for (k, v) in &t.assets {
                assert!(check_asset(v), "{}: aset {k} = {v}", t.slug);
            }
        }
    }

    #[test]
    fn html_berbahaya_ditolak() {
        assert!(check_html("<div onclick=\"x()\">a</div>").is_err());
        assert!(check_html("<img src=x onerror = 'a'>").is_err());
        assert!(check_html("<script>alert(1)</script>").is_err());
        assert!(check_html("<a href=\"javascript:alert(1)\">a</a>").is_err());
        assert!(check_html("<a href=\"j&#97;vascript:alert(1)\">a</a>").is_err());
        assert!(check_html("<a href=\"javascript&colon;alert(1)\">a</a>").is_err());
        assert!(check_html("<p>{{ tamu | safe }}</p>").is_err());
        assert!(check_html("{% autoescape false %}{{ tamu }}{% endautoescape %}").is_err());
        assert!(check_html("<style>@import url(//x)</style>").is_err());
        assert!(check_html("<p class=\"none\">{{ tamu }}</p>").is_ok());
        assert!(check_html("{% if %}").is_err());
    }

    #[test]
    fn query_didekode() {
        let q = url_query("to=Bapak+Budi%20%26+Ibu&g=AB12");
        assert_eq!(q.get("to").map(String::as_str), Some("Bapak Budi & Ibu"));
        assert_eq!(q.get("g").map(String::as_str), Some("AB12"));
    }

    #[test]
    fn templat_kusuma_merender_data_demo() {
        let set = TemplatSet::new(builtins());
        let t = set.get("kusuma").unwrap();
        let inv = Invitation {
            slug: "yona-doni".into(),
            bride_name: "Yona".into(),
            groom_name: "Doni".into(),
            events: vec![Event { title: "Akad Nikah".into(), date: "2026-09-12".into(), time_start: "08:00".into(), ..Default::default() }],
            banks: vec![Bank { bank: "BCA".into(), number: "123".into(), holder: "Yona".into() }],
            ..Default::default()
        };
        let row = InvRow { id: 1, inv, manage_key_hash: String::new(), total_price: 0, payment_method: String::new(), contact_phone: String::new() };
        let page = Page { row: &row, guest: None, to: "<b>Budi</b>".into(), qs: String::new(), preview: false, wishes: WishPage::default(), assets: t.assets.clone(), rsvp_flash: Some(false) };
        let out = set.env.get_template("kusuma").unwrap().render(page_ctx(&page)).unwrap();
        assert!(out.contains("Yona") && out.contains("Doni"));
        assert!(out.contains("&lt;b&gt;Budi") && !out.contains("<b>Budi"), "nama tamu wajib di-escape");
        assert!(out.contains("Sabtu") && out.contains("September 2026"));
        assert!(out.contains(flash_msg(false)), "pesan ?rsvp=galat tampil tanpa JS");
        // Templat bawaan wajib punya jangkar yang dipakai navigasi bawah.
        for id in ["sampul", "acara", "ucapan"] {
            assert!(out.contains(&format!("id=\"{id}\"")), "jangkar #{id} untuk navigasi bawah");
        }
        let nav = bottom_nav(&page, "kusuma-jawi");
        assert!(nav.contains("/u/yona-doni/story") && nav.contains("#ucapan") && !nav.contains("Kelola"), "{nav}");
    }
}
