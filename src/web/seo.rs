//! web/seo.rs — SEO halaman publik: SATU komponen `Seo` untuk judul,
//! description, canonical, Open Graph & Twitter Card, plus `JsonLd` (data
//! terstruktur schema.org yang dibaca Google).
//!
//! Aturan:
//!   * Tiap halaman publik memakai `<Seo …/>` tepat sekali (App tak lagi
//!     memasang description bawaan → tak ada tag ganda).
//!   * Canonical & og:url selalu ke domain produksi `SITE_URL` (bukan host
//!     request) agar staging/IP tak dianggap duplikat.
//!   * Data terstruktur hanya berisi fakta yang tampil di halaman — TANPA
//!     rating/ulasan contoh (Google menghukum ulasan palsu).

use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};
use serde_json::{json, Value};

/// Domain produksi (canonical, sitemap, og:url).
pub const SITE_URL: &str = "https://ilyvowcraft.online";
/// Gambar pratinjau bawaan 1200×630 (WhatsApp, Facebook, X, Google Discover).
pub const OG_IMAGE: &str = "/img/og-ilyvowcraft.jpg";
/// Induk usaha (tampil di header & data Organization).
pub const PARENT_ORG: &str = "Tresno Mulyo Group";

pub fn abs(path: &str) -> String {
    if path.starts_with("http://") || path.starts_with("https://") { path.to_string() } else { format!("{SITE_URL}{path}") }
}

#[component]
pub fn Seo(
    /// Judul lengkap (≤ ±60 karakter, kata kunci di depan).
    #[prop(into)] title: String,
    /// Ringkasan 140–160 karakter.
    #[prop(into)] description: String,
    /// Path kanonik, mis. "/" atau "/tema/javanese-royal" (tanpa query).
    #[prop(into)] path: String,
    #[prop(optional, into)] image: Option<String>,
    /// og:type — "website" (bawaan) atau "article".
    #[prop(optional)] kind: Option<&'static str>,
) -> impl IntoView {
    let url = abs(&path);
    let img = abs(image.as_deref().filter(|s| !s.is_empty()).unwrap_or(OG_IMAGE));
    view! {
        <Title text=title.clone() />
        <Meta name="description" content=description.clone() />
        <Link rel="canonical" href=url.clone() />
        <Meta property="og:site_name" content=crate::brand!() />
        <Meta property="og:locale" content="id_ID" />
        <Meta property="og:type" content=kind.unwrap_or("website") />
        <Meta property="og:title" content=title.clone() />
        <Meta property="og:description" content=description.clone() />
        <Meta property="og:url" content=url />
        <Meta property="og:image" content=img.clone() />
        <Meta name="twitter:card" content="summary_large_image" />
        <Meta name="twitter:title" content=title />
        <Meta name="twitter:description" content=description />
        <Meta name="twitter:image" content=img />
    }
}

/// `<script type="application/ld+json">` — `</` di-escape agar isi teks tak
/// bisa menutup tag script. Bukan skrip yang dieksekusi → tak butuh nonce CSP.
#[component]
pub fn JsonLd(data: Value) -> impl IntoView {
    let s = serde_json::to_string(&data).unwrap_or_default().replace("</", "<\\/");
    view! { <script type="application/ld+json" inner_html=s></script> }
}

/// Organization + WebSite (beranda).
pub fn org_and_site() -> Value {
    json!({
        "@context": "https://schema.org",
        "@graph": [
            {
                "@type": "Organization",
                "@id": format!("{SITE_URL}/#org"),
                "name": crate::brand!(),
                "url": SITE_URL,
                "logo": abs("/img/monogram.jpg"),
                "parentOrganization": { "@type": "Organization", "name": PARENT_ORG },
                "areaServed": "ID",
                "knowsLanguage": "id"
            },
            {
                "@type": "WebSite",
                "@id": format!("{SITE_URL}/#website"),
                "url": SITE_URL,
                "name": crate::brand!(),
                "inLanguage": "id-ID",
                "publisher": { "@id": format!("{SITE_URL}/#org") }
            }
        ]
    })
}

/// Layanan undangan online + rentang harga paket (dari Konten admin, jadi
/// selalu sama dengan harga yang tampil).
pub fn service(prices: &[i64]) -> Value {
    let mut v = json!({
        "@context": "https://schema.org",
        "@type": "Service",
        "name": "Undangan Online Pernikahan",
        "serviceType": "Undangan digital / undangan website pernikahan",
        "provider": { "@id": format!("{SITE_URL}/#org") },
        "areaServed": { "@type": "Country", "name": "Indonesia" },
        "url": SITE_URL,
        "description": "Undangan online pernikahan dengan RSVP, buku tamu & QR check-in, amplop digital, musik latar, galeri foto, dan tema adat Nusantara."
    });
    let p: Vec<i64> = prices.iter().copied().filter(|p| *p > 0).collect();
    if let (Some(lo), Some(hi)) = (p.iter().min(), p.iter().max()) {
        v["offers"] = json!({ "@type": "AggregateOffer", "priceCurrency": "IDR", "lowPrice": lo, "highPrice": hi, "offerCount": p.len(), "url": abs("/paket") });
    }
    v
}

/// FAQPage dari pasangan (tanya, jawab) yang SAMA dengan yang tampil.
pub fn faq(items: &[(&str, String)]) -> Value {
    json!({
        "@context": "https://schema.org",
        "@type": "FAQPage",
        "mainEntity": items.iter().map(|(q, a)| json!({
            "@type": "Question", "name": q,
            "acceptedAnswer": { "@type": "Answer", "text": a }
        })).collect::<Vec<_>>()
    })
}

/// BreadcrumbList: [(nama, path)].
pub fn breadcrumb(items: &[(&str, &str)]) -> Value {
    json!({
        "@context": "https://schema.org",
        "@type": "BreadcrumbList",
        "itemListElement": items.iter().enumerate().map(|(i, (n, p))| json!({
            "@type": "ListItem", "position": i + 1, "name": n, "item": abs(p)
        })).collect::<Vec<_>>()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_terstruktur() {
        assert_eq!(abs("/paket"), "https://ilyvowcraft.online/paket");
        let s = service(&[79_000, 149_000, 249_000, 0]);
        assert_eq!((s["offers"]["lowPrice"].as_i64(), s["offers"]["highPrice"].as_i64()), (Some(79_000), Some(249_000)));
        assert!(service(&[]).get("offers").is_none());
        let f = faq(&[("Apa itu?", "Jawab".into())]);
        assert_eq!(f["mainEntity"][0]["acceptedAnswer"]["text"], "Jawab");
        assert_eq!(breadcrumb(&[("Beranda", "/"), ("Tema", "/tema/x")])["itemListElement"][1]["position"], 2);
    }
}
