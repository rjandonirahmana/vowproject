//! server/storage.rs — RustFS (S3-compatible): foto mempelai & lagu latar.
//! Pola disalin dari bis/src/service/storage.rs.

use aws_sdk_s3::{
    config::{BehaviorVersion, Builder, Credentials, Region},
    primitives::ByteStream,
    Client,
};

use super::config::RustFsConfig;
pub use super::gambar::Ukuran;

pub const MAX_IMAGE: usize = 5 * 1024 * 1024;
pub const MAX_AUDIO: usize = 6 * 1024 * 1024;
/// Video prewedding (latar tema sinema). Dibatasi agar memori server aman —
/// pembeli disarankan mengompres (±1 menit 720p ≈ 10–15 MB).
pub const MAX_VIDEO: usize = 20 * 1024 * 1024;

/// Validasi magic bytes — Content-Type dari klien tak dipercaya begitu saja.
pub fn detect_image(data: &[u8]) -> Option<(&'static str, &'static str)> {
    match data {
        [0xFF, 0xD8, 0xFF, ..] => Some(("image/jpeg", "jpg")),
        [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, ..] => Some(("image/png", "png")),
        [0x52, 0x49, 0x46, 0x46, _, _, _, _, 0x57, 0x45, 0x42, 0x50, ..] => Some(("image/webp", "webp")),
        _ => None,
    }
}

fn detect_audio(data: &[u8]) -> Option<(&'static str, &'static str)> {
    match data {
        [b'I', b'D', b'3', ..] => Some(("audio/mpeg", "mp3")),
        [0xFF, b, ..] if b & 0xE0 == 0xE0 => Some(("audio/mpeg", "mp3")),
        [_, _, _, _, b'f', b't', b'y', b'p', ..] => Some(("audio/mp4", "m4a")),
        [b'O', b'g', b'g', b'S', ..] => Some(("audio/ogg", "ogg")),
        _ => None,
    }
}

/// MP4/MOV (kotak `ftyp`) atau WebM (EBML). Content-Type klien tak dipercaya.
pub fn detect_video(data: &[u8]) -> Option<(&'static str, &'static str)> {
    match data {
        [_, _, _, _, b'f', b't', b'y', b'p', b'q', b't', ..] => Some(("video/quicktime", "mov")),
        [_, _, _, _, b'f', b't', b'y', b'p', ..] => Some(("video/mp4", "mp4")),
        [0x1A, 0x45, 0xDF, 0xA3, ..] => Some(("video/webm", "webm")),
        _ => None,
    }
}

/// Policy S3 baca-saja publik untuk isi bucket — lewat serializer JSON agar
/// nama bucket selalu ter-escape benar.
fn public_read_policy(bucket: &str) -> String {
    serde_json::json!({
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Principal": { "AWS": ["*"] },
            "Action": ["s3:GetObject"],
            "Resource": [format!("arn:aws:s3:::{bucket}/*")],
        }],
    })
    .to_string()
}

/// URL publik objek = `{public_url}/{bucket}/{key}` (path-style). Bucket tidak
/// ditambahkan dua kali bila `RUSTFS_PUBLIC_URL` sudah berakhiran bucket.
fn public_base(public_url: &str, bucket: &str) -> String {
    let base = public_url.trim_end_matches('/');
    if base.ends_with(&format!("/{bucket}")) {
        base.to_string()
    } else {
        format!("{base}/{bucket}")
    }
}

#[derive(Clone)]
pub struct StorageService {
    client: Client,
    bucket: String,
    public_url: String,
    auto_create_bucket: bool,
}

impl StorageService {
    /// `None` bila kredensial kosong (dev lokal tanpa RustFS).
    pub fn new(cfg: &RustFsConfig) -> Option<Self> {
        if cfg.access_key.is_empty() {
            return None;
        }
        let creds = Credentials::new(&cfg.access_key, &cfg.secret_key, None, None, "rustfs");
        let config = Builder::new()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new("us-east-1"))
            .endpoint_url(&cfg.endpoint)
            .credentials_provider(creds)
            .force_path_style(true)
            .build();
        Some(Self {
            client: Client::from_conf(config),
            bucket: cfg.bucket.clone(),
            public_url: public_base(&cfg.public_url, &cfg.bucket),
            auto_create_bucket: cfg.auto_create_bucket,
        })
    }

    pub async fn init(&self) {
        match self.client.list_buckets().send().await {
            Ok(list) => {
                if list.buckets().iter().any(|b| b.name() == Some(&self.bucket)) {
                    tracing::info!(bucket = %self.bucket, "RustFS: bucket sudah ada");
                } else if !self.auto_create_bucket {
                    tracing::warn!(
                        bucket = %self.bucket,
                        "RustFS: bucket belum ada — buat manual (+ policy baca publik) atau set RUSTFS_AUTO_CREATE_BUCKET=true"
                    );
                } else if let Err(e) = self.client.create_bucket().bucket(&self.bucket).send().await {
                    tracing::warn!(error = %format!("{e:#}"), "RustFS: gagal membuat bucket");
                } else {
                    // Bucket baru: foto & lagu diakses tamu langsung lewat URL
                    // publik → izinkan baca-saja (GetObject) untuk semua.
                    // Bucket yang SUDAH ada tidak disentuh (policy-nya milik admin).
                    let policy = public_read_policy(&self.bucket);
                    match self.client.put_bucket_policy().bucket(&self.bucket).policy(policy).send().await {
                        Ok(_) => tracing::info!(bucket = %self.bucket, "RustFS: bucket dibuat + policy baca publik"),
                        Err(e) => tracing::warn!(error = %format!("{e:#}"), "RustFS: bucket dibuat tapi policy publik gagal — set manual agar foto tidak 403"),
                    }
                }
            }
            Err(e) => tracing::warn!(error = %format!("{e:#}"), "RustFS: gagal memeriksa bucket (upload akan gagal)"),
        }
    }

    /// Validasi + optimasi (server/gambar.rs): perkecil dimensi & WebP lossy
    /// berkualitas, metadata dibuang. Tak bisa didekode → ditolak.
    async fn prepare_image(data: Vec<u8>, ukuran: Ukuran) -> anyhow::Result<super::gambar::Hasil> {
        if data.len() > MAX_IMAGE {
            anyhow::bail!("Foto maksimal {} MB", MAX_IMAGE / 1024 / 1024);
        }
        detect_image(&data).ok_or_else(|| anyhow::anyhow!("Foto harus JPEG/PNG/WebP"))?;
        let before = data.len();
        let h = super::gambar::optimasi(data, ukuran).await?;
        tracing::info!(sebelum_kb = before / 1024, sesudah_kb = h.data.len() / 1024, "gambar: dioptimasi");
        Ok(h)
    }

    /// Nama berkas yang belum dipakai: `stem`, lalu `stem-2`, `stem-3`, …
    pub fn unique_name(stem: &str, taken: impl Fn(&str) -> bool) -> String {
        let mut name = stem.to_string();
        let mut n = 2;
        while taken(&name) {
            name = format!("{stem}-{n}");
            n += 1;
        }
        name
    }

    /// Foto dengan jalur terbaca: `foto/{folder}/{nama}.webp`
    /// (mis. foto/yona-doni-k7f3x9m2/sampul.webp).
    pub async fn upload_image_as(&self, data: Vec<u8>, dir: &str, name: &str, ukuran: Ukuran) -> anyhow::Result<String> {
        let h = Self::prepare_image(data, ukuran).await?;
        self.put_key(&format!("foto/{}/{}.{}", path_part(dir, "undangan"), file_stem(name, "foto"), h.ext), h.mime, h.data).await
    }

    /// TIMPA foto yang sudah ada di kunci objek yang sama (sunting undangan:
    /// "ganti" = berkas lama tertimpa, bukan menumpuk). URL diberi `?v=acak`
    /// agar peramban/CDN tak menampilkan salinan lama. `Ok(None)` = URL lama
    /// bukan unggahan kita (gambar bawaan / situs luar) → pemanggil mengunggah baru.
    pub async fn replace_image(&self, data: Vec<u8>, old_url: &str, ukuran: Ukuran) -> anyhow::Result<Option<String>> {
        let Some(key) = self.key_of(old_url) else { return Ok(None) };
        let h = Self::prepare_image(data, ukuran).await?;
        let url = self.put_key(&key, h.mime, h.data).await?;
        Ok(Some(format!("{url}?v={}", super::auth::random_hex(4))))
    }

    /// Lagu unggahan pembeli: `musik/{undangan}/{nama-berkas}-{acak}.{ext}`
    /// (mis. musik/yona-doni-k7f3x9m2/TULUS-Teman-Hidup-3f9a1c.mp3). Akhiran
    /// acak: dua berkas bernama sama (mis. pustaka lagu admin) tak saling timpa.
    pub async fn upload_audio_as(&self, data: Vec<u8>, dir: &str, file_name: &str) -> anyhow::Result<String> {
        if data.len() > MAX_AUDIO {
            anyhow::bail!("Lagu maksimal {} MB", MAX_AUDIO / 1024 / 1024);
        }
        let (mime, ext) = detect_audio(&data).ok_or_else(|| anyhow::anyhow!("Lagu harus MP3/M4A/OGG"))?;
        self.put_key(&format!("musik/{}/{}.{ext}", path_part(dir, "undangan"), unique_stem(file_name, "lagu")), mime, data).await
    }

    /// Video prewedding: `video/{undangan}/{nama-berkas}-{acak}.{ext}`.
    pub async fn upload_video_as(&self, data: Vec<u8>, dir: &str, file_name: &str) -> anyhow::Result<String> {
        if data.len() > MAX_VIDEO {
            anyhow::bail!("Video maksimal {} MB — kompres dulu (mis. 720p)", MAX_VIDEO / 1024 / 1024);
        }
        let (mime, ext) = detect_video(&data).ok_or_else(|| anyhow::anyhow!("Video harus MP4, MOV, atau WebM"))?;
        self.put_key(&format!("video/{}/{}.{ext}", path_part(dir, "undangan"), unique_stem(file_name, "prewedding")), mime, data).await
    }

    /// Kunci objek dari URL publik unggahan KITA (tanpa fragmen #t=/#pos=).
    /// URL lain (lagu bawaan /music/…, gambar /img/…, situs luar) → None.
    pub fn key_of(&self, url: &str) -> Option<String> {
        let url = url.split(['#', '?']).next().unwrap_or(url);
        let key = url.strip_prefix(&self.public_url)?.strip_prefix('/')?;
        (!key.is_empty() && !key.contains("..")).then(|| key.to_string())
    }

    /// Hapus satu unggahan berdasarkan URL publiknya. `false` = bukan milik kita.
    pub async fn delete_url(&self, url: &str) -> anyhow::Result<bool> {
        let Some(key) = self.key_of(url) else { return Ok(false) };
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("hapus {key}: {e}"))?;
        Ok(true)
    }

    async fn put_key(&self, key: &str, mime: &str, data: Vec<u8>) -> anyhow::Result<String> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(data))
            .content_type(mime)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Gagal unggah ke RustFS: {e}"))?;
        Ok(format!("{}/{key}", self.public_url))
    }
}

/// `file_stem` + akhiran acak 6 hex — nama tetap terbaca, kunci tak bentrok.
fn unique_stem(name: &str, fallback: &str) -> String {
    format!("{}-{}", file_stem(name, fallback), super::auth::random_hex(3))
}

/// Nama berkas aman & tetap terbaca untuk kunci objek/URL: ekstensi dibuang,
/// spasi & tanda baca → "-", huruf non-ASCII dilewati, maks 60 karakter.
/// "TULUS - Teman Hidup (Official).mp3" → "TULUS-Teman-Hidup-Official".
pub fn file_stem(name: &str, fallback: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
    let stem = match base.rsplit_once('.') {
        Some((s, ext)) if !s.is_empty() && ext.len() <= 5 => s,
        _ => base,
    };
    let mut out = String::new();
    for c in stem.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if (c.is_whitespace() || c.is_ascii_punctuation()) && !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    let out: String = out.chars().take(60).collect();
    let out = out.trim_matches('-');
    if out.is_empty() { fallback.to_string() } else { out.to_string() }
}

/// Satu segmen folder (kode undangan): huruf kecil, angka, minus.
fn path_part(s: &str, fallback: &str) -> String {
    // Huruf besar dikecilkan dulu (dulu dibuang: "Anindita" → "nindita").
    let p: String = s.chars().map(|c| c.to_ascii_lowercase()).filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-').take(64).collect();
    if p.is_empty() { fallback.to_string() } else { p }
}

#[cfg(test)]
mod tests {
    #[test]
    fn nama_berkas_terbaca() {
        use super::{file_stem, path_part};
        assert_eq!(file_stem("TULUS - Teman Hidup (Official).mp3", "lagu"), "TULUS-Teman-Hidup-Official");
        assert_eq!(file_stem("C:\\Musik\\akad nikah.final.m4a", "lagu"), "akad-nikah-final");
        assert_eq!(file_stem("Lagu Cinta Kita ♥.mp3", "lagu"), "Lagu-Cinta-Kita");
        assert_eq!(file_stem("../../etc/passwd", "lagu"), "passwd");
        assert_eq!(file_stem("♥♥.mp3", "lagu"), "lagu");
        assert_eq!(file_stem(&"a".repeat(200), "lagu").len(), 60);
        assert_eq!(path_part("Anindita-raditya-k7f3/../x", "u"), "anindita-raditya-k7f3x");
    }

    use super::*;

    #[test]
    fn url_publik_memuat_bucket() {
        assert_eq!(public_base("https://image.ulalaapi.store", "undangan"), "https://image.ulalaapi.store/undangan");
        assert_eq!(public_base("https://image.ulalaapi.store/undangan/", "undangan"), "https://image.ulalaapi.store/undangan");
    }

    #[test]
    fn policy_publik_json_valid() {
        let v: serde_json::Value = serde_json::from_str(&public_read_policy("und\"angan")).unwrap();
        assert_eq!(v["Statement"][0]["Resource"][0], "arn:aws:s3:::und\"angan/*");
    }

    #[test]
    fn hanya_hapus_unggahan_sendiri() {
        let cfg = RustFsConfig {
            endpoint: "http://127.0.0.1:9000".into(),
            access_key: "k".into(),
            secret_key: "s".into(),
            bucket: "undangan".into(),
            public_url: "https://image.ulalaapi.store".into(),
            auto_create_bucket: false,
        };
        let st = StorageService::new(&cfg).expect("storage");
        let base = "https://image.ulalaapi.store/undangan";
        assert_eq!(st.key_of(&format!("{base}/foto/abc.jpg#pos=20,30,1.50")).as_deref(), Some("foto/abc.jpg"));
        assert_eq!(st.key_of(&format!("{base}/musik/x.mp3#t=45")).as_deref(), Some("musik/x.mp3"));
        assert_eq!(st.key_of(&format!("{base}/foto/a/sampul.webp?v=3")).as_deref(), Some("foto/a/sampul.webp"));
        // Lagu bawaan, gambar situs, URL luar, dan path aneh TIDAK boleh tersentuh.
        assert!(st.key_of("/music/canon-in-d.mp3#t=45").is_none());
        assert!(st.key_of("/img/layanan/mua-hero.jpg").is_none());
        assert!(st.key_of("https://evil.example/undangan/foto/a.jpg").is_none());
        assert!(st.key_of(&format!("{base}/../rahasia")).is_none());
        assert!(st.key_of(&format!("{base}-lain/foto/a.jpg")).is_none());
    }

    #[test]
    fn nama_unik_tetap_terbaca() {
        let a = unique_stem("TULUS - Teman Hidup.mp3", "lagu");
        assert!(a.starts_with("TULUS-Teman-Hidup-") && a.len() == "TULUS-Teman-Hidup-".len() + 6, "{a}");
        assert_ne!(a, unique_stem("TULUS - Teman Hidup.mp3", "lagu"));
    }

    #[test]
    fn magic_bytes() {
        assert!(detect_audio(b"ID3\x04\0\0").is_some());
        assert!(detect_image(&[0xFF, 0xD8, 0xFF, 0xE0]).is_some());
        assert!(detect_image(b"<svg").is_none());
        assert_eq!(detect_video(b"\0\0\0\x20ftypisom\0\0").map(|v| v.1), Some("mp4"));
        assert_eq!(detect_video(b"\0\0\0\x14ftypqt  \0\0").map(|v| v.1), Some("mov"));
        assert_eq!(detect_video(&[0x1A, 0x45, 0xDF, 0xA3, 0x01]).map(|v| v.1), Some("webm"));
        assert!(detect_video(b"<html>video").is_none());
    }
}
