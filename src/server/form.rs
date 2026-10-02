//! server/form.rs — pembaca formulir multipart bersama untuk semua handler
//! (pemesanan /buat, tema, animasi, banner, ornamen, konten). Satu loop,
//! satu aturan: teks ke `fields` (nilai berulang juga ke `multi`), berkas
//! tak kosong ke `files` (dibatasi `max_files` agar memori tak membengkak).

use std::collections::HashMap;

use axum::extract::Multipart;

pub struct Upload {
    /// Nama input form (mis. `img_file`).
    pub field: String,
    /// Nama berkas asli dari browser (untuk nama objek RustFS yang terbaca).
    pub file_name: String,
    pub data: Vec<u8>,
}

#[derive(Default)]
pub struct Form {
    pub fields: HashMap<String, String>,
    /// Semua nilai untuk input yang muncul berulang (mis. centang `addon`).
    pub multi: HashMap<String, Vec<String>>,
    pub files: Vec<Upload>,
}

impl Form {
    /// Nilai mentah (kosong bila tak ada).
    pub fn raw(&self, k: &str) -> String {
        self.fields.get(k).cloned().unwrap_or_default()
    }
    /// Teks dirapikan (`fmt::clean`) maks `max` karakter.
    pub fn get(&self, k: &str, max: usize) -> String {
        crate::web::fmt::clean(self.fields.get(k).map(String::as_str).unwrap_or(""), max)
    }
    pub fn all(&self, k: &str) -> &[String] {
        self.multi.get(k).map(Vec::as_slice).unwrap_or(&[])
    }
    pub fn has(&self, k: &str) -> bool {
        self.fields.contains_key(k)
    }
    /// Ambil (dan keluarkan) berkas pertama untuk `field`.
    pub fn take_file(&mut self, field: &str) -> Option<Upload> {
        let i = self.files.iter().position(|u| u.field == field)?;
        Some(self.files.swap_remove(i))
    }
}

/// Batas per bagian — DefaultBodyLimit (±41 MB) hanya membatasi TOTAL body,
/// jadi satu input teks bisa memakan semuanya ke memori tanpa batas ini.
/// Teks terpanjang yang sah = CSS animasi (anim::CSS_MAX 20 KB).
pub const MAX_TEXT_FIELD: usize = 64 * 1024;
const MAX_FIELD_NAME: usize = 64;
const MAX_FIELDS: usize = 2_000;
/// Berkas terbesar yang sah (lagu); validasi per jenis tetap di storage.rs.
pub const MAX_FILE: usize = if super::storage::MAX_AUDIO > super::storage::MAX_IMAGE {
    super::storage::MAX_AUDIO
} else {
    super::storage::MAX_IMAGE
};

/// Baca satu bagian per potongan; `Err` begitu melewati `max` (tak sampai
/// dimuat penuh). `keep = false` → dibaca & dibuang (berkas di atas kuota).
async fn read_capped(field: &mut axum::extract::multipart::Field<'_>, max: usize, keep: bool) -> Result<Vec<u8>, ()> {
    let mut buf = Vec::new();
    let mut total = 0usize;
    while let Some(chunk) = field.chunk().await.map_err(|_| ())? {
        total += chunk.len();
        if total > max {
            tracing::warn!(field = field.name().unwrap_or_default(), max, "form: bagian melebihi batas");
            return Err(());
        }
        if keep {
            buf.extend_from_slice(&chunk);
        }
    }
    Ok(buf)
}

/// Baca seluruh multipart. `Err` = unggahan terputus / melebihi batas body
/// atau batas per bagian di atas.
pub async fn read(mut mp: Multipart, max_files: usize) -> Result<Form, ()> {
    let mut f = Form::default();
    let mut count = 0usize;
    loop {
        let field = match mp.next_field().await {
            Ok(Some(field)) => field,
            Ok(None) => return Ok(f),
            Err(e) => {
                tracing::warn!(error = %format!("{e:#}"), "form: multipart rusak/terlalu besar");
                return Err(());
            }
        };
        let mut field = field;
        count += 1;
        let name = field.name().unwrap_or_default().to_string();
        if count > MAX_FIELDS || name.len() > MAX_FIELD_NAME {
            tracing::warn!(count, name_len = name.len(), "form: terlalu banyak input / nama input terlalu panjang");
            return Err(());
        }
        match field.file_name().map(str::to_string) {
            Some(file_name) => {
                let keep = f.files.len() < max_files;
                let data = read_capped(&mut field, MAX_FILE, keep).await?;
                if !data.is_empty() && keep {
                    f.files.push(Upload { field: name, file_name, data });
                }
            }
            None => {
                let v = String::from_utf8(read_capped(&mut field, MAX_TEXT_FIELD, true).await?).unwrap_or_default();
                f.multi.entry(name.clone()).or_default().push(v.clone());
                f.fields.insert(name, v);
            }
        }
    }
}
