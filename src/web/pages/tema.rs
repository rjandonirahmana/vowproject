//! pages/tema.rs — /tema/:tema: pratinjau interaktif satu tema memakai data
//! undangan demo, plus panel musik latar (putar/pilih lagu bawaan), keunggulan
//! tema, dan harga. Desktop = dua kolom (undangan + panel), mobile = bertumpuk.

use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

use crate::web::api::{get_demo, get_konten, get_theme};
use crate::web::components::*;
use crate::web::fmt::rupiah;
use crate::web::icons::Icon;
use crate::web::model::Invitation;
use crate::web::themes::{DEMO_SLUG, DEMO_SONG, SONGS};
use crate::web::skin::ThemeInfo;

use super::{ErrorCard, SiteFooter, SiteHeader};

#[component]
pub fn TemaPage() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.read().get("tema").unwrap_or_default();
    let demo = Resource::new(|| (), |_| get_demo());
    let theme = Resource::new(slug, get_theme);
    let konten = Resource::new(|| (), |_| get_konten());
    let from = move || konten.get().and_then(|r| r.ok()).map(|k| k.min_price()).unwrap_or(0);
    view! {
        <div class="site">
            <SiteHeader />
            <Suspense fallback=|| view! { <div class="inv-loading"><div class="spinner"></div></div> }>
            {move || theme.get().map(|r| {
                let Some(t): Option<ThemeInfo> = r.ok().flatten() else {
                    return Either::Left(view! { <super::NotFoundPage /> });
                };
                let slug = t.slug.clone();
                provide_ornaments(t.ornaments.clone());
                // Demo selalu mulai TERTUTUP seperti yang dialami tamu; tema tanpa
                // animasi pembuka memakai efek "pudar".
                let anim = if t.open_anim.is_empty() || t.open_anim == "none" { "pudar".to_string() } else { t.open_anim.clone() };
                Either::Right(view! {
                    <Title text=format!(concat!("{} — Demo Tema ", crate::brand!()), t.name) />
                    <div class="demo-bar">
                        <span class="chip chip--live"><i class="dot dot--live"></i>"Live Interactive Preview"</span>
                        <span class="chip chip--soft">{if t.listed { t.name.clone() } else { format!("{} • Tema privat", t.name) }}</span>
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
                        <div class="demo-phone">
                            // Hiasan melayang tema — menempel di layar HP selama demo digulir.
                            <FloatDeco kind=t.float_deco.clone() />
                            <Suspense fallback=|| view! { <div class="inv-loading"><div class="spinner"></div></div> }>
                                {move || demo.get().map(|r| match r {
                                    Ok(inv) => Either::Left(view! { <PreviewInvitation inv=Invitation { theme: slug.clone(), ..inv } anim=anim.clone() scroll=t.scroll_anim.clone() /> }),
                                    Err(e) => Either::Right(view! { <ErrorCard msg=err_msg(&e) /> }),
                                })}
                            </Suspense>
                        </div>
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
fn PreviewInvitation(inv: Invitation, anim: String, scroll: String) -> impl IntoView {
    let resepsi = inv.events.last().cloned();
    let names = format!("{} & {}", inv.bride_nick, inv.groom_nick);
    view! {
        <div class=format!("inv inv--embed th-{}{}", inv.theme, scroll_class(&scroll))>
            <div class="inv__glow" aria-hidden="true"></div>
            // Gerbang: sampul + tamu + tombol buka. Isi di bawahnya baru bisa
            // di-scroll setelah dibuka (skrip global: data-demo-open → .is-open).
            <div class=format!("gate gate--{anim} gate--embed")>
                <div class="gate__panel gate__panel--l" aria-hidden="true"></div>
                <div class="gate__panel gate__panel--r" aria-hidden="true"></div>
                <div class="gate__orn" aria-hidden="true"></div>
                <GateFx />
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
                        <h1 class="cover__names">{names}</h1>
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
    // Lagu demo dulu (terpilih), lalu lagu bawaan yang bisa dipilih saat memesan.
    let first = &DEMO_SONG;
    let songs: Vec<&'static crate::web::themes::Song> = std::iter::once(&DEMO_SONG).chain(SONGS.iter()).collect();
    view! {
        <section class="card side-card music">
            <div class="side-card__head">
                <div>
                    <span class="chip chip--gold chip--xs">"Pengaturan Musik Aktif"</span>
                    <h3>"Musik Latar Pernikahan"</h3>
                    <p class="muted small">"Atur lagu pembuka saat tamu membuka undangan digital Anda secara interaktif."</p>
                </div>
                <button type="button" class="icon-btn disc" data-music="toggle" aria-label="Putar / jeda"><Icon name="music_note" /></button>
            </div>
            <p class="eyebrow">{format!("Daftar Lagu • {} pilihan", songs.len())}</p>
            <div class="songs">
                {songs.into_iter().enumerate().map(|(i, s)| view! {
                    <button type="button" class="song" class:is-active=i == 0 data-song=s.url() data-title=format!("{} – {}", s.title, s.artist)>
                        <span class="song__play"><Icon name="play_arrow" class="when-idle" /><Icon name="pause" class="when-playing" /></span>
                        <span class="song__meta">
                            <b>{s.title}</b>
                            <small>{if s.duration.is_empty() { s.artist.to_string() } else { format!("{} • {}", s.artist, s.duration) }}</small>
                        </span>
                        {(!s.tag.is_empty()).then(|| view! { <span class="tag">{s.tag}</span> })}
                    </button>
                }).collect_view()}
            </div>
            <div class="now-playing">
                <span class="disc-art"><Icon name="album" /></span>
                <div>
                    <small class="eyebrow eyebrow--gold">"Sedang Dipilih"</small>
                    <b data-now-playing="">{format!("{} – {}", first.title, first.artist)}</b>
                </div>
                <button type="button" class="icon-btn icon-btn--dark" data-music="toggle" aria-label="Putar / jeda">
                    <Icon name="play_arrow" class="when-idle" />
                    <Icon name="pause" class="when-playing" />
                </button>
            </div>
            <div data-preview-box="">
                <label class="upload">
                    <Icon name="library_music" />
                    <span><b>"Coba lagu Anda sendiri"</b><small>"MP3/M4A maks 6 MB — diputar langsung di browser, tidak diunggah, terhapus saat Anda keluar."</small></span>
                    <input type="file" accept="audio/mpeg,audio/mp4,audio/x-m4a,audio/ogg" data-preview="audio" data-max="6" />
                </label>
                <div class="preview-out" data-preview-out=""></div>
            </div>
            <MusicSeek />
            <ul class="prefs">
                <li><Icon name="check_circle" /><span><b>"Autoplay saat undangan dibuka"</b><small>"Musik mulai ketika tamu mengetuk tombol buka undangan."</small></span></li>
                <li><Icon name="check_circle" /><span><b>"Ulangi musik terus-menerus"</b><small>"Lagu kembali ke awal setelah selesai."</small></span></li>
                <li><Icon name="check_circle" /><span><b>"Upload MP3 sendiri"</b><small>"Maks. 6 MB saat mengisi formulir pemesanan."</small></span></li>
            </ul>
            <MusicAudio url=first.url() looped=true />
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
