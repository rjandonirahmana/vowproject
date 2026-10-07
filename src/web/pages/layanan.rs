//! pages/layanan.rs — potongan bersama halaman layanan pendukung
//! (/cetak, /dekorasi, /mua, /seserahan): judul seksi, kartu paket vendor, testimoni,
//! dan formulir reservasi yang dikirim ke WhatsApp lewat `GET /layanan/wa`.
//!
//! Semua formulir adalah `<form method=get>` biasa → tetap jalan tanpa WASM;
//! sinyal hanya memperkaya (pilihan paket dari `?paket=`, ringkasan live).

use leptos::prelude::*;
use crate::web::skeleton::*;
use leptos_router::hooks::use_query_map;

use crate::web::fmt::rupiah;
use crate::web::icons::Icon;
use crate::web::konten::{icon_line, Konten, Testimoni, VendorPaket};
use crate::web::layanan::vendor_price;

pub const WA_ENDPOINT: &str = "/layanan/wa";

/// Tautan WA tanpa isian formulir (tombol "Chat via WhatsApp").
pub fn wa_href(layanan: &str) -> String {
    format!("{WA_ENDPOINT}?layanan={layanan}")
}

/// Konten situs (paket, harga, foto) dari server; halaman merender isinya di
/// dalam `<Suspense>` dengan `view(Konten)`.
#[component]
pub fn WithKonten<V: IntoView + 'static>(view: fn(Konten) -> V) -> impl IntoView {
    let k = super::use_konten();
    view! {
        <Suspense fallback=|| view! { <SkelPage /> }>
            {move || k.get().map(|r| view(r.unwrap_or_default()))}
        </Suspense>
    }
}

/// Pilihan paket (slug) yang mengikuti `?paket=slug`. Saat query berubah lewat
/// navigasi di halaman (klik "Pilih Paket Ini"), sinyal ikut berubah dan
/// halaman menggulir ke `anchor`.
pub fn paket_dari_query(slugs: Vec<String>, default: String, anchor: &'static str) -> RwSignal<String> {
    let query = use_query_map();
    let slugs = std::sync::Arc::new(slugs);
    let lookup = {
        let slugs = slugs.clone();
        move |s: String| slugs.contains(&s).then_some(s)
    };
    let find = {
        let lookup = lookup.clone();
        move || query.with(|q| q.get("paket").and_then(|s| lookup(s)))
    };
    let sig = RwSignal::new(query.with_untracked(|q| q.get("paket").and_then(|s| lookup(s))).unwrap_or(default));
    Effect::new(move |prev: Option<()>| {
        if let Some(i) = find() {
            sig.set(i);
            if prev.is_some() {
                if let Some(el) = crate::web::fmt::document().and_then(|d| d.get_element_by_id(anchor)) {
                    el.scroll_into_view();
                }
            }
        }
    });
    sig
}

#[component]
pub fn SecHead(
    eyebrow: &'static str,
    title: &'static str,
    #[prop(optional)] lead: &'static str,
    #[prop(optional)] left: bool,
) -> impl IntoView {
    view! {
        <header class="lx-head" class:lx-head--left=left>
            <p class="eyebrow eyebrow--gold">{eyebrow}</p>
            <h2>{title}</h2>
            {(!lead.is_empty()).then(|| view! { <p class="lx-head__lead">{lead}</p> })}
        </header>
    }
}



#[component]
pub fn Quotes(items: Vec<Testimoni>) -> impl IntoView {
    view! {
        <div class="lx-quotes">
            {items.into_iter().enumerate().map(|(i, t)| {
                let initials: String = t.who.split(['&', ' ']).filter_map(|w| w.chars().next()).filter(|c| c.is_uppercase()).take(2).collect();
                view! {
                    <figure class="card lx-quote">
                        <span class="lx-stars" aria-label="Rating 5 dari 5">"★★★★★"</span>
                        <blockquote>{format!("“{}”", t.text)}</blockquote>
                        <figcaption>
                            <span class=format!("avatar avatar--{}", i % 3)>{initials}</span>
                            <span><b>{t.who.clone()}</b><small>{t.meta.clone()}</small></span>
                        </figcaption>
                    </figure>
                }
            }).collect_view()}
        </div>
    }
}

/// Kartu paket dekorasi / MUA. `href` = tujuan tombol (biasanya
/// `?paket=slug#reservasi` agar formulir terisi otomatis).
#[component]
pub fn VendorCard(
    p: VendorPaket,
    href: String,
    #[prop(optional)] wide: bool,
    /// Halaman detail paket (foto & judul jadi tautan + tombol "Lihat detail").
    #[prop(optional)]
    detail: Option<String>,
) -> impl IntoView {
    let price = vendor_price(&p);
    let dark = p.dark;
    let note = if p.note.is_empty() { "Investasi".to_string() } else { p.note.clone() };
    let cta = if p.cta.is_empty() { "Pilih Paket Ini".to_string() } else { p.cta.clone() };
    view! {
        <article class="card lx-vcard" class:lx-vcard--dark=dark class:lx-vcard--wide=wide>
            {match detail.clone() {
                Some(d) => view! {
                    <a class="lx-vcard__img" href=d aria-label=format!("Detail {}", p.name)>
                        <img src=p.img.clone() alt=p.name.clone() loading="lazy" decoding="async" />
                        {(!p.badge.is_empty()).then(|| view! { <span class="chip chip--gold lx-vcard__badge">{p.badge.clone()}</span> })}
                        {(!p.gallery.is_empty()).then(|| view! { <span class="lx-vcard__count"><Icon name="photo_library" />{format!("{} foto", p.gallery.len())}</span> })}
                    </a>
                }.into_any(),
                None => view! {
                    <div class="lx-vcard__img">
                        <img src=p.img.clone() alt=p.name.clone() loading="lazy" decoding="async" />
                        {(!p.badge.is_empty()).then(|| view! { <span class="chip chip--gold lx-vcard__badge">{p.badge.clone()}</span> })}
                    </div>
                }.into_any(),
            }}
            <div class="lx-vcard__body">
                <p class="eyebrow eyebrow--gold">{p.tag.clone()}</p>
                <h3>{match detail.clone() {
                    Some(d) => view! { <a href=d>{p.name.clone()}</a> }.into_any(),
                    None => view! { {p.name.clone()} }.into_any(),
                }}</h3>
                <p class="lx-vcard__desc">{p.desc.clone()}</p>
                <ul class="lx-feat-list">
                    {p.features.iter().map(|f| {
                        let (icon, text) = icon_line(f, "check_circle");
                        view! { <li><span class="ms" aria-hidden="true">{icon.to_string()}</span>{text.to_string()}</li> }
                    }).collect_view()}
                </ul>
                <div class="lx-vcard__foot">
                    <span class="lx-price">
                        <small>{note}</small>
                        <b>{price}</b>
                    </span>
                    <span class="lx-vcard__btns">
                        {detail.map(|d| view! { <a class="btn btn--soft btn--sm" href=d>"Lihat detail"</a> })}
                        <a class=if dark { "btn btn--gold btn--sm" } else { "btn btn--primary btn--sm" } href=href>
                            {cta}<Icon name="arrow_forward" />
                        </a>
                    </span>
                </div>
            </div>
        </article>
    }
}

/// Formulir pra-reservasi (MUA & dekorasi) → WhatsApp admin.
#[component]
pub fn ReservasiForm(
    layanan: &'static str,
    title: &'static str,
    lead: &'static str,
    nama_label: &'static str,
    nama_ph: &'static str,
    lokasi_label: &'static str,
    lokasi: &'static [(&'static str, &'static str)],
    pakets: Vec<VendorPaket>,
    /// Opsi paket tambahan di akhir daftar (slug, label).
    #[prop(optional)]
    extra: Option<(&'static str, &'static str)>,
    paket: RwSignal<String>,
    catatan_label: &'static str,
    submit: &'static str,
) -> impl IntoView {
    let init = paket.get_untracked();
    view! {
        <form class="card lx-form" action=WA_ENDPOINT method="get" target="_blank">
            <input type="hidden" name="layanan" value=layanan />
            <h3>{title}</h3>
            <p class="muted small">{lead}</p>
            <div class="field-row">
                <label class="field">
                    <span class="field__label">{nama_label}</span>
                    <input class="input" name="nama" required maxlength="80" placeholder=nama_ph autocomplete="name" />
                </label>
                <label class="field">
                    <span class="field__label">"Nomor WhatsApp aktif"</span>
                    <input class="input" name="wa" type="tel" required maxlength="20" placeholder="0812xxxxxxxx" inputmode="tel" autocomplete="tel" />
                </label>
            </div>
            <div class="field-row">
                <label class="field">
                    <span class="field__label">"Rencana tanggal acara"</span>
                    <input class="input" name="tanggal" type="date" />
                </label>
                <label class="field">
                    <span class="field__label">{lokasi_label}</span>
                    <select class="input" name="lokasi">
                        {lokasi.iter().map(|(v, l)| view! { <option value=*v>{*l}</option> }).collect_view()}
                    </select>
                </label>
            </div>
            <label class="field">
                <span class="field__label">"Pilihan paket"</span>
                <select class="input" name="paket" on:change=move |e| paket.set(event_target_value(&e))>
                    {pakets.into_iter().map(|p| {
                        let harga = if p.price_from { format!("mulai {}", rupiah(p.price)) } else { rupiah(p.price) };
                        let slug = p.slug.clone();
                        view! {
                            <option value=p.slug.clone() selected=p.slug == init prop:selected=move || paket.get() == slug>
                                {format!("{} ({harga})", p.name)}
                            </option>
                        }
                    }).collect_view()}
                    {extra.map(|(v, l)| view! { <option value=v>{l}</option> })}
                </select>
            </label>
            <label class="field">
                <span class="field__label">{catatan_label}</span>
                <textarea class="input" name="catatan" rows="3" maxlength="300"></textarea>
            </label>
            <button class="btn btn--primary btn--block btn--lg" type="submit"><Icon name="send" />{submit}</button>
            <p class="lx-form__note"><Icon name="lock" />"WhatsApp terbuka di tab baru dengan data Anda. Kami tidak mengirim spam."</p>
        </form>
    }
}
