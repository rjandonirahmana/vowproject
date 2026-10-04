//! pages/katalog.rs — beranda: katalog & pencarian tema, simulasi monogram,
//! paket harga. Filter = query string (?q=&kategori=&nuansa=&palet=&maks=&urut=)
//! sehingga jalan tanpa JS (form GET) dan bisa dibagikan sebagai tautan.

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;

use crate::web::components::{monogram_svg, Monogram};
use crate::web::fmt::{self, rupiah};
use crate::web::icons::Icon;
use crate::web::api::{get_konten, list_themes};
use crate::web::skin::{ThemeInfo, PALETTES};
use crate::web::konten::Konten;

use super::{SiteFooter, SiteHeader};

#[derive(Clone, Default, PartialEq)]
struct Filter {
    q: String,
    kategori: String,
    nuansa: Vec<String>,
    palet: String,
    urut: String,
    /// Jumlah kartu yang ditampilkan (bertambah lewat "Tampilkan lebih banyak").
    tampil: usize,
}

/// Kartu per "halaman" katalog.
const PER_HALAMAN: usize = 24;

impl Filter {
    fn matches(&self, t: &ThemeInfo) -> bool {
        let q = self.q.to_lowercase();
        let hay = format!("{} {} {} {} {} {} {}", t.name, t.description, t.region, t.tags.join(" "), t.palette, t.category, t.nuansa)
            .to_lowercase();
        (q.is_empty() || q.split_whitespace().all(|w| hay.contains(w)))
            && (self.kategori.is_empty() || t.category_key() == self.kategori)
            && (self.nuansa.is_empty() || self.nuansa.iter().any(|n| *n == t.nuansa_key()))
            && (self.palet.is_empty() || t.palette == self.palet)
    }

    /// Query string dengan satu kunci diganti (kosong = dihapus).
    fn with(&self, key: &str, val: &str) -> String {
        let mut parts = Vec::new();
        let mut push = |k: &str, v: &str| {
            if !v.is_empty() {
                parts.push(format!("{k}={}", fmt::url_encode(v)));
            }
        };
        push("q", if key == "q" { val } else { &self.q });
        push("kategori", if key == "kategori" { val } else { &self.kategori });
        if key != "nuansa" {
            for n in &self.nuansa {
                push("nuansa", n);
            }
        }
        push("palet", if key == "palet" { val } else { &self.palet });
        push("urut", if key == "urut" { val } else { &self.urut });
        // Ganti filter lain → kembali ke halaman pertama.
        if key == "tampil" {
            push("tampil", val);
        }
        if parts.is_empty() {
            "/#katalog".into()
        } else {
            format!("/?{}#katalog", parts.join("&"))
        }
    }
}

#[component]
pub fn KatalogPage() -> impl IntoView {
    let query = use_query_map();
    let filter = Memo::new(move |_| {
        let q = query.read();
        Filter {
            q: q.get("q").unwrap_or_default().chars().take(60).collect(),
            kategori: q.get("kategori").unwrap_or_default(),
            nuansa: q.get_all("nuansa").unwrap_or_default(),
            palet: q.get("palet").unwrap_or_default(),
            urut: q.get("urut").unwrap_or_default(),
            tampil: q.get("tampil").and_then(|n| n.parse().ok()).unwrap_or(PER_HALAMAN).clamp(PER_HALAMAN, 1000),
        }
    });
    let themes = Resource::new(|| (), |_| list_themes());
    let konten = Resource::new(|| (), |_| get_konten());
    // Banner beranda (tabel banners, /admin/banner).
    let banners = Resource::new(|| (), |_| crate::web::api::get_banners());
    let from = move || konten.get().and_then(|r| r.ok()).map(|k| k.min_price()).unwrap_or(0);
    let all = Signal::derive(move || themes.get().and_then(|r| r.ok()).unwrap_or_default());
    // Kartu tambahan yang dimuat di browser (infinite scroll / tombol) — TANPA
    // navigasi, jadi halaman tak melompat ke atas. `?tampil=` tetap jadi
    // cadangan tanpa JavaScript. Ganti filter → kembali ke halaman pertama.
    let extra = RwSignal::new(0usize);
    let shown = Memo::new(move |_| filter.get().tampil + extra.get());
    Effect::new(move |prev: Option<Filter>| {
        let f = filter.get();
        if let Some(p) = prev {
            if (&p.q, &p.kategori, &p.nuansa, &p.palet, &p.urut) != (&f.q, &f.kategori, &f.nuansa, &f.palet, &f.urut) {
                extra.set(0);
            }
        }
        f
    });
    // Penanda "sudah hydrate": muat otomatis hanya bila klik ditangani Leptos
    // (sebelum itu klik = tautan biasa ?tampil=…).
    let ready = RwSignal::new(false);
    Effect::new(move |_| ready.set(true));
    let results = Signal::derive(move || {
        let f = filter.get();
        let mut v: Vec<ThemeInfo> = all.get().into_iter().filter(|t| f.matches(t)).collect();
        if f.urut == "baru" {
            v.sort_by_key(|t| std::cmp::Reverse(t.created));
        }
        v
    });
    // (kunci, label, jumlah) dari data — kategori/nuansa baru muncul otomatis.
    let groups = move |field: fn(&ThemeInfo) -> &str| {
        Signal::derive(move || {
            let mut out: Vec<(String, String, usize)> = Vec::new();
            for t in all.get() {
                let label = field(&t).trim().to_string();
                if label.is_empty() {
                    continue;
                }
                let key = fmt::key(&label);
                match out.iter_mut().find(|g| g.0 == key) {
                    Some(g) => g.2 += 1,
                    None => out.push((key, label, 1)),
                }
            }
            out
        })
    };
    let categories = groups(|t| &t.category);
    let nuansa = groups(|t| &t.nuansa);

    view! {
        <Title text=concat!(crate::brand!(), " — Katalog Tema Undangan Pernikahan Digital") />
        <div class="site">
            <SiteHeader active="katalog" />

            // Banner promo lebar (Admin → Konten & Harga → Banner Beranda).
            <Suspense fallback=|| ()>
                {move || banners.get().and_then(|r| r.ok()).map(|items| view! { <BannerCarousel items=items /> })}
            </Suspense>

            <section class="k-hero">
                <span class="chip chip--soft"><Icon name="auto_awesome" />"Curated Atelier Collection 2026"</span>
                <h1>"Eksplorasi Tema Undangan Digital "<em>"Impian"</em>" Pernikahan Anda"</h1>
                <p>"Kurasi karya artisan berkelas dunia dengan sentuhan foil emas, tipografi adiluhung, serta adaptasi layar mobile dan tablet yang sempurna."</p>
                <form class="searchbar" method="get" action="/#katalog">
                    <Icon name="search" />
                    <input name="q" placeholder="Cari tema: adat jawa, sage green, modern…" prop:value=move || filter.get().q />
                    <button class="btn btn--primary btn--sm" type="submit">"Cari Tema"</button>
                </form>
                <div class="k-features">
                    <span><Icon name="visibility" />"Coba gratis, bayar saat siap disebar"</span>
                    <span><Icon name="bolt" />"Jadi 5 menit"</span>
                    <span><Icon name="library_music" />"Musik latar bebas"</span>
                    <span><Icon name="qr_code_2" />"Amplop & QR check-in"</span>
                    <span><Icon name="smartphone" />"Mobile optimal"</span>
                </div>
            </section>

            <Suspense fallback=|| view! { <div class="chips-row" id="katalog"></div> }>
                <div class="chips-row" id="katalog">
                    <a class="chip-link" class:is-on=move || filter.get().kategori.is_empty() href=move || filter.get().with("kategori", "")>
                        {move || format!("Semua ({})", all.get().len())}
                    </a>
                    <For each=move || categories.get() key=|g| g.clone() let:g>
                        {
                            let (k, label, n) = g;
                            let k2 = k.clone();
                            view! {
                                <a class="chip-link" class:is-on=move || filter.get().kategori == k href=move || filter.get().with("kategori", &k2)>
                                    {format!("{label} ({n})")}
                                </a>
                            }
                        }
                    </For>
                </div>
            </Suspense>

            <div class="k-layout">
                <aside class="k-filter card">
                    <form method="get" action="/#katalog">
                        <div class="k-filter__head">
                            <b><Icon name="tune" />"Filter Presisi"</b>
                            <a href="/#katalog">"Reset"</a>
                        </div>
                        <input type="hidden" name="q" prop:value=move || filter.get().q />
                        <input type="hidden" name="kategori" prop:value=move || filter.get().kategori />
                        <p class="eyebrow">"Nuansa & Budaya"</p>
                        // Daftar daerah bisa panjang (20+) → kotak bergulir sendiri.
                        <div class="k-filter__list">
                        <Suspense fallback=|| ()>
                            <For each=move || nuansa.get() key=|g| g.clone() let:g>
                                {
                                    let (k, label, n) = g;
                                    let k2 = k.clone();
                                    view! {
                                        <label class="check">
                                            <input type="checkbox" name="nuansa" value=k
                                                prop:checked=move || filter.get().nuansa.contains(&k2) />
                                            <span>{label}</span>
                                            <small>{n}</small>
                                        </label>
                                    }
                                }
                            </For>
                        </Suspense>
                        </div>
                        <p class="eyebrow">"Palet Warna Utama"</p>
                        <div class="palette">
                            {PALETTES.iter().map(|(k, label, hex)| view! {
                                <label class="palette__opt" title=*label>
                                    <input type="radio" name="palet" value=*k prop:checked=move || filter.get().palet == *k />
                                    <i style=format!("background:{hex}")></i>
                                    <small>{*label}</small>
                                </label>
                            }).collect_view()}
                        </div>
                        <p class="eyebrow">"Urutkan Koleksi"</p>
                        <select class="input" name="urut">
                            <option value="" selected=move || filter.get().urut.is_empty()>"Pilihan editor"</option>
                            <option value="baru" selected=move || filter.get().urut == "baru">"Terbaru"</option>
                        </select>
                        <Suspense fallback=|| ()>
                            <p class="k-filter__note"><Icon name="check_circle" />{move || format!("Semua tema termasuk di setiap paket — mulai {}", rupiah(from()))}</p>
                        </Suspense>
                        // Tombol menempel di bawah panel — selalu terjangkau tanpa scroll ke ujung.
                        <div class="k-filter__foot">
                            <button class="btn btn--primary btn--block btn--sm" type="submit"><Icon name="tune" />"Terapkan Filter"</button>
                        </div>
                    </form>
                </aside>

                <div class="k-results">
                    <Suspense fallback=|| view! { <div class="inv-loading"><div class="spinner"></div></div> }>
                        <p class="k-count">
                            <i class="dot"></i>
                            {move || {
                                let f = filter.get();
                                let n = results.get().len();
                                let shown = n.min(shown.get());
                                let of = if shown < n { format!("{shown} dari {n}") } else { n.to_string() };
                                if f.q.is_empty() { format!("Menampilkan {of} tema adiluhung") }
                                else { format!("Menampilkan {of} tema untuk kata kunci “{}”", f.q) }
                            }}
                        </p>
                        <div class="theme-grid">
                            <For
                                each=move || { let n = shown.get(); results.get().into_iter().take(n).enumerate().collect::<Vec<_>>() }
                                key=|(_, t)| t.slug.clone()
                                children=move |(i, t)| {
                                    // Jangkar kartu pertama tiap "halaman" → tombol lebih banyak tak melompat ke atas.
                                    let anchor = (i > 0 && i % PER_HALAMAN == 0).then(|| format!("k-{i}"));
                                    view! { <ThemeCard t=t from=from() anchor=anchor /> }
                                }
                            />
                        </div>
                        {move || {
                            let f = filter.get();
                            let n = shown.get();
                            let sisa = results.get().len().saturating_sub(n);
                            (sisa > 0).then(|| view! {
                                <div class="k-more">
                                    // data-load-more → skrip global (app.rs) mengekliknya otomatis
                                    // saat tombol mendekati layar.
                                    <a class="btn btn--soft"
                                        href=f.with("tampil", &(n + PER_HALAMAN).to_string()).replace("#katalog", &format!("#k-{n}"))
                                        data-load-more=move || ready.get().then_some("1")
                                        on:click=move |e| { e.prevent_default(); extra.update(|x| *x += PER_HALAMAN); }>
                                        <Icon name="expand_more" />{format!("Tampilkan lebih banyak ({sisa} tema lagi)")}
                                    </a>
                                </div>
                            })
                        }}
                        <Show when=move || results.get().is_empty()>
                            <div class="empty card">
                                <p>"Belum ada tema yang cocok dengan filter ini."</p>
                                <a class="btn btn--soft btn--sm" href="/#katalog">"Hapus Semua Filter"</a>
                            </div>
                        </Show>
                    </Suspense>
                </div>
            </div>

            <LivePreview />
            <super::layanan::WithKonten view=packages />

            <section class="guarantees">
                <div><Icon name="verified" /><b>"Garansi Uang Kembali 100%"</b><p>"Jika undangan digital Anda tidak memuaskan karena kendala teknis dalam 7 hari aktivasi."</p></div>
                <div><Icon name="support_agent" /><b>"Dukungan WhatsApp 24/7"</b><p>"Tim admin siap membantu input data, import kontak tamu, hingga gladi bersih sebar link."</p></div>
                <div><Icon name="favorite" /><b>"Masa Aktif Selamanya"</b><p>"Kisah cinta dan ucapan doa restu tamu tersimpan abadi tanpa biaya perpanjangan tahunan."</p></div>
            </section>

            <SiteFooter />
        </div>
    }
}

#[component]
pub fn ThemeCard(t: ThemeInfo, from: i64, #[prop(default = None)] anchor: Option<String>) -> impl IntoView {
    let rating = (!t.rating.is_empty()).then(|| {
        let r = if t.reviews.is_empty() { t.rating.clone() } else { format!("{} ({})", t.rating, t.reviews) };
        view! { <span class="tcard__rating"><Icon name="star" />{r}</span> }
    });
    view! {
        <article class="tcard card" id=anchor>
            <a class=format!("tcard__art th-{}", t.slug) href=format!("/tema/{}", t.slug) aria-label=format!("Demo {}", t.name)>
                {(!t.badge.is_empty()).then(|| view! { <span class="tcard__badge"><Icon name="star" />{t.badge.clone()}</span> })}
                <div class="mini">
                    <p class="mini__eyebrow">"The Wedding Of"</p>
                    <Monogram initials="A&R" class="monogram--sm" />
                    <p class="mini__names">"Yona & Doni"</p>
                    <p class="mini__date">"Sabtu, 24 Oktober 2026"</p>
                    <span class="mini__btn">"Buka Undangan"</span>
                </div>
                <span class="tcard__region">{t.region.clone()}</span>
                {rating}
                // Wadah pratinjau bergulir (iframe disisipkan global.js saat hover/sentuh).
                <div class="tcard__pv" data-pv=format!("/u/{}?tema={}&pv=1", crate::web::themes::DEMO_SLUG, t.slug) aria-hidden="true"></div>
            </a>
            <div class="tcard__body">
                <h3>{t.name.clone()}</h3>
                <p class="tcard__desc">{t.description.clone()}</p>
                <div class="tcard__tags">
                    {t.tags.iter().map(|tag| view! { <span class="tag">{tag.clone()}</span> }).collect_view()}
                </div>
                <div class="tcard__foot">
                    <div class="price">
                        <small>"Termasuk semua paket"</small>
                        <b>{format!("mulai {}", rupiah(from))}</b>
                    </div>
                    <div class="tcard__btns">
                        <a class="btn btn--soft btn--sm" href=format!("/tema/{}", t.slug)>"Demo"</a>
                        <a class="btn btn--primary btn--sm" href=format!("/buat?tema={}", t.slug)>"Pilih"</a>
                    </div>
                </div>
            </div>
        </article>
    }
}

#[component]
fn LivePreview() -> impl IntoView {
    let pria = RwSignal::new("Doni".to_string());
    let wanita = RwSignal::new("Yona".to_string());
    let tanggal = RwSignal::new("2026-10-24".to_string());
    let initials = move || {
        format!("{}&{}", crate::web::model::initial(&wanita.get()), crate::web::model::initial(&pria.get()))
    };
    let href = move || {
        format!(
            "/buat?wanita={}&pria={}&tanggal={}",
            fmt::url_encode(&wanita.get()),
            fmt::url_encode(&pria.get()),
            fmt::url_encode(&tanggal.get())
        )
    };
    view! {
        <section class="live card">
            <div class="live__form">
                <span class="chip chip--gold"><Icon name="auto_awesome" />{concat!("Fitur Eksklusif ", crate::brand!())}</span>
                <h2>"Coba Live Preview Monogram & Nama Anda"</h2>
                <p class="muted">"Ketik nama Anda dan pasangan untuk langsung melihat simulasi tampilan sampul undangan dengan monogram inisial otomatis."</p>
                <div class="field-row">
                    <label class="field">
                        <span class="field__label">"Mempelai Wanita"</span>
                        <input class="input" maxlength="30" prop:value=move || wanita.get() on:input=move |e| wanita.set(event_target_value(&e)) />
                    </label>
                    <label class="field">
                        <span class="field__label">"Mempelai Pria"</span>
                        <input class="input" maxlength="30" prop:value=move || pria.get() on:input=move |e| pria.set(event_target_value(&e)) />
                    </label>
                </div>
                <label class="field">
                    <span class="field__label">"Tanggal Pernikahan Rencana"</span>
                    <input class="input" type="date" prop:value=move || tanggal.get() on:input=move |e| tanggal.set(event_target_value(&e)) />
                </label>
                <a class="btn btn--primary" href=href>
                    <Icon name="arrow_forward" />
                    "Buat Undangan dengan Nama Ini"
                </a>
            </div>
            <div class="live__card th-botanical-heritage">
                <p class="mini__eyebrow">"The Wedding Of"</p>
                <div class="monogram monogram--md" inner_html=move || monogram_svg(&initials(), "THE WEDDING OF")></div>
                <p class="live__initials">{move || initials().replace('&', " & ")}</p>
                <h3>{move || format!("{} & {}", wanita.get(), pria.get())}</h3>
                <p class="mini__date">{move || fmt::tanggal_panjang(&tanggal.get()).to_uppercase()}</p>
                <div class="live__guest">
                    <small>"Kepada Yth. Bapak/Ibu/Saudara/i:"</small>
                    <b>"Tamu Kehormatan"</b>
                </div>
                <span class="btn btn--primary btn--sm"><Icon name="mail" />"Buka Undangan"</span>
            </div>
        </section>
    }
}

fn packages(k: Konten) -> impl IntoView {
    let kupon = k.active_coupon().map(|c| format!(" • kupon {} potong {}", c.code, rupiah(c.amount))).unwrap_or_default();
    view! {
        <section class="packages" id="paket">
            <p class="eyebrow eyebrow--center">"Transparan & Fleksibel"</p>
            <h2 class="section__title">"Pilihan Paket Undangan"</h2>
            <p class="muted center">"Satu kali bayar, aktif sesuai paket. Tanpa biaya tersembunyi."</p>
            <div class="pkg-grid">
                {k.paket.clone().into_iter().map(|p| view! {
                    <article class="pkg card" class:pkg--popular=p.popular>
                        {p.popular.then(|| view! { <span class="chip chip--gold pkg__flag">"Paling Populer"</span> })}
                        <h3>{p.name.clone()}</h3>
                        <p class="pkg__price">{rupiah(p.price)}</p>
                        <p class="pkg__period">{p.period.clone()}</p>
                        <ul>
                            {p.features.iter().map(|f| view! { <li><Icon name="check_circle" />{f.clone()}</li> }).collect_view()}
                        </ul>
                        <a class=if p.popular { "btn btn--primary btn--block" } else { "btn btn--soft btn--block" }
                            href=format!("/buat?paket={}", p.slug)>
                            {format!("Pilih {}", p.name)}
                        </a>
                    </article>
                }).collect_view()}
            </div>
            <a class="btn btn--outline btn--sm packages__more" href="/paket#fitur">"Bandingkan fitur lengkap tiap paket"<Icon name="arrow_forward" /></a>
            <p class="muted center small">{format!("Semua tema bisa dipakai di paket mana pun • coba & pratinjau gratis, bayar saat siap disebar{kupon}")}</p>
        </section>
    }
}

/// Carousel banner beranda: geser (scroll-snap, bisa diusap tanpa JS), titik &
/// tombol ‹ ›; berganti otomatis tiap 6 detik lewat skrip global (app.rs,
/// `[data-banner]`) — berhenti saat disentuh/di-hover & bila "kurangi animasi".
#[component]
fn BannerCarousel(items: Vec<crate::web::model::Banner>) -> impl IntoView {
    use crate::web::skin::is_safe_url;
    // Server sudah menyaring aktif & jadwal; saring URL sekali lagi.
    let items: Vec<_> = items.into_iter().filter(|b| is_safe_url(&b.img)).collect();
    let n = items.len();
    (n > 0).then(|| {
        view! {
            <section class="bn" data-banner="" aria-roledescription="carousel" aria-label="Promo">
                <div class="bn__track" data-banner-track="">
                    {items.into_iter().enumerate().map(|(i, b)| {
                        let hp = if is_safe_url(&b.img_hp) { b.img_hp.clone() } else { b.img.clone() };
                        // Tautan: path situs atau URL http(s) saja.
                        let link = Some(b.link.trim().to_string()).filter(|l| is_safe_url(l));
                        let first = i == 0;
                        view! {
                            <div class="bn__slide" role="group" aria-roledescription="slide" aria-label=format!("{} dari {n}", i + 1)>
                                <picture>
                                    <source media="(max-width: 720px)" srcset=hp />
                                    <img src=b.img.clone() alt="" decoding="async" loading=if first { "eager" } else { "lazy" } fetchpriority=if first { "high" } else { "auto" } />
                                </picture>
                                {link.clone().map(|l| view! { <a class="bn__hit" href=l aria-label=b.judul.clone() tabindex="-1"></a> })}
                                {(!b.judul.is_empty() || !b.sub.is_empty()).then(|| view! {
                                    <div class="bn__text">
                                        {(!b.judul.is_empty()).then(|| view! { <h2>{b.judul.clone()}</h2> })}
                                        {(!b.sub.is_empty()).then(|| view! { <p>{b.sub.clone()}</p> })}
                                        {link.clone().filter(|_| !b.cta.is_empty()).map(|l| view! {
                                            <a class="btn btn--gold bn__cta" href=l>{b.cta.clone()}<Icon name="arrow_forward" /></a>
                                        })}
                                    </div>
                                })}
                            </div>
                        }
                    }).collect_view()}
                </div>
                {(n > 1).then(|| view! {
                    <button type="button" class="bn__nav bn__nav--prev" data-banner-prev="" aria-label="Banner sebelumnya"><Icon name="chevron_left" /></button>
                    <button type="button" class="bn__nav bn__nav--next" data-banner-next="" aria-label="Banner berikutnya"><Icon name="chevron_right" /></button>
                    <div class="bn__dots">
                        {(0..n).map(|i| view! {
                            <button type="button" class="bn__dot" class:is-on=i == 0 data-banner-dot=i.to_string() aria-label=format!("Banner {}", i + 1)></button>
                        }).collect_view()}
                    </div>
                })}
            </section>
        }
    })
}
