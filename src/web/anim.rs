//! web/anim.rs — SEMUA animasi undangan (tabel `animations`): cara membuka,
//! gerak saat scroll, dan hiasan melayang — bawaan maupun buatan admin.
//!
//! Satu animasi didefinisikan dengan salah satu cara:
//!   * PENGATURAN (`spec`): angka & pilihan yang dijepit ke rentang aman lalu
//!     diubah jadi CSS di sini — untuk admin non-teknis.
//!   * CSS LANJUTAN (`css`): CSS mentah dengan penanda `{a}` = kelas akar
//!     animasi (`.gate--slug` / `.float-deco--slug` / `.rvs--slug`). Scroll
//!     boleh berupa deklarasi variabel `--rv-*` saja (ditempel ke tema), ATAU
//!     aturan lengkap ber-`{a}` — "koreografi": gerak berbeda per jenis elemen
//!     (judul, teks, foto, kartu). Akar undangan selalu berkelas `rvs--{scroll}`.
//!     Animasi bawaan memakai cara ini.
//! Hasilnya dilayani di /tema.css; tema memakai animasi lewat kuncinya (= slug)
//! di kolom open_anim / scroll_anim / float_deco.
//!
//! `builtins()` = isi pabrik animasi bawaan: diisikan ke DB saat start (bila
//! belum ada), dipakai tombol "Kembalikan ke bawaan", dan cadangan bila tabel
//! belum dimigrasi.
//!
//! Tiga jenis:
//!   buka   — gerbang sampul (pintu kiri-kanan / atas-bawah / utuh), gerak,
//!            isi yang keluar, ornamen tengah.
//!   scroll — gerak isi saat muncul (geser, zoom, putar, blur).
//!   hiasan — gambar melayang (jatuh / naik / berkedip).
//!
//! Fungsi di sini murni: server memakainya untuk /tema.css, WASM untuk
//! pratinjau langsung di editor admin.

use serde::{Deserialize, Serialize};

use super::skin::is_safe_url;

pub const KINDS: &[(&str, &str, &str)] = &[
    ("buka", "Cara membuka undangan", "Sampul tertutup yang terbuka saat tamu menekan \"Buka Undangan\"."),
    ("scroll", "Gerak saat scroll", "Cara tiap bagian undangan muncul ketika tamu menggulir."),
    ("hiasan", "Hiasan melayang", "Gambar kecil yang bergerak di latar selama undangan dibuka."),
];

/// Kunci "tanpa animasi" untuk cara membuka & hiasan (bukan baris tabel).
pub const NONE: &str = "none";
/// Batas panjang CSS lanjutan per animasi.
pub const CSS_MAX: usize = 20_000;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AnimSpec {
    // ── buka ──
    /// lr = dua pintu kiri-kanan | tb = atas-bawah | full = satu lembar
    pub split: String,
    /// bg | primary | gold | image
    pub fill: String,
    pub panel_image: String,
    /// Jarak geser pintu (% ukuran pintu), 0–110.
    pub move_pct: i32,
    /// Putaran 3D pintu (derajat), -180–180.
    pub rotate: i32,
    /// Skala akhir pintu ×100 (50–150).
    pub scale: i32,
    pub fade: bool,
    pub border: bool,
    pub duration_ms: i32,
    pub delay_ms: i32,
    /// lembut | pegas | tegas | linear
    pub easing: String,
    /// pudar | mengecil | naik | membesar
    pub content_exit: String,
    pub orn_image: String,
    /// membesar | berputar | jatuh
    pub orn_effect: String,
    pub orn_size: i32,
    // ── scroll ──
    pub dx: i32,
    pub dy: i32,
    /// Skala awal ×100 (60–140).
    pub scale_from: i32,
    pub rot: i32,
    pub blur: i32,
    /// Elemen genap bergerak dari arah berlawanan.
    pub mirror: bool,
    // ── hiasan ──
    pub float_image: String,
    /// jatuh | naik | kedip
    pub float_motion: String,
    pub float_size: i32,
    /// Detik per putaran (4–30).
    pub float_speed: i32,
    pub float_count: i32,
}

impl Default for AnimSpec {
    fn default() -> Self {
        AnimSpec {
            split: "lr".into(),
            fill: "bg".into(),
            panel_image: String::new(),
            move_pct: 101,
            rotate: 0,
            scale: 100,
            fade: false,
            border: true,
            duration_ms: 1100,
            delay_ms: 250,
            easing: "tegas".into(),
            content_exit: "pudar".into(),
            orn_image: String::new(),
            orn_effect: "membesar".into(),
            orn_size: 70,
            dx: 0,
            dy: 28,
            scale_from: 100,
            rot: 0,
            blur: 0,
            mirror: false,
            float_image: String::new(),
            float_motion: "jatuh".into(),
            float_size: 18,
            float_speed: 12,
            float_count: 9,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AnimInfo {
    /// Kunci yang disimpan tema (unik per jenis).
    pub slug: String,
    /// buka | scroll | hiasan
    pub kind: String,
    pub name: String,
    pub spec: AnimSpec,
    /// CSS lanjutan; kosong = dibangkitkan dari `spec`.
    pub css: String,
    /// Animasi bawaan (tak bisa dihapus, bisa dikembalikan ke bawaan).
    pub builtin: bool,
    pub sort_order: i32,
}

impl AnimInfo {
    pub fn key(&self) -> String {
        self.slug.clone()
    }
    /// Kelas akar pengganti `{a}` di CSS lanjutan.
    pub fn root_class(&self) -> String {
        match self.kind.as_str() {
            "hiasan" => format!(".float-deco--{}", self.slug),
            "scroll" => format!(".rvs--{}", self.slug),
            _ => format!(".gate--{}", self.slug),
        }
    }
}

pub const EASINGS: &[(&str, &str, &str)] = &[
    ("lembut", "Lembut", "cubic-bezier(0.2,0.7,0.2,1)"),
    ("tegas", "Tegas (cepat di tengah)", "cubic-bezier(0.7,0,0.2,1)"),
    ("pegas", "Memantul (pegas)", "cubic-bezier(0.34,1.56,0.64,1)"),
    ("linear", "Rata", "linear"),
];
pub const SPLITS: &[(&str, &str)] = &[("lr", "Dua pintu kiri–kanan"), ("tb", "Dua bagian atas–bawah"), ("full", "Satu lembar utuh")];
pub const FILLS: &[(&str, &str)] = &[("bg", "Warna latar tema"), ("primary", "Warna utama tema"), ("gold", "Emas"), ("image", "Gambar unggahan")];
pub const EXITS: &[(&str, &str)] = &[("pudar", "Memudar"), ("mengecil", "Mengecil"), ("naik", "Naik ke atas"), ("membesar", "Membesar")];
pub const ORN_EFFECTS: &[(&str, &str)] = &[("membesar", "Membesar lalu hilang"), ("berputar", "Berputar lalu hilang"), ("jatuh", "Jatuh ke bawah")];
pub const FLOAT_MOTIONS: &[(&str, &str)] = &[("jatuh", "Jatuh perlahan"), ("naik", "Terbang naik"), ("kedip", "Berkedip di tempat")];

/// Templat awal untuk animasi baru (kind, kunci, label, spesifikasi).
pub fn templates() -> Vec<(&'static str, &'static str, &'static str, AnimSpec)> {
    let d = AnimSpec::default();
    vec![
        ("buka", "pintu-geser", "Pintu geser", d.clone()),
        ("buka", "pintu-3d", "Pintu 3D berayun", AnimSpec { move_pct: 0, rotate: 105, fill: "primary".into(), duration_ms: 1300, ..d.clone() }),
        ("buka", "atas-bawah", "Terbelah atas–bawah", AnimSpec { split: "tb".into(), ..d.clone() }),
        ("buka", "lembar-naik", "Lembar terangkat", AnimSpec { split: "full".into(), move_pct: 105, easing: "lembut".into(), ..d.clone() }),
        ("buka", "segel", "Segel pecah + pudar", AnimSpec { split: "full".into(), move_pct: 0, fade: true, scale: 115, orn_size: 90, ..d.clone() }),
        ("scroll", "naik", "Naik perlahan", d.clone()),
        ("scroll", "geser-cermin", "Geser kiri–kanan", AnimSpec { dx: -50, dy: 0, mirror: true, ..d.clone() }),
        ("scroll", "zoom-putar", "Zoom sambil berputar", AnimSpec { dy: 0, scale_from: 80, rot: -8, ..d.clone() }),
        ("hiasan", "jatuh", "Jatuh perlahan", d.clone()),
        ("hiasan", "naik", "Terbang naik", AnimSpec { float_motion: "naik".into(), float_size: 24, ..d.clone() }),
        ("hiasan", "kedip", "Berkedip", AnimSpec { float_motion: "kedip".into(), float_size: 14, ..d }),
    ]
}

fn safe_url(u: &str) -> String {
    if is_safe_url(u) { u.to_string() } else { String::new() }
}

impl AnimSpec {
    /// Jepit semua nilai ke rentang aman — CSS dari data ini selalu valid.
    pub fn sanitized(&self) -> AnimSpec {
        AnimSpec {
            split: super::fmt::pick(&self.split, SPLITS.iter().map(|o| o.0), "lr"),
            fill: super::fmt::pick(&self.fill, FILLS.iter().map(|o| o.0), "bg"),
            panel_image: safe_url(&self.panel_image),
            move_pct: self.move_pct.clamp(0, 110),
            rotate: self.rotate.clamp(-180, 180),
            scale: self.scale.clamp(50, 150),
            fade: self.fade,
            border: self.border,
            duration_ms: self.duration_ms.clamp(300, 3000),
            delay_ms: self.delay_ms.clamp(0, 1500),
            easing: super::fmt::pick(&self.easing, EASINGS.iter().map(|e| e.0), "tegas"),
            content_exit: super::fmt::pick(&self.content_exit, EXITS.iter().map(|o| o.0), "pudar"),
            orn_image: safe_url(&self.orn_image),
            orn_effect: super::fmt::pick(&self.orn_effect, ORN_EFFECTS.iter().map(|o| o.0), "membesar"),
            orn_size: self.orn_size.clamp(30, 200),
            dx: self.dx.clamp(-120, 120),
            dy: self.dy.clamp(-120, 120),
            scale_from: self.scale_from.clamp(60, 140),
            rot: self.rot.clamp(-45, 45),
            blur: self.blur.clamp(0, 20),
            mirror: self.mirror,
            float_image: safe_url(&self.float_image),
            float_motion: super::fmt::pick(&self.float_motion, FLOAT_MOTIONS.iter().map(|o| o.0), "jatuh"),
            float_size: self.float_size.clamp(8, 60),
            float_speed: self.float_speed.clamp(4, 30),
            float_count: self.float_count.clamp(3, 14),
        }
    }

    fn ease(&self) -> &'static str {
        EASINGS.iter().find(|e| e.0 == self.easing).map(|e| e.2).unwrap_or("ease")
    }

    /// Variabel `--rv-*` untuk gerak scroll kustom (ditambahkan ke vars tema).
    pub fn scroll_vars(&self) -> String {
        let s = self.sanitized();
        let tf = |dx: i32| format!("translate({dx}px,{}px) scale({:.2}) rotate({}deg)", s.dy, s.scale_from as f32 / 100.0, s.rot);
        let mut v = format!("--rv-from:{};--rv-dur:{:.2}s;", tf(s.dx), s.duration_ms as f32 / 1000.0);
        if s.mirror {
            v.push_str(&format!("--rv-from-alt:{};", tf(-s.dx)));
        }
        if s.blur > 0 {
            v.push_str(&format!("--rv-filter:blur({}px);", s.blur));
        }
        v
    }
}

pub const KEY_MAX: usize = 40;

/// Asal (`https://host`) yang boleh dipakai `url(…)` di CSS lanjutan selain
/// path lokal & `data:image/` — diisi server saat start (RustFS publik &
/// SITE_URL). CSS ini disajikan ke SEMUA tamu: tanpa batas ini, satu akun admin
/// yang bocor bisa memasang gambar pelacak dari domain luar.
static URL_ORIGINS: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();

pub fn set_url_origins(v: Vec<String>) {
    let _ = URL_ORIGINS.set(v.into_iter().filter(|o| !o.is_empty()).map(|o| o.to_ascii_lowercase()).collect());
}

/// `https://host[:port]` dari sebuah URL, atau None.
pub fn origin_of(u: &str) -> Option<String> {
    let rest = u.strip_prefix("https://").or_else(|| u.strip_prefix("http://"))?;
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    (!host.is_empty()).then(|| format!("{}{}", &u[..u.len() - rest.len()], host).to_ascii_lowercase())
}

/// Periksa semua `url(…)`: path lokal, `data:image/`, atau asal milik sendiri.
/// Pratinjau WASM (daftar asal tak diisi) menerima http(s) — server tetap
/// memeriksa saat menyimpan dan saat merender /tema.css.
pub fn check_css_urls(css: &str) -> Result<(), String> {
    let low = css.to_ascii_lowercase();
    let mut from = 0;
    while let Some(i) = low[from..].find("url(") {
        let start = from + i + 4;
        let arg = low[start..].trim_start().trim_start_matches(['"', '\'']).trim_start();
        let end = arg.find(['"', '\'', ')', ' ']).unwrap_or(arg.len());
        let target = &arg[..end];
        let ok = target.starts_with("data:image/")
            || (target.starts_with('/') && !target.starts_with("//"))
            || match (origin_of(target), URL_ORIGINS.get()) {
                (Some(o), Some(list)) => list.iter().any(|x| *x == o),
                (Some(_), None) => !cfg!(feature = "ssr"),
                _ => false,
            };
        if !ok {
            return Err(format!("url() hanya boleh berkas situs ini (/…), data:image, atau penyimpanan sendiri — bukan \"{target}\"."));
        }
        from = start;
    }
    Ok(())
}

/// Periksa CSS lanjutan. Bukan parser CSS penuh — cukup mencegah keluar dari
/// `<style>`/stylesheet & pemuatan luar: tanpa `<`, `@import`, `expression(`,
/// `javascript:`; kurung kurawal seimbang. Buka/hiasan wajib memakai `{a}`
/// (agar hanya menyentuh animasinya sendiri); scroll = deklarasi saja, atau
/// aturan lengkap ber-`{a}` (koreografi).
pub fn sanitize_css(kind: &str, css: &str) -> Result<String, String> {
    let css = css.trim().replace("\r\n", "\n");
    if css.is_empty() {
        return Ok(css);
    }
    if css.len() > CSS_MAX {
        return Err(format!("CSS terlalu panjang (maks {} karakter).", CSS_MAX));
    }
    let low = css.to_ascii_lowercase();
    if css.contains('<') {
        return Err("CSS tidak boleh memuat karakter \"<\".".into());
    }
    for bad in ["@import", "expression(", "javascript:", "behavior:", "-moz-binding"] {
        if low.contains(bad) {
            return Err(format!("CSS tidak boleh memuat \"{bad}\"."));
        }
    }
    check_css_urls(&css)?;
    if kind == "scroll" && !css.contains("{a}") {
        if css.contains('{') || css.contains('}') || css.contains('@') {
            return Err("Gerak scroll berisi deklarasi variabel (mis. --rv-from:translateY(28px);) atau aturan lengkap yang memakai {a}.".into());
        }
        return Ok(if css.ends_with(';') { css } else { format!("{css};") });
    }
    let mut depth = 0i32;
    for c in css.chars() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth < 0 {
                    break;
                }
            }
            _ => {}
        }
    }
    if depth != 0 {
        return Err("Kurung kurawal { } pada CSS tidak seimbang.".into());
    }
    if !css.contains("{a}") {
        return Err("CSS harus memakai penanda {a} untuk kelas animasi ini, mis. {a} .gate__panel--l { … }".into());
    }
    Ok(css)
}

/// Gerak scroll berbentuk aturan lengkap (koreografi), bukan deklarasi saja.
pub fn is_rules(a: &AnimInfo) -> bool {
    a.kind == "scroll" && a.css.contains("{a}")
}

/// Deklarasi `--rv-*` gerak scroll (ditempel ke variabel tema). Koreografi →
/// kosong (aturannya ikut `css`).
pub fn scroll_vars_of(a: &AnimInfo) -> String {
    if a.kind != "scroll" || is_rules(a) {
        return String::new();
    }
    if a.css.trim().is_empty() {
        a.spec.scroll_vars()
    } else {
        sanitize_css("scroll", &a.css).unwrap_or_default()
    }
}

/// CSS untuk satu animasi buka / hiasan / koreografi scroll. Scroll
/// deklarasi → lewat `scroll_vars_of`.
pub fn css(a: &AnimInfo) -> String {
    if !super::fmt::is_slug(&a.slug, KEY_MAX) || a.slug == NONE || (a.kind == "scroll" && !is_rules(a)) {
        return String::new();
    }
    if !a.css.trim().is_empty() {
        return match sanitize_css(&a.kind, &a.css) {
            Ok(c) => format!("{}\n", c.replace("{a}", &a.root_class())),
            Err(_) => String::new(),
        };
    }
    let s = a.spec.sanitized();
    let k = a.slug.clone();
    match a.kind.as_str() {
        "buka" => open_css(&k, &s),
        "hiasan" => float_css(&k, &s),
        _ => String::new(),
    }
}

fn open_css(k: &str, s: &AnimSpec) -> String {
    let g = format!(".gate--{k}");
    // Selektor "sudah dibuka": halaman tamu (html.inv-opened) & pratinjau (.is-open).
    let opened = |sub: &str| format!(".inv-opened {g}:not(.gate--embed){sub}, .is-open > {g}{sub}");
    let dur = s.duration_ms as f32 / 1000.0;
    let delay = s.delay_ms as f32 / 1000.0;
    let ease = s.ease();
    let fill = match s.fill.as_str() {
        "primary" => "var(--primary)".to_string(),
        "gold" => "var(--foil-deep)".to_string(),
        "image" if !s.panel_image.is_empty() => format!("url(\"{}\") center / cover no-repeat, var(--bg)", s.panel_image),
        _ => "var(--bg)".to_string(),
    };
    let border = if s.border { "inset 0 0 0 1px var(--gold-light)" } else { "none" };
    let (l_box, r_box, axis, rot_axis, l_origin, r_origin) = match s.split.as_str() {
        "tb" => ("left:0;right:0;top:0;bottom:auto;width:100%;height:50.5%;", "left:0;right:0;top:auto;bottom:0;width:100%;height:50.5%;", "Y", "X", "top center", "bottom center"),
        "full" => ("left:0;right:0;top:0;bottom:0;width:100%;height:100%;", "display:none;", "Y", "X", "top center", "center"),
        _ => ("left:0;top:0;bottom:0;width:50.5%;", "right:0;top:0;bottom:0;width:50.5%;", "X", "Y", "left center", "right center"),
    };
    // Pintu A bergerak ke arah negatif, pintu B cermin. "full" = naik ke atas.
    let mv = s.move_pct;
    let sc = s.scale as f32 / 100.0;
    let tf = |sign: i32| {
        let (mv, rot) = if s.split == "full" { (-mv, s.rotate) } else { (sign * mv, sign * -s.rotate) };
        format!("translate{axis}({mv}%) rotate{rot_axis}({rot}deg) scale({sc:.2})")
    };
    let fade = if s.fade { "opacity:0;" } else { "" };
    let exit = match s.content_exit.as_str() {
        "mengecil" => "opacity:0;transform:scale(0.85);",
        "naik" => "opacity:0;transform:translateY(-40px);",
        "membesar" => "opacity:0;transform:scale(1.12);",
        _ => "opacity:0;",
    };
    let orn_exit = match s.orn_effect.as_str() {
        "berputar" => "transform:translate(-50%,-50%) rotate(200deg) scale(0.2);opacity:0;",
        "jatuh" => "transform:translate(-50%,120vh) rotate(40deg);opacity:0;",
        _ => "transform:translate(-50%,-50%) scale(1.9);opacity:0;",
    };
    let total = dur + delay + 0.15;
    let mut c = String::new();
    c.push_str(&format!("{g}{{perspective:1400px}}\n"));
    c.push_str(&format!("{g} .gate__panel{{background:{fill};box-shadow:{border};transition:transform {dur:.2}s {ease} {delay:.2}s,opacity {dur:.2}s ease {delay:.2}s}}\n"));
    c.push_str(&format!("{g} .gate__panel::after{{display:none}}\n"));
    // Pintu berisi gambar/warna kuat: nama & tombol di sampul diberi kartu
    // agar tetap terbaca (seperti animasi bawaan "gerbang").
    if s.fill != "bg" {
        c.push_str(&format!(
            "{g} .gate__content{{background:color-mix(in srgb,var(--card) 92%,transparent);border-radius:28px;margin:auto;max-width:440px;width:calc(100% - 28px);box-shadow:var(--shadow-lg);-webkit-backdrop-filter:blur(4px);backdrop-filter:blur(4px)}}\n"
        ));
    }
    c.push_str(&format!("{g} .gate__panel--l{{{l_box}transform-origin:{l_origin}}}\n"));
    c.push_str(&format!("{g} .gate__panel--r{{{r_box}transform-origin:{r_origin}}}\n"));
    c.push_str(&format!("{} {{transform:{};{fade}}}\n", opened(" .gate__panel--l"), tf(-1)));
    c.push_str(&format!("{} {{transform:{};{fade}}}\n", opened(" .gate__panel--r"), tf(1)));
    c.push_str(&format!("{} {{{exit}}}\n", opened(" .gate__content")));
    c.push_str(&format!("{} {{visibility:hidden;pointer-events:none;transition:visibility 0s {total:.2}s}}\n", opened("")));
    // Ornamen di BELAKANG isi sampul (tak menimpa nama); tampil saat isi
    // memudar ketika sampul dibuka.
    if !s.orn_image.is_empty() {
        c.push_str(&format!(
            "{g} .gate__orn{{display:block;position:absolute;left:50%;top:50%;z-index:0;width:{0}px;height:{0}px;transform:translate(-50%,-50%);background:url(\"{1}\") center / contain no-repeat;transition:transform {dur:.2}s {ease},opacity {dur:.2}s ease;pointer-events:none}}\n",
            s.orn_size, s.orn_image
        ));
        c.push_str(&format!("{} {{{orn_exit}}}\n", opened(" .gate__orn")));
    }
    c
}

fn float_css(k: &str, s: &AnimSpec) -> String {
    let f = format!(".float-deco--{k}");
    let (name, extra) = match s.float_motion.as_str() {
        "naik" => ("fd-fly", "top:auto;bottom:-6%;"),
        "kedip" => ("fd-twinkle", "top:calc(var(--i) * 7% + 4%);"),
        _ => ("fd-fall", ""),
    };
    let img = if s.float_image.is_empty() {
        "radial-gradient(circle,var(--gold-light) 40%,transparent 42%)".to_string()
    } else {
        format!("url(\"{}\") center / contain no-repeat", s.float_image)
    };
    format!(
        "{f} i{{{extra}width:{sz}px;height:{sz}px;border-radius:0;box-shadow:none;background:{img};animation:{name} calc({sp}s + var(--i) * 1.1s) linear infinite;animation-delay:calc(var(--i) * -1.7s)}}\n{f} i:nth-child(n+{n}){{display:none}}\n{f} i:nth-child(-n+{c}){{display:block}}\n",
        sz = s.float_size,
        sp = s.float_speed,
        n = s.float_count + 1,
        c = s.float_count,
    )
}

/// Susun animasi dari isian form admin (`name → value`); nilai dijepit.
/// `mode=css` → CSS lanjutan dipakai; selain itu dari pengaturan.
pub fn from_form(get: impl Fn(&str) -> String) -> Result<AnimInfo, String> {
    let f = super::fmt::Fields(get);
    let slug = f.text("slug", 60).to_lowercase();
    if !super::fmt::is_slug(&slug, KEY_MAX) {
        return Err("Kode animasi hanya huruf kecil, angka, dan tanda minus (maks 40).".into());
    }
    let kind = f.text("kind", 20);
    if !KINDS.iter().any(|k| k.0 == kind) {
        return Err("Jenis animasi tidak dikenal.".into());
    }
    if slug == NONE && kind != "scroll" {
        return Err("Kode \"none\" dipakai untuk \"tanpa animasi\" — pilih kode lain.".into());
    }
    let css = if f.raw("mode") == "css" {
        let c = sanitize_css(&kind, &f.raw("css"))?;
        if c.is_empty() {
            return Err("CSS lanjutan masih kosong — isi CSS-nya atau pilih mode Pengaturan.".into());
        }
        c
    } else {
        String::new()
    };
    let name = f.text("name", 60);
    if name.is_empty() {
        return Err("Nama animasi wajib diisi.".into());
    }
    // Nilai dijepit oleh `sanitized()` di bawah — satu sumber rentang aman.
    let d = AnimSpec::default();
    let spec = AnimSpec {
        split: f.text("split", 20),
        fill: f.text("fill", 20),
        panel_image: f.text("panel_image", 500),
        move_pct: f.num("move_pct", d.move_pct),
        rotate: f.num("rotate", d.rotate),
        scale: f.num("scale", d.scale),
        fade: f.flag("fade"),
        border: f.flag("border"),
        duration_ms: f.num("duration_ms", d.duration_ms),
        delay_ms: f.num("delay_ms", d.delay_ms),
        easing: f.text("easing", 20),
        content_exit: f.text("content_exit", 20),
        orn_image: f.text("orn_image", 500),
        orn_effect: f.text("orn_effect", 20),
        orn_size: f.num("orn_size", d.orn_size),
        dx: f.num("dx", d.dx),
        dy: f.num("dy", d.dy),
        scale_from: f.num("scale_from", d.scale_from),
        rot: f.num("rot", d.rot),
        blur: f.num("blur", d.blur),
        mirror: f.flag("mirror"),
        float_image: f.text("float_image", 500),
        float_motion: f.text("float_motion", 20),
        float_size: f.num("float_size", d.float_size),
        float_speed: f.num("float_speed", d.float_speed),
        float_count: f.num("float_count", d.float_count),
    }
    .sanitized();
    Ok(AnimInfo { slug, kind, name, spec, css, builtin: false, sort_order: 100 })
}

/// Gabungkan baris DB dengan bawaan yang belum ada di DB, urut per jenis.
pub fn merge(db: Vec<AnimInfo>) -> Vec<AnimInfo> {
    let mut all = db;
    for b in builtins() {
        if !all.iter().any(|a| a.kind == b.kind && a.slug == b.slug) {
            all.push(b);
        }
    }
    let kind_order = |k: &str| KINDS.iter().position(|x| x.0 == k).unwrap_or(9);
    all.sort_by(|a, b| (kind_order(&a.kind), a.sort_order, &a.name).cmp(&(kind_order(&b.kind), b.sort_order, &b.name)));
    all
}

// ── Animasi bawaan (isi pabrik) ────────────────────────────────────────────

/// Pola selektor "sudah dibuka": halaman tamu (html.inv-opened) & pratinjau
/// (.is-open). Ditulis lengkap di CSS supaya admin bisa meniru polanya.
const CSS_TIRAI: &str = r#".inv-opened {a}:not(.gate--embed) .gate__panel, .is-open > {a} .gate__panel { transition: transform 1.15s cubic-bezier(0.7, 0, 0.2, 1) 0.25s; }
.inv-opened {a}:not(.gate--embed) .gate__panel--l, .is-open > {a} .gate__panel--l { transform: translateX(-101%); }
.inv-opened {a}:not(.gate--embed) .gate__panel--r, .is-open > {a} .gate__panel--r { transform: translateX(101%); }"#;

const CSS_GERBANG: &str = r#"{a} { perspective: 1400px; }
{a} .gate__panel { background: var(--primary); box-shadow: inset 0 0 0 10px color-mix(in srgb, var(--gold) 70%, transparent), inset 0 0 0 12px var(--gold-light); }
{a} .gate__panel::after { opacity: 0.22; mix-blend-mode: luminosity; }
{a} .gate__panel--l { transform-origin: left center; }
{a} .gate__panel--r { transform-origin: right center; }
{a} .gate__panel::before { content: ""; position: absolute; top: 50%; width: 12px; height: 60px; margin-top: -30px; border-radius: 8px; background: var(--foil-deep); }
{a} .gate__panel--l::before { right: 18px; }
{a} .gate__panel--r::before { left: 18px; }
{a} .gate__content { background: color-mix(in srgb, var(--bg) 88%, transparent); border-radius: 28px; margin: auto; max-width: 440px; width: calc(100% - 28px); box-shadow: var(--shadow-lg); }
.inv-opened {a}:not(.gate--embed) .gate__panel, .is-open > {a} .gate__panel { transition: transform 1.3s cubic-bezier(0.6, 0, 0.2, 1) 0.3s; }
.inv-opened {a}:not(.gate--embed) .gate__panel--l, .is-open > {a} .gate__panel--l { transform: rotateY(105deg); }
.inv-opened {a}:not(.gate--embed) .gate__panel--r, .is-open > {a} .gate__panel--r { transform: rotateY(-105deg); }"#;

/// Tutup amplop: segitiga pendek di atas + segel lilin; isi sampul dimulai
/// DI BAWAH tutup agar segel tak menimpa nama. Di bingkai HP (demo) tinggi
/// layar = tinggi bingkai.
const CSS_AMPLOP: &str = r#"{a} { perspective: 1400px; }
{a} .gate__panel { width: 100%; left: 0; right: 0; }
{a} .gate__panel--l { top: 0; bottom: auto; height: 22vh; min-height: 130px; z-index: 2; transform-origin: top center; background: var(--surface); clip-path: polygon(0 0, 100% 0, 100% 35%, 50% 100%, 0 35%); box-shadow: none; filter: drop-shadow(0 6px 8px rgba(0, 0, 0, 0.12)); }
{a} .gate__panel--r { top: 0; bottom: 0; height: 100%; box-shadow: none; }
{a}::before {
  content: ""; position: absolute; left: 50%; top: max(22vh, 130px); z-index: 4; width: 58px; height: 58px; margin: -40px 0 0 -29px; border-radius: 50%;
  background: radial-gradient(circle at 35% 35%, #d9534f, #8f1f1c); box-shadow: 0 3px 8px rgba(0, 0, 0, 0.3), inset 0 0 0 5px rgba(0, 0, 0, 0.12);
  transition: transform 0.35s, opacity 0.35s;
}
{a} .gate__content { z-index: 3; padding-top: calc(max(22vh, 130px) + 34px); margin-top: 0; }
.inv-opened {a}:not(.gate--embed)::before, .is-open > {a}::before { transform: scale(1.3); opacity: 0; }
.inv-opened {a}:not(.gate--embed) .gate__panel--l, .is-open > {a} .gate__panel--l { transform: rotateX(180deg); opacity: 0; transition: transform 0.9s cubic-bezier(0.5, 0, 0.3, 1) 0.2s, opacity 0.3s 0.85s; }
.inv-opened {a}:not(.gate--embed) .gate__panel--r, .is-open > {a} .gate__panel--r { transform: translateY(105%); transition: transform 1s cubic-bezier(0.6, 0, 0.3, 1) 0.7s; }
.gate--embed{a} .gate__panel--l { height: 130px; }
.gate--embed{a}::before { top: 130px; }
.gate--embed{a} .gate__content { padding-top: 164px; }
@media (max-height: 940px) { {a} .guest__text, {a} .guest .pill { display: none; } }"#;

const CSS_PUDAR: &str = r#"{a} .gate__panel { box-shadow: none; }
.inv-opened {a}:not(.gate--embed), .is-open > {a} { opacity: 0; transform: scale(1.08); transition: opacity 0.9s ease 0.2s, transform 1.2s ease, visibility 0s 1.3s; }"#;

const CSS_KELOPAK: &str = r#"{a} i { width: 12px; height: 16px; border-radius: 80% 0 80% 0; background: linear-gradient(135deg, #fff, var(--gold-pale)); box-shadow: 0 1px 2px rgba(0, 0, 0, 0.08); }
{a} i:nth-child(3n) { background: linear-gradient(135deg, #fff, #f5d5d9); }"#;

const CSS_KUPU: &str = r#"{a} i {
  top: auto; bottom: -6%; width: 22px; height: 18px; animation-name: fd-fly; opacity: 0.9;
  background: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 22 18'%3E%3Cg fill='%23c5a059'%3E%3Cellipse cx='6' cy='6' rx='5.5' ry='5' transform='rotate(-20 6 6)'/%3E%3Cellipse cx='16' cy='6' rx='5.5' ry='5' transform='rotate(20 16 6)'/%3E%3Cellipse cx='7' cy='13' rx='3.5' ry='3.2'/%3E%3Cellipse cx='15' cy='13' rx='3.5' ry='3.2'/%3E%3C/g%3E%3Crect x='10.4' y='3' width='1.2' height='12' rx='.6' fill='%23775a19'/%3E%3C/svg%3E") center / contain no-repeat;
}
{a} i:nth-child(odd) { filter: hue-rotate(80deg) saturate(0.6); }"#;

const CSS_BINTANG: &str = r#"{a} i { width: 4px; height: 4px; border-radius: 50%; background: var(--gold-light); box-shadow: 0 0 8px var(--gold-light); top: calc(var(--i) * 10% + 4%); animation: fd-twinkle calc(2.4s + var(--i) * 0.4s) ease-in-out infinite; }"#;

/// Animasi bawaan (isi pabrik). Kunci sama dengan versi lama di kode, jadi
/// tema yang sudah ada tetap cocok.
pub fn builtins() -> Vec<AnimInfo> {
    let b = |kind: &str, slug: &str, name: &str, css: &str, sort_order: i32| AnimInfo {
        slug: slug.into(),
        kind: kind.into(),
        name: name.into(),
        spec: AnimSpec::default(),
        css: css.into(),
        builtin: true,
        sort_order,
    };
    vec![
        b("buka", "tirai", "Tirai terbuka ke samping", CSS_TIRAI, 10),
        b("buka", "gerbang", "Pintu gerbang 3D berayun", CSS_GERBANG, 20),
        b("buka", "amplop", "Amplop terbuka (tutup terangkat)", CSS_AMPLOP, 30),
        b("buka", "pudar", "Sampul memudar & membesar", CSS_PUDAR, 40),
        b("scroll", "naik", "Naik perlahan", "--rv-from:translateY(28px);", 10),
        b("scroll", "anggun", "Anggun bertahap (ala undangan premium)", "--rv-from:translateY(60px);--rv-dur:1.25s;--rv-stagger:220;", 15),
        b("scroll", "pudar", "Pudar lembut", "--rv-from:none;--rv-dur:1.1s;", 20),
        b("scroll", "zoom", "Membesar (zoom)", "--rv-from:scale(0.9);", 30),
        b("scroll", "geser", "Geser kiri–kanan bergantian", "--rv-from:translateX(-48px);--rv-from-alt:translateX(48px);", 40),
        b("scroll", "lipat", "Terbuka seperti lipatan", "--rv-from:perspective(900px) rotateX(-28deg);--rv-origin:top center;", 50),
        b("scroll", "blur", "Dari samar ke jelas (blur)", "--rv-from:scale(1.04);--rv-filter:blur(10px);--rv-dur:1s;", 60),
        b("scroll", "none", "Tanpa (langsung tampil)", "--rv-from:none;--rv-op:1;--rv-dur:0s;", 70),
        b("hiasan", "kelopak", "Kelopak bunga berguguran", CSS_KELOPAK, 10),
        b("hiasan", "kupu", "Kupu-kupu beterbangan", CSS_KUPU, 20),
        b("hiasan", "bintang", "Kerlip bintang", CSS_BINTANG, 30),
        // Koreografi tema unggulan (CSS di web/gerak/, aset mask di
        // public/img/tema/gerak/ dari scripts/gerak/build.py).
        b("buka", "gebyok-ukir", "Gebyok ukir: sampul tergulir, pintu berayun, menembus kusen", include_str!("gerak/buka-gebyok-ukir.css"), 50),
        b("buka", "taman-daun", "Taman: rimbun daun tersibak, daun beterbangan", include_str!("gerak/buka-taman-daun.css"), 51),
        b("buka", "galaksi", "Galaksi: tersedot ke langit, warp bintang, portal cahaya", include_str!("gerak/buka-galaksi.css"), 52),
        b("buka", "candi-bentar", "Candi bentar: gapura terbelah, matahari terbit", include_str!("gerak/buka-candi-bentar.css"), 53),
        b("buka", "pagelaran-wayang", "Pagelaran wayang: kelir menyala, gunungan dikebutkan, tokoh masuk", include_str!("gerak/buka-pagelaran-wayang.css"), 49),
        b("buka", "layar-naik", "Layar naik: sampul polaroid terangkat seperti layar bioskop", include_str!("gerak/buka-layar-naik.css"), 55),
        b("buka", "video-pintu", "Video pintu: video pembuka tema diputar sekali (cara everlove)", include_str!("gerak/buka-video-pintu.css"), 48),
        b("buka", "tenun-songket", "Tenun songket: helai kain diurai kiri-kanan", include_str!("gerak/buka-tenun-songket.css"), 54),
        // Gerbang alam — tiap gerbang punya gerak TUTUP sendiri (blok .inv-closing di berkasnya).
        b("buka", "air-terjun", "Air terjun: tirai air tersibak; ditutup air tercurah dari atas", include_str!("gerak/buka-air-terjun.css"), 56),
        b("buka", "fajar-rembulan", "Fajar & rembulan: matahari terbit; ditutup senja, bulan naik", include_str!("gerak/buka-fajar-rembulan.css"), 57),
        b("buka", "teratai-mekar", "Teratai mekar: kelopak raksasa mekar; ditutup menguncup", include_str!("gerak/buka-teratai-mekar.css"), 58),
        b("buka", "kupu-kupu", "Kupu-kupu: kawanan mengangkat sampul ke langit; ditutup menurunkannya", include_str!("gerak/buka-kupu-kupu.css"), 59),
        b("buka", "ombak-laut", "Ombak laut: ombak menyapu layar; ditutup pasang naik lalu surut", include_str!("gerak/buka-ombak-laut.css"), 60),
        b("buka", "awan-berarak", "Awan berarak: menembus awan; ditutup awan berkumpul lalu berarak", include_str!("gerak/buka-awan-berarak.css"), 61),
        // Gerbang budaya dunia & religi (3D) — aset scripts/gerak/dunia.py → public/img/tema/dunia/.
        b("buka", "naga-emas", "Naga emas (Tionghoa): pintu pernis merah berayun 3D, naga terbang, lampion naik", include_str!("gerak/buka-naga-emas.css"), 62),
        b("buka", "mashrabiya", "Mashrabiya (Islami): pintu kisi bintang berayun, menembus lengkung, fanous naik", include_str!("gerak/buka-mashrabiya.css"), 63),
        b("buka", "katedral", "Katedral (Kristiani): jendela mawar menyala, pintu gotik berayun, merpati terbang", include_str!("gerak/buka-katedral.css"), 64),
        b("buka", "mandala", "Mandala (Hindu): tiga lapis mandala melesat ke depan, untaian marigold naik", include_str!("gerak/buka-mandala.css"), 65),
        b("buka", "stupa", "Stupa (Buddhis): cahaya merekah, stupa mendekat, bendera doa melayang", include_str!("gerak/buka-stupa.css"), 66),
        b("buka", "shoji-sakura", "Shoji & torii (Jepang): shoji bergeser, melewati torii, sakura gugur 3D", include_str!("gerak/buka-shoji-sakura.css"), 67),
        b("buka", "hanok", "Hanok (Korea): pintu kisi terlipat ke atas 3D, pita dancheong", include_str!("gerak/buka-hanok.css"), 68),
        b("buka", "chofa", "Chofa (Thailand): atap kuil bertingkat terangkat mendekat bergiliran", include_str!("gerak/buka-chofa.css"), 69),
        b("buka", "lampion-hoian", "Lampion Hoi An (Vietnam): lampion sutra terbang naik berlapis kedalaman", include_str!("gerak/buka-lampion-hoian.css"), 70),
        b("scroll", "bayang", "Koreografi Bayang wayang (dari bayangan, tokoh masuk kiri-kanan, judul menyala)", include_str!("gerak/scroll-bayang.css"), 79),
        b("scroll", "sinema", "Koreografi Sinema (untuk video latar: memudar naik tenang, kartu kaca gelap)", include_str!("gerak/scroll-sinema.css"), 85),
        b("scroll", "sekar", "Koreografi Sekar Kedhaton (ala everlove: panel taupe, foto kapsul berlili, isi kartu bergerak, diulang saat digulir)", include_str!("gerak/scroll-sekar.css"), 77),
        b("scroll", "everlove", "Koreografi Everlove (persis undangan premium: 1,5 dtk, serentak, terulang saat digulir)", include_str!("gerak/scroll-everlove.css"), 78),
        b("scroll", "keraton", "Koreografi Keraton (judul zoom, mempelai kiri-kanan, ikon berputar)", include_str!("gerak/scroll-keraton.css"), 80),
        b("scroll", "mekar", "Koreografi Mekar (judul merapat, foto terbuka bundar, kartu kelopak)", include_str!("gerak/scroll-mekar.css"), 81),
        b("scroll", "kosmik", "Koreografi Kosmik (dari samar, judul menyala, mempelai berbalik)", include_str!("gerak/scroll-kosmik.css"), 82),
        b("scroll", "ombak", "Koreografi Ombak (mengayun kiri-kanan, judul memantul, foto 3D)", include_str!("gerak/scroll-ombak.css"), 83),
        b("scroll", "tenun", "Koreografi Tenun (tersingkap seperti benang, foto tirai)", include_str!("gerak/scroll-tenun.css"), 84),
        b("hiasan", "daun-gugur", "Daun gugur berputar 3D", include_str!("gerak/hiasan-daun-gugur.css"), 40),
        b("hiasan", "kunang", "Kunang-kunang & bintang jatuh", include_str!("gerak/hiasan-kunang.css"), 41),
        b("hiasan", "burung", "Kawanan burung terbang melintas", include_str!("gerak/hiasan-burung.css"), 43),
        b("hiasan", "kilau-emas", "Kilau emas berkelip", include_str!("gerak/hiasan-kilau-emas.css"), 42),
    ]
}

pub fn builtin(kind: &str, slug: &str) -> Option<AnimInfo> {
    builtins().into_iter().find(|a| a.kind == kind && a.slug == slug)
}


/// CSS semua animasi kustom (untuk /tema.css).
pub fn catalog_css(list: &[AnimInfo]) -> String {
    list.iter().map(css).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nilai_dijepit_dan_url_disaring() {
        let s = AnimSpec { move_pct: 999, duration_ms: 1, rotate: -999, split: "x".into(), panel_image: "javascript:1".into(), ..Default::default() }.sanitized();
        assert_eq!((s.move_pct, s.duration_ms, s.rotate, s.split.as_str()), (110, 300, -180, "lr"));
        assert!(s.panel_image.is_empty());
    }

    #[test]
    fn css_buka_dan_hiasan() {
        let a = AnimInfo { slug: "pintu-emas".into(), kind: "buka".into(), name: "x".into(), spec: AnimSpec { rotate: 90, orn_image: "/img/segel.png".into(), ..Default::default() }, ..Default::default() };
        let c = css(&a);
        assert!(c.contains(".gate--pintu-emas .gate__panel--l{left:0"));
        assert!(c.contains(".is-open > .gate--pintu-emas .gate__panel--r {transform:translateX(101%) rotateY(-90deg)"));
        assert!(c.contains("url(\"/img/segel.png\")"));
        // Isi bawaan = warna latar → tanpa kartu; isi gambar/warna kuat → kartu.
        assert!(!c.contains(".gate__content{background"));
        let img = AnimInfo { spec: AnimSpec { fill: "image".into(), panel_image: "/p.svg".into(), ..Default::default() }, ..a.clone() };
        assert!(css(&img).contains(".gate--pintu-emas .gate__content{background:color-mix"));
        let h = AnimInfo { slug: "hati".into(), kind: "hiasan".into(), spec: AnimSpec { float_count: 5, float_image: "/img/hati.png".into(), ..Default::default() }, ..Default::default() };
        let c = css(&h);
        assert!(c.contains(".float-deco--hati i{") && c.contains("nth-child(n+6){display:none}"));
        // Slug aneh → tak menghasilkan CSS apa pun.
        assert!(css(&AnimInfo { slug: "x}{".into(), kind: "buka".into(), ..Default::default() }).is_empty());
    }

    #[test]
    fn bawaan_dan_css_lanjutan() {
        // Semua bawaan lolos pemeriksaan & menghasilkan CSS untuk kelasnya sendiri.
        for a in builtins() {
            assert!(sanitize_css(&a.kind, &a.css).is_ok(), "{}/{}", a.kind, a.slug);
            if a.kind == "scroll" && !is_rules(&a) {
                assert!(scroll_vars_of(&a).starts_with("--rv-"));
            } else {
                assert!(a.css.len() <= CSS_MAX, "{} terlalu panjang", a.slug);
                let c = css(&a);
                assert!(c.contains(&a.root_class()) && !c.contains("{a}"), "{}", a.slug);
            }
        }
        assert!(css(&builtin("buka", "tirai").unwrap()).contains(".is-open > .gate--tirai .gate__panel--l { transform: translateX(-101%); }"));
        // CSS berbahaya / tak berpenanda / tak seimbang ditolak.
        assert!(sanitize_css("buka", "{a}{color:red}</style><script>").is_err());
        assert!(sanitize_css("buka", "@import url(x); {a}{}").is_err());
        assert!(sanitize_css("buka", ".gate{color:red}").is_err());
        assert!(sanitize_css("buka", "{a}{color:red").is_err());
        assert!(sanitize_css("scroll", "--rv-from:none}body{display:none").is_err());
        // url(): lokal & data:image boleh; domain luar ditolak (pelacak).
        assert!(sanitize_css("buka", "{a}{background:url(\"/img/a.svg\")}").is_ok());
        assert!(sanitize_css("buka", "{a}{background:url('data:image/svg+xml,%3Csvg/%3E')}").is_ok());
        assert!(sanitize_css("buka", "{a}{background:url(https://evil.tld/t.png)}").is_err());
        assert!(sanitize_css("buka", "{a}{background:URL( //evil.tld/t.png )}").is_err());
        assert_eq!(origin_of("https://Image.Ulalaapi.store/undangan/x.webp").as_deref(), Some("https://image.ulalaapi.store"));
        assert_eq!(sanitize_css("scroll", "--rv-from:none").unwrap(), "--rv-from:none;");
        // Koreografi scroll: aturan lengkap ber-{a} → kelas .rvs--slug, tanpa variabel tema.
        let k = builtin("scroll", "keraton").unwrap();
        assert!(is_rules(&k) && scroll_vars_of(&k).is_empty() && css(&k).contains(".rv-on .rvs--keraton [data-rv]"));
        assert!(css(&builtin("scroll", "naik").unwrap()).is_empty());
        // Baris DB menimpa bawaan dengan kunci sama; bawaan yang tak ada ditambahkan.
        let edited = AnimInfo { name: "Tirai Emas".into(), ..builtin("buka", "tirai").unwrap() };
        let all = merge(vec![edited]);
        assert_eq!(all.iter().filter(|a| a.kind == "buka" && a.slug == "tirai").count(), 1);
        assert_eq!(all.iter().find(|a| a.slug == "tirai").unwrap().name, "Tirai Emas");
        assert_eq!(all.len(), builtins().len());
        assert_eq!(all[0].kind, "buka");
    }

    #[test]
    fn dari_form() {
        let f = |k: &str| match k {
            "slug" => "Pintu-Emas".to_string(),
            "kind" => "buka".into(),
            "name" => "Pintu Emas".into(),
            "move_pct" => "500".into(),
            "fade" => "1".into(),
            _ => String::new(),
        };
        let a = from_form(f).unwrap();
        assert_eq!((a.slug.as_str(), a.spec.move_pct, a.spec.fade, a.spec.border), ("pintu-emas", 110, true, false));
        assert!(from_form(|k| if k == "slug" { "a b".into() } else { "buka".into() }).is_err());
        let css_mode = |k: &str| match k {
            "slug" => "tirai-2".to_string(),
            "kind" => "buka".into(),
            "name" => "Tirai 2".into(),
            "mode" => "css".into(),
            "css" => "{a} .gate__panel { background: gold; }".into(),
            _ => String::new(),
        };
        assert_eq!(from_form(css_mode).unwrap().css, "{a} .gate__panel { background: gold; }");
        assert!(from_form(|k| match k { "slug" => "none".into(), "kind" => "buka".into(), _ => "x".into() }).is_err());
        assert_eq!(super::super::fmt::slug("Pintu Emas  2!", KEY_MAX), "pintu-emas-2");
    }

    #[test]
    fn scroll_kustom() {
        let v = AnimSpec { dx: -50, dy: 0, mirror: true, blur: 4, ..Default::default() }.scroll_vars();
        assert!(v.contains("--rv-from:translate(-50px,0px)") && v.contains("--rv-from-alt:translate(50px,0px)") && v.contains("blur(4px)"));
        for (kind, _, _, spec) in templates() {
            assert!(KINDS.iter().any(|k| k.0 == kind));
            assert_eq!(spec.sanitized(), spec, "templat harus sudah dalam rentang aman");
        }
    }
}
