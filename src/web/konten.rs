//! web/konten.rs — konten situs yang bisa disunting admin di /admin/konten:
//! harga & paket undangan digital, paket cetak/dekorasi/MUA, sewa seserahan,
//! ongkir, galeri, venue, testimoni, profil perias, dan foto hero.
//!
//! Satu `Konten` = seluruh isi; tiap FIELD-nya adalah satu "bagian" yang
//! disimpan sebagai satu baris JSONB di tabel `site_content` (kunci = nama
//! field). Bagian yang belum pernah disunting memakai isi bawaan di bawah.
//! Server menyimpan hasil gabungan di memori (AppState) dan menghitung ulang
//! harga dari sini — angka dari browser tidak dipercaya.
//!
//! Editor admin generik: tiap bagian punya skema `Field` (lihat `SECTIONS`),
//! jadi menambah kolom cukup tambah field struct + baris skema.

use serde::{Deserialize, Serialize};

// ── Tipe ───────────────────────────────────────────────────────────────────

/// Paket undangan digital.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Paket {
    pub slug: String,
    pub name: String,
    pub price: i64,
    pub period: String,
    pub features: Vec<String>,
    pub popular: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Addon {
    pub slug: String,
    pub name: String,
    pub desc: String,
    pub price: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Kupon {
    pub code: String,
    pub amount: i64,
    pub active: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CetakPaket {
    pub slug: String,
    pub name: String,
    pub short: String,
    pub tier: String,
    pub desc: String,
    pub price: i64,
    pub min_qty: i64,
    /// Berat satu set (undangan + amplop + plastik), gram.
    pub gram: i64,
    pub img: String,
    pub popular: bool,
    pub features: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Finishing {
    pub slug: String,
    pub name: String,
    pub price: i64,
    pub gram: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Wilayah {
    pub slug: String,
    pub name: String,
    pub kurir: String,
    pub per_kg: i64,
}

/// Paket dekorasi / MUA. `features` per baris: "ikon | teks" atau "teks".
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VendorPaket {
    pub slug: String,
    pub name: String,
    pub tag: String,
    pub desc: String,
    pub price: i64,
    pub price_from: bool,
    pub img: String,
    pub badge: String,
    pub note: String,
    pub cta: String,
    pub dark: bool,
    pub features: Vec<String>,
    /// Deskripsi lengkap di halaman detail (paragraf dipisah baris kosong).
    pub detail: String,
    /// Foto galeri halaman detail (/dekorasi/{slug}); diunggah dari admin.
    pub gallery: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Venue {
    pub img: String,
    pub area: String,
    pub name: String,
    pub desc: String,
    /// "ikon | teks"
    pub spec: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Foto {
    pub img: String,
    pub kecil: String,
    pub judul: String,
    pub teks: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Testimoni {
    pub text: String,
    pub who: String,
    pub meta: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MuaProfil {
    pub name: String,
    pub role: String,
    pub bio: String,
    pub img: String,
    pub badge: String,
    pub quote: String,
    /// Per baris: "ikon | judul | keterangan"
    pub prestasi: Vec<String>,
}

/// Paket sewa seserahan: satu set wadah hantaran (kotak akrilik, kayu, rotan…).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SeserahanPaket {
    pub slug: String,
    pub name: String,
    /// Bahan / gaya, tampil kecil di atas judul.
    pub tag: String,
    pub desc: String,
    /// Harga sewa satu set selama masa sewa.
    pub price: i64,
    /// Jumlah kotak/wadah dalam satu set — pengali opsi "per kotak".
    pub kotak: i64,
    /// Jaminan (dikembalikan setelah wadah kembali utuh).
    pub deposit: i64,
    pub img: String,
    pub badge: String,
    pub popular: bool,
    pub features: Vec<String>,
}

/// Layanan tambahan seserahan. `per_kotak` = harga × jumlah kotak paket.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SeserahanOpsi {
    pub slug: String,
    pub name: String,
    pub desc: String,
    pub price: i64,
    pub per_kotak: bool,
}

/// Tarif antar-jemput wadah per area (sekali antar + sekali ambil).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SeserahanAntar {
    pub slug: String,
    pub name: String,
    pub price: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SeserahanInfo {
    pub hero: String,
    /// Mis. "3 hari (H-1 s/d H+1 acara)".
    pub lama_sewa: String,
    pub min_booking: String,
    /// Per baris: "ikon | aturan".
    pub syarat: Vec<String>,
}

/// Rekening tujuan pembayaran undangan digital (satu-satunya cara bayar):
/// tampil di /buat & dashboard Kelola, ikut di pesan WA bukti transfer.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Pembayaran {
    /// Nama dompet / bank, mis. "ShopeePay".
    pub metode: String,
    pub nomor: String,
    pub atas_nama: String,
    /// Catatan kecil di bawah instruksi (opsional).
    pub catatan: String,
}

/// Foto hero & angka kecil per halaman.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Umum {
    pub cetak_hero: String,
    pub dekor_hero: String,
    pub mua_hero: String,
    pub cetak_bonus_qty: i64,
    pub cetak_bonus_label: String,
    pub custom_from: i64,
    pub mua_maks_per_hari: i64,
    pub mua_slot_sisa: i64,
    pub mua_slot_persen: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Konten {
    pub paket: Vec<Paket>,
    pub addon: Vec<Addon>,
    pub kupon: Vec<Kupon>,
    pub cetak_paket: Vec<CetakPaket>,
    pub cetak_finishing: Vec<Finishing>,
    pub cetak_wilayah: Vec<Wilayah>,
    pub cetak_testimoni: Vec<Testimoni>,
    pub dekor_paket: Vec<VendorPaket>,
    pub dekor_venue: Vec<Venue>,
    pub dekor_galeri: Vec<Foto>,
    pub mua_paket: Vec<VendorPaket>,
    pub mua_galeri: Vec<Foto>,
    pub mua_profil: MuaProfil,
    pub mua_testimoni: Vec<Testimoni>,
    pub seserahan_paket: Vec<SeserahanPaket>,
    pub seserahan_opsi: Vec<SeserahanOpsi>,
    pub seserahan_antar: Vec<SeserahanAntar>,
    pub seserahan_galeri: Vec<Foto>,
    pub seserahan_info: SeserahanInfo,
    pub seserahan_testimoni: Vec<Testimoni>,
    pub pembayaran: Pembayaran,
    pub umum: Umum,
}

// ── Pembantu ───────────────────────────────────────────────────────────────

/// "ikon | teks" → (ikon, teks). Ikon tak dikenal (tak ada di subset font) →
/// `default`, supaya tak tampil sebagai tulisan.
pub fn icon_line<'a>(line: &'a str, default: &'static str) -> (&'a str, &'a str) {
    match line.split_once('|') {
        Some((i, t)) => {
            let i = i.trim();
            (if super::icons::ICONS.contains(&i) { i } else { default }, t.trim())
        }
        None => (default, line.trim()),
    }
}

impl Konten {
    pub fn package(&self, slug: &str) -> Option<&Paket> {
        self.paket.iter().find(|p| p.slug == slug)
    }
    /// Paket yang dipakai bila slug tak dikenal: yang "populer", atau pertama.
    pub fn package_or_default(&self, slug: &str) -> Paket {
        self.package(slug)
            .or_else(|| self.paket.iter().find(|p| p.popular))
            .or_else(|| self.paket.first())
            .cloned()
            .unwrap_or_default()
    }
    pub fn package_name(&self, slug: &str) -> String {
        self.package(slug).map(|p| p.name.clone()).unwrap_or_else(|| slug.to_string())
    }
    pub fn min_price(&self) -> i64 {
        self.paket.iter().map(|p| p.price).filter(|p| *p > 0).min().unwrap_or(0)
    }
    pub fn active_coupon(&self) -> Option<&Kupon> {
        self.kupon.iter().find(|k| k.active && !k.code.is_empty())
    }
    /// (subtotal, potongan, total) — dihitung ulang di server, tak percaya klien.
    pub fn calc_total(&self, package_slug: &str, addons: &[String], coupon: &str) -> (i64, i64, i64) {
        let mut sub = self.package_or_default(package_slug).price;
        for a in &self.addon {
            if addons.iter().any(|x| *x == a.slug) {
                sub += a.price;
            }
        }
        let disc = self
            .kupon
            .iter()
            .find(|k| k.active && k.code.eq_ignore_ascii_case(coupon.trim()))
            .map(|k| k.amount)
            .unwrap_or(0)
            .clamp(0, sub);
        (sub, disc, sub - disc)
    }
}

// ── Isi bawaan (contoh dari desain Stitch — ganti lewat /admin/konten) ─────

fn s(x: &str) -> String {
    x.to_string()
}
fn v(xs: &[&str]) -> Vec<String> {
    xs.iter().map(|x| x.to_string()).collect()
}

impl Default for Konten {
    fn default() -> Self {
        Konten {
            paket: vec![
                Paket {
                    slug: s("silver"),
                    name: s("Paket Silver"),
                    price: 79_000,
                    period: s("Aktif 6 Bulan"),
                    features: v(&["Maksimal 250 nama tamu spesial", "Navigasi peta lokasi & countdown", "Masa aktif 6 bulan"]),
                    popular: false,
                },
                Paket {
                    slug: s("gold"),
                    name: s("Paket Gold"),
                    price: 149_000,
                    period: s("Aktif Selamanya"),
                    features: v(&["Tamu undangan tanpa batas", "Amplop digital & QRIS", "QR Code check-in resepsi", "Musik latar pilihan / upload sendiri"]),
                    popular: true,
                },
                Paket {
                    slug: s("platinum"),
                    name: s("Paket Platinum VIP"),
                    price: 249_000,
                    period: s("Aktif Selamanya"),
                    features: v(&["Semua fitur Paket Gold", "Domain pribadi (.wedding)", "Revisi tanpa batas", "Bantuan input data & blast WhatsApp"]),
                    popular: false,
                },
            ],
            addon: vec![
                Addon { slug: s("domain"), name: s("Domain Pribadi (.wedding)"), desc: s("nama-pasangan.wedding"), price: 99_000 },
                Addon { slug: s("wa-blast"), name: s("WhatsApp Broadcast Otomatis"), desc: s("Kuota 500 kontak • anti-banned"), price: 49_000 },
                Addon { slug: s("tema-custom"), name: s("Warna & Font Custom"), desc: s("Admin meracik palet & huruf sesuai konsep acaramu"), price: 25_000 },
            ],
            kupon: vec![Kupon { code: s("HEMATNIKAH40"), amount: 40_000, active: true }],
            cetak_paket: vec![
                CetakPaket {
                    slug: s("softcover"),
                    name: s("Softcover Modern"),
                    short: s("Softcover"),
                    tier: s("Entry Luxury"),
                    desc: s("Art Carton 260/310 gsm pilihan tebal, dilaminasi doff halus atau glossy kristal dengan potongan sudut membulat elegan."),
                    price: 3_500,
                    min_qty: 300,
                    gram: 22,
                    img: s("/img/layanan/cetak-softcover.jpg"),
                    popular: false,
                    features: v(&["Art Carton 260 / 310 gsm super tebal", "Laminasi doff halus anti-sidik jari", "Cetak full color offset Ultra-HD", "Termasuk plastik OPP & label nama"]),
                },
                CetakPaket {
                    slug: s("hardcover"),
                    name: s("Hardcover Heritage"),
                    short: s("Hardcover"),
                    tier: s("Masterpiece Intimate"),
                    desc: s("Board kaku mewah 3mm, sentuhan hotprint foil emas berkilau, sabuk amplop motif botanical, dan tali rami rustic / pita satin."),
                    price: 7_500,
                    min_qty: 300,
                    gram: 50,
                    img: s("/img/layanan/cetak-hardcover.jpg"),
                    popular: true,
                    features: v(&[
                        "Board mewah 3mm, kokoh & mantap",
                        "Hotprint gold foil timbul depan & nama",
                        "Sabuk pocket belt amplop motif botanical",
                        "Aksen tali rami alami atau pita satin",
                        "Gratis amplop eksklusif + kartu souvenir",
                    ]),
                },
                CetakPaket {
                    slug: s("akrilik"),
                    name: s("Single Board & Akrilik"),
                    short: s("Single/Akrilik"),
                    tier: s("Avant-Garde Luxe"),
                    desc: s("Kombinasi board linen Jepang berserat organik atau akrilik bening 2mm dengan cetak UV timbul presisi tinggi."),
                    price: 9_000,
                    min_qty: 200,
                    gram: 55,
                    img: s("/img/layanan/cetak-akrilik.jpg"),
                    popular: false,
                    features: v(&["Akrilik bening 2mm atau kertas linen Jepang", "Cetak tinta UV tahan air & gores", "Amplop vellum transparan premium", "Termasuk real wax seal stamp asli"]),
                },
            ],
            cetak_finishing: vec![
                Finishing { slug: s("wax"), name: s("Real Wax Seal Stamp"), price: 1_200, gram: 4 },
                Finishing { slug: s("foil"), name: s("Double Gold Foil Inner"), price: 800, gram: 0 },
            ],
            cetak_wilayah: vec![
                Wilayah { slug: s("jawa"), name: s("Pulau Jawa & Jabodetabek"), kurir: s("JNE/J&T/Cargo"), per_kg: 12_000 },
                Wilayah { slug: s("sumatera"), name: s("Sumatera – kota utama"), kurir: s("J&T/JNE"), per_kg: 25_000 },
                Wilayah { slug: s("bali-nusra"), name: s("Bali, NTB, NTT"), kurir: s("SiCepat/JNE"), per_kg: 35_000 },
                Wilayah { slug: s("kalsul"), name: s("Kalimantan & Sulawesi"), kurir: s("JNE/Cargo"), per_kg: 42_000 },
                Wilayah { slug: s("maluku-papua"), name: s("Maluku & Papua"), kurir: s("JNE Reguler/Cargo"), per_kg: 65_000 },
            ],
            cetak_testimoni: vec![
                Testimoni {
                    text: s("Awalnya ragu kirim paket hardcover 700 pcs ke Medan via cargo. Pas dibuka, kardusnya tebal berlapis bubble wrap, sudut undangan nggak ada yang penyok. Foil emasnya mewah banget!"),
                    who: s("Nadia & Farhan"),
                    meta: s("Medan, Sumatera Utara • 700 pcs Hardcover Heritage"),
                },
                Testimoni {
                    text: s("Sampai di Balikpapan tepat waktu. Wax seal aslinya bikin keluarga besar takjub, ditambah bonus web undangan digital yang gampang di-share ke grup WA."),
                    who: s("Dimas & Anisa"),
                    meta: s("Balikpapan, Kalimantan Timur • 500 pcs Hardcover"),
                },
                Testimoni {
                    text: s("Kirim jauh ke Papua sampai dalam kondisi aman dan kering meski musim hujan. Kualitas cetak linen dan akriliknya benar-benar prestisius."),
                    who: s("Clarissa & Steven"),
                    meta: s("Jayapura, Papua • 400 pcs Single Board Linen"),
                },
            ],
            dekor_paket: vec![
                VendorPaket {
                    slug: s("nature"),
                    name: s("Paket Tawangmangu Nature & Outdoor Garden"),
                    tag: s("Favorit Pasangan Outdoor"),
                    desc: s("Didesain khusus untuk venue lereng perbukitan dan taman pinus. Memberikan nuansa intim, asri, dan wangi alami dedaunan pegunungan."),
                    price: 14_500_000,
                    price_from: false,
                    img: s("/img/layanan/dekor-nature.jpg"),
                    badge: s("Favorit"),
                    note: s("Durasi acara: 1 hari penuh"),
                    cta: s("Pilih Paket Ini"),
                    dark: false,
                    detail: s("Konsep outdoor garden yang menyatu dengan alam lereng Lawu: pelaminan terbuka berlatar pinus, lorong bunga segar, dan pergola kayu yang dirangkai langsung di lokasi.

Tim kami menyesuaikan tata letak dengan kontur lahan dan arah angin, menyiapkan kanopi cadangan bila turun hujan, serta tata suara weatherproof agar ijab kabul tetap terdengar jernih."),
                    gallery: v(&["/img/layanan/dekor-nature.jpg", "/img/layanan/dekor-galeri-altar.jpg", "/img/layanan/venue-tahura.jpg", "/img/layanan/venue-lawu-park.jpg", "/img/layanan/dekor-galeri-meja.jpg", "/img/layanan/venue-nava.jpg"]),
                    features: v(&[
                        "spa | Pelaminan botanical & rustic",
                        "deck | Pergola kayu mahoni alami",
                        "local_florist | Bunga segar petik lereng Lawu",
                        "volume_up | Sound outdoor 5.000–10.000 W weatherproof",
                        "mic | Wireless mic high-range anti-interferensi",
                        "umbrella | Kabel & soket tahan kabut / hujan",
                    ]),
                },
                VendorPaket {
                    slug: s("jawa"),
                    name: s("Paket Tradisional Jawa Solo Basahan / Joglo"),
                    tag: s("Adat Luhur Klasik"),
                    desc: s("Keagungan tata ruang adat Solo. Memadukan ukiran kayu pusaka dengan tata suara ramah gamelan & vokal tembang macapat."),
                    price: 18_000_000,
                    price_from: false,
                    img: s("/img/layanan/dekor-jawa.jpg"),
                    badge: s(""),
                    note: s("Kapasitas: s/d 1.500 undangan"),
                    cta: s("Pilih Paket Ini"),
                    dark: false,
                    detail: s("Tata ruang adat Solo yang megah: gebyok ukir kayu jati, kembar mayang, janur kuning, dan ronce melati asli yang dirangkai pagi hari sebelum acara.

Cocok untuk prosesi panggih, sungkeman, hingga resepsi di pendopo atau gedung. Tata suara disetel khusus untuk gamelan dan vokal tembang agar tetap lembut namun terdengar sampai barisan belakang."),
                    gallery: v(&["/img/layanan/dekor-jawa.jpg", "/img/layanan/dekor-galeri-gebyok.jpg", "/img/layanan/venue-tjolomadoe.jpg", "/img/layanan/venue-gedung-wanita.jpg", "/img/layanan/dekor-galeri-meja.jpg"]),
                    features: v(&[
                        "architecture | Gebyok ukir kayu jati / modern putih gold",
                        "yard | Ronce melati asli & janur kuning pintu masuk",
                        "graphic_eq | Acoustic & gamelan multi-mic system",
                        "light_mode | Lighting ambience warm PAR LED heritage",
                    ]),
                },
                VendorPaket {
                    slug: s("villa"),
                    name: s("Paket Intimate Villa & Rooftop"),
                    tag: s("Intimate Gathering"),
                    desc: s("Konsep santai, hangat, dan berkelas untuk resepsi privat bersama keluarga inti di villa lereng pegunungan."),
                    price: 9_500_000,
                    price_from: false,
                    img: s("/img/layanan/dekor-villa.jpg"),
                    badge: s(""),
                    note: s("Ideal 30–50 tamu"),
                    cta: s("Konsultasi Paket Villa"),
                    dark: false,
                    detail: s("Resepsi privat yang hangat untuk keluarga inti dan sahabat terdekat: meja panjang banquet, lampu gantung hangat, dan rangkaian bunga rendah agar percakapan tetap akrab.

Kami menangani dekorasi, pencahayaan, hingga sound mini line-array dengan engineer yang standby sepanjang acara."),
                    gallery: v(&["/img/layanan/dekor-villa.jpg", "/img/layanan/dekor-galeri-meja.jpg", "/img/layanan/venue-atsiri.jpg", "/img/layanan/venue-nava.jpg"]),
                    features: v(&[
                        "table_restaurant | Dekorasi meja panjang banquet (30–50 pax)",
                        "flare | Fairy lights sky tunnel warm glow",
                        "speaker_group | Sound mini line-array 3.000 W",
                        "support_agent | Sound engineer profesional standby",
                    ]),
                },
                VendorPaket {
                    slug: s("sound"),
                    name: s("Custom Sound System & Silent Genset"),
                    tag: s("Technical Audio Solution"),
                    desc: s("Layanan audio murni bagi pasangan yang sudah punya vendor dekorasi sendiri. Menjamin vokal akad terdengar syahdu tanpa feedback mikrofon."),
                    price: 3_500_000,
                    price_from: true,
                    img: s("/img/layanan/dekor-sound.jpg"),
                    badge: s(""),
                    note: s("Acoustics & electric power"),
                    cta: s("Minta Penawaran Wattase"),
                    dark: false,
                    detail: s("Paket audio murni untuk pasangan yang sudah memiliki vendor dekorasi. Kami melakukan survey kelistrikan, menghitung kebutuhan wattase, dan menyiapkan genset silent sebagai cadangan.

Sound check teknis dilakukan H-1 bersama MC, band, atau pengisi acara agar tidak ada dengung maupun feedback di hari-H."),
                    gallery: v(&["/img/layanan/dekor-sound.jpg", "/img/layanan/dekor-galeri-speaker.jpg", "/img/layanan/venue-gedung-wanita.jpg"]),
                    features: v(&[
                        "tune | Kapasitas 3.000 W s/d 20.000 W",
                        "electric_bolt | Genset silent kedap suara 30–50 kVA",
                        "music_note | Optimasi live band & mini chamber",
                        "checklist | Termasuk sound check teknis H-1",
                    ]),
                },
            ],
            dekor_venue: vec![
                Venue {
                    img: s("/img/layanan/venue-nava.jpg"),
                    area: s("Tawangmangu"),
                    name: s("Nava Hotel Tawangmangu"),
                    desc: s("Ideal untuk konsep poolside sunset wedding & ballroom berkabut dengan panorama pegunungan terbuka."),
                    spec: s("volume_up | Rekomendasi sound: line-array outdoor 5.000 W"),
                },
                Venue {
                    img: s("/img/layanan/venue-lawu-park.jpg"),
                    area: s("Bumi Perkemahan Lawu"),
                    name: s("The Lawu Park Forest"),
                    desc: s("Sensasi hutan pinus otentik. Butuh tenda transparan pelindung kabut dan sound system tahan lembap."),
                    spec: s("forest | Spesialisasi: tenda transparan & genset silent"),
                },
                Venue {
                    img: s("/img/layanan/venue-tahura.jpg"),
                    area: s("Ngargoyoso"),
                    name: s("Tahura (Taman Hutan Raya)"),
                    desc: s("Kawasan konservasi asri dengan vegetasi rimbun, sempurna untuk ikrar suci bernuansa enchanted garden."),
                    spec: s("eco | Spesialisasi: instalasi botanical ramah lingkungan"),
                },
                Venue {
                    img: s("/img/layanan/venue-atsiri.jpg"),
                    area: s("Plaosan, Lawu"),
                    name: s("Rumah Atsiri Indonesia"),
                    desc: s("Perpaduan arsitektur vintage dan kebun bunga aromaterapi. Elegan dengan dekorasi minimalis beraksen gold."),
                    spec: s("yard | Estetika: glasshouse & lavender floral design"),
                },
                Venue {
                    img: s("/img/layanan/venue-tjolomadoe.jpg"),
                    area: s("Colomadu, Karanganyar"),
                    name: s("De Tjolomadoe Heritage Hall"),
                    desc: s("Venue megah bergaya industrial heritage eks pabrik gula. Perlu penataan delay speaker agar tidak bergema."),
                    spec: s("speaker | Spesialisasi: multi-zone audio delay & moving beam"),
                },
                Venue {
                    img: s("/img/layanan/venue-gedung-wanita.jpg"),
                    area: s("Manahan, Solo Kota"),
                    name: s("Gedung Wanita Solo"),
                    desc: s("Venue favorit resepsi besar adat Jawa di jantung Kota Solo dengan kapasitas ribuan tamu."),
                    spec: s("groups | Spesialisasi: gebyok 18 meter & sound 15.000 W"),
                },
            ],
            dekor_galeri: vec![
                Foto { img: s("/img/layanan/dekor-galeri-altar.jpg"), kecil: s("Kebun Teh Kemuning • Ngargoyoso"), judul: s("The Altar in the Clouds"), teks: s("") },
                Foto { img: s("/img/layanan/dekor-galeri-meja.jpg"), kecil: s("Detail Meja Jamuan"), judul: s("Artisanal Cotton & Brass"), teks: s("") },
                Foto { img: s("/img/layanan/dekor-galeri-speaker.jpg"), kecil: s("Estetika Teknis"), judul: s("Speaker Terkamuflase Asri"), teks: s("") },
                Foto { img: s("/img/layanan/dekor-galeri-gebyok.jpg"), kecil: s("Resepsi Adat Solo Basahan"), judul: s("Mahakarya Gebyok & Melati"), teks: s("") },
            ],
            mua_paket: vec![
                VendorPaket {
                    slug: s("solo-putri"),
                    name: s("Solo Putri & Basahan Klasik"),
                    tag: s("Warisan Tradisi Solo"),
                    desc: s("Tata paes presisi dengan lotho hitam pekat alami tanpa bau menyengat, cunduk mentul 7 atau 9 batang, dan busana jarik prada Solo asli."),
                    price: 4_500_000,
                    price_from: true,
                    img: s("/img/layanan/mua-solo-putri.jpg"),
                    badge: s("Adat Favorit"),
                    note: s(""),
                    cta: s("Pilih Paket Ini"),
                    dark: false,
                    detail: String::new(),
                    gallery: vec![],
                    features: v(&[
                        "Rias wajah pengantin manglingi tradisional",
                        "Ukiran paes halus (Solo Putri / Basahan)",
                        "Ronce melati basah grade A segar wangi",
                        "Gratis pemasangan jarik prada & busana",
                    ]),
                },
                VendorPaket {
                    slug: s("modern"),
                    name: s("Modern Botanical Glowing Bride"),
                    tag: s("Modern Glam"),
                    desc: s("Dewy complexion sehat bercahaya yang tidak mudah luntur, soft natural lashes, lip tint nude elegan, dipadukan modern hair styling atau hijab do."),
                    price: 3_800_000,
                    price_from: true,
                    img: s("/img/layanan/mua-modern.jpg"),
                    badge: s(""),
                    note: s(""),
                    cta: s("Pilih Paket Ini"),
                    dark: false,
                    detail: String::new(),
                    gallery: vec![],
                    features: v(&[
                        "Glass-skin longlasting complexion",
                        "Modern hair do / hijab do styling",
                        "Aksen bunga segar & pearl pins impor",
                        "Touch-up standby hingga resepsi selesai",
                    ]),
                },
                VendorPaket {
                    slug: s("all-in"),
                    name: s("All-In Family & Bridesmaids"),
                    tag: s("Paket Rias Komplit"),
                    desc: s("Solusi rias menyeluruh untuk kedua mempelai, ibu kandung & ibu mertua, serta 4 orang pagar ayu dengan tim terkoordinasi."),
                    price: 6_800_000,
                    price_from: false,
                    img: s("/img/layanan/mua-all-in.jpg"),
                    badge: s("Paling Lengkap & Hemat"),
                    note: s("Investasi all-in bundling"),
                    cta: s("Amankan Kuota Tanggal"),
                    dark: true,
                    detail: String::new(),
                    gallery: vec![],
                    features: v(&[
                        "verified | Rias pengantin akad + resepsi (ganti look)",
                        "verified | Rias 2 ibu mempelai + sanggul / hijab",
                        "verified | Rias 4 pagar ayu / bridesmaids",
                        "verified | Gratis trial session & pemasangan beskap",
                    ]),
                },
                VendorPaket {
                    slug: s("prewedding"),
                    name: s("Prewedding Glamour & Styling"),
                    tag: s("Sesi Photoshoot"),
                    desc: s("Riasan kamera high-definition yang tahan terik matahari maupun kabut tebal di lokasi alam seperti Cemoro Kandang atau Candi Cetho."),
                    price: 2_200_000,
                    price_from: true,
                    img: s("/img/layanan/mua-prewed.jpg"),
                    badge: s(""),
                    note: s(""),
                    cta: s("Pilih Paket Ini"),
                    dark: false,
                    detail: String::new(),
                    gallery: vec![],
                    features: v(&[
                        "2x konsep rias wajah & ganti gaya rambut",
                        "Stylist standby selama sesi foto (maks 6 jam)",
                        "Grooming tipis calon pengantin pria",
                        "Touch-up kit portabel untuk lokasi outdoor",
                    ]),
                },
            ],
            mua_galeri: vec![
                Foto {
                    img: s("/img/layanan/mua-paes.jpg"),
                    kecil: s("Filosofi Tradisi Solo"),
                    judul: s("Presisi Paes Alami"),
                    teks: s("Setiap cengkorongan, penitis, dan godheg digambar manual mengikuti proporsi wajah mempelai demi keselarasan pakem keraton."),
                },
                Foto { img: s("/img/layanan/mua-sebelum.jpg"), kecil: s("Sebelum rias"), judul: s(""), teks: s("Kulit asli calon pengantin dengan skin-barrier natural.") },
                Foto { img: s("/img/layanan/mua-sesudah.jpg"), kecil: s("Hasil akhir (manglingi)"), judul: s(""), teks: s("Aura manglingi tanpa mengubah karakter wajah asli.") },
                Foto {
                    img: s("/img/layanan/mua-mata.jpg"),
                    kecil: s("Teknik Rias Mata"),
                    judul: s("Soft Smokey Tajam & Ringan"),
                    teks: s("Bulu mata disusun per helai — nyaman dibuka-tutup selama prosesi akad berjam-jam."),
                },
                Foto {
                    img: s("/img/layanan/mua-melati.jpg"),
                    kecil: s("Aroma & Keharuman"),
                    judul: s("Ronce Melati Basah Grade A"),
                    teks: s("Dirangkai subuh hari dari kebun melati Boyolali & Karanganyar, harum menenangkan sepanjang resepsi."),
                },
            ],
            mua_profil: MuaProfil {
                name: s("Sekar Ayu Wandansari, S.Sn"),
                role: s("Master Bridal Stylist & Budayawan Rias Surakarta"),
                bio: s("Mewarisi ketelitian seni rias dari keluarga perias Keraton Surakarta Hadiningrat dan memperdalam tata kecantikan modern editorial di Jakarta dan Singapura. Sekar mendedikasikan sentuhannya agar setiap pengantin merasa dihargai, percaya diri, dan memancarkan kecantikan abadi."),
                img: s("/img/layanan/mua-sekar.jpg"),
                badge: s("10+ tahun pengalaman"),
                quote: s("Rias pengantin yang baik bukan yang menutupi jati diri, melainkan yang memantik rasa syukur dan mengabadikan ketulusan cinta di hadapan keluarga tercinta."),
                prestasi: v(&[
                    "verified_user | Sertifikasi BNSP rias pengantin | Lembaga Sertifikasi Kompetensi Nasional",
                    "military_tech | Juara I tata rias paes Solo | Festival Budaya Pengantin Jawa Tengah",
                    "clean_hands | Standar higienitas medis | Sanitasi kuas & alat UV-sterilized",
                    "favorite | 500+ pengantin bahagia | Testimoni bintang 5 terverifikasi",
                ]),
            },
            mua_testimoni: vec![
                Testimoni {
                    text: s("Nikah di Tawangmangu bulan Juli, suhunya 15°C dan sempat gerimis kabut. Aku takut muka cracking, tapi complexion-nya nempel dari jam 6 pagi sampai resepsi malam jam 9! Paesnya rapi banget."),
                    who: s("Dinda & Haris"),
                    meta: s("Tawangmangu Wedding"),
                },
                Testimoni {
                    text: s("Paket All-In ngebantu banget. Ibu dan mertua rewel soal sanggul adat, tapi tim MUA sabar menjelaskan dan hasilnya manglingi semua. Nggak ada biaya transport padahal rumah di pelosok Karanganyar."),
                    who: s("Raras & Nanda"),
                    meta: s("Gedung Wanita Karanganyar"),
                },
                Testimoni {
                    text: s("Paes Basahan-nya sangat rapi dan lotho-nya nggak bikin gatal. Melatinya masih wangi dan segar sampai malam. Tim tiba jam 3 subuh tanpa telat sedetik pun."),
                    who: s("Anisa & Pradana"),
                    meta: s("Solo Kota"),
                },
            ],
            seserahan_paket: vec![
                SeserahanPaket {
                    slug: s("akrilik-classic"),
                    name: s("Akrilik Bening Classic"),
                    tag: s("Akrilik"),
                    desc: s("Kotak akrilik bening 3 mm bertutup, isi seserahan terlihat rapi dari segala sisi. Cocok untuk lamaran & akad sederhana."),
                    price: 350_000,
                    kotak: 7,
                    deposit: 300_000,
                    img: s("/img/layanan/seserahan-akrilik.svg"),
                    badge: s(""),
                    popular: false,
                    features: v(&["inventory_2 | 7 kotak (2 besar, 5 sedang)", "Alas kain satin & pita warna netral", "Lap microfiber & plastik pelindung"]),
                },
                SeserahanPaket {
                    slug: s("akrilik-gold"),
                    name: s("Akrilik Gold Mirror"),
                    tag: s("Rangka emas"),
                    desc: s("Rangka kuningan emas dengan alas cermin — tampak mewah di foto dan senada dengan dekorasi pelaminan modern."),
                    price: 550_000,
                    kotak: 9,
                    deposit: 500_000,
                    img: s("/img/layanan/seserahan-gold.svg"),
                    badge: s("Paling Favorit"),
                    popular: true,
                    features: v(&["inventory_2 | 9 kotak (termasuk 1 kotak mahar)", "Alas cermin & kaki emas", "auto_awesome | Gratis kartu nama isi seserahan"]),
                },
                SeserahanPaket {
                    slug: s("kayu-jati-ukir"),
                    name: s("Kayu Jati Ukir Jawa"),
                    tag: s("Adat Jawa"),
                    desc: s("Nampan & kotak kayu jati berukir motif Jepara — pas untuk srah-srahan adat Solo dan Yogyakarta."),
                    price: 650_000,
                    kotak: 7,
                    deposit: 750_000,
                    img: s("/img/layanan/seserahan-kayu.svg"),
                    badge: s("Adat"),
                    popular: false,
                    features: v(&["inventory_2 | 7 wadah kayu jati ukir", "Kain batik sidomukti penutup", "Cocok dengan kembar mayang & pisang sanggan"]),
                },
                SeserahanPaket {
                    slug: s("rotan-rustic"),
                    name: s("Rotan & Anyaman Rustic"),
                    tag: s("Rustic"),
                    desc: s("Keranjang rotan natural dengan lace & kain goni — hangat untuk tema garden, intimate, dan boho."),
                    price: 400_000,
                    kotak: 9,
                    deposit: 300_000,
                    img: s("/img/layanan/seserahan-rotan.svg"),
                    badge: s(""),
                    popular: false,
                    features: v(&["inventory_2 | 9 keranjang rotan berbagai ukuran", "Lace, goni & daun kering", "Ringan dibawa rombongan"]),
                },
            ],
            seserahan_opsi: vec![
                SeserahanOpsi { slug: s("hias"), name: s("Jasa hias & susun isi"), desc: s("Isi dari Anda, kami tata rapi di tiap kotak"), price: 35_000, per_kotak: true },
                SeserahanOpsi { slug: s("bunga"), name: s("Bunga & daun segar"), desc: s("Rangkaian mini di sudut tiap kotak"), price: 25_000, per_kotak: true },
                SeserahanOpsi { slug: s("pita"), name: s("Kain & pita warna custom"), desc: s("Disamakan dengan warna tema acara"), price: 75_000, per_kotak: false },
                SeserahanOpsi { slug: s("kaligrafi"), name: s("Papan nama pengantin"), desc: s("Akrilik cutting nama kedua mempelai"), price: 120_000, per_kotak: false },
                SeserahanOpsi { slug: s("mahar"), name: s("Hias mahar / buket uang"), desc: s("Bingkai atau buket, uang dari Anda"), price: 150_000, per_kotak: false },
            ],
            seserahan_antar: vec![
                SeserahanAntar { slug: s("ambil-sendiri"), name: s("Ambil & kembalikan sendiri di studio"), price: 0 },
                SeserahanAntar { slug: s("tawangmangu"), name: s("Tawangmangu / Karangpandan / Ngargoyoso"), price: 50_000 },
                SeserahanAntar { slug: s("karanganyar"), name: s("Karanganyar Kota / Tasikmadu"), price: 75_000 },
                SeserahanAntar { slug: s("solo"), name: s("Solo Kota / Colomadu / Sukoharjo"), price: 100_000 },
                SeserahanAntar { slug: s("luar"), name: s("Boyolali / Sragen / Wonogiri"), price: 150_000 },
            ],
            seserahan_galeri: vec![
                Foto { img: s("/img/layanan/seserahan-gold.svg"), kecil: s("Akad • Solo Baru"), judul: s("Gold Mirror untuk Akad Modern"), teks: s("") },
                Foto { img: s("/img/layanan/seserahan-kayu.svg"), kecil: s("Srah-srahan Adat"), judul: s("Kayu Jati Ukir & Batik"), teks: s("") },
                Foto { img: s("/img/layanan/seserahan-rotan.svg"), kecil: s("Intimate Garden"), judul: s("Rotan Rustic di Kebun Teh"), teks: s("") },
                Foto { img: s("/img/layanan/seserahan-akrilik.svg"), kecil: s("Lamaran • Karanganyar"), judul: s("Akrilik Bening yang Serba Terlihat"), teks: s("") },
            ],
            seserahan_info: SeserahanInfo {
                hero: s("/img/layanan/seserahan-hero.svg"),
                lama_sewa: s("3 hari (H-1 sampai H+1 acara)"),
                min_booking: s("H-14 sebelum acara"),
                syarat: v(&[
                    "event_available | DP 50% mengunci tanggal; pelunasan + deposit saat wadah diterima.",
                    "savings | Deposit dikembalikan penuh maksimal 1×24 jam setelah wadah kembali utuh.",
                    "inventory_2 | Wadah dikembalikan kosong; bersih-bersih ringan sudah termasuk.",
                    "report | Kotak retak/hilang dipotong dari deposit sesuai harga ganti per kotak.",
                    "schedule | Terlambat mengembalikan dikenakan Rp 50.000 per hari.",
                ]),
            },
            seserahan_testimoni: vec![
                Testimoni {
                    text: s("Kotak gold mirror-nya bikin foto akad kelihatan mewah banget. Isi seserahan disusun rapi sama timnya, tinggal bawa. Deposit balik besok paginya."),
                    who: s("Salsa & Dimas"),
                    meta: s("Akad • Solo Baru"),
                },
                Testimoni {
                    text: s("Pakai kayu jati ukir buat srah-srahan, keluarga besar sampai tanya sewa di mana. Diantar ke Tawangmangu tepat waktu H-1."),
                    who: s("Wulan & Arif"),
                    meta: s("Adat Jawa • Tawangmangu"),
                },
                Testimoni {
                    text: s("Booking mendadak H-10 masih dibantu. Rotan rustic-nya ringan, cocok buat rombongan yang jalan kaki ke rumah mempelai."),
                    who: s("Nadia & Fajar"),
                    meta: s("Lamaran • Karanganyar"),
                },
            ],
            pembayaran: Pembayaran {
                metode: s("ShopeePay"),
                nomor: s("089635816942"),
                atas_nama: s("rjandoni rahmana"),
                catatan: s("Transfer sesuai total (sampai digit terakhir), lalu unggah tangkapan layar bukti transfer di halaman Kelola."),
            },
            umum: Umum {
                cetak_hero: s("/img/layanan/cetak-hero.jpg"),
                dekor_hero: s("/img/layanan/dekor-hero.jpg"),
                mua_hero: s("/img/layanan/mua-hero.jpg"),
                cetak_bonus_qty: 500,
                cetak_bonus_label: s("Nilai bonus Rp 450.000"),
                custom_from: 599_000,
                mua_maks_per_hari: 2,
                mua_slot_sisa: 14,
                mua_slot_persen: 75,
            },
        }
    }
}

// ── Skema editor admin ─────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Teks satu baris.
    Text,
    /// Teks panjang.
    Long,
    /// Rupiah (bilangan bulat ≥ 0).
    Money,
    /// Bilangan bulat ≥ 0.
    Number,
    /// URL gambar / unggahan.
    Image,
    /// Daftar, satu item per baris.
    Lines,
    /// Galeri: daftar URL foto + unggah banyak sekaligus (editor admin).
    Gallery,
    Bool,
}

pub struct Field {
    pub key: &'static str,
    pub label: &'static str,
    pub kind: Kind,
    pub help: &'static str,
}

const fn f(key: &'static str, label: &'static str, kind: Kind) -> Field {
    Field { key, label, kind, help: "" }
}
const fn fh(key: &'static str, label: &'static str, kind: Kind, help: &'static str) -> Field {
    Field { key, label, kind, help }
}

pub struct Section {
    /// = nama field di `Konten` = kunci baris `site_content`.
    pub key: &'static str,
    pub group: &'static str,
    pub title: &'static str,
    pub help: &'static str,
    /// true = satu objek (bukan daftar).
    pub single: bool,
    /// Kolom yang dipakai sebagai judul item di daftar.
    pub title_field: &'static str,
    pub fields: &'static [Field],
    /// Halaman publik untuk tombol "Lihat halaman".
    pub page: &'static str,
}

use Kind::*;

const ICON_HELP: &str = "Satu per baris. Boleh diawali ikon: \"spa | Pelaminan botanical\".";

/// Paket dekorasi = paket vendor + halaman detail (/dekorasi/{slug}).
const DEKOR_FIELDS: &[Field] = &[
    f("name", "Nama paket", Text),
    fh("slug", "Kode", Text, "Kosongkan = dibuat dari nama. Dipakai di alamat halaman detail /dekorasi/{kode} — jangan diubah bila sudah dibagikan."),
    f("tag", "Label kecil di atas judul", Text),
    f("desc", "Deskripsi singkat (kartu)", Long),
    fh("detail", "Deskripsi lengkap (halaman detail)", Long, "Pisahkan paragraf dengan baris kosong."),
    f("price", "Harga (Rp)", Money),
    fh("price_from", "Tampilkan sebagai \"Mulai …\"", Bool, ""),
    f("img", "Foto utama (kartu)", Image),
    fh("gallery", "Galeri foto halaman detail", Gallery, "Unggah beberapa foto sekaligus. Urutan = urutan baris; foto pertama tampil paling besar."),
    f("badge", "Lencana di foto (opsional)", Text),
    f("note", "Catatan kecil di atas harga", Text),
    f("cta", "Teks tombol", Text),
    f("dark", "Kartu gelap (sorotan)", Bool),
    fh("features", "Fitur", Lines, ICON_HELP),
];

const VENDOR_FIELDS: &[Field] = &[
    f("name", "Nama paket", Text),
    fh("slug", "Kode", Text, "Kosongkan = dibuat dari nama. Jangan diubah bila sudah dipakai tautan."),
    f("tag", "Label kecil di atas judul", Text),
    f("desc", "Deskripsi", Long),
    f("price", "Harga (Rp)", Money),
    fh("price_from", "Tampilkan sebagai \"Mulai …\"", Bool, ""),
    f("img", "Foto paket", Image),
    f("badge", "Lencana di foto (opsional)", Text),
    f("note", "Catatan kecil di atas harga", Text),
    f("cta", "Teks tombol", Text),
    f("dark", "Kartu gelap (sorotan)", Bool),
    fh("features", "Fitur", Lines, ICON_HELP),
];

const FOTO_FIELDS: &[Field] = &[
    f("img", "Foto", Image),
    f("kecil", "Label kecil", Text),
    f("judul", "Judul", Text),
    f("teks", "Keterangan", Long),
];

const TESTI_FIELDS: &[Field] = &[
    f("who", "Nama pasangan", Text),
    f("meta", "Lokasi / keterangan", Text),
    f("text", "Isi testimoni", Long),
];

pub const SECTIONS: &[Section] = &[
    Section {
        key: "paket",
        group: "Undangan Digital",
        title: "Paket & Harga",
        help: "Harga dihitung ulang di server saat pemesanan. Menghapus paket tidak mengubah pesanan lama.",
        single: false,
        title_field: "name",
        page: "/paket",
        fields: &[
            f("name", "Nama paket", Text),
            fh("slug", "Kode", Text, "Kosongkan = dibuat dari nama."),
            f("price", "Harga (Rp)", Money),
            f("period", "Masa aktif (teks)", Text),
            f("popular", "Tandai \"Paling Populer\"", Bool),
            fh("features", "Fitur", Lines, "Satu fitur per baris."),
        ],
    },
    Section {
        key: "addon",
        group: "Undangan Digital",
        title: "Add-on",
        help: "",
        single: false,
        title_field: "name",
        page: "/paket",
        fields: &[f("name", "Nama", Text), fh("slug", "Kode", Text, "Kosongkan = dibuat dari nama."), f("desc", "Keterangan", Text), f("price", "Harga (Rp)", Money)],
    },
    Section {
        key: "kupon",
        group: "Undangan Digital",
        title: "Kupon Promo",
        help: "Kupon non-aktif tidak bisa dipakai. Kupon aktif pertama ditampilkan di halaman Paket.",
        single: false,
        title_field: "code",
        page: "/paket",
        fields: &[f("code", "Kode kupon", Text), f("amount", "Potongan (Rp)", Money), f("active", "Aktif", Bool)],
    },
    Section {
        key: "cetak_paket",
        group: "Cetak Undangan",
        title: "Paket Cetak",
        help: "Dipakai katalog & kalkulator. Berat dipakai menghitung ongkir.",
        single: false,
        title_field: "name",
        page: "/cetak",
        fields: &[
            f("name", "Nama paket", Text),
            fh("slug", "Kode", Text, "Kosongkan = dibuat dari nama."),
            f("short", "Nama singkat (kalkulator)", Text),
            f("tier", "Label tingkat", Text),
            f("desc", "Deskripsi", Long),
            f("price", "Harga per pcs (Rp)", Money),
            f("min_qty", "Minimal pesan (pcs)", Number),
            f("gram", "Berat per set (gram)", Number),
            f("img", "Foto", Image),
            f("popular", "Tandai \"Paling Favorit\"", Bool),
            fh("features", "Fitur", Lines, "Satu fitur per baris."),
        ],
    },
    Section {
        key: "cetak_finishing",
        group: "Cetak Undangan",
        title: "Finishing Tambahan",
        help: "",
        single: false,
        title_field: "name",
        page: "/cetak#kalkulator",
        fields: &[f("name", "Nama", Text), fh("slug", "Kode", Text, "Kosongkan = dibuat dari nama."), f("price", "Tambahan per pcs (Rp)", Money), f("gram", "Tambahan berat (gram)", Number)],
    },
    Section {
        key: "cetak_wilayah",
        group: "Cetak Undangan",
        title: "Tarif Ongkir per Wilayah",
        help: "",
        single: false,
        title_field: "name",
        page: "/cetak#kalkulator",
        fields: &[f("name", "Wilayah", Text), fh("slug", "Kode", Text, "Kosongkan = dibuat dari nama."), f("kurir", "Kurir", Text), f("per_kg", "Tarif per kg (Rp)", Money)],
    },
    Section { key: "cetak_testimoni", group: "Cetak Undangan", title: "Testimoni Cetak", help: "", single: false, title_field: "who", page: "/cetak", fields: TESTI_FIELDS },
    Section {
        key: "dekor_paket",
        group: "Dekorasi & Sound",
        title: "Paket Dekorasi",
        help: "Tiap paket punya halaman detail /dekorasi/{kode} berisi deskripsi lengkap & galeri foto.",
        single: false,
        title_field: "name",
        page: "/dekorasi",
        fields: DEKOR_FIELDS,
    },
    Section {
        key: "dekor_venue",
        group: "Dekorasi & Sound",
        title: "Spotlight Venue",
        help: "",
        single: false,
        title_field: "name",
        page: "/dekorasi",
        fields: &[
            f("name", "Nama venue", Text),
            f("area", "Area (label di foto)", Text),
            f("img", "Foto", Image),
            f("desc", "Deskripsi", Long),
            fh("spec", "Spesialisasi", Text, "Boleh diawali ikon: \"forest | Tenda transparan\"."),
        ],
    },
    Section {
        key: "dekor_galeri",
        group: "Dekorasi & Sound",
        title: "Galeri Dekorasi",
        help: "Foto pertama tampil besar; foto keempat melebar penuh.",
        single: false,
        title_field: "judul",
        page: "/dekorasi",
        fields: FOTO_FIELDS,
    },
    Section { key: "mua_paket", group: "Make Up Pengantin", title: "Paket Rias", help: "", single: false, title_field: "name", page: "/mua", fields: VENDOR_FIELDS },
    Section {
        key: "mua_galeri",
        group: "Make Up Pengantin",
        title: "Galeri Rias",
        help: "Urutan menentukan tata letak: 1 = kartu besar, 2 & 3 = pasangan Sebelum/Sesudah, 4 dst = kartu biasa.",
        single: false,
        title_field: "kecil",
        page: "/mua#galeri",
        fields: FOTO_FIELDS,
    },
    Section {
        key: "mua_profil",
        group: "Make Up Pengantin",
        title: "Profil Perias Utama",
        help: "",
        single: true,
        title_field: "name",
        page: "/mua",
        fields: &[
            f("name", "Nama", Text),
            f("role", "Jabatan", Text),
            f("img", "Foto", Image),
            f("badge", "Lencana di foto", Text),
            f("bio", "Profil singkat", Long),
            f("quote", "Kutipan", Long),
            fh("prestasi", "Prestasi / sertifikasi", Lines, "Per baris: \"ikon | judul | keterangan\"."),
        ],
    },
    Section { key: "mua_testimoni", group: "Make Up Pengantin", title: "Testimoni Rias", help: "", single: false, title_field: "who", page: "/mua", fields: TESTI_FIELDS },
    Section {
        key: "seserahan_paket",
        group: "Sewa Seserahan",
        title: "Paket Sewa Seserahan",
        help: "Harga = sewa satu set selama masa sewa. Jumlah kotak dipakai mengalikan layanan tambahan \"per kotak\". Deposit dikembalikan setelah wadah kembali.",
        single: false,
        title_field: "name",
        page: "/seserahan",
        fields: &[
            f("name", "Nama paket", Text),
            fh("slug", "Kode", Text, "Kosongkan = dibuat dari nama. Jangan diubah bila sudah dipakai tautan."),
            fh("tag", "Label kecil di atas judul", Text, "Mis. \"Akrilik\" — jumlah kotak sudah tampil otomatis."),
            f("desc", "Deskripsi", Long),
            f("price", "Harga sewa per set (Rp)", Money),
            f("kotak", "Jumlah kotak / wadah", Number),
            f("deposit", "Deposit / jaminan (Rp)", Money),
            f("img", "Foto", Image),
            f("badge", "Lencana di foto (opsional)", Text),
            f("popular", "Tandai \"Paling Favorit\" (terpilih di kalkulator)", Bool),
            fh("features", "Isi paket", Lines, ICON_HELP),
        ],
    },
    Section {
        key: "seserahan_opsi",
        group: "Sewa Seserahan",
        title: "Layanan Tambahan",
        help: "Opsi yang bisa dicentang di kalkulator. Centang \"per kotak\" agar harganya dikali jumlah kotak paket yang dipilih.",
        single: false,
        title_field: "name",
        page: "/seserahan#kalkulator",
        fields: &[
            f("name", "Nama layanan", Text),
            fh("slug", "Kode", Text, "Kosongkan = dibuat dari nama."),
            f("desc", "Keterangan singkat", Text),
            f("price", "Harga (Rp)", Money),
            f("per_kotak", "Harga per kotak", Bool),
        ],
    },
    Section {
        key: "seserahan_antar",
        group: "Sewa Seserahan",
        title: "Tarif Antar-Jemput",
        help: "Sekali antar + sekali ambil. Isi Rp 0 untuk pilihan ambil sendiri.",
        single: false,
        title_field: "name",
        page: "/seserahan#kalkulator",
        fields: &[f("name", "Area", Text), fh("slug", "Kode", Text, "Kosongkan = dibuat dari nama."), f("price", "Tarif (Rp)", Money)],
    },
    Section {
        key: "seserahan_galeri",
        group: "Sewa Seserahan",
        title: "Galeri Seserahan",
        help: "Foto pertama tampil besar; foto keempat melebar penuh.",
        single: false,
        title_field: "judul",
        page: "/seserahan#galeri",
        fields: FOTO_FIELDS,
    },
    Section {
        key: "seserahan_info",
        group: "Sewa Seserahan",
        title: "Foto Hero & Ketentuan Sewa",
        help: "",
        single: true,
        title_field: "",
        page: "/seserahan#ketentuan",
        fields: &[
            f("hero", "Foto hero halaman Seserahan", Image),
            f("lama_sewa", "Lama sewa", Text),
            f("min_booking", "Batas booking", Text),
            fh("syarat", "Syarat & ketentuan", Lines, "Satu aturan per baris, boleh diawali ikon: \"savings | Deposit kembali 1×24 jam\"."),
        ],
    },
    Section { key: "seserahan_testimoni", group: "Sewa Seserahan", title: "Testimoni Seserahan", help: "", single: false, title_field: "who", page: "/seserahan", fields: TESTI_FIELDS },
    Section {
        key: "pembayaran",
        group: "Undangan Digital",
        title: "Rekening Pembayaran",
        help: "Satu-satunya tujuan transfer pesanan undangan. Tampil di /buat & dashboard Kelola, dan ikut di pesan WA bukti transfer.",
        single: true,
        title_field: "",
        page: "/buat",
        fields: &[
            fh("metode", "Nama dompet / bank", Text, "mis. ShopeePay"),
            f("nomor", "Nomor tujuan", Text),
            f("atas_nama", "Atas nama", Text),
            f("catatan", "Catatan untuk pemesan", Long),
        ],
    },
    Section {
        key: "umum",
        group: "Umum",
        title: "Foto Hero & Angka",
        help: "",
        single: true,
        title_field: "",
        page: "/",
        fields: &[
            f("cetak_hero", "Foto hero halaman Cetak", Image),
            f("dekor_hero", "Foto hero halaman Dekorasi", Image),
            f("mua_hero", "Foto hero halaman MUA", Image),
            f("cetak_bonus_qty", "Cetak: minimal pcs untuk bonus undangan digital", Number),
            f("cetak_bonus_label", "Cetak: label nilai bonus", Text),
            f("custom_from", "Harga mulai paket Custom Desain (Rp)", Money),
            f("mua_maks_per_hari", "MUA: maksimal pengantin per hari", Number),
            f("mua_slot_sisa", "MUA: sisa tanggal tersedia", Number),
            fh("mua_slot_persen", "MUA: persen slot terisi", Number, "0–100"),
        ],
    },
];

pub fn section(key: &str) -> Option<&'static Section> {
    SECTIONS.iter().find(|s| s.key == key)
}

impl Konten {
    /// JSON satu bagian (daftar atau objek).
    pub fn section_json(&self, key: &str) -> serde_json::Value {
        serde_json::to_value(self).ok().and_then(|v| v.get(key).cloned()).unwrap_or(serde_json::Value::Null)
    }

    /// Terapkan satu bagian dari DB. Data rusak/tak cocok → bagian itu tetap bawaan.
    pub fn apply(&mut self, key: &str, data: serde_json::Value) -> bool {
        let Ok(mut all) = serde_json::to_value(&*self) else { return false };
        let Some(obj) = all.as_object_mut() else { return false };
        if !obj.contains_key(key) {
            return false;
        }
        obj.insert(key.to_string(), data);
        match serde_json::from_value::<Konten>(all) {
            Ok(k) => {
                *self = k;
                true
            }
            Err(_) => false,
        }
    }
}

/// Batas foto per galeri.
pub const GALLERY_MAX: usize = 24;

/// Nama input form editor: `it{i}__{field}`.
pub fn input_name(i: usize, key: &str) -> String {
    format!("it{i}__{key}")
}

/// Susun JSON satu bagian dari isian editor. `get(name)` = nilai input;
/// `n` = jumlah baris form (termasuk baris kosong "tambah baru"). Baris
/// bertanda hapus atau kosong dibuang; urutan mengikuti kolom `_urut`.
pub fn form_to_json(sec: &Section, n: usize, get: impl Fn(&str) -> String) -> Result<serde_json::Value, String> {
    use serde_json::{Map, Value};
    let clean = |v: String, max: usize| -> String {
        v.trim().chars().filter(|c| !c.is_control() || *c == '\n').take(max).collect()
    };
    let mut rows: Vec<(i64, Map<String, Value>)> = Vec::new();
    for i in 0..n.min(200) {
        if get(&input_name(i, "_hapus")) == "1" {
            continue;
        }
        let mut obj = Map::new();
        let mut filled = false;
        for fld in sec.fields {
            let raw = get(&input_name(i, fld.key));
            let val = match fld.kind {
                Text => Value::String(clean(raw, 160).replace('\n', " ")),
                Long => Value::String(clean(raw, 1200)),
                Image => {
                    let u = clean(raw, 500);
                    if !u.is_empty() && !super::skin::is_safe_url(&u) {
                        return Err(format!("{}: URL gambar tidak valid ({u}).", fld.label));
                    }
                    Value::String(u)
                }
                Money | Number => {
                    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).take(12).collect();
                    Value::from(digits.parse::<i64>().unwrap_or(0))
                }
                Bool => Value::Bool(raw == "1"),
                Lines => Value::Array(
                    raw.lines().map(|l| clean(l.to_string(), 200)).filter(|l| !l.is_empty()).take(30).map(Value::String).collect(),
                ),
                Gallery => {
                    let mut urls = Vec::new();
                    for l in raw.lines().map(|l| clean(l.to_string(), 500)).filter(|l| !l.is_empty()) {
                        if !super::skin::is_safe_url(&l) {
                            return Err(format!("{}: URL foto tidak valid ({l}).", fld.label));
                        }
                        if !urls.contains(&l) {
                            urls.push(l);
                        }
                    }
                    Value::Array(urls.into_iter().take(GALLERY_MAX).map(Value::String).collect())
                }
            };
            filled |= match &val {
                Value::String(s) => !s.is_empty(),
                Value::Array(a) => !a.is_empty(),
                Value::Number(n) => n.as_i64() != Some(0),
                _ => false,
            };
            obj.insert(fld.key.to_string(), val);
        }
        if !filled && !sec.single {
            continue;
        }
        // Kode kosong → dibuat dari nama/judul; kode ganda ditolak.
        if let Some(Value::String(slug)) = obj.get("slug").cloned() {
            let base = if slug.is_empty() {
                obj.get(sec.title_field).and_then(|v| v.as_str()).unwrap_or("").to_string()
            } else {
                slug
            };
            obj.insert("slug".into(), Value::String(super::fmt::key(&base)));
        }
        let urut: i64 = get(&input_name(i, "_urut")).trim().parse().unwrap_or(i as i64 * 10);
        rows.push((urut, obj));
        if sec.single {
            break;
        }
    }
    rows.sort_by_key(|r| r.0);
    if sec.single {
        return Ok(Value::Object(rows.into_iter().next().map(|r| r.1).unwrap_or_default()));
    }
    let mut seen = std::collections::HashSet::new();
    for (_, o) in &rows {
        if let Some(slug) = o.get("slug").and_then(|v| v.as_str()) {
            if slug.is_empty() {
                return Err("Ada item tanpa nama — isi nama agar kodenya bisa dibuat.".into());
            }
            if !seen.insert(slug.to_string()) {
                return Err(format!("Kode \"{slug}\" dipakai dua kali — kode harus unik."));
            }
        }
    }
    Ok(Value::Array(rows.into_iter().map(|r| Value::Object(r.1)).collect()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skema_cocok_dengan_struct() {
        let k = Konten::default();
        let json = serde_json::to_value(&k).unwrap();
        for sec in SECTIONS {
            let v = json.get(sec.key).unwrap_or_else(|| panic!("bagian {} tak ada di Konten", sec.key));
            let sample = if sec.single { v.clone() } else { v.as_array().and_then(|a| a.first().cloned()).expect("contoh item") };
            for fld in sec.fields {
                assert!(sample.get(fld.key).is_some(), "{}.{} tak ada di struct", sec.key, fld.key);
            }
        }
    }

    #[test]
    fn harga_dari_konten() {
        let mut k = Konten::default();
        assert_eq!(k.calc_total("gold", &[], ""), (149_000, 0, 149_000));
        assert_eq!(k.calc_total("platinum", &["tema-custom".into()], "hematnikah40"), (274_000, 40_000, 234_000));
        k.kupon[0].active = false;
        assert_eq!(k.calc_total("gold", &[], "HEMATNIKAH40").1, 0);
        // Paket tak dikenal → paket populer.
        assert_eq!(k.calc_total("tidak-ada", &[], "").0, 149_000);
        assert_eq!(k.min_price(), 79_000);
    }

    #[test]
    fn apply_bagian() {
        let mut k = Konten::default();
        assert!(k.apply("kupon", serde_json::json!([{ "code": "NIKAH10", "amount": 10000, "active": true }])));
        assert_eq!(k.kupon.len(), 1);
        assert_eq!(k.kupon[0].code, "NIKAH10");
        // Tipe salah → ditolak, isi lama utuh.
        assert!(!k.apply("paket", serde_json::json!("rusak")));
        assert_eq!(k.paket.len(), 3);
        assert!(!k.apply("tidak_ada", serde_json::json!([])));
    }

    #[test]
    fn form_editor() {
        let sec = section("addon").unwrap();
        let data = [
            ("it0__name", "Domain"), ("it0__price", "Rp 99.000"), ("it0___urut", "20"),
            ("it1__name", "Foto Tambahan"), ("it1__price", "15000"), ("it1___urut", "10"),
            ("it2__name", "Dihapus"), ("it2___hapus", "1"),
            // it3 = baris "tambah baru" kosong → dibuang
        ];
        let get = |k: &str| data.iter().find(|d| d.0 == k).map(|d| d.1.to_string()).unwrap_or_default();
        let v = form_to_json(sec, 4, get).unwrap();
        let mut k = Konten::default();
        assert!(k.apply("addon", v));
        assert_eq!(k.addon.len(), 2);
        assert_eq!((k.addon[0].slug.as_str(), k.addon[0].price), ("foto-tambahan", 15_000));
        assert_eq!(k.addon[1].price, 99_000);
        // Kode ganda & URL gambar berbahaya ditolak.
        let dup = [("it0__name", "A"), ("it1__name", "a")];
        assert!(form_to_json(sec, 2, |k| dup.iter().find(|d| d.0 == k).map(|d| d.1.to_string()).unwrap_or_default()).is_err());
        let img = [("it0__img", "javascript:alert(1)"), ("it0__name", "X")];
        assert!(form_to_json(section("dekor_paket").unwrap(), 1, |k| img.iter().find(|d| d.0 == k).map(|d| d.1.to_string()).unwrap_or_default()).is_err());
    }

    #[test]
    fn galeri_dekorasi() {
        let sec = section("dekor_paket").unwrap();
        let data = [
            ("it0__name", "Paket Uji"),
            ("it0__gallery", "/img/a.jpg\n\n/img/b.jpg\n/img/a.jpg"),
        ];
        let get = |k: &str| data.iter().find(|d| d.0 == k).map(|d| d.1.to_string()).unwrap_or_default();
        let mut k = Konten::default();
        assert!(k.apply("dekor_paket", form_to_json(sec, 1, get).unwrap()));
        assert_eq!(k.dekor_paket[0].gallery, vec!["/img/a.jpg", "/img/b.jpg"], "baris kosong & ganda dibuang");
        let bad = [("it0__name", "X"), ("it0__gallery", "javascript:alert(1)")];
        assert!(form_to_json(sec, 1, |k| bad.iter().find(|d| d.0 == k).map(|d| d.1.to_string()).unwrap_or_default()).is_err());
        // Isi bawaan: tiap paket dekorasi punya galeri & detail.
        assert!(Konten::default().dekor_paket.iter().all(|p| !p.gallery.is_empty() && !p.detail.is_empty()));
    }

    #[test]
    fn baris_ikon() {
        assert_eq!(icon_line("spa | Pelaminan", "check_circle"), ("spa", "Pelaminan"));
        assert_eq!(icon_line("bukan_ikon | X", "check_circle"), ("check_circle", "X"));
        assert_eq!(icon_line("Tanpa ikon", "check_circle"), ("check_circle", "Tanpa ikon"));
    }
}
