//! server/waha.rs — klien WAHA (WhatsApp HTTP API) minimal untuk notifikasi
//! admin: bukti transfer pemesan dikirim sebagai GAMBAR + keterangan.
//! Pola dari bis/src/service/waha.rs (timeout eksplisit, galat asli dicatat).
//!
//! `sendImage` di sebagian versi WAHA Core (gratis) tidak tersedia → bila
//! gagal, otomatis jatuh ke `sendText` berisi keterangan + tautan gambar,
//! supaya admin tetap tahu ada pembayaran masuk.

use std::time::Duration;

use base64::Engine;

/// Tanpa batas, reqwest menunggu sampai batas TCP OS bila WAHA mati.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Clone)]
pub struct WahaConfig {
    /// mis. http://172.17.0.1:3001 (tanpa garis miring akhir). Kosong = nonaktif.
    pub base_url: String,
    pub session: String,
    /// Header `X-Api-Key`; kosong = tak dikirim.
    pub api_key: String,
}

#[derive(Clone)]
pub struct WahaClient {
    http: reqwest::Client,
    cfg: WahaConfig,
}

/// Gambar yang dikirim (isi asli + mime, mis. "image/jpeg").
pub struct Gambar<'a> {
    pub data: &'a [u8],
    pub mime: &'a str,
    pub filename: &'a str,
}

/// `62812…` → `62812…@c.us`.
pub fn chat_id(wa: &str) -> String {
    format!("{wa}@c.us")
}

impl WahaClient {
    /// `None` bila WAHA_BASE_URL kosong.
    pub fn new(cfg: WahaConfig) -> Option<Self> {
        if cfg.base_url.is_empty() {
            return None;
        }
        let http = reqwest::Client::builder().timeout(REQUEST_TIMEOUT).build().ok()?;
        Some(Self { http, cfg })
    }

    async fn post(&self, endpoint: &str, body: serde_json::Value) -> anyhow::Result<()> {
        let url = format!("{}/api/{endpoint}", self.cfg.base_url);
        let mut req = self.http.post(&url).json(&body);
        if !self.cfg.api_key.is_empty() {
            req = req.header("X-Api-Key", &self.cfg.api_key);
        }
        let resp = req.send().await.map_err(|e| {
            // `{e}` saja = "error sending request"; penyebabnya di rantai source.
            tracing::warn!(url = %url, error = ?e, "WAHA: gagal mengirim request");
            anyhow::anyhow!("Gagal menghubungi WAHA: {e}")
        })?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("WAHA {endpoint} membalas {status}: {}", body.chars().take(300).collect::<String>());
        }
        Ok(())
    }

    /// Status sesi (mis. "WORKING", "SCAN_QR_CODE", "STOPPED") — untuk log
    /// saat start agar langsung ketahuan WAHA terhubung atau belum.
    pub async fn session_status(&self) -> anyhow::Result<String> {
        let url = format!("{}/api/sessions/{}", self.cfg.base_url, self.cfg.session);
        let mut req = self.http.get(&url).timeout(Duration::from_secs(5));
        if !self.cfg.api_key.is_empty() {
            req = req.header("X-Api-Key", &self.cfg.api_key);
        }
        let resp = req.send().await.map_err(|e| anyhow::anyhow!("WAHA tak terjangkau di {url}: {e}"))?;
        let code = resp.status();
        if !code.is_success() {
            anyhow::bail!("WAHA {url} membalas {code} (API key salah / sesi '{}' tak ada?)", self.cfg.session);
        }
        let v: serde_json::Value = resp.json().await.unwrap_or_default();
        Ok(v.get("status").and_then(|s| s.as_str()).unwrap_or("?").to_string())
    }

    pub async fn send_text(&self, chat_id: &str, text: &str) -> anyhow::Result<()> {
        self.post("sendText", serde_json::json!({ "session": self.cfg.session, "chatId": chat_id, "text": text })).await
    }

    /// Kirim gambar + keterangan. Gagal (mis. WAHA Core tanpa sendImage) →
    /// kirim teks: keterangan + `fallback_url` (tautan gambar) bila ada.
    pub async fn send_image_or_text(&self, chat_id: &str, img: Gambar<'_>, caption: &str, fallback_url: &str) -> anyhow::Result<()> {
        let body = serde_json::json!({
            "session": self.cfg.session,
            "chatId": chat_id,
            "caption": caption,
            "file": {
                "mimetype": img.mime,
                "filename": img.filename,
                "data": base64::engine::general_purpose::STANDARD.encode(img.data),
            },
        });
        match self.post("sendImage", body).await {
            Ok(()) => Ok(()),
            Err(e) => {
                tracing::warn!(error = %format!("{e:#}"), "WAHA: sendImage gagal — kirim teks saja");
                let text = if fallback_url.is_empty() {
                    format!("{caption}\n\n(Gambar bukti gagal dikirim — cek di panel admin.)")
                } else {
                    format!("{caption}\n\nBukti: {fallback_url}")
                };
                self.send_text(chat_id, &text).await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonaktif_tanpa_url() {
        assert!(WahaClient::new(WahaConfig { base_url: String::new(), session: "default".into(), api_key: String::new() }).is_none());
        assert_eq!(chat_id("6289635816942"), "6289635816942@c.us");
    }
}
