//! Halaman artikel editorial: daftar dan artikel terbit untuk SEO.

use leptos::prelude::*;
use leptos_meta::Meta;
use leptos_router::hooks::use_params_map;

use crate::web::api::{get_published_article, get_published_articles};
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
fn ArticleCard(article: Article) -> impl IntoView {
    let href = format!("/artikel/{}", article.slug);
    view! {
        <article class="card article-card">
            {(!article.cover_image.is_empty()).then(|| view! {
                <a class="article-card__image" href=href.clone() tabindex="-1" aria-hidden="true">
                    <img src=article.cover_image.clone() alt="" loading="lazy" decoding="async" />
                </a>
            })}
            <div class="article-card__body">
                <p class="article-date"><Icon name="calendar_month" />{tanggal(&article.published_at)}</p>
                <h2><a href=href>{article.title.clone()}</a></h2>
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
    let paragraphs: Vec<String> = article.content
        .split("\n\n")
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(str::to_string)
        .collect();
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
        "author": { "@type": "Person", "name": author },
        "publisher": { "@type": "Organization", "name": crate::brand!() },
        "mainEntityOfPage": { "@type": "WebPage", "@id": canonical },
        "image": if article.cover_image.is_empty() { crate::web::seo::abs(crate::web::seo::OG_IMAGE) } else { image_url }
    });
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
                    {paragraphs.into_iter().map(|paragraph| view! { <p>{paragraph}</p> }).collect_view()}
                </div>
            </article>
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
