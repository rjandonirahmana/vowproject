//! pages/undangan.rs — undangan untuk tamu: /u/:slug (Sampul), /acara, /rsvp.
//!
//! `InvitationLayout` memuat data SEKALI lalu merender header, nav bawah, dan
//! `<audio>` latar; tab anak berganti lewat `<Outlet/>` sehingga musik tetap
//! berputar saat pindah tab (navigasi SPA setelah hydrate).

use leptos::either::Either;
use leptos::prelude::*;
use crate::web::skeleton::*;
use leptos_meta::{Link, Meta, Title};
use leptos_router::components::{Outlet, A};
use leptos_router::hooks::{use_location, use_params_map, use_query_map};

use crate::web::api::{get_invitation, list_stories, my_stories, RequestStoryKey};
use crate::web::components::*;
use crate::web::icons::{qr_svg, Icon};
use crate::web::model::*;

use super::{ErrorCard, NotFoundPage};

#[derive(Clone)]
pub struct InvCtx {
    pub page: InvitationPage,
    /// Nama tamu dari `?to=` (tamu tanpa kode).
    pub to: String,
    /// Query yang dibawa antar tab ("?g=KODE" / "?to=…" / "").
    pub qs: String,
}

impl InvCtx {
    pub fn guest_name(&self) -> String {
        self.page.guest.as_ref().map(|g| g.name.clone()).filter(|n| !n.is_empty()).unwrap_or_else(|| {
            if self.to.is_empty() {
                "Tamu Undangan".into()
            } else {
                self.to.clone()
            }
        })
    }
    fn href(&self, tab: &str) -> String {
        format!("/u/{}{tab}{}", self.page.inv.slug, self.qs)
    }
}

#[component]
pub fn InvitationLayout() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let res = Resource::new(
        move || {
            let q = query.read();
            (params.read().get("slug").unwrap_or_default(), q.get("g"), q.get("k"), q.get("tema"), q.get("rupa"))
        },
        |(slug, g, k, tema, rupa)| get_invitation(slug, g, k, tema, rupa),
    );
    view! {
        <Suspense fallback=|| view! { <SkelInvitation /> }>
            {move || res.get().map(|r| match r {
                Ok(page) => {
                    let q = query.read_untracked();
                    let to: String = q.get("to").map(|t| t.chars().take(80).collect()).unwrap_or_default();
                    let mut parts: Vec<String> = Vec::new();
                    match (q.get("g"), to.is_empty()) {
                        (Some(g), _) => parts.push(format!("g={}", crate::web::fmt::url_encode(&g))),
                        (None, false) => parts.push(format!("to={}", crate::web::fmt::url_encode(&to))),
                        _ => {}
                    }
                    // Pratinjau pemilik: kunci TIDAK dibawa di URL — server
                    // membacanya dari cookie Kelola (server/owner.rs).
                    // Demo dengan tema pilihan: tema ikut dibawa antar tab.
                    if page.inv.is_demo {
                        if let Some(t) = q.get("tema").filter(|t| *t == page.inv.theme) {
                            parts.push(format!("tema={}", crate::web::fmt::url_encode(&t)));
                        }
                        if let Some(r) = q.get("rupa").filter(|_| !page.skin.rupa.is_empty()) {
                            parts.push(format!("rupa={}", crate::web::fmt::url_encode(&r)));
                        }
                    }
                    let qs = if parts.is_empty() { String::new() } else { format!("?{}", parts.join("&")) };
                    Either::Left(view! { <InvShell ctx=InvCtx { page, to, qs } /> })
                }
                Err(e) if err_msg(&e) == "NOT_FOUND" => Either::Right(Either::Left(view! { <NotFoundPage /> })),
                Err(e) if err_msg(&e) == "LOCKED" => Either::Right(Either::Right(view! { <LockedPage /> }.into_any())),
                Err(e) => Either::Right(Either::Right(view! { <ErrorCard msg=err_msg(&e) /> }.into_any())),
            })}
        </Suspense>
    }
}

#[component]
fn InvShell(ctx: InvCtx) -> impl IntoView {
    provide_context(ctx.clone());
    provide_ornaments(ctx.page.skin.ornaments.clone());
    let inv = ctx.page.inv.clone();
    let loc = use_location();
    let tabs = [("", "Sampul", "favorite"), ("/acara", "Acara", "event_available"), ("/rsvp", "Doa & RSVP", "mark_email_read"), ("/story", "Story", "photo_camera")];
    let base = format!("/u/{}", inv.slug);
    let tab_label = {
        let base = base.clone();
        move || {
            let p = loc.pathname.get();
            match p.strip_prefix(&base).unwrap_or("") {
                "/acara" => "Acara",
                "/rsvp" => "Doa dan RSVP",
                "/story" => "Story Tamu",
                _ => "Sampul",
            }
        }
    };
    let active = move |tab: &'static str| {
        let base = base.clone();
        move || loc.pathname.get().trim_end_matches('/') == format!("{base}{tab}")
    };
    let desc = format!("{} — {}. Kami mengundang Anda untuk hadir dan berbagi doa restu.", inv.couple(), inv.date_label());
    let skin = ctx.page.skin.clone();
    let single = skin.single;
    // Pratinjau tautan WhatsApp memakai foto sampul bila ada.
    let og_image = inv.cover_photo.split('#').next().unwrap_or("").to_string();
    // inv--rail: tata letak desktop ≥1100px (panel sampul kiri + kolom kanan).
    let video = bg_video_src(&skin.bg_video, &inv.video_url);
    let poster = Some(video_poster(&video, &inv.cover_photo));
    let root_class = format!(
        "inv inv--rail th-{}{}{}{}{}{}",
        inv.theme,
        skin.rupa,
        scroll_class(&skin.scroll_anim),
        if video.is_empty() { "" } else { " inv--video" },
        if single { " inv--single" } else { "" },
        if single && !skin.open_anim.is_empty() && skin.open_anim != "none" { " inv--gate" } else { "" }
    );
    view! {
        <Title text=format!("Undangan Pernikahan {}", inv.couple()) />
        <Meta name="description" content=desc.clone() />
        <Meta property="og:title" content=format!("The Wedding of {}", inv.couple()) />
        <Meta property="og:description" content=desc />
        <Meta name="robots" content="noindex, nofollow" />
        // CSS & huruf tema INI saja (bukan /tema.css semua tema) — shell
        // melewatkan tautan tema global di /u/… (app.rs theme_links).
        {(!skin.css.is_empty()).then(|| view! { <Link rel="stylesheet" href=skin.css.clone() /> })}
        {(!skin.fonts.is_empty()).then(|| view! { <Link rel="stylesheet" href=skin.fonts.clone() /> })}
        {(!og_image.is_empty()).then(|| view! { <Meta property="og:image" content=og_image.clone() /> })}
        <div class=root_class>
            <div class="inv__glow" aria-hidden="true"></div>
            <BgVideo src=video.clone() poster=poster.clone() />
            <FloatDeco kind=skin.float_deco.clone() />
            // Desktop ≥1100px: layar terbagi dua — panel sampul tetap di kiri,
            // isi undangan di kanan (CSS .inv--rail). Di HP panel ini disembunyikan.
            {
                let ev = inv.first_event().cloned().unwrap_or_default();
                let date_num = {
                    let p: Vec<&str> = ev.date.split('-').collect();
                    if p.len() == 3 { format!("{}.{}.{}", p[2], p[1], p[0]) } else { String::new() }
                };
                let gate_mode = single && !skin.open_anim.is_empty() && skin.open_anim != "none";
                let acara = if single { "#acara".to_string() } else { ctx.href("/acara") };
                view! {
                    <aside class="inv-side">
                        <div class="inv-side__top">
                            <span class="inv-side__chip">
                                <Monogram initials=inv.initials() class="monogram--xs" />
                                <span><b>"Undangan Pernikahan"</b><small>{format!("Kepada Yth. {}", ctx.guest_name())}</small></span>
                            </span>
                            {(!date_num.is_empty()).then(|| view! { <span class="inv-side__meta">{date_num}</span> })}
                        </div>
                        <div class="inv-side__text">
                            <p class="inv-side__eyebrow"><i aria-hidden="true"></i><span class="script">"The Wedding Of"</span></p>
                            <h2>{inv.bride_first()}" "<em>"&"</em>" "{inv.groom_first()}</h2>
                            <p class="inv-side__date">{inv.date_label()}</p>
                            <div class="inv-side__actions">
                                {gate_mode.then(|| view! {
                                    <button type="button" class="btn btn--lg inv-side__cta inv-side__cta--open" data-open="1">
                                        "Buka Lembaran Undangan"<Icon name="arrow_forward" />
                                    </button>
                                })}
                                <a class=if gate_mode { "btn btn--lg inv-side__cta inv-side__cta--acara" } else { "btn btn--lg inv-side__cta" } href=acara>
                                    "Lihat Rangkaian Acara"<Icon name="arrow_forward" />
                                </a>
                                // Judul lagu hanya untuk pemesan (/buat & Kelola) — tamu cukup tombol putar/jeda.
                                {(!inv.music_url.is_empty()).then(|| view! {
                                    <button type="button" class="inv-side__music" data-music="toggle" aria-label="Putar / jeda musik">
                                        <Icon name="album" />"Musik"
                                    </button>
                                })}
                            </div>
                        </div>
                        <div class="inv-side__foot">
                            <span>{ev.time_label()}</span>
                            <span>{ev.venue.clone()}</span>
                        </div>
                    </aside>
                }
            }
            // Belum dibayar: hanya pemilik (kunci Kelola) yang sampai ke sini.
            {ctx.page.preview.then(|| view! {
                <div class="inv-trial" role="note">
                    <Icon name="lock" />
                    <span>"Pratinjau pemilik — tamu BELUM bisa membuka undangan ini. Aktif setelah pembayaran dikonfirmasi admin."</span>
                </div>
            })}
            <header class="inv-top">
                <a class="inv-top__brand" href=ctx.href("")>
                    <Monogram initials=inv.initials() class="monogram--xs" />
                    <span>
                        <b>{inv.couple()}</b>
                        <small>{tab_label}</small>
                        // Desktop: kartu tamu seperti sampul surat.
                        <span class="inv-top__guest"><em>"Kepada Yth."</em>{ctx.guest_name()}</span>
                    </span>
                </a>
                <div class="inv-top__actions">
                    {(!inv.music_url.is_empty()).then(|| view! {
                        <button type="button" class="icon-btn disc" data-music="toggle" aria-label="Putar / jeda musik">
                            <Icon name="album" />
                        </button>
                    })}
                    {inv.is_demo.then(|| view! {
                        <a class="icon-btn icon-btn--dark" href=format!("/kelola/{}?key=demo&tema={}", inv.slug, crate::web::fmt::url_encode(&inv.theme)) rel="external" aria-label="Kelola">
                            <Icon name="person" />
                        </a>
                    })}
                </div>
            </header>
            <main class="inv__main">
                <Outlet />
            </main>
            <nav class="bottom-nav" aria-label="Navigasi undangan">
                {if single {
                    // Satu halaman: navigasi = lompat ke bagian. Dari tab Story
                    // (halaman terpisah) jangkar dibawa ke halaman utama.
                    let home = ctx.href("");
                    let on_story = move || loc.pathname.get().trim_end_matches('/').ends_with("/story");
                    view! {
                        {[("#sampul", "Sampul", "favorite"), ("#acara", "Acara", "event_available"), ("#rsvp", "Doa & RSVP", "mark_email_read")]
                            .into_iter()
                            .map(|(hash, label, icon)| {
                                let home = home.clone();
                                view! {
                                    <a href=move || if on_story() { format!("{home}{hash}") } else { hash.to_string() } class="bottom-nav__item">
                                        <span class="ms" aria-hidden="true">{icon}</span><span>{label}</span>
                                    </a>
                                }
                            })
                            .collect_view()}
                        <A href=ctx.href("/story") attr:class="bottom-nav__item" class:is-active=on_story>
                            <span class="ms" aria-hidden="true">"photo_camera"</span>
                            <span>"Story"</span>
                        </A>
                    }
                    .into_any()
                } else {
                    tabs.into_iter().map(|(tab, label, icon)| view! {
                        <A href=ctx.href(tab) attr:class="bottom-nav__item" class:is-active=active(tab)>
                            <span class="ms" aria-hidden="true">{icon}</span>
                            <span>{label}</span>
                        </A>
                    }).collect_view().into_any()
                }}
                {inv.is_demo.then(|| view! {
                    <a class="bottom-nav__item" href=format!("/kelola/{}?key=demo&tema={}", inv.slug, crate::web::fmt::url_encode(&inv.theme)) rel="external">
                        <Icon name="admin_panel_settings" />
                        <span>"Kelola"</span>
                    </a>
                })}
            </nav>
            <MusicAudio url=inv.music_url.clone() autoplay=inv.music_autoplay looped=inv.music_loop />
        </div>
    }
}

fn ctx() -> InvCtx {
    expect_context::<InvCtx>()
}

// ── Tab 1: Sampul ──────────────────────────────────────────────────────────

/// Nama panggilan, atau kata pertama nama lengkap.
fn short_name(nick: &str, full: &str) -> String {
    let n = nick.trim();
    if n.is_empty() { full.split_whitespace().next().unwrap_or("").to_string() } else { n.to_string() }
}

/// Sampul: foto berdua dalam bingkai lengkung (atau segel monogram).
#[component]
fn Cover() -> impl IntoView {
    let inv = ctx().page.inv;
    let names = couple_names(&inv);
    let photo = inv.cover_photo.clone();
    view! {
        <section class="hero cover orn-host" id="sampul">
            <Ornamen bagian="sampul" />
            <p class="script cover__eyebrow">"The Wedding Of"</p>
            {if photo.is_empty() {
                view! { <div class="hero__seal"><Monogram initials=inv.initials() caption="THE WEDDING OF" /></div> }.into_any()
            } else {
                view! {
                    <div class="arch-photo arch-photo--cover">
                        <span class="arch-photo__clip"><FotoGerak photos=cover_photos(&inv) alt=names.clone() priority=true /></span>
                        <span class="arch-photo__frame" aria-hidden="true"></span>
                    </div>
                }.into_any()
            }}
            <h1 class="cover__names">{names}</h1>
            <p class="hero__date"><Icon name="local_florist" />{inv.date_label()}<Icon name="local_florist" /></p>
        </section>
    }
}

#[component]
fn GuestCard() -> impl IntoView {
    let c = ctx();
    let vip = c.page.guest.as_ref().is_some_and(|g| matches!(g.category.as_str(), "vip" | "keluarga"));
    view! {
        <section class="guest card">
            {vip.then(|| view! { <span class="chip chip--gold"><Icon name="verified" />"Tamu Spesial / VIP"</span> })}
            <p class="muted">"Kepada Yth. Bapak/Ibu/Saudara/i:"</p>
            <h2 class="guest__name">{c.guest_name()}</h2>
            <p class="guest__text">"Tanpa mengurangi rasa hormat, kami bermaksud mengundang Anda untuk hadir dan berbagi doa restu di hari bahagia kami."</p>
            <p class="guest__note">"*Mohon maaf bila ada kesalahan penulisan nama / gelar."</p>
        </section>
    }
}

/// Isi pembuka setelah sampul (dipakai mode tab & satu halaman).
#[component]
fn SampulExtras() -> impl IntoView {
    let c = ctx();
    let inv = c.page.inv;
    // Video pasangan; undangan demo tema sinema memamerkan video bawaan tema.
    let prewed = if !inv.video_url.is_empty() { inv.video_url.clone() } else if inv.is_demo { c.page.skin.bg_video.clone() } else { String::new() };
    view! {
        <Countdown target_ms=inv.countdown_target_ms() date=inv.date_label() calendar=inv.calendar_link() photos=save_date_photos(&inv) />
        <QuoteCard text=inv.quote_text.clone() source=inv.quote_source.clone() />
        <Couple inv=inv.clone() />
        <LoveStorySection items=inv.love_story.clone() photos=inv.gallery.iter().skip(1).cloned().collect() />
        <PreweddingVideo src=prewed poster=Some(inv.cover_photo.split('#').next().unwrap_or("").to_string()) />
        <Gallery photos=inv.gallery.clone() />
    }
}

#[component]
pub fn SampulPage() -> impl IntoView {
    let c = ctx();
    let single = c.page.skin.single;
    let anim = c.page.skin.open_anim.clone();
    let gate = single && !anim.is_empty() && anim != "none";
    let opening = if gate {
        // Sampul = gerbang layar penuh; tombol membuka tirai (skrip global:
        // data-open → html.inv-opened) sekaligus memutar musik.
        view! {
            <div class=format!("gate gate--{anim}")>
                <div class="gate__panel gate__panel--l" aria-hidden="true"></div>
                <div class="gate__panel gate__panel--r" aria-hidden="true"></div>
                <div class="gate__orn" aria-hidden="true"></div>
                <GateFx />
                {(!c.page.skin.open_video.is_empty()).then(|| {
                    let bg = c.page.skin.bg_video.clone();
                    let poster = video_poster(&bg, "");
                    view! { <GateBg src=bg poster=Some(poster) /> }
                })}
                <GateVideo src=c.page.skin.open_video.clone() />
                <div class="gate__content">
                    <Cover />
                    <GuestCard />
                    <button type="button" class="btn btn--gold btn--lg gate__open" data-open="1">
                        <Icon name="drafts" />"Buka Undangan"
                    </button>
                </div>
            </div>
        }
        .into_any()
    } else if single {
        view! {
            <Cover />
            <GuestCard />
            <div class="stack">
                <a href="#isi" class="btn btn--gold btn--block btn--lg" data-open="1"><Icon name="drafts" />"Buka Undangan & Putar Musik"</a>
            </div>
        }
        .into_any()
    } else {
        view! {
            <Cover />
            <GuestCard />
            <div class="stack">
                <A href=c.href("/acara") attr:class="btn btn--gold btn--block btn--lg" attr:data-open="1">
                    <Icon name="drafts" />
                    "Buka Undangan & Putar Musik"
                </A>
            </div>
        }
        .into_any()
    };
    view! {
        {opening}
        <div id="isi" class="inv__isi">
            <SampulExtras />
            {single.then(|| view! {
                <div id="acara" class="inv__anchor"><AcaraBody countdown=false /></div>
                <div id="rsvp" class="inv__anchor"><RsvpBody /></div>
            })}
        </div>
    }
}

// ── Tab 2: Acara ───────────────────────────────────────────────────────────

#[component]
pub fn AcaraPage() -> impl IntoView {
    view! { <AcaraBody countdown=true /> }
}

#[component]
fn AcaraBody(countdown: bool) -> impl IntoView {
    let inv = ctx().page.inv;
    let last = inv.events.last().cloned();
    let n = inv.events.len();
    let dress = (inv.dress_code.clone(), inv.dress_colors.clone());
    view! {
        <section class="intro orn-host">
            <Ornamen bagian="acara" />
            <span class="intro__icon"><Icon name="local_florist" /></span>
            <p class="eyebrow eyebrow--gold">"Walimatul 'Ursy"</p>
            <h1 class="section__title">"Wedding Event"</h1>
            <p class="intro__text">"Dengan memohon rahmat dan ridho Allah SWT, kami mengundang Anda untuk merayakan ikatan suci kami:"</p>
        </section>
        {countdown.then(|| view! { <Countdown target_ms=inv.countdown_target_ms() title="Menghitung Hari Bahagia" date=inv.date_label() calendar=inv.calendar_link() /> })}
        {inv.events.clone().into_iter().enumerate().map(|(i, ev)| {
            // Dress code ditempel di acara terakhir (biasanya resepsi).
            let d = (i + 1 == n).then(|| dress.clone());
            view! { <EventCard ev=ev dress=d /> }
        }).collect_view()}
        {last.map(|ev| view! { <MapCard ev=ev /> })}
        <LiveStream url=inv.live_url.clone() />
        <GuestGuide />
        <figure class="closing">
            <Icon name="local_florist" />
            <blockquote>"“Semoga Allah memberkahi engkau dalam kebaikan dan menghimpun kalian berdua dalam kebahagiaan.”"</blockquote>
            <figcaption>"HR. Abu Dawud"</figcaption>
        </figure>
    }
}

// ── Tab 3: Doa & RSVP ──────────────────────────────────────────────────────

pub fn session_options(inv: &Invitation) -> Vec<String> {
    let mut out = Vec::new();
    for ev in &inv.events {
        if ev.sessions.is_empty() {
            let t = ev.time_start.replace(':', ".");
            out.push(if t.is_empty() { ev.title.clone() } else { format!("{} ({t} WIB)", ev.title) });
        } else {
            for s in &ev.sessions {
                out.push(format!("{} ({})", s.label, s.time));
            }
        }
    }
    if out.is_empty() {
        out.push("Sesi Utama".into());
    }
    out
}

#[component]
pub fn RsvpPage() -> impl IntoView {
    view! { <RsvpBody /> }
}

#[component]
fn RsvpBody() -> impl IntoView {
    let c = ctx();
    let inv = c.page.inv.clone();
    let refresh = RwSignal::new(0u32);
    let guest = c.page.guest.clone();
    let mut prefill = guest.clone();
    if prefill.is_none() && !c.to.is_empty() {
        prefill = Some(GuestInfo { name: c.to.clone(), pax: 1, ..Default::default() });
    }
    view! {
        <section class="rsvp-hero card card--soft orn-host">
            <Ornamen bagian="rsvp" />
            <span class="intro__icon"><Icon name="favorite" /></span>
            <p class="eyebrow eyebrow--gold">"Buku Tamu Digital"</p>
            <h1 class="section__title">"Wedding Wishes"</h1>
            <p class="intro__text">"Kehadiran dan doa restu Anda merupakan kado terindah bagi kebahagiaan kami berdua."</p>
        </section>
        {inv.is_demo.then(|| view! {
            <p class="notice notice--info">"Ini undangan demo — formulir hanya contoh dan tidak disimpan."</p>
        })}
        <RsvpForm
            slug=inv.slug.clone()
            guest=prefill.clone()
            sessions=session_options(&inv)
            on_done=Callback::new(move |_| refresh.update(|n| *n += 1))
        />
        {guest.as_ref().filter(|g| !g.code.is_empty()).map(|g| view! {
            <section class="checkin card">
                <div class="checkin__qr" inner_html=qr_svg(&g.code)></div>
                <div>
                    <p class="eyebrow eyebrow--gold"><Icon name="qr_code_2" />"QR Check-in Resepsi"</p>
                    <b class="checkin__code">{g.code.clone()}</b>
                    <p class="muted">
                        {if g.table_no.is_empty() { "Tunjukkan kode ini di meja penerima tamu.".to_string() }
                         else { format!("Meja {} • tunjukkan kode ini di meja penerima tamu.", g.table_no) }}
                    </p>
                </div>
            </section>
        })}
        <GiftSection inv=inv.clone() guest=prefill />
        <WishList slug=inv.slug.clone() refresh=refresh />
        <TerimaKasih photos=closing_photos(&inv) names=couple_names(&inv) />
    }
}

fn couple_names(inv: &Invitation) -> String {
    format!("{} & {}", short_name(&inv.bride_nick, &inv.bride_name), short_name(&inv.groom_nick, &inv.groom_name))
}

/// Sampul: foto sampul lalu foto galeri pertama (silang-pudar, ala everlove).
fn cover_photos(inv: &Invitation) -> Vec<String> {
    std::iter::once(inv.cover_photo.clone()).chain(inv.gallery.first().cloned()).collect()
}

/// Save The Date: foto galeri ke-3 & ke-4 (galeri pendek → sampul).
fn save_date_photos(inv: &Invitation) -> Vec<String> {
    let g: Vec<String> = inv.gallery.iter().skip(2).take(2).cloned().collect();
    if g.is_empty() { vec![inv.cover_photo.clone()] } else { g }
}

/// Penutup: dua foto galeri terakhir (galeri ke-2.. dipakai love story lebih
/// dulu); galeri kosong → foto sampul saja.
fn closing_photos(inv: &Invitation) -> Vec<String> {
    match inv.gallery.len() {
        0 => vec![inv.cover_photo.clone()],
        1 => vec![inv.gallery[0].clone()],
        n => inv.gallery[n - 2..].to_vec(),
    }
}

// ── Tab 4: Story tamu ──────────────────────────────────────────────────────

/// Story tamu ala Instagram (penampil = global.js, sama perilakunya dengan
/// story e-ticketing). Menambah story: nomor WA → kunci spesial via WA →
/// kunci + SATU foto (POST multipart /u/{slug}/story/kirim, jalan tanpa WASM).
#[component]
pub fn StoryPage() -> impl IntoView {
    let c = ctx();
    let inv = c.page.inv.clone();
    let slug = inv.slug.clone();
    let query = use_query_map();
    let notice = move || {
        let q = query.read();
        q.get("ok").map(|m| (true, m)).or_else(|| q.get("galat").map(|m| (false, m)))
    };
    let stories = Resource::new({
        let slug = slug.clone();
        move || slug.clone()
    }, list_stories);
    // Galat dari unggahan (redirect ?galat=) → formulir langsung terbuka.
    // Story milik perangkat ini — diambil di klien setelah hydrate (cookie
    // HttpOnly; tak boleh mempengaruhi markup SSR).
    // Sinyal biasa (bukan Resource): dibaca di dalam <Suspense> tanpa membuat
    // SSR menunggu — Effect hanya berjalan di klien.
    let mine_ids = RwSignal::new(Vec::<i64>::new());
    Effect::new({
        let slug = slug.clone();
        move |_| {
            let slug = slug.clone();
            leptos::task::spawn_local(async move {
                if let Ok(v) = my_stories(slug).await {
                    mine_ids.set(v);
                }
            });
        }
    });
    let is_mine = move |id: i64| mine_ids.with(|v| v.contains(&id));
    // "41,57" — dibaca penampil (global.js) untuk tombol hapus story sendiri.
    let mine_attr = move || {
        let ids: Vec<String> = mine_ids.with(|v| v.iter().map(|i| i.to_string()).collect());
        ids.join(",")
    };
    let show_add = RwSignal::new(query.read_untracked().get("galat").is_some());
    let req = ServerAction::<RequestStoryKey>::new();
    // Nomor hasil normalisasi server (62…) → langkah 2.
    let phone = move || req.value().get().and_then(|r| r.ok());
    let del_action = format!("/u/{}/story/hapus", inv.slug);
    let mine_action = format!("/u/{}/story/hapus-saya", inv.slug);
    let req_err = move || req.value().get().and_then(|r| r.err()).map(|e| err_msg(&e));
    let back = c.href("/story");
    // Salinan per closure (masing-masing `move`).
    let (back_grid, back_del) = (back.clone(), back.clone());
    let action = format!("/u/{}/story/kirim", inv.slug);
    let demo = inv.is_demo;
    let (slug_in, name_in) = (inv.slug.clone(), c.to.clone());
    // "Story Anda" = mau menambah story: buka formulir lalu gulir ke sana
    // (formulirnya di bawah daftar story — dulu terbuka tanpa terlihat).
    let open_add = move || {
        show_add.set(true);
        request_animation_frame(|| {
            if let Some(el) = document().get_element_by_id("tambah-story") {
                el.scroll_into_view_with_bool(true);
            }
        });
    };
    view! {
        <section class="section story-hero orn-host">
            <Ornamen bagian="rsvp" />
            <p class="eyebrow eyebrow--gold eyebrow--center">"Momen Para Tamu"</p>
            <h1 class="section__title">"Guest Stories"</h1>
            <p class="intro__text">"Bagikan satu foto terbaikmu untuk kedua mempelai — semua tamu bisa melihatnya seperti story."</p>
        </section>
        {move || notice().map(|(ok, m)| view! {
            <p class=if ok { "notice notice--ok" } else { "notice notice--err" }>{m}</p>
        })}
        <Suspense fallback=|| view! { <div class="story-bar"><div class="story-item"><div class="story-shim-ring"></div></div></div> }>
            {move || stories.get().map(|r| {
                let list = r.unwrap_or_default();
                let json = serde_json::to_string(&list).unwrap_or_else(|_| "[]".into()).replace("</", "<\\/");
                let n = list.len();
                view! {
                    <div class="story-bar" data-story-slug=slug.clone()>
                        <div class="story-item">
                            <button type="button" class="story-add-btn" on:click=move |_| open_add() aria-label="Tambah story">
                                <span class="story-avatar-ring story-avatar-ring--add"><span class="story-avatar-inner"><Icon name="add" /></span></span>
                                <span class="story-username story-username--add">"Story Anda"</span>
                            </button>
                        </div>
                        {list.iter().enumerate().map(|(i, s)| view! {
                            <div class="story-item">
                                <button type="button" class="story-user-btn" data-story-open=i.to_string() data-story-id=s.id.to_string() aria-label=format!("Lihat story {}", s.name)>
                                    <span class="story-avatar-ring"><img class=format!("story-avatar-img sf-{}", s.filter) src=s.photo.clone() alt="" loading="lazy" decoding="async" /></span>
                                    <span class="story-username">{s.name.clone()}</span>
                                </button>
                            </div>
                        }).collect_view()}
                    </div>
                    {if n == 0 {
                        Either::Left(view! { <p class="muted center story-empty">"Belum ada story. Jadilah tamu pertama yang berbagi momen!"</p> })
                    } else {
                        Either::Right(view! {
                            <div class="story-grid">
                                {list.into_iter().enumerate().map(|(i, s)| {
                                    let id = s.id;
                                    let (mine_action, back) = (mine_action.clone(), back_grid.clone());
                                    view! {
                                    <div class="story-tile-wrap">
                                        <button type="button" class="story-tile" data-story-open=i.to_string() data-story-id=s.id.to_string()>
                                            <img class=format!("sf-{}", s.filter) src=s.photo alt=format!("Story {}", s.name) loading="lazy" decoding="async" />
                                            <span class="story-tile__who"><b>{s.name}</b><small>{s.ago}</small></span>
                                            <span class="story-tile__mine" class:is-on=move || is_mine(id)>"Story Anda"</span>
                                        </button>
                                        <form method="post" action=mine_action class="story-tile__del" class:is-on=move || is_mine(id) data-confirm="Hapus story Anda secara permanen?">
                                            <input type="hidden" name="id" value=id.to_string() />
                                            <input type="hidden" name="back" value=back />
                                            <button type="submit" class="icon-btn icon-btn--sm" aria-label="Hapus story saya"><Icon name="delete" /></button>
                                        </form>
                                    </div>
                                }}).collect_view()}
                            </div>
                        })
                    }}
                    <script type="application/json" id="story-data" inner_html=json data-del=mine_action.clone() data-back=back_grid.clone() data-mine=mine_attr></script>
                }
            })}
        </Suspense>

        <section class="card story-add" class:is-open=move || show_add.get() id="tambah-story">
            <h2 class="story-add__title"><Icon name="add_photo_alternate" />"Tambah Story"</h2>
            {demo.then(|| view! { <p class="notice notice--info">"Ini undangan demo — story di atas hanya contoh."</p> })}
            <ol class="story-steps">
                <li class:is-done=move || phone().is_some()>"Nomor WhatsApp"</li>
                <li class:is-on=move || phone().is_some()>"Kunci & Foto"</li>
            </ol>
            {move || phone().is_none().then(|| {
                let (slug_in, name_in) = (slug_in.clone(), name_in.clone());
                view! {
                <ActionForm action=req attr:class="story-form">
                    <input type="hidden" name="slug" value=slug_in />
                    <label class="field">
                        <span class="field__label">"Nama Anda"</span>
                        <input class="input" name="name" required maxlength="60" value=name_in placeholder="Nama yang tampil di story" />
                    </label>
                    <label class="field">
                        <span class="field__label">"Nomor WhatsApp"</span>
                        <input class="input" name="phone" type="tel" inputmode="tel" required maxlength="20" placeholder="0812 3456 7890" />
                    </label>
                    <p class="muted small">"Kami kirim kunci spesial 6 digit ke WhatsApp ini. Satu nomor hanya bisa membuat satu story."</p>
                    {move || req_err().map(|e| view! { <p class="notice notice--err">{e}</p> })}
                    <button class="btn btn--primary btn--block" type="submit" disabled=move || req.pending().get()>
                        <Icon name="send" />
                        {move || if req.pending().get() { "Mengirim kunci…" } else { "Kirim Kunci ke WhatsApp" }}
                    </button>
                </ActionForm>
                }
            })}
            {move || phone().filter(|k| k.has_story).map(|k| view! {
                <form class="story-form" method="post" action=del_action.clone() data-confirm="Hapus story Anda secara permanen?">
                    <input type="hidden" name="phone" value=k.phone.clone() />
                    <input type="hidden" name="back" value=back_del.clone() />
                    <p class="notice notice--info">{format!("Nomor +{} sudah punya story di undangan ini. Kunci sudah dikirim ke WhatsApp — masukkan untuk MENGHAPUS story Anda (setelah itu bisa membuat yang baru).", k.phone)}</p>
                    <label class="field">
                        <span class="field__label">"Kunci Spesial (6 digit)"</span>
                        <input class="input story-key" name="key" required inputmode="numeric" pattern="[0-9]{6}" maxlength="6" autocomplete="one-time-code" placeholder="••••••" />
                    </label>
                    <button class="btn btn--danger btn--block" type="submit"><Icon name="delete" />"Hapus Story Saya Permanen"</button>
                </form>
            })}
            {move || phone().filter(|k| !k.has_story).map(|k| k.phone).map(|p| view! {
                <form class="story-form" method="post" action=action.clone() enctype="multipart/form-data">
                    <input type="hidden" name="phone" value=p.clone() />
                    <input type="hidden" name="back" value=back.clone() />
                    <p class="notice notice--ok">{format!("Kunci sudah dikirim ke WhatsApp +{p}.")}</p>
                    <label class="field">
                        <span class="field__label">"Kunci Spesial (6 digit)"</span>
                        <input class="input story-key" name="key" required inputmode="numeric" pattern="[0-9]{6}" maxlength="6" autocomplete="one-time-code" placeholder="••••••" />
                    </label>
                    <label class="story-pick">
                        <img class="story-pick__img sf-normal" data-story-preview alt="" hidden />
                        <span class="story-pick__hint"><Icon name="photo_camera" /><b>"Pilih satu foto"</b><small>"JPEG / PNG / WebP, maks 5 MB — video tidak bisa"</small></span>
                        <input type="file" name="foto" accept="image/jpeg,image/png,image/webp" required data-story-file />
                    </label>
                    <div class="story-filters" role="radiogroup" aria-label="Filter foto">
                        {STORY_FILTERS.iter().map(|(k, label)| view! {
                            <label class="story-filter">
                                <input type="radio" name="filter" value=*k checked=*k == "normal" data-story-filter />
                                <span class=format!("story-filter__thumb sf-{k}")></span>
                                <small>{*label}</small>
                            </label>
                        }).collect_view()}
                    </div>
                    <label class="field">
                        <span class="field__label">"Keterangan (opsional)"</span>
                        <input class="input" name="caption" maxlength="150" placeholder="Tulis ucapan singkat…" />
                    </label>
                    <button class="btn btn--primary btn--block" type="submit"><Icon name="cloud_upload" />"Terbitkan Story"</button>
                </form>
            })}
        </section>
    }
}

/// Layar untuk tamu yang membuka undangan belum aktif — tanpa data undangan
/// apa pun (server tidak mengirimkannya).
#[component]
fn LockedPage() -> impl IntoView {
    view! {
        <Title text="Undangan belum aktif" />
        <Meta name="robots" content="noindex, nofollow" />
        <div class="inv inv-locked">
            <div class="inv__glow" aria-hidden="true"></div>
            <section class="card inv-locked__card">
                <span class="inv-locked__icon"><Icon name="lock" /></span>
                <h1>"Undangan belum aktif"</h1>
                <p>"Undangan ini sedang disiapkan oleh kedua mempelai. Silakan buka kembali tautan ini beberapa saat lagi."</p>
                <p class="muted small">"Pemilik undangan? Buka pratinjau dari halaman Kelola Anda, lalu selesaikan pembayaran agar tamu bisa membukanya."</p>
            </section>
        </div>
    }
}
