//! web/api.rs — server function (dipanggil SSR langsung, dari WASM via /api/*).

use leptos::prelude::*;

use super::model::*;

#[cfg(feature = "ssr")]
mod srv {
    use std::sync::Arc;

    use leptos::prelude::*;

    pub use crate::server::handlers::clean;
    pub use crate::server::repo::{self, InvRow};
    pub use crate::server::state::AppState;

    pub fn state() -> Result<Arc<AppState>, ServerFnError> {
        use_context::<Arc<AppState>>().ok_or_else(|| ServerFnError::new("state tidak tersedia"))
    }

    pub fn internal(e: anyhow::Error) -> ServerFnError {
        tracing::error!(error = %format!("{e:#}"), "server fn");
        ServerFnError::new("Server sedang sibuk, coba lagi sebentar.")
    }

    /// 404 halaman SSR bila ResponseOptions tersedia (bisa absen — jangan
    /// expect_context, lihat catatan ppm).
    pub fn not_found() -> ServerFnError {
        if let Some(r) = use_context::<leptos_axum::ResponseOptions>() {
            r.set_status(axum::http::StatusCode::NOT_FOUND);
        }
        ServerFnError::new("NOT_FOUND")
    }

    pub async fn load(slug: &str) -> Result<InvRow, ServerFnError> {
        let st = state()?;
        repo::invitation(&st.pool, slug).await.map_err(internal)?.ok_or_else(not_found)
    }

    /// Kunci Kelola: yang dikirim klien, atau (umumnya) dari cookie
    /// `ily_k_{slug}` hasil tukar tautan khusus (server/owner.rs).
    pub async fn owner_key(slug: &str, key: &str) -> Result<String, ServerFnError> {
        let key = key.trim();
        if !key.is_empty() {
            return Ok(key.to_string());
        }
        let headers: axum::http::HeaderMap = leptos_axum::extract().await?;
        Ok(crate::server::owner::key_from(&headers, slug).unwrap_or_default())
    }

    /// Undangan + verifikasi kunci kelola. Demo boleh dibuka dengan kunci "demo".
    pub async fn load_owned(slug: &str, key: &str) -> Result<InvRow, ServerFnError> {
        let key = &owner_key(slug, key).await?;
        let row = load(slug).await?;
        if key.is_empty() || !crate::server::auth::same_hash(&crate::server::auth::token_hash(key), &row.manage_key_hash) {
            return Err(ServerFnError::new("Kunci kelola tidak valid. Buka dari tautan yang Anda terima saat memesan."));
        }
        Ok(row)
    }

    /// Undangan DEMO boleh ditampilkan dengan tema pilihan pengunjung
    /// (`?tema=`); undangan asli selalu memakai temanya sendiri.
    pub fn apply_demo_theme(st: &AppState, row: &mut InvRow, tema: Option<&str>) {
        if row.inv.is_demo {
            if let Some(t) = tema.map(str::trim).filter(|t| st.themes().get(t).is_some()) {
                row.inv.theme = t.to_string();
            }
        }
    }

    /// Admin yang sedang masuk (header request ini), tanpa cek peran.
    pub async fn current_admin(st: &AppState) -> Result<Option<crate::web::model::AdminUser>, ServerFnError> {
        let headers: axum::http::HeaderMap = leptos_axum::extract().await?;
        Ok(crate::server::auth::current(st, &headers).await)
    }

    /// Tamu tak bisa RSVP / kirim tanda kasih ke undangan yang belum aktif.
    pub fn deny_locked(row: &InvRow) -> Result<(), ServerFnError> {
        if row.inv.is_locked() {
            Err(ServerFnError::new("Undangan ini belum diaktifkan — RSVP & ucapan dibuka setelah pembayaran dikonfirmasi."))
        } else {
            Ok(())
        }
    }

    pub fn deny_demo(row: &InvRow) -> Result<(), ServerFnError> {
        if row.inv.is_demo {
            Err(ServerFnError::new("Ini undangan demo — perubahan tidak disimpan. Pesan tema untuk mencoba penuh."))
        } else {
            Ok(())
        }
    }
}

/// Undangan untuk tamu. Belum dibayar → terkunci ("LOCKED", tanpa data apa
/// pun) kecuali dibuka pemilik dengan kunci Kelola (`k`) sebagai pratinjau.
/// `tema` hanya berlaku untuk undangan DEMO ("Coba Demo" dari halaman tema):
/// isi demo ditampilkan dengan tema itu. Undangan asli selalu memakai temanya.
#[server]
pub async fn get_invitation(slug: String, guest: Option<String>, k: Option<String>, tema: Option<String>) -> Result<InvitationPage, ServerFnError> {
    use srv::*;
    let st = state()?;
    let mut row = load(&slug).await?;
    apply_demo_theme(&st, &mut row, tema.as_deref());
    let preview = row.inv.is_locked();
    if preview {
        let k = owner_key(&slug, k.as_deref().unwrap_or("")).await?;
        let owner = Some(k.as_str()).filter(|k| !k.is_empty()).is_some_and(|k| {
            crate::server::auth::same_hash(&crate::server::auth::token_hash(k.trim()), &row.manage_key_hash)
        });
        if !owner {
            if let Some(r) = use_context::<leptos_axum::ResponseOptions>() {
                r.set_status(axum::http::StatusCode::FORBIDDEN);
            }
            return Err(ServerFnError::new("LOCKED"));
        }
    }
    let guest = match guest.as_deref().map(str::trim).filter(|g| !g.is_empty() && g.len() <= 12) {
        // Pratinjau pemilik tidak mencatat "tamu membuka undangan".
        Some(code) if !preview => repo::open_guest(&st.pool, row.id, code).await.map_err(internal)?.map(|(_, g)| g),
        _ => None,
    };
    let skin = st
        .themes()
        .get(&row.inv.theme)
        .map(|t| InvSkin {
            single: t.single_page(),
            open_anim: t.open_anim.clone(),
            float_deco: t.float_deco.clone(),
            ornaments: t.ornaments.clone(),
        })
        .unwrap_or_default();
    Ok(InvitationPage { inv: row.inv, guest, preview, skin })
}

/// Undangan contoh untuk pratinjau tema (tema ditimpa oleh halaman).
#[server]
pub async fn get_demo() -> Result<Invitation, ServerFnError> {
    use srv::*;
    let st = state()?;
    repo::demo_invitation(&st.pool).await.map_err(internal)?.map(|r| r.inv).ok_or_else(not_found)
}

#[server]
pub async fn list_wishes(slug: String) -> Result<WishPage, ServerFnError> {
    use srv::*;
    let st = state()?;
    let row = load(&slug).await?;
    repo::wishes(&st.pool, row.id, 30).await.map_err(internal)
}

#[server]
pub async fn submit_rsvp(
    slug: String,
    guest: Option<String>,
    name: String,
    phone: Option<String>,
    status: String,
    pax: Option<String>,
    session: Option<String>,
    message: Option<String>,
) -> Result<String, ServerFnError> {
    use srv::*;
    let st = state()?;
    let row = load(&slug).await?;
    deny_demo(&row)?;
    deny_locked(&row)?;
    limit_write(&st, "rsvp", &slug).await?;
    let name = clean(&name, 80);
    if name.is_empty() {
        return Err(ServerFnError::new("Nama lengkap wajib diisi."));
    }
    let status = match status.as_str() {
        "hadir" | "ragu" | "tidak" => status,
        _ => return Err(ServerFnError::new("Pilih status kehadiran.")),
    };
    let pax: i32 = pax.and_then(|p| p.trim().parse().ok()).unwrap_or(1).clamp(1, 10);
    let pax = if status == "tidak" { 0 } else { pax };
    let guest_id = match guest.as_deref().filter(|g| !g.is_empty()) {
        Some(code) => repo::guest_id(&st.pool, row.id, code).await.map_err(internal)?,
        None => None,
    };
    repo::upsert_rsvp(
        &st.pool,
        repo::NewRsvp {
            inv_id: row.id,
            guest_id,
            name: &name,
            phone: &clean(phone.as_deref().unwrap_or(""), 20),
            status: &status,
            pax,
            session: &clean(session.as_deref().unwrap_or(""), 60),
            message: &clean(message.as_deref().unwrap_or(""), 600),
        },
    )
    .await
    .map_err(internal)?;
    Ok(match status.as_str() {
        "hadir" => "Terima kasih! Konfirmasi kehadiran & doa restu Anda telah kami terima.".into(),
        "ragu" => "Terima kasih, semoga Anda dapat hadir. Doa restu Anda telah kami terima.".into(),
        _ => "Terima kasih atas doa restunya, semoga kita dipertemukan di lain kesempatan.".into(),
    })
}

#[server]
pub async fn confirm_gift(
    slug: String,
    guest: Option<String>,
    name: String,
    amount: String,
    channel: Option<String>,
    note: Option<String>,
) -> Result<String, ServerFnError> {
    use srv::*;
    let st = state()?;
    let row = load(&slug).await?;
    deny_demo(&row)?;
    deny_locked(&row)?;
    limit_write(&st, "gift", &slug).await?;
    let name = clean(&name, 80);
    let amount: i64 = amount.chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap_or(0);
    if name.is_empty() || amount <= 0 {
        return Err(ServerFnError::new("Isi nama pengirim dan nominal tanda kasih."));
    }
    let guest_id = match guest.as_deref().filter(|g| !g.is_empty()) {
        Some(code) => repo::guest_id(&st.pool, row.id, code).await.map_err(internal)?,
        None => None,
    };
    repo::insert_gift(
        &st.pool,
        row.id,
        guest_id,
        &name,
        amount.min(1_000_000_000),
        &clean(channel.as_deref().unwrap_or(""), 60),
        &clean(note.as_deref().unwrap_or(""), 200),
    )
    .await
    .map_err(internal)?;
    Ok("Terima kasih! Tanda kasih Anda telah kami catat.".into())
}

/// Nomor WA admin (62…) untuk tombol konsultasi; kosong bila tak diset.
#[server]
pub async fn get_contact() -> Result<String, ServerFnError> {
    Ok(srv::state()?.admin_wa.clone())
}

// ── Kelola (pengantin) ─────────────────────────────────────────────────────

#[server]
/// `tema` hanya untuk undangan DEMO: dashboard contoh ikut tema yang sedang
/// dilihat pengunjung (sama seperti `get_invitation`).
pub async fn get_dashboard(slug: String, key: String, tema: Option<String>) -> Result<Dashboard, ServerFnError> {
    use srv::*;
    let st = state()?;
    let key = owner_key(&slug, &key).await?;
    let mut row = load_owned(&slug, &key).await?;
    apply_demo_theme(&st, &mut row, tema.as_deref());
    // Empat query independen → jalan paralel.
    let (stats, guests, activity, minutes_left) = tokio::try_join!(
        repo::stats(&st.pool, row.id),
        repo::guests(&st.pool, row.id),
        repo::activity(&st.pool, row.id),
        repo::unpaid_minutes_left(&st.pool, row.id, st.unpaid_ttl_hours),
    )
    .map_err(internal)?;
    let float_deco = st.themes().get(&row.inv.theme).map(|t| t.float_deco.clone()).unwrap_or_default();
    Ok(Dashboard {
        manage_key: key,
        float_deco,
        minutes_left,
        package_name: st.konten().package_name(&row.inv.package),
        inv: row.inv,
        total_price: row.total_price,
        payment_method: row.payment_method,
        admin_wa: st.admin_wa.clone(),
        stats,
        guests,
        activity,
    })
}

#[server]
pub async fn add_guest(
    slug: String,
    key: String,
    name: String,
    phone: Option<String>,
    category: Option<String>,
    session: Option<String>,
    table_no: Option<String>,
    pax: Option<String>,
) -> Result<String, ServerFnError> {
    use srv::*;
    let st = state()?;
    let row = load_owned(&slug, &key).await?;
    deny_demo(&row)?;
    let name = clean(&name, 80);
    if name.is_empty() {
        return Err(ServerFnError::new("Nama tamu wajib diisi."));
    }
    let category = category.unwrap_or_default();
    let category = if CATEGORIES.iter().any(|(k, _)| *k == category) { category } else { "umum".into() };
    let phone = crate::web::fmt::wa_number(phone.as_deref().unwrap_or(""));
    let code = repo::add_guest(
        &st.pool,
        row.id,
        repo::NewGuest {
            name: &name,
            phone: &phone,
            category: &category,
            session: &clean(session.as_deref().unwrap_or(""), 60),
            table_no: &clean(table_no.as_deref().unwrap_or(""), 20),
            pax: pax.and_then(|p| p.trim().parse().ok()).unwrap_or(1).clamp(1, 10),
        },
    )
    .await
    .map_err(internal)?;
    Ok(format!("{name} ditambahkan (kode {code})."))
}

#[server]
pub async fn delete_guest(slug: String, key: String, code: String) -> Result<(), ServerFnError> {
    use srv::*;
    let st = state()?;
    let row = load_owned(&slug, &key).await?;
    deny_demo(&row)?;
    repo::delete_guest(&st.pool, row.id, &code).await.map_err(internal)
}

#[server]
pub async fn mark_sent(slug: String, key: String, code: String) -> Result<(), ServerFnError> {
    use srv::*;
    let st = state()?;
    let row = load_owned(&slug, &key).await?;
    if row.inv.is_demo {
        return Ok(());
    }
    repo::mark_sent(&st.pool, row.id, &code).await.map_err(internal)
}

#[server]
pub async fn check_in(slug: String, key: String, code: String) -> Result<String, ServerFnError> {
    use srv::*;
    let st = state()?;
    let row = load_owned(&slug, &key).await?;
    // Kode bisa datang dari QR berisi URL penuh (…?g=KODE) atau kode polos.
    let code = code.rsplit("g=").next().unwrap_or(&code).split('&').next().unwrap_or("").trim().to_string();
    if row.inv.is_demo {
        return Ok(format!("Demo: kode {code} terbaca. Di undangan asli, tamu langsung tercatat hadir."));
    }
    match repo::check_in(&st.pool, row.id, &code).await.map_err(internal)? {
        Some((name, false)) => Ok(format!("✓ {name} berhasil check-in. Selamat datang!")),
        Some((name, true)) => Ok(format!("{name} sudah check-in sebelumnya.")),
        None => Err(ServerFnError::new(format!("Kode {code} tidak terdaftar di undangan ini."))),
    }
}

// ── Katalog tema (dari cache server, tabel `themes`) ───────────────────────

/// Tema yang tampil di katalog, urut `sort_order`.
#[server]
pub async fn list_themes() -> Result<Vec<super::skin::ThemeInfo>, ServerFnError> {
    // Versi ringan untuk katalog & /buat: tampilan tema datang dari /tema.css
    // (kelas th-{slug}), jadi token warna & URL gambar tak perlu ikut dikirim
    // (data ini juga diserialisasi ke HTML untuk hydration — ratusan tema).
    Ok(srv::state()?
        .themes()
        .list
        .iter()
        .filter(|t| t.listed)
        .map(|t| super::skin::ThemeInfo {
            tokens: Default::default(),
            image_url: String::new(),
            bg_image: String::new(),
            frame_image: String::new(),
            card_deco: String::new(),
            ornaments: Vec::new(),
            ..t.clone()
        })
        .collect())
}

/// Banner beranda yang sedang tayang (tabel banners; belum dimigrasi = kosong).
#[server]
pub async fn get_banners() -> Result<Vec<Banner>, ServerFnError> {
    let st = srv::state()?;
    Ok(srv::repo::banners_live(&st.pool).await.unwrap_or_else(|e| {
        tracing::debug!(error = %format!("{e:#}"), "banners belum ada — jalankan migration/011_banner.sql");
        Vec::new()
    }))
}

/// Semua banner untuk /admin/banner.
#[server]
pub async fn admin_banners() -> Result<Vec<Banner>, ServerFnError> {
    let (st, _) = require_admin(false).await?;
    srv::repo::banners_all(&st.pool).await.map_err(|e| {
        tracing::error!(error = %format!("{e:#}"), "admin: banners");
        ServerFnError::new("Tabel banner belum ada — jalankan migration/011_banner.sql.")
    })
}

/// Satu tema — termasuk tema privat (dibuka lewat tautan langsung).
#[server]
pub async fn get_theme(slug: String) -> Result<Option<super::skin::ThemeInfo>, ServerFnError> {
    Ok(srv::state()?.themes().get(&slug).cloned())
}

// ── Konten situs (harga, paket, galeri — bisa disunting admin) ─────────────

#[server]
pub async fn get_konten() -> Result<super::konten::Konten, ServerFnError> {
    Ok((*srv::state()?.konten()).clone())
}

// ── Admin (akun + sesi, server/auth.rs) ────────────────────────────────────

/// Batas kiriman publik per IP per undangan (server/security.rs RateLimit).
#[cfg(feature = "ssr")]
async fn limit_write(st: &crate::server::state::AppState, what: &str, slug: &str) -> Result<(), ServerFnError> {
    let headers: axum::http::HeaderMap = leptos_axum::extract().await?;
    let key = format!("{what}:{slug}:{}", crate::server::security::client_ip(&headers));
    st.write_limit
        .hit(&key)
        .map_err(|secs| ServerFnError::new(format!("Terlalu banyak kiriman dari jaringan ini. Coba lagi dalam {} menit.", secs.div_ceil(60))))
}

/// Admin yang masuk; `need_admin` = hanya peran Admin (bukan Editor).
#[cfg(feature = "ssr")]
async fn require_admin(need_admin: bool) -> Result<(std::sync::Arc<crate::server::state::AppState>, AdminUser), ServerFnError> {
    use crate::server::auth::{require, Denied};
    let st = srv::state()?;
    let headers: axum::http::HeaderMap = leptos_axum::extract().await?;
    match require(&st, &headers, need_admin).await {
        Ok(u) => Ok((st, u)),
        Err(Denied::NotAdmin) => Err(ServerFnError::new("Halaman ini khusus peran Admin.")),
        Err(Denied::NoSession) => Err(ServerFnError::new("NO_ADMIN")),
    }
}

#[server]
pub async fn admin_session() -> Result<AdminSessionInfo, ServerFnError> {
    let st = srv::state()?;
    let (count, user) = tokio::join!(srv::repo::admin_count(&st.pool), srv::current_admin(&st));
    Ok(AdminSessionInfo {
        db_ready: count.is_ok(),
        needs_setup: matches!(count, Ok(0)),
        setup_code_set: !st.admin_token.is_empty(),
        user: user?,
    })
}

/// Akun admin yang sedang masuk (untuk bilah "Mode Admin" di halaman publik).
#[server]
pub async fn admin_me() -> Result<Option<AdminUser>, ServerFnError> {
    let st = srv::state()?;
    srv::current_admin(&st).await
}

#[server]
pub async fn admin_themes() -> Result<Vec<AdminTheme>, ServerFnError> {
    let (st, _) = require_admin(false).await?;
    let usage = srv::repo::theme_usage(&st.pool).await.map_err(srv::internal)?;
    Ok(st
        .themes()
        .list
        .iter()
        // Ornamen tidak dikirim (±10 per tema × ratusan tema); halaman yang
        // butuh rinciannya memuat satu tema lewat `get_theme`.
        .map(|t| AdminTheme {
            used: usage.get(&t.slug).copied().unwrap_or(0),
            ornaments: t.ornaments.len(),
            theme: super::skin::ThemeInfo { ornaments: Vec::new(), ..t.clone() },
        })
        .collect())
}

/// Semua animasi (/admin/animasi) + tema pemakainya.
#[server]
pub async fn admin_animations() -> Result<Vec<AdminAnim>, ServerFnError> {
    let (st, _) = require_admin(false).await?;
    let cat = st.themes();
    Ok(cat
        .anims
        .iter()
        .map(|a| {
            let key = a.key();
            let used_by = cat
                .list
                .iter()
                .filter(|t| match a.kind.as_str() {
                    "buka" => t.open_anim == key,
                    "scroll" => t.scroll_anim == key,
                    _ => t.float_deco == key,
                })
                .map(|t| t.name.clone())
                .collect();
            AdminAnim { anim: a.clone(), used_by }
        })
        .collect())
}

#[server]
pub async fn admin_invitations(q: String) -> Result<Vec<AdminInv>, ServerFnError> {
    let (st, _) = require_admin(true).await?;
    let k = st.konten();
    let mut list = srv::repo::admin_invitations(&st.pool, q.trim(), st.unpaid_ttl_hours).await.map_err(srv::internal)?;
    for i in &mut list {
        i.package_name = k.package_name(&i.package);
    }
    Ok(list)
}

#[server]
pub async fn admin_accounts() -> Result<Vec<AdminUser>, ServerFnError> {
    let (st, _) = require_admin(true).await?;
    srv::repo::admins(&st.pool).await.map_err(srv::internal)
}

/// kunci bagian → "siapa • kapan" terakhir disunting.
#[server]
pub async fn admin_konten_meta() -> Result<Vec<(String, String)>, ServerFnError> {
    let (st, _) = require_admin(false).await?;
    Ok(srv::repo::content_meta(&st.pool).await.unwrap_or_default().into_iter().collect())
}
