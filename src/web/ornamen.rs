//! web/ornamen.rs — "Lapisan Ornamen & Gerak": hiasan bergambar yang
//! ditempel ke tiap bagian undangan (sampul, mempelai, kisah, galeri, acara,
//! RSVP) — ala undangan premium: rangkaian bunga di sudut yang muncul dengan
//! animasi lalu bergoyang pelan.
//!
//! Semua disimpan sebagai DATA terstruktur (tabel `theme_ornaments`), bukan
//! HTML/CSS mentah: posisi, ukuran, putaran, animasi masuk, dan gerak diam
//! dipilih dari daftar tetap → aman dari injeksi dan tetap bisa diatur penuh
//! dari /admin/tema/{slug}/ornamen. Fungsi di sini murni (tanpa IO) — dipakai
//! server (render undangan) dan WASM (pratinjau langsung di admin).
//!
//! Juga: gerak per jenis elemen (judul / foto) dan efek Ken Burns per tema
//! — keduanya berupa variabel CSS di `/tema.css`.

use serde::{Deserialize, Serialize};

use super::skin::is_safe_url;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Ornament {
    pub id: i64,
    pub theme: String,
    /// Kunci `BAGIAN`.
    pub bagian: String,
    pub img: String,
    /// Kunci `POSISI` — titik jangkar di bagian.
    pub posisi: String,
    /// Geser dari jangkar, % lebar/tinggi ornamen sendiri (−100…100).
    pub x: i32,
    pub y: i32,
    /// Lebar, % lebar bagian (5…100).
    pub lebar: i32,
    /// Putaran dasar (derajat).
    pub rotasi: i32,
    /// Cermin mendatar (ranting kiri dipakai ulang di kanan).
    pub cermin: bool,
    /// Kunci `MASUK` — animasi saat bagian terlihat.
    pub masuk: String,
    /// Jeda sebelum masuk (ms).
    pub jeda: i32,
    /// Lama animasi masuk (ms).
    pub durasi: i32,
    /// Kunci `GERAK` — gerak diam berulang setelah masuk.
    pub gerak: String,
    /// Lama satu putaran gerak diam (detik).
    pub kecepatan: i32,
    /// Di depan isi (true) atau di belakang (false).
    pub depan: bool,
    /// Tampil di layar HP.
    pub hp: bool,
    /// 10…100 (%).
    pub opasitas: i32,
    pub urutan: i32,
}

impl Default for Ornament {
    fn default() -> Self {
        Ornament {
            id: 0,
            theme: String::new(),
            bagian: "sampul".into(),
            img: String::new(),
            posisi: "kiri-atas".into(),
            x: -20,
            y: -15,
            lebar: 42,
            rotasi: 0,
            cermin: false,
            masuk: "zoom".into(),
            jeda: 0,
            durasi: 1200,
            gerak: "goyang".into(),
            kecepatan: 6,
            depan: false,
            hp: true,
            opasitas: 100,
            urutan: 100,
        }
    }
}

/// (kunci, label, label pendek) bagian undangan yang bisa diberi ornamen —
/// urut sesuai letaknya di undangan.
pub const BAGIAN: &[(&str, &str, &str)] = &[
    ("sampul", "Sampul (foto & nama)", "Sampul"),
    ("mempelai", "Kedua Mempelai", "Mempelai"),
    ("kisah", "Love Story", "Kisah"),
    ("galeri", "Galeri Foto", "Galeri"),
    ("acara", "Pembuka Acara", "Acara"),
    ("rsvp", "Pembuka RSVP", "RSVP"),
];

/// Urutan bagian (untuk mengurutkan daftar ornamen di admin).
pub fn bagian_index(k: &str) -> usize {
    BAGIAN.iter().position(|b| b.0 == k).unwrap_or(BAGIAN.len())
}

/// (kunci, label, gaya jangkar CSS, transform-origin gerak diam)
pub const POSISI: &[(&str, &str, &str, &str)] = &[
    ("kiri-atas", "Kiri atas", "left:0;top:0;", "0% 0%"),
    ("tengah-atas", "Tengah atas", "left:50%;top:0;--oax:-50%;", "50% 0%"),
    ("kanan-atas", "Kanan atas", "right:0;top:0;", "100% 0%"),
    ("kiri-tengah", "Kiri tengah", "left:0;top:50%;--oay:-50%;", "0% 50%"),
    ("kanan-tengah", "Kanan tengah", "right:0;top:50%;--oay:-50%;", "100% 50%"),
    ("kiri-bawah", "Kiri bawah", "left:0;bottom:0;", "0% 100%"),
    ("tengah-bawah", "Tengah bawah", "left:50%;bottom:0;--oax:-50%;", "50% 100%"),
    ("kanan-bawah", "Kanan bawah", "right:0;bottom:0;", "100% 100%"),
    ("tengah", "Tengah (di balik isi)", "left:50%;top:50%;--oax:-50%;--oay:-50%;", "50% 50%"),
];

/// (kunci, label) animasi masuk — keyframes `orn-in-{kunci}` di main.css.
pub const MASUK: &[(&str, &str)] = &[
    ("none", "Langsung tampil"),
    ("pudar", "Memudar"),
    ("zoom", "Membesar"),
    ("mekar", "Mekar berputar"),
    ("ayun-kiri", "Berayun turun dari kiri"),
    ("ayun-kanan", "Berayun turun dari kanan"),
    ("putar-kiri", "Berputar dari kiri"),
    ("putar-kanan", "Berputar dari kanan"),
    ("naik", "Naik dari bawah"),
    ("turun", "Turun dari atas"),
    ("geser-kiri", "Masuk dari kiri"),
    ("geser-kanan", "Masuk dari kanan"),
    ("tumbuh", "Tumbuh dari pangkal"),
];

/// (kunci, label) gerak diam berulang — keyframes `orn-idle-{kunci}`.
pub const GERAK: &[(&str, &str)] = &[
    ("none", "Diam"),
    ("goyang", "Bergoyang tertiup angin"),
    ("melayang", "Melayang naik-turun"),
    ("denyut", "Berdenyut halus"),
    ("putar", "Berputar pelan"),
    ("kilau", "Berkilau"),
];

/// Gerak per jenis elemen saat muncul (judul / foto). "ikut" = sama dengan
/// gerak scroll tema. (kunci, label, --from transform, --filter)
pub const GERAK_ELEMEN: &[(&str, &str, &str, &str)] = &[
    ("ikut", "Ikut gerak scroll tema", "", ""),
    ("naik", "Naik", "translateY(36px)", "none"),
    ("turun", "Turun", "translateY(-36px)", "none"),
    ("pudar", "Memudar", "none", "none"),
    ("zoom", "Membesar", "scale(.82)", "none"),
    ("zoom-masuk", "Zoom masuk dari kecil", "scale(.3)", "none"),
    ("zoom-keluar", "Mengecil ke tempat", "scale(1.18)", "none"),
    ("geser-kiri", "Dari kiri", "translateX(-60px)", "none"),
    ("geser-kanan", "Dari kanan", "translateX(60px)", "none"),
    ("blur", "Dari samar", "scale(1.04)", "blur(12px)"),
    ("lipat", "Membuka lipatan", "perspective(600px) rotateX(-70deg)", "none"),
    ("putar", "Berputar", "rotate(-8deg) scale(.9)", "none"),
    ("lebar", "Huruf merenggang", "scale(.94)", "blur(4px)"),
];

/// Variabel CSS gerak judul/foto + Ken Burns satu tema (bagian dari `.th-{slug}`).
pub fn element_vars(judul: &str, foto: &str, ken_burns: bool) -> String {
    let mut s = String::new();
    for (k, p) in [(judul, "rvj"), (foto, "rvf")] {
        if let Some(g) = GERAK_ELEMEN.iter().find(|g| g.0 == k && g.0 != "ikut") {
            s.push_str(&format!("--{p}-from:{};--{p}-filter:{};", g.2, g.3));
            if k == "lebar" {
                s.push_str(&format!("--{p}-ls:.3em;"));
            }
        }
    }
    if ken_burns {
        s.push_str("--kb:orn-kb;");
    }
    s
}

impl Ornament {
    /// Gaya inline wrapper: jangkar + variabel. Semua nilai angka/daftar tetap.
    pub fn style(&self) -> String {
        let pos = POSISI.iter().find(|p| p.0 == self.posisi).unwrap_or(&POSISI[0]);
        format!(
            "{}--ox:{}%;--oy:{}%;--ow:{}%;--orot:{}deg;--osx:{};--ojeda:{}ms;--odur:{}ms;--ospd:{}s;--oop:{};--oorg:{};",
            pos.2,
            self.x.clamp(-100, 100),
            self.y.clamp(-100, 100),
            self.lebar.clamp(5, 100),
            self.rotasi.clamp(-180, 180),
            if self.cermin { -1 } else { 1 },
            self.jeda.clamp(0, 5000),
            self.durasi.clamp(200, 5000),
            self.kecepatan.clamp(2, 60),
            f64::from(self.opasitas.clamp(10, 100)) / 100.0,
            pos.3,
        )
    }

    pub fn class(&self) -> String {
        let m = super::fmt::pick(&self.masuk, MASUK.iter().map(|m| m.0), "none");
        let g = super::fmt::pick(&self.gerak, GERAK.iter().map(|m| m.0), "none");
        format!("orn orn--in-{m} orn--idle-{g}{}", if self.hp { "" } else { " orn--no-hp" })
    }

    pub fn bagian_label(&self) -> &'static str {
        BAGIAN.iter().find(|b| b.0 == self.bagian).map(|b| b.1).unwrap_or("?")
    }
}

/// Susun ornamen dari form admin. `img` sudah berisi URL unggahan bila ada.
pub fn from_form(get: impl Fn(&str) -> String) -> Result<Ornament, String> {
    let f = super::fmt::Fields(get);
    let img = f.text("img", 500);
    if img.is_empty() {
        return Err("Gambar ornamen wajib diisi (unggah PNG/SVG/WebP transparan atau tulis URL-nya).".into());
    }
    if !is_safe_url(&img) {
        return Err("URL gambar ornamen tidak valid — harus https://… atau /img/…, tanpa spasi/kutip.".into());
    }
    let d = Ornament::default();
    Ok(Ornament {
        id: f.text("id", 20).parse().unwrap_or(0),
        theme: f.text("theme", 60),
        bagian: f.pick("bagian", BAGIAN.iter().map(|b| b.0), "sampul"),
        img,
        posisi: f.pick("posisi", POSISI.iter().map(|p| p.0), "kiri-atas"),
        x: f.int("x", -100, 100, 0),
        y: f.int("y", -100, 100, 0),
        lebar: f.int("lebar", 5, 100, d.lebar),
        rotasi: f.int("rotasi", -180, 180, 0),
        cermin: f.flag("cermin"),
        masuk: f.pick("masuk", MASUK.iter().map(|m| m.0), "zoom"),
        jeda: f.int("jeda", 0, 5000, 0),
        durasi: f.int("durasi", 200, 5000, d.durasi),
        gerak: f.pick("gerak", GERAK.iter().map(|m| m.0), "none"),
        kecepatan: f.int("kecepatan", 2, 60, d.kecepatan),
        depan: f.flag("depan"),
        hp: f.flag("hp"),
        opasitas: f.int("opasitas", 10, 100, 100),
        urutan: f.int("urutan", 0, 9999, 100),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn form_disaring() {
        let f = |pairs: &'static [(&'static str, &'static str)]| {
            move |k: &str| pairs.iter().find(|p| p.0 == k).map(|p| p.1.to_string()).unwrap_or_default()
        };
        let o = from_form(f(&[
            ("img", "/img/tema/ornamen/mawar.svg"), ("bagian", "galeri"), ("posisi", "ngawur"), ("x", "-300"),
            ("lebar", "abc"), ("masuk", "mekar"), ("gerak", "<script>"), ("hp", "1"), ("opasitas", "5"),
        ]))
        .unwrap();
        assert_eq!((o.bagian.as_str(), o.posisi.as_str(), o.x, o.lebar), ("galeri", "kiri-atas", -100, 42));
        assert_eq!((o.masuk.as_str(), o.gerak.as_str(), o.hp, o.opasitas), ("mekar", "none", true, 10));
        assert!(from_form(f(&[("img", "javascript:alert(1)")])).is_err());
        assert!(from_form(f(&[])).is_err());
    }

    #[test]
    fn gaya_hanya_angka_aman() {
        let o = Ornament { posisi: "kanan-bawah".into(), cermin: true, x: 999, ..Default::default() };
        let st = o.style();
        assert!(st.starts_with("right:0;bottom:0;") && st.contains("--osx:-1;") && st.contains("--ox:100%;"));
        assert_eq!(Ornament { gerak: "x;}".into(), hp: false, ..Default::default() }.class(), "orn orn--in-zoom orn--idle-none orn--no-hp");
    }

    #[test]
    fn variabel_elemen() {
        let v = element_vars("zoom", "ikut", true);
        assert!(v.contains("--rvj-from:scale(.82)") && !v.contains("--rvf-") && v.contains("--kb:orn-kb"));
        assert_eq!(element_vars("", "bukan", false), "");
    }
}
