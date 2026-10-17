//! web/jelajah.rs — tema JELAJAH 3D: tamu dibawa MASUK ke dunia 3D dan
//! diputar berkeliling (seperti DUNIA 3D beranda, pages/hub3d.rs). Enam
//! "lensa" berdiri melingkar mengelilingi bangunan pusat tema; di tiap lensa
//! sepasang mempelai (insan laki-laki & perempuan, busana/pose berbeda) berdiri
//! di depan properti (lengkung bunga, bangku, ayunan, meja kue, lampu gantung,
//! bingkai hati). Kamera berputar 60° per lensa sambil mundur-maju saat gulir.
//!
//! SEMUA teks lensa = data undangan yang bisa disunting pemesan (/buat &
//! /kelola/{slug}/sunting): nama panggilan & tanggal, nama lengkap + orang tua,
//! dua babak kisah cinta (cadangan: kutipan/doa), acara, salam penutup.
//! Markup di pages/undangan.rs (`Jelajah3d`); CSS = `jelajah/_dasar.css`
//! (+ variabel dunia) — hanya ikut di /gaya/{slug}.css tema itu. Lapisan SVG
//! disajikan /panggung/{nama}.svg (web/panggung.rs, tak ada aset ganda).

#[cfg_attr(not(feature = "ssr"), allow(dead_code))]
pub struct Dunia {
    pub slug: &'static str,
    /// Latar (langit + cakrawala) & bangunan pusat lingkaran.
    jauh: &'static str,
    pusat: &'static str,
    /// Keluarga sepasang mempelai: insan-modern | insan-adat | insan-muslim | insan-oriental.
    pub insan: &'static str,
}

pub const DUNIA: &[Dunia] = &[
    Dunia { slug: "balon", jauh: "balon-jauh", pusat: "balon-utama", insan: "insan-modern" },
    Dunia { slug: "kastil", jauh: "kastil-jauh", pusat: "kastil-utama", insan: "insan-modern" },
    Dunia { slug: "kunang", jauh: "kunang-jauh", pusat: "kunang-utama", insan: "insan-modern" },
    Dunia { slug: "teh", jauh: "teh-jauh", pusat: "teh-utama", insan: "insan-adat" },
];

/// Properti tiap lensa (urutan = urutan lensa) — scripts/gerak/tema3d_baru.py.
pub const PROPERTI: [&str; 6] = ["jl-lengkung", "jl-bangku", "jl-ayunan", "jl-meja", "jl-lampu", "jl-hati"];
/// Varian busana/pose mempelai per lensa ("" = asli; lihat panggung.py `insan`).
pub const VARIAN_INSAN: [&str; 6] = ["", "-1", "-3", "-2", "-4", "-5"];

/// Dunia jelajah untuk tema; "" = bukan tema jelajah (pakai panggung biasa).
pub fn pilih(slug: &str) -> &'static str {
    match slug {
        "langit-balon-3d" => "balon",
        "kastil-mawar-3d" => "kastil",
        "hutan-kunang-3d" => "kunang",
        "kebun-teh-3d" => "teh",
        _ => "",
    }
}

pub fn dunia(slug: &str) -> Option<&'static Dunia> {
    DUNIA.iter().find(|d| d.slug == slug)
}

/// Nama lapisan insan di lensa ke-`i`.
pub fn insan(d: &Dunia, i: usize) -> String {
    format!("{}{}", d.insan, VARIAN_INSAN[i % VARIAN_INSAN.len()])
}

/// CSS satu dunia: dasar + variabel lapisan (URL /panggung/…, di-cache sehari).
#[cfg(feature = "ssr")]
pub fn css(slug: &str) -> String {
    let Some(d) = dunia(slug) else { return String::new() };
    format!(
        "{dasar}\n.jl--{s}{{--jl-jauh:url(/panggung/{j}.svg);--jl-pusat:url(/panggung/{p}.svg)}}\n",
        dasar = include_str!("jelajah/_dasar.css"),
        s = d.slug,
        j = d.jauh,
        p = d.pusat,
    )
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    #[test]
    fn semua_lapisan_dunia_ada() {
        for d in DUNIA {
            for n in [d.jauh, d.pusat] {
                assert!(super::super::panggung::lapisan(n).is_some(), "{n}");
            }
            for i in 0..6 {
                assert!(super::super::panggung::lapisan(&insan(d, i)).is_some(), "{}", insan(d, i));
            }
            let c = css(d.slug);
            assert_eq!(c.matches('{').count(), c.matches('}').count(), "{}", d.slug);
        }
        for p in PROPERTI {
            assert!(super::super::panggung::lapisan(p).is_some(), "{p}");
        }
        for t in ["langit-balon-3d", "kastil-mawar-3d", "hutan-kunang-3d", "kebun-teh-3d"] {
            assert!(dunia(pilih(t)).is_some(), "{t}");
        }
        assert_eq!(pilih("botanical-heritage"), "");
    }
}
