//! Halaman artikel editorial: daftar dan artikel terbit untuk SEO.

use leptos::prelude::*;
use leptos_meta::Meta;
use leptos_router::hooks::use_params_map;

use crate::web::api::{get_published_article, get_published_articles, get_related_articles};
use crate::web::icons::Icon;
use crate::web::model::Article;
use crate::web::seo::{JsonLd, Seo};

use super::{SiteFooter, SiteHeader};

#[component]
pub fn ArticlesPage() -> impl IntoView {
    let articles = Resource::new(|| (), |_| get_published_articles());
    let collection = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "CollectionPage",
        "name": "Artikel Pernikahan",
        "url": crate::web::seo::abs("/artikel"),
        "inLanguage": "id-ID"
    });
    view! {
        <Seo
            title=format!("Artikel Pernikahan & Undangan Digital — {}", crate::brand!())
            description="Temukan inspirasi pernikahan, panduan persiapan acara, serta tips membuat undangan digital yang personal dan mudah dibagikan."
            path="/artikel"
        />
        <JsonLd data=collection />
        <div class="site">
            <SiteHeader active="artikel" />
            <section class="info-hero article-index__hero">
                <span class="chip chip--soft"><Icon name="auto_stories" />"Jurnal & Inspirasi"</span>
                <h1>"Artikel Pernikahan"</h1>
                <p>"Panduan, inspirasi, dan cerita untuk membantu setiap persiapan terasa lebih tenang dan personal."</p>
            </section>
            <main class="article-index">
                <Suspense fallback=|| ()>
                    {move || articles.get().map(|result| match result {
                        Ok(items) if items.is_empty() => view! {
                            <div class="card article-empty">
                                <Icon name="auto_stories" />
                                <h2>"Artikel baru segera hadir"</h2>
                                <p>"Kami sedang menyiapkan inspirasi dan panduan pernikahan untuk Anda."</p>
                            </div>
                        }.into_any(),
                        Ok(items) => view! {
                            <div class="article-grid">
                                {items.into_iter().map(|article| view! { <ArticleCard article /> }).collect_view()}
                            </div>
                        }.into_any(),
                        Err(_) => view! {
                            <div class="card article-empty">
                                <h2>"Artikel belum dapat dimuat"</h2>
                                <p>"Silakan muat ulang halaman beberapa saat lagi."</p>
                            </div>
                        }.into_any(),
                    })}
                </Suspense>
            </main>
            <SiteFooter />
        </div>
    }
}

#[component]
fn ArticleCard(
    article: Article,
    /// Kartu di bawah judul h2 ("Baca juga") → judul kartu h3.
    #[prop(optional)] sub: bool,
) -> impl IntoView {
    let href = format!("/artikel/{}", article.slug);
    let judul = view! { <a href=href.clone()>{article.title.clone()}</a> };
    let judul = if sub { view! { <h3>{judul}</h3> }.into_any() } else { view! { <h2>{judul}</h2> }.into_any() };
    view! {
        <article class="card article-card">
            {(!article.cover_image.is_empty()).then(|| view! {
                <a class="article-card__image" href=href.clone() tabindex="-1" aria-hidden="true">
                    <img src=article.cover_image.clone() alt="" loading="lazy" decoding="async" />
                </a>
            })}
            <div class="article-card__body">
                <p class="article-date"><Icon name="calendar_month" />{tanggal(&article.published_at)}</p>
                {judul}
                <p>{article.excerpt.clone()}</p>
                <a class="article-read" href=format!("/artikel/{}", article.slug)>
                    "Baca artikel"<Icon name="arrow_forward" />
                </a>
            </div>
        </article>
    }
}

#[component]
pub fn ArticlePage() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.read().get("slug").unwrap_or_default();
    let article = Resource::new(slug, get_published_article);
    view! {
        <div class="site">
            <SiteHeader active="artikel" />
            <Suspense fallback=|| ()>
                {move || article.get().map(|result| match result {
                    Ok(article) => view! { <ArticleDetail article /> }.into_any(),
                    // get_published_article marks unknown and draft slugs as 404 on SSR.
                    Err(_) => view! {
                        <main class="article-not-found">
                            <Icon name="article" />
                            <h1>"Artikel tidak ditemukan"</h1>
                            <p>"Artikel ini tidak tersedia atau belum diterbitkan."</p>
                            <a class="btn btn--primary" href="/artikel">"Kembali ke artikel"</a>
                        </main>
                    }.into_any(),
                })}
            </Suspense>
            <SiteFooter />
        </div>
    }
}

#[component]
fn ArticleDetail(article: Article) -> impl IntoView {
    let path = format!("/artikel/{}", article.slug);
    let image = if article.cover_image.is_empty() { None } else { Some(article.cover_image.clone()) };
    let title = if article.seo_title.is_empty() {
        format!("{} — {}", article.title, crate::brand!())
    } else {
        article.seo_title.clone()
    };
    let description = if article.seo_description.is_empty() {
        article.excerpt.clone()
    } else {
        article.seo_description.clone()
    };
    let blok = urai_isi(&article.content);
    let author = if article.author.is_empty() { crate::brand!().to_string() } else { article.author.clone() };
    let image_url = crate::web::seo::abs(article.cover_image.as_str());
    let canonical = crate::web::seo::abs(&path);
    let structured = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "Article",
        "headline": article.title,
        "description": description,
        "datePublished": article.published_at,
        "dateModified": article.updated_at,
        // Penulis = nama merek → Organization (Person bernama merek menyesatkan
        // Google); nama orang sungguhan → Person.
        "author": if author == crate::brand!() {
            serde_json::json!({ "@type": "Organization", "name": author, "url": crate::web::seo::SITE_URL })
        } else {
            serde_json::json!({ "@type": "Person", "name": author })
        },
        "inLanguage": "id-ID",
        "publisher": { "@type": "Organization", "name": crate::brand!() },
        "mainEntityOfPage": { "@type": "WebPage", "@id": canonical },
        "image": if article.cover_image.is_empty() { crate::web::seo::abs(crate::web::seo::OG_IMAGE) } else { image_url }
    });
    let related = Resource::new({ let s = article.slug.clone(); move || s.clone() }, get_related_articles);
    let breadcrumb = crate::web::seo::breadcrumb(&[
        ("Beranda", "/"),
        ("Artikel", "/artikel"),
        (article.title.as_str(), path.as_str())
    ]);
    view! {
        <Seo title=title.clone() description=description.clone() path=path.clone() image=image.unwrap_or_default() kind="article" />
        <Meta property="article:published_time" content=article.published_at.clone() />
        <Meta property="article:modified_time" content=article.updated_at.clone() />
        <JsonLd data=structured />
        <JsonLd data=breadcrumb />
        <main class="article-detail">
            <nav class="article-crumb" aria-label="Lokasi halaman">
                <a href="/artikel"><Icon name="arrow_back" />"Semua artikel"</a>
                <span>"/"</span>
                <span>{article.title.clone()}</span>
            </nav>
            <article>
                <header class="article-detail__head">
                    <p class="article-date"><Icon name="calendar_month" />{tanggal(&article.published_at)}</p>
                    <h1>{article.title.clone()}</h1>
                    <p class="article-detail__excerpt">{article.excerpt.clone()}</p>
                    <p class="article-detail__byline">{format!("Ditulis oleh {}", author)}</p>
                </header>
                {(!article.cover_image.is_empty()).then(|| view! {
                    <figure class="article-detail__cover">
                        <img src=article.cover_image.clone() alt=article.title.clone() fetchpriority="high" />
                    </figure>
                })}
                <div class="article-detail__content">
                    {blok.into_iter().map(blok_view).collect_view()}
                </div>
                // Tautan internal ke halaman produk: artikel tanpa tautan keluar
                // = halaman buntu bagi pembaca & perayap.
                <aside class="article-cta">
                    <p class="article-cta__judul">"Siap membuat undangan pernikahan?"</p>
                    <p>"Pilih tema di katalog, isi data, dan lihat pratinjau gratis di HP — bayar hanya saat undangan siap disebar."</p>
                    <div class="article-cta__aksi">
                        <a class="btn btn--primary btn--sm" href="/buat">"Buat undangan online"</a>
                        <a class="btn btn--soft btn--sm" href="/#katalog">"Lihat katalog tema"</a>
                        <a class="btn btn--soft btn--sm" href="/paket">"Lihat paket & harga"</a>
                    </div>
                </aside>
            </article>
            // Artikel terkait: tanpa ini tiap artikel hanya ditautkan dari
            // /artikel (1 tautan masuk) dan pembaca berhenti di satu halaman.
            <Suspense fallback=|| ()>
                {move || related.get().and_then(|r| r.ok()).filter(|v| !v.is_empty()).map(|items| view! {
                    <section class="article-related" aria-labelledby="artikel-terkait">
                        <h2 id="artikel-terkait">"Baca juga"</h2>
                        <div class="article-grid">
                            {items.into_iter().map(|article| view! { <ArticleCard article sub=true /> }).collect_view()}
                        </div>
                        <p class="article-related__all"><a href="/artikel">"Semua artikel"<Icon name="arrow_forward" /></a></p>
                    </section>
                })}
            </Suspense>
        </main>
    }
}

/// Format `YYYY-MM-DDTHH:MM:SSZ` menjadi tanggal Indonesia tanpa dependensi tanggal WASM.
fn tanggal(iso: &str) -> String {
    let date = iso.split('T').next().unwrap_or("");
    let mut parts = date.split('-');
    let (Some(year), Some(month), Some(day)) = (parts.next(), parts.next(), parts.next()) else {
        return String::new();
    };
    let month = match month {
        "01" => "Januari",
        "02" => "Februari",
        "03" => "Maret",
        "04" => "April",
        "05" => "Mei",
        "06" => "Juni",
        "07" => "Juli",
        "08" => "Agustus",
        "09" => "September",
        "10" => "Oktober",
        "11" => "November",
        "12" => "Desember",
        _ => return String::new(),
    };
    format!("{day} {month} {year}")
}

// ── Isi artikel: Markdown ringan ────────────────────────────────────────────
// Dulu isi dipecah per baris kosong & semuanya jadi <p> → `## Subjudul` tampil
// sebagai teks mentah, artikel tanpa h2 & tanpa tautan. Pengurai kecil tanpa
// dependensi (WASM tetap ramping), keluarannya node Leptos (teks di-escape,
// bukan inner_html). Didukung: `##`/`###` subjudul (`#` dianggap `##` — h1
// hanya judul artikel), `- `/`* ` daftar, `1. ` daftar bernomor, `**tebal**`,
// `[teks](/tautan)` (hanya `/…` & `https://…`). Selain itu = paragraf.

enum Blok {
    Judul(u8, String),
    Para(String),
    Daftar(bool, Vec<String>),
}

fn urai_isi(isi: &str) -> Vec<Blok> {
    let isi = isi.replace("\r\n", "\n");
    let mut out: Vec<Blok> = Vec::new();
    for bagian in isi.split("\n\n") {
        let mut para: Vec<&str> = Vec::new();
        let mut daftar: Option<(bool, Vec<String>)> = None;
        let tutup = |out: &mut Vec<Blok>, para: &mut Vec<&str>, daftar: &mut Option<(bool, Vec<String>)>| {
            if !para.is_empty() {
                out.push(Blok::Para(para.join("\n")));
                para.clear();
            }
            if let Some((urut, item)) = daftar.take() {
                out.push(Blok::Daftar(urut, item));
            }
        };
        for baris in bagian.lines() {
            let t = baris.trim();
            if t.is_empty() {
                continue;
            }
            if let Some(h) = t.strip_prefix("### ") {
                tutup(&mut out, &mut para, &mut daftar);
                out.push(Blok::Judul(3, h.trim().to_string()));
            } else if let Some(h) = t.strip_prefix("## ").or_else(|| t.strip_prefix("# ")) {
                tutup(&mut out, &mut para, &mut daftar);
                out.push(Blok::Judul(2, h.trim().to_string()));
            } else if let Some((urut, item)) = butir(t) {
                if !para.is_empty() || daftar.as_ref().is_some_and(|(u, _)| *u != urut) {
                    tutup(&mut out, &mut para, &mut daftar);
                }
                daftar.get_or_insert_with(|| (urut, Vec::new())).1.push(item.to_string());
            } else if let (Some((_, item)), true) = (daftar.as_mut(), baris.starts_with([' ', '\t'])) {
                // Baris lanjutan butir daftar (diindentasi).
                if let Some(akhir) = item.last_mut() {
                    akhir.push(' ');
                    akhir.push_str(t);
                }
            } else {
                if daftar.is_some() {
                    tutup(&mut out, &mut para, &mut daftar);
                }
                para.push(t);
            }
        }
        tutup(&mut out, &mut para, &mut daftar);
    }
    out
}

/// `- x` / `* x` → (false, x); `12. x` → (true, x).
fn butir(t: &str) -> Option<(bool, &str)> {
    if let Some(x) = t.strip_prefix("- ").or_else(|| t.strip_prefix("* ")) {
        return Some((false, x.trim()));
    }
    let digit = t.bytes().take_while(u8::is_ascii_digit).count();
    if (1..=3).contains(&digit) {
        if let Some(x) = t[digit..].strip_prefix(". ") {
            return Some((true, x.trim()));
        }
    }
    None
}

fn blok_view(b: Blok) -> AnyView {
    match b {
        Blok::Judul(2, t) => view! { <h2>{sebaris(&t)}</h2> }.into_any(),
        Blok::Judul(_, t) => view! { <h3>{sebaris(&t)}</h3> }.into_any(),
        Blok::Para(t) => view! { <p>{sebaris(&t)}</p> }.into_any(),
        Blok::Daftar(false, item) => view! { <ul>{item.iter().map(|i| view! { <li>{sebaris(i)}</li> }).collect_view()}</ul> }.into_any(),
        Blok::Daftar(true, item) => view! { <ol>{item.iter().map(|i| view! { <li>{sebaris(i)}</li> }).collect_view()}</ol> }.into_any(),
    }
}

/// Inline: `**tebal**` & `[teks](href)`; sisanya teks biasa.
fn sebaris(s: &str) -> Vec<AnyView> {
    let mut out: Vec<AnyView> = Vec::new();
    let mut teks = String::new();
    let mut rest = s;
    while !rest.is_empty() {
        if let Some(r) = rest.strip_prefix("**") {
            if let Some(end) = r.find("**").filter(|e| *e > 0) {
                if !teks.is_empty() { out.push(std::mem::take(&mut teks).into_any()); }
                out.push(view! { <strong>{r[..end].to_string()}</strong> }.into_any());
                rest = &r[end + 2..];
                continue;
            }
        }
        if let Some(r) = rest.strip_prefix('[') {
            if let Some((label, href, sisa)) = r.split_once("](").and_then(|(label, after)| after.split_once(')').map(|(h, sisa)| (label, h.trim(), sisa))) {
                if !label.is_empty() && !label.contains(']') && (href.starts_with('/') && !href.starts_with("//") || href.starts_with("https://")) {
                    if !teks.is_empty() { out.push(std::mem::take(&mut teks).into_any()); }
                    let luar = href.starts_with("https://") && !href.starts_with(crate::web::seo::SITE_URL);
                    out.push(if luar {
                        view! { <a href=href.to_string() rel="noopener" target="_blank">{label.to_string()}</a> }.into_any()
                    } else {
                        view! { <a href=href.to_string()>{label.to_string()}</a> }.into_any()
                    });
                    rest = sisa;
                    continue;
                }
            }
        }
        let ch = rest.chars().next().unwrap();
        teks.push(ch);
        rest = &rest[ch.len_utf8()..];
    }
    if !teks.is_empty() { out.push(teks.into_any()); }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jenis(b: &[Blok]) -> Vec<String> {
        b.iter().map(|x| match x {
            Blok::Judul(n, t) => format!("h{n}:{t}"),
            Blok::Para(t) => format!("p:{t}"),
            Blok::Daftar(u, i) => format!("{}:{}", if *u { "ol" } else { "ul" }, i.join("|")),
        }).collect()
    }

    #[test]
    fn subjudul_dan_paragraf_dalam_satu_blok() {
        let b = urai_isi("Pembuka.\n\n## Siapkan barang\nMasukkan dokumen.\n\n### Kecil\n- a\n- b\n1. satu\n2. dua\nPenutup.");
        assert_eq!(jenis(&b), ["p:Pembuka.", "h2:Siapkan barang", "p:Masukkan dokumen.", "h3:Kecil", "ul:a|b", "ol:satu|dua", "p:Penutup."]);
    }

    #[test]
    fn h1_jadi_h2_dan_baris_lanjutan_daftar() {
        let b = urai_isi("# Judul\n- butir panjang\n  sambungan");
        assert_eq!(jenis(&b), ["h2:Judul", "ul:butir panjang sambungan"]);
    }

    #[test]
    fn butir_bukan_angka_tahun() {
        assert!(butir("2026. tahun").is_none());
        assert_eq!(butir("3. x"), Some((true, "x")));
    }
}
