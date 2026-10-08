//! pages — halaman per rute + kerangka situs (header/footer) untuk halaman
//! pemasaran (katalog, demo tema, formulir pesan, layanan pendukung).

pub mod admin;
pub mod buat;
pub mod cetak;
pub mod dekorasi;
pub mod info;
pub mod katalog;
pub mod kelola;
pub mod layanan;
pub mod mua;
pub mod seserahan;
pub mod sunting;
pub mod tema;
pub mod undangan;

use leptos::prelude::*;
use leptos_meta::Title;

use super::components::Monogram;
use super::konten;
use super::icons::Icon;

/// Wadah SATU Resource `get_konten()` per halaman (SSR) / per sesi tab (klien).
/// Dulu tiap komponen (`WithKonten`, katalog, tema, …) membuat Resource
/// sendiri → beranda mengambil, menanam ke HTML, & mendeserialisasi Konten
/// (±20 KB) TIGA kali. Resource dibuat malas oleh pemakai pertama di bawah
/// owner `App`, jadi tetap hidup & dipakai ulang komponen lain / halaman lain.
#[derive(Clone)]
pub struct KontenSlot {
    res: StoredValue<Option<Resource<Result<konten::Konten, ServerFnError>>>>,
    owner: Owner,
}

pub fn provide_konten_slot() {
    if let Some(owner) = Owner::current() {
        provide_context(KontenSlot { res: StoredValue::new(None), owner });
    }
}

/// Konten situs bersama (lihat `KontenSlot`).
pub fn use_konten() -> Resource<Result<konten::Konten, ServerFnError>> {
    let Some(slot) = use_context::<KontenSlot>() else {
        return Resource::new(|| (), |_| super::api::get_konten());
    };
    if let Some(r) = slot.res.get_value() {
        return r;
    }
    let r = slot.owner.with(|| Resource::new(|| (), |_| super::api::get_konten()));
    slot.res.set_value(Some(r));
    r
}

#[component]
pub fn SiteHeader(#[prop(optional)] active: &'static str) -> impl IntoView {
    let links = [
        ("/", "Katalog Tema", "katalog"),
        ("/paket", "Paket & Harga", "paket"),
        ("/cetak", "Cetak Fisik", "cetak"),
        ("/dekorasi", "Dekorasi & Sound", "dekorasi"),
        ("/mua", "Make Up Pengantin", "mua"),
        ("/seserahan", "Sewa Seserahan", "seserahan"),
        ("/panduan", "Panduan", "panduan"),
    ];
    // Di halaman demo tema (/tema/:slug), "Coba Demo" membuka undangan contoh
    // dengan tema yang sedang dilihat.
    let loc = leptos_router::hooks::use_location();
    let demo_href = move || {
        let path = loc.pathname.get();
        match path.strip_prefix("/tema/").map(|s| s.trim_end_matches('/')).filter(|s| crate::web::skin::is_slug(s)) {
            Some(slug) => format!("/u/{}?tema={slug}", crate::web::themes::DEMO_SLUG),
            None => format!("/u/{}", crate::web::themes::DEMO_SLUG),
        }
    };
    view! {
        <header class="site-top">
            <a class="site-top__brand" href="/">
                <Monogram initials=crate::web::BRAND_MONOGRAM class="monogram--xs" />
                <span>
                    <b>{crate::brand!()}</b>
                    <small>"part of tresno mulyo group"</small>
                </span>
            </a>
            <nav class="site-top__nav" aria-label="Navigasi utama">
                {links.into_iter().map(|(href, label, key)| view! {
                    <a href=href class:is-active=active == key>{label}</a>
                }).collect_view()}
            </nav>
            <div class="site-top__actions">
                <a class="btn btn--soft btn--sm hide-sm" href=demo_href>"Coba Demo"</a>
                <a class="btn btn--primary btn--sm" href="/buat">"Buat Undangan"</a>
            </div>
        </header>
        <AdminBar />
    }
}

/// Bilah "Mode Admin" di halaman publik: hanya muncul untuk admin yang masuk.
/// Tanpa tautan login di situs — panel dibuka lewat alamat /admin. Pengunjung
/// biasa tidak memicu request apa pun (cek penanda cookie dulu, di browser).
#[component]
fn AdminBar() -> impl IntoView {
    let me: LocalResource<Option<crate::web::model::AdminUser>> = LocalResource::new(|| async move {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::JsCast;
            let marked = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.dyn_into::<web_sys::HtmlDocument>().ok())
                .and_then(|d| d.cookie().ok())
                .is_some_and(|c| c.split(';').any(|kv| kv.trim() == "ily_adm_ui=1"));
            if marked {
                return crate::web::api::admin_me().await.ok().flatten();
            }
        }
        None
    });
    let loc = leptos_router::hooks::use_location();
    view! {
        <Suspense fallback=|| ()>
            {move || me.get().flatten().map(|u| {
                let path = loc.pathname.get();
                let links = admin_links(&path);
                view! {
                    <aside class="admin-bar" aria-label="Mode admin">
                        <span class="admin-bar__who"><Icon name="admin_panel_settings" />{format!("Mode Admin • {}", u.display())}</span>
                        <span class="admin-bar__links">
                            {links.into_iter().map(|(href, label)| view! {
                                <a href=href><Icon name="edit_note" />{label}</a>
                            }).collect_view()}
                            <a href="/admin" class="admin-bar__panel">"Panel"</a>
                        </span>
                    </aside>
                }
            })}
        </Suspense>
    }
}

/// Bagian yang bisa disunting untuk halaman publik `path`.
fn admin_links(path: &str) -> Vec<(String, String)> {
    let sec = |k: &str| {
        konten::section(k).map(|s| (format!("/admin/konten/{k}"), format!("Sunting {}", s.title)))
    };
    let keys: &[&str] = match path.trim_end_matches('/') {
        "" => &["paket"],
        "/paket" => &["paket", "addon", "kupon", "umum"],
        "/cetak" => &["cetak_paket", "cetak_finishing", "cetak_wilayah", "cetak_testimoni", "umum"],
        "/dekorasi" => &["dekor_paket", "dekor_venue", "dekor_galeri", "umum"],
        p if p.starts_with("/dekorasi/") => &["dekor_paket"],
        "/mua" => &["mua_paket", "mua_galeri", "mua_profil", "mua_testimoni", "umum"],
        "/seserahan" => &["seserahan_paket", "seserahan_opsi", "seserahan_antar", "seserahan_galeri", "seserahan_info", "seserahan_testimoni"],
        "/buat" => &["paket", "addon", "kupon"],
        _ => &[],
    };
    let mut out: Vec<(String, String)> = keys.iter().filter_map(|k| sec(k)).collect();
    if let Some(slug) = path.strip_prefix("/tema/") {
        out.push((format!("/admin/tema/{slug}"), "Sunting tema ini".into()));
    }
    if path.trim_end_matches('/').is_empty() {
        out.insert(0, ("/admin/banner".into(), "Sunting Banner".into()));
        out.push(("/admin/tema".into(), "Kelola tema katalog".into()));
    }
    out
}

#[component]
pub fn SiteFooter() -> impl IntoView {
    // Di halaman demo tema (/tema/:slug), tautan demo ikut tema itu.
    let loc = leptos_router::hooks::use_location();
    let tema = move || {
        loc.pathname.get().strip_prefix("/tema/").map(|s| s.trim_end_matches('/').to_string()).filter(|s| crate::web::skin::is_slug(s))
    };
    let demo = crate::web::themes::DEMO_SLUG;
    let demo_u = move || tema().map(|t| format!("/u/{demo}?tema={t}")).unwrap_or_else(|| format!("/u/{demo}"));
    let demo_kelola = move || tema().map(|t| format!("/kelola/{demo}?key=demo&tema={t}")).unwrap_or_else(|| format!("/kelola/{demo}?key=demo"));
    view! {
        <footer class="site-foot">
            <div class="site-foot__grid">
                <div class="site-foot__about">
                    <p class="site-foot__brand">{crate::brand!()}</p>
                    <p>"Studio undangan pernikahan digital beraroma heirloom: kurasi estetika botani sage & foil emas, RSVP instan, amplop digital, dan cerita cinta abadi."</p>
                    <p class="site-foot__trust"><Icon name="verified_user" />"Tersertifikasi SSL & perlindungan data pribadi"</p>
                </div>
                <div>
                    <p class="site-foot__head">"Jelajahi Desain"</p>
                    <a href="/?kategori=botanical">"Tema Botani"</a>
                    <a href="/?kategori=adat">"Gaya Tradisional Nusantara"</a>
                    <a href="/?kategori=modern">"Tema Modern Minimalis"</a>
                    <a href="/?kategori=luxury">"Luxury Celestial"</a>
                </div>
                <div>
                    <p class="site-foot__head">"Fitur & Layanan"</p>
                    <a href="/paket">"Paket & Harga"</a>
                    <a href="/paket#fitur">"Perbandingan Fitur"</a>
                    <span>"RSVP & Buku Tamu Interaktif"</span>
                    <span>"Amplop Digital & QR Check-in"</span>
                </div>
                <div>
                    <p class="site-foot__head">"Layanan Pernikahan"</p>
                    <a href="/cetak">"Cetak Undangan Fisik"</a>
                    <a href="/cetak#kalkulator">"Kalkulator Harga & Ongkir"</a>
                    <a href="/dekorasi">"Dekorasi & Sound System"</a>
                    <a href="/mua">"MUA & Rias Pengantin"</a>
                    <a href="/seserahan">"Sewa Seserahan & Hantaran"</a>
                </div>
                <div>
                    <p class="site-foot__head">"Bantuan"</p>
                    <a href="/panduan">"Pusat Panduan & FAQ"</a>
                    <a href=demo_u>"Lihat Undangan Demo"</a>
                    // rel=external: muat penuh agar server menukar ?key=demo jadi cookie.
                    <a href=demo_kelola rel="external">"Demo Dashboard Pengantin"</a>
                    <a href="/privasi">"Kebijakan Privasi & Data Tamu"</a>
                    <a href="/syarat">"Syarat & Ketentuan"</a>
                </div>
            </div>
            <div class="site-foot__bar">
                <span><Icon name="lock" />"Pembayaran aman • QRIS • BCA • Mandiri"</span>
                <span>{concat!("© 2026 ", crate::brand!(), " • ILYproject")}</span>
            </div>
        </footer>
    }
}

#[component]
pub fn NotFoundPage() -> impl IntoView {
    #[cfg(feature = "ssr")]
    if let Some(r) = use_context::<leptos_axum::ResponseOptions>() {
        r.set_status(axum::http::StatusCode::NOT_FOUND);
    }
    view! {
        <Title text=concat!("Halaman tidak ditemukan — ", crate::brand!()) />
        <div class="empty-page">
            <Monogram initials="404" />
            <h1 class="section__title">"Halaman tidak ditemukan"</h1>
            <p class="muted">"Tautan undangan mungkin salah ketik atau sudah tidak aktif."</p>
            <a class="btn btn--primary" href="/">"Kembali ke Katalog"</a>
        </div>
    }
}

#[component]
pub fn ErrorCard(msg: String) -> impl IntoView {
    view! {
        <div class="empty-page">
            <Icon name="info" class="empty-page__icon" />
            <h1 class="section__title">"Terjadi kendala"</h1>
            <p class="muted">{msg}</p>
            <a class="btn btn--primary" href="/">"Kembali"</a>
        </div>
    }
}
