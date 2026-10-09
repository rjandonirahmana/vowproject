//! web/model.rs — tipe data bersama server ↔ WASM (dikirim lewat server fn).

use serde::{Deserialize, Serialize};

use super::fmt;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub label: String,
    pub time: String,
}

/// Satu acara (akad / resepsi / pemberkatan …). Disimpan sebagai JSONB.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Event {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub badge: String,
    #[serde(default)]
    pub tag: String,
    /// `YYYY-MM-DD`
    #[serde(default)]
    pub date: String,
    /// `HH:MM` (WIB)
    #[serde(default)]
    pub time_start: String,
    #[serde(default)]
    pub time_end: String,
    #[serde(default)]
    pub sessions: Vec<Session>,
    #[serde(default)]
    pub venue: String,
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub maps_url: String,
    /// WIB | WITA | WIT (kosong = WIB, data lama).
    #[serde(default)]
    pub tz: String,
}

impl Event {
    pub fn date_label(&self) -> String {
        fmt::tanggal_panjang(&self.date)
    }
    pub fn time_label(&self) -> String {
        fmt::jam_rentang(&self.time_start, &self.time_end, &self.tz)
    }
    /// Link peta: yang diisi pengantin, atau pencarian nama tempat.
    pub fn maps_link(&self) -> String {
        if self.maps_url.trim().is_empty() {
            format!(
                "https://www.google.com/maps/search/?api=1&query={}",
                fmt::url_encode(&format!("{} {}", self.venue, self.address))
            )
        } else {
            self.maps_url.clone()
        }
    }
    pub fn maps_embed(&self) -> String {
        format!(
            "https://maps.google.com/maps?q={}&z=15&output=embed",
            fmt::url_encode(&format!("{} {}", self.venue, self.address))
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DressColor {
    pub name: String,
    pub hex: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Bank {
    pub bank: String,
    pub number: String,
    pub holder: String,
}

/// Satu babak kisah cinta (timeline).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LoveStory {
    pub year: String,
    pub title: String,
    pub text: String,
    /// Foto babak ini (opsional; kosong → diambil bergiliran dari galeri).
    pub img: String,
}

/// Pengaturan tampilan dari tema (server mengisinya dari katalog tema).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InvSkin {
    /// Satu halaman panjang (bukan tab Sampul/Acara/RSVP).
    pub single: bool,
    /// none | tirai | pudar — hanya di mode satu halaman.
    pub open_anim: String,
    /// none | kelopak | kupu | bintang
    pub float_deco: String,
    /// Kunci gerak scroll — akar undangan diberi kelas `rvs--{kunci}` (koreografi).
    pub scroll_anim: String,
    /// Lapisan ornamen per bagian (web/ornamen.rs).
    pub ornaments: Vec<super::ornamen::Ornament>,
    /// Video latar bawaan tema (kosong = bukan tema sinema).
    pub bg_video: String,
    /// Video pembuka di gerbang (kosong = animasi CSS saja).
    pub open_video: String,
    /// Kelas varian struktur (web/rupa.rs), diawali spasi: " rp-s-kubah …".
    pub rupa: String,
    /// `/gaya/{tema}.css?v=…` — CSS tema INI saja (variabel, animasi, rupa).
    pub css: String,
    /// Google Fonts untuk huruf tema ini saja (kosong = huruf dasar).
    pub fonts: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Invitation {
    pub slug: String,
    pub theme: String,
    pub package: String,
    pub status: String,
    pub is_demo: bool,
    pub bride_name: String,
    pub bride_degree: String,
    pub bride_nick: String,
    pub bride_parents: String,
    pub bride_ig: String,
    pub bride_photo: String,
    pub groom_name: String,
    pub groom_degree: String,
    pub groom_nick: String,
    pub groom_parents: String,
    pub groom_ig: String,
    pub groom_photo: String,
    pub events: Vec<Event>,
    pub dress_code: String,
    pub dress_colors: Vec<DressColor>,
    pub quote_text: String,
    pub quote_source: String,
    pub music_title: String,
    pub music_artist: String,
    pub music_url: String,
    pub music_autoplay: bool,
    pub music_loop: bool,
    pub banks: Vec<Bank>,
    pub gift_address: String,
    pub gallery: Vec<String>,
    pub family_name: String,
    /// Foto berdua untuk sampul (kosong = monogram).
    pub cover_photo: String,
    pub love_story: Vec<LoveStory>,
    /// Tautan siaran langsung (YouTube/Instagram/Zoom), https saja.
    pub live_url: String,
    /// Video prewedding (unggahan RustFS / tautan .mp4/.webm). Tema sinema
    /// memutarnya sebagai latar; semua tema menampilkan bagian "Video Prewedding".
    pub video_url: String,
}

impl Invitation {
    pub fn bride_first(&self) -> String {
        first_word(&self.bride_name)
    }
    pub fn groom_first(&self) -> String {
        first_word(&self.groom_name)
    }
    /// "Yona & Doni"
    pub fn couple(&self) -> String {
        format!("{} & {}", self.bride_first(), self.groom_first())
    }
    /// Inisial monogram, mis. "A&R".
    pub fn initials(&self) -> String {
        format!("{}&{}", initial(&self.bride_name), initial(&self.groom_name))
    }
    pub fn first_event(&self) -> Option<&Event> {
        self.events.first()
    }
    pub fn date_label(&self) -> String {
        self.first_event().map(|e| e.date_label()).unwrap_or_default()
    }
    /// Epoch ms mulai acara pertama (WIB) — target hitung mundur.
    pub fn countdown_target_ms(&self) -> i64 {
        self.first_event()
            .and_then(|e| fmt::epoch_ms(&e.date, &e.time_start, &e.tz))
            .unwrap_or(0)
    }
    pub fn calendar_link(&self) -> String {
        match self.first_event() {
            Some(e) => fmt::google_calendar_url(
                &format!("Pernikahan {}", self.couple()),
                e,
                &format!("Undangan pernikahan {} — {}", self.couple(), e.venue),
            ),
            None => String::new(),
        }
    }
    pub fn music_label(&self) -> String {
        match (self.music_title.is_empty(), self.music_artist.is_empty()) {
            (true, _) => String::new(),
            (false, true) => self.music_title.clone(),
            (false, false) => format!("{} – {}", self.music_title, self.music_artist),
        }
    }
}

fn first_word(s: &str) -> String {
    s.split_whitespace().next().unwrap_or("").to_string()
}

pub fn initial(s: &str) -> String {
    s.trim().chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default()
}

/// Inisial dua huruf untuk avatar ("Siti Sarah" → "SS").
pub fn avatar_initials(s: &str) -> String {
    let mut out = String::new();
    for w in s.split_whitespace().filter(|w| w.chars().next().is_some_and(|c| c.is_alphabetic())) {
        out.push_str(&initial(w));
        if out.chars().count() >= 2 {
            break;
        }
    }
    if out.is_empty() {
        "?".into()
    } else {
        out
    }
}

/// Tamu terdaftar yang membuka link pribadinya (`?g=KODE`).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GuestInfo {
    pub code: String,
    pub name: String,
    pub category: String,
    pub session: String,
    pub table_no: String,
    pub pax: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct InvitationPage {
    pub inv: Invitation,
    pub guest: Option<GuestInfo>,
    /// Pratinjau pemilik (belum dibayar, dibuka dengan kunci Kelola `?k=`).
    #[serde(default)]
    pub preview: bool,
    #[serde(default)]
    pub skin: InvSkin,
}

impl Invitation {
    /// Belum dibayar → terkunci untuk tamu (demo tidak pernah terkunci).
    pub fn is_locked(&self) -> bool {
        self.status == "menunggu_pembayaran" && !self.is_demo
    }
}

/// "5 jam 12 menit" / "12 menit".
pub fn durasi(minutes: i64) -> String {
    let (h, m) = (minutes / 60, minutes % 60);
    match (h, m) {
        (0, m) => format!("{m} menit"),
        (h, 0) => format!("{h} jam"),
        (h, m) => format!("{h} jam {m} menit"),
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Wish {
    pub name: String,
    pub status: String,
    pub message: String,
    pub ago: String,
}

/// Lagu di pustaka musik admin (migrasi 027) — satu-satunya sumber musik latar.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Song {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub duration: String,
    pub tag: String,
    pub url: String,
    pub aktif: bool,
    pub urutan: i32,
}

impl Song {
    /// "Tulus • 03:42" / "Tulus".
    pub fn meta(&self) -> String {
        match (self.artist.is_empty(), self.duration.is_empty()) {
            (false, false) => format!("{} • {}", self.artist, self.duration),
            (false, true) => self.artist.clone(),
            (true, false) => self.duration.clone(),
            (true, true) => String::new(),
        }
    }
}

/// Story foto seorang tamu (satu per nomor HP per undangan).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StoryItem {
    pub id: i64,
    pub name: String,
    pub photo: String,
    /// Kunci `STORY_FILTERS` ("normal" = tanpa filter).
    pub filter: String,
    pub caption: String,
    pub ago: String,
    /// Story milik perangkat ini (cookie pembuat cocok) → boleh dihapus sendiri.
    #[serde(default)]
    pub mine: bool,
}

/// Hasil permintaan kunci story: nomor 62… + apakah nomor ini SUDAH punya
/// story (kunci lalu dipakai untuk menghapusnya, bukan membuat).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StoryKey {
    pub phone: String,
    pub has_story: bool,
}

/// Baris moderasi story (Kelola / admin): + nomor tersamar & undangannya.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StoryMod {
    pub id: i64,
    pub name: String,
    pub photo: String,
    pub filter: String,
    pub caption: String,
    pub ago: String,
    /// "0812•••7890" — cukup untuk mengenali pengirim, tak membocorkan nomor.
    pub phone: String,
    pub slug: String,
    pub couple: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StoryModPage {
    pub items: Vec<StoryMod>,
    pub total: i64,
    /// Halaman (mulai 1) & jumlah halaman.
    pub page: i64,
    pub pages: i64,
}

/// Jumlah story per halaman moderasi.
pub const STORY_PER_PAGE: i64 = 12;

/// "6281234567890" → "0812•••7890".
pub fn mask_phone(p: &str) -> String {
    let local = match p.strip_prefix("62") { Some(r) => format!("0{r}"), None => p.to_string() };
    let n = local.chars().count();
    if n < 8 {
        return local;
    }
    let head: String = local.chars().take(4).collect();
    let tail: String = local.chars().skip(n - 4).collect();
    format!("{head}•••{tail}")
}

/// Filter foto story — sama dengan pembuat story e-ticketing (CSS `.sf-{kunci}`).
pub const STORY_FILTERS: &[(&str, &str)] = &[
    ("normal", "Normal"),
    ("clarendon", "Cerah"),
    ("gingham", "Hangat"),
    ("moon", "Monokrom"),
    ("lark", "Lark"),
    ("reyes", "Reyes"),
    ("juno", "Juno"),
    ("slumber", "Slumber"),
    ("crema", "Crema"),
    ("ludwig", "Ludwig"),
];

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WishPage {
    pub total: i64,
    pub items: Vec<Wish>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GuestRow {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub category: String,
    pub session: String,
    pub table_no: String,
    pub pax: i32,
    /// hadir | ragu | tidak | "" (belum respon)
    pub rsvp: String,
    pub rsvp_pax: i32,
    pub opened: bool,
    pub sent_ago: String,
    pub checked_in: bool,
    pub gift_amount: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Stats {
    pub total_guests: i64,
    pub total_pax: i64,
    pub sent: i64,
    pub rsvp_count: i64,
    pub hadir_pax: i64,
    pub hadir_count: i64,
    pub ragu_count: i64,
    pub tidak_count: i64,
    pub gift_total: i64,
    pub gift_count: i64,
    pub checked_in: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Activity {
    pub text: String,
    pub ago: String,
}

/// Data halaman /kelola/{slug}/sunting — isi undangan saat ini + pilihan.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Sunting {
    pub inv: Invitation,
    pub contact_phone: String,
    /// Kunci pemilik (kosong = dibuka admin).
    pub manage_key: String,
    /// Pustaka lagu aktif.
    pub songs: Vec<Song>,
    /// id lagu yang sedang dipakai (0 = tak cocok dengan pustaka / tanpa musik).
    pub song_id: i64,
    /// (slug, nama, wilayah) tema tayang + tema undangan ini bila privat.
    pub themes: Vec<(String, String, String)>,
    /// Indeks themes::QUOTES yang cocok dengan kutipan sekarang.
    pub quote_idx: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Dashboard {
    pub inv: Invitation,
    /// Nama paket (dari konten admin) untuk pesan pembayaran.
    pub package_name: String,
    /// Menit sebelum pesanan belum-dibayar dihapus otomatis (None = aktif).
    pub minutes_left: Option<i64>,
    pub total_price: i64,
    pub payment_method: String,
    /// WA admin (62…) untuk konfirmasi pembayaran; kosong = tak diset.
    pub admin_wa: String,
    pub stats: Stats,
    pub guests: Vec<GuestRow>,
    pub activity: Vec<Activity>,
    /// Hiasan melayang tema undangan (dashboard ikut bernuansa tema).
    #[serde(default)]
    pub float_deco: String,
    /// Rekening tujuan transfer (Konten `pembayaran`).
    #[serde(default)]
    pub payment: crate::web::konten::Pembayaran,
    /// Bukti transfer yang sudah dikirim: (URL gambar — bisa '', "x lalu").
    #[serde(default)]
    pub payment_proof: Option<(String, String)>,
    /// Kunci Kelola yang sudah terverifikasi (dari cookie) — URL tak lagi
    /// memuatnya, jadi dashboard menampilkannya agar tautan khusus bisa disalin.
    #[serde(default)]
    pub manage_key: String,
}

pub fn category_label(c: &str) -> &'static str {
    match c {
        "vip" => "VIP",
        "keluarga" => "VIP Keluarga",
        "sahabat" => "Sahabat",
        "kantor" => "Rekan Kantor",
        _ => "Umum",
    }
}

pub const CATEGORIES: &[(&str, &str)] = &[
    ("umum", "Umum"),
    ("vip", "VIP"),
    ("keluarga", "Keluarga"),
    ("sahabat", "Sahabat"),
    ("kantor", "Rekan Kantor"),
];

pub fn rsvp_label(s: &str) -> &'static str {
    match s {
        "hadir" => "Hadir",
        "ragu" => "Masih Ragu",
        "tidak" => "Berhalangan",
        _ => "Belum Respon",
    }
}

// ── Admin ──────────────────────────────────────────────────────────────────

/// Baris daftar pesanan di /admin/undangan.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AdminInv {
    pub slug: String,
    pub package_name: String,
    pub couple: String,
    pub theme: String,
    pub package: String,
    pub status: String,
    pub total_price: i64,
    pub payment_method: String,
    pub contact_phone: String,
    pub is_demo: bool,
    /// "2026-09-30 14:05"
    pub created: String,
    /// Menit sebelum dihapus otomatis (hanya pesanan belum dibayar & belum kirim bukti).
    pub minutes_left: Option<i64>,
    /// URL gambar bukti transfer ('' = belum ada / tanpa RustFS).
    #[serde(default)]
    pub payment_proof: String,
    /// Kapan bukti dikirim ("2026-10-03 14:05"); '' = belum.
    #[serde(default)]
    pub proof_at: String,
}

/// Tema + jumlah undangan yang memakainya (untuk /admin/tema).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AdminTheme {
    pub theme: super::skin::ThemeInfo,
    pub used: i64,
    /// Jumlah ornamen tema (rinciannya lewat `get_theme`).
    #[serde(default)]
    pub ornaments: usize,
}

pub const INV_STATUSES: &[(&str, &str)] = &[
    ("menunggu_pembayaran", "Menunggu pembayaran"),
    ("aktif", "Aktif"),
    ("nonaktif", "Nonaktif"),
];

/// Akun panel admin. role: `admin` (semua) | `editor` (tema & konten).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AdminUser {
    pub id: i64,
    pub username: String,
    pub name: String,
    pub role: String,
    pub active: bool,
    pub last_login: String,
}

impl AdminUser {
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }
    pub fn display(&self) -> String {
        if self.name.is_empty() { self.username.clone() } else { self.name.clone() }
    }
}

pub const ADMIN_ROLES: &[(&str, &str, &str)] = &[
    ("admin", "Admin", "Semua akses: tema, konten & harga, pesanan, dan akun."),
    ("editor", "Editor", "Tema serta konten & harga layanan. Tanpa pesanan & akun."),
];

/// Status sesi untuk halaman /admin.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AdminSessionInfo {
    /// Tabel akun sudah ada (migrasi 003 dijalankan).
    pub db_ready: bool,
    /// Belum ada akun sama sekali → tampilkan form "akun pertama".
    pub needs_setup: bool,
    /// ADMIN_TOKEN terpasang (dibutuhkan sebagai kode setup).
    pub setup_code_set: bool,
    pub user: Option<AdminUser>,
}

/// Animasi kustom + nama tema yang memakainya (panel admin).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AdminAnim {
    pub anim: crate::web::anim::AnimInfo,
    pub used_by: Vec<String>,
}

/// Tema templat untuk /admin/templat (server/templat.rs, migrasi 029).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AdminTemplat {
    pub slug: String,
    pub name: String,
    pub html: String,
    pub css: String,
    pub fonts: String,
    /// Aset bawaan sebagai JSON rapi (kunci → URL) untuk textarea.
    pub assets: String,
    pub builtin: bool,
    pub edited: bool,
    /// (slug, nama) tema yang memakai templat ini.
    pub used_by: Vec<(String, String)>,
}

/// Tema + pengaturan templatnya (/admin/templat).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TemaTemplat {
    pub slug: String,
    pub name: String,
    pub template: String,
    pub assets: String,
    pub css: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AdminTemplatPage {
    pub templates: Vec<AdminTemplat>,
    pub themes: Vec<TemaTemplat>,
}

/// Banner strip di atas beranda (tabel `banners`, dikelola di /admin/banner).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Banner {
    pub id: i64,
    pub judul: String,
    pub sub: String,
    /// Teks tombol (kosong = tanpa tombol).
    pub cta: String,
    pub link: String,
    /// Desktop: strip 9:1 (2880×320).
    pub img: String,
    /// HP (opsional): 8:3 (1080×405). Kosong = gambar desktop.
    pub img_hp: String,
    pub aktif: bool,
    pub urutan: i32,
    /// Jadwal tayang (WIB, format input `YYYY-MM-DDTHH:MM`); kosong = tanpa batas.
    pub mulai: String,
    pub selesai: String,
    /// Admin: tayang | terjadwal | berakhir | nonaktif.
    pub status: String,
}

#[cfg(test)]
mod story_tests {
    use super::STORY_FILTERS;

    /// Tiap filter story punya aturan CSS `.sf-{kunci}` (thumbnail, kisi,
    /// penampil memakai kelas yang sama) — filter baru tanpa CSS = foto polos.
    #[test]
    fn nomor_pengirim_disamarkan() {
        assert_eq!(super::mask_phone("6281234567890"), "0812•••7890");
        assert_eq!(super::mask_phone("620000000001"), "0000•••0001");
        assert_eq!(super::mask_phone("123"), "123");
    }

    #[test]
    fn semua_filter_story_punya_css() {
        let css = include_str!("../../style/main.css");
        for (k, _) in STORY_FILTERS {
            assert!(css.contains(&format!(".sf-{k} {{")), "CSS .sf-{k} belum ada");
        }
    }
}
