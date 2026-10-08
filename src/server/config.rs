//! server/config.rs — konfigurasi dari environment.

#[derive(Clone)]
pub struct RustFsConfig {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
    /// Base public URL tanpa trailing slash.
    pub public_url: String,
    /// RUSTFS_AUTO_CREATE_BUCKET=true → bucket yang belum ada dibuat saat start
    /// + policy baca publik. Bawaan false: di production bucket & policy-nya
    /// disiapkan admin secara sadar, bukan efek samping start aplikasi.
    pub auto_create_bucket: bool,
}

/// Alamat publik situs (tautan di pesan WA). Override: SITE_URL (mis. untuk staging).
pub const DEFAULT_SITE_URL: &str = "https://ilyvowcraft.online";

/// Penerima bawaan notifikasi bukti transfer (WhatsApp pemilik usaha).
pub const DEFAULT_NOTIFY_WA: &str = "089635816942";

#[derive(Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub auto_migrate: bool,
    pub site_addr: String,
    pub admin_wa: String,
    pub admin_token: String,
    /// Pesanan belum dikonfirmasi admin dihapus (beserta file RustFS) setelah
    /// sekian jam. UNPAID_TTL_HOURS, bawaan 24.
    pub unpaid_ttl_hours: i64,
    pub rustfs: RustFsConfig,
    pub wa: super::wa::WaConfig,
    /// Nomor WA (62…) penerima notifikasi bukti transfer. PAYMENT_NOTIFY_WA,
    /// bawaan = 6289635816942 (nomor pemilik).
    pub notify_wa: String,
    /// Alamat publik situs untuk tautan di pesan WA (SITE_URL, bawaan
    /// https://ilyvowcraft.online). Diset kosong = dari header request (proxy).
    pub site_url: String,
}

impl AppConfig {
    pub fn from_env(default_addr: &str) -> anyhow::Result<Self> {
        let env = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.to_string());
        Ok(Self {
            database_url: std::env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL wajib di-set"))?,
            auto_migrate: env("AUTO_MIGRATE", "false") == "true",
            site_addr: env("SITE_ADDR", default_addr),
            admin_wa: crate::web::fmt::wa_number(&env("ADMIN_WHATSAPP", "")),
            admin_token: env("ADMIN_TOKEN", "").trim().to_string(),
            unpaid_ttl_hours: env("UNPAID_TTL_HOURS", "24").trim().parse::<i64>().unwrap_or(24).clamp(1, 24 * 30),
            rustfs: RustFsConfig {
                endpoint: env("RUSTFS_ENDPOINT", "http://127.0.0.1:9000"),
                access_key: env("RUSTFS_ACCESS_KEY", ""),
                secret_key: env("RUSTFS_SECRET_KEY", ""),
                bucket: env("RUSTFS_BUCKET", "undangan"),
                public_url: env("RUSTFS_PUBLIC_URL", "http://127.0.0.1:9000"),
                auto_create_bucket: env("RUSTFS_AUTO_CREATE_BUCKET", "false") == "true",
            },
            wa: super::wa::WaConfig {
                base_url: env("WAXUM_BASE_URL", "").trim().trim_end_matches('/').to_string(),
                session: env("WAXUM_SESSION", "undangan").trim().to_string(),
                token: env("WAXUM_TOKEN", "").trim().to_string(),
            },
            notify_wa: crate::web::fmt::wa_number(&env("PAYMENT_NOTIFY_WA", DEFAULT_NOTIFY_WA)),
            site_url: env("SITE_URL", DEFAULT_SITE_URL).trim().trim_end_matches('/').to_string(),
        })
    }
}
