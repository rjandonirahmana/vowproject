use std::sync::{Arc, RwLock};

use deadpool_postgres::Pool;

use super::storage::StorageService;
use crate::web::konten::Konten;
use crate::web::skin::{self, ThemeInfo};

/// Disuntikkan sebagai context Leptos (server fn & SSR) dan Extension axum.
pub struct AppState {
    pub pool: Pool,
    /// `None` bila RUSTFS_ACCESS_KEY kosong — unggahan foto/lagu dilewati.
    pub storage: Option<StorageService>,
    /// Nomor WA admin untuk konfirmasi pembayaran (62…).
    pub admin_wa: String,
    /// `None` bila WAHA_BASE_URL kosong — notifikasi bukti transfer dilewati
    /// (bukti tetap tersimpan & tampil di /admin/undangan).
    pub waha: Option<super::waha::WahaClient>,
    /// Penerima notifikasi bukti transfer (62…).
    pub notify_wa: String,
    /// SITE_URL (tanpa garis miring akhir); kosong = dari header request.
    pub site_url: String,
    /// Kode setup akun admin PERTAMA (ADMIN_TOKEN); kosong = setup ditutup.
    pub admin_token: String,
    /// Katalog tema di memori — dibaca tiap render, dimuat ulang setiap admin
    /// menyimpan tema. Tabel kecil (puluhan baris), jadi tak perlu query per request.
    pub themes: RwLock<Arc<ThemeCatalog>>,
    /// Konten & harga situs (bawaan kode + suntingan di `site_content`).
    pub konten: RwLock<Arc<Konten>>,
    /// Jam sebelum pesanan belum-dibayar dihapus otomatis (server/cleanup.rs).
    pub unpaid_ttl_hours: i64,
    /// Batas percobaan login/setup admin yang gagal.
    pub login_limit: super::security::RateLimit,
    /// RSVP & konfirmasi tanda kasih per IP per undangan (anti-spam buku ucapan).
    pub write_limit: super::security::RateLimit,
    /// Pembuatan undangan (+ unggah foto/lagu ke RustFS) per IP.
    pub create_limit: super::security::RateLimit,
    /// Tema templat (migrasi 029) — HTML+CSS dari tabel theme_templates,
    /// sudah dikompilasi; dimuat ulang tiap admin menyimpan templat.
    pub templat: RwLock<Arc<super::templat::TemplatSet>>,
}

pub struct ThemeCatalog {
    pub list: Vec<ThemeInfo>,
    /// slug → posisi di `list` (pencarian O(1) tiap request).
    index: std::collections::HashMap<String, usize>,
    /// Semua animasi (tabel animations + bawaan yang belum ada di tabel).
    pub anims: Vec<crate::web::anim::AnimInfo>,
    /// Isi /tema.css (Bytes: clone per request tanpa menyalin isi).
    pub css: axum::body::Bytes,
    /// Hash isi CSS → `/tema.css?v=…` (cache browser aman selamanya).
    pub version: String,
    /// Parameter `family=` Google Fonts untuk font tema yang terpakai.
    pub fonts: Vec<&'static str>,
}

impl ThemeCatalog {
    pub fn new(list: Vec<ThemeInfo>, db_anims: Vec<crate::web::anim::AnimInfo>) -> Self {
        let anims = crate::web::anim::merge(db_anims);
        let css = skin::catalog_css(&list, &anims) + &crate::web::anim::catalog_css(&anims);
        let h = super::util::fnv1a64(&css);
        let css = axum::body::Bytes::from(css);
        let fonts = skin::font_families(&list);
        let index = list.iter().enumerate().map(|(i, t)| (t.slug.clone(), i)).collect();
        Self { list, index, anims, css, version: format!("{:08x}", h as u32), fonts }
    }

    pub fn get(&self, slug: &str) -> Option<&ThemeInfo> {
        self.index.get(slug).map(|&i| &self.list[i])
    }

    /// Kunci animasi jenis `kind` ada di katalog ("none" selalu sah untuk
    /// cara membuka & hiasan = tanpa animasi).
    pub fn has_anim(&self, key: &str, kind: &str) -> bool {
        (key == crate::web::anim::NONE && kind != "scroll") || self.anims.iter().any(|a| a.kind == kind && a.key() == key)
    }

    pub fn anim(&self, kind: &str, slug: &str) -> Option<&crate::web::anim::AnimInfo> {
        self.anims.iter().find(|a| a.kind == kind && a.slug == slug)
    }
}

impl AppState {
    pub fn themes(&self) -> Arc<ThemeCatalog> {
        self.themes.read().map(|g| g.clone()).unwrap_or_else(|e| e.into_inner().clone())
    }

    /// Isi animasi bawaan ke tabel `animations` bila belum ada (sekali saat
    /// start; suntingan admin tak ditimpa). Tabel belum migrasi 008 → dilewati.
    pub async fn seed_animations(&self) {
        match super::repo::seed_animations(&self.pool, &crate::web::anim::builtins()).await {
            Ok(0) => {}
            Ok(n) => tracing::info!(n, "animasi bawaan diisikan ke tabel animations"),
            Err(e) => tracing::warn!(error = %format!("{e:#}"), "animasi bawaan belum bisa disimpan ke DB — jalankan migration/008_animasi_semua.sql"),
        }
    }

    /// Muat ulang katalog dari DB. Gagal (mis. migrasi 002 belum dijalankan) →
    /// katalog lama dipertahankan; saat start = tema cadangan bawaan.
    pub async fn reload_themes(&self) -> anyhow::Result<()> {
        let mut list = super::repo::themes(&self.pool).await?;
        // Ornamen per tema (migrasi 012) — belum ada = tema tanpa ornamen.
        match super::repo::ornaments(&self.pool).await {
            Ok(all) => {
                let pos: std::collections::HashMap<String, usize> = list.iter().enumerate().map(|(i, t)| (t.slug.clone(), i)).collect();
                for o in all {
                    if let Some(&i) = pos.get(&o.theme) {
                        list[i].ornaments.push(o);
                    }
                }
            }
            Err(e) => tracing::debug!(error = %format!("{e:#}"), "theme_ornaments belum ada"),
        }
        // Tabel animasi (migrasi 008) — belum ada = hanya animasi bawaan dari kode.
        let anims = match super::repo::animations(&self.pool).await {
            Ok(a) => a,
            Err(e) => {
                tracing::debug!(error = %format!("{e:#}"), "animations belum ada");
                Vec::new()
            }
        };
        let cat = Arc::new(ThemeCatalog::new(list, anims));
        match self.themes.write() {
            Ok(mut g) => *g = cat,
            Err(e) => *e.into_inner() = cat,
        }
        Ok(())
    }
}

impl AppState {
    /// Muat ulang katalog setelah admin menyimpan; galat cukup dicatat
    /// (katalog lama tetap dipakai).
    pub async fn refresh_themes(&self) {
        if let Err(e) = self.reload_themes().await {
            tracing::error!(error = %format!("{e:#}"), "muat ulang katalog tema");
        }
    }

    pub async fn refresh_konten(&self) {
        if let Err(e) = self.reload_konten().await {
            tracing::error!(error = %format!("{e:#}"), "muat ulang konten");
        }
    }

    pub fn konten(&self) -> Arc<Konten> {
        self.konten.read().map(|g| g.clone()).unwrap_or_else(|e| e.into_inner().clone())
    }

    /// Gabungkan isi bawaan dengan baris `site_content`. Bagian rusak dilewati
    /// (tetap bawaan) dan dicatat; tabel absen → error, konten lama dipakai.
    pub async fn reload_konten(&self) -> anyhow::Result<()> {
        let mut k = Konten::default();
        for (key, data) in super::repo::content_rows(&self.pool).await? {
            if !k.apply(&key, data) {
                tracing::warn!(key = %key, "site_content: bagian tak cocok skema — memakai isi bawaan");
            }
        }
        let k = Arc::new(k);
        match self.konten.write() {
            Ok(mut g) => *g = k,
            Err(e) => *e.into_inner() = k,
        }
        Ok(())
    }
}

impl AppState {
    pub fn templat(&self) -> Arc<super::templat::TemplatSet> {
        self.templat.read().map(|g| g.clone()).unwrap_or_else(|e| e.into_inner().clone())
    }

    /// Isi templat bawaan lalu muat semua templat dari DB. Tabel belum ada
    /// (migrasi 029) → templat bawaan dari berkas saja.
    pub async fn reload_templat(&self, seed: bool) {
        if seed {
            match super::repo::seed_templates(&self.pool, &super::templat::builtins()).await {
                Ok(0) => {}
                Ok(n) => tracing::info!(n, "templat bawaan diperbarui di theme_templates"),
                Err(e) => tracing::warn!(error = %format!("{e:#}"), "theme_templates belum ada — jalankan migration/029_tema_templat.sql"),
            }
        }
        let list = match super::repo::templates(&self.pool).await {
            Ok(v) if !v.is_empty() => v,
            Ok(_) => super::templat::builtins(),
            Err(e) => {
                tracing::debug!(error = %format!("{e:#}"), "theme_templates belum ada");
                super::templat::builtins()
            }
        };
        let set = Arc::new(super::templat::TemplatSet::new(list));
        match self.templat.write() {
            Ok(mut g) => *g = set,
            Err(e) => *e.into_inner() = set,
        }
    }
}

pub fn fallback_templat() -> RwLock<Arc<super::templat::TemplatSet>> {
    RwLock::new(Arc::new(super::templat::TemplatSet::new(super::templat::builtins())))
}

pub fn fallback_catalog() -> RwLock<Arc<ThemeCatalog>> {
    RwLock::new(Arc::new(ThemeCatalog::new(vec![ThemeInfo::fallback()], Vec::new())))
}
