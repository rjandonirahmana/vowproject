//! pages/hub3d.rs — "HUB 3D" beranda: panggung gulir (sticky) dengan HP 3D yang
//! berputar & lima panel mengikuti alur pesan yang SEBENARNYA (lihat /buat):
//! Pilih tema → Isi data & pratinjau gratis → Pilih paket & bayar (undangan
//! baru terbuka untuk tamu) → Sebarkan tautan → Tamu RSVP, amplop & hadir.
//! Dulu "Sebarkan" datang sebelum bayar & langkah terakhir berisi layanan cetak
//! — bertentangan dengan "bayar saat siap disebar" & judul "sampai tamu hadir".
//!
//! SSR murni: tanpa signal, tanpa JS, tanpa WASM, tanpa gambar. Seluruh gerak =
//! CSS `view-timeline` (style/main.css, blok "HUB 3D"; hanya transform/opacity).
//! Tanpa dukungan scroll-timeline / gerak dikurangi → lima kartu biasa
//! (isi tetap ada di HTML untuk SEO, pembaca layar, & keyboard).

use leptos::prelude::*;

use crate::web::seo::SITE_URL;
use crate::web::themes::DEMO_SLUG;

/// (judul, uraian, poin 1, poin 2, label tombol, tautan — kosong = undangan demo)
const HUB: [(&str, &str, &str, &str, &str, &str); 5] = [
    ("Pilih tema yang terasa seperti Anda",
     "Tema adat Jawa & Nusantara dengan animasi pembuka. Coba dulu di layar HP sebelum memutuskan.",
     "Pratinjau langsung seperti undangan sungguhan", "Coba gratis, bayar saat siap disebar",
     "Lihat katalog tema", "#katalog"),
    ("Isi data & lihat pratinjau gratis",
     "Nama mempelai, acara, foto, dan musik latar — hasilnya langsung tampil di HP. Pratinjau ini khusus Anda, belum terlihat tamu.",
     "Jadi dalam 5 menit", "Musik latar, galeri foto & cerita",
     "Mulai buat undangan", "/buat"),
    ("Pilih paket & bayar saat sudah puas",
     "Sekali bayar tanpa langganan, paling lambat 24 jam setelah pesanan dibuat. Setelah pembayaran dikonfirmasi, undangan terbuka untuk tamu.",
     "Transfer bank, unggah bukti di dashboard", "Tanpa biaya bulanan",
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

/// Rentang gulir (% dari panggung) tiap panel — saling tumpang 4% untuk cross-fade.
const RENTANG: [&str; 5] = ["--s:0%;--e:24%", "--s:16%;--e:44%", "--s:36%;--e:64%", "--s:56%;--e:84%", "--s:76%;--e:100%"];

#[component]
pub fn Hub3d() -> impl IntoView {
    view! {
        <section class="hub3d" id="hub" aria-labelledby="hub3d-judul">
            <div class="hub3d__head">
                <span class="chip chip--soft">"Satu platform, lima langkah"</span>
                <h2 id="hub3d-judul">"Dari memilih tema sampai tamu hadir"</h2>
                <a class="hub3d__lewati" href="#katalog">"Lewati ke katalog ↓"</a>
            </div>
            <div class="hub3d__rel">
                <div class="hub3d__st">
                    <div class="hub3d__tata">
                        <div class="hub3d__teks">
                            {HUB.iter().enumerate().map(|(i, (judul, uraian, p1, p2, cta, href))| view! {
                                <article class="hub3d__t" class:is-awal=(i == 0) class:is-akhir=(i == 4) style=RENTANG[i]>
                                    <span class="hub3d__no">{format!("{:02} / 05", i + 1)}</span>
                                    <h3>{*judul}</h3>
                                    <p>{*uraian}</p>
                                    <ul>
                                        <li>{*p1}</li>
                                        <li>{*p2}</li>
                                    </ul>
                                    <a class="btn btn--primary btn--sm" href=if href.is_empty() { format!("/u/{DEMO_SLUG}") } else { href.to_string() }>{*cta}</a>
                                </article>
                            }).collect_view()}
                        </div>
                        <div class="hub3d__dunia" aria-hidden="true">
                            <i class="hub3d__kn hub3d__kn--1"></i>
                            <i class="hub3d__kn hub3d__kn--2"></i>
                            <i class="hub3d__kn hub3d__kn--3"></i>
                            <div class="hub3d__hp">
                                <div class="hub3d__l is-awal" style="--s:0%;--e:24%">
                                    <small class="m-tag">"contoh"</small>
                                    <div class="m-grid"><i></i><i></i><i></i><i></i><i></i><i></i></div>
                                </div>
                                <div class="hub3d__l" style="--s:16%;--e:44%">
                                    <small class="m-tag">"contoh"</small>
                                    <b class="m-mono">"A & B"</b>
                                    <span class="m-baris"></span><span class="m-baris"></span><span class="m-baris m-baris--pendek"></span>
                                    <span class="m-chip">"Pratinjau — khusus Anda"</span>
                                </div>
                                <div class="hub3d__l" style="--s:36%;--e:64%">
                                    <small class="m-tag">"contoh"</small>
                                    <span class="m-link">"Paket pilihan Anda"</span>
                                    <span class="m-baris"></span><span class="m-baris m-baris--pendek"></span>
                                    <span class="m-btn">"Bayar"</span>
                                    <span class="m-chip">"Undangan aktif untuk tamu"</span>
                                </div>
                                <div class="hub3d__l" style="--s:56%;--e:84%">
                                    <small class="m-tag">"contoh"</small>
                                    <span class="m-link">{format!("{}/u/nama-kita", SITE_URL.trim_start_matches("https://"))}</span>
                                    <span class="m-btn">"Bagikan"</span>
                                </div>
                                <div class="hub3d__l is-akhir" style="--s:76%;--e:100%">
                                    <small class="m-tag">"contoh"</small>
                                    <i class="m-qr"></i>
                                    <span class="m-chip">"Hadir"</span><span class="m-chip">"Amplop"</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}
