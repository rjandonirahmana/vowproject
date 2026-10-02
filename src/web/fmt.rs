//! web/fmt.rs — format tanggal/jam/rupiah Indonesia tanpa dependensi (jalan di
//! server maupun WASM, jadi hasil SSR = hasil hydrate).

use super::model::Event;

const HARI: [&str; 7] = ["Kamis", "Jumat", "Sabtu", "Minggu", "Senin", "Selasa", "Rabu"];
const BULAN: [&str; 12] = [
    "Januari", "Februari", "Maret", "April", "Mei", "Juni", "Juli", "Agustus", "September", "Oktober", "November",
    "Desember",
];

/// Hari sejak 1970-01-01 (algoritma Howard Hinnant).
pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

pub fn days_in_month(y: i64, m: i64) -> i64 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 => 29,
        2 => 28,
        _ => 0,
    }
}

/// "YYYY-MM-DD" yang benar-benar ada di kalender ("2026-02-31" ditolak).
pub fn parse_date(s: &str) -> Option<(i64, i64, i64)> {
    let mut it = s.trim().splitn(3, '-');
    let y = it.next()?.parse().ok()?;
    let m: i64 = it.next()?.parse().ok()?;
    let d: i64 = it.next()?.parse().ok()?;
    ((1..=12).contains(&m) && (1..=days_in_month(y, m)).contains(&d)).then_some((y, m, d))
}

/// "HH:MM" / "H.MM" / "HH:MM:SS" (input type=time). Menit WAJIB 2 digit —
/// "8:5" ditolak karena ambigu (08.05 atau 08.50?).
pub fn parse_time(s: &str) -> Option<(i64, i64)> {
    let (h, m) = s.trim().split_once([':', '.'])?;
    let h: i64 = h.parse().ok()?;
    let mm = m.get(..2)?;
    if !mm.chars().all(|c| c.is_ascii_digit()) || m.len() > 2 && !m[2..].starts_with(':') {
        return None;
    }
    let m: i64 = mm.parse().ok()?;
    (h < 24 && m < 60).then_some((h, m))
}

/// Zona waktu Indonesia: (label, selisih jam dari UTC, zona IANA untuk kalender).
pub const TIMEZONES: &[(&str, i64, &str)] = &[
    ("WIB", 7, "Asia/Jakarta"),
    ("WITA", 8, "Asia/Makassar"),
    ("WIT", 9, "Asia/Jayapura"),
];

fn tz_info(tz: &str) -> (&'static str, i64, &'static str) {
    TIMEZONES.iter().copied().find(|t| t.0 == tz).unwrap_or(TIMEZONES[0])
}

/// "2026-10-24" → "Sabtu, 24 Oktober 2026".
pub fn tanggal_panjang(s: &str) -> String {
    match parse_date(s) {
        Some((y, m, d)) => {
            let hari = HARI[days_from_civil(y, m, d).rem_euclid(7) as usize];
            format!("{hari}, {d} {} {y}", BULAN[(m - 1) as usize])
        }
        None => s.to_string(),
    }
}

/// "2026-10-24" → "24 Okt 2026".
pub fn tanggal_pendek(s: &str) -> String {
    match parse_date(s) {
        Some((y, m, d)) => format!("{d} {} {y}", &BULAN[(m - 1) as usize][..3]),
        None => s.to_string(),
    }
}

/// "08:00","10:00" → "08.00 – 10.00 WIB"; akhir kosong → "08.00 WIB – selesai".
pub fn jam_rentang(a: &str, b: &str, tz: &str) -> String {
    let a = a.trim().replace(':', ".");
    let b = b.trim().replace(':', ".");
    let z = tz_info(tz).0;
    match (a.is_empty(), b.is_empty()) {
        (true, _) => String::new(),
        (false, true) => format!("{a} {z} – selesai"),
        (false, false) => format!("{a} – {b} {z}"),
    }
}

/// Epoch ms untuk tanggal+jam di zona `tz` (WIB/WITA/WIT; lain = WIB).
pub fn epoch_ms(date: &str, time: &str, tz: &str) -> Option<i64> {
    let (y, m, d) = parse_date(date)?;
    let (h, mi) = parse_time(time).unwrap_or((8, 0));
    let secs = days_from_civil(y, m, d) * 86_400 + h * 3600 + mi * 60 - tz_info(tz).1 * 3600;
    Some(secs * 1000)
}

/// Stempel Google Calendar (UTC) "YYYYMMDDTHHMMSSZ".
fn gcal_stamp(ms: i64) -> String {
    let secs = ms.div_euclid(1000);
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}{m:02}{d:02}T{:02}{:02}00Z", rem / 3600, (rem % 3600) / 60)
}

pub fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

pub fn google_calendar_url(title: &str, e: &Event, details: &str) -> String {
    let Some(start) = epoch_ms(&e.date, &e.time_start, &e.tz) else {
        return String::new();
    };
    let end = epoch_ms(&e.date, &e.time_end, &e.tz)
        .filter(|end| *end > start)
        .unwrap_or(start + 3 * 3600 * 1000);
    format!(
        "https://calendar.google.com/calendar/render?action=TEMPLATE&text={}&dates={}/{}&details={}&location={}&ctz={}",
        url_encode(title),
        gcal_stamp(start),
        gcal_stamp(end),
        url_encode(details),
        url_encode(&format!("{}, {}", e.venue, e.address)),
        tz_info(&e.tz).2,
    )
}

pub fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Label bebas → kunci URL: "Adat Minang" → "adat-minang".
pub fn key(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.trim().chars().flat_map(char::to_lowercase) {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    out.truncate(out.trim_end_matches('-').len());
    out
}

/// `key` dibatasi `max` karakter (kode tema/animasi).
pub fn slug(s: &str, max: usize) -> String {
    let k = key(s);
    let cut: String = k.chars().take(max).collect();
    cut.trim_end_matches('-').to_string()
}

/// Kode berbentuk slug: huruf kecil, angka, tanda minus; 1…`max` karakter.
pub fn is_slug(s: &str, max: usize) -> bool {
    !s.is_empty() && s.len() <= max && s.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}

/// Potong & rapikan input teks bebas (tanpa karakter kontrol kecuali baris baru).
pub fn clean(s: &str, max: usize) -> String {
    s.trim().chars().filter(|c| !c.is_control() || *c == '\n').take(max).collect()
}

/// `v` bila termasuk `keys`, selain itu `d`.
pub fn pick<'a>(v: &str, mut keys: impl Iterator<Item = &'a str>, d: &str) -> String {
    if keys.any(|k| k == v) { v.to_string() } else { d.to_string() }
}

/// Pembaca isian formulir bersama (tema, animasi, ornamen — server & WASM):
/// semua nilai dirapikan & dijepit di satu tempat.
pub struct Fields<F: Fn(&str) -> String>(pub F);

impl<F: Fn(&str) -> String> Fields<F> {
    pub fn raw(&self, k: &str) -> String {
        (self.0)(k)
    }
    /// Teks satu baris, tanpa karakter kontrol, maks `max` karakter.
    pub fn text(&self, k: &str, max: usize) -> String {
        (self.0)(k).trim().chars().filter(|c| !c.is_control()).take(max).collect()
    }
    /// Pilihan dari daftar tetap; selain itu `d`.
    pub fn pick<'a>(&self, k: &str, keys: impl Iterator<Item = &'a str>, d: &str) -> String {
        pick(&self.text(k, 60), keys, d)
    }
    /// Bilangan bulat dijepit ke `lo..=hi`; kosong/rusak → `d`.
    pub fn int(&self, k: &str, lo: i32, hi: i32, d: i32) -> i32 {
        (self.0)(k).trim().parse::<i32>().map(|v| v.clamp(lo, hi)).unwrap_or(d)
    }
    /// Bilangan bulat tanpa jepit (dijepit belakangan oleh pemanggil).
    pub fn num(&self, k: &str, d: i32) -> i32 {
        (self.0)(k).trim().parse::<i32>().unwrap_or(d)
    }
    /// Kotak centang tercentang ("1" / "on" / "true").
    pub fn flag(&self, k: &str) -> bool {
        matches!((self.0)(k).as_str(), "1" | "on" | "true")
    }
}

// ── Penyesuaian media lewat fragmen URL (tanpa kolom DB baru) ────────────
// Lagu: `…/lagu.mp3#t=45` — Media Fragments standar, browser mulai di detik 45.
// Foto: `…/foto.jpg#pos=50,30,1.4` — titik fokus x%,y% + zoom, dibaca saat render.
// Fragmen tidak pernah dikirim ke server, jadi berkas di RustFS tetap sama.

pub const MUSIC_START_MAX: u32 = 20 * 60;

/// "45" → "#t=45" (0/kosong/tak valid → "").
pub fn music_start_fragment(secs: &str) -> String {
    match secs.trim().parse::<f64>() {
        Ok(v) if v.is_finite() && v >= 1.0 => format!("#t={}", (v as u32).min(MUSIC_START_MAX)),
        _ => String::new(),
    }
}

/// Detik mulai dari URL lagu (`#t=45`).
pub fn music_start(url: &str) -> u32 {
    url.rsplit_once("#t=").and_then(|(_, t)| t.split([',', '&']).next()?.parse::<f64>().ok()).map(|v| v as u32).unwrap_or(0)
}

/// "50,30,1.4" → (50, 30, 1.4), dibatasi 0–100 % dan zoom 1–3.
pub fn parse_photo_pos(s: &str) -> Option<(f32, f32, f32)> {
    let mut it = s.trim().split(',').map(|p| p.trim().parse::<f32>().ok().filter(|v| v.is_finite()));
    let (x, y, z) = (it.next()??, it.next()??, it.next().flatten().unwrap_or(1.0));
    Some((x.clamp(0.0, 100.0), y.clamp(0.0, 100.0), z.clamp(1.0, 3.0)))
}

/// "#pos=50,30,1.40" (posisi tengah tanpa zoom → "").
pub fn photo_pos_fragment(s: &str) -> String {
    match parse_photo_pos(s) {
        Some((x, y, z)) if (x - 50.0).abs() > 0.5 || (y - 50.0).abs() > 0.5 || z > 1.01 => format!("#pos={x:.0},{y:.0},{z:.2}"),
        _ => String::new(),
    }
}

/// Gaya `<img>` untuk foto ber-fragmen `#pos=` (kosong bila tak ada).
pub fn photo_style(url: &str) -> String {
    match url.rsplit_once("#pos=").and_then(|(_, p)| parse_photo_pos(p)) {
        Some((x, y, z)) => format!("object-position:{x}% {y}%;transform:scale({z});transform-origin:{x}% {y}%"),
        None => String::new(),
    }
}

/// 149000 → "149.000"
pub fn ribuan(n: i64) -> String {
    let neg = n < 0;
    let s = n.unsigned_abs().to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push('.');
        }
        out.push(c);
    }
    if neg {
        format!("-{out}")
    } else {
        out
    }
}

/// 149000 → "Rp 149.000"
pub fn rupiah(n: i64) -> String {
    format!("Rp {}", ribuan(n))
}

/// 42_650_000 → "Rp 42,65 Jt"
pub fn rupiah_ringkas(n: i64) -> String {
    if n >= 1_000_000 {
        let jt = n as f64 / 1_000_000.0;
        let s = format!("{jt:.2}");
        let s = s.trim_end_matches('0').trim_end_matches('.').replace('.', ",");
        format!("Rp {s} Jt")
    } else {
        rupiah(n)
    }
}

/// Selisih detik → "10 menit yang lalu".
pub fn lalu(secs: i64) -> String {
    let s = secs.max(0);
    match s {
        0..=59 => "baru saja".into(),
        60..=3599 => format!("{} menit yang lalu", s / 60),
        3600..=86_399 => format!("{} jam yang lalu", s / 3600),
        86_400..=2_591_999 => format!("{} hari yang lalu", s / 86_400),
        _ => format!("{} bulan yang lalu", s / 2_592_000),
    }
}

/// Nomor HP Indonesia → format wa.me (62…). Kosong bila tak valid.
pub fn wa_number(raw: &str) -> String {
    let d: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    let d = if let Some(r) = d.strip_prefix('0') {
        format!("62{r}")
    } else if d.starts_with('8') {
        format!("62{d}")
    } else {
        d
    };
    // wa.me hanya untuk nomor seluler (62 8…), bukan telepon rumah (62 21…).
    if (11..=15).contains(&d.len()) && d.starts_with("628") {
        d
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tanggal_tak_ada_ditolak() {
        assert!(parse_date("2026-02-31").is_none());
        assert!(parse_date("2026-02-29").is_none());
        assert!(parse_date("2028-02-29").is_some());
        assert!(parse_date("2026-04-31").is_none());
        assert_eq!(days_in_month(2000, 2), 29);
        assert_eq!(days_in_month(1900, 2), 28);
    }

    #[test]
    fn tanggal() {
        assert_eq!(tanggal_panjang("2026-10-24"), "Sabtu, 24 Oktober 2026");
        assert_eq!(tanggal_panjang("2024-10-26"), "Sabtu, 26 Oktober 2024");
        assert_eq!(tanggal_panjang("2026-09-29"), "Selasa, 29 September 2026");
        assert_eq!(civil_from_days(days_from_civil(2026, 2, 28)), (2026, 2, 28));
    }

    #[test]
    fn fragmen_media() {
        assert_eq!(music_start_fragment("45"), "#t=45");
        assert_eq!(music_start_fragment("0"), "");
        assert_eq!(music_start_fragment("abc"), "");
        assert_eq!(music_start_fragment("99999"), format!("#t={MUSIC_START_MAX}"));
        assert_eq!(music_start("/music/a.mp3#t=45"), 45);
        assert_eq!(music_start("/music/a.mp3"), 0);
        assert_eq!(photo_pos_fragment("50,50,1"), "");
        assert_eq!(photo_pos_fragment("20,140,9"), "#pos=20,100,3.00");
        assert_eq!(photo_pos_fragment("x;}"), "");
        assert_eq!(photo_style("https://c/f.jpg#pos=20,30,1.50"), "object-position:20% 30%;transform:scale(1.5);transform-origin:20% 30%");
        assert_eq!(photo_style("https://c/f.jpg"), "");
    }

    #[test]
    fn uang() {
        assert_eq!(rupiah(149_000), "Rp 149.000");
        assert_eq!(rupiah(1_500_000), "Rp 1.500.000");
        assert_eq!(rupiah_ringkas(42_650_000), "Rp 42,65 Jt");
        assert_eq!(rupiah_ringkas(3_000_000), "Rp 3 Jt");
    }

    #[test]
    fn jam_dan_wa() {
        assert_eq!(jam_rentang("08:00", "10:00", "WIB"), "08.00 – 10.00 WIB");
        assert_eq!(jam_rentang("08:00", "", "WITA"), "08.00 WITA – selesai");
        // Jam yang sama di WITA terjadi 1 jam lebih awal (UTC) daripada WIB.
        let wib = epoch_ms("2026-10-24", "08:00", "WIB").unwrap();
        assert_eq!(epoch_ms("2026-10-24", "08:00", "WITA").unwrap(), wib - 3_600_000);
        assert_eq!(epoch_ms("2026-10-24", "08:00", "WIT").unwrap(), wib - 7_200_000);
        assert!(parse_time("8:5").is_none() && parse_time("8:5x").is_none());
        assert_eq!(parse_time("08:05:30"), Some((8, 5)));
        assert_eq!(parse_time("8.30"), Some((8, 30)));
        assert_eq!(wa_number("0812-9842-1092"), "6281298421092");
        assert_eq!(wa_number("123"), "");
        // Telepon rumah tidak bisa dipakai WhatsApp.
        assert_eq!(wa_number("021-5551234"), "");
        assert_eq!(wa_number("+62 812 9842 1092"), "6281298421092");
        // 24 Okt 2026 08:00 WIB = 01:00 UTC
        let ms = epoch_ms("2026-10-24", "08:00", "WIB").unwrap();
        assert_eq!(gcal_stamp(ms), "20261024T010000Z");
    }
}

/// `document` browser (hanya dipanggil dari Effect — tak pernah jalan saat SSR).
pub fn document() -> Option<web_sys::Document> {
    web_sys::window().and_then(|w| w.document())
}
