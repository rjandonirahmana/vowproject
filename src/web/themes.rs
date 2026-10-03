//! web/themes.rs — metode bayar, lagu & kutipan bawaan.
//!
//! Tema ada di tabel `themes` (web/skin.rs); paket, harga, add-on & kupon di
//! `site_content` (web/konten.rs) — keduanya dikelola admin di /admin.

// Paket, add-on, kupon & harga kini di web/konten.rs (bisa disunting admin).

/// Pembayaran HANYA transfer manual ke rekening di Konten `pembayaran`
/// (bawaan ShopeePay), lalu pemesan mengunggah bukti di Kelola.
pub const PAYMENT_METHODS: &[(&str, &str, &str)] = &[("shopeepay", "ShopeePay", "account_balance_wallet")];

// ── Musik bawaan ───────────────────────────────────────────────────────────
// Berkasnya diletakkan di public/music/{slug}.mp3 (royalty-free, bukan bagian
// repo — lihat public/music/README.md). Berkas absen = pemutar diam, tak error.

pub struct Song {
    pub slug: &'static str,
    pub title: &'static str,
    pub artist: &'static str,
    pub duration: &'static str,
    pub tag: &'static str,
    /// URL penuh (mis. RustFS); kosong = berkas lokal /music/{slug}.mp3.
    pub src: &'static str,
}

impl Song {
    pub fn url(&self) -> String {
        if self.src.is_empty() { format!("/music/{}.mp3", self.slug) } else { self.src.to_string() }
    }
}

/// Slug undangan demo (seed 001) — "Coba Demo" & pratinjau bergulir katalog.
pub const DEMO_SLUG: &str = "anindita-raditya";

/// Lagu bawaan SEMUA contoh/demo (undangan demo & halaman demo tema). Undangan
/// demo di DB memakai URL yang sama (migration/010_musik_demo.sql).
pub const DEMO_SONG: Song = Song {
    slug: "teman-hidup",
    title: "Teman Hidup",
    artist: "Tulus",
    duration: "",
    tag: "Lagu Demo",
    src: "https://image.ulalaapi.store/undangan/musik/TULUS-Teman-Hidup.mp3",
};

pub const SONGS: &[Song] = &[
    Song { slug: "kisah-romantis", title: "Kisah Romantis", artist: "Instrumental Piano", duration: "03:42", tag: "Terpopuler", src: "" },
    Song { slug: "canon-in-d", title: "Canon in D – Strings & Harp", artist: "Klasik Royal Romance", duration: "03:15", tag: "", src: "" },
    Song { slug: "gending-sriwijaya", title: "Gending Sriwijaya & Jawa Akustik", artist: "Alunan Gamelan Lembut", duration: "04:10", tag: "Tradisional", src: "" },
    Song { slug: "thousand-years", title: "A Thousand Years – Cello Duet", artist: "Instrumental Pengiring", duration: "03:58", tag: "", src: "" },
    Song { slug: "akad-fingerstyle", title: "Akad – Fingerstyle Guitar", artist: "Gitar Akustik Syahdu", duration: "03:55", tag: "", src: "" },
];

pub fn song(slug: &str) -> Option<&'static Song> {
    SONGS.iter().find(|s| s.slug == slug)
}

pub const QUOTES: &[(&str, &str)] = &[
    (
        "Dan di antara tanda-tanda (kebesaran)-Nya ialah Dia menciptakan pasangan-pasangan untukmu dari jenismu sendiri, agar kamu cenderung dan merasa tenteram kepadanya, dan Dia menjadikan di antaramu rasa kasih dan sayang.",
        "QS. Ar-Rum: 21",
    ),
    (
        "Maha Suci Allah yang telah menciptakan pasangan-pasangan semuanya, baik dari apa yang ditumbuhkan oleh bumi dan dari diri mereka sendiri.",
        "QS. Yasin: 36",
    ),
    (
        "Semoga Allah memberkahimu dalam kebaikan dan menghimpun kalian berdua dalam kebahagiaan.",
        "HR. Abu Dawud",
    ),
    (
        "Cinta itu sabar; cinta itu murah hati; ia tidak cemburu. Ia menutupi segala sesuatu, percaya segala sesuatu, mengharapkan segala sesuatu, sabar menanggung segala sesuatu.",
        "1 Korintus 13:4-7",
    ),
];
