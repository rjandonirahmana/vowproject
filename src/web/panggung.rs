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
    /// Latar jauh yang sesuai budaya (indeks 0 = bawaan).
    jauh: &'static [&'static str],
    utama: &'static str,
    /// Gerbang depan yang sesuai budaya (indeks 0 = bawaan).
    depan: &'static [&'static str],
    langit: &'static str,
    cahaya: &'static str,
}

pub const ADEGAN: &[Adegan] = &[
    Adegan { slug: "masjid", nama: "Masjid Putih", jauh: &["masjid-jauh", "jawa-jauh", "hutan-jauh"], utama: "masjid-utama", depan: &["masjid-depan", "gapura-depan", "pohon-depan"], langit: "linear-gradient(#d3d9e0, #eceff3 55%, #e9ecef)", cahaya: "radial-gradient(circle at 40% 38%, #fff, #eceff2 62%, #d9dee4)" },
    Adegan { slug: "jawa", nama: "Pendopo & Candi Bentar", jauh: &["jawa-jauh", "hutan-jauh", "kapel-jauh"], utama: "jawa-utama", depan: &["jawa-depan", "gapura-depan", "pohon-depan"], langit: "linear-gradient(#efe6d6, #f7f2e8 55%, #efece4)", cahaya: "radial-gradient(circle at 45% 45%, #fffaf0, #fbe8c2 55%, rgba(251,232,194,0) 71%)" },
    Adegan { slug: "bali", nama: "Meru & Candi Bentar", jauh: &["jawa-jauh", "laut-jauh", "hutan-jauh"], utama: "meru-utama", depan: &["jawa-depan", "gapura-depan", "pohon-depan"], langit: "linear-gradient(#efe6d6, #f7f2e8 55%, #efece4)", cahaya: "radial-gradient(circle at 45% 45%, #fffaf0, #fbe8c2 55%, rgba(251,232,194,0) 71%)" },
    Adegan { slug: "tionghoa", nama: "Gerbang Bulan", jauh: &["tionghoa-jauh", "hutan-jauh"], utama: "tionghoa-utama", depan: &["tionghoa-depan", "gapura-depan"], langit: "linear-gradient(#dfe2e6, #f2f1ef 55%, #ecebe8)", cahaya: "radial-gradient(circle at 40% 38%, #fff, #eceff2 62%, #d9dee4)" },
    Adegan { slug: "kapel", nama: "Kapel & Lengkung Bunga", jauh: &["kapel-jauh", "hutan-jauh", "laut-jauh"], utama: "kapel-utama", depan: &["kapel-depan", "pohon-depan", "gapura-depan"], langit: "linear-gradient(#e6e4ea, #f6f3f4 55%, #eeeeec)", cahaya: "radial-gradient(circle at 45% 45%, #fffdf8, #fbeef0 55%, rgba(251,238,240,0) 71%)" },
    Adegan { slug: "minang", nama: "Rumah Gadang", jauh: &["jawa-jauh", "hutan-jauh"], utama: "gadang-utama", depan: &["gapura-depan", "pohon-depan"], langit: "linear-gradient(#efe6d6, #f7f2e8 55%, #efece4)", cahaya: "radial-gradient(circle at 45% 45%, #fffaf0, #fbe8c2 55%, rgba(251,232,194,0) 71%)" },
    Adegan { slug: "toraja", nama: "Tongkonan & Rumah Bolon", jauh: &["jawa-jauh", "hutan-jauh"], utama: "tongkonan-utama", depan: &["gapura-depan", "pohon-depan"], langit: "linear-gradient(#efe6d6, #f7f2e8 55%, #efece4)", cahaya: "radial-gradient(circle at 45% 45%, #fffaf0, #fbe8c2 55%, rgba(251,232,194,0) 71%)" },
    Adegan { slug: "melayu", nama: "Rumah Panggung Melayu", jauh: &["jawa-jauh", "laut-jauh", "hutan-jauh"], utama: "panggung-utama", depan: &["gapura-depan", "pohon-depan", "kapel-depan"], langit: "linear-gradient(#f0e2c4, #f8f1e2 55%, #f1ece0)", cahaya: "radial-gradient(circle at 45% 45%, #fffaf0, #fbe8c2 55%, rgba(251,232,194,0) 71%)" },
    Adegan { slug: "bahari", nama: "Rumah Panggung Pesisir", jauh: &["laut-jauh", "jawa-jauh"], utama: "panggung-utama", depan: &["gapura-depan", "pohon-depan", "kapel-depan"], langit: "linear-gradient(#cfdde6, #eef3f6 55%, #e9eef0)", cahaya: "radial-gradient(circle at 45% 45%, #fffaf0, #fbe8c2 55%, rgba(251,232,194,0) 71%)" },
    Adegan { slug: "dayak", nama: "Rumah Lamin", jauh: &["hutan-jauh", "jawa-jauh"], utama: "lamin-utama", depan: &["gapura-depan", "pohon-depan"], langit: "linear-gradient(#dbe4d8, #f1f4ee 55%, #ebeee8)", cahaya: "radial-gradient(circle at 45% 45%, #fffaf0, #fbe8c2 55%, rgba(251,232,194,0) 71%)" },
    Adegan { slug: "papua", nama: "Honai Lembah", jauh: &["hutan-jauh", "jawa-jauh"], utama: "honai-utama", depan: &["pohon-depan", "gapura-depan"], langit: "linear-gradient(#dbe4d8, #f1f4ee 55%, #ebeee8)", cahaya: "radial-gradient(circle at 45% 45%, #fffaf0, #fbe8c2 55%, rgba(251,232,194,0) 71%)" },
    Adegan { slug: "jepang", nama: "Kuil & Torii", jauh: &["fuji-jauh", "hutan-jauh"], utama: "kuil-utama", depan: &["torii-depan", "gapura-depan", "pohon-depan"], langit: "linear-gradient(#e6e4ea, #f6f3f4 55%, #eeeeec)", cahaya: "radial-gradient(circle at 45% 45%, #fffdf8, #fbeef0 55%, rgba(251,238,240,0) 71%)" },
    Adegan { slug: "korea", nama: "Istana Hanok", jauh: &["hutan-jauh", "tionghoa-jauh"], utama: "tionghoa-utama", depan: &["gapura-depan", "pohon-depan"], langit: "linear-gradient(#dfe2e6, #f2f1ef 55%, #ecebe8)", cahaya: "radial-gradient(circle at 40% 38%, #fff, #eceff2 62%, #d9dee4)" },
    Adegan { slug: "thai", nama: "Wat Emas", jauh: &["jawa-jauh", "hutan-jauh"], utama: "wat-utama", depan: &["gapura-depan", "pohon-depan"], langit: "linear-gradient(#f0e2c4, #f8f1e2 55%, #f1ece0)", cahaya: "radial-gradient(circle at 45% 45%, #fffaf0, #fbe8c2 55%, rgba(251,232,194,0) 71%)" },
    Adegan { slug: "buddhis", nama: "Stupa", jauh: &["hutan-jauh", "jawa-jauh"], utama: "stupa-utama", depan: &["gapura-depan", "pohon-depan"], langit: "linear-gradient(#f0e2c4, #f8f1e2 55%, #f1ece0)", cahaya: "radial-gradient(circle at 45% 45%, #fffaf0, #fbe8c2 55%, rgba(251,232,194,0) 71%)" },
    Adegan { slug: "alam", nama: "Paviliun Hutan", jauh: &["hutan-jauh", "jawa-jauh", "laut-jauh"], utama: "paviliun-utama", depan: &["pohon-depan", "kapel-depan", "gapura-depan"], langit: "linear-gradient(#dbe4d8, #f1f4ee 55%, #ebeee8)", cahaya: "radial-gradient(circle at 45% 45%, #fffaf0, #fbe8c2 55%, rgba(251,232,194,0) 71%)" },
    Adegan { slug: "laut", nama: "Paviliun Pantai", jauh: &["laut-jauh", "kapel-jauh"], utama: "paviliun-utama", depan: &["kapel-depan", "pohon-depan"], langit: "linear-gradient(#cfdde6, #eef3f6 55%, #e9eef0)", cahaya: "radial-gradient(circle at 45% 45%, #fffaf0, #fbe8c2 55%, rgba(251,232,194,0) 71%)" },
    Adegan { slug: "taman", nama: "Paviliun Taman", jauh: &["kapel-jauh", "hutan-jauh", "laut-jauh"], utama: "paviliun-utama", depan: &["kapel-depan", "pohon-depan", "gapura-depan"], langit: "linear-gradient(#d3d9e0, #eceff3 55%, #e9ecef)", cahaya: "radial-gradient(circle at 40% 38%, #fff, #eceff2 62%, #d9dee4)" },
    Adegan { slug: "malam", nama: "Paviliun Malam", jauh: &["kapel-jauh", "hutan-jauh"], utama: "paviliun-utama", depan: &["kapel-depan", "pohon-depan"], langit: "linear-gradient(#1c2340, #2e3557 55%, #3a3f5c)", cahaya: "radial-gradient(circle at 40% 38%, #fffbe8, #f1e6c2 60%, rgba(241,230,194,0) 71%)" },
];

/// Adegan untuk tema — SEMUA tema mendapat panggung. Tema berbudaya kuat
/// memakai bangunan budayanya; tema umum/modern dipilih menurut WARNA tema
/// (`palet` = kunci skin::PALETTES, `gelap` = tema gelap) agar panggung
/// selaras: hijau → hutan, emas → pendopo Jawa, terra → rumah kayu, blush →
/// taman bunga, navy/biru → pantai, gelap/mono → malam. Kapel (bersalib)
/// hanya untuk tema Kristiani.
pub fn pilih(slug: &str, nuansa: &str, wilayah: &str, palet: &str, gelap: bool) -> &'static str {
    match slug {
        "masjid-putih" => return "masjid",
        "pendopo-putih" | "kusuma-jawi" | "sekar-kedhaton" => return "jawa",
        "gerbang-bulan" => return "tionghoa",
        "kapel-putih" => return "kapel",
        "botanical-heritage" | "curug-asri" | "taman-kupu" | "teratai-mekar" | "titi-mentawai" => return "alam",
        "samudra-biru" => return "laut",
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
        _ => {
            let w = wilayah.to_lowercase();
            if w.contains("kepulauan") || w.contains("laut") || w.contains("pesisir") || w.contains("pantai") {
                return "bahari";
            }
            if gelap {
                return "malam";
            }
            match palet {
                "sage" => "alam",
                "gold" => "jawa",
                "terra" => "melayu",
                "blush" => "taman",
                "navy" => "laut",
                "mono" => "malam",
                _ if n == "alam" => "alam",
                _ => "taman",
            }
        }
    }
}

/// Resep panggung UNIK per tema: adegan budaya + GERBANG yang ditembus +
/// LATAR jauh (keduanya dari daftar yang sesuai budaya adegan) + gerak
/// kamera (6) + cermin. Ditetapkan berurutan per kelompok adegan
/// ([`tetapkan`]) → tak ada dua tema dengan resep yang sama.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Resep {
    pub adegan: &'static str,
    pub kamera: u8,
    pub cermin: bool,
    /// Indeks `Adegan::depan` / `Adegan::jauh` (0 = bawaan).
    pub depan: u8,
    pub jauh: u8,
}

const KAMERA: usize = 6;

impl Resep {
    /// Kelas akar `<section>` panggung.
    pub fn kelas(&self) -> String {
        format!("pg pg--{} pg-k{}{}", self.adegan, self.kamera, if self.cermin { " pg-c" } else { "" })
    }
}

fn adegan(slug: &str) -> Option<&'static Adegan> {
    ADEGAN.iter().find(|a| a.slug == slug)
}

/// Tetapkan resep untuk SEMUA tema: dikelompokkan per adegan, diurutkan per
/// slug; tema ke-i dalam kelompok mendapat kombinasi ke-i. Urutan putaran:
/// kamera tercepat, lalu GERBANG, LATAR, terakhir cermin — tema sebudaya yang
/// berdekatan sudah berbeda gerak & gerbangnya.
pub fn tetapkan<'a>(tema: impl IntoIterator<Item = (&'a str, &'static str)>) -> std::collections::HashMap<String, Resep> {
    let mut grup: std::collections::BTreeMap<&'static str, Vec<&'a str>> = Default::default();
    for (slug, a) in tema {
        if !a.is_empty() {
            grup.entry(a).or_default().push(slug);
        }
    }
    let mut out = std::collections::HashMap::new();
    for (a, mut slugs) in grup {
        let Some(ad) = adegan(a) else { continue };
        let (nd, nj) = (ad.depan.len().max(1), ad.jauh.len().max(1));
        slugs.sort_unstable();
        slugs.dedup();
        for (i, s) in slugs.into_iter().enumerate() {
            let kamera = i % KAMERA;
            let depan = (i / KAMERA) % nd;
            let jauh = (i / (KAMERA * nd)) % nj;
            let cermin = (i / (KAMERA * nd * nj)) % 2 == 1;
            out.insert(s.to_string(), Resep { adegan: a, kamera: kamera as u8, cermin, depan: depan as u8, jauh: jauh as u8 });
        }
    }
    out
}

/// Ayat / kutipan bawaan per adegan (bila undangan tak punya kutipan / demo).
pub fn kutipan(adegan: &str) -> (&'static str, &'static str) {
    match adegan {
        "masjid" => ("Dan di antara tanda-tanda kebesaran-Nya ialah Dia menciptakan pasangan-pasangan untukmu dari jenismu sendiri, agar kamu cenderung dan merasa tenteram kepadanya.", "QS Ar-Rum : 21"),
        "kapel" => ("Kasih itu sabar; kasih itu murah hati; ia tidak cemburu. Kasih tidak berkesudahan.", "1 Korintus 13 : 4, 8"),
        "bali" => ("Wahai pasangan pengantin, tetaplah bersatu di sini, jangan berpisah; capailah usia panjang bersama anak dan cucu.", "Rgveda X.85.42"),
        "tionghoa" | "korea" | "jepang" => ("Dua hati yang dipertemukan langit, berjalan bersama dalam satu cahaya bulan.", "Doa Restu"),
        _ => ("Dua jiwa bersatu dalam ikatan suci, saling menjaga dalam suka dan duka hingga akhir masa.", "Doa Restu"),
    }
}

/// Markup panggung untuk mesin TEMPLAT (server/templat.rs) — struktur sama
/// dengan komponen Leptos `Panggung3d` (pages/undangan.rs).
#[cfg(feature = "ssr")]
pub fn html(r: &Resep, inv: &super::model::Invitation, l: super::i18n::Lang) -> String {
    let adegan = r.adegan;
    use super::fmt::html_escape as e;
    let (ayat, sumber) = if inv.is_demo || inv.quote_text.trim().is_empty() {
        let (a, s) = kutipan(adegan);
        (a.to_string(), s.to_string())
    } else {
        (inv.quote_text.clone(), inv.quote_source.clone())
    };
    let foto = [inv.cover_photo.as_str(), inv.gallery.first().map(String::as_str).unwrap_or(""), inv.bride_photo.as_str()]
        .into_iter()
        .map(|u| u.split('#').next().unwrap_or(""))
        .find(|u| !u.is_empty())
        .unwrap_or("");
    let pasangan = if foto.is_empty() {
        format!(r#"<div class="monogram monogram--ringan" role="img" aria-label="Monogram {i}"><b>{i}</b></div>"#, i = e(&inv.initials()))
    } else {
        format!(r#"<img src="{}" alt="{}" loading="lazy" decoding="async">"#, e(foto), e(&inv.couple()))
    };
    let acara: String = inv
        .events
        .iter()
        .take(2)
        .map(|ev| {
            let judul = if ev.title.is_empty() { ev.badge_in(l) } else { ev.title.clone() };
            format!(r#"<div class="pg__acara"><h3>{}</h3><p>{}<br>{}<br>{}</p></div>"#, e(&judul), e(&ev.date_label_in(l)), e(&ev.time_label_in(l)), e(&ev.venue))
        })
        .collect();
    format!(
        r#"<section class="{kelas}" aria-label="{label}"><div class="pg__layar"><i class="pg__langit" aria-hidden="true"></i><i class="pg__bulan" aria-hidden="true"></i><i class="pg__lapis pg__jauh" aria-hidden="true"></i><i class="pg__lapis pg__utama" aria-hidden="true"></i><div class="pg__pasangan">{pasangan}</div><i class="pg__lapis pg__depan" aria-hidden="true"></i><div class="pg__teks pg__t1"><p class="script">The Wedding of</p><h2>{b} &amp; {g}</h2><small>{tgl}</small></div><div class="pg__teks pg__t2"><b>{sumber}</b><p>{ayat}</p></div><div class="pg__teks pg__t3"><div><b>{bn}</b><small>{bp}</small></div><div><b>{gn}</b><small>{gp}</small></div></div><div class="pg__teks pg__t4">{acara}</div><div class="pg__teks pg__t5"><p>{salam}</p><p class="script">{b} &amp; {g}</p></div><p class="pg__teks pg__gulir" aria-hidden="true">{gulir}</p></div></section>"#,
        kelas = r.kelas(),
        label = crate::tx!(l, "Pembuka undangan", "Invitation prologue"),
        b = e(&inv.bride_first()),
        g = e(&inv.groom_first()),
        tgl = e(&inv.date_label_in(l)),
        sumber = e(&sumber),
        ayat = e(&ayat),
        bn = e(&inv.bride_name),
        bp = e(&inv.bride_parents),
        gn = e(&inv.groom_name),
        gp = e(&inv.groom_parents),
        salam = crate::tx!(l, "Merupakan kehormatan bagi kami apabila Bapak/Ibu/Saudara/i berkenan hadir memberikan doa restu.", "It would be our honour to have you with us and receive your blessing."),
        gulir = crate::tx!(l, "Gulir perlahan", "Scroll slowly"),
    )
}

/// CSS panggung untuk SATU tema: adegan + gerbang & latar pilihan resep.
#[cfg(feature = "ssr")]
pub fn css_tema(r: &Resep) -> String {
    let mut c = css(r.adegan).to_string();
    if let Some(ad) = adegan(r.adegan) {
        if let Some(n) = ad.depan.get(r.depan as usize).filter(|_| r.depan > 0) {
            c.push_str(&format!(".pg.pg--{} .pg__depan{{background-image:{}}}\n", r.adegan, super::pendamping::data_uri(svg(n))));
        }
        if let Some(n) = ad.jauh.get(r.jauh as usize).filter(|_| r.jauh > 0) {
            c.push_str(&format!(".pg.pg--{} .pg__jauh{{background-image:{}}}\n", r.adegan, super::pendamping::data_uri(svg(n))));
        }
    }
    c
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
    // Langit & cahaya DITURUNKAN dari warna tema (kecuali malam): panggung
    // selalu senada dengan undangan — tema emas berlangit keemasan, hijau
    // kehijauan, cokelat hangat. Lapisan multiply (_dasar.css) mewarnai bangunan.
    let langit = if malam { a.langit } else { "linear-gradient(180deg, color-mix(in srgb, var(--primary) 34%, var(--bg, #f5f3ee)), color-mix(in srgb, var(--bg, #f5f3ee) 88%, #fff) 56%, color-mix(in srgb, var(--gold) 26%, var(--bg, #f5f3ee)))" };
    let cahaya = if malam { a.cahaya } else { "radial-gradient(circle at 45% 42%, #fffdf6, color-mix(in srgb, var(--gold) 45%, #fff) 52%, color-mix(in srgb, var(--gold) 30%, transparent) 64%, transparent 71%)" };
    format!(
        "{dasar}\n.pg--{s}{{--pg-langit:{l};--pg-bulan:{c};{tinta}}}\n.pg--{s} .pg__jauh{{background-image:{j}}}\n.pg--{s} .pg__utama{{background-image:{u}}}\n.pg--{s} .pg__depan{{background-image:{d}}}\n",
        dasar = include_str!("panggung/_dasar.css"),
        s = a.slug,
        l = langit,
        c = cahaya,
        tinta = if malam { "--pg-tinta:#f6f0e2}.pg--malam .pg__teks::before{background:radial-gradient(closest-side,rgba(18,22,44,.72),rgba(18,22,44,.35) 55%,transparent)}.pg--malam .pg__teks{text-shadow:0 1px 10px rgba(0,0,0,.6)" } else { "" },
        j = uri(a.jauh[0]),
        u = uri(a.utama),
        d = uri(a.depan[0]),
    )
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    #[test]
    fn semua_adegan_lengkap() {
        for a in ADEGAN {
            for n in a.jauh.iter().chain(a.depan.iter()).chain([&a.utama]) {
                assert!(!svg(n).is_empty(), "lapisan {n} tak ada");
            }
            let c = css(a.slug);
            assert!(c.contains(&format!(".pg--{} .pg__depan", a.slug)) && c.contains("view-timeline"), "{}", a.slug);
            assert_eq!(c.matches('{').count(), c.matches('}').count(), "kurung: {}", a.slug);
        }
        assert_eq!(pilih("x", "Islami", "", "sage", false), "masjid");
        assert_eq!(pilih("x", "Minang", "", "terra", false), "minang");
        assert_eq!(pilih("warkah-emas", "Modern", "Surat Bersegel", "gold", false), "jawa");
        assert_eq!(pilih("x", "Modern", "", "sage", false), "alam");
        assert_eq!(pilih("x", "Modern", "", "blush", false), "taman");
        assert_eq!(pilih("x", "Modern", "", "gold", true), "malam");
    }

    #[test]
    fn resep_unik() {
        // 40 tema per adegan (kapasitas terkecil: 6 kamera × 2 gerbang × 2 latar × 2 cermin = 48).
        let slug: Vec<String> = (0..160).map(|i| format!("t{i:03}")).collect();
        let tema: Vec<(&str, &'static str)> = slug.iter().enumerate().map(|(i, s)| (s.as_str(), ADEGAN[i % 4].slug)).collect();
        let r = tetapkan(tema);
        let unik: std::collections::HashSet<Resep> = r.values().copied().collect();
        assert_eq!(unik.len(), 160, "resep kembar");
        for (_, rr) in r {
            let c = css_tema(&rr);
            assert_eq!(c.matches('{').count(), c.matches('}').count());
        }
    }

    #[test]
    fn semua_pilihan_punya_adegan() {
        for n in ["islami", "jawa", "bali", "tionghoa", "kristiani", "minang", "batak", "dayak", "papua", "melayu", "maluku", "jepang", "korea", "thailand", "buddhis", "alam", "modern"] {
            let a = pilih("x", n, "", "gold", false);
            assert!(ADEGAN.iter().any(|x| x.slug == a), "{n} → {a}");
        }
    }
}
