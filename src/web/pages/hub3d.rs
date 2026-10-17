//! pages/hub3d.rs — "DUNIA 3D" beranda: jalan-jalan di dunia 3D yang
//! digerakkan gulir (sticky), terinspirasi belajarweb.dev/#hub. Lima POS di
//! sepanjang jalan setapak = lima langkah alur pesan yang SEBENARNYA (lihat
//! /buat): Pilih tema → Isi data & pratinjau gratis → Pilih paket & bayar →
//! Sebarkan tautan → Tamu RSVP, amplop & hadir. Kamera berjalan dari pos ke
//! pos; panel teks di atas dunia berganti mengikuti pos.
//!
//! Dunia = CSS 3D sungguhan (perspective + preserve-3d): bangunan = papan SVG
//! panggung (disajikan /panggung/{nama}.svg, dari web/panggung.rs — tak ada
//! aset ganda), batu jalan & pohon = elemen datar di ruang 3D. Gerak kamera =
//! satu animasi `view-timeline` pada .dunia3d__dunia (style/main.css blok
//! "DUNIA 3D"; hanya transform/opacity → compositor).
//!
//! INTERAKTIF tanpa WASM & hampir tanpa JS: suasana pagi/senja/malam (radio +
//! :has), balon yang terbang bila diklik (checkbox), RSVP contoh yang memicu
//! konfeti di pos terakhir, peta pos (tautan jangkar), papan penunjuk di dunia.
//! Satu-satunya JS: arah pandang mengikuti kursor (global.js `[data-dunia]`,
//! hanya menulis --mx/--my).
//!
//! Tanpa dukungan scroll-timeline / gerak dikurangi → lima kartu biasa
//! (isi tetap ada di HTML untuk SEO, pembaca layar, & keyboard).

use leptos::prelude::*;

use crate::web::seo::SITE_URL;
use crate::web::themes::DEMO_SLUG;

/// (judul, uraian, poin 1, poin 2, label tombol, tautan — kosong = undangan demo)
const HUB: [(&str, &str, &str, &str, &str, &str); 5] = [
    ("Pilih tema yang terasa seperti Anda",
     "Tema adat Jawa & Nusantara, tema 3D, hingga budaya dunia — lengkap dengan animasi pembuka. Coba dulu di layar HP sebelum memutuskan.",
     "Pratinjau langsung seperti undangan sungguhan", "Coba gratis, bayar saat siap disebar",
     "Lihat katalog tema", "#katalog"),
    ("Isi data & lihat pratinjau gratis",
     "Nama mempelai, acara, foto, dan musik latar — hasilnya langsung tampil di HP. Pratinjau ini khusus Anda, belum terlihat tamu.",
     "Jadi dalam 5 menit", "Musik latar, galeri foto & cerita",
     "Mulai buat undangan", "/buat"),
    ("Pilih paket & bayar saat sudah puas",
     "Sekali bayar tanpa langganan, paling lambat 24 jam setelah pesanan dibuat. Setelah pembayaran dikonfirmasi, undangan terbuka untuk tamu.",
     "Transfer, unggah bukti di dashboard", "Tanpa biaya bulanan",
     "Lihat paket & harga", "/paket"),
    ("Sebarkan lewat satu tautan",
     "Bagikan tautan undangan lewat WhatsApp atau media sosial — tampil sempurna di HP, tablet, maupun desktop.",
     "Mobile optimal", "Panduan langkah demi langkah",
     "Baca panduan", "/panduan"),
    ("Tamu RSVP, kirim amplop & hadir",
     "Tamu mengonfirmasi kehadiran dan mengirim amplop digital dari undangan. Di hari acara, buku tamu QR mencatat siapa yang datang.",
     "RSVP & amplop digital", "QR check-in di hari acara",
     "Coba undangan contoh", ""),
];

/// Rentang gulir (% dari panggung) tiap panel — saling tumpang 4% untuk
/// cross-fade. Pusat panel = saat kamera DIAM di pos itu (keyframes
/// `dunia-kamera` di main.css: diam 0–12, 24–36, 44–56, 64–76, 84–100).
const RENTANG: [&str; 5] = ["--s:0%;--e:24%", "--s:16%;--e:44%", "--s:36%;--e:64%", "--s:56%;--e:84%", "--s:76%;--e:100%"];

/// Pos di dunia (x, z dalam px; z negatif = menjauh), papan bangunan
/// (/panggung/{nama}.svg), lalu `--kaki` (tinggi kaki bangunan di kanvas, 0–1)
/// & `--potong` (bagian kanvas yang ditampilkan dari atas — membuang alas tanah
/// bawaan beberapa lapisan panggung). HARUS sama dengan keyframes
/// `dunia-kamera` (kamera di pos i = translate3d(-x, 0, -z - 500px)).
const POS: [(i32, i32, &str, &str); 5] = [
    (-240, 0, "jawa-utama", "--kaki:.705;--potong:.716"),
    (260, -1300, "paviliun-utama", "--kaki:.705;--potong:.716"),
    (-260, -2600, "teh-utama", "--kaki:.765;--potong:.8"),
    (240, -3900, "balon-utama", "--kaki:.97;--potong:1"),
    (0, -5200, "kastil-utama", "--kaki:.765;--potong:.8"),
];

/// translateZ dunia (px) pada tiap kunci `dunia-kamera` (% gulir) — salinan
/// keyframes di main.css, untuk menghitung kapan benda dilewati kamera.
const KAMERA: [(f32, f32); 14] = [
    (0.0, -500.0), (12.0, -500.0), (18.0, 150.0), (24.0, 800.0), (36.0, 800.0), (40.0, 1450.0), (44.0, 2100.0),
    (56.0, 2100.0), (60.0, 2750.0), (64.0, 3400.0), (76.0, 3400.0), (80.0, 4050.0), (84.0, 4700.0), (100.0, 4700.0),
];

/// `--off` benda di kedalaman `z`: % gulir saat benda tiba ±450px di depan
/// mata (perspective 900px) → disembunyikan sebelum melewati mata (benda di
/// belakang mata digambar terbalik/raksasa di sebagian browser). Benda yang
/// tak pernah dilewati = "" (CSS memakai --off bawaan 200%: tak pernah hilang).
fn off(z: f32) -> String {
    let t = 450.0 - z;
    KAMERA
        .windows(2)
        .find(|w| w[0].1 < t && t <= w[1].1)
        .map(|w| {
            let ((p0, t0), (p1, t1)) = (w[0], w[1]);
            format!(";--off:{:.1}%", p0 + (p1 - p0) * (t - t0) / (t1 - t0))
        })
        .unwrap_or_default()
}

/// Papan penunjuk di tiap pos (teks pendek di dunia).
const PAPAN: [&str; 5] = ["Galeri Tema", "Studio Pratinjau", "Loket Paket", "Menara Kabar", "Pelaminan"];

/// Titik-titik jalan setapak (x, z) yang melengkung melewati pos: kurva
/// Catmull-Rom sederhana di antara pos + perpanjangan ke arah mata.
fn jalan() -> Vec<(f32, f32)> {
    let mut titik: Vec<(f32, f32)> = vec![(-120.0, 700.0)];
    titik.extend(POS.iter().map(|(x, z, _, _)| (*x as f32, *z as f32)));
    titik.push((0.0, -5900.0));
    let mut out = Vec::new();
    for i in 0..titik.len() - 1 {
        let p0 = titik[i.saturating_sub(1)];
        let (p1, p2) = (titik[i], titik[i + 1]);
        let p3 = titik[(i + 2).min(titik.len() - 1)];
        let n = (((p2.1 - p1.1).abs()) / 170.0).ceil().max(2.0) as usize;
        for k in 0..n {
            let t = k as f32 / n as f32;
            let (t2, t3) = (t * t, t * t * t);
            let c = |a: f32, b: f32, c: f32, d: f32| {
                0.5 * (2.0 * b + (-a + c) * t + (2.0 * a - 5.0 * b + 4.0 * c - d) * t2 + (-a + 3.0 * b - 3.0 * c + d) * t3)
            };
            out.push((c(p0.0, p1.0, p2.0, p3.0), c(p0.1, p1.1, p2.1, p3.1)));
        }
    }
    out
}

#[component]
pub fn Hub3d() -> impl IntoView {
    let batu = jalan();
    // Pohon & tiang lentera berselang-seling di kiri-kanan jalan, tiap ±3 batu.
    let tepi: Vec<(f32, f32, bool, usize)> = batu
        .iter()
        .enumerate()
        .filter(|(i, _)| i % 3 == 1)
        // Tak ada pohon/tiang di antara mata & pos saat kamera diam di sana
        // (mata = z pos + 500 + perspective 900) — dulu menutupi bangunan.
        .filter(|(_, (_, z))| !POS.iter().any(|(_, zp, _, _)| (0.0..1400.0).contains(&(z - *zp as f32))))
        .map(|(i, (x, z))| {
            let kanan = (i / 3) % 2 == 0;
            let jarak = 300.0 + ((i * 37) % 120) as f32;
            (if kanan { x + jarak } else { x - jarak }, *z, kanan, i)
        })
        .collect();
    view! {
        <section class="hub3d" id="hub" aria-labelledby="hub3d-judul">
            <div class="hub3d__head">
                <span class="chip chip--soft">"Satu platform, lima langkah"</span>
                <h2 id="hub3d-judul">"Jelajahi dunia undangan Anda"</h2>
                <p class="hub3d__ajak">"Gulir untuk berjalan dari pos ke pos — gerakkan kursor untuk melihat sekeliling, klik balon, dan ganti suasana."</p>
                <a class="hub3d__lewati" href="#katalog">"Lewati ke katalog ↓"</a>
            </div>
            <div class="hub3d__rel">
                // Jangkar tiap pos (peta & tautan langsung) — di tengah rentang diam kamera.
                {["6%", "30%", "50%", "70%", "92%"].iter().enumerate().map(|(i, top)| view! {
                    <span class="hub3d__jangkar" id=format!("hub-{}", i + 1) style=format!("top:{top}")></span>
                }).collect_view()}
                <div class="hub3d__st" data-dunia>
                    // Suasana (radio di .dunia3d__alat) dibaca lewat .hub3d:has(...).
                    <div class="dunia3d" aria-hidden="true">
                        <i class="dunia3d__langit"></i>
                        <i class="dunia3d__bintang"></i>
                        <i class="dunia3d__surya"></i>
                        <i class="dunia3d__gunung"></i>
                        <i class="dunia3d__tanah"></i>
                        <div class="dunia3d__mata">
                            <div class="dunia3d__dunia">
                                {batu.iter().map(|(x, z)| view! {
                                    <i class="dunia3d__batu" style=format!("--x:{x:.0}px;--z:{z:.0}px{}", off(*z))></i>
                                }).collect_view()}
                                {tepi.into_iter().map(|(x, z, kanan, i)| {
                                    let lentera = i % 2 == 0;
                                    view! {
                                        <i class=if lentera { "dunia3d__tiang" } else if kanan { "dunia3d__pohon dunia3d__pohon--b" } else { "dunia3d__pohon" }
                                            style=format!("--x:{x:.0}px;--z:{z:.0}px{}", off(z))></i>
                                    }
                                }).collect_view()}
                                {POS.iter().enumerate().map(|(i, (x, z, papan, kaki))| view! {
                                    <div class=format!("dunia3d__pos dunia3d__pos--{}", i + 1) style=format!("--x:{x}px;--z:{z}px;{kaki}{}", off(*z as f32))>
                                        <i class="dunia3d__bb" style=format!("background-image:url(/panggung/{papan}.svg)")></i>
                                        <i class="dunia3d__bayang"></i>
                                        <span class="dunia3d__papan">{PAPAN[i]}</span>
                                        {match i {
                                            // Galeri: bingkai tema melayang (warna 4 keluarga tema).
                                            0 => view! {
                                                <span class="dunia3d__bingkai dunia3d__bingkai--1"><b>"Adat"</b></span>
                                                <span class="dunia3d__bingkai dunia3d__bingkai--2"><b>"3D"</b></span>
                                                <span class="dunia3d__bingkai dunia3d__bingkai--3"><b>"Dunia"</b></span>
                                            }.into_any(),
                                            // Studio: HP raksasa dengan layar pratinjau bergantian.
                                            1 => view! {
                                                <span class="dunia3d__hp"><i></i><b>"A & B"</b><small>"Pratinjau — khusus Anda"</small></span>
                                            }.into_any(),
                                            // Loket: koin & struk melayang.
                                            2 => view! {
                                                <span class="dunia3d__koin dunia3d__koin--1"></span>
                                                <span class="dunia3d__koin dunia3d__koin--2"></span>
                                                <span class="dunia3d__struk">"Lunas ✓"</span>
                                            }.into_any(),
                                            // Menara kabar: merpati pembawa tautan.
                                            3 => view! {
                                                <span class="dunia3d__merpati dunia3d__merpati--1"></span>
                                                <span class="dunia3d__merpati dunia3d__merpati--2"></span>
                                                <span class="dunia3d__merpati dunia3d__merpati--3"></span>
                                                <span class="dunia3d__tautan">{format!("{}/u/nama-kita", SITE_URL.trim_start_matches("https://"))}</span>
                                            }.into_any(),
                                            // Pelaminan: lampion + konfeti (muncul saat RSVP contoh "Hadir").
                                            _ => view! {
                                                <span class="dunia3d__lampion dunia3d__lampion--1"></span>
                                                <span class="dunia3d__lampion dunia3d__lampion--2"></span>
                                                <span class="dunia3d__lampion dunia3d__lampion--3"></span>
                                                <span class="dunia3d__konfeti">{(0..14).map(|_| view! { <i></i> }).collect_view()}</span>
                                            }.into_any(),
                                        }}
                                    </div>
                                }).collect_view()}
                            </div>
                        </div>
                        <i class="dunia3d__kabut"></i>
                    </div>
                    // Balon yang bisa diklik: di luar ruang 3D (hit-test pasti,
                    // tak tertutup dunia) — melayang di langit, terbang saat diklik.
                    <div class="dunia3d__langitan">
                        {[("#e98aa6", "Balon merah muda"), ("#f3b562", "Balon jingga"), ("#8fb7e8", "Balon biru")].iter().enumerate().map(|(i, (warna, label))| view! {
                            <input class="dunia3d__cek" type="checkbox" id=format!("dunia-balon-{i}") />
                            <label class=format!("dunia3d__balon dunia3d__balon--{}", i + 1) for=format!("dunia-balon-{i}") title="Klik untuk menerbangkan" style=format!("--w:{warna}")>
                                <span class="sr-only">{format!("{label} — terbangkan")}</span>
                            </label>
                        }).collect_view()}
                        <p class="dunia3d__pesan">"Kabar bahagia sudah terbang ke seluruh penjuru 🎈"</p>
                    </div>
                    <div class="hub3d__tata">
                        <div class="hub3d__teks">
                            {HUB.iter().enumerate().map(|(i, (judul, uraian, p1, p2, cta, href))| view! {
                                <article class="hub3d__t" class:is-awal=(i == 0) class:is-akhir=(i == 4) style=RENTANG[i]>
                                    <span class="hub3d__no">{format!("{:02} / 05 · {}", i + 1, PAPAN[i])}</span>
                                    <h3>{*judul}</h3>
                                    <p>{*uraian}</p>
                                    <ul>
                                        <li>{*p1}</li>
                                        <li>{*p2}</li>
                                    </ul>
                                    {(i == 4).then(|| view! {
                                        // RSVP contoh (CSS saja): "Hadir" menyalakan konfeti di pelaminan.
                                        <fieldset class="dunia3d__rsvp">
                                            <legend>"Coba jadi tamu:"</legend>
                                            <label><input type="radio" name="dunia-rsvp" value="hadir" />"Hadir"</label>
                                            <label><input type="radio" name="dunia-rsvp" value="tidak" />"Berhalangan"</label>
                                            <output class="dunia3d__rsvp-msg"></output>
                                        </fieldset>
                                    })}
                                    <a class="btn btn--primary btn--sm" href=if href.is_empty() { format!("/u/{DEMO_SLUG}") } else { href.to_string() }>{*cta}</a>
                                </article>
                            }).collect_view()}
                        </div>
                    </div>
                    <div class="dunia3d__alat">
                        <fieldset class="dunia3d__suasana">
                            <legend class="sr-only">"Suasana dunia"</legend>
                            {[("pagi", "Pagi", true), ("senja", "Senja", false), ("malam", "Malam", false)].iter().map(|(v, l, on)| view! {
                                <label><input type="radio" name="dunia-suasana" value=*v checked=*on />{*l}</label>
                            }).collect_view()}
                        </fieldset>
                        <nav class="dunia3d__peta" aria-label="Pos dunia">
                            {PAPAN.iter().enumerate().map(|(i, p)| view! {
                                <a href=format!("#hub-{}", i + 1) title=*p style=RENTANG[i]><span class="sr-only">{*p}</span></a>
                            }).collect_view()}
                        </nav>
                    </div>
                </div>
            </div>
        </section>
    }
}
