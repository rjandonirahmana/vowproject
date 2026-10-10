//! web/components.rs — potongan undangan yang dipakai ulang oleh halaman tamu
//! (/u/…), pratinjau tema (/tema/…), dan katalog.

use leptos::either::Either;
use leptos::prelude::*;

use super::api::{list_wishes, ConfirmGift, SubmitRsvp};
use super::fmt;
use super::i18n::lang;
use crate::tx;
use super::icons::Icon;
use super::model::*;
use super::skeleton::SkelRows;

/// Pesan galat server fn tanpa awalan teknis.
pub fn err_msg(e: &ServerFnError) -> String {
    match e {
        ServerFnError::ServerError(s) => s.clone(),
        other => {
            let s = other.to_string();
            s.rsplit(": ").next().unwrap_or(&s).to_string()
        }
    }
}

// ── Monogram ───────────────────────────────────────────────────────────────

/// Emblem monogram: cincin foil emas ganda + ranting zaitun + inisial.
/// SVG digambar dari kode supaya inisial tiap pasangan ikut berubah.
pub fn monogram_svg(initials: &str, caption: &str) -> String {
    let mut leaves = String::new();
    let (cx, cy, r) = (100.0_f64, 100.0_f64, 64.0_f64);
    for side in [-1.0_f64, 1.0] {
        for i in 0..11 {
            // Dari bawah (100°) naik ke atas (235°) di sisi kiri; cermin di kanan.
            let t = 100.0 + i as f64 * 13.0;
            let a = t.to_radians();
            let x = cx + side * r * a.sin() * -1.0;
            let y = cy - r * a.cos() * -1.0;
            let tangent = (t + 90.0) * side;
            for (k, off) in [(0, -38.0), (1, 38.0)] {
                if i == 10 && k == 1 {
                    continue;
                }
                let fill = if (i + k) % 3 == 0 { "#5b7553" } else { "url(#mg-gold)" };
                leaves.push_str(&format!(
                    r#"<ellipse cx="{x:.1}" cy="{y:.1}" rx="7.5" ry="2.6" fill="{fill}" transform="rotate({:.1} {x:.1} {y:.1}) translate({:.1} 0)"/>"#,
                    -tangent + off * side,
                    4.0 * if k == 0 { -1.0 } else { 1.0 },
                ));
            }
        }
    }
    let initials = super::fmt::html_escape(initials);
    let caption = super::fmt::html_escape(caption);
    format!(
        r##"<svg viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="Monogram {initials}">
<defs><linearGradient id="mg-gold" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#c5a059"/><stop offset=".5" stop-color="#dfbe72"/><stop offset="1" stop-color="#997833"/></linearGradient>
<path id="mg-arc" d="M 38 118 A 64 64 0 0 0 162 118"/></defs>
<circle cx="100" cy="100" r="92" fill="#fbf9f5" stroke="url(#mg-gold)" stroke-width="3"/>
<circle cx="100" cy="100" r="86" fill="none" stroke="#879878" stroke-width="1.2"/>
<path d="M 72 160 A 64 64 0 0 1 64 48" fill="none" stroke="url(#mg-gold)" stroke-width="1.2"/>
<path d="M 128 160 A 64 64 0 0 0 136 48" fill="none" stroke="url(#mg-gold)" stroke-width="1.2"/>
{leaves}
<text x="100" y="112" text-anchor="middle" font-family="Playfair Display, Georgia, serif" font-size="40" font-weight="600" fill="url(#mg-gold)">{initials}</text>
<text font-family="Playfair Display, Georgia, serif" font-size="9" letter-spacing="2.5" fill="#997833"><textPath href="#mg-arc" startOffset="50%" text-anchor="middle">{caption}</textPath></text>
</svg>"##
    )
}


#[component]
pub fn Monogram(#[prop(into)] initials: String, #[prop(into, optional)] caption: String, #[prop(optional)] class: &'static str) -> impl IntoView {
    view! { <div class=format!("monogram {class}") inner_html=monogram_svg(&initials, &caption)></div> }
}

// ── Potongan undangan ──────────────────────────────────────────────────────

#[component]
pub fn Countdown(
    target_ms: i64,
    #[prop(optional)] title: &'static str,
    /// Tanggal acara ("Sabtu, 24 Oktober 2026") di bawah judul kaligrafi.
    #[prop(optional)] date: String,
    /// Tautan Google Calendar — tombol "Simpan Tanggal" (ala everlove).
    #[prop(optional)] calendar: String,
    /// Foto pasangan di atas judul (bingkai + foto bergerak, ala everlove).
    #[prop(optional)] photos: Vec<String>,
) -> impl IntoView {
    let l = lang();
    let has_photo = photos.iter().any(|p| !p.is_empty());
    let title = if title.is_empty() { tx!(l, "Menuju Hari Bahagia", "Counting Down to Our Day") } else { title };
    view! {
        <div class="countdown card card--soft" data-countdown=target_ms.to_string()>
            {has_photo.then(|| view! {
                <div class="countdown__photo"><FotoGerak photos=photos.clone() alt=tx!(l, "Foto pasangan", "Photo of the couple").to_string() /></div>
            })}
            <p class="eyebrow eyebrow--gold eyebrow--center">{title}</p>
            <h2 class="script countdown__title">"Save The Date"</h2>
            {(!date.is_empty()).then(|| view! { <p class="countdown__date">{date}</p> })}
            <div class="countdown__grid">
                <div class="countdown__tile"><b data-cd="d">"0"</b><span>{tx!(l, "Hari", "Days")}</span></div>
                <div class="countdown__tile"><b data-cd="h">"00"</b><span>{tx!(l, "Jam", "Hours")}</span></div>
                <div class="countdown__tile"><b data-cd="m">"00"</b><span>{tx!(l, "Menit", "Minutes")}</span></div>
                <div class="countdown__tile"><b data-cd="s">"00"</b><span>{tx!(l, "Detik", "Seconds")}</span></div>
            </div>
            {(!calendar.is_empty()).then(|| view! {
                <a class="btn btn--outline btn--sm countdown__cal" href=calendar target="_blank" rel="noopener">
                    <Icon name="calendar_add_on" />{tx!(l, "Simpan Tanggal", "Save the Date")}
                </a>
            })}
        </div>
    }
}

#[component]
pub fn QuoteCard(text: String, source: String) -> impl IntoView {
    (!text.is_empty()).then(|| {
        view! {
            <figure class="quote card">
                <span class="quote__mark">"99"</span>
                <blockquote>{format!("“{text}”")}</blockquote>
                <figcaption class="rule-label">{source}</figcaption>
            </figure>
        }
    })
}

#[component]
pub fn Couple(inv: Invitation) -> impl IntoView {
    let l = lang();
    let person = |label: &'static str, nick: String, name: String, degree: String, parents: String, ig: String, photo: String| {
        let full = if degree.is_empty() { name.clone() } else { format!("{name}, {degree}") };
        let init = avatar_initials(&name);
        let short = if nick.trim().is_empty() { name.split_whitespace().next().unwrap_or("").to_string() } else { nick };
        view! {
            <article class="person card">
                <div class="arch-photo arch-photo--person">
                    <span class="arch-photo__clip">
                        {if photo.is_empty() {
                            Either::Left(view! { <span class="person__initial">{init}</span> })
                        } else {
                            Either::Right(view! { <FotoGerak photos=vec![photo.clone()] alt=name.clone() /> })
                        }}
                    </span>
                    <span class="arch-photo__frame" aria-hidden="true"></span>
                </div>
                <div class="person__body">
                    <span class="chip chip--gold">{label}</span>
                    <p class="script person__nick">{short}</p>
                    <h3 class="person__name">{full}</h3>
                    <p class="person__parents">{parents}</p>
                    {(!ig.is_empty()).then(|| view! {
                        <a class="person__ig" href=format!("https://instagram.com/{ig}") target="_blank" rel="noopener">
                            <Icon name="photo_camera" />
                            {format!("@{ig}")}
                        </a>
                    })}
                </div>
            </article>
        }
    };
    view! {
        <section class="section orn-host">
            <Ornamen bagian="mempelai" />
            <p class="eyebrow eyebrow--center">{tx!(l, "Kedua Mempelai", "The Happy Couple")}</p>
            <h2 class="section__title">"Bride & Groom"</h2>
            <div class="couple">
                {person(tx!(l, "Mempelai Wanita", "The Bride"), inv.bride_nick, inv.bride_name, inv.bride_degree, inv.bride_parents, inv.bride_ig, inv.bride_photo)}
                <span class="couple__amp" aria-hidden="true">"&"</span>
                {person(tx!(l, "Mempelai Pria", "The Groom"), inv.groom_nick, inv.groom_name, inv.groom_degree, inv.groom_parents, inv.groom_ig, inv.groom_photo)}
            </div>
        </section>
    }
}

/// Foto bergerak ala everlove: 1–3 foto bertumpuk dalam satu bingkai — tiap
/// foto tampil 1 dtk lalu silang-pudar 3 dtk ke foto berikutnya sambil Ken
/// Burns zoom-in (satu foto = Ken Burns saja). CSS murni (`.fg` di main.css);
/// global.js hanya menjeda animasi saat bingkai di luar layar. Foto ke-2 dst.
/// `loading=lazy` → tak ikut dimuat sebelum bingkainya mendekati layar.
#[component]
pub fn FotoGerak(photos: Vec<String>, alt: String, #[prop(optional)] priority: bool) -> impl IntoView {
    let photos: Vec<String> = photos.into_iter().filter(|p| !p.is_empty()).take(3).collect();
    let n = photos.len();
    view! {
        <span class="fg" data-fg=n.to_string()>
            {photos.into_iter().enumerate().map(|(i, src)| {
                let first = i == 0;
                view! {
                    <img
                        class="fg__img"
                        src=src.clone()
                        alt=if first { alt.clone() } else { String::new() }
                        aria-hidden=(!first).then_some("true")
                        loading=if first && priority { "eager" } else { "lazy" }
                        fetchpriority=if first && priority { "high" } else { "auto" }
                        decoding="async"
                        style=format!("--fg-i:{i};{}", fmt::photo_style(&src))
                    />
                }
            }).collect_view()}
        </span>
    }
}

/// Kisah cinta dalam garis waktu. Babak tanpa foto sendiri memakai `photos`
/// (galeri pasangan) bergiliran — seperti everlove: satu foto per babak.
#[component]
pub fn LoveStorySection(items: Vec<LoveStory>, #[prop(optional)] photos: Vec<String>) -> impl IntoView {
    let l = lang();
    (!items.is_empty()).then(|| {
        view! {
            <section class="section orn-host">
                <Ornamen bagian="kisah" />
                <p class="eyebrow eyebrow--center">{tx!(l, "Perjalanan Kami", "Our Journey")}</p>
                <h2 class="section__title">"Love Story"</h2>
                <ol class="story card">
                    {items.into_iter().enumerate().map(|(i, it)| {
                        let img = if it.img.is_empty() { photos.get(i).cloned().unwrap_or_default() } else { it.img.clone() };
                        view! {
                        <li class="story__item">
                            <span class="story__dot" aria-hidden="true"><Icon name="favorite" /></span>
                            <div>
                                {(!img.is_empty()).then(|| view! {
                                    <div class="story__photo"><FotoGerak photos=vec![img.clone()] alt=it.title.clone() /></div>
                                })}
                                {(!it.year.is_empty()).then(|| view! { <span class="story__year">{it.year.clone()}</span> })}
                                <h3>{it.title.clone()}</h3>
                                <p>{it.text.clone()}</p>
                            </div>
                        </li>
                    }}).collect_view()}
                </ol>
            </section>
        }
    })
}

/// Tombol siaran langsung (hanya https).
#[component]
pub fn LiveStream(url: String) -> impl IntoView {
    url.starts_with("https://").then(|| {
        view! {
            <section class="live-card card">
                <span class="intro__icon"><Icon name="smartphone" /></span>
                <h2 class="section__title">"Live Streaming"</h2>
                <p class="muted">"Tidak bisa hadir? Saksikan momen sakral kami secara langsung melalui tautan berikut."</p>
                <a class="btn btn--primary" href=url target="_blank" rel="noopener"><Icon name="play_arrow" />"Tonton Siaran Langsung"</a>
            </section>
        }
    })
}

// ── Lapisan ornamen per bagian (web/ornamen.rs) ───────────────────────────

/// Ornamen tema yang sedang tampil — disediakan halaman undangan, pratinjau
/// tema, dan editor admin (sinyal draf → pratinjau langsung).
#[derive(Clone, Copy)]
pub struct OrnLayer(pub Signal<Vec<crate::web::ornamen::Ornament>>);

pub fn provide_ornaments(list: Vec<crate::web::ornamen::Ornament>) {
    provide_context(OrnLayer(Signal::stored(list)));
}

/// Hiasan bergambar satu bagian. Diletakkan sebagai anak PERTAMA bagian
/// (kelas `orn-host` di bagian itu): lapisan belakang di balik isi, lapisan
/// depan di atasnya; animasi masuk dipicu skrip global (`[data-orn]` → `.is-in`).
#[component]
pub fn Ornamen(bagian: &'static str) -> impl IntoView {
    let layer = use_context::<OrnLayer>();
    move || {
        let list: Vec<crate::web::ornamen::Ornament> =
            layer?.0.with(|v| v.iter().filter(|o| o.bagian == bagian && crate::web::skin::is_safe_url(&o.img)).cloned().collect());
        if list.is_empty() {
            return None;
        }
        let (front, back): (Vec<_>, Vec<_>) = list.into_iter().partition(|o| o.depan);
        let layer_view = |v: Vec<crate::web::ornamen::Ornament>, cls: &'static str| {
            (!v.is_empty()).then(|| view! {
                <div class=cls aria-hidden="true">
                    {v.into_iter().map(|o| view! {
                        <span class=o.class() style=o.style() data-orn="1" data-orn-id=o.id.to_string()>
                            <img class="orn__img" src=o.img.clone() alt="" loading="lazy" decoding="async" />
                        </span>
                    }).collect_view()}
                </div>
            })
        };
        Some(view! { {layer_view(back, "orn-layer")}{layer_view(front, "orn-layer orn-layer--depan")} })
    }
}

/// Dekorasi melayang (kelopak / kupu-kupu / bintang) — CSS murni, mati
/// otomatis bila pengguna memilih "kurangi animasi".
#[component]
pub fn FloatDeco(kind: String) -> impl IntoView {
    // Gaya tiap hiasan ada di /tema.css (tabel animations, jenis "hiasan").
    (kind != crate::web::anim::NONE && crate::web::fmt::is_slug(&kind, crate::web::anim::KEY_MAX)).then(|| {
        view! {
            <div class=format!("float-deco float-deco--{kind}") aria-hidden="true">
                // 14 elemen; standarnya 9 tampil, animasi dari pengaturan mengatur jumlahnya sendiri.
                {(0..14).map(|i| view! { <i style=format!("--i:{i}")></i> }).collect_view()}
            </div>
        }
    })
}

/// Butir efek gerbang pembuka (helai songket, daun lepas, bintang warp, kabut…).
/// Tersembunyi kecuali animasi buka memakainya (`{a} .gate__fx`).
#[component]
pub fn GateFx() -> impl IntoView {
    view! {
        <div class="gate__fx" aria-hidden="true">
            {(0..12).map(|i| view! { <i style=format!("--i:{i}")></i> }).collect_view()}
        </div>
    }
}

/// Video latar tema sinema: video pasangan bila ada, kalau tidak video bawaan
/// tema. Kosong = bukan tema sinema.
pub fn bg_video_src(theme_video: &str, inv_video: &str) -> String {
    if theme_video.is_empty() { String::new() } else if !inv_video.is_empty() { inv_video.to_string() } else { theme_video.to_string() }
}

/// Lapisan video latar penuh (tema sinema). TIDAK autoplay dari HTML — /app.js
/// memutarnya setelah gerbang dibuka (atau langsung bila tanpa gerbang), dan
/// melewatinya di mode hemat (html.motion-min) sehingga hanya poster tampil.
#[component]
pub fn BgVideo(src: String, #[prop(optional_no_strip)] poster: Option<String>) -> impl IntoView {
    (!src.is_empty()).then(|| view! {
        <div class="inv-video" aria-hidden="true">
            <video src=src muted=true loop=true playsinline=true preload="none" poster=poster.unwrap_or_default() data-bgvideo="1" tabindex="-1"></video>
            <div class="inv-video__shade"></div>
        </div>
    })
}

/// Video pembuka di dalam gerbang (animasi buka `video-pintu`): diputar
/// sekali oleh /app.js saat "Buka Undangan"; gerbang ditutup setelah selesai.
/// preload="none": /app.js memanaskannya SETELAH halaman termuat & hanya di
/// perangkat/koneksi kuat — dulu "auto" mengunduh ±0,7 MB sejak milidetik
/// ke-200, berebut dengan CSS & font sampul.
#[component]
pub fn GateVideo(src: String) -> impl IntoView {
    (!src.is_empty()).then(|| view! {
        <video class="gate__video" src=src muted=true playsinline=true preload="none" data-gatevideo="1" aria-hidden="true" tabindex="-1"></video>
    })
}

/// Latar BERGERAK sampul (tema dengan video pembuka): potongan ulang video
/// latar diputar di belakang foto & nama sebelum tamu menekan "Buka
/// Undangan" — seperti slideshow bergerak di sampul undangan premium.
/// Diputar /app.js setelah halaman termuat (mode hemat → poster saja).
#[component]
pub fn GateBg(src: String, #[prop(optional_no_strip)] poster: Option<String>) -> impl IntoView {
    (!src.is_empty()).then(|| view! {
        <div class="gate__bg" aria-hidden="true">
            <video src=src muted=true playsinline=true preload="none" poster=poster.unwrap_or_default() data-gatebg="1" tabindex="-1"></video>
        </div>
    })
}

/// Poster video latar: video bawaan situs (/video/x.mp4) punya poster
/// /video/x.jpg (frame akhir); video pasangan memakai foto sampul.
pub fn video_poster(video: &str, cover: &str) -> String {
    let base = video.split('#').next().unwrap_or("");
    if base.starts_with("/video/") {
        if let Some((stem, _)) = base.rsplit_once('.') {
            return format!("{stem}.jpg");
        }
    }
    cover.split('#').next().unwrap_or("").to_string()
}

/// Bagian "Video Prewedding" (tombol putar, bersuara — musik latar dijeda
/// otomatis oleh /app.js saat video ini diputar).
#[component]
pub fn PreweddingVideo(src: String, #[prop(optional_no_strip)] poster: Option<String>) -> impl IntoView {
    (!src.is_empty()).then(|| view! {
        <section class="section orn-host video-sec">
            <p class="eyebrow eyebrow--center">"Prewedding"</p>
            <h2 class="section__title">"Video Prewedding"</h2>
            <div class="video-frame card">
                <video src=src controls=true playsinline=true preload="none" poster=poster.unwrap_or_default() data-prewed="1"></video>
            </div>
        </section>
    })
}

/// Kelas akar koreografi scroll tema (`rvs--{kunci}`), kosong bila kunci tak sah.
pub fn scroll_class(key: &str) -> String {
    if crate::web::fmt::is_slug(key, crate::web::anim::KEY_MAX) { format!(" rvs--{key}") } else { String::new() }
}

#[component]
pub fn EventCard(ev: Event, #[prop(optional_no_strip)] dress: Option<(String, Vec<DressColor>)>) -> impl IntoView {
    let is_resepsi = ev.kind == "resepsi";
    let maps = ev.maps_link();
    let l = lang();
    let date = ev.date_label_in(l);
    let time = ev.time_label_in(l);
    let sessions = ev.sessions.clone();
    // "Sabtu, 24 Oktober 2026" → hari / tanggal / bulan tahun (blok tanggal
    // ala undangan cetak); format lain → satu baris tanggal biasa.
    let parts = date.split_once(", ").and_then(|(day, rest)| {
        let (num, month) = rest.split_once(' ')?;
        Some((day.to_string(), num.to_string(), month.to_string()))
    });
    view! {
        <article class=if is_resepsi { "event card event--resepsi" } else { "event card" }>
            <p class="event__badge">{ev.badge_in(l)}</p>
            <h3 class="event__title">{ev.title.clone()}</h3>
            {match parts {
                Some((day, num, month)) => Either::Left(view! {
                    <div class="event__date">
                        <span class="script event__day">{day}</span>
                        <b class="event__num">{num}</b>
                        <span class="event__month">{month}</span>
                    </div>
                }),
                None => Either::Right(view! {
                    <p class="event__meta"><Icon name="calendar_month" /><span>{date}</span></p>
                }),
            }}
            {if sessions.is_empty() {
                Either::Left(view! { <p class="event__time">{time}</p> })
            } else {
                Either::Right(view! {
                    <div class="event__sessions">
                        {sessions.into_iter().map(|s| view! {
                            <div class="event__session"><span>{s.label}</span><b>{s.time}</b></div>
                        }).collect_view()}
                    </div>
                })
            }}
            <div class="event__venue">
                <Icon name="location_on" />
                <div>
                    <b>{ev.venue.clone()}</b>
                    <span>{ev.address.clone()}</span>
                </div>
            </div>
            {dress.filter(|(t, c)| !t.is_empty() || !c.is_empty()).map(|(text, colors)| view! {
                <div class="event__dress">
                    <p class="eyebrow eyebrow--gold"><Icon name="checkroom" />{tx!(l, "Panduan Busana / Dress Code", "Dress Code")}</p>
                    <p>{text}</p>
                    <div class="swatches">
                        {colors.into_iter().map(|c| view! {
                            <span class="swatch"><i style=format!("background:{}", if crate::web::skin::is_color(&c.hex) { c.hex.as_str() } else { "#161d17" })></i>{c.name}</span>
                        }).collect_view()}
                    </div>
                </div>
            })}
            <a class=if is_resepsi { "btn btn--primary btn--block" } else { "btn btn--soft btn--block" } href=maps target="_blank" rel="noopener">
                <Icon name="map" />
                {if is_resepsi { tx!(l, "Buka Google Maps Resepsi", "Open Reception in Google Maps") } else { tx!(l, "Buka Petunjuk Google Maps", "Open in Google Maps") }}
            </a>
        </article>
    }
}

#[component]
pub fn MapCard(ev: Event) -> impl IntoView {
    let l = lang();
    let addr = format!("{}, {}", ev.venue, ev.address);
    view! {
        <section class="mapcard card">
            <div class="mapcard__head">
                <Icon name="location_on" />
                <div>
                    <h3>{tx!(l, "Peta Lokasi Acara", "Venue Map")}</h3>
                    <p>{ev.venue.clone()}</p>
                </div>
            </div>
            // lazy: ±460 KB skrip Google Maps baru diunduh saat peta mendekati
            // layar (bukan saat tamu masih di sampul). Lewat inner_html karena
            // view! Leptos tak mengenal atribut `loading` pada <iframe>.
            <div class="mapcard__frame" inner_html=format!(
                r#"<iframe src="{}" title="{}" loading="lazy" referrerpolicy="no-referrer-when-downgrade"></iframe>"#,
                super::fmt::html_escape(&ev.maps_embed()),
                tx!(l, "Peta lokasi", "Venue map")
            )></div>
            <div class="mapcard__actions">
                <button type="button" class="btn btn--soft" data-copy=addr data-copied=tx!(l, "Alamat tersalin", "Address copied")>
                    <Icon name="content_copy" />
                    {tx!(l, "Salin Alamat", "Copy Address")}
                </button>
                <a class="btn btn--primary" href=ev.maps_link() target="_blank" rel="noopener">
                    <Icon name="near_me" />
                    {tx!(l, "Petunjuk Arah", "Get Directions")}
                </a>
            </div>
        </section>
    }
}

#[component]
pub fn GuestGuide() -> impl IntoView {
    let l = lang();
    let items: [(&'static str, &'static str, &'static str); 3] = [
        ("qr_code_2", "QR Code Check-in", tx!(l,
            "Simpan barcode undangan digital Anda di menu \"Doa & RSVP\" untuk memudahkan registrasi meja resepsi.",
            "Keep the QR code from the \"Wishes & RSVP\" menu for a quick check-in at the reception desk.")),
        ("local_parking", tx!(l, "Parkir & Valet", "Parking & Valet"), tx!(l,
            "Layanan parkir tersedia di area lobi utama. Mohon datang 15 menit lebih awal.",
            "Parking is available at the main lobby. Please arrive 15 minutes early.")),
        ("photo_camera", "Photo Studio & Wishes Wall", tx!(l,
            "Abadikan kenangan manis bersama pengantin di area photobooth cetak instan.",
            "Capture sweet memories with the couple at the instant-print photo booth.")),
    ];
    view! {
        <section class="guide card card--soft">
            <h3 class="guide__title"><Icon name="verified_user" />{tx!(l, "Panduan Tamu & Fasilitas", "Guest Guide & Facilities")}</h3>
            {items.into_iter().map(|(icon, t, d)| view! {
                <div class="guide__item">
                    <span class="guide__icon"><span class="ms" aria-hidden="true">{icon}</span></span>
                    <div><b>{t}</b><p>{d}</p></div>
                </div>
            }).collect_view()}
        </section>
    }
}

/// Penutup "Terima Kasih" (bagian terakhir undangan, ala everlove): foto
/// bergerak berbingkai lengkung + salam penutup dari kedua mempelai.
#[component]
pub fn TerimaKasih(photos: Vec<String>, names: String) -> impl IntoView {
    let l = lang();
    let has_photo = photos.iter().any(|p| !p.is_empty());
    view! {
        <section class="section thanks">
            {has_photo.then(|| view! {
                <div class="arch-photo arch-photo--thanks">
                    <span class="arch-photo__clip"><FotoGerak photos=photos.clone() alt=names.clone() /></span>
                    <span class="arch-photo__frame" aria-hidden="true"></span>
                </div>
            })}
            <h2 class="script thanks__title">{tx!(l, "Terima Kasih", "Thank You")}</h2>
            <p class="thanks__text">{tx!(l,
                "Merupakan suatu kebahagiaan dan kehormatan bagi kami apabila Bapak/Ibu/Saudara/i berkenan hadir dan memberikan doa restu kepada kami.",
                "It would be a joy and an honour for us if you could attend and give us your blessings.")}</p>
            <p class="thanks__by">{tx!(l, "Kami yang berbahagia", "With love,")}</p>
            <p class="script thanks__names">{names}</p>
        </section>
    }
}

#[component]
pub fn Gallery(photos: Vec<String>) -> impl IntoView {
    let l = lang();
    (!photos.is_empty()).then(|| {
        view! {
            <section class="section orn-host">
                <Ornamen bagian="galeri" />
                <p class="eyebrow eyebrow--center">{tx!(l, "Momen Kebahagiaan", "Our Happy Moments")}</p>
                <h2 class="section__title">"Our Moments"</h2>
                <div class="gallery gallery--masonry">
                    {photos.into_iter().map(|src| view! {
                        <a class="gallery__item" href=src.split('#').next().unwrap_or("").to_string() target="_blank" rel="noopener">
                            <img src=src.clone() alt=tx!(l, "Galeri pengantin", "Wedding gallery") loading="lazy" decoding="async" style=fmt::photo_style(&src) />
                        </a>
                    }).collect_view()}
                </div>
            </section>
        }
    })
}

// ── RSVP & doa ─────────────────────────────────────────────────────────────

#[component]
pub fn RsvpForm(slug: String, guest: Option<GuestInfo>, sessions: Vec<String>, #[prop(optional)] on_done: Option<Callback<()>>) -> impl IntoView {
    let l = lang();
    let action = ServerAction::<SubmitRsvp>::new();
    let status = RwSignal::new("hadir".to_string());
    let pax = RwSignal::new(guest.as_ref().map(|g| g.pax.max(1)).unwrap_or(1));
    let code = guest.as_ref().map(|g| g.code.clone()).unwrap_or_default();
    let name = guest.as_ref().map(|g| g.name.clone()).unwrap_or_default();

    Effect::new(move |_| {
        if let Some(Ok(_)) = action.value().get() {
            if let Some(cb) = on_done {
                cb.run(());
            }
        }
    });

    let opts: [(&'static str, &'static str, &'static str); 3] =
        [("hadir", rsvp_label_in("hadir", l), "check_circle"), ("ragu", rsvp_label_in("ragu", l), "help"), ("tidak", rsvp_label_in("tidak", l), "cancel")];

    view! {
        <ActionForm action=action attr:class="rsvp card" attr:data-hydrate="">
            <input type="hidden" name="slug" value=slug />
            <input type="hidden" name="guest" value=code />
            <input type="hidden" name="lang" value=l.code() />
            <label class="field">
                <span class="field__label"><Icon name="person" />{tx!(l, "Nama Lengkap", "Full Name")}</span>
                <input class="input" name="name" required maxlength="80" value=name placeholder=tx!(l, "Nama Anda", "Your name") />
            </label>
            <label class="field">
                <span class="field__label"><Icon name="smartphone" />{tx!(l, "Nomor WhatsApp", "WhatsApp Number")}</span>
                <input class="input" name="phone" inputmode="tel" maxlength="20" placeholder="0812-xxxx-xxxx" />
            </label>
            <div class="field">
                <span class="field__label"><Icon name="how_to_reg" />{tx!(l, "Status Konfirmasi Kehadiran", "Will you attend?")}</span>
                <div class="seg">
                    {opts.into_iter().map(|(v, label, icon)| view! {
                        <label class="seg__opt" class:is-on=move || status.get() == v>
                            <input type="radio" name="status" value=v checked=v == "hadir" on:change=move |_| status.set(v.to_string()) />
                            <span class="ms" aria-hidden="true">{icon}</span>
                            <span>{label}</span>
                        </label>
                    }).collect_view()}
                </div>
            </div>
            <div class="field-row" class:is-hidden=move || status.get() == "tidak">
                <div class="field">
                    <span class="field__label">{tx!(l, "Jumlah Tamu", "Number of Guests")}</span>
                    <div class="stepper">
                        <button type="button" on:click=move |_| pax.update(|p| *p = (*p - 1).max(1)) aria-label=tx!(l, "Kurangi", "Fewer")>"−"</button>
                        <input name="pax" type="number" min="1" max="10" prop:value=move || pax.get().to_string()
                            on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse() { pax.set(v) } } />
                        <button type="button" on:click=move |_| pax.update(|p| *p = (*p + 1).min(10)) aria-label=tx!(l, "Tambah", "More")>"+"</button>
                    </div>
                </div>
                <label class="field">
                    <span class="field__label">{tx!(l, "Sesi Waktu", "Session")}</span>
                    <select class="input" name="session">
                        {sessions.into_iter().map(|s| view! { <option value=s.clone()>{s.clone()}</option> }).collect_view()}
                    </select>
                </label>
            </div>
            <label class="field">
                <span class="field__label"><Icon name="edit_note" />{tx!(l, "Doa & Ucapan Hangat", "Wishes & Blessings")}</span>
                <textarea class="input" name="message" rows="3" maxlength="600" placeholder=tx!(l, "Tuliskan ucapan dan doa terbaik untuk kedua mempelai...", "Write your warmest wishes for the couple...")></textarea>
            </label>
            <button class="btn btn--primary btn--block btn--lg" type="submit" disabled=move || action.pending().get()>
                <Icon name="send" />
                {move || if action.pending().get() { tx!(l, "Mengirim…", "Sending…") } else { tx!(l, "Kirim Konfirmasi & Doa Restu", "Send RSVP & Wishes") }}
            </button>
            {move || action.value().get().map(|r| match r {
                Ok(msg) => view! { <p class="notice notice--ok">{msg}</p> }.into_any(),
                Err(e) => view! { <p class="notice notice--err">{err_msg(&e)}</p> }.into_any(),
            })}
        </ActionForm>
    }
}

#[component]
pub fn GiftSection(inv: Invitation, guest: Option<GuestInfo>, #[prop(optional)] open: bool) -> impl IntoView {
    let l = lang();
    let action = ServerAction::<ConfirmGift>::new();
    let code = guest.as_ref().map(|g| g.code.clone()).unwrap_or_default();
    let name = guest.as_ref().map(|g| g.name.clone()).unwrap_or_default();
    let banks = inv.banks.clone();
    let bank_names: Vec<String> = banks.iter().map(|b| b.bank.clone()).collect();
    view! {
        <details class="gift card" open=open data-hydrate="">
            <summary class="gift__summary">
                <span class="gift__icon"><Icon name="redeem" /></span>
                <span>
                    <b>{tx!(l, "Tanda Kasih Pernikahan", "Wedding Gift")}</b>
                    <small>{tx!(l, "Amplop digital & kado fisik untuk mempelai", "Digital envelope & physical gifts for the couple")}</small>
                </span>
                <Icon name="expand_more" class="gift__chev" />
            </summary>
            <p class="gift__lead">{tx!(l,
                "Doa restu Anda merupakan karunia terindah bagi kami. Bagi yang ingin memberikan tanda kasih, dapat melalui:",
                "Your blessings are the greatest gift to us. If you wish to send a gift, you may do so through:")}</p>
            {banks.into_iter().map(|b| {
                let digits: String = b.number.chars().filter(|c| c.is_ascii_digit()).collect();
                view! {
                    <div class="bank">
                        <div>
                            <p class="eyebrow">{b.bank}</p>
                            <p class="bank__no">{b.number}</p>
                            <p class="bank__holder">{format!("{} {}", tx!(l, "a.n.", "Account name:"), b.holder)}</p>
                        </div>
                        <button type="button" class="btn btn--outline btn--sm" data-copy=digits data-copied=tx!(l, "Nomor rekening tersalin", "Account number copied")>
                            <Icon name="content_copy" />
                            {tx!(l, "Salin", "Copy")}
                        </button>
                    </div>
                }
            }).collect_view()}
            {(!inv.gift_address.is_empty()).then(|| view! {
                <div class="bank">
                    <div>
                        <p class="eyebrow">{tx!(l, "Kirim Kado Fisik", "Send a Physical Gift")}</p>
                        <p class="bank__holder">{inv.gift_address.clone()}</p>
                    </div>
                    <button type="button" class="btn btn--outline btn--sm" data-copy=inv.gift_address.clone() data-copied=tx!(l, "Alamat tersalin", "Address copied")>
                        <Icon name="content_copy" />
                        {tx!(l, "Salin", "Copy")}
                    </button>
                </div>
            })}
            <ActionForm action=action attr:class="gift__form">
                <p class="eyebrow eyebrow--gold">{tx!(l, "Konfirmasi Tanda Kasih", "Confirm Your Gift")}</p>
                <input type="hidden" name="slug" value=inv.slug.clone() />
                <input type="hidden" name="guest" value=code />
                <input type="hidden" name="lang" value=l.code() />
                <input class="input" name="name" required maxlength="80" value=name placeholder=tx!(l, "Nama pengirim", "Sender's name") />
                <div class="field-row">
                    <input class="input" name="amount" required inputmode="numeric" placeholder=tx!(l, "Nominal (Rp)", "Amount (IDR)") />
                    <select class="input" name="channel">
                        {bank_names.into_iter().map(|b| view! { <option>{b}</option> }).collect_view()}
                        <option>{tx!(l, "Kado Fisik", "Physical Gift")}</option>
                    </select>
                </div>
                <button class="btn btn--gold btn--block" type="submit" disabled=move || action.pending().get()>
                    <Icon name="favorite" />
                    {tx!(l, "Saya Sudah Mengirim", "I've Sent It")}
                </button>
                {move || action.value().get().map(|r| match r {
                    Ok(msg) => view! { <p class="notice notice--ok">{msg}</p> }.into_any(),
                    Err(e) => view! { <p class="notice notice--err">{err_msg(&e)}</p> }.into_any(),
                })}
            </ActionForm>
        </details>
    }
}

#[component]
pub fn WishList(slug: String, refresh: RwSignal<u32>) -> impl IntoView {
    let l = lang();
    let wishes = Resource::new(move || (slug.clone(), refresh.get()), |(s, _)| list_wishes(s));
    view! {
        <section class="wishes" data-hydrate="">
            <Suspense fallback=|| view! { <SkelRows n=3 /> }>
                {move || wishes.get().map(|r| match r {
                    Ok(p) => Either::Left(view! {
                        <div class="wishes__head">
                            <h3><Icon name="chat" />{tx!(l, "Doa & Ucapan", "Wishes")}</h3>
                            <span class="chip chip--gold chip--dot">{format!("{} {}", p.total, tx!(l, "Doa Terkirim", "Wishes Sent"))}</span>
                        </div>
                        {if p.items.is_empty() {
                            Either::Left(view! { <p class="muted center">{tx!(l, "Jadilah yang pertama mengirim doa restu.", "Be the first to send your wishes.")}</p> })
                        } else {
                            Either::Right(p.items.into_iter().enumerate().map(|(i, w)| view! {
                                <article class="wish card">
                                    <div class="wish__head">
                                        <span class=format!("avatar avatar--{}", i % 3)>{avatar_initials(&w.name)}</span>
                                        <div class="wish__who">
                                            <b>{w.name.clone()}</b>
                                            <small>{w.ago_in(l)}</small>
                                        </div>
                                        <span class=format!("status status--{}", w.status)>
                                            {(w.status == "hadir").then(|| view! { <Icon name="check" /> })}
                                            {rsvp_label_in(&w.status, l)}
                                        </span>
                                    </div>
                                    <p class="wish__msg">{w.message.clone()}</p>
                                </article>
                            }).collect_view())
                        }}
                    }),
                    Err(e) => Either::Right(view! { <p class="notice notice--err">{err_msg(&e)}</p> }),
                })}
            </Suspense>
        </section>
    }
}

/// Satu-satunya `<audio>` latar. Dikendalikan skrip global (app.rs).
#[component]
pub fn MusicAudio(url: String, #[prop(optional)] autoplay: bool, #[prop(optional)] looped: bool) -> impl IntoView {
    view! {
        <audio id="bgm" src=url preload="none" loop=looped data-autoplay=if autoplay { "true" } else { "false" }></audio>
    }
}

pub fn wa_share_text(inv: &Invitation, guest_name: &str, link: &str) -> String {
    format!(
        "Kepada Yth. Bapak/Ibu/Saudara/i\n*{guest_name}*\n\nTanpa mengurangi rasa hormat, kami mengundang Anda untuk hadir di acara pernikahan kami:\n\n*{}*\n{}\n\nInfo lengkap, konfirmasi kehadiran & doa restu:\n{link}\n\nMerupakan suatu kehormatan dan kebahagiaan bagi kami apabila Bapak/Ibu/Saudara/i berkenan hadir.\n\nTerima kasih.\n{}",
        inv.couple(),
        inv.date_label(),
        if inv.family_name.is_empty() { inv.couple() } else { inv.family_name.clone() },
    )
}

pub use super::fmt::rupiah;

/// Penggeser "mulai lagu dari detik …" untuk `<audio id=bgm>`; dihidupkan
/// skrip global (app.rs). `name` diisi = ikut terkirim bersama formulir.
#[component]
/// `start` = detik awal yang sudah tersimpan (halaman sunting); 0 = dari awal.
pub fn MusicSeek(#[prop(optional)] name: &'static str, #[prop(optional)] start: u32) -> impl IntoView {
    let mmss = format!("{}:{:02}", start / 60, start % 60);
    view! {
        <div class="seek" data-seek="">
            <div class="seek__head">
                <span class="field__label"><Icon name="schedule" />"Mulai lagu dari"</span>
                <output class="seek__time" data-seek-time="">{mmss}</output>
            </div>
            <input type="range" min="0" max=start.to_string() step="1" value=start.to_string() data-seek-range="" disabled aria-label="Mulai lagu dari detik" />
            <small class="seek__now" data-seek-now=""></small>
            <small class="muted" data-seek-hint="">"Putar lagu dulu, lalu geser ke bagian favorit (mis. reff). Tamu mendengar lagu mulai dari titik ini."</small>
            {(!name.is_empty()).then(|| view! { <input type="hidden" name=name value=start.to_string() data-seek-value="" /> })}
        </div>
    }
}

/// Nomor halaman yang ditampilkan; `0` = elipsis. ≤ 7 halaman → semua.
/// Lebih → 1 … (cur-1) cur (cur+1) … akhir, dan elipsis HANYA bila
/// menyembunyikan ≥ 2 nomor (satu nomor yang terselip ditampilkan saja).
pub fn page_numbers(cur: i64, pages: i64) -> Vec<i64> {
    if pages <= 7 {
        return (1..=pages).collect();
    }
    let lo = (cur - 1).clamp(2, pages - 1);
    let hi = (cur + 1).clamp(2, pages - 1);
    // Tepi: tetap 5 nomor tengah supaya lebar navigasi tak melompat-lompat.
    let (lo, hi) = if cur <= 4 { (2, 5) } else if cur >= pages - 3 { (pages - 4, pages - 1) } else { (lo, hi) };
    let mut out = vec![1];
    if lo > 2 {
        out.push(0);
    }
    out.extend(lo..=hi);
    if hi < pages - 1 {
        out.push(0);
    }
    out.push(pages);
    out
}

/// Navigasi halaman daftar story (Kelola & admin): ‹ 1 … 4 5 6 … 12 › +
/// "Menampilkan 13–24 dari 35 story". Pindah halaman → gulir ke `anchor`.
#[component]
pub fn StoryPager(page: RwSignal<i64>, cur: i64, pages: i64, total: i64, per: i64, #[prop(optional)] anchor: &'static str) -> impl IntoView {
    let go = move |p: i64| {
        page.set(p);
        #[cfg(target_arch = "wasm32")]
        if !anchor.is_empty() {
            if let Some(el) = web_sys::window().and_then(|w| w.document()).and_then(|d| d.get_element_by_id(anchor)) {
                el.scroll_into_view();
            }
        }
        let _ = anchor;
    };
    // Dihitung di luar view!: `>=` di atribut makro dibaca sebagai penutup tag.
    let (at_start, at_end) = (cur <= 1, cur >= pages);
    let from = if total == 0 { 0 } else { (cur - 1) * per + 1 };
    let to = (cur * per).min(total);
    view! {
        <div class="pager">
            <p class="pager__info">{format!("Menampilkan {from}–{to} dari {total} story")}</p>
            {(pages > 1).then(|| view! {
                <nav class="pager__nav" aria-label="Halaman">
                    <button type="button" class="pager__btn" disabled=at_start on:click=move |_| go(cur - 1) aria-label="Halaman sebelumnya"><Icon name="chevron_left" /></button>
                    {page_numbers(cur, pages).into_iter().map(|p| if p == 0 {
                        view! { <span class="pager__gap">"…"</span> }.into_any()
                    } else {
                        view! {
                            <button type="button" class="pager__btn" class:is-on=p == cur aria-current=(p == cur).then_some("page") on:click=move |_| go(p)>{p.to_string()}</button>
                        }.into_any()
                    }).collect_view()}
                    <button type="button" class="pager__btn" disabled=at_end on:click=move |_| go(cur + 1) aria-label="Halaman berikutnya"><Icon name="chevron_right" /></button>
                </nav>
            })}
        </div>
    }
}

#[cfg(test)]
mod pager_tests {
    #[test]
    fn nomor_halaman_dengan_elipsis() {
        assert_eq!(super::page_numbers(1, 1), vec![1]);
        assert_eq!(super::page_numbers(1, 4), vec![1, 2, 3, 4], "4 halaman: semua tampil, tanpa …");
        assert_eq!(super::page_numbers(3, 7), vec![1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(super::page_numbers(1, 12), vec![1, 2, 3, 4, 5, 0, 12]);
        assert_eq!(super::page_numbers(6, 12), vec![1, 0, 5, 6, 7, 0, 12]);
        assert_eq!(super::page_numbers(12, 12), vec![1, 0, 8, 9, 10, 11, 12]);
        // Elipsis tak pernah menyembunyikan satu nomor saja.
        for pages in 1..40 {
            for cur in 1..=pages {
                let v = super::page_numbers(cur, pages);
                assert!(v.contains(&cur) && v[0] == 1 && *v.last().unwrap() == pages, "{cur}/{pages}: {v:?}");
                for w in v.windows(3) {
                    if w[1] == 0 {
                        assert!(w[2] - w[0] > 2, "{cur}/{pages}: … hanya menutupi satu nomor {v:?}");
                    }
                }
            }
        }
    }
}

/// Pilihan bahasa undangan untuk tamu — dipakai /buat & /kelola/…/sunting.
/// Teks bawaan (sapaan, judul bagian, tombol, tanggal) mengikuti pilihan ini;
/// tamu tetap bisa mengganti sendiri lewat tombol ID/EN di undangan.
#[component]
pub fn BahasaUndangan(#[prop(optional)] current: crate::web::i18n::Lang) -> impl IntoView {
    use crate::web::i18n::Lang;
    view! {
        <fieldset class="field lang-pick">
            <legend class="field__label">"Bahasa undangan untuk tamu"</legend>
            <label class="check"><input type="radio" name="lang" value="id" checked=current == Lang::Id /><span>"Bahasa Indonesia"</span></label>
            <label class="check"><input type="radio" name="lang" value="en" checked=current == Lang::En /><span>"English — untuk tamu dari luar negeri"</span></label>
            <small class="muted">"Teks yang Anda tulis sendiri (nama, kutipan, kisah) tidak diterjemahkan. Tamu tetap bisa mengganti bahasa di undangan."</small>
        </fieldset>
    }
}
