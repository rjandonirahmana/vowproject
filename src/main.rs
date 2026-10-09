//! main.rs — server undangan-ily (Leptos SSR + Axum), satu binary satu port.
//!
//!   /                        → katalog tema
//!   /tema/{slug}             → demo tema + kontrol musik
//!   /buat  (POST /buat/kirim) → formulir pemesanan (multipart: foto + lagu)
//!   /u/{slug}[/acara|/rsvp]  → undangan untuk tamu (?g=KODE / ?to=Nama)
//!   /kelola/{slug}[/scan]    → dashboard pengantin (?key=… sekali → cookie, server/owner.rs)
//!   /api/*                   → server function
//!   /pkg/*                   → aset WASM/JS/CSS
//!   /cetak /dekorasi /mua /seserahan → layanan pendukung (formulir → /layanan/wa)
//!   /admin[/tema|/undangan]  → panel admin (ADMIN_TOKEN): tema & aktivasi
//!   /tema.css                → CSS tema, dibangkitkan dari tabel themes
//!   /healthz                 → cek hidup untuk deploy & proxy

#![recursion_limit = "512"]

use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use leptos::config::get_configuration;
use leptos::prelude::provide_context;
use leptos_axum::{generate_route_list, LeptosRoutes};
use undangan::server::{config::AppConfig, db::create_pool, handlers, migrate, security, state::{self, AppState}, storage::StorageService};

/// Batas body formulir: foto mempelai (2) + sampul (1) + galeri + 1 lagu + 1 video + 2 MB teks/overhead.
const MAX_FORM: usize = (3 + handlers::MAX_GALLERY) * undangan::server::storage::MAX_IMAGE
    + undangan::server::storage::MAX_AUDIO
    + undangan::server::storage::MAX_VIDEO
    + 2 * 1024 * 1024;
use undangan::web::app::{shell, App};

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("gagal memasang rustls crypto provider");
    dotenvy::dotenv().ok();

    // LOG_FORMAT=json → satu baris JSON per log (untuk pengumpul log produksi).
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| "undangan=info,tower_http=info".into());
    if std::env::var("LOG_FORMAT").is_ok_and(|v| v == "json") {
        tracing_subscriber::registry().with(filter).with(tracing_subscriber::fmt::layer().json()).init();
    } else {
        tracing_subscriber::registry().with(filter).with(tracing_subscriber::fmt::layer()).init();
    }

    let leptos_conf = get_configuration(Some("Cargo.toml"))
        .map_err(|e| anyhow::anyhow!("gagal memuat konfigurasi leptos: {e}"))?;
    let leptos_options = leptos_conf.leptos_options;
    let site_root = leptos_options.site_root.to_string();
    let dev = matches!(leptos_options.env, leptos::config::Env::DEV);

    let cfg = AppConfig::from_env(&leptos_options.site_addr.to_string())?;
    // url(…) di CSS animasi admin hanya boleh ke penyimpanan & situs sendiri.
    undangan::web::anim::set_url_origins(
        [&cfg.rustfs.public_url, &cfg.site_url].iter().filter_map(|u| undangan::web::anim::origin_of(u)).collect(),
    );

    let pool_size = std::env::var("DB_POOL_SIZE").ok().and_then(|v| v.parse().ok()).filter(|n| *n > 0).unwrap_or(10);
    let pool = create_pool(&cfg.database_url, pool_size).await.context("gagal membuat pool Postgres")?;
    tracing::info!("Postgres pool siap");
    if cfg.auto_migrate {
        migrate::run(&pool).await.context("migrasi database gagal")?;
    } else {
        tracing::info!("AUTO_MIGRATE mati — migrasi dijalankan manual (psql -f migration/…)");
    }

    // Kode ini membutuhkan skema terbaru; beri tahu SEJELAS mungkin bila belum.
    match migrate::missing(&pool).await {
        Ok(m) if m.is_empty() => {}
        Ok(m) => {
            for name in &m {
                tracing::error!("SKEMA BELUM LENGKAP — jalankan: psql \"$DATABASE_URL\" -f migration/{name}");
            }
            if m.contains(&"004_keamanan.sql") {
                tracing::error!("Tanpa 004, halaman undangan/Kelola & pemesanan GAGAL (kolom manage_key_hash tidak ada).");
            }
        }
        Err(e) => tracing::warn!(error = %format!("{e:#}"), "gagal memeriksa skema"),
    }

    let storage = StorageService::new(&cfg.rustfs);
    match &storage {
        Some(s) => {
            if tokio::time::timeout(std::time::Duration::from_secs(3), s.init()).await.is_err() {
                tracing::warn!(endpoint = %cfg.rustfs.endpoint, "RustFS: timeout saat start — lanjut jalan");
            }
        }
        None => tracing::warn!("RUSTFS_ACCESS_KEY kosong — unggah foto/lagu dinonaktifkan"),
    }

    if cfg.admin_token.len() < 12 {
        tracing::warn!("ADMIN_TOKEN kosong/terlalu pendek (<12) — akun admin pertama belum bisa dibuat");
    }
    let state = Arc::new(AppState {
        pool,
        storage,
        admin_wa: cfg.admin_wa.clone(),
        wa: {
            let w = undangan::server::wa::WaClient::new(cfg.wa.clone());
            match (&w, cfg.notify_wa.is_empty()) {
                (None, _) => tracing::warn!("WAXUM_BASE_URL kosong — semua notifikasi WhatsApp dinonaktifkan"),
                (Some(_), true) => tracing::warn!("waxum aktif tapi PAYMENT_NOTIFY_WA/ADMIN_WHATSAPP kosong — admin tak menerima notifikasi bukti transfer"),
                (Some(_), false) => tracing::info!(to = %cfg.notify_wa, session = %cfg.wa.session, "waxum: notifikasi WhatsApp aktif"),
            }
            if !cfg.wa.base_url.is_empty() && cfg.wa.token.is_empty() {
                tracing::warn!("WAXUM_TOKEN kosong — waxum akan menolak (401) bila autentikasinya aktif");
            }
            w
        },
        notify_wa: cfg.notify_wa.clone(),
        site_url: cfg.site_url.clone(),
        admin_token: if cfg.admin_token.len() >= 12 { cfg.admin_token.clone() } else { String::new() },
        themes: state::fallback_catalog(),
        konten: std::sync::RwLock::new(std::sync::Arc::new(undangan::web::konten::Konten::default())),
        // 5 percobaan gagal per username / IP per 15 menit.
        login_limit: security::RateLimit::new(5, std::time::Duration::from_secs(15 * 60)),
        // Longgar karena tamu sering berbagi IP (CGNAT operator seluler / Wi-Fi
        // gedung): 20 kiriman per IP per undangan per 10 menit.
        write_limit: security::RateLimit::new(20, std::time::Duration::from_secs(10 * 60)),
        // 10 pembuatan undangan per IP per jam — tiap kiriman bisa membawa foto & lagu.
        create_limit: security::RateLimit::new(10, std::time::Duration::from_secs(60 * 60)),
        unpaid_ttl_hours: cfg.unpaid_ttl_hours,
        // Longgar untuk Wi-Fi gedung / CGNAT (banyak tamu satu IP): ±100 buka
        // halaman per menit per IP. Bot di atas itu ditolak 429.
        req_limit: security::RateLimit::new(env_num("REQ_PER_MIN", 600), std::time::Duration::from_secs(60)),
        inflight: std::sync::Arc::new(tokio::sync::Semaphore::new(env_num("MAX_INFLIGHT", 128) as usize)),
        cap_limit: security::RateLimit::new(u32::MAX, std::time::Duration::from_secs(60 * 60)),
        banners: std::sync::RwLock::new(None),
        templat: state::fallback_templat(),
    });
    // Hapus pesanan yang tak dikonfirmasi admin dalam UNPAID_TTL_HOURS (+ file RustFS).
    tokio::spawn(undangan::server::cleanup::run(state.clone()));
    // waxum: cek sesi sekali saat start — log jelas bila belum tersambung,
    // jangan sampai baru ketahuan ketika WA pertama tak sampai.
    if let Some(w) = state.wa.clone() {
        tokio::spawn(async move {
            match w.session_status().await {
                Ok(s) if s == "logged_in" || s == "connected" => tracing::info!(status = %s, "waxum: sesi tersambung"),
                Ok(s) => tracing::error!(status = %s, "waxum: sesi BELUM siap (pasangkan QR/kode di konsol waxum) — WA tak akan terkirim"),
                Err(e) => tracing::error!(error = %format!("{e:#}"), "waxum: gagal dicek — WA tak akan terkirim"),
            }
        });
    }
    match state.reload_konten().await {
        Ok(()) => tracing::info!("konten situs dimuat"),
        Err(e) => tracing::warn!(error = %format!("{e:#}"), "tabel site_content belum siap (jalankan migration/003_admin_konten.sql) — memakai konten bawaan"),
    }
    state.seed_animations().await;
    state.reload_templat(true).await;
    match state.reload_themes().await {
        Ok(()) => tracing::info!(n = state.themes().list.len(), "katalog tema dimuat"),
        Err(e) => tracing::warn!(error = %format!("{e:#}"), "tabel themes belum siap (jalankan migration/002_themes.sql) — memakai tema bawaan"),
    }

    let socket_addr: std::net::SocketAddr = cfg
        .site_addr
        .parse()
        .map_err(|e| anyhow::anyhow!("SITE_ADDR tidak valid `{}`: {e}", cfg.site_addr))?;

    let routes = generate_route_list(App);

    let form_routes = axum::Router::new()
        .route("/buat/kirim", axum::routing::post(handlers::create_invitation).layer(axum::extract::DefaultBodyLimit::max(MAX_FORM)))
        .route("/kelola/{slug}/tamu.csv", axum::routing::get(handlers::export_guests))
        .route("/kelola/{slug}/bukti", axum::routing::post(handlers::upload_payment_proof).layer(axum::extract::DefaultBodyLimit::max(MAX_FORM)))
        .route("/kelola/{slug}/sunting/simpan", axum::routing::post(handlers::update_invitation).layer(axum::extract::DefaultBodyLimit::max(MAX_FORM)))
        .route("/u/{slug}/story/kirim", axum::routing::post(handlers::post_story).layer(axum::extract::DefaultBodyLimit::max(MAX_FORM)))
        .route("/u/{slug}/story/hapus-saya", axum::routing::post(handlers::delete_my_story))
        .route("/u/{slug}/story/hapus", axum::routing::post(handlers::delete_story_with_key))
        .route("/kelola/{slug}/story/hapus", axum::routing::post(handlers::owner_delete_story))
        .route("/layanan/wa", axum::routing::get(handlers::layanan_wa))
        .route("/tema.css", axum::routing::get(handlers::theme_css))
        .route("/gaya/{file}", axum::routing::get(handlers::gaya_css))
        .route("/sitemap.xml", axum::routing::get(handlers::sitemap))
        .route("/app.js", axum::routing::get(handlers::app_js))
        // Tema templat: mesin gerak bersama + kirim RSVP (server/templat.rs).
        .route("/tata.js", axum::routing::get(undangan::server::templat::tata_js))
        .route("/tata.css", axum::routing::get(undangan::server::templat::tata_css))
        .route("/u/{slug}/rsvp/kirim", axum::routing::post(undangan::server::templat::rsvp_kirim))
        // healthz (di bawah) = proses hidup; readyz = siap melayani (DB menjawab).
        .route("/readyz", axum::routing::get(handlers::readyz))
        .route("/admin/masuk", axum::routing::post(handlers::admin_login))
        .route("/admin/setup", axum::routing::post(handlers::admin_setup))
        .route("/admin/keluar", axum::routing::post(handlers::admin_logout))
        .route("/admin/konten/simpan", axum::routing::post(handlers::admin_save_konten).layer(axum::extract::DefaultBodyLimit::max(MAX_FORM)))
        .route("/admin/akun/simpan", axum::routing::post(handlers::admin_account_save))
        .route("/admin/sandi", axum::routing::post(handlers::admin_own_password))
        .route("/admin/tema/simpan", axum::routing::post(handlers::admin_save_theme).layer(axum::extract::DefaultBodyLimit::max(MAX_FORM)))
        .route("/admin/tema/hapus", axum::routing::post(handlers::admin_delete_theme))
        .route("/admin/animasi/simpan", axum::routing::post(handlers::admin_save_animation).layer(axum::extract::DefaultBodyLimit::max(MAX_FORM)))
        .route("/admin/animasi/hapus", axum::routing::post(handlers::admin_delete_animation))
        .route("/admin/animasi/bawaan", axum::routing::post(handlers::admin_reset_animation))
        .route("/admin/ornamen/simpan", axum::routing::post(handlers::admin_save_ornament).layer(axum::extract::DefaultBodyLimit::max(MAX_FORM)))
        .route("/admin/ornamen/{aksi}", axum::routing::post(handlers::admin_ornament_action))
        .route("/admin/banner/simpan", axum::routing::post(handlers::admin_save_banner).layer(axum::extract::DefaultBodyLimit::max(MAX_FORM)))
        .route("/admin/banner/{aksi}", axum::routing::post(handlers::admin_banner_action))
        .route("/admin/templat/simpan", axum::routing::post(handlers::admin_save_templat).layer(axum::extract::DefaultBodyLimit::max(2 * 1024 * 1024)))
        .route("/admin/templat/{aksi}", axum::routing::post(handlers::admin_templat_action).layer(axum::extract::DefaultBodyLimit::max(2 * 1024 * 1024)))
        .route("/admin/lagu/simpan", axum::routing::post(handlers::admin_save_song).layer(axum::extract::DefaultBodyLimit::max(MAX_FORM)))
        .route("/admin/lagu/{aksi}", axum::routing::post(handlers::admin_song_action))
        .route("/admin/undangan/simpan", axum::routing::post(handlers::admin_update_invitation))
        .route("/admin/undangan/kunci", axum::routing::post(handlers::admin_reset_key))
        // Bawaan KECIL: form urlencoded (RSVP, login, aksi admin) dibaca utuh ke
        // memori SEBELUM handler/cek sesi — batas besar di sini = beberapa
        // puluh POST 73 MB dari bot menghabiskan RAM. Batas MAX_FORM hanya di
        // rute multipart di atas (dibaca bertahap + antrean 3 unggahan).
        .layer(axum::extract::DefaultBodyLimit::max(64 * 1024))
        .layer(axum::Extension(state.clone()));

    let leptos_router = axum::Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            {
                let st = state.clone();
                move || provide_context(st.clone())
            },
            {
                let opts = leptos_options.clone();
                move || shell(opts.clone())
            },
        )
        // .br/.gz dibuat sekali oleh `cargo leptos build --precompress` (Dockerfile);
        // dev lokal tanpa berkas itu → CompressionLayer yang mengompres.
        // Nama berkas /pkg tetap antar rilis (undangan.js/.wasm) → `no-cache`:
        // browser WAJIB revalidasi (ETag/Last-Modified, 304 murah) sehingga
        // tak pernah memakai WASM lama dengan HTML baru setelah deploy.
        .route_service(
            "/pkg/{*path}",
            tower_http::set_header::SetResponseHeader::overriding(
                tower_http::services::ServeDir::new(&site_root).precompressed_br().precompressed_gzip(),
                axum::http::header::CACHE_CONTROL,
                axum::http::HeaderValue::from_static("no-cache"),
            ),
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    let app = axum::Router::new()
        .route("/healthz", axum::routing::get(|| async { "ok" }))
        .merge(form_routes)
        .merge(leptos_router)
        // Header keamanan (CSP ber-nonce, nosniff, Referrer-Policy, dll.) SEMUA
        // ditulis server/security.rs `headers` — jangan diduplikasi di sini.
        // Tema templat: GET /u/{slug} dirender dari HTML di DB (bila temanya
        // memakai templat). Di DALAM exchange (?k= sudah jadi cookie)
        // & di dalam kompresi (HTML-nya ikut dikompresi).
        .layer(axum::middleware::from_fn(undangan::server::templat::serve))
        .layer(tower_http::compression::CompressionLayer::new())
        // Tautan Kelola ?key= / pratinjau ?k= → cookie HttpOnly + 303 ke URL bersih.
        .layer(axum::middleware::from_fn(undangan::server::owner::exchange))
        // Demo lama /u/anindita-raditya → /u/yona-doni (301).
        .layer(axum::middleware::from_fn(security::legacy_demo))
        .layer(axum::middleware::from_fn(security::csrf))
        // Paling luar setelah header keamanan: request bot ditolak sebelum
        // menyentuh router, sesi, atau DB.
        .layer(axum::middleware::from_fn(security::pelindung))
        .layer(axum::middleware::from_fn(security::headers))
        .layer(axum::Extension(security::DevMode(dev)))
        .layer(axum::Extension(state.clone()))
        // Unggahan 17 MB di sinyal lemah butuh waktu; proxy juga memberi batas.
        .layer(tower_http::timeout::TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            std::time::Duration::from_secs(120),
        ))
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let listener = TcpListener::bind(socket_addr).await.with_context(|| format!("gagal mengikat {socket_addr}"))?;
    tracing::info!("undangan listening on http://{socket_addr}");
    axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await?;
    tracing::info!("undangan berhenti dengan rapi");
    Ok(())
}

/// Bilangan dari env (> 0), selain itu bawaan.
fn env_num(k: &str, d: u32) -> u32 {
    std::env::var(k).ok().and_then(|v| v.trim().parse().ok()).filter(|n| *n > 0).unwrap_or(d)
}

/// Ctrl-C (dev) atau SIGTERM (docker/systemd): selesaikan request yang
/// sedang berjalan sebelum keluar.
async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let term = async {
        if let Ok(mut s) = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            s.recv().await;
        }
    };
    #[cfg(not(unix))]
    let term = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = term => {} }
    tracing::info!("sinyal berhenti diterima — menyelesaikan request berjalan");
}
