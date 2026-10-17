//! pages/tema.rs — /tema/:tema: pratinjau interaktif satu tema memakai data
//! undangan demo, plus panel musik latar (putar/pilih lagu bawaan), keunggulan
//! tema, dan harga. Desktop = dua kolom (undangan + panel), mobile = bertumpuk.

use leptos::either::Either;
use leptos::prelude::*;
use crate::web::skeleton::*;
use leptos_meta::Meta;
use crate::web::seo::{JsonLd, Seo};
use leptos_router::hooks::use_params_map;

use crate::web::api::{get_contact, get_demo, get_related_themes, get_theme, list_songs};
use crate::web::components::*;
use crate::web::fmt::rupiah;
use crate::web::icons::Icon;
use crate::web::model::Invitation;
use crate::web::themes::DEMO_SLUG;
use crate::web::skin::ThemeInfo;

use super::{ErrorCard, SiteFooter, SiteHeader};

#[component]
pub fn TemaPage() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.read().get("tema").unwrap_or_default();
    let demo = Resource::new(|| (), |_| get_demo());
    let theme = Resource::new(slug, get_theme);
    let related = Resource::new(slug, get_related_themes);
    let konten = super::use_konten();
    let from = move || konten.get().and_then(|r| r.ok()).map(|k| k.min_price()).unwrap_or(0);
    view! {
        <div class="site">
            <SiteHeader />
            <Suspense fallback=|| view! { <SkelDetail phone=true /> }>
            {move || theme.get().map(|r| {
                let Some(t): Option<ThemeInfo> = r.ok().flatten() else {
                    return Either::Left(view! { <super::NotFoundPage /> });
                };
                let slug = t.slug.clone();
                provide_ornaments(t.ornaments.clone());
                // Demo selalu mulai TERTUTUP seperti yang dialami tamu; tema tanpa
                // animasi pembuka memakai efek "pudar".
                let anim = if t.open_anim.is_empty() || t.open_anim == "none" { "pudar".to_string() } else { t.open_anim.clone() };
                let jenis = if t.ulang_tahun() { "Undangan Ulang Tahun" } else { "Undangan Online" };
                let h1 = format!("{jenis} Tema {}", t.name);
                let label: Vec<String> = [&t.category, &t.nuansa, &t.region].into_iter().filter(|s| !s.trim().is_empty()).cloned().collect();
                let title = crate::web::seo::judul_pas(&[
                    format!(concat!("{} — Demo Gratis | ", crate::brand!()), h1),
                    format!("{h1} — Demo Gratis"),
                    h1.clone(),
                ]);
                let ciri = [t.nuansa.trim(), t.category.trim()].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(", ");
                let description = format!(
                    "Demo gratis {} tema {}{}: {} RSVP, QR check-in, amplop digital & musik.",
                    jenis.to_lowercase(), t.name, if ciri.is_empty() { String::new() } else { format!(" ({ciri})") },
                    t.description.trim().trim_end_matches('.').to_string() + "."
                );
                Either::Right(view! {
                    <Seo title=title description=description path=format!("/tema/{}", t.slug) image=t.image_url.clone() />
                    // CSS & huruf tema INI saja (/gaya: variabel, animasi, rupa
                    // ±100 KB) — dulu /tema.css semua tema (±450 KB, memblokir
                    // render ±2,5 dtk di HP). <link> di body (sah di HTML5):
                    // ikut terpasang juga lewat navigasi klien, markup identik
                    // SSR/klien. Tema templat dirender di iframe /u/… yang memuat
                    // CSS-nya sendiri → tak perlu apa-apa di sini.
                    {t.template.is_empty().then(|| view! {
                        <link rel="stylesheet" href=format!("/gaya/{}.css", t.slug) />
                        {let f = crate::web::skin::font_families([&t]);
                         (!f.is_empty()).then(|| view! { <link rel="stylesheet" href=crate::web::fmt::font_css(&f, "swap") /> })}
                    })}
                    // Tema privat (pesanan custom) tak boleh muncul di Google.
                    {(!t.listed).then(|| view! { <Meta name="robots" content="noindex, nofollow" /> })}
                    <JsonLd data=crate::web::seo::breadcrumb(&[("Katalog Tema", "/"), (t.name.as_str(), format!("/tema/{}", t.slug).as_str())]) />
                    <header class="tema-head">
                        <nav class="tema-crumb" aria-label="Lokasi halaman">
                            <a href="/#katalog"><Icon name="arrow_back" />"Katalog Tema"</a>
                            <span aria-hidden="true">"/"</span>
                            <span aria-current="page">{t.name.clone()}</span>
                        </nav>
                        <h1>{h1}</h1>
                        {(!t.description.trim().is_empty()).then(|| view! { <p class="tema-head__lead">{t.description.clone()}</p> })}
                        <ul class="tema-head__tags" aria-label="Ciri tema">
                            {label.into_iter().map(|l| view! { <li class="chip chip--soft">{l}</li> }).collect_view()}
                            {(!t.listed).then(|| view! { <li class="chip chip--gold">"Tema privat"</li> })}
                        </ul>
                    </header>
                    <div class="demo-bar">
                        <span class="chip chip--live"><i class="dot dot--live"></i>"Live Interactive Preview"</span>
                        <a class="btn btn--soft btn--sm demo-bar__full" href=format!("/u/{}?tema={}", DEMO_SLUG, t.slug) target="_blank" rel="noopener">
                            <Icon name="open_in_new" />
                            "Coba Demo Layar Penuh"
                        </a>
                        <a class="btn btn--soft btn--sm" href=format!("/kelola/{}?key=demo&tema={}", DEMO_SLUG, t.slug) target="_blank" rel="noopener">
                            <Icon name="dashboard" />
                            "Demo Dashboard"
                        </a>
                        <a class="btn btn--primary btn--sm" href=format!("/buat?tema={}", t.slug)>
                            <Icon name="shopping_bag" />
                            "Gunakan Tema Ini"
                        </a>
                    </div>
                    <div class="demo-layout">
                        {if t.template.is_empty() {
                            Either::Left(view! {
                                <div class="demo-phone">
                                    // Hiasan melayang tema — menempel di layar HP selama demo digulir.
                                    <FloatDeco kind=t.float_deco.clone() />
                                    <Suspense fallback=|| view! { <SkelPhone /> }>
                                        {move || demo.get().map(|r| match r {
                                            Ok(inv) => Either::Left(view! { <PreviewInvitation inv=Invitation { theme: slug.clone(), ..inv } anim=anim.clone() scroll=t.scroll_anim.clone() video=t.bg_video.clone() open_video=t.open_video.clone() rupa=crate::web::rupa::classes(&t.rupa) /> }),
                                            Err(e) => Either::Right(view! { <ErrorCard msg=err_msg(&e) /> }),
                                        })}
                                    </Suspense>
                                </div>
                            })
                        } else {
                            // Tema templat: HTML-nya dirender server (server/templat.rs),
                            // bukan komponen — demo = halaman undangan asli di bingkai HP.
                            Either::Right(view! {
                                <div class="demo-phone demo-phone--frame">
                                    <iframe class="demo-frame" src=format!("/u/{}?tema={}", DEMO_SLUG, t.slug) title=format!("Demo tema {}", t.name)></iframe>
                                </div>
                            })
                        }}
                        <aside class="demo-side">
                            <MusicPanel />
                            <PhotoTryPanel />
                            <section class="card side-card">
                                <div class="side-card__head">
                                    <h3>"Keunggulan Tema Ini"</h3>
                                    <span class="chip chip--gold">"Lengkap & Siap Pakai"</span>
                                </div>
                                <ul class="checklist">
                                    <li><Icon name="check_circle" />"Nama tamu tak terbatas (link pribadi otomatis via WhatsApp)."</li>
                                    <li><Icon name="check_circle" />"Desain responsif: sempurna di iPhone, Android, tablet & laptop."</li>
                                    <li><Icon name="check_circle" />"Dashboard manajemen ucapan, doa & konfirmasi RSVP real-time."</li>
                                    <li><Icon name="check_circle" />"Tanda kasih / amplop digital dengan salin nomor rekening sekali ketuk."</li>
                                    <li><Icon name="check_circle" />"QR check-in tamu di meja penerima."</li>
                                </ul>
                                <div class="side-price">
                                    <div>
                                        <small>"Termasuk di semua paket"</small>
                                        <b>{move || format!("mulai {}", rupiah(from()))}</b>
                                        <small>"Pratinjau gratis — bayar saat siap disebar"</small>
                                    </div>
                                    <a class="btn btn--primary" href=format!("/buat?tema={}", t.slug)>
                                        <Icon name="shopping_bag" />
                                        "Pesan Undangan"
                                    </a>
                                </div>
                            </section>
                        </aside>
                    </div>
                    // Tautan internal antar-tema: tanpa ini tiap halaman tema
                    // hanya ditautkan dari beranda (diukur Okt 2026: 1 tautan
                    // masuk per tema) — pengunjung pun buntu setelah demo.
                    <Suspense fallback=|| ()>
                        {move || related.get().and_then(|r| r.ok()).filter(|v| !v.is_empty()).map(|items| view! {
                            <section class="tema-serupa" aria-labelledby="tema-serupa">
                                <h2 id="tema-serupa">"Tema serupa"</h2>
                                <ul class="tema-serupa__list">
                                    {items.into_iter().map(|o| {
                                        let sub = [o.category.as_str(), o.nuansa.as_str()].into_iter().filter(|s| !s.trim().is_empty()).collect::<Vec<_>>().join(" · ");
                                        view! {
                                            <li>
                                                <a class="tema-serupa__item" href=format!("/tema/{}", o.slug)>
                                                    <b>{o.name.clone()}</b>
                                                    <small>{sub}</small>
                                                    {(!o.region.trim().is_empty()).then(|| view! { <small class="tema-serupa__asal">{o.region.clone()}</small> })}
                                                </a>
                                            </li>
                                        }
                                    }).collect_view()}
                                </ul>
                                <p class="tema-serupa__more">
                                    <a class="btn btn--soft btn--sm" href="/#katalog"><Icon name="grid_view" />"Lihat semua tema"</a>
                                    <a class="btn btn--soft btn--sm" href="/paket"><Icon name="loyalty" />"Paket & harga"</a>
                                </p>
                            </section>
                        })}
                    </Suspense>
                })
            })}
            </Suspense>
            <SiteFooter />
        </div>
    }
}

/// Undangan lengkap dalam satu gulungan (versi desktop dari tab Sampul →
/// Acara → RSVP), dirender dengan tema yang sedang dipratinjau.
#[component]
fn PreviewInvitation(inv: Invitation, anim: String, scroll: String, video: String, open_video: String, rupa: String) -> impl IntoView {
    let resepsi = inv.events.last().cloned();
    let names = format!("{} & {}", inv.bride_nick, inv.groom_nick);
    view! {
        <div class=format!("inv inv--embed th-{}{}{}{}", inv.theme, rupa, scroll_class(&scroll), if video.is_empty() { "" } else { " inv--video" })>
            <div class="inv__glow" aria-hidden="true"></div>
            <BgVideo src=video.clone() poster=Some(video_poster(&video, &inv.cover_photo)) />
            // Gerbang: sampul + tamu + tombol buka. Isi di bawahnya baru bisa
            // di-scroll setelah dibuka (skrip global: data-demo-open → .is-open).
            <div class=format!("gate gate--{anim} gate--embed")>
                <div class="gate__panel gate__panel--l" aria-hidden="true"></div>
                <div class="gate__panel gate__panel--r" aria-hidden="true"></div>
                <div class="gate__orn" aria-hidden="true"></div>
                <GateFx />
                {(!open_video.is_empty()).then(|| view! { <GateBg src=video.clone() poster=Some(video_poster(&video, "")) /> })}
                <GateVideo src=open_video.clone() />
                <div class="gate__content">
                    <section class="hero cover orn-host">
                        <Ornamen bagian="sampul" />
                        <p class="script cover__eyebrow">"The Wedding Of"</p>
                        {if inv.cover_photo.is_empty() {
                            view! { <div class="hero__seal hero__seal--sm"><Monogram initials=inv.initials() caption="THE WEDDING OF" /></div> }.into_any()
                        } else {
                            let photo = inv.cover_photo.clone();
                            view! {
                                <div class="arch-photo arch-photo--cover">
                                    <span class="arch-photo__clip"><img src=photo.clone() alt="" style=crate::web::fmt::photo_style(&photo) /></span>
                                    <span class="arch-photo__frame" aria-hidden="true"></span>
                                </div>
                            }.into_any()
                        }}
                        // h2: h1 halaman = nama tema (tema-head), bukan nama pasangan demo.
                        <h2 class="cover__names">{names}</h2>
                        <p class="pill"><Icon name="calendar_month" />{inv.date_label()}</p>
                    </section>
                    <section class="guest card">
                        <p class="muted">"Kepada Yth. Bapak/Ibu/Saudara/i:"</p>
                        <h2 class="guest__name">"Bapak Sandiaga & Keluarga"</h2>
                        <p class="guest__note">"*Mohon maaf bila ada kesalahan penulisan nama / gelar."</p>
                    </section>
                    <button type="button" class="btn btn--gold btn--lg gate__open" data-demo-open="1">
                        <Icon name="drafts" />"Buka Undangan"
                    </button>
                </div>
            </div>
            <Countdown target_ms=inv.countdown_target_ms() />
            <QuoteCard text=inv.quote_text.clone() source=inv.quote_source.clone() />
            <Couple inv=inv.clone() />
            <LoveStorySection items=inv.love_story.clone() />
            <section class="section orn-host">
                <Ornamen bagian="acara" />
                <p class="eyebrow eyebrow--center">"Rangkaian Acara"</p>
                <h2 class="section__title">"Akad Nikah & Resepsi"</h2>
                {inv.events.clone().into_iter().map(|ev| view! { <EventCard ev=ev /> }).collect_view()}
            </section>
            {resepsi.map(|ev| view! { <MapCard ev=ev /> })}
            <LiveStream url=inv.live_url.clone() />
            <Gallery photos=inv.gallery.clone() />
            <section class="section orn-host">
                <Ornamen bagian="rsvp" />
                <p class="eyebrow eyebrow--center">"Kehadiran & Hadiah"</p>
                <h2 class="section__title">"Konfirmasi & Tanda Kasih"</h2>
                <p class="notice notice--info">"Formulir di pratinjau ini hanya contoh dan tidak disimpan."</p>
                <RsvpForm slug=inv.slug.clone() guest=None sessions=super::undangan::session_options(&inv) />
                <GiftSection inv=inv.clone() guest=None open=true />
            </section>
            <p class="closing-note">
                "Merupakan suatu kehormatan & kebahagiaan bagi kami apabila Bapak/Ibu/Saudara/i berkenan hadir dan memberikan doa restu."
                <br />
                <b>{format!("— {} —", inv.family_name)}</b>
            </p>
        </div>
    }
}

#[component]
fn MusicPanel() -> impl IntoView {
    // Pustaka lagu admin (sama dengan pilihan di /buat); lagu pertama terpilih.
    let songs = Resource::new(|| (), |_| list_songs());
    let contact = Resource::new(|| (), |_| get_contact());
    view! {
        <section class="card side-card music">
            <div class="side-card__head">
                <div>
                    <span class="chip chip--gold chip--xs">"Pengaturan Musik Aktif"</span>
                    <h3>"Musik Latar Pernikahan"</h3>
                    <p class="muted small">"Pilih dari pustaka lagu kami saat memesan. Lagu belum ada? Minta ke admin."</p>
                </div>
                <button type="button" class="icon-btn disc" data-music="toggle" aria-label="Putar / jeda"><Icon name="music_note" /></button>
            </div>
            <Suspense fallback=|| view! { <SkelRows n=3 /> }>
                {move || songs.get().map(|r| {
                    let list = r.unwrap_or_default();
                    let first = list.first().cloned().unwrap_or_default();
                    view! {
                        <p class="eyebrow">{format!("Pustaka Lagu • {} pilihan", list.len())}</p>
                        <div class="songs">
                            {list.into_iter().enumerate().map(|(i, s)| view! {
                                <button type="button" class="song" class:is-active=i == 0 data-song=s.url.clone() data-title=format!("{} – {}", s.title, s.artist)>
                                    <span class="song__play"><Icon name="play_arrow" class="when-idle" /><Icon name="pause" class="when-playing" /></span>
                                    <span class="song__meta"><b>{s.title.clone()}</b><small>{s.meta()}</small></span>
                                    {(!s.tag.is_empty()).then(|| view! { <span class="tag">{s.tag.clone()}</span> })}
                                </button>
                            }).collect_view()}
                        </div>
                        <div class="now-playing">
                            <span class="disc-art"><Icon name="album" /></span>
                            <div>
                                <small class="eyebrow eyebrow--gold">"Sedang Dipilih"</small>
                                <b data-now-playing="">{if first.title.is_empty() { "—".to_string() } else { format!("{} – {}", first.title, first.artist) }}</b>
                            </div>
                            <button type="button" class="icon-btn icon-btn--dark" data-music="toggle" aria-label="Putar / jeda">
                                <Icon name="play_arrow" class="when-idle" />
                                <Icon name="pause" class="when-playing" />
                            </button>
                        </div>
                        <MusicAudio url=first.url looped=true />
                    }
                })}
            </Suspense>
            <MusicSeek />
            <ul class="prefs">
                <li><Icon name="check_circle" /><span><b>"Autoplay saat undangan dibuka"</b><small>"Musik mulai ketika tamu mengetuk tombol buka undangan."</small></span></li>
                <li><Icon name="check_circle" /><span><b>"Ulangi musik terus-menerus"</b><small>"Lagu kembali ke awal setelah selesai."</small></span></li>
                <li><Icon name="check_circle" /><span><b>"Lagu dari pustaka admin"</b><small>"Belum ada lagu favorit Anda? Request — kami tambahkan."</small></span></li>
            </ul>
            <Suspense fallback=|| ()>
                {move || contact.get().and_then(|r| r.ok()).filter(|w| !w.is_empty()).map(|wa| view! {
                    <a class="btn btn--soft btn--sm btn--block song-request" target="_blank" rel="noopener"
                        href=format!("https://wa.me/{wa}?text={}", crate::web::fmt::url_encode("Halo admin, saya ingin request lagu untuk musik latar undangan: (judul – penyanyi)"))>
                        <Icon name="library_music" />"Request lagu ke admin"
                    </a>
                })}
            </Suspense>
        </section>
    }
}

/// Coba foto mempelai sendiri di pratinjau demo — hanya di browser (URL blob),
/// tak pernah dikirim ke server; dibuang saat halaman ditinggalkan.
#[component]
fn PhotoTryPanel() -> impl IntoView {
    view! {
        <section class="card side-card">
            <div class="side-card__head">
                <div>
                    <h3>"Coba Foto Mempelai"</h3>
                    <p class="muted small">"Lihat foto Anda langsung di tema ini. Foto tidak diunggah dan otomatis terhapus saat Anda keluar dari halaman."</p>
                </div>
            </div>
            {[("Mempelai wanita", 1), ("Mempelai pria", 2)].into_iter().map(|(label, n)| view! {
                <div class="preview-row" data-preview-box="">
                    <label class="upload">
                        <Icon name="photo_camera" />
                        <span><b>{label}</b><small>"JPEG/PNG/WebP maks 5 MB — geser & zoom di bingkai"</small></span>
                        <input type="file" accept="image/jpeg,image/png,image/webp" data-preview="image" data-max="5"
                            data-target=format!(".demo-phone .couple > .person:nth-child({n}) .person__photo") />
                    </label>
                    <div class="preview-out preview-out--photo" data-preview-out=""></div>
                </div>
            }).collect_view()}
        </section>
    }
}
