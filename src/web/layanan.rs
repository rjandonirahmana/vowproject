//! web/layanan.rs — logika layanan pendukung pernikahan (cetak, dekorasi,
//! MUA, sewa seserahan): kalkulator biaya cetak + ongkir, kalkulator sewa
//! seserahan, dan penyusun pesan WhatsApp.
//!
//! Datanya (paket, harga, tarif ongkir) ada di `Konten` (web/konten.rs) dan
//! bisa disunting admin. Dipakai dua sisi: halaman (/cetak, /dekorasi, /mua)
//! dan handler server `GET /layanan/wa` yang menyusun ulang pesan & estimasi
//! dari query dengan data server — lalu mengalihkan ke wa.me admin.

use super::fmt::rupiah;
use super::konten::{CetakPaket, Konten, SeserahanAntar, SeserahanPaket, VendorPaket, Wilayah};

// ── Kalkulator cetak ───────────────────────────────────────────────────────

pub const CETAK_QTY_MIN: i64 = 200;
pub const CETAK_QTY_MAX: i64 = 2_000;
/// Di atas berat ini ongkir cargo biasanya lebih hemat.
pub const CETAK_CARGO_KG: i64 = 10;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct CetakInput {
    pub paket: String,
    pub qty: i64,
    /// Slug finishing yang dicentang.
    pub finishing: Vec<String>,
    pub wilayah: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Estimasi {
    /// Jumlah setelah disesuaikan ke minimum paket & batas slider.
    pub qty: i64,
    pub per_pcs: i64,
    pub cetak: i64,
    pub gram: i64,
    pub ongkir: i64,
    pub total: i64,
    pub bonus: bool,
    pub cargo: bool,
}

impl CetakInput {
    pub fn paket<'a>(&self, k: &'a Konten) -> Option<&'a CetakPaket> {
        k.cetak_paket
            .iter()
            .find(|p| p.slug == self.paket)
            .or_else(|| k.cetak_paket.iter().find(|p| p.popular))
            .or(k.cetak_paket.first())
    }
    pub fn wilayah<'a>(&self, k: &'a Konten) -> Option<&'a Wilayah> {
        k.cetak_wilayah.iter().find(|w| w.slug == self.wilayah).or(k.cetak_wilayah.first())
    }

    pub fn estimasi(&self, k: &Konten) -> Estimasi {
        let (price, min_qty, gram_pcs) = self.paket(k).map(|p| (p.price, p.min_qty, p.gram)).unwrap_or((0, 0, 0));
        let (fin_price, fin_gram) = k
            .cetak_finishing
            .iter()
            .filter(|f| self.finishing.contains(&f.slug))
            .fold((0, 0), |(a, b), f| (a + f.price.max(0), b + f.gram.max(0)));
        let qty = self.qty.clamp(CETAK_QTY_MIN, CETAK_QTY_MAX).max(min_qty);
        let per_pcs = price + fin_price;
        let gram = qty * (gram_pcs + fin_gram);
        let kg = (gram + 999) / 1000;
        let ongkir = kg * self.wilayah(k).map(|w| w.per_kg).unwrap_or(0);
        let cetak = qty * per_pcs;
        let bonus_qty = k.umum.cetak_bonus_qty;
        Estimasi {
            qty,
            per_pcs,
            cetak,
            gram,
            ongkir,
            total: cetak + ongkir,
            bonus: bonus_qty > 0 && qty >= bonus_qty,
            cargo: kg > CETAK_CARGO_KG,
        }
    }
}

// ── Kalkulator sewa seserahan ──────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SeserahanInput {
    pub paket: String,
    /// Slug layanan tambahan yang dicentang.
    pub opsi: Vec<String>,
    pub antar: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct SeserahanEst {
    pub kotak: i64,
    pub sewa: i64,
    pub tambahan: i64,
    pub antar: i64,
    /// Biaya (sewa + tambahan + antar) — tidak termasuk deposit.
    pub biaya: i64,
    pub deposit: i64,
    /// Dibayar saat wadah diterima: biaya + deposit (deposit kembali).
    pub bayar: i64,
}

impl SeserahanInput {
    pub fn paket<'a>(&self, k: &'a Konten) -> Option<&'a SeserahanPaket> {
        k.seserahan_paket
            .iter()
            .find(|p| p.slug == self.paket)
            .or_else(|| k.seserahan_paket.iter().find(|p| p.popular))
            .or(k.seserahan_paket.first())
    }
    pub fn antar<'a>(&self, k: &'a Konten) -> Option<&'a SeserahanAntar> {
        k.seserahan_antar.iter().find(|a| a.slug == self.antar).or(k.seserahan_antar.first())
    }
    /// Harga layanan tambahan `price` untuk paket berisi `kotak` wadah.
    pub fn harga_opsi(price: i64, per_kotak: bool, kotak: i64) -> i64 {
        if per_kotak { price.max(0) * kotak } else { price.max(0) }
    }

    pub fn estimasi(&self, k: &Konten) -> SeserahanEst {
        let (sewa, kotak, deposit) = self.paket(k).map(|p| (p.price.max(0), p.kotak.max(0), p.deposit.max(0))).unwrap_or_default();
        let tambahan = k
            .seserahan_opsi
            .iter()
            .filter(|o| self.opsi.contains(&o.slug))
            .map(|o| Self::harga_opsi(o.price, o.per_kotak, kotak))
            .sum();
        let antar = self.antar(k).map(|a| a.price.max(0)).unwrap_or(0);
        let biaya = sewa + tambahan + antar;
        SeserahanEst { kotak, sewa, tambahan, antar, biaya, deposit, bayar: biaya + deposit }
    }
}

/// 25_000 g → "~25,0 kg"
pub fn berat(gram: i64) -> String {
    format!("~{},{} kg", gram / 1000, (gram % 1000) / 100)
}

/// "Rp 4.500.000" atau "Mulai Rp 3.500.000".
pub fn vendor_price(p: &VendorPaket) -> String {
    if p.price_from {
        format!("Mulai {}", rupiah(p.price))
    } else {
        rupiah(p.price)
    }
}

// ── Pilihan formulir yang tetap di kode ────────────────────────────────────

/// (slug, label)
pub const DEKOR_WILAYAH: &[(&str, &str)] = &[
    ("tawangmangu", "Tawangmangu (resort / hutan pinus / villa)"),
    ("ngargoyoso", "Ngargoyoso / Karangpandan (kebun teh / alam)"),
    ("karanganyar", "Karanganyar Kota / Tasikmadu"),
    ("solo", "Solo Kota / Colomadu / Sukoharjo"),
    ("lainnya", "Lainnya (Boyolali / Sragen / Wonogiri)"),
];

pub const MUA_LOKASI: &[(&str, &str)] = &[
    ("tawangmangu", "Tawangmangu / lereng Gunung Lawu"),
    ("karanganyar", "Karanganyar Kota / Ngargoyoso"),
    ("solo", "Solo Kota (Surakarta)"),
    ("sekitar", "Boyolali / Sukoharjo / sekitarnya"),
];

// ── Pesan WhatsApp ─────────────────────────────────────────────────────────

/// Susun teks pesan WA dari query `GET /layanan/wa`. `get` mengembalikan nilai
/// query (sudah dipangkas) atau string kosong.
pub fn pesan_wa(k: &Konten, get: impl Fn(&str) -> String) -> String {
    let baris = |out: &mut String, label: &str, v: String| {
        if !v.is_empty() {
            out.push_str(&format!("\n• {label}: {v}"));
        }
    };
    let label_of = |list: &[(&str, &str)], slug: String| list.iter().find(|x| x.0 == slug).map(|x| x.1.to_string()).unwrap_or_default();
    let vendor_of = |list: &[VendorPaket], slug: String| {
        list.iter()
            .find(|p| p.slug == slug)
            .map(|p| {
                if p.price_from {
                    format!("{} (mulai {})", p.name, rupiah(p.price))
                } else {
                    format!("{} ({})", p.name, rupiah(p.price))
                }
            })
            .unwrap_or_else(|| if slug == "custom" { "Konsep khusus / konsultasi dulu".into() } else { String::new() })
    };

    let mut s = String::from(concat!("Halo admin ", crate::brand!(), ", "));
    match get("layanan").as_str() {
        "cetak" => {
            let input = CetakInput {
                paket: get("paket"),
                qty: get("qty").parse().unwrap_or(k.umum.cetak_bonus_qty.max(CETAK_QTY_MIN)),
                finishing: k
                    .cetak_finishing
                    .iter()
                    .filter(|f| !get(&format!("fin_{}", f.slug)).is_empty())
                    .map(|f| f.slug.clone())
                    .collect(),
                wilayah: get("wilayah"),
            };
            let e = input.estimasi(k);
            let fin: Vec<&str> = k.cetak_finishing.iter().filter(|f| input.finishing.contains(&f.slug)).map(|f| f.name.as_str()).collect();
            s.push_str("saya ingin konsultasi & mengunci harga cetak undangan fisik.\n");
            if let Some(p) = input.paket(k) {
                baris(&mut s, "Paket", format!("{} ({}/pcs)", p.name, rupiah(p.price)));
            }
            baris(&mut s, "Jumlah", format!("{} pcs", e.qty));
            baris(&mut s, "Finishing", if fin.is_empty() { "Standar".into() } else { fin.join(", ") });
            if let Some(w) = input.wilayah(k) {
                baris(&mut s, "Tujuan kirim", format!("{} ({})", w.name, w.kurir));
            }
            baris(&mut s, "Biaya cetak", rupiah(e.cetak));
            baris(&mut s, "Ongkir", format!("{} ({})", rupiah(e.ongkir), berat(e.gram)));
            baris(&mut s, "Total estimasi", rupiah(e.total));
            if e.bonus {
                s.push_str("\n\nSaya juga ingin klaim bonus undangan digital 1 tahun.");
            }
            s.push_str("\n\nMohon info harga final & jadwal produksinya. Terima kasih.");
        }
        "sampel" => s.push_str(
            "saya ingin memesan Wedding Sample Kit (sampel cetak, foil, jenis kertas & wax seal) untuk dikirim ke alamat saya.",
        ),
        l @ ("mua" | "dekorasi") => {
            let mua = l == "mua";
            s.push_str(if mua {
                "saya ingin cek tanggal & reservasi MUA / rias pengantin.\n"
            } else {
                "saya ingin booking dekorasi & sound system serta jadwal survey lokasi gratis.\n"
            });
            baris(&mut s, "Nama", get("nama"));
            baris(&mut s, "WhatsApp", get("wa"));
            baris(&mut s, "Tanggal acara", super::fmt::tanggal_panjang(&get("tanggal")));
            if mua {
                baris(&mut s, "Lokasi", label_of(MUA_LOKASI, get("lokasi")));
                baris(&mut s, "Paket", vendor_of(&k.mua_paket, get("paket")));
            } else {
                baris(&mut s, "Wilayah venue", label_of(DEKOR_WILAYAH, get("lokasi")));
                baris(&mut s, "Paket", vendor_of(&k.dekor_paket, get("paket")));
            }
            baris(&mut s, "Catatan", get("catatan"));
        }
        "seserahan" => {
            let input = SeserahanInput {
                paket: get("paket"),
                opsi: k.seserahan_opsi.iter().filter(|o| !get(&format!("opsi_{}", o.slug)).is_empty()).map(|o| o.slug.clone()).collect(),
                antar: get("antar"),
            };
            let e = input.estimasi(k);
            s.push_str("saya ingin cek ketersediaan & booking sewa seserahan.\n");
            baris(&mut s, "Nama", get("nama"));
            baris(&mut s, "WhatsApp", get("wa"));
            baris(&mut s, "Tanggal acara", super::fmt::tanggal_panjang(&get("tanggal")));
            if let Some(p) = input.paket(k) {
                baris(&mut s, "Paket", format!("{} – {} kotak ({})", p.name, p.kotak, rupiah(p.price)));
            }
            let opsi: Vec<String> = k
                .seserahan_opsi
                .iter()
                .filter(|o| input.opsi.contains(&o.slug))
                .map(|o| format!("{} ({})", o.name, rupiah(SeserahanInput::harga_opsi(o.price, o.per_kotak, e.kotak))))
                .collect();
            baris(&mut s, "Tambahan", opsi.join(", "));
            if let Some(a) = input.antar(k) {
                baris(&mut s, "Antar-jemput", format!("{} ({})", a.name, rupiah(a.price)));
            }
            baris(&mut s, "Total biaya", rupiah(e.biaya));
            baris(&mut s, "Deposit (kembali)", rupiah(e.deposit));
            baris(&mut s, "Catatan", get("catatan"));
            s.push_str("\n\nMohon info ketersediaan tanggal & cara pembayaran DP. Terima kasih.");
        }
        "custom" => s.push_str(
            "saya tertarik paket Custom Desain undangan digital. Boleh minta info portofolio, estimasi harga & waktu pengerjaan?",
        ),
        _ => s.push_str("saya ingin bertanya tentang layanan pernikahan (cetak undangan, dekorasi, MUA, dan sewa seserahan)."),
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(paket: &str, qty: i64, fin: &[&str], wilayah: &str) -> CetakInput {
        CetakInput { paket: paket.into(), qty, finishing: fin.iter().map(|s| s.to_string()).collect(), wilayah: wilayah.into() }
    }

    #[test]
    fn estimasi_sesuai_desain() {
        // Desain: Hardcover 500 pcs ke Jawa → Rp 3.750.000 + 25 kg × 12.000.
        let k = Konten::default();
        let e = input("hardcover", 500, &[], "jawa").estimasi(&k);
        assert_eq!((e.cetak, e.gram, e.ongkir, e.total), (3_750_000, 25_000, 300_000, 4_050_000));
        assert!(e.bonus && e.cargo);
        assert_eq!(berat(e.gram), "~25,0 kg");
    }

    #[test]
    fn estimasi_minimum_dan_finishing() {
        let k = Konten::default();
        // Softcover min 300 pcs: 200 dinaikkan; wax +1.200 & +4 g/pcs.
        let e = input("softcover", 200, &["wax"], "maluku-papua").estimasi(&k);
        assert_eq!((e.qty, e.per_pcs, e.gram, e.ongkir), (300, 4_700, 300 * 26, 8 * 65_000));
        assert!(!e.bonus && !e.cargo);
        // Slug tak dikenal tidak panik; qty dibatasi.
        assert_eq!(input("x", 99_999, &["x"], "x").estimasi(&k).qty, CETAK_QTY_MAX);
        // Harga dari konten admin, bukan konstanta.
        let mut k2 = k.clone();
        k2.cetak_paket[1].price = 10_000;
        assert_eq!(input("hardcover", 500, &[], "jawa").estimasi(&k2).cetak, 5_000_000);
    }

    #[test]
    fn pesan() {
        let k = Konten::default();
        let q = |pairs: &'static [(&'static str, &'static str)]| {
            move |key: &str| pairs.iter().find(|p| p.0 == key).map(|p| p.1.to_string()).unwrap_or_default()
        };
        let m = pesan_wa(&k, q(&[("layanan", "cetak"), ("paket", "hardcover"), ("qty", "500"), ("wilayah", "jawa"), ("fin_wax", "1")]));
        assert!(m.contains("Hardcover Heritage") && m.contains("Real Wax Seal") && m.contains("bonus"), "{m}");
        let m = pesan_wa(&k, q(&[("layanan", "mua"), ("nama", "Roro"), ("paket", "all-in"), ("lokasi", "solo"), ("tanggal", "2026-10-24")]));
        assert!(m.contains("• Nama: Roro") && m.contains("Rp 6.800.000") && m.contains("Solo Kota") && m.contains("24 Oktober 2026"), "{m}");
        assert!(!m.contains("Catatan"));
        assert!(pesan_wa(&k, q(&[("layanan", "dekorasi"), ("paket", "sound")])).contains("mulai Rp 3.500.000"));
        assert!(pesan_wa(&k, q(&[])).contains("layanan pernikahan"));
    }

    #[test]
    fn seserahan() {
        let k = Konten::default();
        // Gold Mirror 9 kotak: 550.000 + hias 35.000×9 + pita 75.000 + Solo 100.000.
        let i = SeserahanInput { paket: "akrilik-gold".into(), opsi: vec!["hias".into(), "pita".into(), "x".into()], antar: "solo".into() };
        let e = i.estimasi(&k);
        assert_eq!((e.kotak, e.tambahan, e.biaya, e.deposit, e.bayar), (9, 390_000, 1_040_000, 500_000, 1_540_000));
        // Paket tak dikenal → paket favorit; antar tak dikenal → pilihan pertama (Rp 0).
        let e = SeserahanInput { paket: "?".into(), opsi: vec![], antar: "?".into() }.estimasi(&k);
        assert_eq!((e.sewa, e.antar), (550_000, 0));
        // Pesan WA dihitung ulang di server dari query.
        let q = |key: &str| {
            [("layanan", "seserahan"), ("nama", "Salsa"), ("paket", "kayu-jati-ukir"), ("opsi_bunga", "1"), ("antar", "tawangmangu"), ("tanggal", "2026-11-07")]
                .iter()
                .find(|p| p.0 == key)
                .map(|p| p.1.to_string())
                .unwrap_or_default()
        };
        let m = pesan_wa(&k, q);
        assert!(m.contains("Kayu Jati Ukir Jawa – 7 kotak") && m.contains("Bunga & daun segar (Rp 175.000)"), "{m}");
        assert!(m.contains("Total biaya: Rp 875.000") && m.contains("Deposit (kembali): Rp 750.000") && m.contains("7 November 2026"), "{m}");
    }
}
