//! web/app.rs — shell HTML SSR, router, dan skrip inline global.

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{ParentRoute, Route, Router, Routes},
    path, SsrMode,
};

#[cfg(feature = "ssr")]
use super::icons::icon_font_href;
use super::pages::{
    buat::BuatPage,
    cetak::CetakPage,
    dekorasi::{DekorasiDetailPage, DekorasiPage},
    info::{PanduanPage, PaketPage, PrivasiPage, SyaratPage},
    katalog::KatalogPage,
    admin::{AdminAkun, AdminAnimEdit, AdminAnims, AdminBanner, AdminHome, AdminKonten, AdminLagu, AdminOrnaments, AdminStory, AdminStoryPanduan, AdminTemplat, AdminKontenEdit, AdminProfil, AdminThemeEdit, AdminThemes, AdminUndangan},
    kelola::{KelolaPage, ScanPage},
    sunting::SuntingPage,
    mua::MuaPage,
    seserahan::SeserahanPage,
    tema::TemaPage,
    undangan::{AcaraPage, InvitationLayout, RsvpPage, SampulPage, StoryPage},
    NotFoundPage,
};

#[cfg(feature = "ssr")]
pub fn shell(options: leptos::config::LeptosOptions) -> AnyView {
    // Nonce CSP per request (server/security.rs) → skrip hydration Leptos
    // (lewat use_nonce) dan skrip global di bawah ikut diizinkan CSP.
    if let Some(n) = use_context::<axum::http::request::Parts>()
        .and_then(|p| p.extensions.get::<crate::server::security::CspNonce>().cloned())
    {
        provide_context(leptos::nonce::Nonce::from_value(n.0));
    }
    let nonce = leptos::nonce::use_nonce();
    // Pratinjau kartu katalog (iframe ?pv=1): tanpa WASM — gerbang, animasi
    // muncul & hitung mundur sudah ditangani /app.js, jadi iframe tampil jauh
    // lebih cepat (tak mengunduh/kompilasi WASM untuk tiap kartu).
    let preview = use_context::<axum::http::request::Parts>().is_some_and(|p| {
        crate::server::security::is_demo_preview(p.uri.path(), p.uri.query().unwrap_or(""))
    });
    view! {
        <!DOCTYPE html>
        <html lang="id">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover" />
                <meta name="theme-color" content="#f4fcf0" />
                {site_verification().iter().map(|(n, c)| view! { <meta name=*n content=c.clone() /> }).collect_view()}
                <link rel="stylesheet" href="/pkg/undangan.css" />
                <link rel="icon" type="image/svg+xml" href="/favicon.svg" />
                <link rel="preconnect" href="https://fonts.googleapis.com" />
                <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="" />
                <link
                    rel="stylesheet"
                    href="https://fonts.googleapis.com/css2?family=Playfair+Display:ital,wght@0,500;0,600;1,500&family=Plus+Jakarta+Sans:wght@400;500;600;700&display=swap"
                />
                <link rel="stylesheet" href=icon_font_href() />
                // Tema dari DB: CSS variabel per tema + font judul yang dipakai tema.
                {theme_links()}
                {google_verification()}
                {(!preview).then(|| view! {
                    <AutoReload options=options.clone() />
                    <HydrationScripts options=options.clone() />
                })}
                <MetaTags />
            </head>
            <body>
                <App />
                <script nonce=nonce src=format!("/app.js?v={}", global_js_version())></script>
            </body>
        </html>
    }
    .into_any()
}

/// Meta verifikasi Google Search Console dari env GOOGLE_SITE_VERIFICATION
/// (kode dari Search Console → "Tag HTML"). Kosong → tak dipasang.
#[cfg(feature = "ssr")]
fn google_verification() -> Option<AnyView> {
    let v = std::env::var("GOOGLE_SITE_VERIFICATION").ok()?;
    let v = v.trim().to_string();
    (!v.is_empty() && v.len() <= 100 && v.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'))
        .then(|| view! { <meta name="google-site-verification" content=v /> }.into_any())
}

/// `<link>` /tema.css (ber-versi) dan Google Fonts untuk font tema.
/// Halaman undangan /u/… melewatkannya: InvShell memasang CSS & huruf tema
/// undangan itu saja (/gaya/{tema}.css) — ±220 KB CSS semua tema tak diurai.
#[cfg(feature = "ssr")]
fn theme_links() -> AnyView {
    let undangan = use_context::<axum::http::request::Parts>().is_some_and(|p| p.uri.path().starts_with("/u/"));
    if undangan {
        return ().into_any();
    }
    let cat = use_context::<std::sync::Arc<crate::server::state::AppState>>().map(|s| s.themes());
    let css = match &cat {
        Some(c) => format!("/tema.css?v={}", c.version),
        None => "/tema.css".into(),
    };
    let fonts = cat
        .filter(|c| !c.fonts.is_empty())
        .map(|c| format!("https://fonts.googleapis.com/css2?family={}&display=swap", c.fonts.join("&family=")));
    view! {
        <link rel="stylesheet" href=css />
        {fonts.map(|href| view! { <link rel="stylesheet" href=href /> })}
    }
    .into_any()
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    super::pages::provide_konten_slot();

    view! {
        // Judul cadangan saja; description/canonical/og ditulis tiap halaman
        // publik lewat web::seo::Seo (dulu di sini → tag description ganda).
        <Title text=concat!(crate::brand!(), " — Undangan Online Pernikahan") />

        <Router>
            <Routes fallback=NotFoundPage>
                <Route path=path!("/") view=KatalogPage />
                <Route path=path!("/tema/:tema") view=TemaPage ssr=SsrMode::Async />
                <Route path=path!("/buat") view=BuatPage />
                <Route path=path!("/paket") view=PaketPage />
                <Route path=path!("/panduan") view=PanduanPage />
                <Route path=path!("/cetak") view=CetakPage />
                <Route path=path!("/dekorasi") view=DekorasiPage />
                <Route path=path!("/dekorasi/:slug") view=DekorasiDetailPage ssr=SsrMode::Async />
                <Route path=path!("/mua") view=MuaPage />
                <Route path=path!("/seserahan") view=SeserahanPage />
                <Route path=path!("/admin") view=AdminHome />
                <Route path=path!("/admin/tema") view=AdminThemes />
                <Route path=path!("/admin/tema/:slug") view=AdminThemeEdit />
                <Route path=path!("/admin/tema/:slug/ornamen") view=AdminOrnaments />
                <Route path=path!("/admin/animasi") view=AdminAnims />
                <Route path=path!("/admin/templat") view=AdminTemplat />
                <Route path=path!("/admin/banner") view=AdminBanner />
                <Route path=path!("/admin/lagu") view=AdminLagu />
                <Route path=path!("/admin/story") view=AdminStory />
                <Route path=path!("/admin/story-panduan") view=AdminStoryPanduan />
                <Route path=path!("/admin/animasi/baru") view=AdminAnimEdit />
                <Route path=path!("/admin/animasi/:kind/:slug") view=AdminAnimEdit />
                <Route path=path!("/admin/undangan") view=AdminUndangan />
                <Route path=path!("/admin/konten") view=AdminKonten />
                <Route path=path!("/admin/konten/:key") view=AdminKontenEdit />
                <Route path=path!("/admin/akun") view=AdminAkun />
                <Route path=path!("/admin/profil") view=AdminProfil />
                <Route path=path!("/privasi") view=PrivasiPage />
                <Route path=path!("/syarat") view=SyaratPage />
                // Async: <head> (og:title/description untuk pratinjau tautan WhatsApp)
                // dan status 404 baru dikirim setelah data undangan siap.
                <ParentRoute path=path!("/u/:slug") view=InvitationLayout ssr=SsrMode::Async>
                    <Route path=path!("") view=SampulPage />
                    <Route path=path!("acara") view=AcaraPage />
                    <Route path=path!("rsvp") view=RsvpPage />
                    <Route path=path!("story") view=StoryPage />
                </ParentRoute>
                <Route path=path!("/kelola/:slug") view=KelolaPage ssr=SsrMode::Async />
                <Route path=path!("/kelola/:slug/scan") view=ScanPage />
                <Route path=path!("/kelola/:slug/sunting") view=SuntingPage ssr=SsrMode::Async />
            </Routes>
        </Router>
    }
    .into_any()
}

/// Skrip global (web/global.js): musik, salin, hitung mundur, animasi muncul,
/// banner… — dilayani sebagai berkas ber-versi `/app.js?v=hash` sehingga
/// di-cache browser selamanya (dulu ±24 KB inline di SETIAP halaman).
#[cfg(feature = "ssr")]
pub const GLOBAL_JS: &str = include_str!("global.js");

/// Tag verifikasi kepemilikan situs dari env (dibaca sekali):
/// GOOGLE_SITE_VERIFICATION (Search Console, metode "Tag HTML" — isi
/// `content`-nya saja) & BING_SITE_VERIFICATION (Bing Webmaster).
#[cfg(feature = "ssr")]
fn site_verification() -> &'static [(&'static str, String)] {
    static V: std::sync::OnceLock<Vec<(&'static str, String)>> = std::sync::OnceLock::new();
    V.get_or_init(|| {
        [("google-site-verification", "GOOGLE_SITE_VERIFICATION"), ("msvalidate.01", "BING_SITE_VERIFICATION")]
            .into_iter()
            .filter_map(|(name, var)| std::env::var(var).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty()).map(|v| (name, v)))
            .collect()
    })
}

/// Hash isi skrip global → `?v=` (berubah otomatis tiap skrip diubah).
#[cfg(feature = "ssr")]
pub fn global_js_version() -> &'static str {
    static V: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    V.get_or_init(|| format!("{:08x}", crate::server::util::fnv1a64(GLOBAL_JS) as u32))
}
