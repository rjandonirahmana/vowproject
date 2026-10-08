//! server/handlers.rs — endpoint axum di luar server fn:
//!
//!   POST /buat/kirim                 formulir pemesanan (multipart: foto + lagu)
//!   GET  /kelola/{slug}/tamu.csv     ekspor daftar tamu + RSVP (cookie kunci Kelola / ?key=)
//!   GET  /layanan/wa                 formulir cetak/dekorasi/MUA → wa.me admin
//!   GET  /tema.css                   CSS semua tema (dari tabel themes, di-cache)
//!   POST /admin/masuk | /admin/setup | /admin/keluar   sesi akun admin (server/auth.rs)
//!   POST /admin/tema/simpan | /admin/tema/hapus        kelola tema (editor+)
//!   POST /admin/animasi/simpan | /bawaan | /hapus     semua animasi undangan (editor+)
//!   POST /admin/banner/simpan | /admin/banner/{urut,hapus}  banner beranda (editor+)
//!   POST /admin/konten/simpan                          konten & harga (editor+, multipart)
//!   POST /admin/undangan/simpan                        aktivasi pesanan (admin)
//!   POST /admin/akun/simpan | /admin/sandi             akun (admin) / sandi sendiri

use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Multipart, Path, Query},
    http::{header, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Extension,
};
use serde_json::json;

use super::{auth, security};
use super::repo::{self, MempelaiInput, NewInvitation};
use super::state::AppState;
use crate::web::model::AdminUser;
use crate::web::{fmt, themes};

pub use crate::web::fmt::clean;

/// Panjang kunci rahasia di ujung tautan: 8 karakter dari 32 simbol = 40 bit
/// (±1,1 triliun kemungkinan) — nama boleh diketahui, kuncinya tidak tertebak.
pub const INVITATION_KEY_LEN: usize = 8;

/// Foto galeri maksimal per undangan (tiap foto ≤ MAX_IMAGE).
pub const MAX_GALLERY: usize = 6;

/// Tautan /u/{nama-wanita}-{nama-pria}-{kunci}, mis. "yona-doni-k7f3x9m2".
/// Nama = kata pertama tiap mempelai (terbaca & mudah dikenali tamu); kunci
/// acak menjamin unik & tak bisa ditebak walau nama pasangan sama.
pub fn new_invitation_id(bride: &str, groom: &str) -> String {
    let first = |s: &str| fmt::key(s.split_whitespace().next().unwrap_or(""));
    let names: String = fmt::key(&format!("{} {}", first(bride), first(groom))).chars().take(32).collect();
    let names = names.trim_end_matches('-');
    let key = random_key(INVITATION_KEY_LEN);
    if names.is_empty() { key } else { format!("{names}-{key}") }
}

pub fn random_key(len: usize) -> String {
    use rand::Rng;
    const A: &[u8] = b"abcdefghijkmnpqrstuvwxyz23456789";
    let mut rng = rand::rng();
    (0..len).map(|_| A[rng.random_range(0..A.len())] as char).collect()
}

/// Hapus unggahan pesanan yang batal disimpan (best-effort, galat dicatat).
async fn discard_uploads(state: &AppState, urls: &[String]) {
    let Some(st) = state.storage.as_ref() else { return };
    for url in urls {
        if let Err(e) = st.delete_url(url).await {
            tracing::warn!(error = %format!("{e:#}"), url = %url, "buat: gagal menghapus unggahan yatim");
        }
    }
}

fn back_with_error(msg: &str) -> Response {
    Redirect::to(&format!("/buat?galat={}", fmt::url_encode(msg))).into_response()
}

/// Isian TEKS undangan dari formulir — sama untuk /buat/kirim (buat baru) dan
/// /kelola/{slug}/sunting (sunting): mempelai (tanpa foto), acara, busana,
/// kutipan, rekening, kisah, siaran langsung. Err = pesan untuk pengguna.
pub struct IsiTeks {
    pub bride: MempelaiInput,
    pub groom: MempelaiInput,
    pub events: serde_json::Value,
    pub dress_code: String,
    pub quote_text: String,
    pub quote_source: String,
    pub banks: serde_json::Value,
    pub family_name: String,
    pub love_story: Vec<serde_json::Value>,
    pub live_url: String,
    pub dress_colors: serde_json::Value,
    pub music_autoplay: bool,
}

pub fn baca_isi(form: &super::form::Form) -> Result<IsiTeks, &'static str> {
    let get = |k: &str, max: usize| form.get(k, max);
    let bride_name = get("bride_name", 80);
    let groom_name = get("groom_name", 80);
    if bride_name.is_empty() || groom_name.is_empty() {
        return Err("Nama lengkap kedua mempelai wajib diisi.");
    }
    let akad_date = get("akad_date", 10);
    if fmt::parse_date(&akad_date).is_none() {
        return Err("Tanggal akad / pemberkatan wajib diisi.");
    }
    // ── Kisah cinta, siaran langsung, warna busana ──
    let love_story: Vec<serde_json::Value> = (1..=4)
        .filter_map(|i| {
            let title = get(&format!("story{i}_title"), 80);
            let text = get(&format!("story{i}_text"), 500);
            (!title.is_empty() || !text.is_empty())
                .then(|| json!({ "year": get(&format!("story{i}_year"), 20), "title": title, "text": text }))
        })
        .collect();
    let live_url = get("live_url", 300);
    let live_url = if live_url.starts_with("https://") && crate::web::skin::is_safe_url(&live_url) { live_url } else { String::new() };
    let dress_colors: Vec<serde_json::Value> = (1..=4)
        .filter_map(|i| {
            let name = get(&format!("dress{i}_name"), 30);
            let hex = get(&format!("dress{i}_hex"), 9).to_lowercase();
            (!name.is_empty() && crate::web::skin::is_color(&hex)).then(|| json!({ "name": name, "hex": hex }))
        })
        .collect();

    // ── Acara ──
    let tz = get("tz", 4);
    let tz = if fmt::TIMEZONES.iter().any(|t| t.0 == tz) { tz } else { "WIB".to_string() };
    let akad_title = get("akad_title", 40);
    let mut events = vec![json!({
        "kind": "akad",
        "title": if akad_title.is_empty() { "Akad Nikah".to_string() } else { akad_title },
        "badge": "Pemberkatan & Akad",
        "tag": "Sesi Khidmat",
        "date": akad_date,
        "time_start": get("akad_start", 5),
        "time_end": get("akad_end", 5),
        "sessions": [],
        "venue": get("akad_venue", 120),
        "address": get("akad_address", 200),
        "maps_url": get("akad_maps", 300),
        "tz": tz,
    })];
    let resepsi_date = get("resepsi_date", 10);
    if fmt::parse_date(&resepsi_date).is_some() {
        let mut sessions = Vec::new();
        for (i, label) in [(1, "Sesi Siang"), (2, "Sesi Malam")] {
            let t = get(&format!("resepsi_s{i}"), 40);
            if !t.is_empty() {
                sessions.push(json!({ "label": label, "time": t }));
            }
        }
        events.push(json!({
            "kind": "resepsi",
            "title": "Resepsi Pernikahan",
            "badge": "Resepsi Agung",
            "tag": "Selebrasi",
            "date": resepsi_date,
            "time_start": get("resepsi_start", 5),
            "time_end": get("resepsi_end", 5),
            "sessions": sessions,
            "venue": get("resepsi_venue", 120),
            "address": get("resepsi_address", 200),
            "maps_url": get("resepsi_maps", 300),
            "tz": tz,
        }));
    }

    // ── Rekening ──
    let mut banks = Vec::new();
    for i in 1..=2 {
        let number = get(&format!("bank{i}_number"), 40);
        if !number.is_empty() {
            banks.push(json!({
                "bank": get(&format!("bank{i}_name"), 60),
                "number": number,
                "holder": get(&format!("bank{i}_holder"), 80),
            }));
        }
    }

    let quote_idx: usize = get("quote", 2).parse().unwrap_or(0);
    let (quote_text, quote_source) = themes::QUOTES.get(quote_idx).copied().unwrap_or(themes::QUOTES[0]);
    let ig = |k: &str| get(k, 40).trim_start_matches('@').to_string();
    let orang = |p: &str, name: String| MempelaiInput {
        name,
        degree: get(&format!("{p}_degree"), 30),
        nick: get(&format!("{p}_nick"), 30),
        parents: get(&format!("{p}_parents"), 200),
        ig: ig(&format!("{p}_ig")),
        photo: String::new(),
    };
    Ok(IsiTeks {
        bride: orang("bride", bride_name),
        groom: orang("groom", groom_name),
        events: json!(events),
        dress_code: get("dress_code", 200),
        quote_text: quote_text.to_string(),
        quote_source: quote_source.to_string(),
        banks: json!(banks),
        family_name: get("family_name", 120),
        love_story,
        live_url,
        dress_colors: json!(dress_colors),
        music_autoplay: form.has("music_autoplay"),
    })
}

pub async fn create_invitation(Extension(state): Extension<Arc<AppState>>, headers: axum::http::HeaderMap, mp: Multipart) -> Response {
    // Dicek SEBELUM membaca multipart: kiriman spam tak sempat diunggah ke RustFS.
    if let Err(secs) = state.create_limit.hit(&format!("buat:{}", security::client_ip(&headers))) {
        return back_with_error(&format!("Terlalu banyak pembuatan undangan dari jaringan ini. Coba lagi dalam {} menit.", secs.div_ceil(60)));
    }
    // Kuota global: bot dari ribuan IP tak bisa membanjiri DB & RustFS.
    if state.cap_limit.hit_max("buat", 300).is_err() {
        tracing::warn!("buat: kuota global per jam habis — kemungkinan serangan bot");
        return back_with_error("Layanan pemesanan sedang sangat ramai. Coba lagi beberapa saat lagi atau hubungi admin.");
    }
    let mut form = match super::form::read(mp, 5 + MAX_GALLERY).await {
        Ok(f) => f,
        Err(()) => return back_with_error("Unggahan terputus atau terlalu besar (foto maks 5 MB, lagu maks 6 MB)."),
    };
    // Berkas dipisah dulu: satu per input + galeri (banyak berkas, satu nama input).
    let (gallery_files, mut files): (Vec<super::form::Upload>, Vec<super::form::Upload>) =
        std::mem::take(&mut form.files).into_iter().partition(|u| u.field == "gallery");
    let mut addons: Vec<String> = form.all("addon").to_vec();
    let get = |k: &str, max: usize| form.get(k, max);

    let isi = match baca_isi(&form) {
        Ok(i) => i,
        Err(m) => return back_with_error(m),
    };
    let (bride_name, groom_name) = (isi.bride.name.clone(), isi.groom.name.clone());
    let contact = fmt::wa_number(&get("contact_phone", 20));
    if contact.is_empty() {
        return back_with_error("Nomor WhatsApp pemesan wajib diisi (untuk konfirmasi pembayaran).");
    }

    // Tautan undangan = nama + kunci acak: tak bisa ditebak dan tak bentrok
    // walau nama mempelai sama. Slug dikunci SEBELUM unggah dan tak pernah
    // diganti sesudahnya — folder RustFS foto/{slug}/ mengikutinya, jadi slug
    // yang berganti setelah unggah = berkas di folder lain / menimpa milik
    // undangan lain.
    let mut slug = String::new();
    for _ in 0..5 {
        let s = new_invitation_id(&bride_name, &groom_name);
        match repo::slug_taken(&state.pool, &s).await {
            Ok(false) => {
                slug = s;
                break;
            }
            Ok(true) => continue,
            Err(e) => {
                tracing::error!(error = %format!("{e:#}"), "buat: cek slug");
                return back_with_error("Gagal menyimpan undangan, coba lagi.");
            }
        }
    }
    if slug.is_empty() {
        return back_with_error("Gagal menyimpan undangan, coba lagi.");
    }

    let theme = get("theme", 60);
    let theme = if state.themes().get(&theme).is_some() { theme } else { crate::web::skin::DEFAULT_THEME.to_string() };
    let konten = state.konten();
    let package = konten.package_or_default(&get("package", 20)).slug;
    addons.retain(|a| konten.addon.iter().any(|x| x.slug == *a));
    let coupon = get("coupon", 30).to_uppercase();
    let (_, _, total) = konten.calc_total(&package, &addons, &coupon);
    let payment = get("payment", 20);
    let payment = if themes::PAYMENT_METHODS.iter().any(|(k, _, _)| *k == payment) { payment } else { themes::PAYMENT_METHODS[0].0.to_string() };

    // ── Unggahan (opsional; tanpa RustFS dilewati) ──
    // Jalur terbaca per undangan: foto/{slug}/sampul.jpg, musik/{slug}/{nama-lagu}.mp3.
    let mut urls: HashMap<&str, String> = HashMap::new();
    // Semua yang sudah terunggah — dihapus lagi bila langkah berikutnya gagal
    // (tanpa ini objek yatim menumpuk di RustFS).
    let mut uploaded: Vec<String> = Vec::new();
    // Musik TIDAK diunggah pemesan (migrasi 027): hanya dari pustaka admin.
    for (key, nama) in [("bride_photo", "mempelai-wanita"), ("groom_photo", "mempelai-pria"), ("cover_photo", "sampul")] {
        let Some(i) = files.iter().position(|u| u.field == key) else { continue };
        let up = files.swap_remove(i);
        let Some(st) = state.storage.as_ref() else {
            tracing::warn!(field = key, "buat: RustFS belum dikonfigurasi — unggahan dilewati");
            continue;
        };
        match st.upload_image_as(up.data, &slug, nama, super::storage::Ukuran::Foto).await {
            Ok(url) => {
                uploaded.push(url.clone());
                urls.insert(key, url);
            }
            Err(e) => {
                discard_uploads(&state, &uploaded).await;
                return back_with_error(&e.to_string());
            }
        }
    }

    // Video prewedding (tema sinema): unggahan, atau tautan https langsung ke
    // berkas .mp4/.webm milik pembeli.
    let mut video_url = String::new();
    if let Some(i) = files.iter().position(|u| u.field == super::form::VIDEO_FIELD) {
        let up = files.swap_remove(i);
        match state.storage.as_ref() {
            None => tracing::warn!("buat: RustFS belum dikonfigurasi — video dilewati"),
            Some(st) => match st.upload_video_as(up.data, &slug, &up.file_name).await {
                Ok(url) => {
                    uploaded.push(url.clone());
                    video_url = url;
                }
                Err(e) => {
                    discard_uploads(&state, &uploaded).await;
                    return back_with_error(&e.to_string());
                }
            },
        }
    }
    if video_url.is_empty() {
        let link = get("video_link", 300);
        let path = link.split(['?', '#']).next().unwrap_or("").to_ascii_lowercase();
        if link.starts_with("https://") && crate::web::skin::is_safe_url(&link) && (path.ends_with(".mp4") || path.ends_with(".webm")) {
            video_url = link;
        }
    }

    let mut gallery: Vec<String> = Vec::new();
    for (i, up) in gallery_files.into_iter().take(MAX_GALLERY).enumerate() {
        let Some(st) = state.storage.as_ref() else {
            tracing::warn!("buat: RustFS belum dikonfigurasi — galeri dilewati");
            break;
        };
        match st.upload_image_as(up.data, &slug, &format!("galeri-{}", i + 1), super::storage::Ukuran::Foto).await {
            Ok(url) => {
                uploaded.push(url.clone());
                gallery.push(url);
            }
            Err(e) => {
                discard_uploads(&state, &uploaded).await;
                return back_with_error(&format!("Galeri: {e}"));
            }
        }
    }

    // ── Musik: HANYA dari pustaka admin (diverifikasi ulang: harus lagu aktif) ──
    let song = match get("music_song", 20).parse::<i64>() {
        Ok(id) => repo::active_song(&state.pool, id).await.unwrap_or_else(|e| {
            tracing::warn!(error = %format!("{e:#}"), "buat: pustaka lagu");
            None
        }),
        Err(_) => None,
    };
    let (music_title, music_artist, mut music_url) = match song {
        Some(s) => (s.title, s.artist, s.url),
        None => (String::new(), String::new(), String::new()),
    };
    // Titik mulai lagu & posisi foto disimpan sebagai fragmen URL (fmt.rs).
    if !music_url.is_empty() {
        music_url.push_str(&fmt::music_start_fragment(&get("music_start", 10)));
    }
    for key in ["bride_photo", "groom_photo", "cover_photo"] {
        if let Some(url) = urls.get_mut(key) {
            url.push_str(&fmt::photo_pos_fragment(&get(&format!("{key}_pos"), 30)));
        }
    }

    // ±200 bit acak; yang disimpan hanya hash-nya — kunci polos tampil sekali
    // di tautan Kelola setelah pesan (dan bisa diterbitkan ulang oleh admin).
    let manage_key = random_key(40);
    let IsiTeks { mut bride, mut groom, events, dress_code, quote_text, quote_source, banks, family_name, love_story, live_url, dress_colors, music_autoplay } = isi;
    bride.photo = urls.remove("bride_photo").unwrap_or_default();
    groom.photo = urls.remove("groom_photo").unwrap_or_default();
    let n = NewInvitation {
        slug: slug.clone(),
        manage_key_hash: auth::token_hash(&manage_key),
        theme,
        package,
        bride,
        groom,
        events,
        dress_code,
        quote_text,
        quote_source,
        music_title,
        music_artist,
        music_url,
        music_autoplay,
        banks,
        family_name,
        addons: json!(addons),
        coupon,
        total_price: total,
        payment_method: payment,
        contact_phone: contact,
        cover_photo: urls.remove("cover_photo").unwrap_or_default(),
        love_story: json!(love_story),
        live_url,
        gallery: json!(gallery),
        dress_colors,
    };

    // Slug TIDAK diganti di sini (berkas sudah di foto/{slug}/). Bentrok hanya
    // mungkin bila pemesan lain merebut slug yang sama di antara cek & INSERT
    // (±40 bit acak) — berkas di folder itu bisa jadi miliknya, jadi jangan dihapus.
    if let Err(e) = repo::create_invitation(&state.pool, &n).await {
        if repo::is_unique_violation(&e) {
            tracing::warn!(slug = %slug, "buat: slug direbut di antara cek & insert — unggahan dibiarkan");
        } else {
            tracing::error!(error = %format!("{e:#}"), "buat: insert");
            discard_uploads(&state, &uploaded).await;
        }
        return back_with_error("Gagal menyimpan undangan, coba lagi.");
    }
    if !video_url.is_empty() {
        match repo::set_invitation_video(&state.pool, &slug, &video_url).await {
            Ok(true) => {}
            Ok(false) => {
                tracing::warn!(slug = %slug, "buat: kolom video_url belum ada (migrasi 022) — video dibuang");
                if state.storage.as_ref().is_some_and(|st| st.key_of(&video_url).is_some()) {
                    discard_uploads(&state, std::slice::from_ref(&video_url)).await;
                }
            }
            Err(e) => tracing::error!(error = %format!("{e:#}"), "buat: simpan video"),
        }
    }
    tracing::info!(slug = %slug, total, "undangan baru dibuat");
    // Tautan Kelola ke WA pemesan: satu-satunya kunci dashboard — kalau tab
    // tertutup sebelum disimpan, pemesan tetap memegangnya di WhatsApp.
    if let Some(wa) = state.wa.as_ref() {
        let konten = state.konten();
        let text = pesan_pesanan_baru(
            &format!("{} & {}", n.bride.name, n.groom.name),
            &public_origin(&state, &headers),
            &slug,
            &manage_key,
            total,
            &konten.pembayaran,
            state.unpaid_ttl_hours,
        );
        wa.spawn_text(n.contact_phone.clone(), text, "pesanan baru → pemesan");
    }
    Redirect::to(&format!("/kelola/{slug}?key={manage_key}&baru=1")).into_response()
}



/// POST /kelola/{slug}/sunting (multipart) — pemilik (kunci/cookie Kelola)
/// atau peran Admin menyunting ISI undangan: teks (sama dengan /buat lewat
/// baca_isi), tema, foto mempelai & sampul (ganti / hapus / posisi), galeri
/// (hapus sebagian + tambah), video, lagu dari pustaka, WA pemesan. Paket &
/// harga tidak bisa diubah di sini. Berkas lama yang diganti/dihapus dibuang
/// dari RustFS SETELAH data tersimpan.
pub async fn update_invitation(
    Extension(state): Extension<Arc<AppState>>,
    Path(slug): Path<String>,
    headers: axum::http::HeaderMap,
    mp: Multipart,
) -> Response {
    let back = format!("/kelola/{}/sunting", fmt::key(&slug));
    if let Err(secs) = state.write_limit.hit(&format!("sunting:{}:{slug}", security::client_ip(&headers))) {
        return to(&back, "galat", &format!("Terlalu sering menyimpan. Coba lagi dalam {} menit.", secs.div_ceil(60)));
    }
    let Ok(mut form) = super::form::read(mp, 4 + MAX_GALLERY).await else {
        return to(&back, "galat", "Unggahan terputus atau terlalu besar (foto maks 5 MB, video maks 20 MB).");
    };
    // Galeri: `galeri_baru` (slot kosong) & `galeri_ganti_{i}` (timpa foto ke-i).
    let (gallery_files, mut files): (Vec<super::form::Upload>, Vec<super::form::Upload>) =
        std::mem::take(&mut form.files).into_iter().partition(|u| u.field == "galeri_baru" || u.field == "gallery" || u.field.starts_with("galeri_ganti_"));
    let row = match repo::invitation(&state.pool, &slug).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, "Undangan tidak ditemukan").into_response(),
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "sunting: muat undangan");
            return to(&back, "galat", "Server sedang sibuk, coba lagi sebentar.");
        }
    };
    let key = Some(form.raw("key")).filter(|k| !k.trim().is_empty()).or_else(|| super::owner::key_from(&headers, &slug)).unwrap_or_default();
    let owner = !key.trim().is_empty() && auth::same_hash(&auth::token_hash(key.trim()), &row.manage_key_hash);
    if !owner && auth::require(&state, &headers, true).await.is_err() {
        return to(&back, "galat", "Kunci kelola tidak valid. Buka dari tautan yang Anda terima saat memesan.");
    }
    if row.inv.is_demo {
        return to(&back, "galat", "Undangan demo tidak bisa disunting.");
    }
    let isi = match baca_isi(&form) {
        Ok(i) => i,
        Err(m) => return to(&back, "galat", m),
    };
    let get = |k: &str, max: usize| form.get(k, max);
    let theme = get("theme", 60);
    let theme = if state.themes().get(&theme).is_some() { theme } else { row.inv.theme.clone() };
    let contact = fmt::wa_number(&get("contact_phone", 20));
    let contact = if contact.is_empty() { row.contact_phone.clone() } else { contact };

    // Unggahan baru (dibuang bila langkah berikutnya gagal) & berkas lama yang
    // tak dipakai lagi (dibuang setelah tersimpan).
    let mut uploaded: Vec<String> = Vec::new();
    let mut lama: Vec<String> = Vec::new();
    let base_of = |u: &str| u.split('#').next().unwrap_or("").to_string();

    // ── Foto mempelai & sampul: ganti (unggah) / hapus / tetap (+ posisi baru) ──
    let mut foto: HashMap<&str, String> = HashMap::new();
    for (k, nama, sekarang) in [
        ("bride_photo", "mempelai-wanita", &row.inv.bride_photo),
        ("groom_photo", "mempelai-pria", &row.inv.groom_photo),
        ("cover_photo", "sampul", &row.inv.cover_photo),
    ] {
        let pos = fmt::photo_pos_fragment(&get(&format!("{k}_pos"), 30));
        let baru = files.iter().position(|u| u.field == k).map(|i| files.swap_remove(i));
        let url = if let Some(up) = baru {
            let Some(st) = state.storage.as_ref() else {
                return to(&back, "galat", "Penyimpanan foto belum dikonfigurasi — hubungi admin.");
            };
            match ganti_foto(st, up.data, sekarang, &slug, nama).await {
                Ok((u, timpa)) => {
                    // Timpa = berkas yang sama ditulis ulang (tak bisa "dibatalkan");
                    // unggahan baru dicatat agar dibuang bila langkah berikutnya gagal.
                    if !timpa {
                        uploaded.push(u.clone());
                        lama.push(sekarang.clone());
                    }
                    format!("{u}{pos}")
                }
                Err(e) => {
                    discard_uploads(&state, &uploaded).await;
                    return to(&back, "galat", &e.to_string());
                }
            }
        } else if form.has(&format!("hapus_{k}")) {
            lama.push(sekarang.clone());
            String::new()
        } else if sekarang.is_empty() {
            String::new()
        } else if pos.is_empty() {
            sekarang.clone()
        } else {
            format!("{}{pos}", base_of(sekarang))
        };
        foto.insert(k, url);
    }

    // ── Galeri (6 slot): dicentang hapus → dibuang dari RustFS; berkas di slot
    //    berisi → TIMPA foto itu di tempat (urutan tetap); slot kosong → tambah.
    let hapus: Vec<String> = form.all("galeri_hapus").to_vec();
    let (mut ganti, baru): (Vec<super::form::Upload>, Vec<super::form::Upload>) =
        gallery_files.into_iter().partition(|u| u.field.starts_with("galeri_ganti_"));
    let mut gallery: Vec<String> = Vec::new();
    for (i, g) in row.inv.gallery.iter().enumerate() {
        if hapus.iter().any(|h| *h == i.to_string()) {
            lama.push(g.clone());
            continue;
        }
        let Some(j) = ganti.iter().position(|u| u.field == format!("galeri_ganti_{i}")) else {
            gallery.push(g.clone());
            continue;
        };
        let up = ganti.swap_remove(j);
        let Some(st) = state.storage.as_ref() else {
            gallery.push(g.clone());
            continue;
        };
        match ganti_foto(st, up.data, g, &slug, &format!("galeri-{}", i + 1)).await {
            Ok((u, timpa)) => {
                if !timpa {
                    uploaded.push(u.clone());
                    lama.push(g.clone());
                }
                gallery.push(u);
            }
            Err(e) => {
                discard_uploads(&state, &uploaded).await;
                return to(&back, "galat", &format!("Galeri foto {}: {e}", i + 1));
            }
        }
    }
    for up in baru {
        if gallery.len() >= MAX_GALLERY {
            break;
        }
        let Some(st) = state.storage.as_ref() else { break };
        match st.upload_image_as(up.data, &slug, &format!("galeri-{}", auth::random_hex(3)), super::storage::Ukuran::Foto).await {
            Ok(u) => {
                uploaded.push(u.clone());
                gallery.push(u);
            }
            Err(e) => {
                discard_uploads(&state, &uploaded).await;
                return to(&back, "galat", &format!("Galeri: {e}"));
            }
        }
    }

    // ── Video prewedding ──
    let mut video_url = row.inv.video_url.clone();
    if let Some(i) = files.iter().position(|u| u.field == super::form::VIDEO_FIELD) {
        let up = files.swap_remove(i);
        if let Some(st) = state.storage.as_ref() {
            match st.upload_video_as(up.data, &slug, &up.file_name).await {
                Ok(u) => {
                    uploaded.push(u.clone());
                    lama.push(std::mem::replace(&mut video_url, u));
                }
                Err(e) => {
                    discard_uploads(&state, &uploaded).await;
                    return to(&back, "galat", &e.to_string());
                }
            }
        }
    } else if form.has("hapus_video") {
        lama.push(std::mem::take(&mut video_url));
    } else {
        let link = get("video_link", 300);
        let path = link.split(['?', '#']).next().unwrap_or("").to_ascii_lowercase();
        if link != video_url && link.starts_with("https://") && crate::web::skin::is_safe_url(&link) && (path.ends_with(".mp4") || path.ends_with(".webm")) {
            lama.push(std::mem::replace(&mut video_url, link));
        }
    }

    // ── Lagu: pustaka admin; -1 = tetap lagu sekarang ──
    let (mut music_title, mut music_artist, mut music_url) = match get("music_song", 20).parse::<i64>() {
        Ok(-1) => (row.inv.music_title.clone(), row.inv.music_artist.clone(), base_of(&row.inv.music_url)),
        Ok(id) if id > 0 => match repo::active_song(&state.pool, id).await {
            Ok(Some(s)) => (s.title, s.artist, s.url),
            _ => (row.inv.music_title.clone(), row.inv.music_artist.clone(), base_of(&row.inv.music_url)),
        },
        _ => (String::new(), String::new(), String::new()),
    };
    if music_url.is_empty() {
        (music_title, music_artist) = (String::new(), String::new());
    } else {
        music_url.push_str(&fmt::music_start_fragment(&get("music_start", 10)));
    }

    // Kisah cinta: foto per babak (diatur admin) dipertahankan per urutan.
    let mut love_story = isi.love_story;
    for (i, ls) in love_story.iter_mut().enumerate() {
        if let Some(old) = row.inv.love_story.get(i).filter(|o| !o.img.is_empty()) {
            ls["img"] = json!(old.img);
        }
    }

    let IsiTeks { mut bride, mut groom, events, dress_code, quote_text, quote_source, banks, family_name, live_url, dress_colors, music_autoplay, .. } = isi;
    bride.photo = foto.remove("bride_photo").unwrap_or_default();
    groom.photo = foto.remove("groom_photo").unwrap_or_default();
    let n = NewInvitation {
        slug: slug.clone(),
        manage_key_hash: String::new(),
        theme,
        package: row.inv.package.clone(),
        bride,
        groom,
        events,
        dress_code,
        quote_text,
        quote_source,
        music_title,
        music_artist,
        music_url,
        music_autoplay,
        banks,
        family_name,
        addons: json!([]),
        coupon: String::new(),
        total_price: row.total_price,
        payment_method: row.payment_method.clone(),
        contact_phone: contact,
        cover_photo: foto.remove("cover_photo").unwrap_or_default(),
        love_story: json!(love_story),
        live_url,
        gallery: json!(gallery),
        dress_colors,
    };
    if let Err(e) = repo::update_invitation_isi(&state.pool, row.id, &n).await {
        tracing::error!(error = %format!("{e:#}"), slug = %slug, "sunting: simpan");
        discard_uploads(&state, &uploaded).await;
        return to(&back, "galat", "Gagal menyimpan perubahan, coba lagi.");
    }
    if video_url != row.inv.video_url {
        if let Err(e) = repo::set_invitation_video(&state.pool, &slug, &video_url).await {
            tracing::error!(error = %format!("{e:#}"), "sunting: simpan video");
        }
    }
    // Berkas lama: hanya unggahan kita (lagu bawaan / URL luar tak tersentuh).
    let lama: Vec<String> = lama.into_iter().filter(|u| !u.is_empty()).map(|u| base_of(&u)).collect();
    discard_uploads(&state, &lama).await;
    tracing::info!(slug = %slug, admin = !owner, "undangan disunting");
    to(&back, "ok", "Perubahan tersimpan — buka \"Lihat Undangan\" untuk melihat hasilnya.")
}

/// Ganti satu foto undangan: TIMPA berkas lama di kunci yang sama bila itu
/// unggahan kita (`true`), selain itu unggah sebagai berkas baru (`false`).
async fn ganti_foto(st: &super::storage::StorageService, data: Vec<u8>, lama: &str, slug: &str, nama: &str) -> anyhow::Result<(String, bool)> {
    if st.key_of(lama).is_some() {
        if let Some(u) = st.replace_image(data, lama, super::storage::Ukuran::Foto).await? {
            return Ok((u, true));
        }
        anyhow::bail!("Foto lama tak bisa ditimpa, coba lagi.");
    }
    let u = st.upload_image_as(data, slug, &format!("{nama}-{}", auth::random_hex(3)), super::storage::Ukuran::Foto).await?;
    Ok((u, false))
}

fn csv_cell(s: &str) -> String {
    // Satu baris per tamu; cegah formula injection saat dibuka di Excel/Sheets.
    let s = s.replace(['\n', '\r'], " ");
    let s = if s.starts_with(['=', '+', '-', '@', '\t']) { format!("'{s}") } else { s };
    format!("\"{}\"", s.replace('"', "\"\""))
}

pub async fn export_guests(
    Extension(state): Extension<Arc<AppState>>,
    Path(slug): Path<String>,
    Query(q): Query<HashMap<String, String>>,
    headers: axum::http::HeaderMap,
) -> Response {
    // Kunci dari cookie (server/owner.rs); ?key= tetap diterima.
    let cookie_key = super::owner::key_from(&headers, &slug);
    let key = q.get("key").map(String::as_str).filter(|k| !k.is_empty()).or(cookie_key.as_deref()).unwrap_or("");
    // Pemilik (kunci cocok) atau peran Admin yang sedang masuk.
    let admin = || async { auth::require(&state, &headers, true).await.is_ok() };
    let inv = match repo::invitation(&state.pool, &slug).await {
        Ok(Some(i)) if auth::same_hash(&auth::token_hash(key), &i.manage_key_hash) => i,
        Ok(Some(i)) if admin().await => i,
        Ok(_) => return (StatusCode::FORBIDDEN, "Kunci kelola tidak valid").into_response(),
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "export");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Gagal memuat data").into_response();
        }
    };
    let guests = match repo::guests(&state.pool, inv.id).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "export guests");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Gagal memuat data").into_response();
        }
    };
    let mut out = String::from("\u{feff}Kode,Nama,WhatsApp,Kategori,Sesi,Meja,Pax Undangan,Status RSVP,Pax Hadir,Dibuka,Check-in,Tanda Kasih\n");
    for g in guests {
        let row = [
            csv_cell(&g.code),
            csv_cell(&g.name),
            csv_cell(&g.phone),
            csv_cell(crate::web::model::category_label(&g.category)),
            csv_cell(&g.session),
            csv_cell(&g.table_no),
            g.pax.to_string(),
            csv_cell(crate::web::model::rsvp_label(&g.rsvp)),
            g.rsvp_pax.to_string(),
            if g.opened { "Ya" } else { "Belum" }.into(),
            if g.checked_in { "Ya" } else { "Belum" }.into(),
            g.gift_amount.to_string(),
        ];
        out.push_str(&row.join(","));
        out.push('\n');
    }
    (
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8".to_string()),
            (header::CONTENT_DISPOSITION, format!("attachment; filename=\"tamu-{slug}.csv\"")),
        ],
        out,
    )
        .into_response()
}

#[cfg(test)]
mod tests {

    #[test]
    fn pesan_sebelum_jangkar() {
        assert_eq!(with_notice("/admin/banner#banner-5", "ok", "Siap"), "/admin/banner?ok=Siap#banner-5");
        assert_eq!(with_notice("/admin/undangan?q=a", "galat", "x"), "/admin/undangan?q=a&galat=x");
        assert_eq!(with_notice("/admin", "ok", "y"), "/admin?ok=y");
    }

    use super::*;

    #[test]
    fn id_undangan_nama_plus_kunci() {
        let mut seen = std::collections::HashSet::new();
        for _ in 0..5_000 {
            // Nama pasangan SAMA → tautan tetap berbeda.
            let id = new_invitation_id("Yona", "Doni");
            let key = id.strip_prefix("yona-doni-").expect("awalan nama");
            assert_eq!(key.len(), INVITATION_KEY_LEN);
            // Tanpa karakter mirip (0/o, 1/l) agar aman diketik ulang.
            assert!(key.chars().all(|c| (c.is_ascii_lowercase() || c.is_ascii_digit()) && !"0o1l".contains(c)));
            assert!(seen.insert(id), "ID kembar");
        }
        assert_eq!(new_invitation_id("Siti Nur", "Ahmad").split('-').take(2).collect::<Vec<_>>(), ["siti", "ahmad"]);
        // Nama tanpa huruf latin → hanya kunci.
        assert_eq!(new_invitation_id("", "").len(), INVITATION_KEY_LEN);
        assert_eq!(csv_cell("=HYPERLINK()"), "\"'=HYPERLINK()\"");
    }
}

/// Formulir layanan (cetak, sampel, dekorasi, MUA) → 303 ke WhatsApp admin
/// dengan pesan tersusun. Estimasi cetak dihitung ulang di sini, bukan dari
/// angka di browser. Tanpa ADMIN_WHATSAPP, wa.me membiarkan pengguna memilih
/// kontak sendiri.
pub async fn layanan_wa(
    Extension(state): Extension<Arc<AppState>>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let konten = state.konten();
    let text = crate::web::layanan::pesan_wa(&konten, |k| q.get(k).map(|v| clean(v, 300)).unwrap_or_default());
    Redirect::to(&format!("https://wa.me/{}?text={}", state.admin_wa, fmt::url_encode(&text))).into_response()
}

// ── Tema & admin ───────────────────────────────────────────────────────────

pub async fn theme_css(Extension(state): Extension<Arc<AppState>>) -> Response {
    let cat = state.themes();
    (
        [
            (header::CONTENT_TYPE, "text/css; charset=utf-8"),
            // URL selalu memuat ?v=hash-isi → aman di-cache lama.
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
        ],
        cat.css.clone(),
    )
        .into_response()
}

/// GET /sitemap.xml — halaman publik + demo tiap tema yang tampil di katalog +
/// detail paket dekorasi. Dari cache AppState (tanpa query DB per request).
pub async fn sitemap(Extension(state): Extension<Arc<AppState>>) -> Response {
    use crate::web::seo::SITE_URL;
    let cat = state.themes();
    let konten = state.konten();
    let mut urls: Vec<(String, &str, &str)> = [
        ("/", "daily", "1.0"), ("/paket", "weekly", "0.9"), ("/buat", "monthly", "0.8"), ("/panduan", "monthly", "0.7"),
        ("/cetak", "monthly", "0.6"), ("/dekorasi", "monthly", "0.6"), ("/mua", "monthly", "0.6"), ("/seserahan", "monthly", "0.6"),
        ("/privasi", "yearly", "0.2"), ("/syarat", "yearly", "0.2"),
    ]
    .iter()
    .map(|(p, f, pr)| (p.to_string(), *f, *pr))
    .collect();
    urls.extend(cat.list.iter().filter(|t| t.listed && crate::web::skin::is_slug(&t.slug)).map(|t| (format!("/tema/{}", t.slug), "weekly", "0.8")));
    urls.extend(konten.dekor_paket.iter().filter(|p| crate::web::fmt::is_slug(&p.slug, 64)).map(|p| (format!("/dekorasi/{}", p.slug), "monthly", "0.5")));
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");
    for (path, freq, prio) in urls {
        xml.push_str(&format!("  <url><loc>{SITE_URL}{path}</loc><changefreq>{freq}</changefreq><priority>{prio}</priority></url>\n"));
    }
    xml.push_str("</urlset>\n");
    ([(header::CONTENT_TYPE, "application/xml; charset=utf-8"), (header::CACHE_CONTROL, "public, max-age=3600")], xml).into_response()
}

/// GET /readyz — 200 bila Postgres menjawab, 503 bila tidak.
pub async fn readyz(Extension(state): Extension<Arc<AppState>>) -> Response {
    let ok = match state.pool.get().await {
        Ok(c) => c.simple_query("SELECT 1").await.is_ok(),
        Err(_) => false,
    };
    if ok { (StatusCode::OK, "ready").into_response() } else { (StatusCode::SERVICE_UNAVAILABLE, "db tidak siap").into_response() }
}

/// GET /app.js — skrip global (web/global.js). URL selalu ber-`?v=hash`.
pub async fn app_js() -> Response {
    (
        [
            (header::CONTENT_TYPE, "text/javascript; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
        ],
        crate::web::app::GLOBAL_JS,
    )
        .into_response()
}

/// Nilai cookie dari header `Cookie`.
pub fn cookie_value<'a>(headers: &'a axum::http::HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .filter_map(|kv| kv.trim().split_once('='))
        .find(|(k, _)| *k == name)
        .map(|(_, v)| v)
}

fn to(path: &str, key: &str, msg: &str) -> Response {
    Redirect::to(&with_notice(path, key, msg)).into_response()
}

/// `path?key=msg` — query disisipkan SEBELUM `#jangkar` (kalau ditaruh di
/// belakangnya, pesan ikut masuk fragmen dan tak terbaca halaman).
fn with_notice(path: &str, key: &str, msg: &str) -> String {
    let (base, hash) = match path.split_once('#') {
        Some((b, h)) => (b, format!("#{h}")),
        None => (path, String::new()),
    };
    let sep = if base.contains('?') { '&' } else { '?' };
    format!("{base}{sep}{key}={}{hash}", fmt::url_encode(msg))
}

fn with_cookie(mut res: Response, cookie: String) -> Response {
    if let Ok(v) = cookie.parse() {
        res.headers_mut().append(header::SET_COOKIE, v);
    }
    res
}

/// Admin yang masuk & berperan cukup; selain itu redirect dengan pesan.
async fn require(state: &AppState, headers: &axum::http::HeaderMap, need_admin: bool) -> Result<AdminUser, Response> {
    auth::require(state, headers, need_admin).await.map_err(|d| match d {
        auth::Denied::NotAdmin => to("/admin", "galat", "Hanya peran Admin yang boleh melakukan ini."),
        auth::Denied::NoSession => to("/admin", "galat", "Sesi berakhir, silakan masuk lagi."),
    })
}

async fn start_session(state: &AppState, headers: &axum::http::HeaderMap, user_id: i64, to_path: &str) -> Response {
    let token = auth::new_token();
    if let Err(e) = repo::create_session(&state.pool, &auth::token_hash(&token), user_id, auth::SESSION_DAYS).await {
        tracing::error!(error = %format!("{e:#}"), "admin: buat sesi");
        return to("/admin", "galat", "Gagal membuat sesi, coba lagi.");
    }
    let res = with_cookie(Redirect::to(to_path).into_response(), auth::cookie(&token, auth::SESSION_DAYS * 86_400, headers));
    with_cookie(res, auth::marker_cookie(true))
}

pub async fn admin_login(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::Form(f): axum::Form<HashMap<String, String>>,
) -> Response {
    let username = f.get("username").map(|u| u.trim().to_lowercase()).unwrap_or_default();
    let password = f.get("password").cloned().unwrap_or_default();
    let (ku, ki) = (format!("u:{username}"), format!("ip:{}", security::client_ip(&headers)));
    if let Some(secs) = state.login_limit.blocked(&[&ku, &ki]) {
        tracing::warn!(user = %username, ip = %ki, "admin: login diblokir sementara (terlalu banyak gagal)");
        return security::too_many(secs);
    }
    let row = match repo::admin_login_row(&state.pool, &username).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "admin: login");
            return to("/admin", "galat", "Database akun belum siap — jalankan migration/003_admin_konten.sql.");
        }
    };
    let (user, hash) = match row {
        Some((u, h)) if u.active => (Some(u), Some(h)),
        _ => (None, None),
    };
    // Selalu verifikasi (hash palsu bila akun tak ada/nonaktif) — waktu sama.
    let ok = auth::verify_password_async(password, hash).await;
    match user.filter(|_| ok) {
        Some(u) => {
            tracing::info!(user = %u.username, "admin: masuk");
            state.login_limit.clear(&ku);
            start_session(&state, &headers, u.id, "/admin").await
        }
        None => {
            state.login_limit.fail(&[&ku, &ki]);
            tracing::warn!(user = %username, ip = %ki, "admin: login gagal");
            // Perlambat tebak-tebakan sandi.
            tokio::time::sleep(std::time::Duration::from_millis(800)).await;
            to("/admin", "galat", "Username atau sandi salah (atau akun dinonaktifkan).")
        }
    }
}

/// Akun pertama: hanya bila belum ada akun sama sekali, dengan kode ADMIN_TOKEN.
pub async fn admin_setup(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::Form(f): axum::Form<HashMap<String, String>>,
) -> Response {
    let get = |k: &str| f.get(k).cloned().unwrap_or_default();
    match repo::admin_count(&state.pool).await {
        Ok(0) => {}
        Ok(_) => return to("/admin", "galat", "Akun admin sudah ada — silakan masuk."),
        Err(_) => return to("/admin", "galat", "Database akun belum siap — jalankan migration/003_admin_konten.sql."),
    }
    let ki = format!("setup-ip:{}", security::client_ip(&headers));
    if let Some(secs) = state.login_limit.blocked(&[&ki]) {
        return security::too_many(secs);
    }
    if state.admin_token.is_empty() || !auth::same_hash(get("code").trim(), &state.admin_token) {
        state.login_limit.fail(&[&ki]);
        tracing::warn!(ip = %ki, "admin: kode setup salah");
        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
        return to("/admin", "galat", "Kode setup (ADMIN_TOKEN) salah.");
    }
    let username = get("username").trim().to_lowercase();
    let (pw, pw2) = (get("password"), get("password2"));
    if !auth::valid_username(&username) {
        return to("/admin", "galat", "Username 3–32 karakter: huruf kecil, angka, titik, minus, garis bawah.");
    }
    if pw.chars().count() < auth::MIN_PASSWORD || pw != pw2 {
        return to("/admin", "galat", "Sandi minimal 8 karakter dan kedua isian harus sama.");
    }
    let hash = match auth::hash_password_async(pw.clone()).await {
        Ok(h) => h,
        Err(_) => return to("/admin", "galat", "Gagal memproses sandi."),
    };
    match repo::create_admin(&state.pool, &username, &clean(&get("name"), 60), &hash, "admin").await {
        Ok(id) => {
            tracing::info!(user = %username, "admin: akun pertama dibuat");
            start_session(&state, &headers, id, "/admin?ok=Akun%20admin%20dibuat.").await
        }
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "admin: setup");
            to("/admin", "galat", "Gagal membuat akun.")
        }
    }
}

pub async fn admin_logout(Extension(state): Extension<Arc<AppState>>, headers: axum::http::HeaderMap) -> Response {
    if let Some(t) = cookie_value(&headers, auth::COOKIE) {
        let _ = repo::delete_session(&state.pool, &auth::token_hash(t)).await;
    }
    let res = with_cookie(Redirect::to("/admin").into_response(), auth::cookie("", 0, &headers));
    with_cookie(res, auth::marker_cookie(false))
}

/// Admin mengelola akun lain: `aksi` = baru | ubah | sandi.
pub async fn admin_account_save(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::Form(f): axum::Form<HashMap<String, String>>,
) -> Response {
    let me = match require(&state, &headers, true).await {
        Ok(u) => u,
        Err(r) => return r,
    };
    let get = |k: &str| f.get(k).cloned().unwrap_or_default();
    let back = "/admin/akun";
    let role = get("role");
    if !role.is_empty() && !crate::web::model::ADMIN_ROLES.iter().any(|r| r.0 == role) {
        return to(back, "galat", "Peran tidak dikenal.");
    }
    match get("aksi").as_str() {
        "baru" => {
            let username = get("username").trim().to_lowercase();
            let pw = get("password");
            if !auth::valid_username(&username) {
                return to(back, "galat", "Username 3–32 karakter: huruf kecil, angka, titik, minus, garis bawah.");
            }
            if pw.chars().count() < auth::MIN_PASSWORD {
                return to(back, "galat", "Sandi minimal 8 karakter.");
            }
            let Ok(hash) = auth::hash_password_async(pw.clone()).await else { return to(back, "galat", "Gagal memproses sandi.") };
            match repo::create_admin(&state.pool, &username, &clean(&get("name"), 60), &hash, &role).await {
                Ok(_) => {
                    tracing::info!(by = %me.username, user = %username, role = %role, "admin: akun dibuat");
                    to(back, "ok", &format!("Akun {username} dibuat. Berikan username & sandinya secara pribadi."))
                }
                Err(e) if repo::is_unique_violation(&e) => to(back, "galat", "Username sudah dipakai."),
                Err(e) => {
                    tracing::error!(error = %format!("{e:#}"), "admin: buat akun");
                    to(back, "galat", "Gagal membuat akun.")
                }
            }
        }
        "ubah" => {
            let id: i64 = get("id").parse().unwrap_or(0);
            let active = get("active") == "1";
            // Jangan sampai panel tak punya Admin aktif (termasuk menurunkan diri sendiri).
            if (role != "admin" || !active) && repo::other_active_admins(&state.pool, id).await.unwrap_or(0) == 0 {
                return to(back, "galat", "Harus ada minimal satu Admin aktif.");
            }
            match repo::update_admin(&state.pool, id, &clean(&get("name"), 60), &role, active).await {
                Ok(()) => to(back, "ok", "Akun diperbarui."),
                Err(e) => {
                    tracing::error!(error = %format!("{e:#}"), "admin: ubah akun");
                    to(back, "galat", "Gagal menyimpan akun.")
                }
            }
        }
        "sandi" => {
            let id: i64 = get("id").parse().unwrap_or(0);
            let pw = get("password");
            if pw.chars().count() < auth::MIN_PASSWORD {
                return to(back, "galat", "Sandi baru minimal 8 karakter.");
            }
            let Ok(hash) = auth::hash_password_async(pw.clone()).await else { return to(back, "galat", "Gagal memproses sandi.") };
            match repo::set_admin_password(&state.pool, id, &hash, None).await {
                Ok(()) => to(back, "ok", "Sandi diganti; sesi lama akun itu dikeluarkan."),
                Err(_) => to(back, "galat", "Gagal mengganti sandi."),
            }
        }
        _ => to(back, "galat", "Aksi tidak dikenal."),
    }
}

/// Ganti sandi sendiri (semua peran).
pub async fn admin_own_password(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::Form(f): axum::Form<HashMap<String, String>>,
) -> Response {
    let me = match require(&state, &headers, false).await {
        Ok(u) => u,
        Err(r) => return r,
    };
    let get = |k: &str| f.get(k).cloned().unwrap_or_default();
    let back = "/admin/profil";
    let Ok(Some((_, hash))) = repo::admin_login_row(&state.pool, &me.username).await else {
        return to(back, "galat", "Akun tidak ditemukan.");
    };
    if !auth::verify_password_async(get("old"), Some(hash)).await {
        return to(back, "galat", "Sandi lama salah.");
    }
    let (pw, pw2) = (get("password"), get("password2"));
    if pw.chars().count() < auth::MIN_PASSWORD || pw != pw2 {
        return to(back, "galat", "Sandi baru minimal 8 karakter dan kedua isian harus sama.");
    }
    let Ok(new_hash) = auth::hash_password_async(pw.clone()).await else { return to(back, "galat", "Gagal memproses sandi.") };
    let keep = cookie_value(&headers, auth::COOKIE).map(auth::token_hash);
    match repo::set_admin_password(&state.pool, me.id, &new_hash, keep.as_deref()).await {
        Ok(()) => to(back, "ok", "Sandi diganti. Sesi di perangkat lain dikeluarkan."),
        Err(_) => to(back, "galat", "Gagal mengganti sandi."),
    }
}

/// Simpan satu bagian konten dari editor generik (multipart: ada unggahan foto).
pub async fn admin_save_konten(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    mp: Multipart,
) -> Response {
    let me = match require(&state, &headers, false).await {
        Ok(u) => u,
        Err(r) => return r,
    };
    let Ok(mut form) = super::form::read(mp, 40).await else {
        return to("/admin/konten", "galat", "Unggahan terputus atau terlalu besar (foto maks 5 MB).");
    };
    let files = std::mem::take(&mut form.files);
    let mut f = form.fields;
    let key = f.get("section").cloned().unwrap_or_default();
    let Some(sec) = crate::web::konten::section(&key) else {
        return to("/admin/konten", "galat", "Bagian konten tidak dikenal.");
    };
    let back = format!("/admin/konten/{key}");
    // Galeri: foto bercentang "Hapus" dibuang dari daftarnya (`…__del__{j}`).
    let dels: Vec<(String, String)> = f
        .iter()
        .filter_map(|(k, v)| k.split_once("__del__").map(|(target, _)| (target.to_string(), v.clone())))
        .collect();
    for (target, url) in dels {
        if let Some(list) = f.get_mut(&target) {
            *list = list.lines().filter(|l| l.trim() != url).collect::<Vec<_>>().join("\n");
        }
    }
    // Unggahan: Gambar → menimpa isian URL-nya; Galeri → ditambahkan di akhir.
    // Jalur terbaca: foto/{bagian}-{kode item}/{nama-berkas-asli}.jpg.
    for up in files {
        let Some(target) = up.field.strip_suffix("__file") else { continue };
        let Some(st) = state.storage.as_ref() else {
            return to(&back, "galat", "RustFS belum dikonfigurasi — isi URL/path gambar saja (mis. /img/layanan/…).");
        };
        let (item, field_key) = target.split_once("__").unwrap_or(("", target));
        let is_gallery = sec.fields.iter().any(|fl| fl.key == field_key && fl.kind == crate::web::konten::Kind::Gallery);
        let item_key = [format!("{item}__slug"), format!("{item}__name"), format!("{item}__judul")]
            .iter()
            .filter_map(|k| f.get(k))
            .map(|v| fmt::key(v))
            .find(|v| !v.is_empty())
            .unwrap_or_else(|| "umum".into());
        let dir = format!("{}-{item_key}", fmt::key(&key));
        let current = f.get(target).cloned().unwrap_or_default();
        // Nama sama dengan foto yang sudah ada di daftar → beri akhiran -2, -3, …
        let name = super::storage::StorageService::unique_name(&super::storage::file_stem(&up.file_name, "foto"), |n| {
            current.lines().any(|l| l.contains(&format!("/{dir}/{n}.")))
        });
        match st.upload_image_as(up.data, &dir, &name, super::storage::Ukuran::Aset).await {
            Ok(url) if is_gallery => {
                let lines = if current.trim().is_empty() { url } else { format!("{current}\n{url}") };
                f.insert(target.to_string(), lines);
            }
            Ok(url) => {
                f.insert(target.to_string(), url);
            }
            Err(e) => return to(&back, "galat", &e.to_string()),
        }
    }
    let n: usize = f.get("n").and_then(|v| v.parse().ok()).unwrap_or(0);
    let data = match crate::web::konten::form_to_json(sec, n, |k| f.get(k).cloned().unwrap_or_default()) {
        Ok(d) => d,
        Err(msg) => return to(&back, "galat", &msg),
    };
    // Validasi akhir: harus bisa dibaca sebagai tipe Konten.
    let mut probe = (*state.konten()).clone();
    if !probe.apply(&key, data.clone()) {
        return to(&back, "galat", "Isian tidak valid untuk bagian ini.");
    }
    if let Err(e) = repo::save_content(&state.pool, &key, &data, &me.username).await {
        tracing::error!(error = %format!("{e:#}"), "admin: simpan konten");
        return to(&back, "galat", "Gagal menyimpan — sudah menjalankan migration/003_admin_konten.sql?");
    }
    state.refresh_konten().await;
    tracing::info!(by = %me.username, section = %key, "admin: konten disimpan");
    to(&back, "ok", "Tersimpan & langsung tayang.")
}

pub async fn admin_save_theme(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    mp: Multipart,
) -> Response {
    if let Err(r) = require(&state, &headers, false).await {
        return r;
    }
    // Unggahan: image_file → image_url, bg_file → bg_image, dst.
    let Ok(mut form) = super::form::read(mp, 4).await else {
        return to("/admin/tema", "galat", "Unggahan terputus atau terlalu besar (gambar maks 5 MB).");
    };
    let uploads = std::mem::take(&mut form.files);
    let mut f = form.fields;
    let editing = f.get("orig").cloned().unwrap_or_default();
    // Slug tidak bisa diganti saat menyunting (undangan menyimpan slug-nya).
    if !editing.is_empty() {
        f.insert("slug".into(), editing.clone());
    }
    let back = if editing.is_empty() { "/admin/tema/baru".to_string() } else { format!("/admin/tema/{editing}") };
    let mut theme = match crate::web::skin::from_form(|k| f.get(k).cloned().unwrap_or_default()) {
        Ok(t) => t,
        Err(msg) => return to(&back, "galat", &msg),
    };
    if editing.is_empty() {
        match repo::theme_exists(&state.pool, &theme.slug).await {
            Ok(false) => {}
            Ok(true) => return to(&back, "galat", &format!("Kode tema \"{}\" sudah dipakai tema lain.", theme.slug)),
            Err(e) => {
                tracing::error!(error = %format!("{e:#}"), "admin: cek tema");
                return to(&back, "galat", "Database belum siap — sudah menjalankan migration/002_themes.sql?");
            }
        }
    }
    if f.get("image_clear").is_some_and(|v| v == "1") {
        theme.image_url.clear();
    }
    // Animasi harus ada di katalog dan jenisnya cocok.
    {
        let cat = state.themes();
        for (key, kind, label) in [(&theme.open_anim, "buka", "Cara membuka"), (&theme.scroll_anim, "scroll", "Gerak saat scroll"), (&theme.float_deco, "hiasan", "Hiasan melayang")] {
            if !cat.has_anim(key, kind) {
                return to(&back, "galat", &format!("{label}: animasi \"{key}\" tidak ditemukan."));
            }
        }
    }
    for up in uploads {
        let Some(st) = state.storage.as_ref() else {
            return to(&back, "galat", "RustFS belum dikonfigurasi — tempel URL gambar saja.");
        };
        // Jalur terbaca: foto/tema/{slug}/{nama-berkas-asli}.webp
        let url = match st.upload_image_as(up.data, &format!("tema-{}", theme.slug), &up.file_name, super::storage::Ukuran::Aset).await {
            Ok(url) => url,
            Err(e) => return to(&back, "galat", &e.to_string()),
        };
        match up.field.as_str() {
            "image_file" => theme.image_url = url,
            "bg_file" => theme.bg_image = url,
            "frame_file" => theme.frame_image = url,
            "deco_file" => theme.card_deco = url,
            _ => {}
        }
    }
    if let Err(e) = repo::upsert_theme(&state.pool, &theme).await {
        tracing::error!(error = %format!("{e:#}"), "admin: simpan tema");
        return to(&back, "galat", "Gagal menyimpan tema — sudah menjalankan migration/002_themes.sql?");
    }
    state.refresh_themes().await;
    tracing::info!(slug = %theme.slug, "admin: tema disimpan");
    to(&format!("/admin/tema/{}", theme.slug), "ok", "Tema tersimpan & langsung tayang.")
}

pub async fn admin_delete_theme(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::Form(f): axum::Form<HashMap<String, String>>,
) -> Response {
    if let Err(r) = require(&state, &headers, false).await {
        return r;
    }
    let slug = f.get("slug").cloned().unwrap_or_default();
    if slug == crate::web::skin::DEFAULT_THEME {
        return to(&format!("/admin/tema/{slug}"), "galat", "Tema bawaan tidak bisa dihapus.");
    }
    match repo::delete_theme(&state.pool, &slug).await {
        Ok(true) => {
            state.refresh_themes().await;
            to("/admin/tema", "ok", &format!("Tema {slug} dihapus."))
        }
        Ok(false) => to(
            &format!("/admin/tema/{slug}"),
            "galat",
            "Tema masih dipakai undangan — sembunyikan saja dari katalog (hapus centang \"Tampil di katalog\").",
        ),
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "admin: hapus tema");
            to(&format!("/admin/tema/{slug}"), "galat", "Gagal menghapus tema.")
        }
    }
}

/// POST /admin/animasi/simpan — buat/sunting animasi (multipart: panel_file,
/// orn_file, float_file → gambar di RustFS). Animasi bawaan pun bisa disunting.
pub async fn admin_save_animation(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    mp: Multipart,
) -> Response {
    if let Err(r) = require(&state, &headers, false).await {
        return r;
    }
    let Ok(mut form) = super::form::read(mp, 3).await else {
        return to("/admin/animasi", "galat", "Unggahan terputus atau terlalu besar (gambar maks 5 MB).");
    };
    let uploads = std::mem::take(&mut form.files);
    let mut f = form.fields;
    let editing = f.get("orig").cloned().unwrap_or_default();
    let kind = f.get("kind").cloned().unwrap_or_default();
    let cat = state.themes();
    // Kode & jenis tak bisa diganti saat menyunting (tema menyimpan kuncinya).
    let old = if editing.is_empty() {
        None
    } else {
        match cat.anim(&kind, &editing) {
            Some(a) => Some(a.clone()),
            None => return to("/admin/animasi", "galat", "Animasi tidak ditemukan."),
        }
    };
    if old.is_some() {
        f.insert("slug".into(), editing.clone());
    }
    let back = match &old {
        Some(_) => format!("/admin/animasi/{kind}/{editing}"),
        None => format!("/admin/animasi/baru?jenis={}", fmt::url_encode(&kind)),
    };
    let mut anim = match crate::web::anim::from_form(|k| f.get(k).cloned().unwrap_or_default()) {
        Ok(a) => a,
        Err(msg) => return to(&back, "galat", &msg),
    };
    match &old {
        Some(o) => {
            anim.builtin = o.builtin;
            anim.sort_order = o.sort_order;
        }
        None if cat.anim(&anim.kind, &anim.slug).is_some() => {
            return to(&back, "galat", &format!("Kode \"{}\" sudah dipakai animasi lain jenis ini.", anim.slug));
        }
        None => {}
    }
    for up in uploads {
        let Some(st) = state.storage.as_ref() else {
            return to(&back, "galat", "RustFS belum dikonfigurasi — tempel URL gambar saja.");
        };
        let url = match st.upload_image_as(up.data, &format!("animasi-{}", anim.slug), &up.file_name, super::storage::Ukuran::Aset).await {
            Ok(url) => url,
            Err(e) => return to(&back, "galat", &e.to_string()),
        };
        match up.field.as_str() {
            "panel_file" => {
                anim.spec.panel_image = url;
                anim.spec.fill = "image".into();
            }
            "orn_file" => anim.spec.orn_image = url,
            "float_file" => anim.spec.float_image = url,
            _ => {}
        }
    }
    if let Err(e) = repo::upsert_animation(&state.pool, &anim).await {
        tracing::error!(error = %format!("{e:#}"), "admin: simpan animasi");
        return to(&back, "galat", "Gagal menyimpan animasi — sudah menjalankan migration/008_animasi_semua.sql?");
    }
    state.refresh_themes().await;
    tracing::info!(slug = %anim.slug, kind = %anim.kind, "admin: animasi disimpan");
    to(&format!("/admin/animasi/{}/{}", anim.kind, anim.slug), "ok", "Animasi tersimpan & langsung tayang di semua tema yang memakainya.")
}

/// POST /admin/animasi/bawaan — kembalikan animasi bawaan ke isi pabrik.
pub async fn admin_reset_animation(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::Form(f): axum::Form<HashMap<String, String>>,
) -> Response {
    if let Err(r) = require(&state, &headers, false).await {
        return r;
    }
    let kind = f.get("kind").cloned().unwrap_or_default();
    let slug = f.get("slug").cloned().unwrap_or_default();
    let back = format!("/admin/animasi/{kind}/{slug}");
    let Some(a) = crate::web::anim::builtin(&kind, &slug) else {
        return to(&back, "galat", "Bukan animasi bawaan.");
    };
    if let Err(e) = repo::upsert_animation(&state.pool, &a).await {
        tracing::error!(error = %format!("{e:#}"), "admin: reset animasi");
        return to(&back, "galat", "Gagal — sudah menjalankan migration/008_animasi_semua.sql?");
    }
    state.refresh_themes().await;
    to(&back, "ok", "Animasi dikembalikan ke bawaan.")
}

/// POST /admin/animasi/hapus — hanya animasi buatan admin yang tak dipakai tema.
pub async fn admin_delete_animation(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::Form(f): axum::Form<HashMap<String, String>>,
) -> Response {
    if let Err(r) = require(&state, &headers, false).await {
        return r;
    }
    let kind = f.get("kind").cloned().unwrap_or_default();
    let slug = f.get("slug").cloned().unwrap_or_default();
    let back = format!("/admin/animasi/{kind}/{slug}");
    if state.themes().anim(&kind, &slug).is_some_and(|a| a.builtin) {
        return to(&back, "galat", "Animasi bawaan tidak bisa dihapus — sunting atau kembalikan ke bawaan saja.");
    }
    match repo::animation_users(&state.pool, &kind, &slug).await {
        Ok(users) if !users.is_empty() => {
            return to(&back, "galat", &format!("Masih dipakai tema: {}. Ganti animasi di tema itu dulu.", users.join(", ")));
        }
        Ok(_) => {}
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "admin: cek pemakai animasi");
            return to(&back, "galat", "Gagal memeriksa pemakai animasi.");
        }
    }
    if let Err(e) = repo::delete_animation(&state.pool, &kind, &slug).await {
        tracing::error!(error = %format!("{e:#}"), "admin: hapus animasi");
        return to(&back, "galat", "Gagal menghapus animasi.");
    }
    state.refresh_themes().await;
    to("/admin/animasi", "ok", &format!("Animasi {slug} dihapus."))
}

/// POST /admin/banner/simpan — tambah/ubah banner (multipart: img_file,
/// img_hp_file → RustFS `foto/banner/{nama-berkas}.webp`).
pub async fn admin_save_banner(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    mp: Multipart,
) -> Response {
    if let Err(r) = require(&state, &headers, false).await {
        return r;
    }
    // Beranda membaca banner dari cache 30 dtk — kosongkan agar perubahan segera tampil.
    if let Ok(mut g) = state.banners.write() {
        *g = None;
    }
    let back = "/admin/banner";
    let Ok(mut form) = super::form::read(mp, 2).await else {
        return to(back, "galat", "Unggahan terputus atau terlalu besar (gambar maks 5 MB).");
    };
    let files = std::mem::take(&mut form.files);
    let get = |k: &str, max: usize| form.get(k, max);
    let id: i64 = get("id", 20).parse().unwrap_or(0);
    let mut b = crate::web::model::Banner {
        id,
        judul: get("judul", 120),
        sub: get("sub", 300),
        cta: get("cta", 40),
        link: get("link", 300),
        img: get("img", 500),
        img_hp: get("img_hp", 500),
        aktif: form.raw("aktif") == "1",
        urutan: get("urutan", 6).parse().unwrap_or(100),
        mulai: get("mulai", 16),
        selesai: get("selesai", 16),
        status: String::new(),
    };
    // Unggahan menimpa isian URL. Nama berkas terbaca; bentrok dgn banner lain → akhiran -2, -3…
    let used: Vec<String> = repo::banners_all(&state.pool)
        .await
        .map(|v| v.into_iter().filter(|x| x.id != id).flat_map(|x| [x.img, x.img_hp]).collect())
        .unwrap_or_default();
    for up in files {
        let Some(st) = state.storage.as_ref() else {
            return to(back, "galat", "RustFS belum dikonfigurasi — isi alamat gambar saja (mis. /img/banner/… atau https://…).");
        };
        let name = super::storage::StorageService::unique_name(&super::storage::file_stem(&up.file_name, "banner"), |n| {
            used.iter().any(|u| u.contains(&format!("/banner/{n}.")))
        });
        let url = match st.upload_image_as(up.data, "banner", &name, super::storage::Ukuran::Banner).await {
            Ok(u) => u,
            Err(e) => return to(back, "galat", &e.to_string()),
        };
        match up.field.as_str() {
            "img_file" => b.img = url,
            "img_hp_file" => b.img_hp = url,
            _ => {}
        }
    }
    if !crate::web::skin::is_safe_url(&b.img) {
        return to(back, "galat", "Gambar desktop wajib diisi (unggah berkas atau alamat /img/… / https://…).");
    }
    if !b.img_hp.is_empty() && !crate::web::skin::is_safe_url(&b.img_hp) {
        return to(back, "galat", "Alamat gambar HP tidak valid.");
    }
    // `{demo}` = penanda undangan demo (diisi saat tampil, repo::banners_live).
    if !b.link.is_empty() && !crate::web::skin::is_safe_url(&b.link.replace("{demo}", crate::web::themes::DEMO_SLUG)) {
        return to(back, "galat", "Tautan harus diawali / (halaman situs) atau https://.");
    }
    let dt_ok = |v: &str| v.is_empty() || (v.len() == 16 && fmt::parse_date(&v[..10]).is_some() && v.as_bytes()[10] == b'T');
    if !dt_ok(&b.mulai) || !dt_ok(&b.selesai) {
        return to(back, "galat", "Format jadwal tidak valid.");
    }
    if !b.mulai.is_empty() && !b.selesai.is_empty() && b.selesai <= b.mulai {
        return to(back, "galat", "Jadwal selesai harus setelah jadwal mulai.");
    }
    match repo::save_banner(&state.pool, &b).await {
        Ok(new_id) => {
            tracing::info!(id = new_id, "admin: banner disimpan");
            to(&format!("{back}#banner-{new_id}"), "ok", if id == 0 { "Banner ditambahkan." } else { "Banner tersimpan & langsung tayang." })
        }
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "admin: simpan banner");
            to(back, "galat", "Gagal menyimpan — sudah menjalankan migration/011_banner.sql?")
        }
    }
}

/// POST /admin/banner/urut (id, arah=naik|turun) & /admin/banner/hapus (id).
pub async fn admin_banner_action(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(aksi): axum::extract::Path<String>,
    axum::Form(f): axum::Form<HashMap<String, String>>,
) -> Response {
    if let Err(r) = require(&state, &headers, false).await {
        return r;
    }
    // Beranda membaca banner dari cache 30 dtk — kosongkan agar perubahan segera tampil.
    if let Ok(mut g) = state.banners.write() {
        *g = None;
    }
    let back = "/admin/banner";
    let id: i64 = f.get("id").and_then(|v| v.parse().ok()).unwrap_or(0);
    let res = match aksi.as_str() {
        "urut" => repo::move_banner(&state.pool, id, f.get("arah").is_some_and(|a| a == "naik")).await.map(|_| ("Urutan diperbarui.", format!("{back}#banner-{id}"))),
        "hapus" => repo::delete_banner(&state.pool, id).await.map(|_| ("Banner dihapus.", back.to_string())),
        _ => return to(back, "galat", "Aksi tidak dikenal."),
    };
    match res {
        Ok((msg, to_url)) => to(&to_url, "ok", msg),
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "admin: banner {aksi}");
            to(back, "galat", "Gagal memproses banner.")
        }
    }
}

pub async fn admin_update_invitation(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::Form(f): axum::Form<HashMap<String, String>>,
) -> Response {
    let me = match require(&state, &headers, true).await {
        Ok(u) => u,
        Err(r) => return r,
    };
    let slug = clean(f.get("slug").map(String::as_str).unwrap_or(""), 60);
    let status = f.get("status").cloned().unwrap_or_default();
    let theme = f.get("theme").cloned().unwrap_or_default();
    let back = format!("/admin/undangan?q={}", fmt::url_encode(f.get("q").map(String::as_str).unwrap_or("")));
    if !crate::web::model::INV_STATUSES.iter().any(|(k, _)| *k == status) || state.themes().get(&theme).is_none() {
        return to(&back, "galat", "Status atau tema tidak valid.");
    }
    match repo::admin_update_invitation(&state.pool, &slug, &status, &theme).await {
        Ok(Some(ub)) => {
            tracing::info!(by = %me.username, slug = %slug, status = %status, theme = %theme, "admin: undangan diperbarui");
            // Baru diaktifkan → kabari pemesan (hanya saat status BERUBAH ke aktif).
            let mut info = String::new();
            if status == "aktif" && ub.lama != "aktif" {
                match state.wa.as_ref() {
                    Some(wa) if !ub.contact_phone.is_empty() => {
                        // DB hanya menyimpan HASH kunci Kelola → tautan lama tak bisa
                        // disusun ulang. Terbitkan kunci baru & kirim di pesan yang
                        // sama (tautan lama berhenti berlaku). Gagal menyimpan kunci →
                        // pesan tetap terkirim tanpa tautan Kelola.
                        let key = random_key(40);
                        let key = match repo::reset_manage_key(&state.pool, &slug, &auth::token_hash(&key)).await {
                            Ok(true) => Some(key),
                            Ok(false) => None,
                            Err(e) => {
                                tracing::error!(error = %format!("{e:#}"), slug = %slug, "aktivasi: terbitkan kunci Kelola");
                                None
                            }
                        };
                        let text = pesan_aktif(&ub.couple, &public_origin(&state, &headers), &slug, key.as_deref());
                        wa.spawn_text(ub.contact_phone.clone(), text, "undangan aktif → pemesan");
                        info = if key.is_some() {
                            " Pemesan dikabari lewat WhatsApp beserta tautan Kelola baru (tautan lama tidak berlaku).".into()
                        } else {
                            " Pemesan dikabari lewat WhatsApp.".into()
                        };
                    }
                    _ => info = " (WA pemesan tidak dikirim — waxum mati / nomor kosong.)".into(),
                }
            }
            to(&back, "ok", &format!("/u/{slug} diperbarui.{info}"))
        }
        Ok(None) => to(&back, "galat", "Undangan tidak ditemukan (undangan demo tidak bisa diubah)."),
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "admin: ubah undangan");
            to(&back, "galat", "Gagal menyimpan perubahan.")
        }
    }
}

/// Terbitkan ulang tautan Kelola (pelanggan kehilangan tautan). Kunci lama
/// langsung tidak berlaku; tautan baru ditampilkan sekali ke admin.
pub async fn admin_reset_key(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::Form(f): axum::Form<HashMap<String, String>>,
) -> Response {
    let me = match require(&state, &headers, true).await {
        Ok(u) => u,
        Err(r) => return r,
    };
    let slug = clean(f.get("slug").map(String::as_str).unwrap_or(""), 60);
    let back = format!("/admin/undangan?q={}", fmt::url_encode(&slug));
    let key = random_key(40);
    match repo::reset_manage_key(&state.pool, &slug, &auth::token_hash(&key)).await {
        Ok(true) => {
            tracing::info!(by = %me.username, slug = %slug, "admin: kunci Kelola diterbitkan ulang");
            let link = format!("/kelola/{slug}?key={key}");
            // Langsung ke WA pemesan (nomor saat memesan); admin tetap melihat tautannya.
            let phone = match repo::invitation(&state.pool, &slug).await {
                Ok(Some(r)) => Some((r.contact_phone, r.inv.couple())),
                _ => None,
            };
            let ok = match (state.wa.as_ref(), phone) {
                (Some(wa), Some((p, couple))) if !p.is_empty() => {
                    let text = format!(
                        "Halo! Tautan dashboard Kelola undangan *{couple}* telah diperbarui oleh admin {brand}:\n\n{origin}{link}\n\nSimpan pesan ini — tautan lama sudah tidak berlaku. Jangan bagikan tautan ini ke orang lain.",
                        brand = crate::brand!(),
                        origin = public_origin(&state, &headers),
                    );
                    wa.spawn_text(p, text, "tautan Kelola baru → pemesan");
                    "Tautan Kelola baru diterbitkan & dikirim ke WhatsApp pemesan — tautan lama tidak berlaku."
                }
                _ => "Tautan Kelola baru diterbitkan — kirim ke pemesan (WA otomatis tidak terkirim), tautan lama tidak berlaku.",
            };
            Redirect::to(&format!("{back}&ok={}&kelola={}", fmt::url_encode(ok), fmt::url_encode(&link)))
                .into_response()
        }
        Ok(false) => to(&back, "galat", "Undangan tidak ditemukan (undangan demo tidak bisa diubah)."),
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "admin: reset kunci");
            to(&back, "galat", "Gagal menerbitkan tautan — sudah menjalankan migration/004_keamanan.sql?")
        }
    }
}

// ── Ornamen tema (/admin/tema/{slug}/ornamen) ─────────────────────────────

/// POST /admin/ornamen/simpan (multipart): satu ornamen, id 0 = baru.
pub async fn admin_save_ornament(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    mp: Multipart,
) -> Response {
    if let Err(r) = require(&state, &headers, false).await {
        return r;
    }
    let Ok(mut form) = super::form::read(mp, 1).await else {
        return to("/admin/tema", "galat", "Unggahan terputus atau terlalu besar (gambar maks 5 MB).");
    };
    let file = form.take_file("img_file");
    let mut f = form.fields;
    let theme = clean(f.get("theme").map(String::as_str).unwrap_or(""), 60);
    if state.themes().get(&theme).is_none() {
        return to("/admin/tema", "galat", "Tema tidak ditemukan.");
    }
    let back = format!("/admin/tema/{theme}/ornamen");
    if let Some(up) = file {
        let Some(st) = state.storage.as_ref() else {
            return to(&back, "galat", "RustFS belum dikonfigurasi — isi alamat gambar saja (mis. /img/tema/ornamen/… atau https://…).");
        };
        let used: Vec<String> = state.themes().get(&theme).map(|t| t.ornaments.iter().map(|o| o.img.clone()).collect()).unwrap_or_default();
        let name = super::storage::StorageService::unique_name(&super::storage::file_stem(&up.file_name, "ornamen"), |n| {
            used.iter().any(|u| u.contains(&format!("/{n}.")))
        });
        match st.upload_image_as(up.data, &format!("ornamen-{theme}"), &name, super::storage::Ukuran::Aset).await {
            Ok(u) => {
                f.insert("img".into(), u);
            }
            Err(e) => return to(&back, "galat", &e.to_string()),
        }
    }
    let get = |k: &str| clean(f.get(k).map(String::as_str).unwrap_or(""), 500);
    let mut o = match crate::web::ornamen::from_form(get) {
        Ok(o) => o,
        Err(m) => return to(&back, "galat", &m),
    };
    o.theme = theme.clone();
    match repo::save_ornament(&state.pool, &o).await {
        Ok(id) => {
            state.refresh_themes().await;
            tracing::info!(theme = %theme, id, "admin: ornamen disimpan");
            to(&format!("{back}#orn-{id}"), "ok", if o.id == 0 { "Ornamen ditambahkan." } else { "Ornamen tersimpan & langsung tampil di undangan." })
        }
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "admin: simpan ornamen");
            to(&back, "galat", "Gagal menyimpan — sudah menjalankan migration/012_ornamen.sql?")
        }
    }
}

/// POST /admin/ornamen/hapus (theme, id) & /admin/ornamen/salin (theme, dari).
pub async fn admin_ornament_action(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(aksi): axum::extract::Path<String>,
    axum::Form(f): axum::Form<HashMap<String, String>>,
) -> Response {
    if let Err(r) = require(&state, &headers, false).await {
        return r;
    }
    let cat = state.themes();
    let theme = clean(f.get("theme").map(String::as_str).unwrap_or(""), 60);
    if cat.get(&theme).is_none() {
        return to("/admin/tema", "galat", "Tema tidak ditemukan.");
    }
    let back = format!("/admin/tema/{theme}/ornamen");
    let res = match aksi.as_str() {
        "hapus" => {
            let id: i64 = f.get("id").and_then(|v| v.parse().ok()).unwrap_or(0);
            repo::delete_ornament(&state.pool, &theme, id).await.map(|_| "Ornamen dihapus.".to_string())
        }
        "salin" => {
            let from = clean(f.get("dari").map(String::as_str).unwrap_or(""), 60);
            if from == theme || cat.get(&from).is_none() {
                return to(&back, "galat", "Pilih tema sumber yang lain.");
            }
            repo::copy_ornaments(&state.pool, &from, &theme).await.map(|n| format!("{n} ornamen disalin."))
        }
        _ => return to(&back, "galat", "Aksi tidak dikenal."),
    };
    match res {
        Ok(msg) => {
            state.refresh_themes().await;
            to(&back, "ok", &msg)
        }
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "admin: ornamen {aksi}");
            to(&back, "galat", "Gagal memproses ornamen.")
        }
    }
}

// ── Bukti transfer (/kelola/{slug}/bukti) ─────────────────────────────────

/// Alamat publik situs untuk tautan di pesan WA: SITE_URL, atau dari header
/// proxy (X-Forwarded-Proto/Host) bila kosong.
fn public_origin(state: &AppState, headers: &axum::http::HeaderMap) -> String {
    if !state.site_url.is_empty() {
        return state.site_url.clone();
    }
    let h = |k: &str| headers.get(k).and_then(|v| v.to_str().ok()).map(|v| v.split(',').next().unwrap_or(v).trim().to_string());
    let host = h("x-forwarded-host").or_else(|| h("host")).unwrap_or_else(|| "localhost".into());
    format!("{}://{host}", h("x-forwarded-proto").unwrap_or_else(|| "http".into()))
}

/// WA ke pemesan setelah pesanan dibuat: tautan Kelola + cara bayar.
fn pesan_pesanan_baru(couple: &str, origin: &str, slug: &str, manage_key: &str, total: i64, pay: &crate::web::konten::Pembayaran, ttl_jam: i64) -> String {
    let bayar = if pay.nomor.is_empty() {
        String::new()
    } else {
        format!("\nTransfer *{}* ke {} {} a/n {}, lalu unggah tangkapan layarnya di dashboard Kelola.\n", fmt::rupiah(total), pay.metode, pay.nomor, pay.atas_nama)
    };
    format!(
        concat!(
            "Terima kasih telah memesan di ", crate::brand!(), "! 💌\n\n",
            "Undangan: *{couple}*\n",
            "Total: *{total}*\n{bayar}\n",
            "Dashboard Kelola (simpan pesan ini, JANGAN dibagikan):\n{origin}/kelola/{slug}?key={key}\n\n",
            "Pesanan yang belum dibayar/dikirimi bukti dalam {ttl} jam akan dihapus otomatis."
        ),
        couple = couple,
        total = fmt::rupiah(total),
        bayar = bayar,
        origin = origin,
        slug = slug,
        key = manage_key,
        ttl = ttl_jam,
    )
}

/// WA ke pemesan saat admin mengaktifkan undangan.
/// `key` = kunci Kelola BARU (diterbitkan saat aktivasi); None = tanpa tautan Kelola.
fn pesan_aktif(couple: &str, origin: &str, slug: &str, key: Option<&str>) -> String {
    let kelola = match key {
        Some(k) => format!(
            "Atur daftar tamu, tautan pribadi tiap tamu, RSVP & story di dashboard Kelola (simpan pesan ini, JANGAN dibagikan):\n{origin}/kelola/{slug}?key={k}\n_Tautan Kelola sebelumnya sudah tidak berlaku._\n\n"
        ),
        None => "Atur daftar tamu & tautan pribadi tiap tamu di dashboard Kelola (tautan dari pesan sebelumnya).\n\n".to_string(),
    };
    format!(
        concat!(
            "Kabar baik! 🎉 Pembayaran sudah kami terima dan undangan *{couple}* kini AKTIF.\n\n",
            "Bagikan ke tamu: {origin}/u/{slug}\n\n",
            "{kelola}",
            "Terima kasih — ", crate::brand!()
        ),
        couple = couple,
        origin = origin,
        slug = slug,
        kelola = kelola,
    )
}

/// Keterangan pesan WA admin untuk bukti transfer baru.
fn proof_caption(row: &repo::InvRow, package_name: &str, metode: &str, origin: &str) -> String {
    let wa = fmt::wa_number(&row.contact_phone);
    format!(
        concat!(
            "🧾 *Bukti pembayaran baru* — ", crate::brand!(), "\n\n",
            "Undangan: *{couple}*\n",
            "Paket: {paket} — *{total}*\n",
            "Metode: {metode}\n",
            "WA pemesan: {wa}\n\n",
            "Lihat undangan: {origin}/u/{slug}\n",
            "Cek & aktifkan: {origin}/admin/undangan?q={slug}"
        ),
        couple = row.inv.couple(),
        paket = package_name,
        total = fmt::rupiah(row.total_price),
        metode = metode,
        wa = if wa.is_empty() { "-".to_string() } else { format!("+{wa}") },
        origin = origin,
        slug = row.inv.slug,
    )
}

/// POST multipart dari dashboard Kelola: gambar bukti transfer → RustFS
/// (foto/{slug}/bukti-transfer-….webp) + kolom payment_proof, lalu WA ke
/// admin lewat waxum (gambar + tautan). Pemilik dikenali dari cookie kunci
/// Kelola (server/owner.rs) atau input `key`.
pub async fn upload_payment_proof(
    Extension(state): Extension<Arc<AppState>>,
    Path(slug): Path<String>,
    headers: axum::http::HeaderMap,
    mp: Multipart,
) -> Response {
    let back = format!("/kelola/{}#bayar", fmt::key(&slug));
    if let Err(secs) = state.write_limit.hit(&format!("bukti:{}:{slug}", security::client_ip(&headers))) {
        return to(&back, "galat", &format!("Terlalu banyak unggahan. Coba lagi dalam {} menit.", secs.div_ceil(60)));
    }
    let Ok(mut form) = super::form::read(mp, 1).await else {
        return to(&back, "galat", "Unggahan terputus atau terlalu besar (gambar maks 5 MB).");
    };
    let row = match repo::invitation(&state.pool, &slug).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, "Undangan tidak ditemukan").into_response(),
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "bukti: muat undangan");
            return to(&back, "galat", "Server sedang sibuk, coba lagi sebentar.");
        }
    };
    let key = Some(form.raw("key")).filter(|k| !k.trim().is_empty()).or_else(|| super::owner::key_from(&headers, &slug)).unwrap_or_default();
    if !auth::same_hash(&auth::token_hash(key.trim()), &row.manage_key_hash) {
        return to(&back, "galat", "Kunci kelola tidak valid. Buka dari tautan yang Anda terima saat memesan.");
    }
    if row.inv.is_demo {
        return to(&back, "galat", "Ini dashboard demo — bukti transfer tidak dikirim.");
    }
    if !row.inv.is_locked() {
        return to(&back, "ok", "Undangan sudah aktif — tidak perlu mengirim bukti lagi.");
    }
    let Some(up) = form.take_file("bukti") else {
        return to(&back, "galat", "Pilih gambar bukti transfer (tangkapan layar) dulu.");
    };
    if up.data.len() > super::storage::MAX_IMAGE {
        return to(&back, "galat", "Gambar bukti maksimal 5 MB.");
    }
    let Some((mime, _)) = super::storage::detect_image(&up.data) else {
        return to(&back, "galat", "Bukti harus berupa gambar JPEG/PNG/WebP (tangkapan layar).");
    };
    if state.storage.is_none() && state.wa.is_none() {
        tracing::error!(slug = %slug, "bukti: RustFS & waxum sama-sama tak dikonfigurasi");
        return to(&back, "galat", "Unggah bukti belum tersedia — kirim bukti lewat WhatsApp admin.");
    }

    // 1. Simpan gambar (bila RustFS ada). Isi asli disimpan untuk WA.
    //    Akhiran acak (bukan jam:menit:detik): URL tak bisa ditebak dari slug
    //    dan dua unggahan di detik yang sama tak saling timpa.
    let original = up.data.clone();
    let url = match state.storage.as_ref() {
        Some(st) => {
            let name = format!("bukti-transfer-{}", auth::random_hex(8));
            match st.upload_image_as(up.data, &row.inv.slug, &name, super::storage::Ukuran::Foto).await {
                Ok(u) => u,
                Err(e) => return to(&back, "galat", &e.to_string()),
            }
        }
        None => String::new(),
    };
    // 2. Catat di DB (hanya bila masih menunggu pembayaran).
    match repo::set_payment_proof(&state.pool, row.id, &url).await {
        Ok(Some(old)) => {
            if !old.is_empty() && old != url {
                if let Some(st) = state.storage.as_ref() {
                    let _ = st.delete_url(&old).await;
                }
            }
        }
        Ok(None) => {
            discard_uploads(&state, std::slice::from_ref(&url)).await;
            return to(&back, "ok", "Undangan sudah aktif — tidak perlu mengirim bukti lagi.");
        }
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "bukti: simpan (sudah menjalankan migration/017_bukti_bayar.sql?)");
            discard_uploads(&state, std::slice::from_ref(&url)).await;
            return to(&back, "galat", "Gagal menyimpan bukti, coba lagi.");
        }
    }
    tracing::info!(slug = %row.inv.slug, total = row.total_price, "bukti transfer diterima");

    // 3. WA ke admin — di latar (dicoba ulang bila waxum sedang menyambung):
    //    pemesan tak menunggu. Gagal = dicatat; bukti tetap di /admin/undangan.
    if let (Some(wa), false) = (state.wa.clone(), state.notify_wa.is_empty()) {
        let konten = state.konten();
        let caption = proof_caption(&row, &konten.package_name(&row.inv.package), &konten.pembayaran.metode, &public_origin(&state, &headers));
        let to = state.notify_wa.clone();
        let mime = mime.to_string();
        let slug = row.inv.slug.clone();
        tokio::spawn(async move {
            let img = super::wa::Gambar { data: &original, mime: &mime };
            match wa.send_image_or_text(&to, img, &caption, &url, super::wa::Gigih::Latar).await {
                Ok(()) => tracing::info!(slug = %slug, "bukti: WA admin terkirim"),
                Err(e) => tracing::error!(slug = %slug, error = %format!("{e:#}"), "bukti: WA admin GAGAL"),
            }
        });
    }
    to(&back, "ok", "Bukti transfer terkirim! Admin akan memeriksa lalu mengaktifkan undangan Anda.")
}

#[cfg(test)]
mod bukti_tests {
    use super::*;

    #[test]
    fn pesan_aktif_memuat_tautan_kelola_baru() {
        let t = pesan_aktif("Ani & Budi", "https://ilyvowcraft.online", "ani-budi-x1", Some("KUNCI123"));
        assert!(t.contains("https://ilyvowcraft.online/u/ani-budi-x1") && t.contains("/kelola/ani-budi-x1?key=KUNCI123"), "{t}");
        assert!(!pesan_aktif("A & B", "https://x", "a-b", None).contains("?key="));
    }

    #[test]
    fn keterangan_wa_memuat_tautan_dan_total() {
        let row = repo::InvRow {
            id: 1,
            inv: crate::web::model::Invitation {
                slug: "ani-budi-k7f3x9m2".into(),
                bride_name: "Ani Lestari".into(),
                groom_name: "Budi Santoso".into(),
                ..Default::default()
            },
            manage_key_hash: String::new(),
            total_price: 149_000,
            payment_method: "shopeepay".into(),
            contact_phone: "0812-3456-7890".into(),
        };
        let c = proof_caption(&row, "Gold", "ShopeePay", "https://ilyvowcraft.online");
        assert!(c.contains("https://ilyvowcraft.online/u/ani-budi-k7f3x9m2"), "{c}");
        assert!(c.contains("https://ilyvowcraft.online/admin/undangan?q=ani-budi-k7f3x9m2"), "{c}");
        assert!(c.contains("+6281234567890") && c.contains("ShopeePay") && c.contains("Gold"), "{c}");
        assert!(c.contains("149"), "{c}");
    }
}

/// POST multipart dari tab Story: kunci spesial (dari WA) + SATU foto →
/// RustFS foto/{slug}/story-….webp + baris invitation_stories. Hanya gambar
/// (JPEG/PNG/WebP, dicek dari isi berkas) — video ditolak. Satu nomor = satu
/// story per undangan; kunci baru ditandai terpakai setelah story tersimpan.
pub async fn post_story(
    Extension(state): Extension<Arc<AppState>>,
    Path(slug): Path<String>,
    headers: axum::http::HeaderMap,
    mp: Multipart,
) -> Response {
    let base = format!("/u/{}/story", fmt::key(&slug));
    if let Err(secs) = state.write_limit.hit(&format!("story:{}:{slug}", security::client_ip(&headers))) {
        return to(&base, "galat", &format!("Terlalu banyak unggahan. Coba lagi dalam {} menit.", secs.div_ceil(60)));
    }
    let Ok(mut form) = super::form::read(mp, 1).await else {
        return to(&base, "galat", "Unggahan terputus atau terlalu besar (foto maks 5 MB).");
    };
    // Kembali ke tab Story dengan query tamu yang sama (?to= / ?g=), bukan URL lain.
    let back = Some(form.raw("back")).filter(|b| b.starts_with(&base) && !b.contains(['\r', '\n', '#'])).unwrap_or(base.clone());
    let row = match repo::invitation(&state.pool, &slug).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, "Undangan tidak ditemukan").into_response(),
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "story: muat undangan");
            return to(&back, "galat", "Server sedang sibuk, coba lagi sebentar.");
        }
    };
    if row.inv.is_demo {
        return to(&back, "galat", "Ini undangan demo — story contoh saja.");
    }
    if row.inv.is_locked() {
        return to(&back, "galat", "Undangan ini belum diaktifkan.");
    }
    let phone = fmt::wa_number(&form.raw("phone"));
    let key: String = form.raw("key").chars().filter(|c| c.is_ascii_digit()).collect();
    if phone.is_empty() || key.len() != 6 {
        return to(&back, "galat", "Masukkan kunci 6 digit yang dikirim ke WhatsApp Anda.");
    }
    let (key_id, name) = match repo::check_story_key(&state.pool, row.id, &phone, &auth::token_hash(&key)).await {
        Ok(Some(k)) => k,
        Ok(None) => return to(&back, "galat", "Kunci salah atau sudah kedaluwarsa. Minta kunci baru bila perlu."),
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "story: cek kunci (sudah menjalankan migration/026_story.sql?)");
            return to(&back, "galat", "Server sedang sibuk, coba lagi sebentar.");
        }
    };
    let Some(up) = form.take_file("foto") else {
        return to(&back, "galat", "Pilih satu foto untuk story Anda.");
    };
    if up.data.len() > super::storage::MAX_IMAGE {
        return to(&back, "galat", "Foto maksimal 5 MB.");
    }
    if super::storage::detect_image(&up.data).is_none() {
        return to(&back, "galat", "Story hanya boleh FOTO (JPEG/PNG/WebP) — video tidak didukung.");
    }
    let Some(st) = state.storage.as_ref() else {
        tracing::error!(slug = %slug, "story: RustFS belum dikonfigurasi");
        return to(&back, "galat", "Unggah foto belum tersedia di server ini.");
    };
    let filter = form.raw("filter");
    let filter = if crate::web::model::STORY_FILTERS.iter().any(|(k, _)| *k == filter) { filter } else { "normal".to_string() };
    let caption = form.get("caption", 150);
    // Token pembuat: perangkat ini boleh menghapus story-nya sendiri. Satu
    // token per undangan per perangkat (dipakai ulang bila sudah ada).
    let token = story_token(&headers, &slug).unwrap_or_else(super::auth::new_token);
    let tail: String = phone.chars().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect();
    let obj = format!("story-{tail}-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S"));
    let url = match st.upload_image_as(up.data, &row.inv.slug, &obj, super::storage::Ukuran::Foto).await {
        Ok(u) => u,
        Err(e) => return to(&back, "galat", &e.to_string()),
    };
    match repo::insert_story(&state.pool, row.id, &phone, &name, &url, &filter, &caption, &super::auth::token_hash(&token)).await {
        Ok(true) => {}
        Ok(false) => {
            discard_uploads(&state, std::slice::from_ref(&url)).await;
            return to(&back, "galat", "Nomor ini sudah membuat story di undangan ini (1 nomor = 1 story).");
        }
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "story: simpan");
            discard_uploads(&state, std::slice::from_ref(&url)).await;
            return to(&back, "galat", "Gagal menyimpan story, coba lagi.");
        }
    }
    if let Err(e) = repo::use_story_key(&state.pool, key_id).await {
        tracing::warn!(error = %format!("{e:#}"), "story: tandai kunci terpakai");
    }
    tracing::info!(slug = %row.inv.slug, "story tamu baru");
    with_cookie(to(&back, "ok", "Story Anda sudah tayang! Terima kasih telah berbagi momen."), story_cookie(&slug, &token, &headers))
}

/// Cookie HttpOnly `ily_s_{slug}`: token pembuat story di perangkat ini.
fn story_cookie_name(slug: &str) -> String {
    format!("ily_s_{slug}")
}

/// Token pembuat story dari cookie request ini (belum diverifikasi).
pub fn story_token(headers: &axum::http::HeaderMap, slug: &str) -> Option<String> {
    let ok = !slug.is_empty() && slug.len() <= 80 && slug.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    ok.then(|| cookie_value(headers, &story_cookie_name(slug)))
        .flatten()
        .filter(|v| (16..=128).contains(&v.len()) && v.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'))
        .map(str::to_string)
}

fn story_cookie(slug: &str, token: &str, headers: &axum::http::HeaderMap) -> String {
    let https = headers.get("x-forwarded-proto").and_then(|v| v.to_str().ok()) == Some("https");
    format!("{}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}{}", story_cookie_name(slug), 400 * 86_400, if https { "; Secure" } else { "" })
}

/// Hapus story PERMANEN (baris + foto RustFS) setelah pemiliknya terverifikasi.
async fn purge_story_file(state: &AppState, url: &str) {
    if let Some(st) = state.storage.as_ref() {
        if let Err(e) = st.delete_url(url).await {
            tracing::warn!(error = %format!("{e:#}"), "story: hapus foto RustFS");
        }
    }
}

/// POST /u/{slug}/story/hapus-saya (id) — pembuat menghapus dari perangkatnya
/// sendiri (cookie token pembuat harus cocok dengan story itu).
pub async fn delete_my_story(
    Extension(state): Extension<Arc<AppState>>,
    Path(slug): Path<String>,
    headers: axum::http::HeaderMap,
    axum::Form(f): axum::Form<HashMap<String, String>>,
) -> Response {
    let base = format!("/u/{}/story", fmt::key(&slug));
    let back = f.get("back").filter(|b| b.starts_with(&base) && !b.contains(['\r', '\n', '#'])).cloned().unwrap_or(base);
    let id: i64 = f.get("id").and_then(|v| v.parse().ok()).unwrap_or(0);
    let Some(token) = story_token(&headers, &slug) else {
        return to(&back, "galat", "Story ini bukan dibuat dari perangkat ini. Hapus lewat kunci WhatsApp di formulir Tambah Story.");
    };
    let row = match repo::invitation(&state.pool, &slug).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, "Undangan tidak ditemukan").into_response(),
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "story: muat undangan");
            return to(&back, "galat", "Server sedang sibuk, coba lagi sebentar.");
        }
    };
    match repo::delete_story_by_token(&state.pool, row.id, id, &auth::token_hash(&token)).await {
        Ok(Some(url)) => {
            purge_story_file(&state, &url).await;
            tracing::info!(slug = %slug, id, "story dihapus pembuatnya");
            to(&back, "ok", "Story Anda sudah dihapus permanen.")
        }
        Ok(None) => to(&back, "galat", "Story tidak ditemukan atau bukan milik Anda."),
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "story: hapus (pembuat)");
            to(&back, "galat", "Gagal menghapus story, coba lagi.")
        }
    }
}

/// POST /u/{slug}/story/hapus (phone, key) — pembuat menghapus dari perangkat
/// lain: kunci 6 digit dari WhatsApp ke nomor pembuat (repo::check_story_key).
pub async fn delete_story_with_key(
    Extension(state): Extension<Arc<AppState>>,
    Path(slug): Path<String>,
    headers: axum::http::HeaderMap,
    axum::Form(f): axum::Form<HashMap<String, String>>,
) -> Response {
    let base = format!("/u/{}/story", fmt::key(&slug));
    let back = f.get("back").filter(|b| b.starts_with(&base) && !b.contains(['\r', '\n', '#'])).cloned().unwrap_or(base);
    if let Err(secs) = state.write_limit.hit(&format!("storyhapus:{}:{slug}", security::client_ip(&headers))) {
        return to(&back, "galat", &format!("Terlalu banyak percobaan. Coba lagi dalam {} menit.", secs.div_ceil(60)));
    }
    let phone = fmt::wa_number(f.get("phone").map(String::as_str).unwrap_or(""));
    let key: String = f.get("key").map(String::as_str).unwrap_or("").chars().filter(|c| c.is_ascii_digit()).collect();
    if phone.is_empty() || key.len() != 6 {
        return to(&back, "galat", "Masukkan kunci 6 digit yang dikirim ke WhatsApp Anda.");
    }
    let row = match repo::invitation(&state.pool, &slug).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, "Undangan tidak ditemukan").into_response(),
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "story: muat undangan");
            return to(&back, "galat", "Server sedang sibuk, coba lagi sebentar.");
        }
    };
    let key_id = match repo::check_story_key(&state.pool, row.id, &phone, &auth::token_hash(&key)).await {
        Ok(Some((id, _))) => id,
        Ok(None) => return to(&back, "galat", "Kunci salah atau sudah kedaluwarsa. Minta kunci baru bila perlu."),
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "story: cek kunci (hapus)");
            return to(&back, "galat", "Server sedang sibuk, coba lagi sebentar.");
        }
    };
    match repo::delete_story_by_phone(&state.pool, row.id, &phone).await {
        Ok(Some(url)) => {
            purge_story_file(&state, &url).await;
            let _ = repo::use_story_key(&state.pool, key_id).await;
            tracing::info!(slug = %slug, "story dihapus pembuatnya (kunci WA)");
            to(&back, "ok", "Story Anda sudah dihapus permanen. Anda bisa membuat story baru.")
        }
        Ok(None) => to(&back, "galat", "Nomor ini tidak punya story di undangan ini."),
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "story: hapus (kunci)");
            to(&back, "galat", "Gagal menghapus story, coba lagi.")
        }
    }
}

/// POST multipart /admin/lagu/simpan — tambah/sunting lagu pustaka. Berkas
/// (MP3/M4A/OGG ≤ 6 MB) → RustFS musik/pustaka/{judul}.{ext}; atau isi URL.
pub async fn admin_save_song(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    mp: Multipart,
) -> Response {
    if let Err(r) = require(&state, &headers, false).await {
        return r;
    }
    let back = "/admin/lagu";
    let Ok(mut form) = super::form::read(mp, 1).await else {
        return to(back, "galat", "Unggahan terputus atau terlalu besar (lagu maks 6 MB).");
    };
    let mut s = crate::web::model::Song {
        id: form.get("id", 20).parse().unwrap_or(0),
        title: form.get("title", 120),
        artist: form.get("artist", 120),
        duration: form.get("duration", 8),
        tag: form.get("tag", 30),
        url: form.get("url", 500),
        aktif: form.raw("aktif") == "1",
        urutan: 0,
    };
    if s.title.is_empty() {
        return to(back, "galat", "Judul lagu wajib diisi.");
    }
    if let Some(up) = form.take_file("file") {
        let Some(st) = state.storage.as_ref() else {
            return to(back, "galat", "RustFS belum dikonfigurasi — isi alamat lagu (https://…) saja.");
        };
        let name = if up.file_name.trim().is_empty() { s.title.clone() } else { up.file_name.clone() };
        match st.upload_audio_as(up.data, "pustaka", &name).await {
            Ok(u) => s.url = u,
            Err(e) => return to(back, "galat", &e.to_string()),
        }
    }
    if s.url.is_empty() || !crate::web::skin::is_safe_url(&s.url) {
        return to(back, "galat", "Unggah berkas lagu atau isi alamat /music/… / https://….");
    }
    match repo::save_song(&state.pool, &s).await {
        Ok(id) => {
            tracing::info!(id, "admin: lagu disimpan");
            to(&format!("{back}#lagu-{id}"), "ok", if s.id == 0 { "Lagu ditambahkan ke pustaka." } else { "Lagu tersimpan." })
        }
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "admin: simpan lagu");
            to(back, "galat", "Gagal menyimpan — sudah menjalankan migration/027_pustaka_lagu.sql?")
        }
    }
}

/// POST /admin/lagu/urut (id, arah=naik|turun) & /admin/lagu/hapus (id).
/// Berkas RustFS hanya dihapus bila tak ada undangan yang memakai lagu itu.
pub async fn admin_song_action(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(aksi): axum::extract::Path<String>,
    axum::Form(f): axum::Form<HashMap<String, String>>,
) -> Response {
    if let Err(r) = require(&state, &headers, false).await {
        return r;
    }
    let back = "/admin/lagu";
    let id: i64 = f.get("id").and_then(|v| v.parse().ok()).unwrap_or(0);
    match aksi.as_str() {
        "urut" => match repo::move_song(&state.pool, id, f.get("arah").is_some_and(|a| a == "naik")).await {
            Ok(()) => to(&format!("{back}#lagu-{id}"), "ok", "Urutan diperbarui."),
            Err(e) => {
                tracing::error!(error = %format!("{e:#}"), "admin: urut lagu");
                to(back, "galat", "Gagal mengubah urutan.")
            }
        },
        "hapus" => match repo::delete_song(&state.pool, id).await {
            Ok(Some((url, used))) => {
                if !used {
                    if let Some(st) = state.storage.as_ref() {
                        let _ = st.delete_url(&url).await;
                    }
                }
                to(back, "ok", if used { "Lagu dihapus dari pustaka (berkasnya tetap — masih dipakai undangan)." } else { "Lagu dihapus." })
            }
            Ok(None) => to(back, "galat", "Lagu tidak ditemukan."),
            Err(e) => {
                tracing::error!(error = %format!("{e:#}"), "admin: hapus lagu");
                to(back, "galat", "Gagal menghapus lagu.")
            }
        },
        _ => to(back, "galat", "Aksi tidak dikenal."),
    }
}

/// POST /kelola/{slug}/story/hapus (id[, key]) — PENGELOLA undangan (pembeli)
/// menghapus story tamu mana pun dari penampil story di Kelola. Kunci Kelola
/// dari cookie `ily_k_{slug}` (server/owner.rs) atau input `key`.
pub async fn owner_delete_story(
    Extension(state): Extension<Arc<AppState>>,
    Path(slug): Path<String>,
    headers: axum::http::HeaderMap,
    axum::Form(f): axum::Form<HashMap<String, String>>,
) -> Response {
    let back = format!("/kelola/{}#story", fmt::key(&slug));
    let id: i64 = f.get("id").and_then(|v| v.parse().ok()).unwrap_or(0);
    let row = match repo::invitation(&state.pool, &slug).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, "Undangan tidak ditemukan").into_response(),
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "kelola: muat undangan (hapus story)");
            return to(&back, "story_galat", "Server sedang sibuk, coba lagi sebentar.");
        }
    };
    let key = f.get("key").cloned().filter(|k| !k.trim().is_empty()).or_else(|| super::owner::key_from(&headers, &slug)).unwrap_or_default();
    if !auth::same_hash(&auth::token_hash(key.trim()), &row.manage_key_hash) {
        return to(&back, "story_galat", "Kunci kelola tidak valid. Buka Kelola dari tautan yang Anda terima saat memesan.");
    }
    if row.inv.is_demo {
        return to(&back, "story_galat", "Dashboard demo — story contoh tidak bisa dihapus.");
    }
    match repo::delete_story(&state.pool, row.id, id).await {
        Ok(Some(url)) => {
            purge_story_file(&state, &url).await;
            tracing::info!(slug = %slug, id, "story dihapus pengelola");
            to(&back, "story_ok", "Story dihapus permanen.")
        }
        Ok(None) => to(&back, "story_galat", "Story tidak ditemukan (mungkin sudah dihapus)."),
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "kelola: hapus story");
            to(&back, "story_galat", "Gagal menghapus story, coba lagi.")
        }
    }
}

// ── Tema templat (/admin/templat, migrasi 029) ─────────────────────────────

fn parse_assets(raw: &str) -> Result<std::collections::BTreeMap<String, String>, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(Default::default());
    }
    let m: std::collections::BTreeMap<String, String> =
        serde_json::from_str(raw).map_err(|e| format!("Aset harus JSON {{\"kunci\": \"/img/…\"}}: {e}"))?;
    for (k, v) in &m {
        if k.is_empty() || k.len() > 40 || !k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(format!("Kunci aset \"{k}\" hanya boleh huruf/angka/garis bawah."));
        }
        if !super::templat::check_asset(v) {
            return Err(format!("Aset {k}: alamat harus /lokal atau https://…"));
        }
    }
    Ok(m)
}

/// POST /admin/templat/simpan — buat / sunting templat (HTML, CSS, font, aset).
/// Khusus peran Admin: HTML/CSS ini menjadi halaman undangan yang dibuka tamu.
pub async fn admin_save_templat(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::Form(f): axum::Form<HashMap<String, String>>,
) -> Response {
    if let Err(r) = require(&state, &headers, true).await {
        return r;
    }
    let get = |k: &str| f.get(k).map(|v| v.replace("\r\n", "\n")).unwrap_or_default();
    let slug = fmt::key(&get("slug"));
    let back = format!("/admin/templat#tpl-{slug}");
    if slug.is_empty() || slug.len() > 40 {
        return to("/admin/templat", "galat", "Kode templat wajib diisi (huruf kecil, angka, tanda minus).");
    }
    let t = super::templat::Templat {
        slug: slug.clone(),
        name: fmt::clean(&get("name"), 80),
        html: get("html"),
        css: get("css"),
        fonts: get("fonts").trim().to_string(),
        assets: match parse_assets(&get("assets")) {
            Ok(m) => m,
            Err(e) => return to(&back, "galat", &e),
        },
        ..Default::default()
    };
    if t.name.is_empty() {
        return to(&back, "galat", "Nama templat wajib diisi.");
    }
    if let Err(e) = super::templat::check_html(&t.html).and_then(|_| super::templat::check_css(&t.css)) {
        return to(&back, "galat", &e);
    }
    if !super::templat::check_fonts(&t.fonts) {
        return to(&back, "galat", "Font: isi nilai family= Google Fonts, mis. Pinyon+Script&family=Cormorant+Infant:wght@400;600");
    }
    if let Err(e) = repo::save_template(&state.pool, &t).await {
        tracing::error!(error = %format!("{e:#}"), "admin: simpan templat");
        return to(&back, "galat", "Gagal menyimpan — sudah menjalankan migration/029_tema_templat.sql?");
    }
    state.reload_templat(false).await;
    to(&back, "ok", "Templat tersimpan.")
}

/// POST /admin/templat/{bawaan|pasang}
pub async fn admin_templat_action(
    Extension(state): Extension<Arc<AppState>>,
    Path(aksi): Path<String>,
    headers: axum::http::HeaderMap,
    axum::Form(f): axum::Form<HashMap<String, String>>,
) -> Response {
    if let Err(r) = require(&state, &headers, true).await {
        return r;
    }
    let get = |k: &str| f.get(k).map(|v| v.replace("\r\n", "\n")).unwrap_or_default();
    match aksi.as_str() {
        "bawaan" => {
            let slug = get("slug");
            let back = format!("/admin/templat#tpl-{slug}");
            match repo::reset_template(&state.pool, &slug).await {
                Ok(1) => {
                    state.reload_templat(true).await;
                    to(&back, "ok", "Templat dikembalikan ke isi bawaan.")
                }
                Ok(_) => to(&back, "galat", "Bukan templat bawaan."),
                Err(e) => {
                    tracing::error!(error = %format!("{e:#}"), "admin: reset templat");
                    to(&back, "galat", "Gagal — sudah menjalankan migration/029_tema_templat.sql?")
                }
            }
        }
        "pasang" => {
            let theme = get("theme");
            let template = get("template");
            let back = format!("/admin/templat#tema-{theme}");
            if state.themes().get(&theme).is_none() {
                return to("/admin/templat", "galat", "Tema tidak ditemukan.");
            }
            if !template.is_empty() && state.templat().get(&template).is_none() {
                return to(&back, "galat", "Templat tidak ditemukan.");
            }
            let assets = match parse_assets(&get("assets")) {
                Ok(m) => m,
                Err(e) => return to(&back, "galat", &e),
            };
            let css = get("css");
            if let Err(e) = super::templat::check_css(&css) {
                return to(&back, "galat", &e);
            }
            if let Err(e) = repo::set_theme_template(&state.pool, &theme, &template, &assets, &css).await {
                tracing::error!(error = %format!("{e:#}"), "admin: pasang templat");
                return to(&back, "galat", "Gagal — sudah menjalankan migration/029_tema_templat.sql?");
            }
            state.refresh_themes().await;
            to(&back, "ok", if template.is_empty() { "Tema kembali memakai tampilan komponen bawaan." } else { "Templat terpasang di tema." })
        }
        _ => to("/admin/templat", "galat", "Aksi tidak dikenal."),
    }
}
