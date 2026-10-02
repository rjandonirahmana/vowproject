//! main.rs — server undangan-ily (Leptos SSR + Axum), satu binary satu port.
//!
//!   /                        → katalog tema
//!   /tema/{slug}             → demo tema + kontrol musik
//!   /buat  (POST /buat/kirim) → formulir pemesanan (multipart: foto + lagu)
//!   /u/{slug}[/acara|/rsvp]  → undangan untuk tamu (?g=KODE / ?to=Nama)
//!   /kelola/{slug}[/scan]    → dashboard pengantin (?key=…)
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

/// Batas body formulir: foto mempelai (2) + sampul (1) + galeri + 1 lagu + 2 MB teks/overhead.
const MAX_FORM: usize = (3 + handlers::MAX_GALLERY) * undangan::server::storage::MAX_IMAGE
    + undangan::server::storage::MAX_AUDIO
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
    });
    // Hapus pesanan yang tak dikonfirmasi admin dalam UNPAID_TTL_HOURS (+ file RustFS).
    tokio::spawn(undangan::server::cleanup::run(state.clone()));
    match state.reload_konten().await {
        Ok(()) => tracing::info!("konten situs dimuat"),
        Err(e) => tracing::warn!(error = %format!("{e:#}"), "tabel site_content belum siap (jalankan migration/003_admin_konten.sql) — memakai konten bawaan"),
    }
    state.seed_animations().await;
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
        .route("/buat/kirim", axum::routing::post(handlers::create_invitation))
        .route("/kelola/{slug}/tamu.csv", axum::routing::get(handlers::export_guests))
        .route("/layanan/wa", axum::routing::get(handlers::layanan_wa))
        .route("/tema.css", axum::routing::get(handlers::theme_css))
        .route("/app.js", axum::routing::get(handlers::app_js))
        // healthz (di bawah) = proses hidup; readyz = siap melayani (DB menjawab).
        .route("/readyz", axum::routing::get(handlers::readyz))
        .route("/admin/masuk", axum::routing::post(handlers::admin_login))
        .route("/admin/setup", axum::routing::post(handlers::admin_setup))
        .route("/admin/keluar", axum::routing::post(handlers::admin_logout))
        .route("/admin/konten/simpan", axum::routing::post(handlers::admin_save_konten))
        .route("/admin/akun/simpan", axum::routing::post(handlers::admin_account_save))
        .route("/admin/sandi", axum::routing::post(handlers::admin_own_password))
        .route("/admin/tema/simpan", axum::routing::post(handlers::admin_save_theme))
        .route("/admin/tema/hapus", axum::routing::post(handlers::admin_delete_theme))
        .route("/admin/animasi/simpan", axum::routing::post(handlers::admin_save_animation))
        .route("/admin/animasi/hapus", axum::routing::post(handlers::admin_delete_animation))
        .route("/admin/animasi/bawaan", axum::routing::post(handlers::admin_reset_animation))
        .route("/admin/ornamen/simpan", axum::routing::post(handlers::admin_save_ornament))
        .route("/admin/ornamen/{aksi}", axum::routing::post(handlers::admin_ornament_action))
        .route("/admin/banner/simpan", axum::routing::post(handlers::admin_save_banner))
        .route("/admin/banner/{aksi}", axum::routing::post(handlers::admin_banner_action))
        .route("/admin/undangan/simpan", axum::routing::post(handlers::admin_update_invitation))
        .route("/admin/undangan/kunci", axum::routing::post(handlers::admin_reset_key))
        // 2 foto + 1 lagu + teks — diturunkan dari batas unggahan di storage.rs.
        .layer(axum::extract::DefaultBodyLimit::max(MAX_FORM))
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
        .layer(tower_http::compression::CompressionLayer::new())
        .layer(axum::middleware::from_fn(security::csrf))
        .layer(axum::middleware::from_fn(security::headers))
        .layer(axum::Extension(security::DevMode(dev)))
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
