//! web/panggung.rs — PANGGUNG 3D: prolog sinematik undangan yang digerakkan
//! gulir (kamera maju menembus gerbang → mempelai di depan bangunan utama →
//! mundur memperlihatkan acara), terinspirasi video undangan 3D.
//!
//! Adegan = rakitan tiga lapisan SVG (`panggung/svg`, dibangkitkan
//! `scripts/gerak/panggung.py`): latar JAUH, bangunan UTAMA, gerbang DEPAN —
//! plus warna langit & cahaya. Adegan DITURUNKAN dari tema (slug → nuansa →
//! wilayah), tanpa kolom DB, jadi semua tema satu halaman mendapatkannya. CSS
//! (`panggung/_dasar.css` + lapisan sebagai data URI) hanya ikut di
//! `/gaya/{slug}.css` tema itu.

// Lapisan & warna hanya dibaca saat merakit CSS (server).
#[cfg_attr(not(feature = "ssr"), allow(dead_code))]
pub struct Adegan {
    pub slug: &'static str,
    pub nama: &'static str,
    jauh: &'static str,
    utama: &'static str,
    depan: &'static str,
    langit: &'static str,
    cahaya: &'static str,
}

pub const ADEGAN: &[Adegan] = &[
    Adegan { slug: "masjid", nama: "Masjid Putih", jauh: "masjid-jauh", utama: "masjid-utama", depan: "masjid-depan", langit: "linear-gradient(#d3d9e0, #eceff3 55%, #e9ecef)", cahaya: "radial-gradient(circle at 40% 38%, #fff, #eceff2 62%, #d9dee4)" },
    Adegan { slug: "jawa", nama: "Pendopo & Candi Bentar", jauh: "jawa-jauh", utama: "jawa-utama", depan: "jawa-depan", langit: "linear-gradient(#efe6d6, #f7f2e8 55%, #efece4)", cahaya: "radial-gradient(circle at 45% 45%, #fffaf0, #fbe8c2 55%, rgba(251,232,194,0) 71%)" },
    Adegan { slug: "bali", nama: "Meru & Candi Bentar", jauh: "jawa-jauh", utama: "meru-utama", depan: "jawa-depan", langit: "linear-gradient(#efe6d6, #f7f2e8 55%, #efece4)", cahaya: "radial-gradient(circle at 45% 45%, #fffaf0, #fbe8c2 55%, rgba(251,232,194,0) 71%)" },
    Adegan { slug: "tionghoa", nama: "Gerbang Bulan", jauh: "tionghoa-jauh", utama: "tionghoa-utama", depan: "tionghoa-depan", langit: "linear-gradient(#dfe2e6, #f2f1ef 55%, #ecebe8)", cahaya: "radial-gradient(circle at 40% 38%, #fff, #eceff2 62%, #d9dee4)" },
    Adegan { slug: "kapel", nama: "Kapel & Lengkung Bunga", jauh: "kapel-jauh", utama: "kapel-utama", depan: "kapel-depan", langit: "linear-gradient(#e6e4ea, #f6f3f4 55%, #eeeeec)", cahaya: "radial-gradient(circle at 45% 45%, #fffdf8, #fbeef0 55%, rgba(251,238,240,0) 71%)" },
    Adegan { slug: "minang", nama: "Rumah Gadang", jauh: "jawa-jauh", utama: "gadang-utama", depan: "gapura-depan", langit: "linear-gradient(#efe6d6, #f7f2e8 55%, #efece4)", cahaya: "radial-gradient(circle at 45% 45%, #fffaf0, #fbe8c2 55%, rgba(251,232,194,0) 71%)" },
    Adegan { slug: "toraja", nama: "Tongkonan & Rumah Bolon", jauh: "jawa-jauh", utama: "tongkonan-utama", depan: "gapura-depan", langit: "linear-gradient(#efe6d6, #f7f2e8 55%, #efece4)", cahaya: "radial-gradient(circle at 45% 45%, #fffaf0, #fbe8c2 55%, rgba(251,232,194,0) 71%)" },
    Adegan { slug: "melayu", nama: "Rumah Panggung Melayu", jauh: "jawa-jauh", utama: "panggung-utama", depan: "gapura-depan", langit: "linear-gradient(#f0e2c4, #f8f1e2 55%, #f1ece0)", cahaya: "radial-gradient(circle at 45% 45%, #fffaf0, #fbe8c2 55%, rgba(251,232,194,0) 71%)" },
    Adegan { slug: "bahari", nama: "Rumah Panggung Pesisir", jauh: "laut-jauh", utama: "panggung-utama", depan: "gapura-depan", langit: "linear-gradient(#cfdde6, #eef3f6 55%, #e9eef0)", cahaya: "radial-gradient(circle at 45% 45%, #fffaf0, #fbe8c2 55%, rgba(251,232,194,0) 71%)" },
    Adegan { slug: "dayak", nama: "Rumah Lamin", jauh: "hutan-jauh", utama: "lamin-utama", depan: "gapura-depan", langit: "linear-gradient(#dbe4d8, #f1f4ee 55%, #ebeee8)", cahaya: "radial-gradient(circle at 45% 45%, #fffaf0, #fbe8c2 55%, rgba(251,232,194,0) 71%)" },
    Adegan { slug: "papua", nama: "Honai Lembah", jauh: "hutan-jauh", utama: "honai-utama", depan: "pohon-depan", langit: "linear-gradient(#dbe4d8, #f1f4ee 55%, #ebeee8)", cahaya: "radial-gradient(circle at 45% 45%, #fffaf0, #fbe8c2 55%, rgba(251,232,194,0) 71%)" },
    Adegan { slug: "jepang", nama: "Kuil & Torii", jauh: "fuji-jauh", utama: "kuil-utama", depan: "torii-depan", langit: "linear-gradient(#e6e4ea, #f6f3f4 55%, #eeeeec)", cahaya: "radial-gradient(circle at 45% 45%, #fffdf8, #fbeef0 55%, rgba(251,238,240,0) 71%)" },
    Adegan { slug: "korea", nama: "Istana Hanok", jauh: "hutan-jauh", utama: "tionghoa-utama", depan: "gapura-depan", langit: "linear-gradient(#dfe2e6, #f2f1ef 55%, #ecebe8)", cahaya: "radial-gradient(circle at 40% 38%, #fff, #eceff2 62%, #d9dee4)" },
    Adegan { slug: "thai", nama: "Wat Emas", jauh: "jawa-jauh", utama: "wat-utama", depan: "gapura-depan", langit: "linear-gradient(#f0e2c4, #f8f1e2 55%, #f1ece0)", cahaya: "radial-gradient(circle at 45% 45%, #fffaf0, #fbe8c2 55%, rgba(251,232,194,0) 71%)" },
    Adegan { slug: "buddhis", nama: "Stupa", jauh: "hutan-jauh", utama: "stupa-utama", depan: "gapura-depan", langit: "linear-gradient(#f0e2c4, #f8f1e2 55%, #f1ece0)", cahaya: "radial-gradient(circle at 45% 45%, #fffaf0, #fbe8c2 55%, rgba(251,232,194,0) 71%)" },
    Adegan { slug: "alam", nama: "Paviliun Hutan", jauh: "hutan-jauh", utama: "paviliun-utama", depan: "pohon-depan", langit: "linear-gradient(#dbe4d8, #f1f4ee 55%, #ebeee8)", cahaya: "radial-gradient(circle at 45% 45%, #fffaf0, #fbe8c2 55%, rgba(251,232,194,0) 71%)" },
    Adegan { slug: "laut", nama: "Paviliun Pantai", jauh: "laut-jauh", utama: "paviliun-utama", depan: "kapel-depan", langit: "linear-gradient(#cfdde6, #eef3f6 55%, #e9eef0)", cahaya: "radial-gradient(circle at 45% 45%, #fffaf0, #fbe8c2 55%, rgba(251,232,194,0) 71%)" },
    Adegan { slug: "taman", nama: "Paviliun Taman", jauh: "kapel-jauh", utama: "paviliun-utama", depan: "kapel-depan", langit: "linear-gradient(#d3d9e0, #eceff3 55%, #e9ecef)", cahaya: "radial-gradient(circle at 40% 38%, #fff, #eceff2 62%, #d9dee4)" },
    Adegan { slug: "malam", nama: "Paviliun Malam", jauh: "kapel-jauh", utama: "paviliun-utama", depan: "kapel-depan", langit: "linear-gradient(#1c2340, #2e3557 55%, #3a3f5c)", cahaya: "radial-gradient(circle at 40% 38%, #fffbe8, #f1e6c2 60%, rgba(241,230,194,0) 71%)" },
];

/// Adegan untuk tema; "" = tanpa panggung (tema sinema: video latar sudah
/// menjadi gerak utamanya).
pub fn pilih(slug: &str, nuansa: &str, wilayah: &str, sinema: bool) -> &'static str {
    if sinema {
        return "";
    }
    match slug {
        "masjid-putih" => return "masjid",
        "pendopo-putih" => return "jawa",
        "gerbang-bulan" => return "tionghoa",
        "kapel-putih" => return "kapel",
        "botanical-heritage" | "curug-asri" | "taman-kupu" | "teratai-mekar" | "titi-mentawai" => return "alam",
        "samudra-biru" => return "laut",
        "langit-awan" => return "taman",
        "fajar-rembulan" | "midnight-celestial" => return "malam",
        "bali-frangipani" | "penjor-pura-bali" | "geringsing-tenganan" => return "bali",
        "phinisi-layar-tujuh" | "raja-ampat" | "lancang-kuning-kepri" | "sandeq-mandar" => return "bahari",
        _ => {}
    }
    let n = nuansa.to_lowercase();
    match n.as_str() {
        "islami" | "aceh" => "masjid",
        "jawa" | "sunda" | "madura" | "betawi" => "jawa",
        "bali" | "hindu" => "bali",
        "tionghoa" | "vietnam" => "tionghoa",
        "kristiani" => "kapel",
        "minang" => "minang",
        "batak" | "toraja" | "nias" => "toraja",
        "dayak" | "banjar" => "dayak",
        "papua" | "mentawai" => "papua",
        "melayu" | "palembang" | "lampung" => "melayu",
        "bugis-makassar" | "mandar" | "buton" | "maluku" | "ntt" | "sasak" | "mbojo" | "sangihe" | "minahasa" | "gorontalo" | "kaili" => "bahari",
        "jepang" => "jepang",
        "korea" => "korea",
        "thailand" => "thai",
        "buddhis" => "buddhis",
        "alam" => "alam",
        _ => {
            let w = wilayah.to_lowercase();
            if w.contains("kepulauan") || w.contains("laut") || w.contains("pesisir") { "bahari" } else { "taman" }
        }
    }
}

/// Ayat / kutipan bawaan per adegan (bila undangan tak punya kutipan / demo).
pub fn kutipan(adegan: &str) -> (&'static str, &'static str) {
    match adegan {
        "masjid" => ("Dan di antara tanda-tanda kebesaran-Nya ialah Dia menciptakan pasangan-pasangan untukmu dari jenismu sendiri, agar kamu cenderung dan merasa tenteram kepadanya.", "QS Ar-Rum : 21"),
        "kapel" => ("Kasih itu sabar; kasih itu murah hati; ia tidak cemburu. Kasih tidak berkesudahan.", "1 Korintus 13 : 4, 8"),
        "bali" => ("Wahai pasangan pengantin, tetaplah bersatu di sini, jangan berpisah; capailah usia panjang bersama anak dan cucu.", "Rgveda X.85.42"),
        "tionghoa" | "korea" | "jepang" => ("Dua hati yang dipertemukan langit, berjalan bersama dalam satu cahaya bulan.", "Restu Leluhur"),
        _ => ("Dua jiwa bersatu dalam ikatan suci, saling menjaga dalam suka dan duka hingga akhir masa.", "Doa Restu"),
    }
}

#[cfg(feature = "ssr")]
pub fn css(adegan: &str) -> &'static str {
    use std::collections::HashMap;
    use std::sync::OnceLock;
    static CSS: OnceLock<HashMap<&'static str, String>> = OnceLock::new();
    CSS.get_or_init(|| ADEGAN.iter().map(|a| (a.slug, rakit(a))).collect()).get(adegan).map(String::as_str).unwrap_or("")
}

#[cfg(feature = "ssr")]
fn svg(nama: &str) -> &'static str {
    match nama {
        "fuji-jauh" => include_str!("panggung/svg/fuji-jauh.svg"),
        "gadang-utama" => include_str!("panggung/svg/gadang-utama.svg"),
        "gapura-depan" => include_str!("panggung/svg/gapura-depan.svg"),
        "honai-utama" => include_str!("panggung/svg/honai-utama.svg"),
        "hutan-jauh" => include_str!("panggung/svg/hutan-jauh.svg"),
        "jawa-depan" => include_str!("panggung/svg/jawa-depan.svg"),
        "jawa-jauh" => include_str!("panggung/svg/jawa-jauh.svg"),
        "jawa-utama" => include_str!("panggung/svg/jawa-utama.svg"),
        "kapel-depan" => include_str!("panggung/svg/kapel-depan.svg"),
        "kapel-jauh" => include_str!("panggung/svg/kapel-jauh.svg"),
        "kapel-utama" => include_str!("panggung/svg/kapel-utama.svg"),
        "kuil-utama" => include_str!("panggung/svg/kuil-utama.svg"),
        "lamin-utama" => include_str!("panggung/svg/lamin-utama.svg"),
        "laut-jauh" => include_str!("panggung/svg/laut-jauh.svg"),
        "masjid-depan" => include_str!("panggung/svg/masjid-depan.svg"),
        "masjid-jauh" => include_str!("panggung/svg/masjid-jauh.svg"),
        "masjid-utama" => include_str!("panggung/svg/masjid-utama.svg"),
        "meru-utama" => include_str!("panggung/svg/meru-utama.svg"),
        "panggung-utama" => include_str!("panggung/svg/panggung-utama.svg"),
        "paviliun-utama" => include_str!("panggung/svg/paviliun-utama.svg"),
        "pohon-depan" => include_str!("panggung/svg/pohon-depan.svg"),
        "stupa-utama" => include_str!("panggung/svg/stupa-utama.svg"),
        "tionghoa-depan" => include_str!("panggung/svg/tionghoa-depan.svg"),
        "tionghoa-jauh" => include_str!("panggung/svg/tionghoa-jauh.svg"),
        "tionghoa-utama" => include_str!("panggung/svg/tionghoa-utama.svg"),
        "tongkonan-utama" => include_str!("panggung/svg/tongkonan-utama.svg"),
        "torii-depan" => include_str!("panggung/svg/torii-depan.svg"),
        "wat-utama" => include_str!("panggung/svg/wat-utama.svg"),
        _ => "",
    }
}

#[cfg(feature = "ssr")]
fn rakit(a: &Adegan) -> String {
    let uri = |n: &str| super::pendamping::data_uri(svg(n));
    let malam = a.slug == "malam";
    format!(
        "{dasar}\n.pg--{s}{{--pg-langit:{l};--pg-bulan:{c};{tinta}}}\n.pg--{s} .pg__jauh{{background-image:{j}}}\n.pg--{s} .pg__utama{{background-image:{u}}}\n.pg--{s} .pg__depan{{background-image:{d}}}\n",
        dasar = include_str!("panggung/_dasar.css"),
        s = a.slug,
        l = a.langit,
        c = a.cahaya,
        tinta = if malam { "--pg-tinta:#f6f0e2}.pg--malam .pg__teks::before{background:radial-gradient(closest-side,rgba(18,22,44,.72),rgba(18,22,44,.35) 55%,transparent)}.pg--malam .pg__teks{text-shadow:0 1px 10px rgba(0,0,0,.6)" } else { "" },
        j = uri(a.jauh),
        u = uri(a.utama),
        d = uri(a.depan),
    )
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    #[test]
    fn semua_adegan_lengkap() {
        for a in ADEGAN {
            for n in [a.jauh, a.utama, a.depan] {
                assert!(!svg(n).is_empty(), "lapisan {n} tak ada");
            }
            let c = css(a.slug);
            assert!(c.contains(&format!(".pg--{} .pg__depan", a.slug)) && c.contains("view-timeline"), "{}", a.slug);
            assert_eq!(c.matches('{').count(), c.matches('}').count(), "kurung: {}", a.slug);
        }
        assert_eq!(pilih("x", "Islami", "", false), "masjid");
        assert_eq!(pilih("x", "Minang", "", false), "minang");
        assert_eq!(pilih("x", "Modern", "Evening", false), "taman");
        assert_eq!(pilih("sinema-kenangan", "Modern", "", true), "");
    }

    #[test]
    fn semua_pilihan_punya_adegan() {
        for n in ["islami", "jawa", "bali", "tionghoa", "kristiani", "minang", "batak", "dayak", "papua", "melayu", "maluku", "jepang", "korea", "thailand", "buddhis", "alam", "modern"] {
            let a = pilih("x", n, "", false);
            assert!(ADEGAN.iter().any(|x| x.slug == a), "{n} → {a}");
        }
    }
}
