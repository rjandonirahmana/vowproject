//! web/skin.rs — tema undangan sebagai DATA (tabel `themes`), bukan kode.
//!
//! Satu tema = "kulit" di atas mesin undangan yang sama: token warna CSS,
//! huruf judul, tata letak sampul, dan ornamen (bawaan atau gambar unggahan).
//! Admin merakit tema baru di /admin/tema dari bahan-bahan di bawah; server
//! membangkitkan `/tema.css` berisi `.th-{slug} { --bg: …; … }` sehingga semua
//! elemen yang sudah memakai kelas `th-{slug}` otomatis ikut.
//!
//! Menambah BAHAN baru (tata letak, ornamen, font) baru perlu kode; menambah
//! TEMA tidak. Fungsi di sini murni (tanpa IO) — dipakai server untuk CSS dan
//! WASM untuk pratinjau langsung di form admin.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::fmt;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeInfo {
    pub slug: String,
    pub name: String,
    /// Label bebas, mis. "Adat" — kunci filter = `fmt::key(label)`.
    pub category: String,
    /// Label bebas, mis. "Jawa".
    pub nuansa: String,
    /// Kunci dari `PALETTES`.
    pub palette: String,
    /// Label kecil di pojok gambar kartu.
    pub region: String,
    pub description: String,
    pub tags: Vec<String>,
    pub badge: String,
    pub rating: String,
    pub reviews: String,
    pub layout: String,
    pub ornament: String,
    pub font: String,
    /// Nama variabel CSS tanpa `--` → warna hex. Kunci absen = diturunkan.
    pub tokens: BTreeMap<String, String>,
    pub dark: bool,
    /// Gambar ornamen/latar unggahan admin (menimpa `ornament`).
    pub image_url: String,
    /// `sudut` | `pola` | `penuh`
    pub image_mode: String,
    /// false = tema privat (pesanan custom): tak tampil di katalog, tetap
    /// bisa dipakai lewat tautan /tema/{slug} dan /buat?tema={slug}.
    pub listed: bool,
    pub sort_order: i32,
    /// Detik epoch pembuatan (urutan "Terbaru").
    pub created: i64,
    /// Kunci `SCRIPT_FONTS` (kosong = judul memakai huruf judul biasa).
    pub script_font: String,
    /// Ilustrasi latar penuh (tetap di belakang saat di-scroll).
    pub bg_image: String,
    /// Video latar bawaan tema (mp4/webm). Tak kosong = tema SINEMA: video
    /// diputar penuh di belakang isi; video prewedding pasangan menggantikannya.
    pub bg_video: String,
    /// Video pembuka (mp4) yang diputar sekali di gerbang saat "Buka Undangan"
    /// (animasi buka `video-pintu`); kosong = gerbang memakai animasi CSS saja.
    pub open_video: String,
    /// Bingkai PNG/SVG transparan di atas foto sampul & foto mempelai.
    pub frame_image: String,
    /// Hiasan bunga di tepi atas kartu.
    pub card_deco: String,
    /// Kunci animasi hiasan melayang (tabel animations, jenis "hiasan") / "none".
    pub float_deco: String,
    /// Kunci animasi cara membuka (jenis "buka") / "none".
    pub open_anim: String,
    /// `PAGE_MODES`: tab | satu
    pub page_mode: String,
    /// Kunci gerak isi saat muncul di layar (jenis "scroll").
    pub scroll_anim: String,
    /// Gerak khusus judul / foto saat muncul (`ornamen::GERAK_ELEMEN`; "ikut" = sama dengan scroll).
    pub gerak_judul: String,
    pub gerak_foto: String,
    /// Foto sampul perlahan membesar-bergeser (efek Ken Burns).
    pub ken_burns: bool,
    /// Lapisan ornamen per bagian (tabel theme_ornaments; dimuat terpisah).
    pub ornaments: Vec<super::ornamen::Ornament>,
}

pub const DEFAULT_THEME: &str = "botanical-heritage";

impl ThemeInfo {
    /// Tema cadangan bila tabel `themes` belum dimigrasi — tampilan bawaan
    /// `:root` di main.css.
    pub fn fallback() -> Self {
        ThemeInfo {
            slug: DEFAULT_THEME.into(),
            name: "Botanical Heritage Romance".into(),
            category: "Botanical".into(),
            nuansa: "Islami".into(),
            palette: "sage".into(),
            region: "Botanical Heirs".into(),
            description: "Emblem monogram inisial melingkar berhias daun zaitun sage green dengan cap lak emas.".into(),
            layout: "klasik".into(),
            ornament: "daun".into(),
            font: "playfair".into(),
            listed: true,
            ..Default::default()
        }
    }

    pub fn category_key(&self) -> String {
        fmt::key(&self.category)
    }
    pub fn nuansa_key(&self) -> String {
        fmt::key(&self.nuansa)
    }
    fn token(&self, k: &str) -> Option<&str> {
        self.tokens.get(k).map(String::as_str).filter(|v| is_color(v))
    }
    /// Warna untuk swatch/ornamen: token tema atau bawaan `:root`.
    pub fn color(&self, k: &str) -> String {
        self.token(k)
            .map(str::to_string)
            .unwrap_or_else(|| TOKENS.iter().find(|t| t.key == k).map(|t| t.default.to_string()).unwrap_or_default())
    }
}

// ── Bahan: token warna ─────────────────────────────────────────────────────

pub struct Token {
    pub key: &'static str,
    pub label: &'static str,
    /// Nilai `:root` di main.css (tema Botanical Heritage).
    pub default: &'static str,
    /// Tampil di bagian utama form admin (sisanya di "Lanjutan").
    pub main: bool,
    /// Rumus turunan bila admin tidak mengisinya.
    pub derive: &'static str,
}

pub const TOKENS: &[Token] = &[
    Token { key: "bg", label: "Latar halaman", default: "#f4fcf0", main: true, derive: "" },
    Token { key: "card", label: "Kartu", default: "#ffffff", main: true, derive: "" },
    Token { key: "primary", label: "Warna utama (judul & tombol)", default: "#273f2b", main: true, derive: "" },
    Token { key: "on-primary", label: "Teks di atas warna utama", default: "#ffffff", main: true, derive: "" },
    Token { key: "gold", label: "Aksen emas", default: "#c5a059", main: true, derive: "" },
    Token { key: "gold-deep", label: "Aksen gelap (label & ikon)", default: "#775a19", main: true, derive: "" },
    Token { key: "ink", label: "Teks utama", default: "#161d17", main: true, derive: "" },
    Token { key: "surface-low", label: "Permukaan lembut", default: "#eef6eb", main: false, derive: "color-mix(in srgb, var(--bg) 96%, var(--ink))" },
    Token { key: "surface", label: "Permukaan", default: "#e9f0e5", main: false, derive: "color-mix(in srgb, var(--bg) 93%, var(--ink))" },
    Token { key: "surface-high", label: "Permukaan tegas", default: "#e3eadf", main: false, derive: "color-mix(in srgb, var(--bg) 89%, var(--ink))" },
    Token { key: "primary-2", label: "Warna utama sekunder", default: "#3e5641", main: false, derive: "color-mix(in srgb, var(--primary) 82%, var(--bg))" },
    Token { key: "sage", label: "Aksen daun", default: "#5b7553", main: false, derive: "color-mix(in srgb, var(--primary) 70%, var(--bg))" },
    Token { key: "sage-mist", label: "Aksen daun pudar", default: "#879878", main: false, derive: "color-mix(in srgb, var(--primary) 48%, var(--bg))" },
    Token { key: "gold-light", label: "Emas terang", default: "#dfbe72", main: false, derive: "color-mix(in srgb, var(--gold) 78%, #fff)" },
    Token { key: "gold-c", label: "Emas pastel (chip)", default: "#fed488", main: false, derive: "color-mix(in srgb, var(--gold) 50%, #fff)" },
    Token { key: "gold-pale", label: "Emas pucat", default: "#ffdea5", main: false, derive: "color-mix(in srgb, var(--gold) 36%, #fff)" },
    Token { key: "ink-2", label: "Teks sekunder", default: "#434842", main: false, derive: "color-mix(in srgb, var(--ink) 78%, var(--bg))" },
    Token { key: "muted", label: "Teks redup", default: "#5c665e", main: false, derive: "color-mix(in srgb, var(--ink) 58%, var(--bg))" },
    Token { key: "line", label: "Garis", default: "#dde5da", main: false, derive: "color-mix(in srgb, var(--bg) 88%, var(--ink))" },
];

/// `#abc`, `#aabbcc`, `#aabbccdd` saja — nilai lain dibuang (cegah injeksi CSS).
pub fn is_color(v: &str) -> bool {
    let h = v.strip_prefix('#').unwrap_or("");
    matches!(h.len(), 3 | 4 | 6 | 8) && h.chars().all(|c| c.is_ascii_hexdigit())
}

/// URL gambar aman untuk `url("…")`: http(s) atau path lokal, tanpa kutip,
/// kurung, spasi, atau backslash.
pub fn is_safe_url(u: &str) -> bool {
    (u.starts_with("https://") || u.starts_with("http://") || u.starts_with('/'))
        && !u.starts_with("//")
        && u.len() <= 500
        && !u.chars().any(|c| c.is_whitespace() || matches!(c, '"' | '\'' | '(' | ')' | '\\' | '<' | '>' | ';' | '{' | '}'))
}

// ── Bahan: tata letak sampul ───────────────────────────────────────────────

/// (kunci, label, keterangan, variabel CSS yang dipakai `.hero` & `.mini`)
pub const LAYOUTS: &[(&str, &str, &str, &str)] = &[
    ("klasik", "Klasik Tengah", "Segel monogram di tengah, nama bertumpuk.", ""),
    (
        "gerbang",
        "Gerbang Lengkung",
        "Sampul di dalam kartu berbentuk gapura.",
        "--hero-bg:var(--card);--hero-radius:220px 220px 28px 28px;--hero-pad:44px 22px 30px;--hero-shadow:var(--shadow);--mini-radius:90px 90px 14px 14px;",
    ),
    (
        "bingkai",
        "Bingkai Ganda",
        "Bingkai emas tipis berlapis, kesan formal.",
        "--hero-bg:var(--card);--hero-border:1px solid var(--gold);--hero-outline:1px solid var(--line-gold);--hero-radius:14px;--hero-pad:30px 20px;--mini-radius:10px;",
    ),
    (
        "editorial",
        "Editorial Rata Kiri",
        "Tanpa segel, nama besar rata kiri ala majalah.",
        "--hero-align:flex-start;--hero-text:left;--seal-display:none;--hero-name-size:44px;--hero-pad:28px 2px 8px;--mini-radius:4px;",
    ),
];

// ── Bahan: ornamen ─────────────────────────────────────────────────────────

pub const ORNAMENTS: &[(&str, &str)] = &[
    ("none", "Polos"),
    ("daun", "Ranting daun (sudut)"),
    ("melati", "Taburan melati"),
    ("kawung", "Motif kawung"),
    ("bintang", "Langit berbintang"),
];

pub const IMAGE_MODES: &[(&str, &str)] = &[
    ("sudut", "Hiasan sudut"),
    ("pola", "Pola berulang"),
    ("penuh", "Latar penuh (samar)"),
];

fn svg_url(svg: &str) -> String {
    // Data URI: cukup escape karakter yang bermasalah di dalam url("…").
    let enc = svg.replace('%', "%25").replace('#', "%23").replace('"', "'").replace('<', "%3C").replace('>', "%3E");
    format!("url(\"data:image/svg+xml,{enc}\")")
}

fn sprig(stroke: &str, leaf: &str, flip: bool) -> String {
    let t = if flip { " transform='rotate(180 80 80)'" } else { "" };
    svg_url(&format!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 160 160'><g{t} fill='none' stroke='{stroke}' stroke-width='1.4'>\
<path d='M8 150 C40 110 70 80 150 12'/>\
<g fill='{leaf}' stroke='none'>\
<ellipse cx='40' cy='112' rx='13' ry='5' transform='rotate(-55 40 112)'/><ellipse cx='52' cy='122' rx='13' ry='5' transform='rotate(15 52 122)'/>\
<ellipse cx='70' cy='82' rx='14' ry='5.5' transform='rotate(-60 70 82)'/><ellipse cx='84' cy='92' rx='14' ry='5.5' transform='rotate(10 84 92)'/>\
<ellipse cx='100' cy='54' rx='13' ry='5' transform='rotate(-62 100 54)'/><ellipse cx='114' cy='62' rx='13' ry='5' transform='rotate(8 114 62)'/>\
<ellipse cx='132' cy='28' rx='11' ry='4.5' transform='rotate(-35 132 28)'/></g></g></svg>"
    ))
}

/// Variabel `--orn*` untuk `.inv__glow::before` dan kartu katalog.
fn ornament_vars(t: &ThemeInfo) -> String {
    if is_safe_url(&t.image_url) {
        let (size, pos, rep, op) = match t.image_mode.as_str() {
            "pola" => ("200px auto", "0 0", "repeat", ".16"),
            "penuh" => ("cover", "center", "no-repeat", ".22"),
            _ => ("min(220px,45vw) auto", "right -12px top 56px", "no-repeat", ".9"),
        };
        return format!("--orn:url(\"{}\");--orn-size:{size};--orn-pos:{pos};--orn-repeat:{rep};--orn-opacity:{op};", t.image_url);
    }
    let gold = t.color("gold");
    let sage = t.color("sage");
    match t.ornament.as_str() {
        "daun" => format!(
            "--orn:{},{};--orn-size:150px auto,150px auto;--orn-pos:left -18px top 64px,right -18px bottom 96px;--orn-repeat:no-repeat;--orn-opacity:.55;",
            sprig(&gold, &sage, false),
            sprig(&gold, &sage, true)
        ),
        "melati" => format!(
            "--orn:{};--orn-size:120px 120px;--orn-pos:0 0;--orn-repeat:repeat;--orn-opacity:.28;",
            svg_url(&format!(
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 120 120'><g fill='#fff' stroke='{gold}' stroke-width='.8'>\
<g transform='translate(28 30)'><circle r='4'/><circle cx='0' cy='-7' r='4'/><circle cx='6.6' cy='-2.2' r='4'/><circle cx='4' cy='5.7' r='4'/><circle cx='-4' cy='5.7' r='4'/><circle cx='-6.6' cy='-2.2' r='4'/></g>\
<g transform='translate(88 84) scale(.7)'><circle r='4'/><circle cx='0' cy='-7' r='4'/><circle cx='6.6' cy='-2.2' r='4'/><circle cx='4' cy='5.7' r='4'/><circle cx='-4' cy='5.7' r='4'/><circle cx='-6.6' cy='-2.2' r='4'/></g>\
</g><circle cx='28' cy='30' r='2' fill='{gold}'/><circle cx='88' cy='84' r='1.5' fill='{gold}'/></svg>"
            ))
        ),
        "kawung" => format!(
            "--orn:{};--orn-size:56px 56px;--orn-pos:0 0;--orn-repeat:repeat;--orn-opacity:.1;",
            svg_url(&format!(
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 56 56'><g fill='none' stroke='{gold}' stroke-width='1.6'>\
<ellipse cx='28' cy='14' rx='7' ry='12'/><ellipse cx='28' cy='42' rx='7' ry='12'/><ellipse cx='14' cy='28' rx='12' ry='7'/><ellipse cx='42' cy='28' rx='12' ry='7'/>\
<circle cx='28' cy='28' r='2.5' fill='{gold}'/></g></svg>"
            ))
        ),
        "bintang" => format!(
            "--orn:radial-gradient(1px 1px at 20% 30%,#fff8 50%,transparent),radial-gradient(1px 1px at 70% 12%,#fff8 50%,transparent),\
radial-gradient(1.5px 1.5px at 85% 60%,{gold} 50%,transparent),radial-gradient(1px 1px at 40% 80%,#fff8 50%,transparent),\
radial-gradient(1.2px 1.2px at 10% 70%,{gold} 50%,transparent),radial-gradient(1px 1px at 55% 45%,#fff6 50%,transparent);\
--orn-size:100% 100%;--orn-pos:0 0;--orn-repeat:no-repeat;--orn-opacity:1;"
        ),
        _ => String::new(),
    }
}

// ── Bahan: huruf judul ─────────────────────────────────────────────────────

/// (kunci, label, font-family CSS, parameter Google Fonts; kosong = sudah dimuat)
pub const FONTS: &[(&str, &str, &str, &str)] = &[
    ("playfair", "Playfair Display — klasik elegan", "\"Playfair Display\"", ""),
    ("cormorant", "Cormorant Garamond — romantis tipis", "\"Cormorant Garamond\"", "Cormorant+Garamond:ital,wght@0,500;0,600;1,500"),
    ("cinzel", "Cinzel — kapital keraton", "\"Cinzel\"", "Cinzel:wght@500;600"),
    ("marcellus", "Marcellus — antik lembut", "\"Marcellus\"", "Marcellus"),
    ("dm-serif", "DM Serif Display — modern tegas", "\"DM Serif Display\"", "DM+Serif+Display:ital@0;1"),
    ("lora", "Lora — hangat & terbaca", "\"Lora\"", "Lora:ital,wght@0,500;0,600;1,500"),
];

/// Parameter `family=` tambahan untuk font yang dipakai tema-tema ini.
pub fn font_families<'a>(themes: impl IntoIterator<Item = &'a ThemeInfo>) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for t in themes {
        if let Some(f) = FONTS.iter().find(|f| f.0 == t.font && !f.3.is_empty()) {
            if !out.contains(&f.3) {
                out.push(f.3);
            }
        }
        if let Some(f) = SCRIPT_FONTS.iter().find(|f| f.0 == t.script_font && !f.3.is_empty()) {
            if !out.contains(&f.3) {
                out.push(f.3);
            }
        }
    }
    out
}

// ── Bahan: huruf kaligrafi, dekorasi, animasi, mode halaman ────────────────

/// (kunci, label, font-family, parameter Google Fonts). Semua berlisensi OFL.
pub const SCRIPT_FONTS: &[(&str, &str, &str, &str)] = &[
    ("", "Tanpa kaligrafi (pakai huruf judul)", "", ""),
    ("pinyon", "Pinyon Script — klasik mewah", "\"Pinyon Script\"", "Pinyon+Script"),
    ("great-vibes", "Great Vibes — mengalir romantis", "\"Great Vibes\"", "Great+Vibes"),
    ("parisienne", "Parisienne — anggun ringan", "\"Parisienne\"", "Parisienne"),
    ("allura", "Allura — tipis elegan", "\"Allura\"", "Allura"),
    ("alex-brush", "Alex Brush — goresan kuas", "\"Alex Brush\"", "Alex+Brush"),
];


pub const PAGE_MODES: &[(&str, &str)] = &[
    ("tab", "Tab: Sampul • Acara • RSVP"),
    ("satu", "Satu halaman panjang"),
];

/// (kunci, label, variabel CSS). Dibaca aturan `.rv-on [data-rv]` di main.css.
/// Paket gaya: sekali klik mengisi 3 lapis animasi (masih bisa diubah).
/// Kuncinya animasi bawaan (web/anim.rs `builtins`); paket yang animasinya
/// tak ada di katalog disembunyikan editor.
/// (kunci, nama, keterangan, pembuka, scroll, hiasan)
pub const MOTION_PRESETS: &[(&str, &str, &str, &str, &str, &str)] = &[
    ("elegan", "Elegan Lembut", "Tirai terbuka, isi naik perlahan, kelopak berguguran", "tirai", "naik", "kelopak"),
    ("keraton", "Megah Keraton", "Pintu gerbang 3D, isi membesar, kerlip bintang", "gerbang", "zoom", "bintang"),
    ("surat", "Surat Cinta", "Amplop bersegel, isi bergeser bergantian, kupu-kupu", "amplop", "geser", "kupu"),
    ("modern", "Modern Bersih", "Sampul memudar, isi pudar lembut, tanpa hiasan", "pudar", "pudar", "none"),
    ("malam", "Malam Gala", "Sampul memudar, isi dari samar ke jelas, bintang", "pudar", "blur", "bintang"),
    ("klasik", "Klasik Tanpa Pembuka", "Mode tab biasa, isi naik perlahan", "none", "naik", "none"),
    ("wayang", "Pagelaran Wayang", "Kelir menyala, gunungan dikebutkan, isi muncul dari bayangan, kunang-kunang", "pagelaran-wayang", "bayang", "kunang"),
    ("gebyok", "Pendopo Gebyok", "Sampul tergulir, pintu gebyok berayun & ditembus, koreografi keraton, kelopak gugur", "gebyok-ukir", "keraton", "kelopak"),
    ("taman", "Taman Botani", "Rimbun daun tersibak, isi mekar, daun gugur berputar", "taman-daun", "mekar", "daun-gugur"),
    ("galaksi", "Langit Galaksi", "Warp bintang & portal cahaya, isi dari kedalaman, kunang-kunang", "galaksi", "kosmik", "kunang"),
    ("pura", "Gerbang Pura", "Candi bentar terbelah, isi mengayun seperti ombak, kupu-kupu", "candi-bentar", "ombak", "kupu"),
    ("sinema", "Sinema Kenangan", "Sampul polaroid naik seperti layar bioskop, isi tenang di atas video latar", "layar-naik", "sinema", "none"),
    ("songket", "Tenun Songket", "Helai songket diurai, isi tersingkap seperti benang, kilau emas", "tenun-songket", "tenun", "kilau-emas"),
];

impl ThemeInfo {
    /// Satu halaman panjang: dipilih eksplisit, ATAU otomatis bila tema punya
    /// animasi pembuka (gerbang hanya ada di mode satu halaman).
    pub fn single_page(&self) -> bool {
        self.page_mode == "satu" || (!self.open_anim.is_empty() && self.open_anim != "none")
    }
}

// ── Palet (filter katalog) ─────────────────────────────────────────────────

/// (kunci, label, warna swatch)
pub const PALETTES: &[(&str, &str, &str)] = &[
    ("sage", "Sage", "#3e5641"),
    ("gold", "Gold", "#e9c176"),
    ("terra", "Terra", "#b0643f"),
    ("navy", "Navy", "#1f2a44"),
    ("blush", "Blush", "#f3cfc6"),
    ("mono", "Monokrom", "#2c2a27"),
];

// ── Keluaran ───────────────────────────────────────────────────────────────

/// Deklarasi variabel CSS satu tema (tanpa selektor). Dipakai `/tema.css`
/// dan atribut `style` pratinjau admin.
pub fn theme_vars(t: &ThemeInfo) -> String {
    let mut s = String::new();
    let custom = TOKENS.iter().any(|k| t.token(k.key).is_some());
    for k in TOKENS {
        match t.token(k.key) {
            Some(v) => s.push_str(&format!("--{}:{v};", k.key)),
            None if custom && !k.derive.is_empty() => s.push_str(&format!("--{}:{};", k.key, k.derive)),
            None => {}
        }
    }
    if custom {
        s.push_str(
            "--line-gold:color-mix(in srgb,var(--gold) 32%,transparent);\
--foil:linear-gradient(135deg,var(--gold-light) 0%,var(--gold-c) 50%,var(--gold-pale) 100%);\
--foil-deep:linear-gradient(135deg,var(--gold),var(--gold-light) 50%,var(--gold-deep));",
        );
    }
    if t.dark {
        s.push_str(
            "color-scheme:dark;--shadow:0 6px 24px -4px rgba(0,0,0,.45);--shadow-lg:0 14px 36px -6px rgba(0,0,0,.55);\
--glow-opacity:.25;--ok-bg:#2c3a26;--ok:#bfe0a8;--err-bg:#4a1f1c;--err:#ffb4ab;\
--side-bg:var(--surface-low);--side-ink:var(--ink);--side-accent:var(--gold);--side-btn:var(--gold);--side-btn-ink:var(--on-primary);",
        );
    }
    if let Some(f) = FONTS.iter().find(|f| f.0 == t.font && f.0 != "playfair") {
        s.push_str(&format!("--font-display:{},Georgia,serif;", f.2));
    }
    if let Some(l) = LAYOUTS.iter().find(|l| l.0 == t.layout) {
        s.push_str(l.3);
    }
    if let Some(f) = SCRIPT_FONTS.iter().find(|f| f.0 == t.script_font && !f.2.is_empty()) {
        s.push_str(&format!("--font-script:{},cursive;--script-scale:1.45;--script-weight:400;", f.2));
    }
    let img = |u: &str| is_safe_url(u).then(|| format!("url(\"{u}\")"));
    if let Some(u) = img(&t.bg_image) {
        // Latar ilustrasi → kartu agak tembus pandang agar ilustrasi terasa.
        s.push_str(&format!(
            "--bg-illus:{u};--card-glass:color-mix(in srgb,var(--card) 84%,transparent);--card-blur:blur(6px);"
        ));
    }
    if let Some(u) = img(&t.frame_image) {
        s.push_str(&format!("--frame-img:{u};"));
    }
    if let Some(u) = img(&t.card_deco) {
        s.push_str(&format!("--card-deco:{u};"));
    }
    s.push_str(&ornament_vars(t));
    s.push_str(&super::ornamen::element_vars(&t.gerak_judul, &t.gerak_foto, t.ken_burns));
    s
}

/// Variabel tema + gerak scroll (`--rv-*`) dari katalog animasi.
pub fn theme_vars_with(t: &ThemeInfo, anims: &[super::anim::AnimInfo]) -> String {
    let mut v = theme_vars(t);
    if let Some(a) = anims.iter().find(|a| a.kind == "scroll" && a.key() == t.scroll_anim) {
        v.push_str(&super::anim::scroll_vars_of(a));
    }
    v
}

/// Seluruh stylesheet tema — dilayani di `/tema.css`.
pub fn catalog_css(themes: &[ThemeInfo], anims: &[super::anim::AnimInfo]) -> String {
    let mut css = String::from("/* dibangkitkan dari tabel themes — jangan disunting */\n");
    for t in themes {
        if !is_slug(&t.slug) {
            continue;
        }
        css.push_str(&format!(".th-{}{{{}}}\n", t.slug, theme_vars_with(t, anims)));
    }
    css
}

/// Kunci animasi berbentuk slug; keberadaannya dicek server terhadap katalog.
fn anim_key(v: &str, d: &str) -> String {
    let v = v.trim();
    if fmt::is_slug(v, super::anim::KEY_MAX) { v.to_string() } else { d.to_string() }
}

/// Panjang maksimum kode tema.
pub const SLUG_MAX: usize = 48;

pub fn is_slug(s: &str) -> bool {
    fmt::is_slug(s, SLUG_MAX)
}

/// Susun tema dari isian form admin (`name → value`). Semua nilai disaring;
/// galat dikembalikan sebagai kalimat untuk ditampilkan ke admin.
pub fn from_form(get: impl Fn(&str) -> String) -> Result<ThemeInfo, String> {
    let f = fmt::Fields(get);
    let slug = fmt::key(&f.text("slug", 60));
    if !is_slug(&slug) || slug.len() < 3 || slug == "baru" {
        return Err("Kode tema (slug) minimal 3 karakter: huruf kecil, angka, tanda minus.".into());
    }
    let name = f.text("name", 80);
    if name.is_empty() {
        return Err("Nama tema wajib diisi.".into());
    }
    let advanced = f.flag("adv");
    let mut tokens = BTreeMap::new();
    for t in TOKENS.iter().filter(|t| t.main || advanced) {
        let v = f.text(&format!("tok_{}", t.key), 9).to_lowercase();
        if is_color(&v) {
            tokens.insert(t.key.to_string(), v);
        }
    }
    // Gambar opsional: kosong boleh, isi wajib URL aman.
    let img = |k: &str| -> Result<String, String> {
        let u = f.text(k, 500);
        if u.is_empty() || is_safe_url(&u) {
            Ok(u)
        } else {
            Err(format!("URL gambar ({k}) tidak valid — harus https://… atau /img/…, tanpa spasi/kutip."))
        }
    };
    let gerak = || super::ornamen::GERAK_ELEMEN.iter().map(|g| g.0);
    Ok(ThemeInfo {
        slug,
        name,
        category: f.text("category", 40),
        nuansa: f.text("nuansa", 40),
        palette: f.pick("palette", PALETTES.iter().map(|p| p.0), "sage"),
        region: f.text("region", 40),
        description: f.text("description", 240),
        tags: f.text("tags", 200).split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).take(5).collect(),
        badge: f.text("badge", 30),
        rating: f.text("rating", 6),
        reviews: f.text("reviews", 10),
        layout: f.pick("layout", LAYOUTS.iter().map(|l| l.0), "klasik"),
        ornament: f.pick("ornament", ORNAMENTS.iter().map(|o| o.0), "none"),
        font: f.pick("font", FONTS.iter().map(|x| x.0), "playfair"),
        tokens,
        dark: f.flag("dark"),
        image_url: img("image_url")?,
        image_mode: f.pick("image_mode", IMAGE_MODES.iter().map(|m| m.0), "sudut"),
        listed: f.flag("listed"),
        sort_order: f.int("sort_order", 0, 9999, 100),
        created: 0,
        script_font: f.pick("script_font", SCRIPT_FONTS.iter().map(|x| x.0), ""),
        bg_image: img("bg_image")?,
        bg_video: img("bg_video")?,
        open_video: img("open_video")?,
        frame_image: img("frame_image")?,
        card_deco: img("card_deco")?,
        float_deco: anim_key(&f.raw("float_deco"), "none"),
        open_anim: anim_key(&f.raw("open_anim"), "none"),
        page_mode: f.pick("page_mode", PAGE_MODES.iter().map(|x| x.0), "tab"),
        scroll_anim: anim_key(&f.raw("scroll_anim"), "naik"),
        gerak_judul: f.pick("gerak_judul", gerak(), "ikut"),
        gerak_foto: f.pick("gerak_foto", gerak(), "ikut"),
        ken_burns: f.flag("ken_burns"),
        ornaments: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warna_dan_url_disaring() {
        assert!(is_color("#fff") && is_color("#1a2b3c") && is_color("#1a2b3c80"));
        assert!(!is_color("red") && !is_color("#12345") && !is_color("#fff;} body{x"));
        assert!(is_safe_url("https://img.example.com/a.png") && is_safe_url("/img/a.png"));
        assert!(!is_safe_url("javascript:alert(1)") && !is_safe_url("https://x/a.png\");}") && !is_safe_url("//evil.com/a"));
    }

    #[test]
    fn css_hanya_berisi_nilai_aman() {
        let mut t = ThemeInfo { slug: "uji".into(), layout: "gerbang".into(), font: "cinzel".into(), ..Default::default() };
        t.tokens.insert("bg".into(), "#101010".into());
        t.tokens.insert("primary".into(), "red;}*{x:y".into());
        let css = catalog_css(&[t.clone(), ThemeInfo { slug: "Bad Slug}".into(), ..Default::default() }], &[]);
        assert!(css.contains(".th-uji{--bg:#101010;"));
        assert!(!css.contains("red;") && !css.contains("Bad"));
        // Token tak diisi diturunkan dari token lain.
        assert!(css.contains("--surface:color-mix("));
        assert!(css.contains("--font-display:\"Cinzel\"") && css.contains("--hero-radius:"));
        // Tema tanpa token apa pun = tampilan :root apa adanya.
        assert!(!theme_vars(&ThemeInfo::fallback()).contains("--surface:"));
    }

    #[test]
    fn form_admin() {
        let f = |pairs: &'static [(&'static str, &'static str)]| {
            move |k: &str| pairs.iter().find(|p| p.0 == k).map(|p| p.1.to_string()).unwrap_or_default()
        };
        let t = from_form(f(&[
            ("slug", "Bali Sunset!"), ("name", "Bali Sunset"), ("layout", "gerbang"), ("font", "comic-sans"),
            ("tok_bg", "#FFF8F0"), ("tok_primary", "merah"), ("tok_surface", "#eeeeee"), ("tags", "A, B ,,C"), ("listed", "1"),
        ]))
        .unwrap();
        assert_eq!(t.slug, "bali-sunset");
        assert_eq!((t.layout.as_str(), t.font.as_str()), ("gerbang", "playfair"));
        assert_eq!(t.tokens.get("bg").map(String::as_str), Some("#fff8f0"));
        // Warna tak valid dibuang; token lanjutan diabaikan tanpa adv=1.
        assert!(!t.tokens.contains_key("primary") && !t.tokens.contains_key("surface"));
        assert_eq!(t.tags, vec!["A", "B", "C"]);
        assert!(t.listed && !t.dark);
        assert!(from_form(f(&[("slug", "ok-slug")])).is_err());
        assert!(from_form(f(&[("slug", "ab"), ("name", "X")])).is_err());
        assert!(from_form(f(&[("slug", "abc"), ("name", "X"), ("image_url", "javascript:x")])).is_err());
    }

    #[test]
    fn bahan_tampilan_baru() {
        let t = ThemeInfo {
            slug: "x".into(),
            script_font: "pinyon".into(),
            bg_image: "/img/tema/lily-bg.svg".into(),
            frame_image: "https://cdn.x/frame.png".into(),
            card_deco: "javascript:alert(1)".into(),
            ..Default::default()
        };
        let v = theme_vars(&t);
        assert!(v.contains("--font-script:\"Pinyon Script\""));
        assert!(v.contains("--bg-illus:url(\"/img/tema/lily-bg.svg\")") && v.contains("--card-glass"));
        assert!(v.contains("--frame-img:url(\"https://cdn.x/frame.png\")"));
        assert!(!v.contains("javascript"));
        assert_eq!(font_families(&[t]), vec!["Pinyon+Script"]);
        let f = |k: &str| match k { "slug" => "abc".into(), "name" => "X".into(), "page_mode" => "satu".into(), "float_deco" => "kupu".into(), "bg_image" => "bukan url".into(), _ => String::new() };
        assert!(from_form(f).is_err());
    }

    #[test]
    fn animasi_per_tema() {
        let mut t = ThemeInfo { slug: "x".into(), scroll_anim: "geser".into(), ..Default::default() };
        let anims = super::super::anim::merge(vec![]);
        assert!(theme_vars_with(&t, &anims).contains("--rv-from-alt:translateX(48px)"));
        assert!(!t.single_page());
        t.open_anim = "amplop".into();
        assert!(t.single_page(), "animasi pembuka → otomatis satu halaman");
        // Semua paket memakai kunci yang valid.
        let has = |kind: &str, k: &str| (k == "none" && kind != "scroll") || anims.iter().any(|a| a.kind == kind && a.slug == k);
        for p in MOTION_PRESETS {
            assert!(has("buka", p.3) && has("scroll", p.4) && has("hiasan", p.5), "{}", p.0);
        }
    }

    #[test]
    fn gambar_admin_menimpa_ornamen() {
        let t = ThemeInfo { ornament: "daun".into(), image_url: "https://cdn.x/o.png".into(), image_mode: "pola".into(), ..Default::default() };
        let v = theme_vars(&t);
        assert!(v.contains("url(\"https://cdn.x/o.png\")") && v.contains("repeat"));
        let bad = ThemeInfo { ornament: "daun".into(), image_url: "https://x/a\")".into(), ..Default::default() };
        assert!(theme_vars(&bad).contains("data:image/svg+xml"));
    }
}
