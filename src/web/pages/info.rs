//! pages/info.rs — halaman informasi statis: /paket (harga & fitur),
//! /panduan (cara pakai + FAQ), /privasi, /syarat.
//!
//! Teks privasi & syarat adalah draf umum — TINJAU sebelum situs dipublikasikan.

use leptos::prelude::*;
use crate::web::seo::Seo;

use crate::web::api::get_contact;
use crate::web::fmt::{self, rupiah};
use crate::web::icons::Icon;
use crate::web::konten::Konten;

use super::{SiteFooter, SiteHeader};

/// Kerangka halaman info: header situs + judul + isi + footer.
#[component]
fn InfoShell(
    active: &'static str,
    title: &'static str,
    eyebrow: &'static str,
    lead: &'static str,
    /// Path kanonik halaman (mis. "/paket").
    path: &'static str,
    /// Judul untuk Google (bawaan: "{title} — merek").
    #[prop(optional)] seo_title: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let full = seo_title.map(str::to_string).unwrap_or_else(|| format!(concat!("{} — ", crate::brand!()), title));
    view! {
        <Seo title=full description=lead path=path />
        <div class="site">
            <SiteHeader active=active />
            <section class="info-hero">
                <span class="chip chip--soft">{eyebrow}</span>
                <h1>{title}</h1>
                <p>{lead}</p>
            </section>
            <div class="info-body">{children()}</div>
            <SiteFooter />
        </div>
    }
}

/// Tombol konsultasi WA admin (ADMIN_WHATSAPP); tak tampil bila tak diset.
#[component]
fn ConsultButton(#[prop(optional)] text: &'static str) -> impl IntoView {
    let contact = Resource::new(|| (), |_| get_contact());
    let text = if text.is_empty() { "Konsultasi via WhatsApp" } else { text };
    view! {
        <Suspense fallback=|| ()>
            {move || contact.get().and_then(|r| r.ok()).filter(|wa| !wa.is_empty()).map(|wa| {
                let msg = fmt::url_encode(concat!("Halo admin ", crate::brand!(), ", saya ingin bertanya tentang undangan digital."));
                view! {
                    <a class="btn btn--primary" href=format!("https://wa.me/{wa}?text={msg}") target="_blank" rel="noopener">
                        <Icon name="support_agent" />{text}
                    </a>
                }
            })}
        </Suspense>
    }
}

// ── /paket ─────────────────────────────────────────────────────────────────

/// (fitur, silver, gold, platinum) — "✓" / "—" / teks bebas.
const FEATURES: &[(&str, &str, &str, &str)] = &[
    ("Jumlah nama tamu (link pribadi)", "250", "Tanpa batas", "Tanpa batas"),
    ("Masa aktif undangan", "6 bulan", "Selamanya", "Selamanya"),
    ("Semua tema katalog", "✓", "✓", "✓"),
    ("Hitung mundur & Google Calendar", "✓", "✓", "✓"),
    ("Peta lokasi & petunjuk arah", "✓", "✓", "✓"),
    ("RSVP & buku ucapan doa", "✓", "✓", "✓"),
    ("Dashboard pengantin & ekspor CSV", "✓", "✓", "✓"),
    ("Amplop digital (salin rekening)", "—", "✓", "✓"),
    ("QR check-in di meja penerima tamu", "—", "✓", "✓"),
    ("Musik latar pilihan / upload MP3", "Pilihan", "✓", "✓"),
    ("Foto mempelai", "—", "✓", "✓"),
    ("Domain pribadi (.wedding)", "Add-on", "Add-on", "✓"),
    ("Revisi data", "2x", "5x", "Tanpa batas"),
    ("Bantuan input data & blast WhatsApp", "—", "—", "✓"),
];

#[component]
pub fn PaketPage() -> impl IntoView {
    view! {
        <InfoShell
            active="paket"
            path="/paket"
            seo_title=concat!("Paket & Harga Undangan Online Pernikahan | ", crate::brand!())
            title="Paket & Harga"
            eyebrow="Transparan & Fleksibel"
            lead="Satu kali bayar, tanpa biaya langganan. Semua tema katalog bisa dipakai di paket mana pun — coba & pratinjau gratis, bayar saat undangan siap disebar."
        >
            <div class="trial-steps">
                <span><b>"1"</b>"Pilih tema & isi data"</span>
                <Icon name="arrow_forward" />
                <span><b>"2"</b>"Pratinjau gratis (khusus Anda)"</span>
                <Icon name="arrow_forward" />
                <span><b>"3"</b>"Bayar maks. 24 jam — undangan terbuka untuk tamu"</span>
            </div>
            <super::layanan::WithKonten view=paket_body />

            <section class="bundle card">
                <div>
                    <p class="eyebrow eyebrow--gold">"Lengkapi Hari Bahagia"</p>
                    <h2>"Satu Tim untuk Undangan, Cetak, Rias, Dekorasi & Seserahan"</h2>
                    <p class="muted">"Pesan beberapa layanan sekaligus lewat satu admin — jadwal lebih mudah dikoordinasikan dan tanyakan harga bundling khususnya."</p>
                </div>
                <div class="bundle__grid">
                    <a class="bundle__item" href="/cetak">
                        <Icon name="mail" /><b>"Cetak Undangan Fisik"</b>
                        <small>"Bonus undangan digital 1 tahun untuk cetak ≥500 pcs"</small>
                    </a>
                    <a class="bundle__item" href="/mua">
                        <Icon name="auto_awesome" /><b>"MUA & Rias Pengantin"</b>
                        <small>"Solo Putri, Basahan, hingga modern glowing bride"</small>
                    </a>
                    <a class="bundle__item" href="/dekorasi">
                        <Icon name="local_florist" /><b>"Dekorasi & Sound System"</b>
                        <small>"Survey lokasi gratis Tawangmangu – Solo Raya"</small>
                    </a>
                    <a class="bundle__item" href="/seserahan">
                        <Icon name="redeem" /><b>"Sewa Seserahan"</b>
                        <small>"Kotak akrilik, kayu ukir & rotan + jasa hias"</small>
                    </a>
                </div>
            </section>

            <section class="card info-card">
                <h2>"Pertanyaan Seputar Harga"</h2>
                <Faq items=&[
                    ("Apakah harga berbeda per tema?", "Tidak. Semua tema katalog termasuk di setiap paket — harga hanya ditentukan paket & add-on yang Anda pilih."),
                    ("Bisa dicoba dulu sebelum bayar?", "Bisa. Setelah formulir dikirim, Anda bisa melihat pratinjau lengkap dari halaman Kelola. Selama belum dibayar, tautan undangan TERKUNCI untuk tamu. Bila pembayaran belum dikonfirmasi admin dalam 24 jam, undangan beserta foto & lagu yang diunggah dihapus otomatis."),
                    ("Apakah ada biaya bulanan?", "Tidak. Harga paket dibayar sekali. Paket Gold & Platinum aktif selamanya; Paket Silver aktif 6 bulan."),
                    ("Bisa ganti tema setelah membayar?", "Bisa. Hubungi admin via WhatsApp, tema diganti tanpa biaya tambahan dan seluruh data tamu serta ucapan tetap tersimpan."),
                    ("Bisa upgrade paket?", "Bisa kapan saja — cukup bayar selisih harga paket."),
                    ("Bagaimana jika tidak puas?", "Garansi uang kembali 100% bila undangan tidak bisa dipakai karena kendala teknis dalam 7 hari setelah aktivasi."),
                ] />
            </section>

            <section class="info-cta card card--soft">
                <h2>"Siap membuat undangan?"</h2>
                <p class="muted">"Isi data mempelai dalam ±5 menit, undangan langsung bisa dipratinjau."</p>
                <div class="info-cta__btns">
                    <a class="btn btn--primary" href="/buat"><Icon name="arrow_forward" />"Buat Undangan Sekarang"</a>
                    <ConsultButton />
                </div>
            </section>
        </InfoShell>
    }
}


/// Bagian /paket yang bergantung pada harga (konten admin).
fn paket_body(k: Konten) -> impl IntoView {
    // Tabel perbandingan statis ditulis untuk 3 kolom paket pertama.
    let cols: Vec<_> = k.paket.iter().take(3).cloned().collect();
    view! {
            <div class="pkg-grid">
                {k.paket.clone().into_iter().map(|p| view! {
                    <article class="pkg card" class:pkg--popular=p.popular>
                        {p.popular.then(|| view! { <span class="chip chip--gold pkg__flag">"Paling Populer"</span> })}
                        <h3>{p.name.clone()}</h3>
                        <p class="pkg__price">{rupiah(p.price)}</p>
                        <p class="pkg__period">{p.period.clone()}</p>
                        <ul>
                            {p.features.iter().map(|f| view! { <li><Icon name="check_circle" />{f.clone()}</li> }).collect_view()}
                        </ul>
                        <a class=if p.popular { "btn btn--primary btn--block" } else { "btn btn--soft btn--block" }
                            href=format!("/buat?paket={}", p.slug)>
                            {format!("Pilih {}", p.name)}
                        </a>
                    </article>
                }).collect_view()}
                <article class="pkg card pkg--custom">
                    <span class="chip chip--soft pkg__flag">"Eksklusif"</span>
                    <h3>"Custom Desain"</h3>
                    <p class="pkg__price">{format!("mulai {}", rupiah(k.umum.custom_from))}</p>
                    <p class="pkg__period">"Aktif Selamanya • dikerjakan desainer"</p>
                    <ul>
                        <li><Icon name="check_circle" />"Desain & ornamen dibuat khusus untuk Anda"</li>
                        <li><Icon name="check_circle" />"Warna, huruf & tata letak sesuai konsep acara"</li>
                        <li><Icon name="check_circle" />"Semua fitur Platinum + revisi tanpa batas"</li>
                        <li><Icon name="check_circle" />"Tema privat — tidak dijual ke pasangan lain"</li>
                    </ul>
                    <a class="btn btn--outline btn--block" href="/layanan/wa?layanan=custom" target="_blank" rel="noopener">"Konsultasi Desain"</a>
                </article>
            </div>

            <section class="card info-card" id="fitur">
                <h2>"Perbandingan Fitur Lengkap"</h2>
                <div class="table-wrap">
                    <table class="compare">
                        <thead>
                            <tr>
                                <th>"Fitur"</th>
                                {cols.iter().map(|p| view! {
                                    <th class:is-pop=p.popular>{p.name.trim_start_matches("Paket ").to_string()}</th>
                                }).collect_view()}
                            </tr>
                        </thead>
                        <tbody>
                            {FEATURES.iter().map(|(f, a, b, c)| view! {
                                <tr>
                                    <td>{*f}</td>
                                    {[*a, *b, *c].into_iter().take(cols.len()).enumerate().map(|(i, v)| view! {
                                        <td class:is-pop=cols.get(i).is_some_and(|p| p.popular) class:is-no=v == "—">
                                            {if v == "✓" { view! { <Icon name="check_circle" class="ok" /> }.into_any() } else { v.into_any() }}
                                        </td>
                                    }).collect_view()}
                                </tr>
                            }).collect_view()}
                            <tr class="compare__price">
                                <td>"Harga"</td>
                                {cols.iter().map(|p| view! { <td class:is-pop=p.popular>{rupiah(p.price)}</td> }).collect_view()}
                            </tr>
                        </tbody>
                    </table>
                </div>
            </section>

            <section class="info-grid">
                <div class="card info-card">
                    <h2>"Layanan Tambahan"</h2>
                    {k.addon.clone().into_iter().map(|a| view! {
                        <div class="info-row">
                            <span><b>{a.name.clone()}</b><small>{a.desc.clone()}</small></span>
                            <b>{format!("+{}", rupiah(a.price))}</b>
                        </div>
                    }).collect_view()}
                </div>
                <div class="card info-card">
                    <h2>"Promo Aktif"</h2>
                    {k.active_coupon().cloned().map(|c| view! {
                        <div class="info-row">
                            <span><b>{format!("Kupon {}", c.code)}</b><small>"Masukkan saat mengisi formulir pemesanan"</small></span>
                            <b>{format!("-{}", rupiah(c.amount))}</b>
                        </div>
                    })}
                    <p class="muted small">"Pembayaran via QRIS, Virtual Account, atau kartu kredit. Undangan langsung bisa dipratinjau setelah formulir dikirim; admin mengaktifkan penuh setelah pembayaran terkonfirmasi."</p>
                </div>
            </section>

    }
}

// ── /panduan ───────────────────────────────────────────────────────────────

const STEPS: &[(&str, &str, &str)] = &[
    ("grid_view", "Pilih tema", "Jelajahi katalog, buka Demo untuk mencoba tampilan & musik, lalu tekan \"Pilih\"."),
    ("edit_note", "Isi data & pesan", "Lengkapi data mempelai, jadwal akad/resepsi, musik, dan rekening amplop. Pilih paket lalu kirim formulir."),
    ("admin_panel_settings", "Simpan tautan Kelola", "Setelah memesan Anda mendapat tautan halaman Kelola (berisi kunci rahasia). Simpan baik-baik — jangan dibagikan ke tamu."),
    ("person_add", "Tambah daftar tamu", "Di halaman Kelola, tambahkan nama tamu, kategori (VIP/Keluarga/…), sesi, dan nomor meja."),
    ("send", "Sebar via WhatsApp", "Tekan \"Kirim WA\" di tiap tamu: pesan undangan berisi tautan pribadi (nama tamu tampil di sampul) langsung terbuka di WhatsApp."),
    ("qr_code_scanner", "Check-in di hari H", "Tamu menunjukkan QR di tab Doa & RSVP; penerima tamu memindainya lewat menu Scanner Buku Tamu."),
];

#[component]
pub fn PanduanPage() -> impl IntoView {
    view! {
        <InfoShell
            active="panduan"
            path="/panduan"
            seo_title=concat!("Panduan Membuat Undangan Online Pernikahan | ", crate::brand!())
            title="Panduan Pasangan"
            eyebrow="Pusat Bantuan"
            lead="Dari memilih tema sampai check-in tamu di hari bahagia — semua langkahnya ada di sini."
        >
            <section class="steps-guide">
                {STEPS.iter().enumerate().map(|(i, (icon, t, d))| view! {
                    <article class="card guide-step">
                        <span class="guide-step__num">{i + 1}</span>
                        <span class="guide-step__icon"><span class="ms" aria-hidden="true">{*icon}</span></span>
                        <h3>{*t}</h3>
                        <p>{*d}</p>
                    </article>
                }).collect_view()}
            </section>

            <section class="info-grid">
                <div class="card info-card">
                    <h2><Icon name="link" />"Dua jenis tautan tamu"</h2>
                    <div class="info-row info-row--stack">
                        <b>"Tautan pribadi — "<code>"/u/nama-pasangan-k7f3x9m2?g=KODE"</code></b>
                        <small>"Dibuat otomatis untuk tamu yang Anda daftarkan. Nama & label VIP tampil, RSVP terhubung ke daftar tamu, dan tamu mendapat QR check-in."</small>
                    </div>
                    <div class="info-row info-row--stack">
                        <b>"Tautan umum — "<code>"/u/nama-pasangan-k7f3x9m2?to=Nama+Tamu"</code></b>
                        <small>"Untuk sebar cepat tanpa mendaftarkan tamu. Nama dari tautan tampil di sampul; RSVP tetap tercatat sebagai ucapan umum."</small>
                    </div>
                </div>
                <div class="card info-card">
                    <h2><Icon name="music_note" />"Tips musik latar"</h2>
                    <p class="muted">"Browser tidak mengizinkan musik berbunyi sebelum tamu mengetuk layar. Karena itu musik mulai saat tamu menekan \"Buka Undangan & Putar Musik\", lalu tetap berputar saat berpindah tab Acara dan RSVP."</p>
                    <p class="muted">"Upload MP3 maksimal 6 MB; 128 kbps sudah jernih dan cepat dimuat di HP."</p>
                </div>
            </section>

            <section class="card info-card">
                <h2>"Pertanyaan yang Sering Diajukan"</h2>
                <Faq items=&[
                    ("Tautan Kelola saya hilang, bagaimana?", "Hubungi admin via WhatsApp dari nomor yang Anda pakai saat memesan. Demi keamanan kunci Kelola tidak kami simpan dalam bentuk asli, jadi setelah verifikasi admin menerbitkan tautan Kelola BARU — tautan lama otomatis tidak berlaku."),
                    ("Apakah tamu perlu memasang aplikasi?", "Tidak. Undangan dibuka langsung di browser HP — cukup ketuk tautan dari WhatsApp."),
                    ("Tamu mengirim RSVP dua kali, apakah dobel?", "Untuk tamu terdaftar (tautan ?g=KODE), RSVP berikutnya memperbarui yang lama, bukan menambah baru."),
                    ("Bagaimana mengetahui amplop digital yang masuk?", "Tamu yang sudah transfer bisa menekan \"Saya Sudah Mengirim\"; laporannya tampil di dashboard Kelola dan ikut ekspor CSV. Cocokkan tetap dengan mutasi rekening Anda."),
                    ("Scanner QR tidak menyalakan kamera?", "Pemindai memakai fitur kamera bawaan browser (Chrome/Edge Android terbaru). Di browser lain, ketik saja kode tamu 6 huruf di kolom yang tersedia."),
                    ("Bisa ubah data setelah undangan jadi?", "Bisa, hubungi admin. Kuota revisi mengikuti paket Anda (lihat halaman Paket & Harga)."),
                    ("Apakah data tamu aman?", "Daftar tamu hanya bisa dilihat lewat tautan Kelola Anda. Selengkapnya di halaman Kebijakan Privasi."),
                ] />
            </section>

            <section class="info-cta card card--soft">
                <h2>"Masih bingung?"</h2>
                <p class="muted">"Tim kami siap membantu input data, import kontak tamu, hingga gladi bersih sebar link."</p>
                <div class="info-cta__btns">
                    <ConsultButton text="Chat Admin di WhatsApp" />
                    <a class="btn btn--soft" href=format!("/u/{}", crate::web::themes::DEMO_SLUG)>"Lihat Undangan Demo"</a>
                </div>
            </section>
        </InfoShell>
    }
}

#[component]
fn Faq(items: &'static [(&'static str, &'static str)]) -> impl IntoView {
    view! {
        <div class="faq">
            {items.iter().map(|(q, a)| view! {
                <details class="faq__item">
                    <summary>{*q}<Icon name="expand_more" /></summary>
                    <p>{*a}</p>
                </details>
            }).collect_view()}
        </div>
    }
}

// ── /privasi & /syarat ─────────────────────────────────────────────────────

#[component]
fn LegalSection(title: &'static str, points: &'static [&'static str]) -> impl IntoView {
    view! {
        <section class="legal__sec">
            <h2>{title}</h2>
            <ul>{points.iter().map(|p| view! { <li>{*p}</li> }).collect_view()}</ul>
        </section>
    }
}

#[component]
pub fn PrivasiPage() -> impl IntoView {
    view! {
        <InfoShell
            active=""
            path="/privasi"
            title="Kebijakan Privasi & Data Tamu"
            eyebrow="Berlaku sejak 29 September 2026"
            lead=concat!("Bagaimana ", crate::brand!(), " mengumpulkan, memakai, dan melindungi data pasangan serta tamu undangan.")
        >
            <article class="card legal">
                <LegalSection title="1. Data yang kami kumpulkan" points=&[
                    "Data pemesan: nama kedua mempelai, nama orang tua, username Instagram (opsional), nomor WhatsApp pemesan, foto & lagu yang diunggah, serta nomor rekening untuk amplop digital.",
                    "Data tamu yang ditambahkan pengantin: nama, nomor WhatsApp (opsional), kategori, sesi, dan nomor meja.",
                    "Data yang diisi tamu: nama, nomor WhatsApp (opsional), status kehadiran, jumlah tamu, doa & ucapan, serta laporan tanda kasih.",
                    "Data teknis: waktu tautan pribadi dibuka dan waktu check-in, untuk ditampilkan di dashboard pengantin.",
                ] />
                <LegalSection title="2. Penggunaan data" points=&[
                    "Menampilkan undangan, mengelola RSVP, buku ucapan, dan check-in tamu.",
                    "Menghubungi pemesan terkait pembayaran dan bantuan teknis.",
                    "Kami tidak menjual atau menyewakan data pasangan maupun tamu kepada pihak mana pun, dan tidak memakainya untuk iklan.",
                ] />
                <LegalSection title="3. Siapa yang dapat melihat data" points=&[
                    "Isi undangan (nama mempelai, jadwal, lokasi, rekening amplop, serta doa & ucapan) dapat dilihat siapa pun yang memegang tautan undangan.",
                    "Daftar tamu, nomor WhatsApp tamu, status RSVP, dan laporan tanda kasih hanya tampil di halaman Kelola yang dilindungi kunci rahasia milik pengantin.",
                    concat!("Admin ", crate::brand!(), " dapat mengakses data untuk keperluan bantuan teknis dan verifikasi pembayaran."),
                ] />
                <LegalSection title="4. Penyimpanan & keamanan" points=&[
                    "Data disimpan di server kami dan dikirim melalui koneksi terenkripsi (HTTPS).",
                    "Foto dan lagu disimpan di penyimpanan objek kami dan dapat diakses publik melalui tautan undangan.",
                    "Undangan dan datanya disimpan selama masa aktif paket, kecuali pengantin meminta penghapusan lebih awal.",
                ] />
                <LegalSection title="5. Hak Anda" points=&[
                    "Pengantin dapat meminta salinan, koreksi, atau penghapusan seluruh data undangan dan tamu melalui WhatsApp admin.",
                    "Tamu yang ingin ucapan atau datanya dihapus dapat menghubungi pengantin atau admin kami.",
                    "Kebijakan ini dapat diperbarui; perubahan penting akan diumumkan di halaman ini.",
                ] />
            </article>
        </InfoShell>
    }
}

#[component]
pub fn SyaratPage() -> impl IntoView {
    view! {
        <InfoShell
            active=""
            path="/syarat"
            title="Syarat & Ketentuan Layanan"
            eyebrow="Berlaku sejak 29 September 2026"
            lead=concat!("Ketentuan penggunaan layanan undangan pernikahan digital ", crate::brand!(), ".")
        >
            <article class="card legal">
                <LegalSection title="1. Layanan" points=&[
                    concat!(crate::brand!(), " menyediakan pembuatan undangan pernikahan digital, RSVP, buku ucapan, amplop digital (penampilan nomor rekening), dan check-in QR."),
                    concat!("Amplop digital hanya menampilkan rekening milik pengantin. Dana ditransfer langsung oleh tamu ke rekening tersebut; ", crate::brand!(), " tidak menampung atau menyalurkan dana."),
                ] />
                <LegalSection title="2. Pemesanan & pembayaran" points=&[
                    "Harga mengikuti paket dan add-on yang dipilih saat pemesanan, termasuk potongan kupon yang berlaku.",
                    "Undangan dapat dipratinjau pemesan setelah formulir dikirim; tamu baru bisa membukanya setelah pembayaran terkonfirmasi.",
                    "Pesanan yang pembayarannya belum dikonfirmasi admin dalam 24 jam dihapus otomatis, termasuk seluruh foto dan lagu yang diunggah.",
                    "Garansi uang kembali berlaku 7 hari sejak aktivasi untuk kendala teknis yang tidak dapat kami perbaiki.",
                ] />
                <LegalSection title="3. Konten pengguna" points=&[
                    "Pemesan bertanggung jawab atas kebenaran data dan memiliki hak atas foto serta lagu yang diunggah.",
                    "Lagu bawaan disediakan dengan lisensi untuk dipakai di undangan; dilarang mengunduh ulang untuk keperluan lain.",
                    "Kami berhak menghapus konten atau ucapan yang melanggar hukum, mengandung SARA, pornografi, atau ujaran kebencian.",
                ] />
                <LegalSection title="4. Tautan Kelola" points=&[
                    "Tautan Kelola berisi kunci rahasia. Siapa pun yang memegangnya dapat melihat dan mengubah daftar tamu — jangan membagikannya.",
                    "Kehilangan tautan dapat diurus melalui admin setelah verifikasi nomor WhatsApp pemesan.",
                ] />
                <LegalSection title="5. Batasan tanggung jawab" points=&[
                    "Kami berupaya menjaga layanan tetap tersedia, namun tidak bertanggung jawab atas gangguan jaringan pihak ketiga (operator seluler, WhatsApp, Google Maps).",
                    "Pengiriman pesan WhatsApp dilakukan dari akun WhatsApp milik pengantin; kebijakan WhatsApp berlaku atas pengiriman tersebut.",
                ] />
            </article>
        </InfoShell>
    }
}
