//! web/themes.rs — metode bayar, lagu & kutipan bawaan.
//!
//! Tema ada di tabel `themes` (web/skin.rs); paket, harga, add-on & kupon di
//! `site_content` (web/konten.rs) — keduanya dikelola admin di /admin.

// Paket, add-on, kupon & harga kini di web/konten.rs (bisa disunting admin).

/// Pembayaran HANYA transfer manual ke rekening di Konten `pembayaran`
/// (bawaan ShopeePay), lalu pemesan mengunggah bukti di Kelola.
pub const PAYMENT_METHODS: &[(&str, &str, &str)] = &[("shopeepay", "ShopeePay", "account_balance_wallet")];

/// Slug undangan demo (seed 001) — "Coba Demo" & pratinjau bergulir katalog.
pub const DEMO_SLUG: &str = "yona-doni";
/// Slug demo lama (sebelum Okt 2026) — dialihkan 301 ke DEMO_SLUG.
pub const OLD_DEMO_SLUG: &str = "anindita-raditya";

// Musik latar kini dari pustaka admin (tabel `songs`, migrasi 027) — lagu
// bawaan lama di kode tak pernah punya berkas audio.

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
