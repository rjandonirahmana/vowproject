//! server/cleanup.rs — tugas latar: sesi admin kedaluwarsa, dan pesanan yang tidak dikonfirmasi
//! admin dalam `UNPAID_TTL_HOURS` (bawaan 24 jam), termasuk SEMUA file
//! unggahannya di RustFS (foto mempelai, lagu, galeri).
//!
//! Jalan tiap 15 menit. `pg_try_advisory_lock` mencegah dua instance
//! membersihkan bersamaan. Urutan: hapus baris DB dulu (atomik, cek status di
//! DELETE itu sendiri — admin yang mengaktifkan tepat waktu selalu menang),
//! baru file. File yang gagal dihapus hanya dicatat (yatim, tanpa data).

use std::sync::Arc;
use std::time::Duration;

use super::repo;
use super::state::AppState;

const EVERY: Duration = Duration::from_secs(15 * 60);
const LOCK_KEY: i64 = 0x554e_4441_4e47_0002;

pub async fn run(state: Arc<AppState>) {
    // Beri waktu server siap dulu.
    tokio::time::sleep(Duration::from_secs(30)).await;
    let mut tick = tokio::time::interval(EVERY);
    loop {
        tick.tick().await;
        if let Err(e) = once(&state).await {
            tracing::warn!(error = %format!("{e:#}"), "cleanup: gagal (dicoba lagi 15 menit)");
        }
    }
}

pub async fn once(state: &AppState) -> anyhow::Result<usize> {
    let c = state.pool.get().await?;
    let locked: bool = c.query_one("SELECT pg_try_advisory_lock($1)", &[&LOCK_KEY]).await?.get(0);
    if !locked {
        return Ok(0);
    }
    let res = repo::purge_unpaid(&state.pool, state.unpaid_ttl_hours).await;
    match repo::purge_expired_sessions(&state.pool).await {
        Ok(0) => {}
        Ok(n) => tracing::info!(n, "cleanup: sesi admin kedaluwarsa dihapus"),
        Err(e) => tracing::warn!(error = %format!("{e:#}"), "cleanup: hapus sesi kedaluwarsa gagal"),
    }
    match repo::purge_story_keys(&state.pool).await {
        Ok(0) => {}
        Ok(n) => tracing::info!(n, "cleanup: kunci story lama dihapus"),
        Err(e) => tracing::warn!(error = %format!("{e:#}"), "cleanup: hapus kunci story gagal"),
    }
    let _ = c.execute("SELECT pg_advisory_unlock($1)", &[&LOCK_KEY]).await;
    let purged = res?;
    for p in &purged {
        let mut removed = 0;
        for url in &p.files {
            match state.storage.as_ref() {
                Some(st) => match st.delete_url(url).await {
                    Ok(true) => removed += 1,
                    Ok(false) => {} // lagu bawaan / URL luar — bukan unggahan kita
                    Err(e) => tracing::warn!(slug = %p.slug, error = %format!("{e:#}"), "cleanup: file RustFS gagal dihapus"),
                },
                None => tracing::warn!(slug = %p.slug, url = %url, "cleanup: RustFS tak dikonfigurasi — file tidak dihapus"),
            }
        }
        tracing::info!(slug = %p.slug, files = removed, hours = state.unpaid_ttl_hours, "cleanup: pesanan belum dibayar dihapus");
    }
    Ok(purged.len())
}
