//! pages/dekorasi.rs — /dekorasi: vendor dekorasi pernikahan & sound system
//! Tawangmangu – Solo Raya (desain Stitch "Vendor Dekorasi & Sound System").

use leptos::prelude::*;
use leptos_meta::{Meta, Title};

use crate::web::icons::Icon;
use crate::web::konten::{icon_line, Konten};
use crate::web::layanan::DEKOR_WILAYAH;

use super::layanan::{paket_dari_query, wa_href, ReservasiForm, SecHead, VendorCard, WithKonten};
use super::{SiteFooter, SiteHeader};

struct Poin {
    icon: &'static str,
    title: &'static str,
    text: &'static str,
}

const HERO_POIN: &[Poin] = &[
    Poin { icon: "location_on", title: "Cakupan wilayah lengkap", text: "Tawangmangu, Karanganyar Kota, Karangpandan, Ngargoyoso, Solo Raya" },
    Poin { icon: "power", title: "Keamanan tenaga acara", text: "Genset silent cadangan 30–50 kVA & crew standby penuh H-1" },
    Poin { icon: "verified", title: "Garansi akustik & bunga", text: "Bunga segar petik petani Lawu & sound check terkalibrasi" },
];

const SOP: &[Poin] = &[
    Poin { icon: "schedule", title: "Loading H-1 acara", text: "Rangka dekorasi & speaker terpasang 12 jam sebelum acara dimulai." },
    Poin { icon: "local_florist", title: "Fresh-cut petani lokal", text: "Mawar, krisan, dan hortensia dipanen subuh hari acara langsung dari Karanganyar." },
    Poin { icon: "badge", title: "Teknisi bersertifikat", text: "Sound engineer yang menguasai tuning akustik ruang & ketahanan angin outdoor." },
    Poin { icon: "graphic_eq", title: "Sound check gratis", text: "Gladi bersih vokal MC, ikrar ijab kabul, dan rehearsal band tanpa biaya tambahan." },
];

const SURVEY: &[Poin] = &[
    Poin { icon: "check_circle", title: "Pengukuran titik pelaminan", text: "Simulasi tata letak bunga, meja VIP, dan akses masuk pengantin." },
    Poin { icon: "check_circle", title: "Uji beban listrik & titik genset", text: "Mencegah listrik anjlok saat audio dan tata lampu menyala bersamaan." },
    Poin { icon: "check_circle", title: "Estimasi cuaca & arah angin lereng", text: "Saran mitigasi hujan dan kabut sore hari Tawangmangu." },
];

#[component]
pub fn DekorasiPage() -> impl IntoView {
    view! {
        <Title text=concat!("Dekorasi Pernikahan & Sound System Tawangmangu – Solo — ", crate::brand!()) />
        <Meta name="description" content="Dekorasi pernikahan botanical, adat Jawa Solo, dan intimate villa lengkap dengan sound system & genset silent. Survey lokasi gratis Tawangmangu, Karanganyar, Solo Raya." />
        <div class="site">
            <SiteHeader active="dekorasi" />
            <WithKonten view=dekor_body />
            <SiteFooter />
        </div>
    }
}

fn dekor_body(k: Konten) -> impl IntoView {
    let paket = paket_dari_query(
        k.dekor_paket.iter().map(|p| p.slug.clone()).collect(),
        k.dekor_paket.first().map(|p| p.slug.clone()).unwrap_or_default(),
        "reservasi",
    );
    let hero = if crate::web::skin::is_safe_url(&k.umum.dekor_hero) { k.umum.dekor_hero.clone() } else { String::new() };
    view! {
            <section class="lx-hero-dark" style=format!("--bg-img:url({hero})")>
                <div class="lx-hero-dark__in">
                    <span class="chip chip--gold"><Icon name="spa" />"Artisanal Event Stylist & Sound Acoustics • Solo Raya"</span>
                    <h1>"Dekorasi Pernikahan Impian & Sound System Berkualitas"</h1>
                    <p>"Estetika sakral nan megah di lereng sejuk Gunung Lawu, Tawangmangu, Karanganyar hingga Solo Raya. Harmoni visual botanical dan akustik jernih untuk outdoor pine forest, villa romantic garden, maupun gedung tradisional."</p>
                    <div class="lx-hero__btns">
                        <a class="btn btn--gold" href="#reservasi"><Icon name="calendar_month" />"Jadwalkan Survey Lokasi Gratis"</a>
                        <a class="btn btn--glass" href="#paket-dekor"><Icon name="expand_more" />"Eksplorasi Paket & Portofolio"</a>
                    </div>
                    <div class="lx-hero-dark__feats">
                        {HERO_POIN.iter().map(|k| view! {
                            <div>
                                <span class="ms" aria-hidden="true">{k.icon}</span>
                                <span><b>{k.title}</b><small>{k.text}</small></span>
                            </div>
                        }).collect_view()}
                    </div>
                </div>
            </section>

            <section class="lx-sec" id="paket-dekor">
                <div class="lx-head-row">
                    <SecHead left=true eyebrow="Kurasi Penataan Pesta" title="Pilihan Paket Dekorasi & Sound System" />
                    <p class="lx-head__lead lx-head__lead--side">"Setiap paket menyatu dengan iklim pegunungan Lawu: material tahan kelembapan tinggi dan tata suara standar panggung outdoor."</p>
                </div>
                <div class="lx-grid lx-grid--2">
                    {k.dekor_paket.clone().into_iter().map(|p| {
                        let href = format!("/dekorasi?paket={}#reservasi", p.slug);
                        let detail = format!("/dekorasi/{}", p.slug);
                        view! { <VendorCard p=p wide=true href=href detail=detail /> }
                    }).collect_view()}
                </div>
            </section>

            <section class="lx-sec">
                <SecHead eyebrow="Venue yang Kami Kuasai" title="Spotlight Venue Favorit di Lereng Lawu & Solo"
                    lead="Tim teknis kami memahami denah, kelistrikan, sirkulasi angin, dan resonansi akustik di venue-venue ikonik berikut." />
                <div class="lx-grid lx-grid--3">
                    {k.dekor_venue.clone().into_iter().map(|v| {
                        let (icon, spec) = icon_line(&v.spec, "verified_user");
                        let (icon, spec) = (icon.to_string(), spec.to_string());
                        view! {
                            <article class="card lx-venue">
                                <div class="lx-venue__img">
                                    <img src=v.img.clone() alt=v.name.clone() loading="lazy" decoding="async" />
                                    {(!v.area.is_empty()).then(|| view! { <span class="tcard__badge">{v.area.clone()}</span> })}
                                </div>
                                <div class="lx-venue__body">
                                    <h3>{v.name.clone()}</h3>
                                    <p>{v.desc.clone()}</p>
                                    {(!spec.is_empty()).then(|| view! { <p class="lx-venue__spec"><span class="ms" aria-hidden="true">{icon}</span>{spec}</p> })}
                                </div>
                            </article>
                        }
                    }).collect_view()}
                </div>
            </section>

            <section class="lx-band">
                <div class="lx-band__in">
                    <div>
                        <p class="eyebrow lx-cta__eyebrow">"Standard Operating Procedure"</p>
                        <h2>"Jaminan Ketenangan & Keamanan Acara"</h2>
                        <p>"Hari pernikahan adalah momen sekali seumur hidup. Kami menerapkan standar disiplin tanpa kompromi untuk seluruh tim lapangan."</p>
                    </div>
                    <div class="lx-band__grid">
                        {SOP.iter().map(|k| view! {
                            <div class="lx-band__item">
                                <span class="ms" aria-hidden="true">{k.icon}</span>
                                <b>{k.title}</b>
                                <small>{k.text}</small>
                            </div>
                        }).collect_view()}
                    </div>
                </div>
            </section>

            <section class="lx-sec">
                <SecHead eyebrow="Galeri Dokumentasi" title="Kemegahan Magis di Alam Terbuka"
                    lead="Saat kabut tipis Gunung Lawu berpadu hangat dengan cahaya fairy lights dan harum melati." />
                <div class="lx-mosaic">
                    {k.dekor_galeri.clone().into_iter().enumerate().map(|(i, g)| {
                        let cls = match i { 0 => "lx-mosaic__big", 3 => "lx-mosaic__wide", _ => "" };
                        view! {
                            <figure class=format!("lx-mosaic__item {cls}")>
                                <img src=g.img.clone() alt=g.judul.clone() loading="lazy" decoding="async" />
                                <figcaption><small>{g.kecil.clone()}</small><b>{g.judul.clone()}</b></figcaption>
                            </figure>
                        }
                    }).collect_view()}
                </div>
            </section>

            <section class="lx-sec lx-reserve" id="reservasi">
                <div class="lx-reserve__info">
                    <p class="eyebrow lx-cta__eyebrow"><Icon name="near_me" />"Layanan Eksklusif"</p>
                    <h2>"Survey Lokasi Gratis ke Tawangmangu & Solo"</h2>
                    <p>"Hindari kendala teknis di hari-H. Tim kami datang langsung ke venue untuk mengukur denah panggung, mengecek suplai listrik, dan menghitung pantulan suara."</p>
                    {SURVEY.iter().map(|k| view! {
                        <div class="lx-reserve__point">
                            <span class="ms" aria-hidden="true">{k.icon}</span>
                            <span><b>{k.title}</b><small>{k.text}</small></span>
                        </div>
                    }).collect_view()}
                    <a class="btn btn--gold" href=wa_href("dekorasi") target="_blank" rel="noopener"><Icon name="support_agent" />"Konsultasi WhatsApp Cepat"</a>
                </div>
                <ReservasiForm
                    layanan="dekorasi"
                    title="Jadwalkan Konsultasi & Booking Tanggal"
                    lead="Penawaran resmi dan jadwal survey kami kirim via WhatsApp."
                    nama_label="Nama calon pengantin / keluarga"
                    nama_ph="Contoh: Yona & Doni"
                    lokasi_label="Wilayah venue"
                    lokasi=DEKOR_WILAYAH
                    pakets=k.dekor_paket.clone()
                    extra=("custom", "Konsep khusus / konsultasi dulu")
                    paket=paket
                    catatan_label="Nama venue / catatan khusus (opsional)"
                    submit="Kirim Reservasi Survey via WhatsApp"
                />
            </section>

            <section class="lx-sec">
                <div class="lx-coverage">
                    <span><Icon name="local_shipping" />"Basecamp & gudang logistik di Karangpandan – Tawangmangu. Armada siap tanjakan Lawu hingga 1.800 mdpl; respon 24 jam untuk genset cadangan & florist touch-up."</span>
                    <a class="btn btn--primary btn--sm" href="#reservasi">"Booking Sekarang"</a>
                </div>
            </section>
    }
}

// ── /dekorasi/:slug — detail paket + galeri foto (dikelola admin) ────────────

#[component]
pub fn DekorasiDetailPage() -> impl IntoView {
    let params = leptos_router::hooks::use_params_map();
    let k = Resource::new(|| (), |_| crate::web::api::get_konten());
    view! {
        <div class="site">
            <SiteHeader active="dekorasi" />
            <Suspense fallback=|| view! { <div class="inv-loading"><div class="spinner"></div></div> }>
                {move || k.get().map(|r| {
                    let k = r.unwrap_or_default();
                    let slug = params.read().get("slug").unwrap_or_default();
                    match k.dekor_paket.iter().find(|p| p.slug == slug).cloned() {
                        Some(p) => {
                            let others: Vec<_> = k.dekor_paket.iter().filter(|o| o.slug != p.slug).cloned().collect();
                            view! { <DekorDetail p=p others=others /> }.into_any()
                        }
                        None => view! { <super::NotFoundPage /> }.into_any(),
                    }
                })}
            </Suspense>
            <SiteFooter />
        </div>
    }
}

#[component]
fn DekorDetail(p: crate::web::konten::VendorPaket, others: Vec<crate::web::konten::VendorPaket>) -> impl IntoView {
    use crate::web::layanan::vendor_price;
    // Galeri kosong → foto utama kartu.
    let photos: Vec<String> = if p.gallery.is_empty() { vec![p.img.clone()] } else { p.gallery.clone() };
    let n = photos.len();
    let photos = StoredValue::new(photos);
    let cur = RwSignal::new(0usize);
    let lightbox = RwSignal::new(false);
    let step = move |d: isize| cur.update(|c| *c = ((*c as isize + d).rem_euclid(n as isize)) as usize);
    let photo = move || photos.with_value(|v| v.get(cur.get()).cloned().unwrap_or_default());
    let reserve = format!("/dekorasi?paket={}#reservasi", p.slug);
    let paragraphs: Vec<String> = p.detail.split("\n\n").map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    let note = if p.note.is_empty() { "Investasi".to_string() } else { p.note.clone() };
    let first = photos.with_value(|v| v.first().cloned().unwrap_or_default());
    view! {
        <Title text=format!(concat!("{} — Dekorasi Pernikahan ", crate::brand!()), p.name) />
        <Meta name="description" content=p.desc.clone() />
        <Meta property="og:image" content=first />
        <nav class="dd-crumb" aria-label="Lokasi halaman">
            <a href="/dekorasi"><Icon name="arrow_back" />"Dekorasi & Sound"</a>
            <span>"/"</span>
            <span>{p.name.clone()}</span>
        </nav>
        <section class="dd">
            <div class="dd__gallery">
                <div class="dd__main">
                    <button type="button" class="dd__open" on:click=move |_| lightbox.set(true) aria-label="Perbesar foto">
                        <img src=photo alt=p.name.clone() fetchpriority="high" />
                    </button>
                    {(!p.badge.is_empty()).then(|| view! { <span class="chip chip--gold dd__badge">{p.badge.clone()}</span> })}
                    {(n > 1).then(|| view! {
                        <button type="button" class="dd__nav dd__nav--prev" on:click=move |_| step(-1) aria-label="Foto sebelumnya"><Icon name="chevron_left" /></button>
                        <button type="button" class="dd__nav dd__nav--next" on:click=move |_| step(1) aria-label="Foto berikutnya"><Icon name="chevron_right" /></button>
                        <span class="dd__count">{move || format!("{} / {n}", cur.get() + 1)}</span>
                    })}
                </div>
                {(n > 1).then(|| view! {
                    <div class="dd__thumbs">
                        {photos.get_value().into_iter().enumerate().map(|(i, src)| view! {
                            <button type="button" class="dd__thumb" class:is-on=move || cur.get() == i on:click=move |_| cur.set(i) aria-label=format!("Foto {}", i + 1)>
                                <img src=src alt="" loading="lazy" decoding="async" />
                            </button>
                        }).collect_view()}
                    </div>
                })}
            </div>
            <aside class="card dd__info">
                <p class="eyebrow eyebrow--gold">{p.tag.clone()}</p>
                <h1>{p.name.clone()}</h1>
                <p class="dd__desc">{p.desc.clone()}</p>
                <div class="dd__price">
                    <small>{note}</small>
                    <b>{vendor_price(&p)}</b>
                </div>
                <ul class="lx-feat-list dd__feats">
                    {p.features.iter().map(|f| {
                        let (icon, text) = icon_line(f, "check_circle");
                        view! { <li><span class="ms" aria-hidden="true">{icon.to_string()}</span>{text.to_string()}</li> }
                    }).collect_view()}
                </ul>
                <div class="dd__btns">
                    <a class="btn btn--primary btn--lg" href=reserve.clone()><Icon name="calendar_month" />{if p.cta.is_empty() { "Pesan Paket Ini".to_string() } else { p.cta.clone() }}</a>
                    <a class="btn btn--soft btn--lg" href=wa_href("dekorasi") target="_blank" rel="noopener"><Icon name="chat" />"Tanya via WhatsApp"</a>
                </div>
                <p class="muted small dd__survey"><Icon name="near_me" />"Survey lokasi gratis Tawangmangu – Solo Raya sebelum booking."</p>
            </aside>
        </section>

        {(!paragraphs.is_empty()).then(|| view! {
            <section class="lx-sec dd__about">
                <SecHead left=true eyebrow="Tentang Paket" title="Detail Konsep & Layanan" />
                {paragraphs.into_iter().map(|t| view! { <p>{t}</p> }).collect_view()}
            </section>
        })}

        {(n > 1).then(|| view! {
            <section class="lx-sec" id="galeri">
                <SecHead eyebrow="Galeri" title="Dokumentasi Paket Ini" lead="Klik foto untuk melihat ukuran penuh." />
                <div class="dd__grid">
                    {photos.get_value().into_iter().enumerate().map(|(i, src)| view! {
                        <button type="button" class="dd__cell" on:click=move |_| { cur.set(i); lightbox.set(true); } aria-label=format!("Perbesar foto {}", i + 1)>
                            <img src=src alt="" loading="lazy" decoding="async" />
                        </button>
                    }).collect_view()}
                </div>
            </section>
        })}

        {(!others.is_empty()).then(|| view! {
            <section class="lx-sec">
                <SecHead eyebrow="Pilihan Lain" title="Paket Dekorasi Lainnya" />
                <div class="lx-grid lx-grid--3">
                    {others.into_iter().map(|o| {
                        let href = format!("/dekorasi?paket={}#reservasi", o.slug);
                        let detail = format!("/dekorasi/{}", o.slug);
                        view! { <VendorCard p=o href=href detail=detail /> }
                    }).collect_view()}
                </div>
            </section>
        })}

        <Show when=move || lightbox.get()>
            <div class="dd-lightbox" role="dialog" aria-label="Foto ukuran penuh" on:click=move |_| lightbox.set(false)>
                <img src=photo alt="" on:click=|e| e.stop_propagation() />
                {(n > 1).then(|| view! {
                    <button type="button" class="dd__nav dd__nav--prev" on:click=move |e| { e.stop_propagation(); step(-1) } aria-label="Foto sebelumnya"><Icon name="chevron_left" /></button>
                    <button type="button" class="dd__nav dd__nav--next" on:click=move |e| { e.stop_propagation(); step(1) } aria-label="Foto berikutnya"><Icon name="chevron_right" /></button>
                })}
                <button type="button" class="dd-lightbox__close" aria-label="Tutup"><Icon name="close" /></button>
                <span class="dd__count">{move || format!("{} / {n}", cur.get() + 1)}</span>
            </div>
        </Show>
    }
}
