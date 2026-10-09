//! web/rupa.rs — RUPA: varian STRUKTUR per bagian undangan.
//!
//! Dulu 50 tema berbagi satu kerangka (sampul lengkung, mempelai lengkung
//! bertumpuk, kartu acara blok tanggal, galeri kisi) — beda warna & motif
//! saja, sehingga diukur piksel pun "struktur 0,99 identik". Rupa memecah
//! tampilan menjadi 8 BAGIAN; tiap bagian punya beberapa varian bentuk, dan
//! tiap tema memilih satu varian per bagian (`themes.rupa` JSONB, migrasi 033).
//!
//! Semua varian = CSS murni di atas markup yang sama (tanpa komponen baru →
//! WASM tak bertambah). CSS-nya HANYA ikut build server (`cfg(ssr)`) dan
//! disajikan per tema di `/tema/{slug}.css` bersama variabel & animasi tema
//! itu saja — undangan tak lagi memuat CSS semua tema.
//!
//! Selektor tiap berkas `style/rupa/{b}-{varian}.css` diawali kelas akar
//! `.inv.rp-{b}-{varian}` (b = huruf bagian). Varian "asli" = tampilan dasar
//! main.css (tanpa kelas, tanpa CSS).

use std::collections::BTreeMap;

pub struct Bagian {
    /// Kunci di JSON `rupa` (= nama bagian).
    pub key: &'static str,
    /// Huruf kelas: `rp-{huruf}-{varian}`.
    pub huruf: &'static str,
    pub label: &'static str,
    /// (kunci varian, label). Yang pertama selalu "asli".
    pub varian: &'static [(&'static str, &'static str)],
}

pub const ASLI: &str = "asli";

pub const BAGIAN: &[Bagian] = &[
    Bagian {
        key: "sampul",
        huruf: "s",
        label: "Sampul",
        varian: &[
            (ASLI, "Lengkung (bawaan)"),
            ("kubah", "Kubah — pintu masjid/istana Melayu"),
            ("gunungan", "Gunungan — siluet wayang"),
            ("atap", "Atap runcing — gonjong/bolon/tongkonan"),
            ("lingkaran", "Lingkaran bercincin"),
            ("jendela", "Jendela berbingkai geser"),
            ("penuh", "Foto penuh, nama di atas foto"),
            ("wajik", "Belah ketupat"),
            ("medali", "Oval medali"),
        ],
    },
    Bagian {
        key: "judul",
        huruf: "j",
        label: "Judul bagian",
        varian: &[
            (ASLI, "Kaligrafi + garis ◆ (bawaan)"),
            ("kapital", "Huruf kapital berjarak"),
            ("pita", "Di atas pita"),
            ("bingkai", "Dalam bingkai bersudut"),
            ("samping", "Rata kiri bergaris tegak"),
            ("tumpal", "Deret tumpal di atas"),
        ],
    },
    Bagian {
        key: "mempelai",
        huruf: "m",
        label: "Mempelai",
        varian: &[
            (ASLI, "Panel lengkung bertumpuk (bawaan)"),
            ("sejajar", "Dua potret bulat berdampingan"),
            ("zigzag", "Selang-seling kiri–kanan"),
            ("medali", "Medali bercincin tebal"),
            ("panel", "Satu panel terbagi dua"),
            ("potret", "Potret penuh, nama di bawah foto"),
        ],
    },
    Bagian {
        key: "kisah",
        huruf: "c",
        label: "Kisah cinta",
        varian: &[
            (ASLI, "Garis waktu dalam satu kartu (bawaan)"),
            ("kartu", "Tiap babak satu kartu"),
            ("selang", "Foto selang-seling kiri–kanan"),
            ("angka", "Bab bernomor"),
            ("tengah", "Garis tengah, foto bulat"),
            ("polaroid", "Polaroid & judul tulisan tangan"),
        ],
    },
    Bagian {
        key: "acara",
        huruf: "a",
        label: "Kartu acara",
        varian: &[
            (ASLI, "Bingkai ganda + blok tanggal (bawaan)"),
            ("tiket", "Tiket berlubang sobek"),
            ("lontar", "Lembar lontar bergulung"),
            ("linimasa", "Garis waktu"),
            ("kalender", "Lembar kalender"),
            ("tegas", "Kepala berwarna tegas"),
        ],
    },
    Bagian {
        key: "galeri",
        huruf: "g",
        label: "Galeri",
        varian: &[
            (ASLI, "Kisi dua kolom (bawaan)"),
            ("kolom", "Tiga kolom rapat"),
            ("film", "Pita film geser"),
            ("mozaik", "Mozaik satu besar"),
            ("polaroid", "Polaroid miring"),
        ],
    },
    Bagian {
        key: "kartu",
        huruf: "k",
        label: "Gaya kartu",
        varian: &[
            (ASLI, "Lembut membulat (bawaan)"),
            ("tegas", "Sudut tajam bergaris"),
            ("kaca", "Kaca bening"),
            ("potong", "Sudut terpotong"),
            ("bertepi", "Tepi atas bermotif"),
            ("cetak", "Bayang cetak bergeser"),
        ],
    },
    Bagian {
        key: "pemisah",
        huruf: "p",
        label: "Pemisah bagian",
        varian: &[
            (ASLI, "Tanpa pemisah (bawaan)"),
            ("wajik", "Deret belah ketupat"),
            ("ombak", "Garis ombak"),
            ("tumpal", "Deret tumpal"),
            ("titik", "Titik-titik"),
            ("ganda", "Garis ganda"),
        ],
    },
];

pub fn bagian(key: &str) -> Option<&'static Bagian> {
    BAGIAN.iter().find(|b| b.key == key)
}

/// Saring: hanya bagian & varian terdaftar, "asli" dibuang (= bawaan).
pub fn sanitize(rupa: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    rupa.iter()
        .filter(|(k, v)| v.as_str() != ASLI && bagian(k).is_some_and(|b| b.varian.iter().any(|(x, _)| x == v)))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

/// Kelas akar undangan, diawali spasi: " rp-s-kubah rp-j-pita …".
pub fn classes(rupa: &BTreeMap<String, String>) -> String {
    let mut out = String::new();
    for b in BAGIAN {
        if let Some(v) = rupa.get(b.key).filter(|v| v.as_str() != ASLI && b.varian.iter().any(|(x, _)| x == *v)) {
            out.push_str(&format!(" rp-{}-{v}", b.huruf));
        }
    }
    out
}

/// `?rupa=sampul:kubah,judul:pita` (hanya undangan demo — uji coba bentuk).
pub fn parse_override(q: &str) -> BTreeMap<String, String> {
    let m = q
        .split(',')
        .filter_map(|kv| kv.split_once(':'))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .collect();
    sanitize(&m)
}

/// Berapa bagian yang berbeda antara dua rupa ("asli" = absen).
pub fn jarak(a: &BTreeMap<String, String>, b: &BTreeMap<String, String>) -> usize {
    BAGIAN
        .iter()
        .filter(|g| a.get(g.key).map_or(ASLI, String::as_str) != b.get(g.key).map_or(ASLI, String::as_str))
        .count()
}

#[cfg(feature = "ssr")]
mod gaya {
    /// (kelas, CSS). Satu berkas per varian — ditambah di sini saat varian baru dibuat.
    macro_rules! berkas {
        ($($n:literal),* $(,)?) => { &[$(($n, include_str!(concat!("../../style/rupa/", $n, ".css")))),*] };
    }
    pub const CSS: &[(&str, &str)] = berkas![
        "s-kubah", "s-gunungan", "s-atap", "s-lingkaran", "s-jendela", "s-penuh", "s-wajik", "s-medali",
        "j-kapital", "j-pita", "j-bingkai", "j-samping", "j-tumpal",
        "m-sejajar", "m-zigzag", "m-medali", "m-panel", "m-potret",
        "c-kartu", "c-selang", "c-angka", "c-tengah", "c-polaroid",
        "a-tiket", "a-lontar", "a-linimasa", "a-kalender", "a-tegas",
        "g-kolom", "g-film", "g-mozaik", "g-polaroid",
        "k-tegas", "k-kaca", "k-potong", "k-bertepi", "k-cetak",
        "p-wajik", "p-ombak", "p-tumpal", "p-titik", "p-ganda",
    ];
}

/// CSS varian yang dipakai satu rupa.
#[cfg(feature = "ssr")]
pub fn css(rupa: &BTreeMap<String, String>) -> String {
    let mut out = String::new();
    for b in BAGIAN {
        if let Some(v) = rupa.get(b.key) {
            let name = format!("{}-{v}", b.huruf);
            if let Some((_, c)) = gaya::CSS.iter().find(|(n, _)| *n == name) {
                out.push_str(c);
                out.push('\n');
            }
        }
    }
    out
}

/// CSS semua varian (pratinjau admin & halaman demo yang bisa ganti rupa).
#[cfg(feature = "ssr")]
pub fn all_css() -> String {
    gaya::CSS.iter().map(|(_, c)| *c).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varian_terdaftar_punya_css_dan_sebaliknya() {
        #[cfg(feature = "ssr")]
        {
            for b in BAGIAN {
                assert_eq!(b.varian[0].0, ASLI, "{}", b.key);
                for (v, _) in &b.varian[1..] {
                    let n = format!("{}-{v}", b.huruf);
                    let c = gaya::CSS.iter().find(|(x, _)| *x == n).unwrap_or_else(|| panic!("CSS {n} belum ada"));
                    assert!(c.1.contains(&format!(".rp-{n}")), "{n}: selektor wajib memakai .rp-{n}");
                    assert!(!c.1.contains('<'), "{n}");
                }
            }
            for (n, _) in gaya::CSS {
                let (h, v) = n.split_once('-').unwrap();
                assert!(BAGIAN.iter().any(|b| b.huruf == h && b.varian.iter().any(|(x, _)| x == &v)), "{n} tak terdaftar");
            }
        }
    }

    #[test]
    fn saring_kelas_jarak() {
        let r = parse_override("sampul:kubah,judul:asli,mempelai:ngawur,x:y,galeri:film");
        assert_eq!(r.len(), 2);
        assert_eq!(classes(&r), " rp-s-kubah rp-g-film");
        assert_eq!(jarak(&r, &BTreeMap::new()), 2);
    }
}
