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
use crate::tx;
use crate::web::i18n::Lang;
use crate::web::fmt::{self, html_escape as esc};
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
    vec![
        bawaan(
            "kusuma",
            include_str!("../../templat/kusuma/templat.html"),
            include_str!("../../templat/kusuma/gaya.css"),
            include_str!("../../templat/kusuma/meta.json"),
        ),
        bawaan(
            "warkah",
            include_str!("../../templat/warkah/templat.html"),
            include_str!("../../templat/warkah/gaya.css"),
            include_str!("../../templat/warkah/meta.json"),
        ),
    ]
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
    V.get_or_init(|| super::util::hash8(&format!("{TATA_JS}{TATA_CSS}")))
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
    /// "September" / "2026" — tanggal terbelah (SABTU | 12 | SEPTEMBER / 2026).
    month: String,
    year: String,
    /// Kalender bulan acara: sel kosong sebelum tanggal 1 (minggu mulai
    /// Senin) & jumlah hari — templat cukup `range()`.
    cal_lead: i64,
    cal_days: i64,
    /// "08.00" (jam mulai saja) — baris susunan acara.
    time_start: String,
    date_label: String,
    time_label: String,
    venue: String,
    address: String,
    maps: String,
    sessions: Vec<Session>,
}

fn ev(e: &Event, l: Lang) -> Ev {
    let label = e.date_label_in(l);
    let parsed = fmt::parse_date(&e.date);
    let bulan: &[&str; 12] = match l {
        Lang::Id => &fmt::BULAN,
        Lang::En => &fmt::MONTH_EN,
    };
    let month = parsed.map(|(_, m, _)| bulan.get((m - 1).clamp(0, 11) as usize).copied().unwrap_or("").to_string()).unwrap_or_default();
    let year = parsed.map(|(y, _, _)| y.to_string()).unwrap_or_default();
    let day = parsed.map(|(_, _, d)| d.to_string()).unwrap_or_default();
    let month_year = if month.is_empty() { String::new() } else { format!("{month} {year}") };
    let (cal_lead, cal_days) = parsed.map(|(y, m, _)| (fmt::weekday_mon0(y, m, 1), fmt::days_in_month(y, m))).unwrap_or((0, 0));
    Ev {
        title: e.title.clone(),
        kind: e.kind.clone(),
        day_name: label.split(',').next().filter(|_| label.contains(',')).unwrap_or("").trim().to_string(),
        day,
        month_year,
        month,
        year,
        cal_lead,
        cal_days,
        time_start: e.time_start.trim().replace(':', "."),
        date_label: label,
        time_label: e.time_label_in(l),
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

fn wish_out(w: &Wish, l: Lang) -> WishOut {
    WishOut {
        initials: avatar_initials(&w.name),
        name: w.name.clone(),
        message: w.message.clone(),
        status: rsvp_label_in(&w.status, l).to_string(),
        ago: w.ago_in(l),
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
    /// Bahasa teks bawaan (?lang= tamu, atau pilihan pasangan).
    lang: Lang,
    /// Tautan tombol ganti bahasa (bahasa lainnya, query lain tetap).
    lang_href: String,
}

/// Pesan tetap untuk ?rsvp= — teks tak pernah diambil dari URL.
fn flash_msg(ok: bool, l: Lang) -> &'static str {
    if ok {
        tx!(l, "Terima kasih! Konfirmasi & ucapan Anda telah kami terima.", "Thank you! We have received your RSVP and wishes.")
    } else {
        tx!(l, "Gagal mengirim — periksa isian (nama, status kehadiran, jumlah tamu) lalu coba lagi.", "Could not send — please check your name, attendance and number of guests, then try again.")
    }
}

/// Frasa bawaan templat dalam bahasa `l` — templat menulis `{{ t.buka }}`.
/// SATU tempat untuk semua teks tetap templat (kusuma, warkah, …); teks yang
/// ditulis pasangan tetap apa adanya.
fn teks(l: Lang) -> minijinja::Value {
    macro_rules! t {
        ($($k:ident: $id:expr, $en:expr;)*) => {
            minijinja::context! { $($k => tx!(l, $id, $en),)* }
        };
    }
    t! {
        kepada: "Kepada Yth.", "To";
        kepada_lengkap: "Kepada Yth. Bapak/Ibu/Saudara/i", "Dear honoured guest";
        sapaan: "Bapak/Ibu/Saudara/i", "Dear honoured guest";
        maaf_nama: "*Mohon maaf apabila ada kesalahan penulisan nama/gelar", "*Please forgive any misspelling of names or titles";
        tamu_spesial: "Tamu Spesial", "Special Guest";
        buka: "Buka Undangan", "Open Invitation";
        tutup: "Tutup", "Close";
        tutup_undangan: "Tutup Undangan", "Close Invitation";
        gulir: "Gulir ke bawah", "Scroll down";
        sampul_label: "Sampul undangan", "Invitation cover";
        salam: "Assalamu’alaikum Warahmatullahi Wabarakatuh", "Assalamu’alaikum Warahmatullahi Wabarakatuh";
        pembuka_doa: "Maha Suci Allah yang telah menciptakan makhluk-Nya berpasang-pasangan. Ya Allah, semoga ridho-Mu tercurah mengiringi pernikahan kami.",
            "Glory be to Allah, who created all things in pairs. O Allah, may Your blessings accompany our marriage.";
        ayat: "Dan di antara tanda-tanda kebesaran-Nya ialah Dia menciptakan pasangan-pasangan untukmu dari jenismu sendiri, agar kamu cenderung dan merasa tenteram kepadanya, dan Dia menjadikan di antaramu rasa kasih dan sayang.",
            "And among His signs is that He created for you spouses from among yourselves, that you may find tranquillity in them; and He placed between you affection and mercy.";
        ayat_sumber: "Q.S. Ar-Rum : 21", "Qur'an, Ar-Rum 30:21";
        mempelai: "Mempelai", "The Couple";
        putri_dari: "Putri dari", "Daughter of";
        putra_dari: "Putra dari", "Son of";
        ortu_kosong: "Bapak & Ibu", "Mr. & Mrs.";
        info_lead: "Dengan memohon rahmat dan ridho Allah SWT, kami bermaksud menyelenggarakan pernikahan putra-putri kami",
            "By the grace of Allah SWT, we joyfully announce the marriage of our beloved children";
        hari_h: "Hari pernikahan, tanggal", "Wedding day,";
        hari: "hari", "days";
        jam: "jam", "hours";
        menit: "menit", "minutes";
        detik: "detik", "seconds";
        simpan_tanggal: "Simpan Tanggal", "Save the Date";
        simpan_kalender: "Simpan ke Kalender", "Add to Calendar";
        konfirmasi_hadir: "Konfirmasi Kehadiran", "RSVP";
        waktu_tempat: "Waktu & Tempat", "Time & Venue";
        lokasi: "Lokasi Acara", "Venue";
        pukul: "Pukul", "At";
        petunjuk: "Petunjuk Arah", "Get Directions";
        susunan: "Susunan Acara", "Schedule";
        live: "Live Streaming", "Live Streaming";
        live_teks: "Saksikan acara kami secara virtual.", "Watch our ceremony online.";
        live_ajak: "Temui kami secara virtual untuk menyaksikan acara pernikahan kami.", "Join us online to witness our wedding.";
        live_tonton: "Tonton Live", "Watch Live";
        live_lihat: "Lihat Live Streaming", "Watch Live Streaming";
        kisah: "Kisah Kami", "Our Story";
        galeri: "Galeri Foto", "Gallery";
        foto_sebelum: "Foto sebelumnya", "Previous photo";
        foto_berikut: "Foto berikutnya", "Next photo";
        tanda_kasih: "Tanda Kasih", "Wedding Gift";
        kado_lead: "Doa restu Anda adalah karunia terindah bagi kami. Bila ingin memberi tanda kasih, ketuk kotak di bawah ini.",
            "Your blessings are the greatest gift to us. If you wish to send a gift, tap the box below.";
        kado_lead_panjang: "Doa restu Anda merupakan karunia yang sangat berarti bagi kami. Dan jika memberi adalah ungkapan tanda kasih, Anda dapat memberi melalui di bawah ini.",
            "Your blessings mean the world to us. Should you wish to give a gift, you may do so below.";
        ketuk_buka: "Ketuk untuk membuka", "Tap to open";
        no_rek: "No. Rekening", "Account Number";
        atas_nama: "Atas Nama", "Account Name";
        an: "a.n.", "Account name:";
        kirim_kado: "Kirim Kado", "Send a Gift";
        alamat_penerima: "Alamat Penerima", "Recipient Address";
        salin: "Salin", "Copy";
        salin_nomor: "Salin Nomor", "Copy Number";
        salin_alamat: "Salin Alamat", "Copy Address";
        buku_tamu: "Buku Tamu", "Guestbook";
        ucapan_lead: "Kirim doa, ucapan, dan konfirmasi kehadiran Anda.", "Send your wishes and let us know if you can attend.";
        ucapan_ajak: "Berikan doa dan ucapan terbaik untuk kami.", "Send us your best wishes and prayers.";
        demo: "Ini undangan demo — kiriman hanya contoh dan tidak disimpan.", "This is a demo invitation — submissions are examples and are not saved.";
        nama: "Nama", "Name";
        ucapan_doa: "Ucapan & doa", "Wishes";
        hadir: "Hadir", "Attending";
        tidak_hadir: "Tidak Hadir", "Not attending";
        jumlah_tamu: "Jumlah tamu", "Number of guests";
        orang: "orang", "guest(s)";
        sesi: "Sesi", "Session";
        kirim: "Kirim", "Send";
        kirim_ucapan: "Kirim Ucapan", "Send Wishes";
        ucapan_n: "ucapan", "wishes";
        pertama: "Jadilah yang pertama memberi ucapan.", "Be the first to send your wishes.";
        story_link: "Lihat & bagikan Story Tamu →", "View & share guest stories →";
        terima_kasih: "Terima Kasih", "Thank You";
        penutup: "Merupakan suatu kebahagiaan dan kehormatan bagi kami apabila Bapak/Ibu/Saudara/i berkenan hadir dan memberikan doa restu.",
            "It would be a joy and an honour for us if you could attend and give us your blessings.";
        yang_berbahagia: "Kami yang berbahagia", "With love,";
        oleh: "Undangan digital oleh", "Digital invitation by";
        musik: "Putar / jeda musik", "Play / pause music";
    }
}

fn page_ctx(p: &Page) -> minijinja::Value {
    let inv = &p.row.inv;
    let l = p.lang;
    let tamu = p.guest.as_ref().map(|g| g.name.clone()).filter(|n| !n.is_empty()).unwrap_or_else(|| if p.to.is_empty() { tx!(l, "Tamu Undangan", "Dear Guest").into() } else { p.to.clone() });
    let first = inv.first_event().map(|e| ev(e, l));
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
        date_label => inv.date_label_in(l),
        date_num => date_num,
        first => first,
        events => inv.events.iter().map(|e| ev(e, l)).collect::<Vec<_>>(),
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
        wishes => p.wishes.items.iter().map(|w| wish_out(w, l)).collect::<Vec<_>>(),
        wish_total => p.wishes.total,
        sessions => sessions,
        rsvp_action => format!("/u/{}/rsvp/kirim", inv.slug),
        story_url => format!("/u/{}/story{}", inv.slug, p.qs),
        prefill_name => if p.guest.is_some() || !p.to.is_empty() { tamu_name(p) } else { String::new() },
        guest_code => p.guest.as_ref().map(|g| g.code.clone()).unwrap_or_default(),
        rsvp_flash => p.rsvp_flash.map(|ok| minijinja::context! { ok => ok, msg => flash_msg(ok, l) }),
        lang => l.code(),
        t => teks(l),
        a => p.assets.clone(),
    }
}

fn tamu_name(p: &Page) -> String {
    p.guest.as_ref().map(|g| g.name.clone()).filter(|n| !n.is_empty()).unwrap_or_else(|| p.to.clone())
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
    let l = p.lang;
    let mut items = vec![
        ("#sampul".to_string(), tx!(l, "Sampul", "Cover"), "sampul"),
        ("#acara".to_string(), tx!(l, "Acara", "Events"), "acara"),
        ("#ucapan".to_string(), tx!(l, "Doa & RSVP", "Wishes & RSVP"), "rsvp"),
        (format!("/u/{}/story{}", inv.slug, p.qs), "Story", "story"),
    ];
    if inv.is_demo {
        items.push((format!("/kelola/{}?key=demo&tema={}", inv.slug, fmt::url_encode(theme)), tx!(l, "Kelola", "Manage"), "kelola"));
    } else if p.preview {
        // Pratinjau pemilik (belum dibayar): kembali ke dashboard Kelola.
        items.push((format!("/kelola/{}", inv.slug), tx!(l, "Kelola", "Manage"), "kelola"));
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
    format!("<nav class=\"t-nav\" aria-label=\"{}\">{links}</nav>\n", tx!(l, "Navigasi undangan", "Invitation navigation"))
}

/// Tombol "Tutup Undangan" + ganti bahasa — ditulis platform seperti
/// navigasi bawah, jadi setiap templat punya; tata.js menyembunyikan tombol
/// tutup bila templat tanpa gerbang.
fn top_buttons(p: &Page) -> String {
    let l = p.lang;
    format!(
        "<button type=\"button\" class=\"t-close\" data-close aria-label=\"{aria}\" title=\"{aria}\">\
<svg viewBox=\"0 0 24 24\" aria-hidden=\"true\"><path d=\"M3.5 8.5 12 3l8.5 5.5v10a1.5 1.5 0 0 1-1.5 1.5H5a1.5 1.5 0 0 1-1.5-1.5z\"/><path d=\"m3.5 9 8.5 6 8.5-6\"/></svg>\
<span>{tutup}</span></button>\n\
<a class=\"t-lang\" href=\"{href}\" hreflang=\"{other}\" aria-label=\"{lang_aria}\">{other_up}</a>\n",
        aria = tx!(l, "Tutup undangan, kembali ke sampul", "Close the invitation, back to the cover"),
        tutup = tx!(l, "Tutup", "Close"),
        href = esc(&p.lang_href),
        other = l.other().code(),
        other_up = l.other().code().to_uppercase(),
        lang_aria = tx!(l, "Baca dalam bahasa Inggris", "Read in Indonesian"),
    )
}

/// Bungkus hasil templat menjadi dokumen utuh (head, CSS, data, mesin JS).
fn document(st: &AppState, t: &Templat, theme: &crate::web::skin::ThemeInfo, p: &Page, body: &str, nonce: &str) -> String {
    let inv = &p.row.inv;
    let l = p.lang;
    let desc = format!(
        "{} — {}. {}",
        inv.couple(),
        inv.date_label_in(l),
        tx!(l, "Kami mengundang Anda untuk hadir dan berbagi doa restu.", "We joyfully invite you to celebrate with us and share your blessings.")
    );
    let og_image = inv.cover_photo.split('#').next().unwrap_or("");
    let fonts = if t.fonts.is_empty() || !check_fonts(&t.fonts) {
        String::new()
    } else {
        format!("<link rel=\"stylesheet\" href=\"{}\">\n", esc(&fmt::font_css(&[t.fonts.as_str()], "swap")))
    };
    let data = serde_json::json!({ "slug": inv.slug, "gate_ms": 1500, "video_delay_ms": 700, "video_max_ms": 9000 }).to_string().replace("</", "<\\/");
    // CSS tema di atas CSS templat; keduanya sudah diperiksa saat disimpan,
    // diperiksa ULANG di sini (baris lama/DB disunting langsung).
    let css_ok = |c: &str| if check_css(c).is_ok() { c.to_string() } else { String::new() };
    format!(
        "<!DOCTYPE html>\n<html lang=\"{html_lang}\">\n<head>\n<meta charset=\"utf-8\">\n\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1, viewport-fit=cover\">\n\
<title>{title}</title>\n<meta name=\"description\" content=\"{desc}\">\n<meta name=\"robots\" content=\"noindex, nofollow\">\n\
<meta property=\"og:title\" content=\"The Wedding of {couple}\">\n<meta property=\"og:description\" content=\"{desc}\">\n{og}\
<link rel=\"icon\" type=\"image/svg+xml\" href=\"/favicon.svg\">\n{fonts}\
<link rel=\"stylesheet\" href=\"/tema.css?v={tv}\">\n<link rel=\"stylesheet\" href=\"/tata.css?v={av}\">\n\
<style>\n{tcss}\n{xcss}\n</style>\n</head>\n<body class=\"t-{tslug} th-{theme} t-has-nav\">\n{body}\n{buttons}{nav}\
<script type=\"application/json\" id=\"tata-data\">{data}</script>\n\
<script nonce=\"{nonce}\" src=\"/tata.js?v={av}\"></script>\n</body>\n</html>\n",
        title = esc(&format!("{} {}", tx!(l, "Undangan Pernikahan", "Wedding Invitation of"), inv.couple())),
        desc = esc(&desc),
        couple = esc(&inv.couple()),
        og = if og_image.is_empty() { String::new() } else { format!("<meta property=\"og:image\" content=\"{}\">\n", esc(og_image)) },
        tv = st.themes().version,
        av = asset_version(),
        tcss = css_ok(&t.css),
        xcss = css_ok(&theme.template_css),
        tslug = esc(&t.slug),
        nav = bottom_nav(p, &theme.slug),
        buttons = top_buttons(p),
        theme = esc(&theme.slug),
        html_lang = p.lang.code(),
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
        let owner = !key.is_empty() && super::auth::same_hash(&super::auth::token_hash(key.trim()), &row.manage_key_hash);
        if !owner && super::auth::require(st, headers, true).await.is_err() {
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
    // Bahasa: ?lang= pilihan tamu (dibawa antar tab) atau pilihan pasangan.
    let chosen = q.get("lang").and_then(|l| Lang::from_code(l));
    let lang = chosen.unwrap_or_else(|| row.inv.language());
    let lang_href = {
        let mut p2 = parts.clone();
        p2.push(format!("lang={}", lang.other().code()));
        format!("/u/{}?{}", row.inv.slug, p2.join("&"))
    };
    if let Some(l) = chosen {
        parts.push(format!("lang={}", l.code()));
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
    let page = Page { row: &row, guest, to, qs, preview, wishes, assets, rsvp_flash, lang, lang_href };
    let body = unslash(set.env.get_template(&t.slug)?.render(page_ctx(&page))?);
    Ok(Some(document(st, t, theme, &page, &body, nonce)))
}

/// Autoescape minijinja menulis "/" sebagai `&#x2f;`. Di HTML artinya sama
/// persis, tapi URL "&#x2f;img&#x2f;…" lolos dari penulis-ulang aset RustFS
/// (server/aset.rs) → tiap gambar kena 301. Templat dilarang memuat <script>
/// / <style>, jadi semua kemunculannya ada di teks/atribut HTML — aman dibalik.
fn unslash(html: String) -> String {
    if html.contains("&#x2f;") { html.replace("&#x2f;", "/") } else { html }
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
    // Bahasa halaman tempat formulir dikirim (input tersembunyi `lang`).
    let l = Lang::of(&get("lang"));
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
                lang: if get("lang").is_empty() { row.inv.language() } else { l },
            };
            match crate::web::api::rsvp_core(&st, &row, &ip, input).await {
                // Jumlah ucapan dari DB: kiriman ulang tamu terdaftar memperbarui
                // baris yang sama, jadi penghitung di layar tak boleh sekadar +1.
                Ok((msg, baru)) => Ok((msg, baru, repo::wish_total(&st.pool, inv_id).await.ok())),
                Err(e) => Err(e),
            }
        }
        Ok(None) => Err(tx!(l, "Undangan tidak ditemukan.", "Invitation not found.").into()),
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "rsvp templat");
            Err(tx!(l, "Server sedang sibuk, coba lagi sebentar.", "The server is busy, please try again in a moment.").into())
        }
    };
    if !json {
        let g = get("guest");
        let g = g.trim();
        let g = if g.is_empty() || g.len() > 12 { String::new() } else { format!("g={}&", fmt::url_encode(g)) };
        let hasil = if res.is_ok() { "ok" } else { "galat" };
        let lq = if get("lang").is_empty() { String::new() } else { format!("lang={}&", l.code()) };
        return Redirect::to(&format!("/u/{}?{g}{lq}rsvp={hasil}#ucapan", fmt::url_encode(&slug))).into_response();
    }
    let body = match res {
        Ok((msg, baru, total)) => {
            let message = super::handlers::clean(&get("message"), 600);
            let wish = (!message.is_empty()).then(|| {
                wish_out(&Wish { name: super::handlers::clean(&get("name"), 80), status: get("status"), message, ago: "baru saja".into(), age: 0 }, l)
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
    fn hari_dalam_minggu() {
        // 1 Okt 2026 = Kamis, 1 Sep 2026 = Selasa, 1 Feb 2024 = Kamis (kabisat), 1 Mar 2026 = Minggu.
        assert_eq!(fmt::weekday_mon0(2026, 10, 1), 3);
        assert_eq!(fmt::weekday_mon0(2026, 9, 1), 1);
        assert_eq!(fmt::weekday_mon0(2024, 2, 1), 3);
        assert_eq!(fmt::weekday_mon0(2026, 3, 1), 6);
    }

    #[test]
    fn templat_bawaan_merender_data_demo() {
        for slug in ["kusuma", "warkah"] {
            render_demo(slug);
        }
    }

    /// Pratinjau tanpa DB/server: `cargo test --lib pratinjau_templat -- --ignored`
    /// menulis body templat (data demo lengkap) + CSS-nya ke target/pratinjau/
    /// untuk dibungkus & dibuka di browser (mis. audit piksel Playwright).
    #[test]
    #[ignore]
    fn pratinjau_templat() {
        let set = TemplatSet::new(builtins());
        let ev = |title: &str, date: &str, t0: &str, t1: &str, venue: &str| Event {
            title: title.into(),
            date: date.into(),
            time_start: t0.into(),
            time_end: t1.into(),
            venue: venue.into(),
            address: "Jl. Dharmawangsa VIII No. 12, Jakarta Selatan".into(),
            ..Default::default()
        };
        let foto = |n: &str| format!("/img/layanan/{n}.jpg");
        let inv = Invitation {
            slug: "yona-doni".into(),
            is_demo: true,
            bride_name: "Yona Ayu Lestari".into(),
            bride_nick: "Yona".into(),
            bride_parents: "Bpk. Hendra Wijaya & Ibu Ratna Sari".into(),
            groom_name: "Doni Prasetyo".into(),
            groom_nick: "Doni".into(),
            groom_parents: "Bpk. Agus Salim & Ibu Dewi Kartika".into(),
            cover_photo: foto("mua-sekar"),
            gallery: ["mua-sekar", "dekor-villa", "mua-modern", "dekor-nature", "venue-atsiri", "dekor-jawa"].iter().map(|n| foto(n)).collect(),
            events: vec![
                ev("Akad Nikah", "2026-10-24", "08:00", "10:00", "Masjid Agung Al-Azhar"),
                ev("Resepsi", "2026-10-24", "11:00", "14:00", "Gedung Kirana Ballroom"),
            ],
            dress_code: "Nuansa bumi & pastel".into(),
            dress_colors: vec![DressColor { name: "Cokelat".into(), hex: "#6b4f3a".into() }, DressColor { name: "Pasir".into(), hex: "#d8c3a5".into() }, DressColor { name: "Sage".into(), hex: "#9caf88".into() }],
            banks: vec![Bank { bank: "BCA".into(), number: "1234567890".into(), holder: "Yona Ayu Lestari".into() }],
            gift_address: "Jl. Melati No. 8, Bandung".into(),
            family_name: "Keluarga Besar Wijaya & Salim".into(),
            music_url: "/music/contoh.mp3".into(),
            ..Default::default()
        };
        let row = InvRow { id: 1, inv, manage_key_hash: String::new(), total_price: 0, payment_method: String::new(), contact_phone: String::new() };
        let wishes = WishPage {
            total: 2,
            items: vec![
                Wish { name: "Budi Santoso".into(), status: "hadir".into(), message: "Selamat menempuh hidup baru, semoga sakinah mawaddah warahmah.".into(), ago: "5 jam yang lalu".into(), age: 5 * 3600 },
                Wish { name: "Siti Sarah".into(), status: "tidak".into(), message: "Barakallahu lakuma, mohon maaf belum bisa hadir.".into(), ago: "1 hari yang lalu".into(), age: 86_400 },
            ],
        };
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/pratinjau");
        std::fs::create_dir_all(&dir).unwrap();
        for t in &builtins() {
            let page = Page { row: &row, guest: None, to: "Bapak Budi & Keluarga".into(), qs: String::new(), preview: false, wishes: wishes.clone(), assets: t.assets.clone(), rsvp_flash: None, lang: Lang::Id, lang_href: String::new() };
            let body = unslash(set.env.get_template(&t.slug).unwrap().render(page_ctx(&page)).unwrap());
            std::fs::write(dir.join(format!("{}.body.html", t.slug)), body + &bottom_nav(&page, &t.slug)).unwrap();
            std::fs::write(dir.join(format!("{}.css", t.slug)), &t.css).unwrap();
            std::fs::write(dir.join(format!("{}.fonts", t.slug)), &t.fonts).unwrap();
        }
    }

    #[test]
    fn templat_berbahasa_inggris() {
        let set = TemplatSet::new(builtins());
        for slug in ["kusuma", "warkah"] {
            let t = set.get(slug).unwrap();
            let inv = Invitation {
                slug: "yona-doni".into(),
                bride_name: "Yona".into(),
                groom_name: "Doni".into(),
                lang: "en".into(),
                events: vec![Event { title: "Akad".into(), date: "2026-09-12".into(), time_start: "08:00".into(), ..Default::default() }],
                ..Default::default()
            };
            let row = InvRow { id: 1, inv, manage_key_hash: String::new(), total_price: 0, payment_method: String::new(), contact_phone: String::new() };
            let page = Page { row: &row, guest: None, to: String::new(), qs: String::new(), preview: false, wishes: WishPage::default(), assets: t.assets.clone(), rsvp_flash: None, lang: Lang::En, lang_href: String::new() };
            let out = set.env.get_template(slug).unwrap().render(page_ctx(&page)).unwrap();
            assert!(out.contains("Saturday") && out.contains("September 2026"), "{slug}: tanggal Inggris");
            assert!(out.contains("Open Invitation") && !out.contains("Buka Undangan"), "{slug}: tombol Inggris");
            assert!(out.contains("name=\"lang\" value=\"en\""), "{slug}: RSVP membawa bahasa");
            assert!(!out.contains("Kepada Yth"), "{slug}: sapaan masih Indonesia");
        }
    }

    fn render_demo(slug: &str) {
        let set = TemplatSet::new(builtins());
        let t = set.get(slug).unwrap();
        let inv = Invitation {
            slug: "yona-doni".into(),
            bride_name: "Yona".into(),
            groom_name: "Doni".into(),
            events: vec![Event { title: "Akad Nikah".into(), date: "2026-09-12".into(), time_start: "08:00".into(), ..Default::default() }],
            banks: vec![Bank { bank: "BCA".into(), number: "123".into(), holder: "Yona".into() }],
            ..Default::default()
        };
        let row = InvRow { id: 1, inv, manage_key_hash: String::new(), total_price: 0, payment_method: String::new(), contact_phone: String::new() };
        let page = Page { row: &row, guest: None, to: "<b>Budi</b>".into(), qs: String::new(), preview: false, wishes: WishPage::default(), assets: t.assets.clone(), rsvp_flash: Some(false), lang: Lang::Id, lang_href: String::new() };
        let out = set.env.get_template(slug).unwrap().render(page_ctx(&page)).unwrap();
        assert!(out.contains("Yona") && out.contains("Doni"));
        assert!(unslash(out.clone()).contains("src=\"/img/"), "{slug}: URL aset polos agar ditulis ulang ke RustFS");
        assert!(out.contains("&lt;b&gt;Budi") && !out.contains("<b>Budi"), "nama tamu wajib di-escape");
        assert!(out.contains("Sabtu") && out.contains("September 2026"), "{slug}: tanggal acara");
        assert!(out.contains(flash_msg(false, Lang::Id)), "pesan ?rsvp=galat tampil tanpa JS");
        // Templat bawaan wajib punya jangkar yang dipakai navigasi bawah.
        for id in ["sampul", "acara", "ucapan"] {
            assert!(out.contains(&format!("id=\"{id}\"")), "jangkar #{id} untuk navigasi bawah");
        }
        if slug == "warkah" {
            // 12 Sep 2026 = Sabtu; 1 Sep = Selasa → 1 sel kosong, hati di tanggal 12.
            assert!(out.contains("Hari pernikahan, tanggal 12") && out.contains("data-burst"), "kalender & segel warkah");
        }
        let nav = bottom_nav(&page, "kusuma-jawi");
        assert!(nav.contains("/u/yona-doni/story") && nav.contains("#ucapan") && !nav.contains("Kelola"), "{nav}");
    }
}
