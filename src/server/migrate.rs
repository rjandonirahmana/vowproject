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

/// Penanda skema per migrasi: objek yang PASTI ada setelah migrasi itu jalan.
#[derive(Clone, Copy)]
enum Tanda {
    /// Tabel / indeks (`to_regclass`).
    Objek(&'static str),
    /// Kolom tabel.
    Kolom(&'static str, &'static str),
}

/// Setiap migrasi yang mengubah skema WAJIB punya penanda di sini; migrasi
/// yang hanya mengisi data masuk `HANYA_DATA`. Test `semua_migrasi_tercakup`
/// gagal bila ada berkas baru yang belum digolongkan — daftar ini tak bisa
/// lagi tertinggal diam-diam (dulu berhenti di 017 padahal migrasi sampai 029).
const PENANDA: &[(&str, Tanda)] = &[
    ("001_init.sql", Tanda::Objek("public.invitations")),
    ("002_themes.sql", Tanda::Objek("public.themes")),
    ("003_admin_konten.sql", Tanda::Objek("public.site_content")),
    ("004_keamanan.sql", Tanda::Kolom("invitations", "manage_key_hash")),
    ("005_tampilan.sql", Tanda::Kolom("invitations", "cover_photo")),
    ("006_animasi.sql", Tanda::Kolom("themes", "scroll_anim")),
    ("007_animasi_kustom.sql", Tanda::Objek("public.animations")),
    ("008_animasi_semua.sql", Tanda::Kolom("animations", "builtin")),
    ("011_banner.sql", Tanda::Objek("public.banners")),
    ("012_ornamen.sql", Tanda::Objek("public.theme_ornaments")),
    ("015_indeks.sql", Tanda::Objek("public.invitations_unpaid_idx")),
    ("017_bukti_bayar.sql", Tanda::Kolom("invitations", "payment_proof_at")),
    ("017b_provenance.sql", Tanda::Kolom("theme_ornaments", "source")),
    ("022_sinema.sql", Tanda::Kolom("invitations", "video_url")),
    ("023_wayang_pintu.sql", Tanda::Kolom("themes", "open_video")),
    ("025_sekar_kedhaton.sql", Tanda::Kolom("themes", "motion_locked")),
    ("026_story.sql", Tanda::Objek("public.story_keys")),
    ("027_pustaka_lagu.sql", Tanda::Objek("public.songs")),
    ("028_story_hapus.sql", Tanda::Kolom("invitation_stories", "owner_token_hash")),
    ("029_tema_templat.sql", Tanda::Objek("public.theme_templates")),
    ("033_rupa.sql", Tanda::Kolom("themes", "rupa")),
    ("037_story_panduan.sql", Tanda::Objek("public.site_stories")),
];

/// Migrasi data saja (seed tema/animasi/demo) — tak punya objek skema untuk
/// dicek; kelengkapannya dilihat dari schema_migrations bila AUTO_MIGRATE.
#[cfg_attr(not(test), allow(dead_code))]
const HANYA_DATA: &[&str] = &[
    "009_tema_nusantara.sql",
    "010_musik_demo.sql",
    "013_gerak_everlove.sql",
    "014_ornamen_adat.sql",
    "016_banner_cepat.sql",
    "018_koreografi.sql",
    "019_sepuluh_tema.sql",
    "020_wayang_everlove.sql",
    "021_demo_yona_doni.sql",
    "024_desain_v2.sql",
    "030_perbaikan_gerak.sql",
    "031_tema_provinsi.sql",
    "032_gerak_ringan.sql",
    "034_tema_suku.sql",
    "035_rupa_suku.sql",
    "036_ornamen_rapi.sql",
];

/// Migrasi skema yang BELUM diterapkan (dicek dari objek skemanya, bukan dari
/// schema_migrations — migrasi di proyek ini biasanya dijalankan manual).
pub async fn missing(pool: &Pool) -> Result<Vec<&'static str>> {
    let c = pool.get().await.context("cek skema: ambil koneksi")?;
    let mut out = Vec::new();
    for (nama, tanda) in PENANDA {
        let ada: bool = match tanda {
            Tanda::Objek(o) => c.query_one("SELECT to_regclass($1) IS NOT NULL", &[o]).await,
            Tanda::Kolom(t, k) => {
                c.query_one(
                    "SELECT EXISTS (SELECT 1 FROM information_schema.columns
                                     WHERE table_schema = 'public' AND table_name = $1 AND column_name = $2)",
                    &[t, k],
                )
                .await
            }
        }
        .with_context(|| format!("cek skema {nama}"))?
        .get(0);
        if !ada {
            out.push(*nama);
        }
    }
    Ok(out)
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
    fn semua_migrasi_tercakup() {
        for (nama, _) in MIGRATIONS {
            let n = PENANDA.iter().filter(|(p, _)| p == nama).count() + HANYA_DATA.iter().filter(|p| *p == nama).count();
            assert_eq!(n, 1, "{nama}: tambahkan ke PENANDA (ubah skema) atau HANYA_DATA (seed) di migrate.rs");
        }
        for (p, _) in PENANDA {
            assert!(MIGRATIONS.iter().any(|(n, _)| n == p), "{p} tak ada di migration/");
        }
    }

    #[test]
    fn checksum_stabil_dan_peka() {
        let a = "CREATE TABLE x (id int);";
        assert_eq!(checksum(a), checksum(a));
        assert_ne!(checksum(a), checksum("CREATE TABLE y (id int);"));
    }
}
