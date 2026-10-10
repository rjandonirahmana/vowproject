//! server/util.rs — pembantu kecil bersama.

/// Hash FNV-1a 64-bit — sidik jari isi (checksum migrasi, versi /tema.css).
/// BUKAN untuk keamanan.
pub fn fnv1a64(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x1000_0000_01b3);
    }
    h
}

/// 8 hex dari FNV-1a — versi aset `?v=` (app.js, tata.*, /tema.css).
pub fn hash8(s: &str) -> String {
    format!("{:08x}", fnv1a64(s) as u32)
}
