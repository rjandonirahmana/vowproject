//! config/database.rs — pool Postgres. Pola disalin dari e-ticketing
//! (config/database.rs): timeout statement di sisi server, keepalive, dan
//! smoke-test saat start supaya `DATABASE_URL` yang salah gagal cepat.

use std::time::Duration;

use deadpool_postgres::{Config, Pool, Runtime};
use tokio_postgres::NoTls;

const STATEMENT_TIMEOUT_MS_BAWAAN: u64 = 15_000;
const IDLE_TX_TIMEOUT_MS: u64 = 30_000;

pub async fn create_pool(database_url: &str, max_size: usize) -> anyhow::Result<Pool> {
    let mut cfg = Config::new();
    cfg.url = Some(database_url.to_string());

    let statement_timeout_ms = std::env::var("DB_STATEMENT_TIMEOUT_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(STATEMENT_TIMEOUT_MS_BAWAAN);

    if statement_timeout_ms > 0 {
        cfg.options = Some(format!(
            "-c statement_timeout={statement_timeout_ms} \
             -c idle_in_transaction_session_timeout={IDLE_TX_TIMEOUT_MS}"
        ));
    }

    cfg.keepalives = Some(true);
    cfg.keepalives_idle = Some(Duration::from_secs(30));
    cfg.connect_timeout = Some(Duration::from_secs(5));

    cfg.pool = Some(deadpool_postgres::PoolConfig {
        max_size,
        timeouts: deadpool_postgres::Timeouts {
            wait: Some(Duration::from_secs(5)),
            create: Some(Duration::from_secs(5)),
            recycle: Some(Duration::from_secs(2)),
        },
        ..Default::default()
    });

    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;

    let client = pool.get().await?;
    client.simple_query("SELECT 1").await?;

    Ok(pool)
}
