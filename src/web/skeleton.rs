//! web/skeleton.rs — kerangka shimmer pengganti spinner saat data dimuat.
//!
//! Tiap kerangka meniru BENTUK kontainer yang sedang dimuat (undangan,
//! dashboard, kartu katalog, daftar, ubin story, …) supaya tata letak tidak
//! melompat saat isi asli tiba. Kelas `.sk*` di style/main.css (blok
//! "SKELETON"); kilau mati otomatis di prefers-reduced-motion.

use leptos::prelude::*;

/// Satu blok abu-abu berkilau. `class` = bentuk (sk-line, sk-title, sk-pill,
/// sk-circle, sk-media, …); `w` = lebar CSS opsional (mis. "60%").
#[component]
fn Sk(#[prop(optional)] class: &'static str, #[prop(optional)] w: &'static str) -> impl IntoView {
    let style = if w.is_empty() { String::new() } else { format!("width:{w}") };
    view! { <span class=format!("sk {class}") style=style aria-hidden="true"></span> }
}

/// Pembungkus aksesibel: pembaca layar mendengar "Memuat…" satu kali.
#[component]
fn SkWrap(class: &'static str, children: Children) -> impl IntoView {
    view! {
        <div class=format!("skl {class}") role="status" aria-busy="true">
            <span class="sr-only">"Memuat…"</span>
            {children()}
        </div>
    }
}

/// Baris teks bertingkat (paragraf).
#[component]
pub fn SkelLines(#[prop(default = 3)] n: usize) -> impl IntoView {
    const W: [&str; 4] = ["92%", "78%", "86%", "54%"];
    view! {
        <SkWrap class="skl-lines">
            {(0..n).map(|i| view! { <Sk class="sk-line" w=W[i % W.len()] /> }).collect_view()}
        </SkWrap>
    }
}

/// Kartu sampul undangan dalam bentuk kerangka — SATU blok untuk layar muat
/// undangan & layar HP demo /tema: cincin monogram berdenyut, "The Wedding
/// Of", nama, tanggal, garis, tamu, tombol. Berpalet SENDIRI (`.skl-und`),
/// tak memakai warna tema mana pun — dulu ikut var(--bg)/--primary tema yang
/// sedang aktif sehingga loading tampak seperti "salah satu tema".
#[component]
fn SkelSampul() -> impl IntoView {
    view! {
        <div class="skl-und__card">
            <span class="skl-und__ring"><Sk class="sk-circle skl-und__mono" /></span>
            <Sk class="sk-line sk-sm" w="36%" />
            <Sk class="sk-title skl-und__names" w="72%" />
            <Sk class="sk-line" w="48%" />
            <span class="skl-und__rule" aria-hidden="true"><i></i></span>
            <Sk class="sk-line sk-sm" w="40%" />
            <Sk class="sk-title" w="58%" />
            <Sk class="sk-pill" w="62%" />
        </div>
    }
}

/// Layar muat undangan (/u/…). HP = kartu sampul di tengah + navigasi bawah;
/// desktop = panel kiri : kartu kanan, seperti undangan aslinya.
#[component]
pub fn SkelInvitation() -> impl IntoView {
    view! {
        <SkWrap class="skl-und skl-inv">
            <div class="skl-und__side" aria-hidden="true">
                <Sk class="sk-line sk-sm" w="34%" />
                <Sk class="sk-hero" w="70%" />
                <Sk class="sk-line" w="44%" />
                <Sk class="sk-pill" w="38%" />
            </div>
            <div class="skl-und__stage">
                <SkelSampul />
            </div>
            <div class="skl-nav skl-und__nav">
                {(0..5).map(|_| view! { <span class="skl-nav__i"><Sk class="sk-circle sk-20" /><Sk class="sk-line sk-xs" /></span> }).collect_view()}
            </div>
        </SkWrap>
    }
}

/// Isi layar HP demo di /tema/:slug (sampul undangan).
#[component]
pub fn SkelPhone() -> impl IntoView {
    view! {
        <SkWrap class="skl-und skl-phone">
            <SkelSampul />
        </SkWrap>
    }
}

/// Halaman detail (tema / dekorasi): media besar + teks & tombol.
/// `phone` = media berbentuk HP (demo tema), selain itu foto 4:3.
#[component]
pub fn SkelDetail(#[prop(optional)] phone: bool) -> impl IntoView {
    view! {
        <SkWrap class="skl-detail">
            <div class="skl-detail__text">
                <Sk class="sk-line" w="28%" />
                <Sk class="sk-hero" w="80%" />
                <Sk class="sk-line" w="94%" />
                <Sk class="sk-line" w="86%" />
                <Sk class="sk-line" w="60%" />
                <div class="skl-row"><Sk class="sk-pill" w="38%" /><Sk class="sk-pill" w="30%" /></div>
            </div>
            <Sk class=if phone { "sk-media sk-phone" } else { "sk-media sk-photo" } />
        </SkWrap>
    }
}

/// Kisi kartu (katalog tema, paket layanan): gambar + judul + baris + harga.
#[component]
pub fn SkelCards(#[prop(default = 6)] n: usize, #[prop(optional)] class: &'static str) -> impl IntoView {
    view! {
        <div class=format!("skl skl-cards {class}") role="status" aria-busy="true">
            <span class="sr-only">"Memuat…"</span>
            {(0..n).map(|_| view! {
                <div class="skl-card skl-card--media">
                    <Sk class="sk-media sk-card-art" />
                    <div class="skl-card__body">
                        <Sk class="sk-line sk-sm" w="34%" />
                        <Sk class="sk-title" w="72%" />
                        <Sk class="sk-line" w="88%" />
                        <div class="skl-row skl-between"><Sk class="sk-line" w="30%" /><Sk class="sk-pill sk-sm" w="28%" /></div>
                    </div>
                </div>
            }).collect_view()}
        </div>
    }
}

/// Halaman layanan (cetak/dekorasi/MUA/seserahan): hero + kisi kartu.
#[component]
pub fn SkelPage() -> impl IntoView {
    view! {
        <div class="skl-page">
            <SkelDetail />
            <SkelCards n=3 />
        </div>
    }
}

/// Daftar baris (ucapan, lagu, pesanan admin): avatar + 2 baris + aksi.
#[component]
pub fn SkelRows(#[prop(default = 4)] n: usize, #[prop(optional)] action: bool) -> impl IntoView {
    view! {
        <SkWrap class="skl-rows">
            {(0..n).map(|i| view! {
                <div class="skl-card skl-row">
                    <Sk class="sk-circle sk-40" />
                    <div class="skl-grow">
                        <Sk class="sk-line" w=if i % 2 == 0 { "38%" } else { "46%" } />
                        <Sk class="sk-line" w=if i % 2 == 0 { "82%" } else { "64%" } />
                    </div>
                    {action.then(|| view! { <Sk class="sk-pill sk-sm" w="76px" /> })}
                </div>
            }).collect_view()}
        </SkWrap>
    }
}

/// Pilihan berbentuk kartu (paket / tema terpilih di /buat).
#[component]
pub fn SkelOptions(#[prop(default = 3)] n: usize) -> impl IntoView {
    view! {
        <SkWrap class="skl-rows">
            {(0..n).map(|_| view! {
                <div class="skl-card skl-row">
                    <Sk class="sk-box sk-40" />
                    <div class="skl-grow"><Sk class="sk-line" w="44%" /><Sk class="sk-line sk-sm" w="70%" /></div>
                    <Sk class="sk-line" w="22%" />
                </div>
            }).collect_view()}
        </SkWrap>
    }
}

/// Ubin story 9:14 (moderasi Kelola).
#[component]
pub fn SkelTiles(#[prop(default = 4)] n: usize) -> impl IntoView {
    view! {
        <div class="skl story-grid" role="status" aria-busy="true">
            <span class="sr-only">"Memuat…"</span>
            {(0..n).map(|_| view! { <Sk class="sk-media sk-story" /> }).collect_view()}
        </div>
    }
}

/// Dashboard Kelola: kepala, 4 kartu statistik, daftar tamu.
#[component]
pub fn SkelDashboard() -> impl IntoView {
    view! {
        <SkWrap class="skl-dash">
            <div class="skl-card skl-row">
                <Sk class="sk-circle sk-48" />
                <div class="skl-grow"><Sk class="sk-title" w="40%" /><Sk class="sk-line" w="28%" /></div>
                <Sk class="sk-pill sk-sm" w="120px" />
            </div>
            <div class="skl-stats">
                {(0..4).map(|_| view! {
                    <div class="skl-card"><Sk class="sk-line sk-sm" w="50%" /><Sk class="sk-title" w="40%" /></div>
                }).collect_view()}
            </div>
            <SkelRows n=5 action=true />
        </SkWrap>
    }
}

/// Pemindai QR buku tamu: kepala, jendela kamera, tombol.
#[component]
pub fn SkelScanner() -> impl IntoView {
    view! {
        <SkWrap class="skl-scan">
            <div class="skl-card skl-row">
                <Sk class="sk-circle sk-40" />
                <div class="skl-grow"><Sk class="sk-line" w="50%" /><Sk class="sk-line sk-sm" w="30%" /></div>
            </div>
            <Sk class="sk-media sk-square" />
            <Sk class="sk-pill" />
        </SkWrap>
    }
}
