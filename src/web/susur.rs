//! web/susur.rs — tema SUSUR 3D: tamu DIAJAK BERJALAN menyusuri sebuah dunia
//! (desa & sawah, pesisir nelayan) dari gapura sampai pelaminan. Beda dengan
//! JELAJAH (kamera memutari lingkaran, web/jelajah.rs) dan PANGGUNG (lapisan
//! datar yang diterbangi, web/panggung.rs): di sini kamera melangkah MAJU di
//! jalan setapak yang berkelok — naik-turun dan bergoyang halus tiap langkah,
//! berhenti di enam SINGGAH lalu menoleh ke pemandangan (saung, kincir,
//! lumbung, mercusuar, …). Rumah, lumbung, pendopo & mercusuar dirakit dari
//! bidang CSS 3D sungguhan (dinding + atap limasan/pelana), bukan gambar.
//!
//! Satu sumber kebenaran: jalur, singgah & benda didefinisikan di
//! susur/dunia.rs (dipakai bersama perender WebGL2 susur-gl/); dari
//! data yang sama dibangkitkan (1) keyframes kamera, hadap papan & geser latar
//! (`css`) dan (2) `--off` tiap benda = % gulir saat benda dilewati kamera →
//! disembunyikan (visibility) sebelum masuk ke belakang mata.
//!
//! Teks singgah = data undangan (lihat pages/undangan.rs `Susur3d`). Aset
//! gambar dari scripts/gerak/susur.py (src/web/susur/svg), ikut sebagai data
//! URI di /gaya/{slug}.css tema itu saja.
//!
//! PERENDER: dunia digambar WebGL2 oleh crate susur-gl/ (WASM ±120 KB
//! terpisah dari bundel Leptos, /susur/gl.js) — satu kanvas + satu atlas
//! gambar, memori GPU kecil & tetap. Di bawah perspektif CSS, browser
//! merasterisasi TIAP bidang 3D sebagai lapisan sendiri pada ukuran CSS × DPR
//! (±150 lapisan, 45–100 MB, naik-turun ikut gulir) — dunia CSS 3D di sini
//! (`isi`, `css`) kini hanya CADANGAN di <template> bila WebGL2/WASM gagal:
//! dirakit SEKALI per proses lalu disimpan (per request hanya menyisipkan
//! nama mempelai), tetap ditekan memorinya (zoom ½, `.ss-r2`, tanah polos).
#![cfg_attr(not(feature = "ssr"), allow(dead_code))]

// Data & matematika dunia (tanpa dependensi) — dipakai juga crate susur-gl/.
mod dunia;
pub use dunia::*;

/// Gaya inline satu benda.
pub fn gaya(b: &Benda, off: Option<f32>) -> String {
    let mut s = format!("--x:{:.0}px;--z:{:.0}px", b.x, b.z);
    let px = |s: &mut String, k: &str, v: f32| {
        if v != 0.0 {
            s.push_str(&format!(";--{k}:{v:.0}px"));
        }
    };
    px(&mut s, "y", b.y);
    px(&mut s, "w", b.w);
    px(&mut s, "h", b.h);
    px(&mut s, "d", b.d);
    px(&mut s, "t", b.t);
    if b.ry != 0.0 {
        s.push_str(&format!(";--ry:{:.1}deg", b.ry));
    }
    if let Some(o) = off {
        s.push_str(&format!(";--off:{o:.1}%;--ot:{o:.1}"));
    }
    s
}

/// Jarak terdekat (px) yang pernah dicapai kamera ke benda tegak — di atas
/// ini benda tak pernah tampil lebih besar dari ±⅓ ukuran CSS-nya.
const JAUH: f32 = 1500.0;

/// Benda tegak yang selama tampak SELALU ≥ `JAUH` dari mata → dirasterisasi
/// setengah resolusi (`.ss-r2`: zoom ½ + scale 2, geometri sama). Bidang
/// tanah tidak (ubin polos sudah gratis), begitu pula benda yang berputar
/// lewat `transform` (kincir, sinar) — `scale` memindah poros putarnya.
pub fn kasar(tabel: &[Pose], b: &Benda, off: Option<f32>) -> bool {
    if b.datar || b.kelas.contains("ss-kincir") || b.kelas.contains("ss-sinar") {
        return false;
    }
    let akhir = off.map_or(tabel.len(), |o| ((o * 10.0) as usize).min(tabel.len()));
    let r = b.w.max(b.d) / 2.0;
    tabel[..akhir].iter().all(|k| dalam(k, b.x, b.z) - r >= JAUH)
}

/// Penanda teks papan di HTML dunia yang disimpan (diganti per request).
#[cfg(feature = "ssr")]
const TEKS: [&str; 2] = ["\u{1}nama\u{1}", "\u{1}salam\u{1}"];

/// HTML isi `.ss__dunia` satu dunia: dirakit sekali per proses (jalur,
/// tabel pose 0,1 %, `--off` & resolusi tiap benda), lalu tiap request hanya
/// mengganti penanda nama mempelai & salam (ter-escape).
#[cfg(feature = "ssr")]
pub fn isi(slug: &str, pasangan: &str, salam: &str) -> String {
    use std::sync::OnceLock;
    static ISI: [OnceLock<String>; DUNIA.len()] = [const { OnceLock::new() }; DUNIA.len()];
    let Some(i) = DUNIA.iter().position(|d| d.slug == slug) else {
        return String::new();
    };
    let html = ISI[i].get_or_init(|| rakit(&DUNIA[i]));
    let e = super::fmt::html_escape;
    html.replace(TEKS[0], &e(pasangan)).replace(TEKS[1], &e(salam))
}

#[cfg(feature = "ssr")]
fn rakit(d: &Dunia) -> String {
    let j = d.jalur();
    let t = tabel(d, &j);
    let mut s = String::new();
    for b in d.benda(&j) {
        let o = off(&t, &b);
        let tanda = |ya: bool, k: &'static str| if ya { k } else { "" };
        s.push_str(&format!(
            "<div class=\"ss-b {}{}{}\" style=\"{}\">",
            b.kelas,
            tanda(o.is_some(), " ss-o"),
            tanda(kasar(&t, &b, o), " ss-r2"),
            gaya(&b, o)
        ));
        for _ in 0..b.anak {
            s.push_str("<i></i>");
        }
        if b.teks > 0 {
            s.push_str(TEKS[b.teks as usize - 1]);
        }
        s.push_str("</div>");
    }
    s
}

// ───────────────────────────── CSS ─────────────────────────────

/// Aset gambar dunia (nama berkas di susur/svg tanpa .svg).
#[cfg(feature = "ssr")]
fn svg(nama: &str) -> &'static str {
    match nama {
        "kelapa" => include_str!("susur/svg/kelapa.svg"),
        "pisang" => include_str!("susur/svg/pisang.svg"),
        "bambu" => include_str!("susur/svg/bambu.svg"),
        "padi" => include_str!("susur/svg/padi.svg"),
        "padi-kuning" => include_str!("susur/svg/padi-kuning.svg"),
        "orang-orangan" => include_str!("susur/svg/orang-orangan.svg"),
        "penjor" => include_str!("susur/svg/penjor.svg"),
        "gapura-sawah" => include_str!("susur/svg/gapura-sawah.svg"),
        "gapura-pesisir" => include_str!("susur/svg/gapura-pesisir.svg"),
        "gebyok" => include_str!("susur/svg/gebyok.svg"),
        "kincir" => include_str!("susur/svg/kincir.svg"),
        "layangan" => include_str!("susur/svg/layangan.svg"),
        "gunung" => include_str!("susur/svg/gunung.svg"),
        "perahu" => include_str!("susur/svg/perahu.svg"),
        "kursi" => include_str!("susur/svg/kursi.svg"),
        "pandan" => include_str!("susur/svg/pandan.svg"),
        "layar" => include_str!("susur/svg/layar.svg"),
        "bohlam" => include_str!("susur/svg/bohlam.svg"),
        _ => "",
    }
}

/// GET /susur/svg/{nama}.svg — gambar benda untuk atlas perender WebGL2.
#[cfg(feature = "ssr")]
pub fn svg_aset(nama: &str) -> Option<&'static str> {
    Some(svg(nama)).filter(|s| !s.is_empty())
}

// ───────────────────────────── perender WebGL2 ─────────────────────────────

/// Perender WebGL2 (crate susur-gl/, dibangun scripts/susur-gl.sh) — WASM
/// kecil terpisah dari bundel Leptos, dimuat langsung tanpa menunggu hydrate.
#[cfg(feature = "ssr")]
pub const GL_WASM: &[u8] = include_bytes!("susur/gl/susur_gl_bg.wasm");
#[cfg(feature = "ssr")]
const GL_GLUE: &str = include_str!("susur/gl/susur_gl.js");

/// Hash isi perender + aset SVG → `?v=` (berubah otomatis tiap dibangun ulang).
#[cfg(feature = "ssr")]
pub fn gl_versi() -> &'static str {
    use std::sync::OnceLock;
    static V: OnceLock<String> = OnceLock::new();
    V.get_or_init(|| {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let svg = ASET.iter().flat_map(|(_, a)| a.iter()).flat_map(|n| svg(n).bytes());
        for b in GL_WASM.iter().copied().chain(GL_GLUE.bytes()).chain(svg) {
            h ^= b as u64;
            h = h.wrapping_mul(0x1000_0000_01b3);
        }
        format!("{:08x}", h as u32)
    })
}

/// GET /susur/gl.js — glue wasm-bindgen + pemanggil init (URL wasm
/// ber-versi). WASM gagal dimuat/dijalankan → dunia CSS cadangan (kanvas
/// disembunyikan, tidak dibuang — hydrate Leptos mengharapkannya ada).
#[cfg(feature = "ssr")]
pub fn gl_js() -> &'static str {
    use std::sync::OnceLock;
    static JS: OnceLock<String> = OnceLock::new();
    JS.get_or_init(|| {
        format!(
            "{GL_GLUE}\n__wbg_init({{module_or_path:\"/susur/gl.wasm?v={}\"}}).catch(()=>{{for(const k of document.querySelectorAll(\"canvas.ss__gl\")){{\
const b=k.closest(\".pg.ss\"),d=b&&b.querySelector(\".ss__dunia\"),t=b&&b.querySelector(\"template.ss__cadangan\");if(d&&t)d.innerHTML=t.innerHTML;k.classList.add(\"is-gagal\")}}}});\n",
            gl_versi()
        )
    })
}

/// Aset per dunia → variabel `--ss-{nama}` di akar.
#[cfg(feature = "ssr")]
const ASET: &[(&str, &[&str])] = &[
    ("sawah", &["kelapa", "pisang", "bambu", "padi", "padi-kuning", "orang-orangan", "penjor", "gapura-sawah", "gebyok", "kincir", "layangan", "gunung"]),
    ("pesisir", &["kelapa", "perahu", "gapura-pesisir", "kursi", "pandan", "layar", "bohlam"]),
];

/// Keyframes kamera/hadap/latar satu dunia — dari `Dunia::pose`.
#[cfg(feature = "ssr")]
fn keyframes(d: &Dunia) -> String {
    let j = d.jalur();
    let kunci = d.kunci();
    let mut kamera = String::from("@keyframes ss-kamera{");
    let mut hadap = String::from("@keyframes ss-hadap{");
    let mut latar = String::from("@keyframes ss-latar{");
    for &p in &kunci {
        let k = d.pose(&j, p);
        kamera.push_str(&format!(
            "{p:.1}%{{transform:rotateZ({:.2}deg) rotateY({:.2}deg) translate3d({:.1}px,{:.1}px,{:.1}px)}}",
            k.goyang, k.a, -k.x, k.angguk, -k.z
        ));
        hadap.push_str(&format!("{p:.1}%{{rotate:y {:.2}deg}}", -k.a));
        latar.push_str(&format!("{p:.1}%{{translate:calc(var(--pp) * {:.3}) 0}}", -k.a.to_radians().tan()));
    }
    format!("{kamera}}}\n{hadap}}}\n{latar}}}\n")
}

/// CSS satu dunia: dasar + aset (data URI) + keyframes yang dibangkitkan.
#[cfg(feature = "ssr")]
pub fn css(slug: &str) -> String {
    let Some(d) = dunia(slug) else { return String::new() };
    let mut out = String::from(include_str!("susur/_dasar.css"));
    out.push_str(match d.slug {
        "sawah" => include_str!("susur/_sawah.css"),
        _ => include_str!("susur/_pesisir.css"),
    });
    out.push_str(&format!(".ss--{}{{", d.slug));
    for n in ASET.iter().find(|(s, _)| *s == d.slug).map(|(_, a)| *a).unwrap_or(&[]) {
        out.push_str(&format!("--ss-{n}:{};", super::pendamping::data_uri(svg(n))));
    }
    out.push_str(&format!("--ss-insan:url(/panggung/{}.svg);--ss-lengkung:url(/panggung/jl-lengkung.svg)}}\n", d.insan));
    out.push_str(&keyframes(d));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kamera_diam_di_singgah_dan_benda_dilewati() {
        for d in DUNIA {
            let j = d.jalur();
            // Diam: pose sama di tepi kiri & kanan rentang diam.
            for i in 0..6 {
                let (a, b) = ((pusat(i) - DIAM).max(0.0), (pusat(i) + DIAM).min(100.0));
                let (ka, kb) = (d.pose(&j, a), d.pose(&j, b));
                assert!((ka.x - kb.x).abs() < 0.01 && (ka.z - kb.z).abs() < 0.01 && (ka.a - kb.a).abs() < 0.01, "{} {i}", d.slug);
                assert_eq!(ka.angguk, 0.0);
            }
            // Gerak menyambung (tak ada lompatan antar 0,1 %).
            let t = tabel(d, &j);
            for w in t.windows(2) {
                let lompat = (w[1].x - w[0].x).hypot(w[1].z - w[0].z);
                assert!(lompat < 60.0, "{} lompat {lompat}", d.slug);
                assert!((w[1].a - w[0].a).abs() < 3.0, "{} putar", d.slug);
            }
            let benda = d.benda(&j);
            assert!(benda.len() < 320, "{} benda {}", d.slug, benda.len());
            // Tak ada benda tegak yang sudah "dilewati" sejak awal (tersembunyi
            // dari mula); bidang tanah sedekat itu memang di luar layar.
            for b in benda.iter().filter(|b| !b.datar) {
                if let Some(o) = off(&t, b) {
                    assert!(o > 0.0, "{} {} ({}, {}) tersembunyi sejak awal", d.slug, b.kelas, b.x, b.z);
                }
            }
            // Pusat pelaminan masih terlihat di singgah terakhir.
            let insan = benda.iter().find(|b| b.kelas.contains("ss-insan")).unwrap();
            assert!(off(&t, insan).is_none(), "{}", d.slug);
            let k = d.pose(&j, 100.0);
            assert!(dalam(&k, insan.x, insan.z) > 500.0, "{}", d.slug);
        }
        assert_eq!(pilih("desa-sawah-3d"), "sawah");
        assert_eq!(pilih("kebun-teh-3d"), "");
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn css_seimbang_dan_aset_ada() {
        for d in DUNIA {
            let c = css(d.slug);
            assert_eq!(c.matches('{').count(), c.matches('}').count(), "{}", d.slug);
            assert!(c.contains("@keyframes ss-kamera{0.0%"), "{}", d.slug);
        }
        for (_, a) in ASET {
            for n in *a {
                assert!(!svg(n).is_empty(), "{n}");
            }
        }
        for d in DUNIA {
            assert!(super::super::panggung::lapisan(d.insan).is_some());
        }
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn isi_dunia_tersimpan_dan_teks_ter_escape() {
        for d in DUNIA {
            let h = isi(d.slug, "Ana <b> & \"Budi\"", "Halo");
            assert!(!h.contains('\u{1}') && !h.contains("<b>"), "{}", d.slug);
            assert!(h.contains("Halo"), "{}", d.slug);
            assert_eq!(h.matches("<div").count(), h.matches("</div>").count());
            // Kedua kalinya dari simpanan, nama berbeda.
            assert!(isi(d.slug, "Citra", "Halo").len() < h.len() || d.slug != "sawah");
        }
        assert!(isi("sawah", "Ana <b>", "").contains("Ana &lt;b&gt;"));
        // Desa: ada benda yang tak pernah didekati → setengah resolusi.
        assert!(isi("sawah", "", "").contains("ss-r2"));
        assert!(isi("kebun", "", "").is_empty());
    }
}
