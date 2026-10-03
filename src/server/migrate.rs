//! config/migrate.rs — penjalan migrasi database.
//!
//! Berkas `migration/*.sql` di-embed ke binari lewat `build.rs`, dikirim UTUH
//! ke Postgres lewat `batch_execute` (Postgres sendiri yang memisah
//! pernyataannya — bukan klien yang memotong tiap titik-koma), dicatat di
//! `schema_migrations`, dan dijaga dari dua instance yang jalan bersamaan
//! lewat `pg_advisory_lock`. Pola disalin dari e-ticketing, disederhanakan:
//! proyek ini fresh, tidak ada histori "sudah dimigrasi tangan" yang perlu
//! garis dasar.

use anyhow::{Context, Result};
use deadpool_postgres::Pool;

include!(concat!(env!("OUT_DIR"), "/migrations.rs"));

const LOCK_KEY: i64 = 0x554e_4441_4e47_0001;

fn checksum(s: &str) -> String {
    format!("{:016x}", super::util::fnv1a64(s))
}

pub async fn run(pool: &Pool) -> Result<()> {
    let mut conn = pool.get().await.context("migrate: ambil koneksi")?;

    conn.batch_execute("SET statement_timeout = 0")
        .await
        .context("migrate: mematikan statement_timeout")?;

    conn.execute("SELECT pg_advisory_lock($1)", &[&LOCK_KEY])
        .await
        .context("migrate: pg_advisory_lock")?;

    let hasil = jalankan(&mut conn).await;

    if let Err(e) = conn
        .execute("SELECT pg_advisory_unlock($1)", &[&LOCK_KEY])
        .await
    {
        tracing::warn!(error = %e, "migrate: gagal melepas advisory lock");
    }

    if let Err(e) = conn.batch_execute("RESET statement_timeout").await {
        tracing::warn!(error = %e, "migrate: gagal mengembalikan statement_timeout");
    }

    hasil
}

async fn jalankan(conn: &mut deadpool_postgres::Object) -> Result<()> {
    conn.batch_execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
             version     TEXT        PRIMARY KEY,
             checksum    TEXT        NOT NULL,
             applied_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
         )",
    )
    .await
    .context("migrate: buat schema_migrations")?;

    let baris = conn
        .query("SELECT version, checksum FROM schema_migrations", &[])
        .await
        .context("migrate: baca schema_migrations")?;

    let mut sudah: std::collections::HashMap<String, String> =
        std::collections::HashMap::with_capacity(baris.len());
    for r in baris {
        sudah.insert(r.get::<_, String>(0), r.get::<_, String>(1));
    }

    let mut dijalankan = 0_usize;

    for (nama, sql) in MIGRATIONS {
        let cs = checksum(sql);

        if let Some(lama) = sudah.get(*nama) {
            if lama != &cs {
                tracing::warn!(
                    migration = nama,
                    "migrate: berkas BERUBAH setelah dijalankan — perubahan itu tidak ikut masuk"
                );
            }
            continue;
        }

        tracing::info!(migration = nama, "migrate: menjalankan");

        let tx = conn
            .transaction()
            .await
            .with_context(|| format!("migrate: buka transaksi {nama}"))?;

        if let Err(e) = tx.batch_execute(sql).await {
            // Objek sudah ada = migrasi ini sudah dijalankan manual (psql -f)
            // tanpa tercatat. Jangan dinaikkan ulang: batalkan transaksinya,
            // cukup catat sebagai sudah diterapkan.
            // Hanya 001 (ditulis sebelum aturan idempoten) yang boleh lewat jalur
            // ini. Migrasi 002+ idempoten (IF NOT EXISTS / ON CONFLICT) sehingga
            // "sudah ada" tak pernah terjadi; bila tetap terjadi, itu tanda
            // migrasi manual setengah jalan — jangan dicatat "selesai".
            if sudah_ada(&e) && *nama == "001_init.sql" {
                drop(tx);
                tracing::warn!(
                    migration = nama,
                    error = %e,
                    "migrate: objek sudah ada (migrasi manual?) — dilewati & dicatat sebagai sudah diterapkan"
                );
                conn.execute(
                    "INSERT INTO schema_migrations (version, checksum) VALUES ($1, $2) ON CONFLICT (version) DO NOTHING",
                    &[nama, &cs],
                )
                .await
                .with_context(|| format!("migrate: catat {nama}"))?;
                continue;
            }
            return Err(e).with_context(|| format!("migrate: GAGAL di {nama}"));
        }

        tx.execute(
            "INSERT INTO schema_migrations (version, checksum) VALUES ($1, $2)",
            &[nama, &cs],
        )
        .await
        .with_context(|| format!("migrate: catat {nama}"))?;

        tx.commit()
            .await
            .with_context(|| format!("migrate: commit {nama}"))?;

        dijalankan += 1;
    }

    if dijalankan == 0 {
        tracing::info!("migrate: skema sudah mutakhir");
    } else {
        tracing::info!(count = dijalankan, "migrate: selesai");
    }
    Ok(())
}

/// Migrasi yang BELUM diterapkan (dicek dari objek skemanya, bukan dari
/// schema_migrations — migrasi di proyek ini biasanya dijalankan manual).
pub async fn missing(pool: &Pool) -> Result<Vec<&'static str>> {
    let c = pool.get().await.context("cek skema: ambil koneksi")?;
    let r = c
        .query_one(
            "SELECT to_regclass('public.themes') IS NOT NULL,
                    to_regclass('public.admin_users') IS NOT NULL AND to_regclass('public.site_content') IS NOT NULL,
                    EXISTS (SELECT 1 FROM information_schema.columns
                            WHERE table_name = 'invitations' AND column_name = 'manage_key_hash'),
                    EXISTS (SELECT 1 FROM information_schema.columns
                            WHERE table_name = 'invitations' AND column_name = 'cover_photo'),
                    EXISTS (SELECT 1 FROM information_schema.columns
                            WHERE table_name = 'themes' AND column_name = 'scroll_anim'),
                    EXISTS (SELECT 1 FROM information_schema.columns
                            WHERE table_name = 'animations' AND column_name = 'builtin'),
                    to_regclass('public.banners') IS NOT NULL,
                    to_regclass('public.theme_ornaments') IS NOT NULL,
                    to_regclass('public.invitations_unpaid_idx') IS NOT NULL,
                    EXISTS (SELECT 1 FROM information_schema.columns
                            WHERE table_name = 'invitations' AND column_name = 'payment_proof_at')",
            &[],
        )
        .await
        .context("cek skema")?;
    let checks = [(0, "002_themes.sql"), (1, "003_admin_konten.sql"), (2, "004_keamanan.sql"), (3, "005_tampilan.sql"), (4, "006_animasi.sql"), (5, "008_animasi_semua.sql (007 boleh dilewati)"), (6, "011_banner.sql"), (7, "012_ornamen.sql"), (8, "015_indeks.sql"), (9, "017_bukti_bayar.sql")];
    Ok(checks.into_iter().filter(|(i, _)| !r.get::<_, bool>(*i)).map(|(_, n)| n).collect())
}

/// Galat "sudah ada" (tabel/indeks/constraint/kolom/tipe) dari Postgres.
fn sudah_ada(e: &tokio_postgres::Error) -> bool {
    use tokio_postgres::error::SqlState;
    matches!(
        e.code(),
        Some(c) if *c == SqlState::DUPLICATE_TABLE
            || *c == SqlState::DUPLICATE_OBJECT
            || *c == SqlState::DUPLICATE_COLUMN
            || *c == SqlState::DUPLICATE_SCHEMA
            || *c == SqlState::DUPLICATE_FUNCTION
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daftar_migrasi_terurut() {
        let nama: Vec<&str> = MIGRATIONS.iter().map(|(n, _)| *n).collect();
        let mut urut = nama.clone();
        urut.sort_unstable();
        assert_eq!(nama, urut, "MIGRATIONS harus urut menurut nama berkas");
        assert!(!MIGRATIONS.is_empty(), "tak ada migrasi yang ter-embed");
    }

    #[test]
    fn checksum_stabil_dan_peka() {
        let a = "CREATE TABLE x (id int);";
        assert_eq!(checksum(a), checksum(a));
        assert_ne!(checksum(a), checksum("CREATE TABLE y (id int);"));
    }
}
