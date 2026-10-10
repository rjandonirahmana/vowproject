//! web/api.rs — server function (dipanggil SSR langsung, dari WASM via /api/*).

use leptos::prelude::*;

#[cfg(feature = "ssr")]
use super::i18n::Lang;
use super::model::*;
#[cfg(feature = "ssr")]
use crate::tx;

#[cfg(feature = "ssr")]
mod srv {
    use std::sync::Arc;

    use crate::tx;
    use crate::web::i18n::Lang;

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
    /// Tanpa kunci yang cocok, peran ADMIN yang sedang masuk tetap diizinkan
    /// (/admin/undangan → "Kelola": membantu/memeriksa undangan pembeli).
    pub async fn load_owned(slug: &str, key: &str) -> Result<InvRow, ServerFnError> {
        let key = &owner_key(slug, key).await?;
        let row = load(slug).await?;
        if !is_owner(key, &row) && !admin_view(&row).await? {
            return Err(ServerFnError::new("Kunci kelola tidak valid. Buka dari tautan yang Anda terima saat memesan."));
        }
        Ok(row)
    }

    pub fn is_owner(key: &str, row: &InvRow) -> bool {
        !key.is_empty() && crate::server::auth::same_hash(&crate::server::auth::token_hash(key.trim()), &row.manage_key_hash)
    }

    /// Admin (peran Admin, bukan Editor) sedang masuk → boleh membuka Kelola /
    /// pratinjau undangan pembeli mana pun. Dicatat di log.
    pub async fn admin_view(row: &InvRow) -> Result<bool, ServerFnError> {
        let st = state()?;
        let ok = current_admin(&st).await?.is_some_and(|u| u.is_admin());
        if ok {
            tracing::info!(slug = %row.inv.slug, "admin membuka Kelola/pratinjau pembeli");
        }
        Ok(ok)
    }

    /// Admin yang sedang masuk (header request ini), tanpa cek peran.
    pub async fn current_admin(st: &AppState) -> Result<Option<crate::web::model::AdminUser>, ServerFnError> {
        let headers: axum::http::HeaderMap = leptos_axum::extract().await?;
        Ok(crate::server::auth::current(st, &headers).await)
    }

    /// Tamu tak bisa RSVP / kirim tanda kasih ke undangan yang belum aktif.
    pub fn deny_locked(row: &InvRow) -> Result<(), ServerFnError> {
        deny_locked_in(row, Lang::Id)
    }

    pub fn deny_locked_in(row: &InvRow, l: Lang) -> Result<(), ServerFnError> {
        if row.inv.is_locked() {
            Err(ServerFnError::new(tx!(l,
                "Undangan ini belum diaktifkan — RSVP & ucapan dibuka setelah pembayaran dikonfirmasi.",
                "This invitation is not active yet — RSVP and wishes open once payment is confirmed.")))
        } else {
            Ok(())
        }
    }

    pub fn deny_demo(row: &InvRow) -> Result<(), ServerFnError> {
        deny_demo_in(row, Lang::Id)
    }

    pub fn deny_demo_in(row: &InvRow, l: Lang) -> Result<(), ServerFnError> {
        if row.inv.is_demo {
            Err(ServerFnError::new(tx!(l,
                "Ini undangan demo — perubahan tidak disimpan. Pesan tema untuk mencoba penuh.",
                "This is a demo invitation — nothing is saved. Order a theme to try it fully.")))
        } else {
            Ok(())
        }
    }
}

/// Undangan untuk tamu. Belum dibayar → terkunci ("LOCKED", tanpa data apa
/// pun) kecuali dibuka pemilik dengan kunci Kelola (`k`) sebagai pratinjau.
/// `tema` hanya berlaku untuk undangan DEMO ("Coba Demo" dari halaman tema):
/// isi demo ditampilkan dengan tema itu. Undangan asli selalu memakai temanya.
/// `rupa` ("sampul:kubah,judul:pita") juga hanya untuk demo: uji varian bentuk.
#[server]
pub async fn get_invitation(slug: String, guest: Option<String>, k: Option<String>, tema: Option<String>, rupa: Option<String>) -> Result<InvitationPage, ServerFnError> {
    use srv::*;
    let st = state()?;
    let mut row = load(&slug).await?;
    st.demo_tema(&mut row, tema.as_deref());
    st.demo_lagu(&mut row).await;
    let preview = row.inv.is_locked();
    if preview {
        let k = owner_key(&slug, k.as_deref().unwrap_or("")).await?;
        if !is_owner(&k, &row) && !admin_view(&row).await? {
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
    let cat = st.themes();
    // Timpaan rupa hanya untuk demo; dinormalisasi agar URL CSS-nya kanonis.
    let over = rupa
        .filter(|r| row.inv.is_demo && r.len() <= 300)
        .map(|r| crate::web::rupa::parse_override(&r))
        .filter(|m| !m.is_empty());
    let skin = cat
        .get(&row.inv.theme)
        .map(|t| {
            let over_q = over.as_ref().map(|m| m.iter().map(|(k, v)| format!("{k}:{v}")).collect::<Vec<_>>().join(",")).unwrap_or_default();
            let (css, fonts) = cat.links(t, &over_q);
            InvSkin {
                single: t.single_page(),
                open_anim: t.open_anim.clone(),
                float_deco: t.float_deco.clone(),
                scroll_anim: t.scroll_anim.clone(),
                ornaments: t.ornaments.clone(),
                bg_video: t.bg_video.clone(),
                open_video: t.open_video.clone(),
                rupa: crate::web::rupa::classes(over.as_ref().unwrap_or(&t.rupa)),
                css,
                fonts,
            }
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
    lang: Option<String>,
) -> Result<String, ServerFnError> {
    use srv::*;
    let st = state()?;
    let row = load(&slug).await?;
    let headers: axum::http::HeaderMap = leptos_axum::extract().await?;
    let ip = crate::server::security::client_ip(&headers);
    let lang = Lang::of(lang.as_deref().unwrap_or(&row.inv.lang));
    rsvp_core(&st, &row, &ip, RsvpInput { guest, name, phone, status, pax, session, message, lang })
        .await
        .map(|(msg, _)| msg)
        .map_err(ServerFnError::new)
}

/// Isian RSVP + ucapan (server fn Leptos & formulir tema templat).
#[cfg(feature = "ssr")]
#[derive(Default)]
pub struct RsvpInput {
    pub guest: Option<String>,
    pub name: String,
    pub phone: Option<String>,
    pub status: String,
    pub pax: Option<String>,
    pub session: Option<String>,
    pub message: Option<String>,
    /// Bahasa pesan balasan untuk tamu.
    pub lang: Lang,
}

/// Validasi + batas kiriman per IP + simpan RSVP. Ok = (pesan, baris baru?);
/// `false` = RSVP tamu terdaftar yang sudah ada diperbarui. Err = pesan untuk tamu.
#[cfg(feature = "ssr")]
pub async fn rsvp_core(
    st: &crate::server::state::AppState,
    row: &crate::server::repo::InvRow,
    ip: &str,
    i: RsvpInput,
) -> Result<(String, bool), String> {
    use crate::server::handlers::clean;
    use crate::server::repo;
    let msg = |e: ServerFnError| match e {
        ServerFnError::ServerError(m) => m,
        e => e.to_string(),
    };
    let l = i.lang;
    srv::deny_demo_in(row, l).map_err(msg)?;
    srv::deny_locked_in(row, l).map_err(msg)?;
    st.write_limit.hit(&format!("rsvp:{}:{ip}", row.inv.slug)).map_err(|secs| match l {
        Lang::Id => format!("Terlalu banyak kiriman dari jaringan ini. Coba lagi dalam {} menit.", secs.div_ceil(60)),
        Lang::En => format!("Too many submissions from this network. Please try again in {} minutes.", secs.div_ceil(60)),
    })?;
    // Per undangan (semua IP): buku ucapan tak bisa dibanjiri bot terdistribusi.
    st.cap_limit
        .hit_max(&format!("rsvp:{}", row.inv.slug), 600)
        .map_err(|_| tx!(l, "Buku ucapan sedang sangat ramai. Coba lagi beberapa saat lagi.", "The guestbook is very busy right now. Please try again shortly.").to_string())?;
    let busy = |e: anyhow::Error| {
        tracing::error!(error = %format!("{e:#}"), "rsvp");
        tx!(l, "Server sedang sibuk, coba lagi sebentar.", "The server is busy, please try again in a moment.").to_string()
    };
    let name = clean(&i.name, 80);
    if name.is_empty() {
        return Err(tx!(l, "Nama lengkap wajib diisi.", "Please enter your full name.").into());
    }
    let status = match i.status.as_str() {
        "hadir" | "ragu" | "tidak" => i.status,
        _ => return Err(tx!(l, "Pilih status kehadiran.", "Please choose whether you will attend.").into()),
    };
    // Isian rusak ditolak, bukan diam-diam dijadikan 1 tamu.
    let pax = match (status.as_str(), i.pax.as_deref().map(str::trim).filter(|p| !p.is_empty())) {
        ("tidak", _) => 0,
        (_, None) => 1,
        (_, Some(p)) => match p.parse::<i32>() {
            Ok(n) if (1..=10).contains(&n) => n,
            _ => return Err(tx!(l, "Jumlah tamu harus angka 1–10.", "Number of guests must be 1–10.").into()),
        },
    };
    // Kode tamu diisi tapi tak dikenal ≠ tamu umum: jangan diam-diam jadi anonim.
    let guest_id = match i.guest.as_deref().map(str::trim).filter(|g| !g.is_empty()) {
        Some(code) => match repo::guest_id(&st.pool, row.id, code).await.map_err(busy)? {
            Some(id) => Some(id),
            None => return Err(tx!(l, "Kode tamu tidak dikenal — buka undangan dari tautan yang Anda terima.", "Unknown guest code — please open the invitation from the link you received.").into()),
        },
        None => None,
    };
    let baru = repo::upsert_rsvp(
        &st.pool,
        repo::NewRsvp {
            inv_id: row.id,
            guest_id,
            name: &name,
            phone: &clean(i.phone.as_deref().unwrap_or(""), 20),
            status: &status,
            pax,
            session: &clean(i.session.as_deref().unwrap_or(""), 60),
            message: &clean(i.message.as_deref().unwrap_or(""), 600),
        },
    )
    .await
    .map_err(busy)?;
    let msg = match status.as_str() {
        "hadir" => tx!(l, "Terima kasih! Konfirmasi kehadiran & doa restu Anda telah kami terima.", "Thank you! We have received your RSVP and blessings."),
        "ragu" => tx!(l, "Terima kasih, semoga Anda dapat hadir. Doa restu Anda telah kami terima.", "Thank you — we hope you can make it. Your blessings have been received."),
        _ => tx!(l, "Terima kasih atas doa restunya, semoga kita dipertemukan di lain kesempatan.", "Thank you for your blessings — we hope to see you another time."),
    };
    Ok((msg.into(), baru))
}

#[server]
pub async fn confirm_gift(
    slug: String,
    guest: Option<String>,
    name: String,
    amount: String,
    channel: Option<String>,
    note: Option<String>,
    lang: Option<String>,
) -> Result<String, ServerFnError> {
    use srv::*;
    let st = state()?;
    let row = load(&slug).await?;
    let l = Lang::of(lang.as_deref().unwrap_or(&row.inv.lang));
    deny_demo_in(&row, l)?;
    deny_locked_in(&row, l)?;
    limit_write(&st, "gift", &slug).await?;
    if st.cap_limit.hit_max(&format!("gift:{slug}"), 300).is_err() {
        return Err(ServerFnError::new(tx!(l, "Sedang sangat ramai. Coba lagi beberapa saat lagi.", "It's very busy right now. Please try again shortly.")));
    }
    let name = clean(&name, 80);
    let amount: i64 = amount.chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap_or(0);
    if name.is_empty() || amount <= 0 {
        return Err(ServerFnError::new(tx!(l, "Isi nama pengirim dan nominal tanda kasih.", "Please enter the sender's name and the amount.")));
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
    Ok(tx!(l, "Terima kasih! Tanda kasih Anda telah kami catat.", "Thank you! Your gift has been noted.").into())
}

/// Nomor WA admin (62…) untuk tombol konsultasi; kosong bila tak diset.
// ── Story tamu ─────────────────────────────────────────────────────────────

/// Story undangan untuk tab "Story". Tabel belum dimigrasi → daftar kosong.
#[server]
pub async fn list_stories(slug: String) -> Result<Vec<StoryItem>, ServerFnError> {
    use srv::*;
    let st = state()?;
    let row = load(&slug).await?;
    if row.inv.is_locked() {
        return Ok(Vec::new());
    }
    // TANPA membaca cookie: daftar ini ikut SSR (Resource diserialisasi untuk
    // hydrate) — `extract()` di sini membuat render SSR pertama galat & hydrate
    // kadang tak cocok. Tanda "story saya" lewat `my_stories` (klien saja).
    match repo::stories(&st.pool, row.id, "").await {
        Ok(v) => Ok(v),
        Err(e) => {
            tracing::warn!(error = %format!("{e:#}"), "story: daftar");
            Ok(Vec::new())
        }
    }
}

/// Id story milik perangkat ini (cookie HttpOnly `ily_s_{slug}` pembuat).
/// Dipanggil dari klien (LocalResource) — bukan bagian render SSR.
#[server]
pub async fn my_stories(slug: String) -> Result<Vec<i64>, ServerFnError> {
    use srv::*;
    let st = state()?;
    let headers: axum::http::HeaderMap = leptos_axum::extract().await?;
    let Some(token) = crate::server::handlers::story_token(&headers, &slug) else { return Ok(Vec::new()) };
    let row = load(&slug).await?;
    Ok(repo::stories(&st.pool, row.id, &crate::server::auth::token_hash(&token))
        .await
        .map(|v| v.into_iter().filter(|s| s.mine).map(|s| s.id).collect())
        .unwrap_or_default())
}

/// Langkah 1 menambah story: nomor WhatsApp tamu → kunci spesial 6 digit
/// dikirim lewat WhatsApp/waxum (hanya hash yang disimpan, berlaku 15 menit). Satu
/// nomor = satu story per undangan; maks 3 kunci per nomor per jam.
#[server]
pub async fn request_story_key(slug: String, name: String, phone: String) -> Result<StoryKey, ServerFnError> {
    use srv::*;
    let st = state()?;
    let row = load(&slug).await?;
    if row.inv.is_demo {
        return Err(ServerFnError::new("Ini undangan demo — story contoh saja. Pesan undangan untuk mengaktifkan story tamu."));
    }
    deny_locked(&row)?;
    let name = clean(&name, 60);
    if name.is_empty() {
        return Err(ServerFnError::new("Nama wajib diisi."));
    }
    let phone = crate::web::fmt::wa_number(&phone);
    if phone.is_empty() {
        return Err(ServerFnError::new("Nomor WhatsApp tidak valid (contoh: 0812 3456 7890)."));
    }
    let Some(wa) = st.wa.clone() else {
        return Err(ServerFnError::new("Layanan WhatsApp belum aktif — story belum bisa ditambahkan."));
    };
    limit_write(&st, "storykey", &slug).await?;
    // Tiap kunci = satu pesan WA ke nomor yang diketik pengunjung: dibatasi per
    // undangan & global agar tak bisa dipakai menyepam nomor orang lewat bot.
    if st.cap_limit.hit_max(&format!("storykey:{slug}"), 60).is_err() || st.cap_limit.hit_max("wa-story", 300).is_err() {
        return Err(ServerFnError::new("Pengiriman kunci sedang dibatasi karena terlalu ramai. Coba lagi nanti."));
    }
    // Nomor yang sudah punya story tetap dikirimi kunci — untuk MENGHAPUS
    // story-nya (dari perangkat mana pun); 1 nomor tetap = 1 story.
    let has_story = repo::story_phone_taken(&st.pool, row.id, &phone).await.map_err(internal)?;
    if repo::story_keys_recent(&st.pool, row.id, &phone).await.map_err(internal)? >= 3 {
        return Err(ServerFnError::new("Kunci sudah dikirim 3 kali dalam 1 jam terakhir. Cek WhatsApp Anda atau coba lagi nanti."));
    }
    let key = {
        use rand::Rng;
        format!("{:06}", rand::rng().random_range(0..1_000_000u32))
    };
    repo::insert_story_key(&st.pool, row.id, &phone, &name, &crate::server::auth::token_hash(&key)).await.map_err(internal)?;
    let tujuan = if has_story { "menghapus story Anda" } else { "menambahkan story" };
    let text = format!(
        "Halo {name}! 👋\n\nKunci spesial untuk {tujuan} di undangan pernikahan *{}*:\n\n*{key}*\n\nBerlaku 15 menit, hanya untuk nomor ini. Jangan bagikan ke orang lain.",
        row.inv.couple()
    );
    // Tamu sedang menunggu di layar → percobaan ulang singkat saja.
    if let Err(e) = wa.send_text(&phone, &text, crate::server::wa::Gigih::Cepat).await {
        tracing::error!(slug = %slug, error = %format!("{e:#}"), "story: kirim kunci WA gagal");
        return Err(ServerFnError::new("Gagal mengirim WhatsApp. Pastikan nomor aktif di WhatsApp lalu coba lagi."));
    }
    Ok(StoryKey { phone, has_story })
}

/// Moderasi story di Kelola (hanya pemegang kunci Kelola = pembeli undangan).
#[server]
pub async fn owner_stories(slug: String, key: String, page: i64) -> Result<StoryModPage, ServerFnError> {
    use srv::*;
    let st = state()?;
    let row = load_owned(&slug, &key).await?;
    repo::stories_page(&st.pool, Some(row.id), "", page).await.map_err(|e| {
        tracing::error!(error = %format!("{e:#}"), "kelola: story");
        ServerFnError::new("Daftar story gagal dimuat — coba muat ulang halaman.")
    })
}

/// Moderasi story SEMUA undangan (admin).
#[server]
pub async fn admin_stories(q: String, page: i64) -> Result<StoryModPage, ServerFnError> {
    let (st, _) = require_admin(false).await?;
    srv::repo::stories_page(&st.pool, None, &q, page).await.map_err(|e| {
        tracing::error!(error = %format!("{e:#}"), "admin: story");
        ServerFnError::new("Daftar story gagal dimuat — lihat log server.")
    })
}

/// Admin menghapus story permanen (baris + foto RustFS).
#[server]
pub async fn admin_delete_story(id: i64) -> Result<(), ServerFnError> {
    let (st, _) = require_admin(false).await?;
    if let Some(url) = srv::repo::delete_story_admin(&st.pool, id).await.map_err(srv::internal)? {
        if let Some(s) = st.storage.as_ref() {
            let _ = s.delete_url(&url).await;
        }
        tracing::info!(id, "admin: story dihapus");
    }
    Ok(())
}

/// Pemilik undangan menghapus story tamu (dari Kelola).
#[server]
pub async fn delete_story(slug: String, key: String, id: i64) -> Result<(), ServerFnError> {
    use srv::*;
    let st = state()?;
    let row = load_owned(&slug, &key).await?;
    deny_demo(&row)?;
    if let Some(url) = repo::delete_story(&st.pool, row.id, id).await.map_err(internal)? {
        if let Some(s) = st.storage.as_ref() {
            let _ = s.delete_url(&url).await;
        }
    }
    Ok(())
}

#[server]
pub async fn get_contact() -> Result<String, ServerFnError> {
    Ok(srv::state()?.admin_wa.clone())
}

// ── Kelola (pengantin) ─────────────────────────────────────────────────────

/// Isi undangan saat ini untuk /kelola/{slug}/sunting (pemilik atau Admin).
#[server]
pub async fn get_sunting(slug: String, key: String) -> Result<crate::web::model::Sunting, ServerFnError> {
    use srv::*;
    let st = state()?;
    let key = owner_key(&slug, &key).await?;
    let row = load_owned(&slug, &key).await?;
    if row.inv.is_demo {
        return Err(ServerFnError::new("Ini undangan demo — pesan undangan Anda sendiri untuk mencoba menyunting."));
    }
    let songs = repo::songs(&st.pool, true).await.unwrap_or_default();
    let base = row.inv.music_url.split('#').next().unwrap_or("").to_string();
    let song_id = songs.iter().find(|s| !base.is_empty() && s.url == base).map(|s| s.id).unwrap_or(0);
    let cat = st.themes();
    let mut themes: Vec<(String, String, String)> = cat.list.iter().filter(|t| t.listed).map(|t| (t.slug.clone(), t.name.clone(), t.region.clone())).collect();
    if !themes.iter().any(|t| t.0 == row.inv.theme) {
        if let Some(t) = cat.get(&row.inv.theme) {
            themes.insert(0, (t.slug.clone(), t.name.clone(), "Tema custom milik Anda".into()));
        }
    }
    let quote_idx = crate::web::themes::QUOTES.iter().position(|(t, _)| *t == row.inv.quote_text).unwrap_or(0);
    Ok(crate::web::model::Sunting {
        manage_key: if is_owner(&key, &row) { key } else { String::new() },
        contact_phone: row.contact_phone.clone(),
        inv: row.inv,
        songs,
        song_id,
        themes,
        quote_idx,
    })
}

#[server]
/// `tema` hanya untuk undangan DEMO: dashboard contoh ikut tema yang sedang
/// dilihat pengunjung (sama seperti `get_invitation`).
pub async fn get_dashboard(slug: String, key: String, tema: Option<String>) -> Result<Dashboard, ServerFnError> {
    use srv::*;
    let st = state()?;
    let key = owner_key(&slug, &key).await?;
    let mut row = load_owned(&slug, &key).await?;
    // Dibuka admin (tanpa kunci pemilik): kunci rahasia pembeli tak pernah
    // dikirim — manage_key kosong = "mode admin" di halaman Kelola.
    let key = if is_owner(&key, &row) { key } else { String::new() };
    st.demo_tema(&mut row, tema.as_deref());
    // Empat query independen → jalan paralel.
    let (stats, guests, activity, minutes_left) = tokio::try_join!(
        repo::stats(&st.pool, row.id),
        repo::guests(&st.pool, row.id),
        repo::activity(&st.pool, row.id),
        repo::unpaid_minutes_left(&st.pool, row.id, st.unpaid_ttl_hours),
    )
    .map_err(internal)?;
    // Kolom bukti belum dimigrasi (017) → dianggap belum ada, bukan 500.
    let payment_proof = if row.inv.is_locked() { repo::payment_proof(&st.pool, row.id).await.unwrap_or_default() } else { None };
    let float_deco = st.themes().get(&row.inv.theme).map(|t| t.float_deco.clone()).unwrap_or_default();
    Ok(Dashboard {
        payment: st.konten().pembayaran.clone(),
        payment_proof,
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
    // Dibaca tiap katalog dibuka → cache 30 dtk (jadwal tayang banner per menit).
    Ok(crate::server::state::cached(&st.banners, std::time::Duration::from_secs(30), async {
        srv::repo::banners_live(&st.pool).await.unwrap_or_else(|e| {
            tracing::debug!(error = %format!("{e:#}"), "banners belum ada — jalankan migration/011_banner.sql");
            Vec::new()
        })
    })
    .await)
}

/// Story panduan beranda (aktif, urut). Tabel belum ada (037) = kosong.
#[server]
pub async fn get_site_stories() -> Result<Vec<SiteStory>, ServerFnError> {
    let st = srv::state()?;
    Ok(crate::server::state::cached(&st.panduan, std::time::Duration::from_secs(30), async {
        srv::repo::site_stories(&st.pool, true).await.unwrap_or_else(|e| {
            tracing::debug!(error = %format!("{e:#}"), "site_stories belum ada — jalankan migration/037_story_panduan.sql");
            Vec::new()
        })
    })
    .await)
}

/// Semua story panduan untuk /admin/story-panduan.
#[server]
pub async fn admin_site_stories() -> Result<Vec<SiteStory>, ServerFnError> {
    let (st, _) = require_admin(false).await?;
    srv::repo::site_stories(&st.pool, false).await.map_err(|e| {
        tracing::error!(error = %format!("{e:#}"), "admin: site_stories");
        ServerFnError::new("Tabel story panduan belum ada — jalankan migration/037_story_panduan.sql.")
    })
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

/// Pustaka musik untuk pengantin (/buat) & panel musik demo tema: hanya lagu
/// aktif. Tabel belum dimigrasi → daftar kosong.
#[server]
pub async fn list_songs() -> Result<Vec<Song>, ServerFnError> {
    let st = srv::state()?;
    Ok(srv::repo::songs(&st.pool, true).await.unwrap_or_else(|e| {
        tracing::warn!(error = %format!("{e:#}"), "pustaka lagu");
        Vec::new()
    }))
}

#[server]
pub async fn admin_songs() -> Result<Vec<Song>, ServerFnError> {
    let (st, _) = require_admin(false).await?;
    srv::repo::songs(&st.pool, false).await.map_err(|e| {
        tracing::error!(error = %format!("{e:#}"), "admin: lagu");
        ServerFnError::new("Tabel lagu belum ada — jalankan migration/027_pustaka_lagu.sql.")
    })
}

/// Templat tema + tema dan templat yang dipakainya (/admin/templat).
#[server]
pub async fn admin_templates() -> Result<AdminTemplatPage, ServerFnError> {
    let (st, _) = require_admin(true).await?;
    let cat = st.themes();
    let pretty = |m: &std::collections::BTreeMap<String, String>| serde_json::to_string_pretty(m).unwrap_or_default();
    let set = st.templat();
    let templates = set
        .list()
        .into_iter()
        .map(|t| AdminTemplat {
            slug: t.slug.clone(),
            name: t.name.clone(),
            html: t.html.clone(),
            css: t.css.clone(),
            fonts: t.fonts.clone(),
            assets: pretty(&t.assets),
            builtin: t.builtin,
            edited: t.edited,
            used_by: cat.list.iter().filter(|x| x.template == t.slug).map(|x| (x.slug.clone(), x.name.clone())).collect(),
        })
        .collect();
    let themes = cat
        .list
        .iter()
        .map(|t| TemaTemplat {
            slug: t.slug.clone(),
            name: t.name.clone(),
            template: t.template.clone(),
            assets: if t.template_assets.is_empty() { String::new() } else { pretty(&t.template_assets) },
            css: t.template_css.clone(),
        })
        .collect();
    Ok(AdminTemplatPage { templates, themes })
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
