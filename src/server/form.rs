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

/// Baca seluruh multipart. `Err` = unggahan terputus / melebihi batas body.
pub async fn read(mut mp: Multipart, max_files: usize) -> Result<Form, ()> {
    let mut f = Form::default();
    loop {
        let field = match mp.next_field().await {
            Ok(Some(field)) => field,
            Ok(None) => return Ok(f),
            Err(e) => {
                tracing::warn!(error = %format!("{e:#}"), "form: multipart rusak/terlalu besar");
                return Err(());
            }
        };
        let name = field.name().unwrap_or_default().to_string();
        match field.file_name().map(str::to_string) {
            Some(file_name) => {
                let data = field.bytes().await.map_err(|_| ())?;
                if !data.is_empty() && f.files.len() < max_files {
                    f.files.push(Upload { field: name, file_name, data: data.to_vec() });
                }
            }
            None => {
                let v = field.text().await.unwrap_or_default();
                f.multi.entry(name.clone()).or_default().push(v.clone());
                f.fields.insert(name, v);
            }
        }
    }
}
