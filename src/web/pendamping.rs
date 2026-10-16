//! web/pendamping.rs — lapisan PENDAMPING 3D undangan: makhluk / benda khas
//! budaya tema yang bergerak di belakang isi undangan (rusa untuk tema alam,
//! naga untuk Tionghoa, matahari terbit untuk Jepang, …).
//!
//! Jenisnya DITURUNKAN dari tema (slug → nuansa → wilayah), bukan kolom DB —
//! semua tema langsung mendapatkannya tanpa migrasi. CSS tiap jenis ada di
//! `pendamping/{jenis}.css` (penanda `{p}` = `.pdp--{jenis}`, `{svg:nama}` =
//! siluet `pendamping/svg/{nama}.svg` sebagai data URI → tanpa aset RustFS)
//! dan hanya ikut di `/gaya/{slug}.css` tema yang memakainya.

/// Jenis pendamping: slug, nama tampilan, rentang anak ke-n (1-based,
/// inklusif) yang berupa BURUNG (diberi kelas `sy` → badan + sayap 3D).
pub struct Jenis {
    pub slug: &'static str,
    pub nama: &'static str,
    burung: (u8, u8),
}

pub const JENIS: &[Jenis] = &[
    Jenis { slug: "rusa", nama: "Rusa di bukit", burung: (0, 0) },
    Jenis { slug: "naga", nama: "Naga emas meliuk", burung: (0, 0) },
    Jenis { slug: "matahari-terbit", nama: "Matahari terbit & Fuji", burung: (7, 9) },
    Jenis { slug: "bangau", nama: "Kawanan bangau", burung: (2, 6) },
    Jenis { slug: "gajah", nama: "Gajah & lentera langit", burung: (0, 0) },
    Jenis { slug: "lampion", nama: "Untai lampion sutra", burung: (0, 0) },
    Jenis { slug: "bulan-bintang", nama: "Bulan sabit & fanous", burung: (0, 0) },
    Jenis { slug: "merpati", nama: "Merpati & cahaya", burung: (2, 6) },
    Jenis { slug: "teratai", nama: "Teratai & pelita", burung: (0, 0) },
    Jenis { slug: "enggang", nama: "Burung enggang", burung: (1, 2) },
    Jenis { slug: "cendrawasih", nama: "Cendrawasih", burung: (0, 0) },
    Jenis { slug: "lumba", nama: "Lumba-lumba & ombak", burung: (0, 0) },
    Jenis { slug: "merak", nama: "Merak mekar", burung: (0, 0) },
    Jenis { slug: "kupu", nama: "Kupu-kupu emas", burung: (0, 0) },
    Jenis { slug: "rasi", nama: "Langit berbintang", burung: (0, 0) },
];

/// Jumlah elemen `<i>` lapisan (jenis memakai sebagian).
pub const BUTIR: u8 = 16;

/// Anak ke-n (1-based) jenis ini adalah burung bersayap 3D.
pub fn burung(jenis: &str, n: u8) -> bool {
    JENIS.iter().find(|j| j.slug == jenis).is_some_and(|j| n >= j.burung.0 && n <= j.burung.1)
}

/// Pendamping untuk tema; "" = tanpa pendamping (tema sinema: video latar
/// sudah menjadi gerak utamanya).
pub fn pilih(slug: &str, nuansa: &str, wilayah: &str, sinema: bool) -> &'static str {
    if sinema {
        return "";
    }
    match slug {
        "botanical-heritage" | "curug-asri" => return "rusa",
        "taman-kupu" => return "kupu",
        "teratai-mekar" | "bali-frangipani" => return "teratai",
        "samudra-biru" | "phinisi-layar-tujuh" | "raja-ampat" | "lancang-kuning-kepri" => return "lumba",
        "langit-awan" => return "bangau",
        // Tema malam: rasi bintang (bukan kupu-kupu bawaan nuansa "Modern").
        "fajar-rembulan" | "midnight-celestial" | "danau-rembulan-3d" => return "rasi",
        // Tema 3D gulir-beda: lentera (salju), kupu (lavender), rasi senja (gurun).
        "gurun-senja-3d" => return "rasi",
        "salju-lentera-3d" => return "lampion",
        "ladang-lavender-3d" => return "kupu",
        _ => {}
    }
    let n = nuansa.to_lowercase();
    let w = wilayah.to_lowercase();
    match n.as_str() {
        "tionghoa" => "naga",
        "jepang" => "matahari-terbit",
        "korea" => "bangau",
        "thailand" => "gajah",
        "vietnam" => "lampion",
        "islami" | "aceh" => "bulan-bintang",
        "kristiani" => "merpati",
        "hindu" | "buddhis" | "bali" => "teratai",
        "dayak" => "enggang",
        "papua" => "cendrawasih",
        "maluku" | "mandar" | "sangihe" | "buton" => "lumba",
        "jawa" | "sunda" | "madura" | "betawi" => "merak",
        "alam" => "rusa",
        _ if w.contains("kepulauan") || w.contains("laut") || w.contains("pesisir") => "lumba",
        _ => "kupu",
    }
}

/// CSS satu jenis (dasar lapisan + gerak jenis itu); "" bila tak dikenal.
#[cfg(feature = "ssr")]
pub fn css(jenis: &str) -> &'static str {
    use std::collections::HashMap;
    use std::sync::OnceLock;
    static CSS: OnceLock<HashMap<&'static str, String>> = OnceLock::new();
    CSS.get_or_init(|| {
        JENIS
            .iter()
            .filter_map(|j| sumber(j.slug).map(|src| (j.slug, rakit(j.slug, src))))
            .collect()
    })
    .get(jenis)
    .map(String::as_str)
    .unwrap_or("")
}

#[cfg(feature = "ssr")]
fn sumber(jenis: &str) -> Option<&'static str> {
    Some(match jenis {
        "rusa" => include_str!("pendamping/rusa.css"),
        "naga" => include_str!("pendamping/naga.css"),
        "matahari-terbit" => include_str!("pendamping/matahari-terbit.css"),
        "bangau" => include_str!("pendamping/bangau.css"),
        "gajah" => include_str!("pendamping/gajah.css"),
        "lampion" => include_str!("pendamping/lampion.css"),
        "bulan-bintang" => include_str!("pendamping/bulan-bintang.css"),
        "merpati" => include_str!("pendamping/merpati.css"),
        "teratai" => include_str!("pendamping/teratai.css"),
        "enggang" => include_str!("pendamping/enggang.css"),
        "cendrawasih" => include_str!("pendamping/cendrawasih.css"),
        "lumba" => include_str!("pendamping/lumba.css"),
        "merak" => include_str!("pendamping/merak.css"),
        "kupu" => include_str!("pendamping/kupu.css"),
        "rasi" => include_str!("pendamping/rasi.css"),
        _ => return None,
    })
}

#[cfg(feature = "ssr")]
fn svg(nama: &str) -> Option<&'static str> {
    Some(match nama {
        "rusa" => include_str!("pendamping/svg/rusa.svg"),
        "rusa-lompat" => include_str!("pendamping/svg/rusa-lompat.svg"),
        "naga-kepala" => include_str!("pendamping/svg/naga-kepala.svg"),
        "naga-badan" => include_str!("pendamping/svg/naga-badan.svg"),
        "naga-ekor" => include_str!("pendamping/svg/naga-ekor.svg"),
        "naga-kaki" => include_str!("pendamping/svg/naga-kaki.svg"),
        "fuji" => include_str!("pendamping/svg/fuji.svg"),
        "fuji-salju" => include_str!("pendamping/svg/fuji-salju.svg"),
        "bangau" => include_str!("pendamping/svg/bangau.svg"),
        "sayap" => include_str!("pendamping/svg/sayap.svg"),
        "merpati" => include_str!("pendamping/svg/merpati.svg"),
        "enggang" => include_str!("pendamping/svg/enggang.svg"),
        "cendrawasih" => include_str!("pendamping/svg/cendrawasih.svg"),
        "lumba" => include_str!("pendamping/svg/lumba.svg"),
        "gajah" => include_str!("pendamping/svg/gajah.svg"),
        "teratai" => include_str!("pendamping/svg/teratai.svg"),
        "kupu-sayap" => include_str!("pendamping/svg/kupu-sayap.svg"),
        "merak-badan" => include_str!("pendamping/svg/merak-badan.svg"),
        "bulu-merak" => include_str!("pendamping/svg/bulu-merak.svg"),
        "sabit" => include_str!("pendamping/svg/sabit.svg"),
        "fanous" => include_str!("pendamping/svg/fanous.svg"),
        _ => return None,
    })
}

/// Ganti `{p}` & `{svg:nama}`; SVG dijadikan data URI ber-persen-encode.
#[cfg(feature = "ssr")]
fn rakit(jenis: &str, src: &str) -> String {
    let dasar = include_str!("pendamping/_dasar.css");
    let mut out = String::with_capacity(dasar.len() + src.len() + 4096);
    let full = format!("{dasar}\n{}", src.replace("{p}", &format!(".pdp--{jenis}")));
    let mut rest = full.as_str();
    while let Some(i) = rest.find("{svg:") {
        out.push_str(&rest[..i]);
        let after = &rest[i + 5..];
        let Some(j) = after.find('}') else { break };
        match svg(&after[..j]) {
            Some(s) => out.push_str(&data_uri(s)),
            None => out.push_str("none"),
        }
        rest = &after[j + 1..];
    }
    out.push_str(rest);
    out
}

#[cfg(feature = "ssr")]
pub(crate) fn data_uri(svg: &str) -> String {
    let mut s = String::from("url(\"data:image/svg+xml,");
    for c in svg.trim().chars() {
        match c {
            '"' => s.push('\''),
            '%' => s.push_str("%25"),
            '#' => s.push_str("%23"),
            '<' => s.push_str("%3C"),
            '>' => s.push_str("%3E"),
            '\n' | '\r' => {}
            c => s.push(c),
        }
    }
    s.push_str("\")");
    s
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    #[test]
    fn semua_jenis_lengkap() {
        for j in JENIS {
            let c = css(j.slug);
            assert!(c.contains(&format!(".pdp--{}", j.slug)), "{}", j.slug);
            assert!(!c.contains("{p}") && !c.contains("{svg:"), "penanda tersisa: {}", j.slug);
            assert!(!c.contains("url(none") && !c.contains(": none center"), "svg hilang: {}", j.slug);
            assert_eq!(c.matches('{').count(), c.matches('}').count(), "kurung: {}", j.slug);
            assert!(!c.contains('<'), "'<' mentah: {}", j.slug);
        }
    }

    #[test]
    fn pemetaan_tema() {
        assert_eq!(pilih("naga-emas", "Tionghoa", "Tionghoa", false), "naga");
        assert_eq!(pilih("sakura-kyoto", "Jepang", "Jepang", false), "matahari-terbit");
        assert_eq!(pilih("curug-asri", "Alam", "Air Terjun", false), "rusa");
        assert_eq!(pilih("sinema-kenangan", "Modern", "Video", true), "");
        assert_eq!(pilih("x", "Melayu", "Kepulauan Bangka Belitung", false), "lumba");
        assert_eq!(pilih("x", "Modern", "Evening", false), "kupu");
        // Tema malam danau: rasi bintang, bukan kupu-kupu siang.
        assert_eq!(pilih("danau-rembulan-3d", "Modern", "Danau Rembulan", false), "rasi");
    }
}
