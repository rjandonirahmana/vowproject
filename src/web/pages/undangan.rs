//! pages/undangan.rs — undangan untuk tamu: /u/:slug (Sampul), /acara, /rsvp.
//!
//! `InvitationLayout` memuat data SEKALI lalu merender header, nav bawah, dan
//! `<audio>` latar; tab anak berganti lewat `<Outlet/>` sehingga musik tetap
//! berputar saat pindah tab (navigasi SPA setelah hydrate).

use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::components::{Outlet, A};
use leptos_router::hooks::{use_location, use_params_map, use_query_map};

use crate::web::api::get_invitation;
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
        move || (params.read().get("slug").unwrap_or_default(), query.read().get("g"), query.read().get("k"), query.read().get("tema")),
        |(slug, g, k, tema)| get_invitation(slug, g, k, tema),
    );
    view! {
        <Suspense fallback=|| view! { <div class="inv-loading"><div class="spinner"></div></div> }>
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
    let tabs = [("", "Sampul", "favorite"), ("/acara", "Acara", "event_available"), ("/rsvp", "Doa & RSVP", "mark_email_read")];
    let base = format!("/u/{}", inv.slug);
    let tab_label = {
        let base = base.clone();
        move || {
            let p = loc.pathname.get();
            match p.strip_prefix(&base).unwrap_or("") {
                "/acara" => "Acara",
                "/rsvp" => "Doa dan RSVP",
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
    let root_class = format!(
        "inv inv--rail th-{}{}{}",
        inv.theme,
        if single { " inv--single" } else { "" },
        if single && !skin.open_anim.is_empty() && skin.open_anim != "none" { " inv--gate" } else { "" }
    );
    view! {
        <Title text=format!("Undangan Pernikahan {}", inv.couple()) />
        <Meta name="description" content=desc.clone() />
        <Meta property="og:title" content=format!("The Wedding of {}", inv.couple()) />
        <Meta property="og:description" content=desc />
        <Meta name="robots" content="noindex, nofollow" />
        {(!og_image.is_empty()).then(|| view! { <Meta property="og:image" content=og_image.clone() /> })}
        <div class=root_class>
            <div class="inv__glow" aria-hidden="true"></div>
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
                                {(!inv.music_url.is_empty() && !inv.music_label().is_empty()).then(|| view! {
                                    <button type="button" class="inv-side__music" data-music="toggle" aria-label="Putar / jeda musik">
                                        <Icon name="album" />{inv.music_label()}
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
                    // Satu halaman: navigasi = lompat ke bagian.
                    [("#sampul", "Sampul", "favorite"), ("#acara", "Acara", "event_available"), ("#rsvp", "Doa & RSVP", "mark_email_read")]
                        .into_iter()
                        .map(|(href, label, icon)| view! {
                            <a href=href class="bottom-nav__item"><span class="ms" aria-hidden="true">{icon}</span><span>{label}</span></a>
                        })
                        .collect_view()
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
    let names = format!("{} & {}", short_name(&inv.bride_nick, &inv.bride_name), short_name(&inv.groom_nick, &inv.groom_name));
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
                        <span class="arch-photo__clip"><img src=photo.clone() alt=names.clone() style=crate::web::fmt::photo_style(&photo) fetchpriority="high" /></span>
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
    let inv = c.page.inv.clone();
    let vip = c.page.guest.as_ref().is_some_and(|g| matches!(g.category.as_str(), "vip" | "keluarga"));
    view! {
        <section class="guest card">
            {vip.then(|| view! { <span class="chip chip--gold"><Icon name="verified" />"Tamu Spesial / VIP"</span> })}
            <p class="muted">"Kepada Yth. Bapak/Ibu/Saudara/i:"</p>
            <h2 class="guest__name">{c.guest_name()}</h2>
            <p class="guest__text">"Tanpa mengurangi rasa hormat, kami bermaksud mengundang Anda untuk hadir dan berbagi doa restu di hari bahagia kami."</p>
            <p class="guest__note">"*Mohon maaf bila ada kesalahan penulisan nama / gelar."</p>
            {(!inv.music_label().is_empty()).then(|| view! {
                <span class="pill"><Icon name="music_note" />{inv.music_label()}</span>
            })}
        </section>
    }
}

/// Isi pembuka setelah sampul (dipakai mode tab & satu halaman).
#[component]
fn SampulExtras() -> impl IntoView {
    let inv = ctx().page.inv;
    view! {
        <Countdown target_ms=inv.countdown_target_ms() />
        <QuoteCard text=inv.quote_text.clone() source=inv.quote_source.clone() />
        <Couple inv=inv.clone() />
        <LoveStorySection items=inv.love_story.clone() />
        <Gallery photos=inv.gallery.clone() />
    }
}

#[component]
pub fn SampulPage() -> impl IntoView {
    let c = ctx();
    let inv = c.page.inv.clone();
    let single = c.page.skin.single;
    let anim = c.page.skin.open_anim.clone();
    let gate = single && !anim.is_empty() && anim != "none";
    let calendar = view! {
        <a class="btn btn--soft btn--block" href=inv.calendar_link() target="_blank" rel="noopener">
            <Icon name="calendar_add_on" />
            "Simpan ke Google Calendar"
        </a>
    };
    let opening = if gate {
        // Sampul = gerbang layar penuh; tombol membuka tirai (skrip global:
        // data-open → html.inv-opened) sekaligus memutar musik.
        view! {
            <div class=format!("gate gate--{anim}")>
                <div class="gate__panel gate__panel--l" aria-hidden="true"></div>
                <div class="gate__panel gate__panel--r" aria-hidden="true"></div>
                <div class="gate__orn" aria-hidden="true"></div>
                <div class="gate__content">
                    <Cover />
                    <GuestCard />
                    <button type="button" class="btn btn--gold btn--lg gate__open" data-open="1">
                        <Icon name="drafts" />"Buka Undangan"
                    </button>
                </div>
            </div>
            <div class="stack">{calendar}</div>
        }
        .into_any()
    } else if single {
        view! {
            <Cover />
            <GuestCard />
            <div class="stack">
                <a href="#isi" class="btn btn--gold btn--block btn--lg" data-open="1"><Icon name="drafts" />"Buka Undangan & Putar Musik"</a>
                {calendar}
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
                {calendar}
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
            <h1 class="section__title">"Rangkaian Hari Bahagia"</h1>
            <p class="intro__text">"Dengan memohon rahmat dan ridho Allah SWT, kami mengundang Anda untuk merayakan ikatan suci kami:"</p>
        </section>
        {countdown.then(|| view! { <Countdown target_ms=inv.countdown_target_ms() title="Menghitung Hari Bahagia" /> })}
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
            <h1 class="section__title">"Konfirmasi Kehadiran & Doa Restu"</h1>
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
