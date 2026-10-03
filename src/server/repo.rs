//! server/repo.rs — semua query Postgres (tokio-postgres, SQL tulisan tangan).

use anyhow::{Context, Result};
use deadpool_postgres::Pool;
use serde_json::Value;
use tokio_postgres::Row;

use crate::web::fmt::lalu;
use crate::web::model::*;

// ── Statement cache ────────────────────────────────────────────────────────
// Semua query lewat `prepare_cached`: rencana query disiapkan SEKALI per
// koneksi pool lalu dipakai ulang — hemat satu bolak-balik ke Postgres per
// query dibanding mengirim teks SQL (prepare + execute) setiap kali.

type Params<'a> = &'a [&'a (dyn tokio_postgres::types::ToSql + Sync)];
type PgResult<T> = std::result::Result<T, tokio_postgres::Error>;

async fn db_rows(c: &deadpool_postgres::Client, sql: &str, p: Params<'_>) -> PgResult<Vec<Row>> {
    let st = c.prepare_cached(sql).await?;
    c.query(&st, p).await
}

async fn db_row(c: &deadpool_postgres::Client, sql: &str, p: Params<'_>) -> PgResult<Row> {
    let st = c.prepare_cached(sql).await?;
    c.query_one(&st, p).await
}

async fn db_opt(c: &deadpool_postgres::Client, sql: &str, p: Params<'_>) -> PgResult<Option<Row>> {
    let st = c.prepare_cached(sql).await?;
    c.query_opt(&st, p).await
}

async fn db_exec(c: &deadpool_postgres::Client, sql: &str, p: Params<'_>) -> PgResult<u64> {
    let st = c.prepare_cached(sql).await?;
    c.execute(&st, p).await
}

/// Undangan + data internal yang tak boleh dikirim ke tamu.
pub struct InvRow {
    pub id: i64,
    pub inv: Invitation,
    /// SHA-256 kunci Kelola (kunci polos tak pernah disimpan).
    pub manage_key_hash: String,
    pub total_price: i64,
    pub payment_method: String,
    /// WA pemesan (62…) — hanya untuk admin, tak pernah dikirim ke tamu.
    pub contact_phone: String,
}

fn json<T: serde::de::DeserializeOwned + Default>(v: Value) -> T {
    serde_json::from_value(v).unwrap_or_default()
}

/// Baris dibaca sebagai `to_jsonb(invitations.*)` lalu di-deserialize dengan
/// nilai bawaan: kolom yang belum dimigrasi TIDAK membuat halaman error.
fn row_to_inv(r: &Row) -> InvRow {
    let j: Value = r.get("j");
    let s = |k: &str| j.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
    InvRow {
        id: j.get("id").and_then(|v| v.as_i64()).unwrap_or(0),
        manage_key_hash: s("manage_key_hash"),
        total_price: j.get("total_price").and_then(|v| v.as_i64()).unwrap_or(0),
        payment_method: s("payment_method"),
        contact_phone: s("contact_phone"),
        inv: serde_json::from_value(j.clone()).unwrap_or_default(),
    }
}

pub async fn invitation(pool: &Pool, slug: &str) -> Result<Option<InvRow>> {
    let c = pool.get().await?;
    let sql = "SELECT to_jsonb(i) AS j FROM invitations i WHERE slug = $1 AND status <> 'nonaktif'";
    Ok(db_opt(&c,sql, &[&slug]).await?.as_ref().map(row_to_inv))
}

pub async fn demo_invitation(pool: &Pool) -> Result<Option<InvRow>> {
    let c = pool.get().await?;
    let sql = "SELECT to_jsonb(i) AS j FROM invitations i WHERE is_demo ORDER BY id LIMIT 1";
    Ok(db_opt(&c,sql, &[]).await?.as_ref().map(row_to_inv))
}

/// Slug sudah dipakai undangan mana pun (termasuk nonaktif) — dicek SEBELUM
/// unggah, karena folder RustFS foto/{slug}/ & musik/{slug}/ ikut slug.
pub async fn slug_taken(pool: &Pool, slug: &str) -> Result<bool> {
    let c = pool.get().await?;
    Ok(db_opt(&c, "SELECT 1 FROM invitations WHERE slug = $1", &[&slug]).await?.is_some())
}

/// Pelanggaran UNIQUE (kode 23505) — mis. slug direbut pemesan lain.
pub fn is_unique_violation(e: &anyhow::Error) -> bool {
    e.chain()
        .filter_map(|c| c.downcast_ref::<tokio_postgres::Error>())
        .any(|pe| pe.code() == Some(&tokio_postgres::error::SqlState::UNIQUE_VIOLATION))
}

/// Tamu terdaftar via kode link pribadi; sekaligus mencatat waktu buka pertama.
pub async fn open_guest(pool: &Pool, inv_id: i64, code: &str) -> Result<Option<(i64, GuestInfo)>> {
    let c = pool.get().await?;
    let row = db_opt(&c,
        
            "UPDATE guests SET opened_at = COALESCE(opened_at, NOW())
             WHERE invitation_id = $1 AND code = $2
             RETURNING id, code, name, category, session, table_no, pax",
            &[&inv_id, &code.to_uppercase()],
        )
        .await?;
    Ok(row.map(|r| {
        (
            r.get("id"),
            GuestInfo {
                code: r.get("code"),
                name: r.get("name"),
                category: r.get("category"),
                session: r.get("session"),
                table_no: r.get("table_no"),
                pax: r.get("pax"),
            },
        )
    }))
}

pub async fn guest_id(pool: &Pool, inv_id: i64, code: &str) -> Result<Option<i64>> {
    let c = pool.get().await?;
    Ok(db_opt(&c,"SELECT id FROM guests WHERE invitation_id = $1 AND code = $2", &[&inv_id, &code.to_uppercase()])
        .await?
        .map(|r| r.get(0)))
}

pub async fn wishes(pool: &Pool, inv_id: i64, limit: i64) -> Result<WishPage> {
    let c = pool.get().await?;
    let total: i64 = db_row(&c,
        "SELECT COUNT(*) FROM rsvps WHERE invitation_id = $1 AND message <> ''", &[&inv_id])
        .await?
        .get(0);
    let rows = db_rows(&c,
        
            "SELECT name, status, message, EXTRACT(EPOCH FROM NOW() - created_at)::BIGINT AS age
             FROM rsvps WHERE invitation_id = $1 AND message <> ''
             ORDER BY created_at DESC LIMIT $2",
            &[&inv_id, &limit],
        )
        .await?;
    Ok(WishPage {
        total,
        items: rows
            .iter()
            .map(|r| Wish {
                name: r.get("name"),
                status: r.get("status"),
                message: r.get("message"),
                ago: lalu(r.get("age")),
            })
            .collect(),
    })
}

pub struct NewRsvp<'a> {
    pub inv_id: i64,
    pub guest_id: Option<i64>,
    pub name: &'a str,
    pub phone: &'a str,
    pub status: &'a str,
    pub pax: i32,
    pub session: &'a str,
    pub message: &'a str,
}

pub async fn upsert_rsvp(pool: &Pool, n: NewRsvp<'_>) -> Result<()> {
    let c = pool.get().await?;
    match n.guest_id {
        // Tamu terdaftar: satu RSVP per tamu, kirim ulang = perbarui.
        Some(gid) => {
            db_exec(&c,
                "INSERT INTO rsvps (invitation_id, guest_id, name, phone, status, pax, session, message)
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
                 ON CONFLICT (guest_id) WHERE guest_id IS NOT NULL DO UPDATE SET
                   name = EXCLUDED.name, phone = EXCLUDED.phone, status = EXCLUDED.status,
                   pax = EXCLUDED.pax, session = EXCLUDED.session,
                   message = CASE WHEN EXCLUDED.message = '' THEN rsvps.message ELSE EXCLUDED.message END,
                   created_at = NOW()",
                &[&n.inv_id, &gid, &n.name, &n.phone, &n.status, &n.pax, &n.session, &n.message],
            )
            .await?;
        }
        None => {
            db_exec(&c,
                "INSERT INTO rsvps (invitation_id, name, phone, status, pax, session, message)
                 VALUES ($1,$2,$3,$4,$5,$6,$7)",
                &[&n.inv_id, &n.name, &n.phone, &n.status, &n.pax, &n.session, &n.message],
            )
            .await?;
        }
    }
    Ok(())
}

pub async fn insert_gift(
    pool: &Pool,
    inv_id: i64,
    guest_id: Option<i64>,
    name: &str,
    amount: i64,
    channel: &str,
    note: &str,
) -> Result<()> {
    let c = pool.get().await?;
    db_exec(&c,
        "INSERT INTO gifts (invitation_id, guest_id, name, amount, channel, note) VALUES ($1,$2,$3,$4,$5,$6)",
        &[&inv_id, &guest_id, &name, &amount, &channel, &note],
    )
    .await?;
    Ok(())
}

// ── Dashboard pengantin ────────────────────────────────────────────────────

pub async fn guests(pool: &Pool, inv_id: i64) -> Result<Vec<GuestRow>> {
    let c = pool.get().await?;
    let rows = db_rows(&c,
        
            "SELECT g.code, g.name, g.phone, g.category, g.session, g.table_no, g.pax,
                    g.opened_at IS NOT NULL AS opened, g.checked_in_at IS NOT NULL AS checked_in,
                    EXTRACT(EPOCH FROM NOW() - g.sent_at)::BIGINT AS sent_age,
                    COALESCE(r.status, '') AS rsvp, COALESCE(r.pax, 0) AS rsvp_pax,
                    COALESCE((SELECT SUM(amount) FROM gifts WHERE guest_id = g.id), 0)::BIGINT AS gift
             FROM guests g
             LEFT JOIN rsvps r ON r.guest_id = g.id
             WHERE g.invitation_id = $1
             ORDER BY g.created_at DESC, g.id DESC",
            &[&inv_id],
        )
        .await?;
    Ok(rows
        .iter()
        .map(|r| GuestRow {
            code: r.get("code"),
            name: r.get("name"),
            phone: r.get("phone"),
            category: r.get("category"),
            session: r.get("session"),
            table_no: r.get("table_no"),
            pax: r.get("pax"),
            rsvp: r.get("rsvp"),
            rsvp_pax: r.get("rsvp_pax"),
            opened: r.get("opened"),
            checked_in: r.get("checked_in"),
            sent_ago: r.get::<_, Option<i64>>("sent_age").map(lalu).unwrap_or_default(),
            gift_amount: r.get("gift"),
        })
        .collect())
}

pub async fn stats(pool: &Pool, inv_id: i64) -> Result<Stats> {
    let c = pool.get().await?;
    let r = db_row(&c,
        
            // Satu pindaian per tabel (agregat FILTER), bukan 11 subquery.
            "SELECT g.*, r.*, f.* FROM
               (SELECT COUNT(*) AS total_guests, COALESCE(SUM(pax),0)::BIGINT AS total_pax,
                       COUNT(*) FILTER (WHERE sent_at IS NOT NULL) AS sent,
                       COUNT(*) FILTER (WHERE checked_in_at IS NOT NULL) AS checked_in
                  FROM guests WHERE invitation_id = $1) g,
               (SELECT COUNT(*) AS rsvp_count,
                       COUNT(*) FILTER (WHERE status = 'hadir') AS hadir_count,
                       COALESCE(SUM(pax) FILTER (WHERE status = 'hadir'),0)::BIGINT AS hadir_pax,
                       COUNT(*) FILTER (WHERE status = 'ragu') AS ragu_count,
                       COUNT(*) FILTER (WHERE status = 'tidak') AS tidak_count
                  FROM rsvps WHERE invitation_id = $1) r,
               (SELECT COALESCE(SUM(amount),0)::BIGINT AS gift_total, COUNT(*) AS gift_count
                  FROM gifts WHERE invitation_id = $1) f",
            &[&inv_id],
        )
        .await?;
    Ok(Stats {
        total_guests: r.get("total_guests"),
        total_pax: r.get("total_pax"),
        sent: r.get("sent"),
        checked_in: r.get("checked_in"),
        rsvp_count: r.get("rsvp_count"),
        hadir_count: r.get("hadir_count"),
        hadir_pax: r.get("hadir_pax"),
        ragu_count: r.get("ragu_count"),
        tidak_count: r.get("tidak_count"),
        gift_total: r.get("gift_total"),
        gift_count: r.get("gift_count"),
    })
}

/// Umpan aktivitas: RSVP, undangan dibuka, tanda kasih — 8 terbaru.
pub async fn activity(pool: &Pool, inv_id: i64) -> Result<Vec<Activity>> {
    let c = pool.get().await?;
    let rows = db_rows(&c,
        
            "SELECT kind, name, status, pax, amount, EXTRACT(EPOCH FROM NOW() - at)::BIGINT AS age FROM (
               SELECT 'rsvp' AS kind, name, status, pax, 0::BIGINT AS amount, created_at AS at
                 FROM rsvps WHERE invitation_id = $1
               UNION ALL
               SELECT 'open', name, '', 0, 0, opened_at FROM guests
                 WHERE invitation_id = $1 AND opened_at IS NOT NULL
               UNION ALL
               SELECT 'gift', name, '', 0, amount, created_at FROM gifts WHERE invitation_id = $1
             ) a ORDER BY at DESC LIMIT 8",
            &[&inv_id],
        )
        .await?;
    Ok(rows
        .iter()
        .map(|r| {
            let name: String = r.get("name");
            let text = match r.get::<_, &str>("kind") {
                "rsvp" => match r.get::<_, &str>("status") {
                    "hadir" => format!("{name} mengonfirmasi hadir ({} orang) dan mengirimkan doa restu.", r.get::<_, i32>("pax")),
                    "ragu" => format!("{name} masih ragu untuk hadir."),
                    _ => format!("{name} berhalangan hadir dan mengirim doa restu."),
                },
                "open" => format!("{name} membuka undangan digital dari tautan pribadinya."),
                _ => format!("{name} melaporkan tanda kasih {}.", crate::web::fmt::rupiah(r.get("amount"))),
            };
            Activity { text, ago: lalu(r.get("age")) }
        })
        .collect())
}

pub struct NewGuest<'a> {
    pub name: &'a str,
    pub phone: &'a str,
    pub category: &'a str,
    pub session: &'a str,
    pub table_no: &'a str,
    pub pax: i32,
}

/// Kode 6 karakter tanpa huruf/angka yang mirip (0/O, 1/I).
pub fn new_code() -> String {
    use rand::Rng;
    const A: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rng = rand::rng();
    (0..6).map(|_| A[rng.random_range(0..A.len())] as char).collect()
}

pub async fn add_guest(pool: &Pool, inv_id: i64, g: NewGuest<'_>) -> Result<String> {
    let c = pool.get().await?;
    for _ in 0..5 {
        let code = new_code();
        let n = db_exec(&c,
            
                "INSERT INTO guests (invitation_id, code, name, phone, category, session, table_no, pax)
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT (code) DO NOTHING",
                &[&inv_id, &code, &g.name, &g.phone, &g.category, &g.session, &g.table_no, &g.pax],
            )
            .await?;
        if n == 1 {
            return Ok(code);
        }
    }
    anyhow::bail!("gagal membuat kode tamu unik")
}

pub async fn delete_guest(pool: &Pool, inv_id: i64, code: &str) -> Result<()> {
    let c = pool.get().await?;
    db_exec(&c,"DELETE FROM guests WHERE invitation_id = $1 AND code = $2", &[&inv_id, &code]).await?;
    Ok(())
}

pub async fn mark_sent(pool: &Pool, inv_id: i64, code: &str) -> Result<()> {
    let c = pool.get().await?;
    db_exec(&c,"UPDATE guests SET sent_at = NOW() WHERE invitation_id = $1 AND code = $2", &[&inv_id, &code]).await?;
    Ok(())
}

/// Check-in QR. Mengembalikan (nama, sudah check-in sebelumnya?).
pub async fn check_in(pool: &Pool, inv_id: i64, code: &str) -> Result<Option<(String, bool)>> {
    let c = pool.get().await?;
    let code = code.trim().to_uppercase();
    // Hanya baris yang BELUM check-in yang diubah: dua pemindaian bersamaan →
    // tepat satu yang mendapat "baru", sisanya "sudah".
    if let Some(r) = db_opt(&c,
        
            "UPDATE guests SET checked_in_at = NOW()
             WHERE invitation_id = $1 AND code = $2 AND checked_in_at IS NULL
             RETURNING name",
            &[&inv_id, &code],
        )
        .await?
    {
        return Ok(Some((r.get(0), false)));
    }
    let r = db_opt(&c,"SELECT name FROM guests WHERE invitation_id = $1 AND code = $2", &[&inv_id, &code]).await?;
    Ok(r.map(|r| (r.get(0), true)))
}

// ── Checkout ───────────────────────────────────────────────────────────────

/// Data satu mempelai dari formulir pesan.
pub struct MempelaiInput {
    pub name: String,
    pub degree: String,
    pub nick: String,
    pub parents: String,
    pub ig: String,
    pub photo: String,
}

pub struct NewInvitation {
    pub slug: String,
    pub manage_key_hash: String,
    pub theme: String,
    pub package: String,
    pub bride: MempelaiInput,
    pub groom: MempelaiInput,
    pub events: Value,
    pub dress_code: String,
    pub quote_text: String,
    pub quote_source: String,
    pub music_title: String,
    pub music_artist: String,
    pub music_url: String,
    pub music_autoplay: bool,
    pub banks: Value,
    pub family_name: String,
    pub addons: Value,
    pub coupon: String,
    pub total_price: i64,
    pub payment_method: String,
    pub contact_phone: String,
    pub cover_photo: String,
    pub love_story: Value,
    pub live_url: String,
    pub gallery: Value,
    pub dress_colors: Value,
}

pub async fn create_invitation(pool: &Pool, n: &NewInvitation) -> Result<()> {
    let c = pool.get().await?;
    let (b, g) = (&n.bride, &n.groom);
    db_exec(&c,
        "INSERT INTO invitations (
            slug, manage_key_hash, theme, package,
            bride_name, bride_degree, bride_nick, bride_parents, bride_ig, bride_photo,
            groom_name, groom_degree, groom_nick, groom_parents, groom_ig, groom_photo,
            events, dress_code, quote_text, quote_source,
            music_title, music_artist, music_url, music_autoplay,
            banks, family_name, addons, coupon, total_price, payment_method, contact_phone,
            cover_photo, love_story, live_url, gallery, dress_colors
         ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,
                   $21,$22,$23,$24,$25,$26,$27,$28,$29,$30,$31,$32,$33,$34,$35,$36)",
        &[
            &n.slug, &n.manage_key_hash, &n.theme, &n.package,
            &b.name, &b.degree, &b.nick, &b.parents, &b.ig, &b.photo,
            &g.name, &g.degree, &g.nick, &g.parents, &g.ig, &g.photo,
            &n.events, &n.dress_code, &n.quote_text, &n.quote_source,
            &n.music_title, &n.music_artist, &n.music_url, &n.music_autoplay,
            &n.banks, &n.family_name, &n.addons, &n.coupon, &n.total_price, &n.payment_method, &n.contact_phone,
            &n.cover_photo, &n.love_story, &n.live_url, &n.gallery, &n.dress_colors,
        ],
    )
    .await
    .context("insert invitation")?;
    Ok(())
}

// ── Tema (katalog di DB) ───────────────────────────────────────────────────

pub async fn themes(pool: &Pool) -> Result<Vec<crate::web::skin::ThemeInfo>> {
    let c = pool.get().await?;
    // to_jsonb: kolom tema yang belum dimigrasi → nilai bawaan, bukan error.
    let rows = db_rows(&c,
        
            "SELECT to_jsonb(t) || jsonb_build_object('created', EXTRACT(EPOCH FROM created_at)::BIGINT) AS j
             FROM themes t ORDER BY sort_order, created_at",
            &[],
        )
        .await
        .context("select themes")?;
    Ok(rows.iter().filter_map(|r| serde_json::from_value(r.get::<_, Value>("j")).ok()).collect())
}

/// Kolom tema yang disimpan dari editor admin — SATU daftar untuk INSERT dan
/// UPDATE. Nilai diambil per NAMA dari JSON `ThemeInfo` lewat
/// `jsonb_populate_record` (tak ada lagi 31 parameter posisional yang bisa
/// tertukar urutannya saat kolom bertambah).
const THEME_COLS: &[&str] = &[
    "slug", "name", "category", "nuansa", "palette", "region", "description", "tags", "badge", "rating", "reviews",
    "layout", "ornament", "font", "tokens", "dark", "image_url", "image_mode", "listed", "sort_order",
    "script_font", "bg_image", "frame_image", "card_deco", "float_deco", "open_anim", "page_mode", "scroll_anim",
    "gerak_judul", "gerak_foto", "ken_burns",
];

fn upsert_theme_sql() -> &'static str {
    static SQL: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    SQL.get_or_init(|| {
        let cols = THEME_COLS.join(", ");
        let set: Vec<String> = THEME_COLS.iter().filter(|c| **c != "slug").map(|c| format!("{c} = EXCLUDED.{c}")).collect();
        format!(
            "INSERT INTO themes ({cols}) SELECT {cols} FROM jsonb_populate_record(NULL::themes, $1)
             ON CONFLICT (slug) DO UPDATE SET {}, updated_at = NOW()",
            set.join(", ")
        )
    })
}

pub async fn upsert_theme(pool: &Pool, t: &crate::web::skin::ThemeInfo) -> Result<()> {
    let c = pool.get().await?;
    let json = serde_json::to_value(t)?;
    db_exec(&c, upsert_theme_sql(), &[&json]).await.context("upsert theme")?;
    Ok(())
}

pub async fn theme_exists(pool: &Pool, slug: &str) -> Result<bool> {
    let c = pool.get().await?;
    Ok(db_opt(&c,"SELECT 1 FROM themes WHERE slug = $1", &[&slug]).await?.is_some())
}

/// slug → jumlah undangan yang memakai tema itu.
pub async fn theme_usage(pool: &Pool) -> Result<std::collections::HashMap<String, i64>> {
    let c = pool.get().await?;
    let rows = db_rows(&c,"SELECT theme, COUNT(*) AS n FROM invitations GROUP BY theme", &[]).await?;
    Ok(rows.iter().map(|r| (r.get("theme"), r.get("n"))).collect())
}

/// Hapus tema yang belum dipakai undangan mana pun. `false` = masih dipakai
/// (termasuk bila checkout bersamaan memakainya — dijaga FK migrasi 004).
pub async fn delete_theme(pool: &Pool, slug: &str) -> Result<bool> {
    let c = pool.get().await?;
    match db_exec(&c,
        
            "DELETE FROM themes WHERE slug = $1 AND NOT EXISTS (SELECT 1 FROM invitations WHERE theme = $1)",
            &[&slug],
        )
        .await
    {
        Ok(n) => Ok(n > 0),
        Err(e) if e.code() == Some(&tokio_postgres::error::SqlState::FOREIGN_KEY_VIOLATION) => Ok(false),
        Err(e) => Err(e.into()),
    }
}

/// Ganti kunci Kelola (kunci lama langsung tak berlaku). Demo tak bisa.
pub async fn reset_manage_key(pool: &Pool, slug: &str, hash: &str) -> Result<bool> {
    let c = pool.get().await?;
    let n = db_exec(&c,
        "UPDATE invitations SET manage_key_hash = $2, updated_at = NOW() WHERE slug = $1 AND NOT is_demo", &[&slug, &hash])
        .await?;
    Ok(n > 0)
}

// ── Admin: pesanan ─────────────────────────────────────────────────────────

pub async fn admin_invitations(pool: &Pool, q: &str, ttl_hours: i64) -> Result<Vec<AdminInv>> {
    let c = pool.get().await?;
    // $1 = kata kunci apa adanya (kosong = semua), $3 = pola ILIKE yang aman.
    let q = q.replace(['%', '_'], "");
    let like = format!("%{q}%");
    let rows = db_rows(&c,
        
            "SELECT slug, bride_name, groom_name, theme, package, status, total_price, payment_method,
                    contact_phone, is_demo, to_char(created_at AT TIME ZONE 'Asia/Jakarta', 'YYYY-MM-DD HH24:MI') AS created,
                    -- Kolom bukti (017) dibaca lewat jsonb: belum dimigrasi → kosong, bukan galat.
                    COALESCE(to_jsonb(i) ->> 'payment_proof', '') AS payment_proof,
                    to_char((to_jsonb(i) ->> 'payment_proof_at')::timestamptz AT TIME ZONE 'Asia/Jakarta', 'YYYY-MM-DD HH24:MI') AS proof_at,
                    CASE WHEN status = 'menunggu_pembayaran' AND NOT is_demo AND to_jsonb(i) ->> 'payment_proof_at' IS NULL
                         THEN GREATEST(0, EXTRACT(EPOCH FROM (created_at + make_interval(hours => $2::int) - NOW())) / 60)::BIGINT
                    END AS minutes_left
             FROM invitations i
             WHERE $1 = '' OR slug ILIKE $3 OR bride_name ILIKE $3 OR groom_name ILIKE $3 OR contact_phone ILIKE $3
             ORDER BY (status = 'menunggu_pembayaran' AND to_jsonb(i) ->> 'payment_proof_at' IS NOT NULL) DESC,
                      (status = 'menunggu_pembayaran') DESC, created_at DESC
             LIMIT 200",
            &[&q, &(ttl_hours as i32), &like],
        )
        .await
        .context("admin invitations")?;
    Ok(rows
        .iter()
        .map(|r| AdminInv {
            slug: r.get("slug"),
            package_name: String::new(),
            couple: format!("{} & {}", r.get::<_, String>("bride_name"), r.get::<_, String>("groom_name")),
            theme: r.get("theme"),
            package: r.get("package"),
            status: r.get("status"),
            total_price: r.get("total_price"),
            payment_method: r.get("payment_method"),
            contact_phone: r.get("contact_phone"),
            is_demo: r.get("is_demo"),
            created: r.get("created"),
            minutes_left: r.get("minutes_left"),
            payment_proof: r.get("payment_proof"),
            proof_at: r.get::<_, Option<String>>("proof_at").unwrap_or_default(),
        })
        .collect())
}

/// Bukti transfer pemesan: (URL gambar, "x menit lalu"). Belum ada → None.
/// Kolom belum dimigrasi (017) → Err; pemanggil memperlakukannya "belum ada".
pub async fn payment_proof(pool: &Pool, inv_id: i64) -> Result<Option<(String, String)>> {
    let c = pool.get().await?;
    let r = db_opt(&c,
        "SELECT payment_proof, EXTRACT(EPOCH FROM NOW() - payment_proof_at)::BIGINT AS age
           FROM invitations WHERE id = $1 AND payment_proof_at IS NOT NULL",
        &[&inv_id],
    )
    .await?;
    Ok(r.map(|r| (r.get("payment_proof"), lalu(r.get("age")))))
}

/// Simpan bukti transfer (hanya pesanan yang masih menunggu pembayaran).
/// Mengembalikan URL bukti LAMA (untuk dihapus dari RustFS) bila diganti;
/// `None` = undangan tak ditemukan / sudah tidak menunggu pembayaran.
pub async fn set_payment_proof(pool: &Pool, inv_id: i64, url: &str) -> Result<Option<String>> {
    let c = pool.get().await?;
    let r = db_opt(&c,
        "UPDATE invitations i SET payment_proof = $2, payment_proof_at = NOW(), updated_at = NOW()
           FROM (SELECT id, payment_proof AS lama FROM invitations WHERE id = $1 FOR UPDATE) o
          WHERE i.id = o.id AND i.status = 'menunggu_pembayaran' AND NOT i.is_demo
          RETURNING o.lama",
        &[&inv_id, &url],
    )
    .await?;
    Ok(r.map(|r| r.get(0)))
}

pub async fn admin_update_invitation(pool: &Pool, slug: &str, status: &str, theme: &str) -> Result<bool> {
    let c = pool.get().await?;
    let n = db_exec(&c,
        
            "UPDATE invitations SET status = $2, theme = $3, updated_at = NOW() WHERE slug = $1 AND NOT is_demo",
            &[&slug, &status, &theme],
        )
        .await?;
    Ok(n > 0)
}

// ── Konten situs (site_content) ────────────────────────────────────────────

pub async fn content_rows(pool: &Pool) -> Result<Vec<(String, Value)>> {
    let c = pool.get().await?;
    let rows = db_rows(&c,"SELECT key, data FROM site_content", &[]).await.context("select site_content")?;
    Ok(rows.iter().map(|r| (r.get("key"), r.get("data"))).collect())
}

pub async fn save_content(pool: &Pool, key: &str, data: &Value, by: &str) -> Result<()> {
    let c = pool.get().await?;
    db_exec(&c,
        "INSERT INTO site_content (key, data, updated_by) VALUES ($1, $2, $3)
         ON CONFLICT (key) DO UPDATE SET data = EXCLUDED.data, updated_by = EXCLUDED.updated_by, updated_at = NOW()",
        &[&key, data, &by],
    )
    .await
    .context("save site_content")?;
    Ok(())
}

/// kunci → "nama • 2026-09-30 14:05" (siapa & kapan terakhir disunting).
pub async fn content_meta(pool: &Pool) -> Result<std::collections::HashMap<String, String>> {
    let c = pool.get().await?;
    let rows = db_rows(&c,
        
            "SELECT key, updated_by || ' • ' || to_char(updated_at AT TIME ZONE 'Asia/Jakarta', 'YYYY-MM-DD HH24:MI') AS m FROM site_content",
            &[],
        )
        .await?;
    Ok(rows.iter().map(|r| (r.get("key"), r.get("m"))).collect())
}

// ── Akun admin & sesi ──────────────────────────────────────────────────────

/// Kolom akun admin — SEMUA query memakai alias tabel `u` (admin_users u),
/// jadi satu konstanta berlaku juga untuk JOIN dengan admin_sessions.
const ADMIN_COLS: &str = "u.id, u.username, u.name, u.role, u.active,
    COALESCE(to_char(u.last_login_at AT TIME ZONE 'Asia/Jakarta', 'YYYY-MM-DD HH24:MI'), '') AS last_login";

fn row_to_admin(r: &Row) -> AdminUser {
    AdminUser {
        id: r.get("id"),
        username: r.get("username"),
        name: r.get("name"),
        role: r.get("role"),
        active: r.get("active"),
        last_login: r.get("last_login"),
    }
}

pub async fn admin_count(pool: &Pool) -> Result<i64> {
    let c = pool.get().await?;
    Ok(db_row(&c,"SELECT COUNT(*) FROM admin_users", &[]).await?.get(0))
}

pub async fn admins(pool: &Pool) -> Result<Vec<AdminUser>> {
    let c = pool.get().await?;
    let sql = format!("SELECT {ADMIN_COLS} FROM admin_users u ORDER BY u.active DESC, u.role, u.username");
    Ok(db_rows(&c,&sql, &[]).await?.iter().map(row_to_admin).collect())
}

/// (user, hash sandi) untuk login.
pub async fn admin_login_row(pool: &Pool, username: &str) -> Result<Option<(AdminUser, String)>> {
    let c = pool.get().await?;
    let sql = format!("SELECT {ADMIN_COLS}, u.password_hash FROM admin_users u WHERE u.username = $1");
    Ok(db_opt(&c,&sql, &[&username]).await?.map(|r| (row_to_admin(&r), r.get("password_hash"))))
}

pub async fn create_admin(pool: &Pool, username: &str, name: &str, hash: &str, role: &str) -> Result<i64> {
    let c = pool.get().await?;
    Ok(db_row(&c,
        
            "INSERT INTO admin_users (username, name, password_hash, role) VALUES ($1, $2, $3, $4) RETURNING id",
            &[&username, &name, &hash, &role],
        )
        .await
        .context("insert admin")?
        .get(0))
}

pub async fn update_admin(pool: &Pool, id: i64, name: &str, role: &str, active: bool) -> Result<()> {
    let c = pool.get().await?;
    db_exec(&c,"UPDATE admin_users SET name = $2, role = $3, active = $4 WHERE id = $1", &[&id, &name, &role, &active]).await?;
    if !active {
        db_exec(&c,"DELETE FROM admin_sessions WHERE user_id = $1", &[&id]).await?;
    }
    Ok(())
}

/// Ganti sandi & keluarkan semua sesi akun itu kecuali `keep` (hash token).
pub async fn set_admin_password(pool: &Pool, id: i64, hash: &str, keep: Option<&str>) -> Result<()> {
    // Ganti sandi & keluarkan sesi lain secara atomik (satu pernyataan).
    let c = pool.get().await?;
    db_exec(
        &c,
        "WITH u AS (UPDATE admin_users SET password_hash = $2 WHERE id = $1 RETURNING id)
         DELETE FROM admin_sessions s USING u WHERE s.user_id = u.id AND s.token_hash <> COALESCE($3, '')",
        &[&id, &hash, &keep],
    )
    .await?;
    Ok(())
}

/// Jumlah akun admin aktif selain `except` — cegah panel tanpa admin.
pub async fn other_active_admins(pool: &Pool, except: i64) -> Result<i64> {
    let c = pool.get().await?;
    Ok(db_row(&c,
        "SELECT COUNT(*) FROM admin_users WHERE role = 'admin' AND active AND id <> $1", &[&except])
        .await?
        .get(0))
}

pub async fn create_session(pool: &Pool, token_hash: &str, user_id: i64, days: i64) -> Result<()> {
    let c = pool.get().await?;
    db_exec(&c,
        "INSERT INTO admin_sessions (token_hash, user_id, expires_at) VALUES ($1, $2, NOW() + make_interval(days => $3::int))",
        &[&token_hash, &user_id, &(days as i32)],
    )
    .await?;
    db_exec(&c,"UPDATE admin_users SET last_login_at = NOW() WHERE id = $1", &[&user_id]).await?;
    Ok(())
}

/// Hapus sesi kedaluwarsa — dijalankan tugas latar (server/cleanup.rs),
/// jadi tabel tetap bersih walau lama tak ada yang login.
pub async fn purge_expired_sessions(pool: &Pool) -> Result<u64> {
    let c = pool.get().await?;
    Ok(db_exec(&c, "DELETE FROM admin_sessions WHERE expires_at < NOW()", &[]).await?)
}

pub async fn session_user(pool: &Pool, token_hash: &str) -> Result<Option<AdminUser>> {
    let c = pool.get().await?;
    let sql = format!(
        "SELECT {ADMIN_COLS} FROM admin_sessions s JOIN admin_users u ON u.id = s.user_id
         WHERE s.token_hash = $1 AND s.expires_at > NOW() AND u.active"
    );
    Ok(db_opt(&c,&sql, &[&token_hash]).await?.as_ref().map(row_to_admin))
}

pub async fn delete_session(pool: &Pool, token_hash: &str) -> Result<()> {
    let c = pool.get().await?;
    db_exec(&c,"DELETE FROM admin_sessions WHERE token_hash = $1", &[&token_hash]).await?;
    Ok(())
}

// ── Pembersihan pesanan belum dibayar ──────────────────────────────────────

/// Undangan yang dihapus + URL file unggahannya (untuk dibersihkan di RustFS).
pub struct Purged {
    pub slug: String,
    pub files: Vec<String>,
}

/// Hapus ATOMIK undangan yang masih `menunggu_pembayaran` lebih dari `hours`
/// jam (tamu, RSVP, tanda kasih ikut terhapus lewat ON DELETE CASCADE).
/// Admin yang mengaktifkan di detik yang sama menang: syarat status dicek di
/// pernyataan DELETE itu sendiri.
pub async fn purge_unpaid(pool: &Pool, hours: i64) -> Result<Vec<Purged>> {
    let c = pool.get().await?;
    let rows = db_rows(&c,
        
            "DELETE FROM invitations
             WHERE status = 'menunggu_pembayaran' AND NOT is_demo
               -- Sudah kirim bukti transfer → jangan dihapus (lewat jsonb: aman sebelum 017).
               AND to_jsonb(invitations) ->> 'payment_proof_at' IS NULL
               AND created_at < NOW() - make_interval(hours => $1::int)
             RETURNING to_jsonb(invitations) AS j",
            &[&(hours as i32)],
        )
        .await
        .context("purge unpaid")?;
    Ok(rows
        .iter()
        .map(|r| {
            let j: Value = r.get("j");
            let s = |k: &str| j.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let mut files: Vec<String> =
                ["bride_photo", "groom_photo", "music_url", "cover_photo", "payment_proof"].iter().map(|c| s(c)).filter(|u| !u.is_empty()).collect();
            files.extend(j.get("gallery").cloned().map(json::<Vec<String>>).unwrap_or_default());
            Purged { slug: s("slug"), files }
        })
        .collect())
}

/// Menit tersisa sebelum pesanan belum-dibayar dihapus (None = sudah aktif/demo).
pub async fn unpaid_minutes_left(pool: &Pool, inv_id: i64, hours: i64) -> Result<Option<i64>> {
    let c = pool.get().await?;
    let r = db_opt(&c,
        
            "SELECT GREATEST(0, EXTRACT(EPOCH FROM (created_at + make_interval(hours => $2::int) - NOW())) / 60)::BIGINT
             FROM invitations WHERE id = $1 AND status = 'menunggu_pembayaran' AND NOT is_demo
               AND to_jsonb(invitations) ->> 'payment_proof_at' IS NULL",
            &[&inv_id, &(hours as i32)],
        )
        .await?;
    Ok(r.map(|r| r.get(0)))
}

// ── Animasi kustom (animations) ────────────────────────────────────────────

pub async fn animations(pool: &Pool) -> Result<Vec<crate::web::anim::AnimInfo>> {
    let c = pool.get().await?;
    let rows = db_rows(&c,"SELECT to_jsonb(a) AS j FROM animations a", &[]).await.context("select animations")?;
    Ok(rows
        .iter()
        .filter_map(|r| {
            let j = r.get::<_, Value>("j");
            // Tabel versi 007 (belum migrasi 008): kunci tema = "k-" + slug.
            let legacy = j.get("builtin").is_none();
            let mut a: crate::web::anim::AnimInfo = serde_json::from_value(j).ok()?;
            if legacy && !a.slug.starts_with("k-") {
                a.slug = format!("k-{}", a.slug);
            }
            Some(a)
        })
        .collect())
}

pub async fn upsert_animation(pool: &Pool, a: &crate::web::anim::AnimInfo) -> Result<()> {
    let c = pool.get().await?;
    let spec = serde_json::to_value(&a.spec)?;
    db_exec(&c,
        "INSERT INTO animations (kind, slug, name, spec, css, builtin, sort_order) VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (kind, slug) DO UPDATE SET name = EXCLUDED.name, spec = EXCLUDED.spec, css = EXCLUDED.css,
             builtin = EXCLUDED.builtin, sort_order = EXCLUDED.sort_order, updated_at = NOW()",
        &[&a.kind, &a.slug, &a.name, &spec, &a.css, &a.builtin, &a.sort_order],
    )
    .await
    .context("upsert animation")?;
    Ok(())
}

/// Isi animasi bawaan yang belum ada (suntingan admin TIDAK ditimpa).
/// Dipanggil saat start; tabel belum migrasi 008 → error diabaikan pemanggil.
pub async fn seed_animations(pool: &Pool, list: &[crate::web::anim::AnimInfo]) -> Result<u64> {
    // Satu pernyataan untuk semua animasi bawaan (bukan satu INSERT per baris).
    let rows: Vec<Value> = list
        .iter()
        .map(|a| serde_json::json!({ "kind": a.kind, "slug": a.slug, "name": a.name, "spec": a.spec, "css": a.css, "sort_order": a.sort_order }))
        .collect();
    let c = pool.get().await?;
    Ok(db_exec(
        &c,
        "INSERT INTO animations (kind, slug, name, spec, css, builtin, sort_order)
         SELECT x.kind, x.slug, x.name, x.spec, x.css, TRUE, x.sort_order
           FROM jsonb_to_recordset($1) AS x(kind TEXT, slug TEXT, name TEXT, spec JSONB, css TEXT, sort_order INT)
         ON CONFLICT (kind, slug) DO NOTHING",
        &[&Value::Array(rows)],
    )
    .await
    .context("seed animations")?)
}

/// Tema yang memakai animasi `key` jenis `kind` — animasi dipakai tak boleh dihapus.
pub async fn animation_users(pool: &Pool, kind: &str, key: &str) -> Result<Vec<String>> {
    let col = match kind {
        "buka" => "open_anim",
        "scroll" => "scroll_anim",
        _ => "float_deco",
    };
    let c = pool.get().await?;
    let rows = db_rows(&c,&format!("SELECT name FROM themes WHERE {col} = $1 ORDER BY name"), &[&key]).await?;
    Ok(rows.iter().map(|r| r.get(0)).collect())
}

pub async fn delete_animation(pool: &Pool, kind: &str, slug: &str) -> Result<()> {
    let c = pool.get().await?;
    db_exec(&c,"DELETE FROM animations WHERE kind = $1 AND slug = $2 AND NOT builtin", &[&kind, &slug]).await?;
    Ok(())
}

// ── Ornamen tema (tabel theme_ornaments) ─────────────────────────────────

pub async fn ornaments(pool: &Pool) -> Result<Vec<crate::web::ornamen::Ornament>> {
    let c = pool.get().await?;
    let rows = db_rows(&c,
        "SELECT to_jsonb(o) AS j FROM theme_ornaments o ORDER BY theme, bagian, urutan, id", &[])
        .await
        .context("select theme_ornaments")?;
    Ok(rows.iter().filter_map(|r| serde_json::from_value(r.get::<_, Value>("j")).ok()).collect())
}

/// Simpan ornamen (id 0 = baru). Mengembalikan id.
pub async fn save_ornament(pool: &Pool, o: &crate::web::ornamen::Ornament) -> Result<i64> {
    let c = pool.get().await?;
    let p: [&(dyn tokio_postgres::types::ToSql + Sync); 19] = [
        &o.theme, &o.bagian, &o.img, &o.posisi, &o.x, &o.y, &o.lebar, &o.rotasi, &o.cermin, &o.masuk, &o.jeda,
        &o.durasi, &o.gerak, &o.kecepatan, &o.depan, &o.hp, &o.opasitas, &o.urutan, &o.id,
    ];
    let row = if o.id == 0 {
        db_row(&c,
            "INSERT INTO theme_ornaments (theme, bagian, img, posisi, x, y, lebar, rotasi, cermin, masuk, jeda,
                                          durasi, gerak, kecepatan, depan, hp, opasitas, urutan)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18)
             RETURNING id",
            &p[..18],
        )
        .await
    } else {
        db_row(&c,
            "UPDATE theme_ornaments SET bagian=$2, img=$3, posisi=$4, x=$5, y=$6, lebar=$7, rotasi=$8, cermin=$9,
                    masuk=$10, jeda=$11, durasi=$12, gerak=$13, kecepatan=$14, depan=$15, hp=$16, opasitas=$17,
                    urutan=$18, updated_at=NOW()
              WHERE id = $19 AND theme = $1 RETURNING id",
            &p,
        )
        .await
    }
    .context("simpan ornamen")?;
    Ok(row.get(0))
}

pub async fn delete_ornament(pool: &Pool, theme: &str, id: i64) -> Result<()> {
    let c = pool.get().await?;
    db_exec(&c,"DELETE FROM theme_ornaments WHERE id = $1 AND theme = $2", &[&id, &theme]).await?;
    Ok(())
}

/// Salin semua ornamen tema `from` ke tema `to` (ditambahkan, tak menimpa).
pub async fn copy_ornaments(pool: &Pool, from: &str, to: &str) -> Result<u64> {
    let c = pool.get().await?;
    Ok(db_exec(&c,
        "INSERT INTO theme_ornaments (theme, bagian, img, posisi, x, y, lebar, rotasi, cermin, masuk, jeda,
                                      durasi, gerak, kecepatan, depan, hp, opasitas, urutan)
         SELECT $2, bagian, img, posisi, x, y, lebar, rotasi, cermin, masuk, jeda, durasi, gerak, kecepatan,
                depan, hp, opasitas, urutan
           FROM theme_ornaments WHERE theme = $1 ORDER BY bagian, urutan, id",
        &[&from, &to],
    )
    .await?)
}

// ── Banner beranda (tabel banners) ────────────────────────────────────────

const BANNER_COLS: &str = "id, judul, sub, cta, link, img, img_hp, aktif, urutan,
    COALESCE(to_char(mulai AT TIME ZONE 'Asia/Jakarta', 'YYYY-MM-DD\"T\"HH24:MI'), '') AS mulai,
    COALESCE(to_char(selesai AT TIME ZONE 'Asia/Jakarta', 'YYYY-MM-DD\"T\"HH24:MI'), '') AS selesai,
    CASE WHEN NOT aktif THEN 'nonaktif'
         WHEN mulai IS NOT NULL AND mulai > NOW() THEN 'terjadwal'
         WHEN selesai IS NOT NULL AND selesai <= NOW() THEN 'berakhir'
         ELSE 'tayang' END AS status";

fn banner_row(r: &tokio_postgres::Row) -> crate::web::model::Banner {
    crate::web::model::Banner {
        id: r.get("id"),
        judul: r.get("judul"),
        sub: r.get("sub"),
        cta: r.get("cta"),
        link: r.get("link"),
        img: r.get("img"),
        img_hp: r.get("img_hp"),
        aktif: r.get("aktif"),
        urutan: r.get("urutan"),
        mulai: r.get("mulai"),
        selesai: r.get("selesai"),
        status: r.get("status"),
    }
}

/// Banner yang sedang tayang (aktif & dalam jadwal), urut.
pub async fn banners_live(pool: &Pool) -> Result<Vec<crate::web::model::Banner>> {
    let c = pool.get().await?;
    let rows = db_rows(&c,
        
            &format!(
                "SELECT {BANNER_COLS} FROM banners
                  WHERE aktif AND img <> '' AND (mulai IS NULL OR mulai <= NOW()) AND (selesai IS NULL OR selesai > NOW())
                  ORDER BY urutan, id"
            ),
            &[],
        )
        .await
        .context("select banners")?;
    Ok(rows.iter().map(banner_row).collect())
}

pub async fn banners_all(pool: &Pool) -> Result<Vec<crate::web::model::Banner>> {
    let c = pool.get().await?;
    let rows = db_rows(&c,&format!("SELECT {BANNER_COLS} FROM banners ORDER BY urutan, id"), &[]).await.context("select banners")?;
    Ok(rows.iter().map(banner_row).collect())
}

/// Simpan (id 0 = baru, urutan paling akhir). Jadwal = `YYYY-MM-DDTHH:MM` WIB / kosong.
pub async fn save_banner(pool: &Pool, b: &crate::web::model::Banner) -> Result<i64> {
    let c = pool.get().await?;
    let row = if b.id == 0 {
        db_row(&c,
            "INSERT INTO banners (judul, sub, cta, link, img, img_hp, aktif, urutan, mulai, selesai)
             VALUES ($1, $2, $3, $4, $5, $6, $7, (SELECT COALESCE(MAX(urutan), 0) + 10 FROM banners),
                     NULLIF($8, '')::timestamp AT TIME ZONE 'Asia/Jakarta', NULLIF($9, '')::timestamp AT TIME ZONE 'Asia/Jakarta')
             RETURNING id",
            &[&b.judul, &b.sub, &b.cta, &b.link, &b.img, &b.img_hp, &b.aktif, &b.mulai, &b.selesai],
        )
        .await
    } else {
        db_row(&c,
            "UPDATE banners SET judul = $2, sub = $3, cta = $4, link = $5, img = $6, img_hp = $7, aktif = $8, urutan = $9,
                    mulai = NULLIF($10, '')::timestamp AT TIME ZONE 'Asia/Jakarta',
                    selesai = NULLIF($11, '')::timestamp AT TIME ZONE 'Asia/Jakarta', updated_at = NOW()
              WHERE id = $1 RETURNING id",
            &[&b.id, &b.judul, &b.sub, &b.cta, &b.link, &b.img, &b.img_hp, &b.aktif, &b.urutan, &b.mulai, &b.selesai],
        )
        .await
    }
    .context("simpan banner")?;
    Ok(row.get(0))
}

pub async fn delete_banner(pool: &Pool, id: i64) -> Result<()> {
    let c = pool.get().await?;
    db_exec(&c,"DELETE FROM banners WHERE id = $1", &[&id]).await?;
    Ok(())
}

/// Geser satu banner naik/turun lalu rapikan urutan menjadi 10, 20, 30, …
pub async fn move_banner(pool: &Pool, id: i64, up: bool) -> Result<()> {
    let mut c = pool.get().await?;
    let tx = c.transaction().await?;
    let mut ids: Vec<i64> = tx.query("SELECT id FROM banners ORDER BY urutan, id FOR UPDATE", &[]).await?.iter().map(|r| r.get(0)).collect();
    if let Some(i) = ids.iter().position(|x| *x == id) {
        let j = if up { i.checked_sub(1) } else { (i + 1 < ids.len()).then_some(i + 1) };
        if let Some(j) = j {
            ids.swap(i, j);
        }
    }
    // Rapikan urutan 10, 20, 30, … dalam SATU pernyataan.
    tx.execute(
        "UPDATE banners b SET urutan = (v.n * 10)::INT FROM unnest($1::BIGINT[]) WITH ORDINALITY AS v(id, n) WHERE b.id = v.id",
        &[&ids],
    )
    .await?;
    tx.commit().await?;
    Ok(())
}
