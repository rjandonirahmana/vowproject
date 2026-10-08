//! server/wa.rs — klien waxum (https://github.com/imtaqin/waxum), gateway
//! WhatsApp REST pengganti WAHA. Dipakai untuk:
//!   * admin  — bukti transfer baru (gambar + keterangan);
//!   * pemesan — tautan Kelola saat pesanan dibuat, undangan aktif, tautan
//!     Kelola diterbitkan ulang;
//!   * tamu   — kunci spesial story (6 digit).
//!
//! API: `POST {base}/api/v1/sessions/{sid}/messages/{text|image}` dengan
//! `Authorization: Bearer {token}`. Penerima cukup nomor `62812…` — waxum
//! sendiri mengubahnya ke JID (termasuk alamat LID kontak).
//!
//! "Terkirim saat perlu": galat SEMENTARA (jaringan putus, 5xx, 503 sesi belum
//! tersambung, 429) dicoba ulang dengan jeda bertahap; galat permanen (4xx:
//! token salah, nomor tak valid) langsung dikembalikan — mengulang tak menolong.

use std::time::Duration;

use base64::Engine;

/// Per percobaan. Tanpa batas, reqwest menunggu sampai batas TCP OS.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Clone)]
pub struct WaConfig {
    /// mis. http://172.17.0.1:3451 (tanpa garis miring akhir). Kosong = nonaktif.
    pub base_url: String,
    /// ID sesi waxum (yang sudah dipasangkan lewat QR / kode).
    pub session: String,
    /// SUPERADMIN_TOKEN atau token ber-scope sesi dari `POST /api/v1/tokens`.
    pub token: String,
}

#[derive(Clone)]
pub struct WaClient {
    http: reqwest::Client,
    cfg: WaConfig,
}

/// Gambar yang dikirim (isi + mime, mis. "image/jpeg").
pub struct Gambar<'a> {
    pub data: &'a [u8],
    pub mime: &'a str,
}

/// Seberapa gigih satu pengiriman.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Gigih {
    /// Pengguna sedang menunggu (kunci story): 2 percobaan, jeda 2 dtk.
    Cepat,
    /// Notifikasi di latar: 4 percobaan, jeda 3 → 10 → 30 dtk (±45 dtk) —
    /// cukup untuk melewati sesi yang sedang menyambung ulang.
    Latar,
}

impl Gigih {
    fn jeda(self) -> &'static [u64] {
        match self {
            Gigih::Cepat => &[2],
            Gigih::Latar => &[3, 10, 30],
        }
    }
}

/// Galat dari satu percobaan: `sementara` = layak diulang.
#[derive(Debug)]
struct Gagal {
    sementara: bool,
    pesan: String,
}

/// Status HTTP yang layak diulang: 408/429/5xx (503 = sesi belum tersambung).
fn layak_ulang(code: u16) -> bool {
    code == 408 || code == 429 || code >= 500
}

impl WaClient {
    /// `None` bila WAXUM_BASE_URL kosong.
    pub fn new(cfg: WaConfig) -> Option<Self> {
        if cfg.base_url.is_empty() {
            return None;
        }
        let http = reqwest::Client::builder().timeout(REQUEST_TIMEOUT).build().ok()?;
        Some(Self { http, cfg })
    }

    fn url(&self, path: &str) -> String {
        format!("{}/api/v1/sessions/{}/{path}", self.cfg.base_url, self.cfg.session)
    }

    fn auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if self.cfg.token.is_empty() { req } else { req.bearer_auth(&self.cfg.token) }
    }

    async fn post_once(&self, path: &str, body: &serde_json::Value) -> Result<(), Gagal> {
        let resp = self.auth(self.http.post(self.url(path)).json(body)).send().await.map_err(|e| Gagal {
            sementara: true,
            // `{e}` saja = "error sending request"; penyebabnya di rantai source.
            pesan: format!("waxum tak terjangkau: {e:?}"),
        })?;
        let code = resp.status();
        if code.is_success() {
            return Ok(());
        }
        let body = resp.text().await.unwrap_or_default();
        Err(Gagal { sementara: layak_ulang(code.as_u16()), pesan: format!("waxum {path} membalas {code}: {}", body.chars().take(300).collect::<String>()) })
    }

    /// Kirim dengan percobaan ulang untuk galat sementara.
    async fn post(&self, path: &str, body: serde_json::Value, gigih: Gigih) -> anyhow::Result<()> {
        let jeda = gigih.jeda();
        let mut ke = 0;
        loop {
            match self.post_once(path, &body).await {
                Ok(()) => {
                    if ke > 0 {
                        tracing::info!(path, percobaan = ke + 1, "WA: terkirim setelah dicoba ulang");
                    }
                    return Ok(());
                }
                Err(g) if g.sementara && ke < jeda.len() => {
                    tracing::warn!(path, percobaan = ke + 1, ulang_dalam_dtk = jeda[ke], error = %g.pesan, "WA: gagal sementara — dicoba ulang");
                    tokio::time::sleep(Duration::from_secs(jeda[ke])).await;
                    ke += 1;
                }
                Err(g) => anyhow::bail!(g.pesan),
            }
        }
    }

    /// Status sesi waxum (mis. "logged_in", "waiting_for_qr", "disconnected") —
    /// dicek saat start agar langsung ketahuan sudah terhubung atau belum.
    pub async fn session_status(&self) -> anyhow::Result<String> {
        let url = self.url("status");
        let resp = self
            .auth(self.http.get(&url).timeout(Duration::from_secs(5)))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("waxum tak terjangkau di {url}: {e}"))?;
        let code = resp.status();
        if !code.is_success() {
            anyhow::bail!("waxum {url} membalas {code} (token salah / sesi '{}' tak ada?)", self.cfg.session);
        }
        let v: serde_json::Value = resp.json().await.unwrap_or_default();
        Ok(v.get("status").and_then(|s| s.as_str()).unwrap_or("?").to_string())
    }

    /// `to` = nomor `62812…` (fmt::wa_number).
    pub async fn send_text(&self, to: &str, text: &str, gigih: Gigih) -> anyhow::Result<()> {
        self.post("messages/text", serde_json::json!({ "to": to, "text": text }), gigih).await
    }

    /// Kirim gambar + keterangan. Gagal permanen → teks: keterangan +
    /// `fallback_url` (tautan gambar) bila ada, supaya pesan intinya tetap sampai.
    pub async fn send_image_or_text(&self, to: &str, img: Gambar<'_>, caption: &str, fallback_url: &str, gigih: Gigih) -> anyhow::Result<()> {
        let body = serde_json::json!({
            "to": to,
            "caption": caption,
            "image": {
                "data": base64::engine::general_purpose::STANDARD.encode(img.data),
                "mimetype": img.mime,
            },
        });
        match self.post("messages/image", body, gigih).await {
            Ok(()) => Ok(()),
            Err(e) => {
                tracing::warn!(error = %format!("{e:#}"), "WA: kirim gambar gagal — kirim teks saja");
                let text = if fallback_url.is_empty() {
                    format!("{caption}\n\n(Gambar bukti gagal dikirim — cek di panel admin.)")
                } else {
                    format!("{caption}\n\nBukti: {fallback_url}")
                };
                self.send_text(to, &text, gigih).await
            }
        }
    }

    /// Kirim teks di latar (pemanggil tak menunggu); hasil hanya dicatat.
    pub fn spawn_text(&self, to: String, text: String, apa: &'static str) {
        if to.is_empty() {
            tracing::warn!(apa, "WA: nomor tujuan kosong — tidak dikirim");
            return;
        }
        let wa = self.clone();
        tokio::spawn(async move {
            match wa.send_text(&to, &text, Gigih::Latar).await {
                Ok(()) => tracing::info!(apa, "WA: terkirim"),
                Err(e) => tracing::error!(apa, error = %format!("{e:#}"), "WA: GAGAL setelah dicoba ulang"),
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonaktif_tanpa_url() {
        assert!(WaClient::new(WaConfig { base_url: String::new(), session: "undangan".into(), token: String::new() }).is_none());
    }

    #[test]
    fn alamat_api_v1() {
        let wa = WaClient::new(WaConfig { base_url: "http://172.17.0.1:3451".into(), session: "undangan".into(), token: "t".into() }).unwrap();
        assert_eq!(wa.url("messages/text"), "http://172.17.0.1:3451/api/v1/sessions/undangan/messages/text");
    }

    #[test]
    fn hanya_galat_sementara_diulang() {
        assert!(layak_ulang(503) && layak_ulang(500) && layak_ulang(429) && layak_ulang(408));
        assert!(!layak_ulang(400) && !layak_ulang(401) && !layak_ulang(404));
        assert_eq!(Gigih::Latar.jeda().len(), 3);
    }
}
