//! server/gambar.rs — optimasi foto sebelum disimpan ke RustFS.
//!
//! Foto HP (JPEG 3–8 MB, 4000×3000) jauh melebihi yang dibutuhkan undangan.
//! Langkah:
//!   1. dekode JPEG/PNG/WebP dengan batas aman (cegah "bom dekompresi");
//!   2. terapkan rotasi EXIF (foto HP tegak) — metadata (termasuk GPS) dibuang;
//!   3. perkecil sisi terpanjang ke `max_side` (Lanczos3) bila lebih besar;
//!   4. simpan sebagai WebP lossy kualitas tinggi (transparansi tetap ada).
//! Hasil lebih besar dari aslinya (mis. gambar kecil yang sudah optimal) →
//! berkas asli dipakai apa adanya.
//!
//! Catatan: WebP/PNG *lossless* justru MEMBESARKAN foto kamera (JPEG sudah
//! lossy); yang efektif untuk foto adalah perkecil dimensi + lossy berkualitas.

use std::io::Cursor;
use std::sync::OnceLock;

use image::{imageops::FilterType, DynamicImage, ImageDecoder, ImageReader, Limits};
use tokio::sync::Semaphore;

/// Kualitas WebP (0–100). 82 ≈ tak terbedakan mata untuk foto.
pub const QUALITY: f32 = 82.0;

/// Sisi terpanjang maksimum per jenis penggunaan.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ukuran {
    /// Foto undangan (sampul, mempelai, galeri) — tampil ≤ ±800px CSS (2× retina).
    Foto,
    /// Banner strip (2880×320).
    Banner,
    /// Gambar admin lain (latar tema, ornamen, paket layanan).
    Aset,
}

impl Ukuran {
    pub fn max_side(self) -> u32 {
        match self {
            Ukuran::Foto => 1600,
            Ukuran::Banner => 2880,
            Ukuran::Aset => 2000,
        }
    }
}

/// Hasil optimasi: (isi, mime, ekstensi).
pub struct Hasil {
    pub data: Vec<u8>,
    pub mime: &'static str,
    pub ext: &'static str,
}

/// Maksimal 2 foto diproses bersamaan (dekode foto 12 MP ±50 MB RAM).
fn antrian() -> &'static Semaphore {
    static S: OnceLock<Semaphore> = OnceLock::new();
    S.get_or_init(|| Semaphore::new(2))
}

/// Versi async: jalan di thread blocking + antrian. `Ok((asli, None))` = pakai
/// berkas asli. Thread panik → `Err` (data asli ikut hilang bersama thread;
/// JANGAN dianggap "tak dioptimasi", nanti yang terunggah objek kosong).
pub async fn optimasi(data: Vec<u8>, ukuran: Ukuran) -> anyhow::Result<(Vec<u8>, Option<Hasil>)> {
    let _izin = antrian().acquire().await;
    let max = ukuran.max_side();
    match tokio::task::spawn_blocking(move || {
        let hasil = optimasi_sync(&data, max);
        (data, hasil)
    })
    .await
    {
        Ok((data, Ok(h))) => Ok((data, h)),
        Ok((data, Err(e))) => {
            tracing::warn!(error = %format!("{e:#}"), "gambar: optimasi gagal — berkas asli disimpan");
            Ok((data, None))
        }
        Err(e) => {
            tracing::error!(error = %e, "gambar: thread optimasi panik");
            anyhow::bail!("Foto gagal diproses, coba unggah ulang.")
        }
    }
}

/// Inti optimasi (murni, untuk test). `Ok(None)` = hasil tak lebih kecil.
pub fn optimasi_sync(data: &[u8], max_side: u32) -> anyhow::Result<Option<Hasil>> {
    let mut reader = ImageReader::new(Cursor::new(data)).with_guessed_format()?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(12_000);
    limits.max_image_height = Some(12_000);
    limits.max_alloc = Some(256 * 1024 * 1024);
    reader.limits(limits);
    let mut dec = reader.into_decoder()?;
    let orientasi = dec.orientation()?;
    let mut img = DynamicImage::from_decoder(dec)?;
    img.apply_orientation(orientasi);

    let (w, h) = (img.width(), img.height());
    if w.max(h) > max_side {
        img = img.resize(max_side, max_side, FilterType::Lanczos3);
    }
    let (w, h) = (img.width(), img.height());
    let webp = if img.color().has_alpha() {
        let rgba = img.to_rgba8();
        webp::Encoder::from_rgba(&rgba, w, h).encode(QUALITY).to_vec()
    } else {
        let rgb = img.to_rgb8();
        webp::Encoder::from_rgb(&rgb, w, h).encode(QUALITY).to_vec()
    };
    Ok((webp.len() < data.len()).then_some(Hasil { data: webp, mime: "image/webp", ext: "webp" }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb, Rgba};

    /// "Foto" sintetis 4000×3000 bergradasi + derau (mirip foto, sulit dikompres).
    fn foto_jpeg(w: u32, h: u32) -> Vec<u8> {
        let img = ImageBuffer::from_fn(w, h, |x, y| {
            let n = ((x * 7919 + y * 104_729) % 23) as u8;
            Rgb([(x * 255 / w) as u8 ^ n, (y * 255 / h) as u8, ((x + y) % 256) as u8 ^ (n / 2)])
        });
        let mut out = Vec::new();
        DynamicImage::ImageRgb8(img)
            .write_to(&mut Cursor::new(&mut out), image::ImageFormat::Jpeg)
            .unwrap();
        out
    }

    #[test]
    fn foto_besar_diperkecil_dan_jadi_webp() {
        let asli = foto_jpeg(4000, 3000);
        let h = optimasi_sync(&asli, Ukuran::Foto.max_side()).unwrap().expect("harus lebih kecil");
        assert_eq!(h.ext, "webp");
        let img = image::load_from_memory(&h.data).unwrap();
        assert_eq!((img.width(), img.height()), (1600, 1200), "sisi terpanjang 1600, rasio tetap");
        assert!(h.data.len() * 4 < asli.len(), "minimal 75% lebih kecil: {} → {}", asli.len(), h.data.len());
    }

    #[test]
    fn transparansi_tetap() {
        let img = ImageBuffer::from_fn(800, 600, |x, _| Rgba([200, 150, 60, if x < 400 { 0 } else { 255 }]));
        let mut png = Vec::new();
        DynamicImage::ImageRgba8(img).write_to(&mut Cursor::new(&mut png), image::ImageFormat::Png).unwrap();
        // PNG polos sangat kecil → mungkin tak diperkecil; yang penting bila diproses, alfa tetap.
        if let Some(h) = optimasi_sync(&png, 2000).unwrap() {
            let out = image::load_from_memory(&h.data).unwrap().to_rgba8();
            assert_eq!(out.get_pixel(10, 10)[3], 0);
            assert_eq!(out.get_pixel(700, 10)[3], 255);
        }
    }

    #[test]
    fn bukan_gambar_ditolak() {
        assert!(optimasi_sync(b"bukan gambar sama sekali", 1600).is_err());
    }
}
