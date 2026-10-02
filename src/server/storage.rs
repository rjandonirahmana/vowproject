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

/// Validasi magic bytes — Content-Type dari klien tak dipercaya begitu saja.
fn detect_image(data: &[u8]) -> Option<(&'static str, &'static str)> {
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
        })
    }

    pub async fn init(&self) {
        match self.client.list_buckets().send().await {
            Ok(list) => {
                if list.buckets().iter().any(|b| b.name() == Some(&self.bucket)) {
                    tracing::info!(bucket = %self.bucket, "RustFS: bucket sudah ada");
                } else if let Err(e) = self.client.create_bucket().bucket(&self.bucket).send().await {
                    tracing::warn!(error = %format!("{e:#}"), "RustFS: gagal membuat bucket");
                } else {
                    // Bucket baru: foto & lagu diakses tamu langsung lewat URL
                    // publik → izinkan baca-saja (GetObject) untuk semua.
                    // Bucket yang SUDAH ada tidak disentuh (policy-nya milik admin).
                    let policy = format!(
                        r#"{{"Version":"2012-10-17","Statement":[{{"Effect":"Allow","Principal":{{"AWS":["*"]}},"Action":["s3:GetObject"],"Resource":["arn:aws:s3:::{}/*"]}}]}}"#,
                        self.bucket
                    );
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
    /// berkualitas; hasil tak lebih kecil → berkas asli.
    async fn prepare_image(data: Vec<u8>, ukuran: Ukuran) -> anyhow::Result<(Vec<u8>, &'static str, &'static str)> {
        if data.len() > MAX_IMAGE {
            anyhow::bail!("Foto maksimal {} MB", MAX_IMAGE / 1024 / 1024);
        }
        let (mime, ext) = detect_image(&data).ok_or_else(|| anyhow::anyhow!("Foto harus JPEG/PNG/WebP"))?;
        let before = data.len();
        Ok(match super::gambar::optimasi(data, ukuran).await {
            (_, Some(h)) => {
                tracing::info!(sebelum_kb = before / 1024, sesudah_kb = h.data.len() / 1024, "gambar: dioptimasi");
                (h.data, h.mime, h.ext)
            }
            (asli, None) => (asli, mime, ext),
        })
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
    /// (mis. foto/anindita-raditya-k7f3x9m2/sampul.webp).
    pub async fn upload_image_as(&self, data: Vec<u8>, dir: &str, name: &str, ukuran: Ukuran) -> anyhow::Result<String> {
        let (data, mime, ext) = Self::prepare_image(data, ukuran).await?;
        self.put_key(&format!("foto/{}/{}.{ext}", path_part(dir, "undangan"), file_stem(name, "foto")), mime, data).await
    }

    /// Lagu unggahan pembeli: `musik/{undangan}/{nama-berkas-asli}.{ext}`
    /// (mis. musik/anindita-raditya-k7f3x9m2/TULUS-Teman-Hidup.mp3).
    pub async fn upload_audio_as(&self, data: Vec<u8>, dir: &str, file_name: &str) -> anyhow::Result<String> {
        if data.len() > MAX_AUDIO {
            anyhow::bail!("Lagu maksimal {} MB", MAX_AUDIO / 1024 / 1024);
        }
        let (mime, ext) = detect_audio(&data).ok_or_else(|| anyhow::anyhow!("Lagu harus MP3/M4A/OGG"))?;
        self.put_key(&format!("musik/{}/{}.{ext}", path_part(dir, "undangan"), file_stem(file_name, "lagu")), mime, data).await
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
    fn hanya_hapus_unggahan_sendiri() {
        let cfg = RustFsConfig {
            endpoint: "http://127.0.0.1:9000".into(),
            access_key: "k".into(),
            secret_key: "s".into(),
            bucket: "undangan".into(),
            public_url: "https://image.ulalaapi.store".into(),
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
    fn magic_bytes() {
        assert!(detect_audio(b"ID3\x04\0\0").is_some());
        assert!(detect_image(&[0xFF, 0xD8, 0xFF, 0xE0]).is_some());
        assert!(detect_image(b"<svg").is_none());
    }
}
