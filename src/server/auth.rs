//! server/auth.rs — akun panel admin: hash sandi argon2id, token sesi acak
//! (cookie) yang di DB hanya disimpan hash SHA-256-nya, dan pengecekan peran.

use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use axum::http::HeaderMap;
use sha2::{Digest, Sha256};

use super::handlers::cookie_value;
use super::repo;
use super::state::AppState;
use crate::web::model::AdminUser;

pub const COOKIE: &str = "ily_admin";
/// Penanda NON-rahasia (bisa dibaca skrip) bahwa browser ini pernah masuk
/// admin — hanya menentukan apakah halaman publik perlu menanyakan sesi.
/// Keamanan tetap di cookie HttpOnly `COOKIE`.
pub const MARKER: &str = "ily_adm_ui";
pub const SESSION_DAYS: i64 = 14;
pub const MIN_PASSWORD: usize = 8;

pub fn hash_password(pw: &str) -> anyhow::Result<String> {
    use rand::RngCore;
    let mut salt = [0u8; 16];
    rand::rng().fill_bytes(&mut salt);
    let salt = SaltString::encode_b64(&salt).map_err(|e| anyhow::anyhow!("salt: {e}"))?;
    Ok(Argon2::default().hash_password(pw.as_bytes(), &salt).map_err(|e| anyhow::anyhow!("hash: {e}"))?.to_string())
}

pub fn verify_password(pw: &str, hash: &str) -> bool {
    PasswordHash::new(hash).is_ok_and(|h| Argon2::default().verify_password(pw.as_bytes(), &h).is_ok())
}

/// Argon2 sengaja mahal (±19 MB RAM, puluhan ms CPU). Dijalankan di thread
/// blocking — di thread async ia membekukan worker tokio (VPS 2 CPU = 2 worker:
/// segelintir login serentak menghentikan SEMUA halaman). Maks 2 bersamaan →
/// banjir percobaan login dari banyak IP tak bisa menghabiskan RAM.
fn antrian_argon() -> &'static tokio::sync::Semaphore {
    static S: std::sync::OnceLock<tokio::sync::Semaphore> = std::sync::OnceLock::new();
    S.get_or_init(|| tokio::sync::Semaphore::new(2))
}

/// Hash pembanding untuk username yang tak ada: waktu respons sama dengan
/// sandi salah → penyerang tak bisa menebak username mana yang terdaftar.
fn hash_palsu() -> &'static str {
    static H: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    H.get_or_init(|| hash_password(&random_hex(16)).unwrap_or_default())
}

pub async fn verify_password_async(pw: String, hash: Option<String>) -> bool {
    let _izin = antrian_argon().acquire().await;
    let ada = hash.is_some();
    tokio::task::spawn_blocking(move || {
        let hash = hash.unwrap_or_else(|| hash_palsu().to_string());
        verify_password(&pw, &hash) && ada
    })
    .await
    .unwrap_or(false)
}

pub async fn hash_password_async(pw: String) -> anyhow::Result<String> {
    let _izin = antrian_argon().acquire().await;
    tokio::task::spawn_blocking(move || hash_password(&pw)).await.map_err(|e| anyhow::anyhow!("hash: {e}"))?
}

/// Byte → hex huruf kecil (tanpa alokasi per byte).
pub fn hex(bytes: &[u8]) -> String {
    const H: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(H[(b >> 4) as usize] as char);
        s.push(H[(b & 15) as usize] as char);
    }
    s
}

/// `n` byte acak sebagai hex (token sesi = 32 byte, nonce CSP = 16 byte).
/// `len` karakter acak dari `alphabet` (CSPRNG) — kunci Kelola, kode tamu.
pub fn random_chars(alphabet: &[u8], len: usize) -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    (0..len).map(|_| alphabet[rng.random_range(0..alphabet.len())] as char).collect()
}

pub fn random_hex(n: usize) -> String {
    use rand::RngCore;
    let mut b = vec![0u8; n];
    rand::rng().fill_bytes(&mut b);
    hex(&b)
}

/// Token sesi acak 32 byte (hex) untuk cookie.
pub fn new_token() -> String {
    random_hex(32)
}

/// SHA-256 hex — token sesi admin & kunci Kelola disimpan dalam bentuk ini.
pub fn token_hash(token: &str) -> String {
    hex(&Sha256::digest(token.as_bytes()))
}

/// Bandingkan dua string rahasia tanpa berhenti di karakter pertama yang beda.
/// Kosong tak pernah cocok.
pub fn same_hash(a: &str, b: &str) -> bool {
    !a.is_empty() && a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Alasan akses admin ditolak.
pub enum Denied {
    NoSession,
    NotAdmin,
}

/// Admin yang masuk & berperan cukup (`need_admin` = peran Admin, bukan
/// Editor). Dipakai handler form (redirect) maupun server fn (galat).
pub async fn require(state: &AppState, headers: &HeaderMap, need_admin: bool) -> Result<AdminUser, Denied> {
    match current(state, headers).await {
        Some(u) if !need_admin || u.is_admin() => Ok(u),
        Some(_) => Err(Denied::NotAdmin),
        None => Err(Denied::NoSession),
    }
}

/// Admin yang sedang masuk (sesi valid & akun aktif).
pub async fn current(state: &AppState, headers: &HeaderMap) -> Option<AdminUser> {
    let token = cookie_value(headers, COOKIE)?;
    if token.len() != 64 {
        return None;
    }
    repo::session_user(&state.pool, &token_hash(token)).await.ok().flatten()
}

pub fn marker_cookie(on: bool) -> String {
    format!("{MARKER}={}; Path=/; SameSite=Lax; Max-Age={}", if on { "1" } else { "" }, if on { SESSION_DAYS * 86_400 } else { 0 })
}

pub fn cookie(value: &str, max_age: i64, headers: &HeaderMap) -> String {
    // Secure hanya di balik proxy HTTPS (dev lokal via http tetap bisa masuk).
    let https = headers.get("x-forwarded-proto").and_then(|v| v.to_str().ok()) == Some("https");
    format!("{COOKIE}={value}; Path=/; HttpOnly; SameSite=Strict; Max-Age={max_age}{}", if https { "; Secure" } else { "" })
}

/// username: huruf kecil, angka, titik, minus, garis bawah; 3–32 karakter.
pub fn valid_username(u: &str) -> bool {
    (3..=32).contains(&u.len()) && u.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '-' | '_'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sandi_dan_token() {
        let h = hash_password("rahasia-panjang").unwrap();
        assert!(h.starts_with("$argon2id$"));
        assert!(verify_password("rahasia-panjang", &h));
        assert!(!verify_password("salah", &h));
        assert!(!verify_password("x", "bukan-hash"));
        assert!(PasswordHash::new(hash_palsu()).is_ok(), "hash pembanding wajib valid (waktu verifikasi sama)");
        let t = new_token();
        assert_eq!(t.len(), 64);
        assert_ne!(t, new_token());
        assert_eq!(token_hash("abc").len(), 64);
        assert!(valid_username("sekar.ayu") && !valid_username("Sekar Ayu") && !valid_username("ab"));
    }
}
