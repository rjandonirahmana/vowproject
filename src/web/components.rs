//! web/components.rs — potongan undangan yang dipakai ulang oleh halaman tamu
//! (/u/…), pratinjau tema (/tema/…), dan katalog.

use leptos::either::Either;
use leptos::prelude::*;

use super::api::{list_wishes, ConfirmGift, SubmitRsvp};
use super::fmt;
use super::icons::Icon;
use super::model::*;

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
    let initials = html_escape(initials);
    let caption = html_escape(caption);
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

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

#[component]
pub fn Monogram(#[prop(into)] initials: String, #[prop(into, optional)] caption: String, #[prop(optional)] class: &'static str) -> impl IntoView {
    view! { <div class=format!("monogram {class}") inner_html=monogram_svg(&initials, &caption)></div> }
}

// ── Potongan undangan ──────────────────────────────────────────────────────

#[component]
pub fn Countdown(target_ms: i64, #[prop(optional)] title: &'static str) -> impl IntoView {
    let title = if title.is_empty() { "Menuju Hari Bahagia" } else { title };
    view! {
        <div class="countdown card card--soft" data-countdown=target_ms.to_string()>
            <p class="eyebrow eyebrow--gold">
                <Icon name="hourglass_top" />
                {title}
            </p>
            <div class="countdown__grid">
                <div class="countdown__tile"><b data-cd="d">"0"</b><span>"Hari"</span></div>
                <div class="countdown__tile"><b data-cd="h">"00"</b><span>"Jam"</span></div>
                <div class="countdown__tile"><b data-cd="m">"00"</b><span>"Menit"</span></div>
                <div class="countdown__tile"><b data-cd="s">"00"</b><span>"Detik"</span></div>
            </div>
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
                            Either::Right(view! { <img src=photo.clone() alt=name.clone() loading="lazy" decoding="async" style=fmt::photo_style(&photo) /> })
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
            <p class="eyebrow eyebrow--center">"Kedua Mempelai"</p>
            <h2 class="section__title">"Insan yang Menyatukan Janji"</h2>
            <div class="couple">
                {person("Mempelai Wanita", inv.bride_nick, inv.bride_name, inv.bride_degree, inv.bride_parents, inv.bride_ig, inv.bride_photo)}
                <span class="couple__amp" aria-hidden="true">"&"</span>
                {person("Mempelai Pria", inv.groom_nick, inv.groom_name, inv.groom_degree, inv.groom_parents, inv.groom_ig, inv.groom_photo)}
            </div>
        </section>
    }
}

/// Kisah cinta dalam garis waktu.
#[component]
pub fn LoveStorySection(items: Vec<LoveStory>) -> impl IntoView {
    (!items.is_empty()).then(|| {
        view! {
            <section class="section orn-host">
                <Ornamen bagian="kisah" />
                <p class="eyebrow eyebrow--center">"Perjalanan Kami"</p>
                <h2 class="section__title">"Love Story"</h2>
                <ol class="story card">
                    {items.into_iter().map(|it| view! {
                        <li class="story__item">
                            <span class="story__dot" aria-hidden="true"><Icon name="favorite" /></span>
                            <div>
                                {(!it.year.is_empty()).then(|| view! { <span class="story__year">{it.year.clone()}</span> })}
                                <h3>{it.title.clone()}</h3>
                                <p>{it.text.clone()}</p>
                            </div>
                        </li>
                    }).collect_view()}
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

#[component]
pub fn EventCard(ev: Event, #[prop(optional_no_strip)] dress: Option<(String, Vec<DressColor>)>) -> impl IntoView {
    let is_resepsi = ev.kind == "resepsi";
    let maps = ev.maps_link();
    let date = ev.date_label();
    let time = ev.time_label();
    let sessions = ev.sessions.clone();
    view! {
        <article class=if is_resepsi { "event card event--resepsi" } else { "event card" }>
            <div class="event__top">
                <span class=if is_resepsi { "chip chip--gold" } else { "chip" }>
                    <Icon name=if is_resepsi { "celebration" } else { "auto_stories" } />
                    {ev.badge.clone()}
                </span>
                <span class="event__tag">{ev.tag.clone()}</span>
            </div>
            <h3 class="event__title">{ev.title.clone()}</h3>
            <p class="event__meta">
                <Icon name="calendar_month" />
                <span>{date}</span>
            </p>
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
                    <p class="eyebrow eyebrow--gold"><Icon name="checkroom" />"Panduan Busana / Dress Code"</p>
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
                {if is_resepsi { "Buka Google Maps Resepsi" } else { "Buka Petunjuk Google Maps" }}
            </a>
        </article>
    }
}

#[component]
pub fn MapCard(ev: Event) -> impl IntoView {
    let addr = format!("{}, {}", ev.venue, ev.address);
    view! {
        <section class="mapcard card">
            <div class="mapcard__head">
                <Icon name="location_on" />
                <div>
                    <h3>"Peta Lokasi Acara"</h3>
                    <p>{ev.venue.clone()}</p>
                </div>
            </div>
            <div class="mapcard__frame">
                <iframe src=ev.maps_embed() title="Peta lokasi"></iframe>
            </div>
            <div class="mapcard__actions">
                <button type="button" class="btn btn--soft" data-copy=addr data-copied="Alamat tersalin">
                    <Icon name="content_copy" />
                    "Salin Alamat"
                </button>
                <a class="btn btn--primary" href=ev.maps_link() target="_blank" rel="noopener">
                    <Icon name="near_me" />
                    "Petunjuk Arah"
                </a>
            </div>
        </section>
    }
}

#[component]
pub fn GuestGuide() -> impl IntoView {
    let items: [(&'static str, &'static str, &'static str); 3] = [
        ("qr_code_2", "QR Code Check-in", "Simpan barcode undangan digital Anda di menu \"Doa & RSVP\" untuk memudahkan registrasi meja resepsi."),
        ("local_parking", "Parkir & Valet", "Layanan parkir tersedia di area lobi utama. Mohon datang 15 menit lebih awal."),
        ("photo_camera", "Photo Studio & Wishes Wall", "Abadikan kenangan manis bersama pengantin di area photobooth cetak instan."),
    ];
    view! {
        <section class="guide card card--soft">
            <h3 class="guide__title"><Icon name="verified_user" />"Panduan Tamu & Fasilitas"</h3>
            {items.into_iter().map(|(icon, t, d)| view! {
                <div class="guide__item">
                    <span class="guide__icon"><span class="ms" aria-hidden="true">{icon}</span></span>
                    <div><b>{t}</b><p>{d}</p></div>
                </div>
            }).collect_view()}
        </section>
    }
}

#[component]
pub fn Gallery(photos: Vec<String>) -> impl IntoView {
    (!photos.is_empty()).then(|| {
        view! {
            <section class="section orn-host">
                <Ornamen bagian="galeri" />
                <p class="eyebrow eyebrow--center">"Momen Kebahagiaan"</p>
                <h2 class="section__title">"Our Moments"</h2>
                <div class="gallery gallery--masonry">
                    {photos.into_iter().map(|src| view! {
                        <a class="gallery__item" href=src.split('#').next().unwrap_or("").to_string() target="_blank" rel="noopener">
                            <img src=src.clone() alt="Galeri pengantin" loading="lazy" decoding="async" style=fmt::photo_style(&src) />
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
        [("hadir", "Hadir", "check_circle"), ("ragu", "Masih Ragu", "help"), ("tidak", "Berhalangan", "cancel")];

    view! {
        <ActionForm action=action attr:class="rsvp card">
            <input type="hidden" name="slug" value=slug />
            <input type="hidden" name="guest" value=code />
            <label class="field">
                <span class="field__label"><Icon name="person" />"Nama Lengkap"</span>
                <input class="input" name="name" required maxlength="80" value=name placeholder="Nama Anda" />
            </label>
            <label class="field">
                <span class="field__label"><Icon name="smartphone" />"Nomor WhatsApp"</span>
                <input class="input" name="phone" inputmode="tel" maxlength="20" placeholder="0812-xxxx-xxxx" />
            </label>
            <div class="field">
                <span class="field__label"><Icon name="how_to_reg" />"Status Konfirmasi Kehadiran"</span>
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
                    <span class="field__label">"Jumlah Tamu"</span>
                    <div class="stepper">
                        <button type="button" on:click=move |_| pax.update(|p| *p = (*p - 1).max(1)) aria-label="Kurangi">"−"</button>
                        <input name="pax" type="number" min="1" max="10" prop:value=move || pax.get().to_string()
                            on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse() { pax.set(v) } } />
                        <button type="button" on:click=move |_| pax.update(|p| *p = (*p + 1).min(10)) aria-label="Tambah">"+"</button>
                    </div>
                </div>
                <label class="field">
                    <span class="field__label">"Sesi Waktu"</span>
                    <select class="input" name="session">
                        {sessions.into_iter().map(|s| view! { <option value=s.clone()>{s.clone()}</option> }).collect_view()}
                    </select>
                </label>
            </div>
            <label class="field">
                <span class="field__label"><Icon name="edit_note" />"Doa & Ucapan Hangat"</span>
                <textarea class="input" name="message" rows="3" maxlength="600" placeholder="Tuliskan ucapan dan doa terbaik untuk kedua mempelai..."></textarea>
            </label>
            <button class="btn btn--primary btn--block btn--lg" type="submit" disabled=move || action.pending().get()>
                <Icon name="send" />
                {move || if action.pending().get() { "Mengirim…" } else { "Kirim Konfirmasi & Doa Restu" }}
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
    let action = ServerAction::<ConfirmGift>::new();
    let code = guest.as_ref().map(|g| g.code.clone()).unwrap_or_default();
    let name = guest.as_ref().map(|g| g.name.clone()).unwrap_or_default();
    let banks = inv.banks.clone();
    let bank_names: Vec<String> = banks.iter().map(|b| b.bank.clone()).collect();
    view! {
        <details class="gift card" open=open>
            <summary class="gift__summary">
                <span class="gift__icon"><Icon name="redeem" /></span>
                <span>
                    <b>"Tanda Kasih Pernikahan"</b>
                    <small>"Amplop digital & kado fisik untuk mempelai"</small>
                </span>
                <Icon name="expand_more" class="gift__chev" />
            </summary>
            <p class="gift__lead">"Doa restu Anda merupakan karunia terindah bagi kami. Bagi yang ingin memberikan tanda kasih, dapat melalui:"</p>
            {banks.into_iter().map(|b| {
                let digits: String = b.number.chars().filter(|c| c.is_ascii_digit()).collect();
                view! {
                    <div class="bank">
                        <div>
                            <p class="eyebrow">{b.bank}</p>
                            <p class="bank__no">{b.number}</p>
                            <p class="bank__holder">{format!("a.n. {}", b.holder)}</p>
                        </div>
                        <button type="button" class="btn btn--outline btn--sm" data-copy=digits data-copied="Nomor rekening tersalin">
                            <Icon name="content_copy" />
                            "Salin"
                        </button>
                    </div>
                }
            }).collect_view()}
            {(!inv.gift_address.is_empty()).then(|| view! {
                <div class="bank">
                    <div>
                        <p class="eyebrow">"Kirim Kado Fisik"</p>
                        <p class="bank__holder">{inv.gift_address.clone()}</p>
                    </div>
                    <button type="button" class="btn btn--outline btn--sm" data-copy=inv.gift_address.clone() data-copied="Alamat tersalin">
                        <Icon name="content_copy" />
                        "Salin"
                    </button>
                </div>
            })}
            <ActionForm action=action attr:class="gift__form">
                <p class="eyebrow eyebrow--gold">"Konfirmasi Tanda Kasih"</p>
                <input type="hidden" name="slug" value=inv.slug.clone() />
                <input type="hidden" name="guest" value=code />
                <input class="input" name="name" required maxlength="80" value=name placeholder="Nama pengirim" />
                <div class="field-row">
                    <input class="input" name="amount" required inputmode="numeric" placeholder="Nominal (Rp)" />
                    <select class="input" name="channel">
                        {bank_names.into_iter().map(|b| view! { <option>{b}</option> }).collect_view()}
                        <option>"Kado Fisik"</option>
                    </select>
                </div>
                <button class="btn btn--gold btn--block" type="submit" disabled=move || action.pending().get()>
                    <Icon name="favorite" />
                    "Saya Sudah Mengirim"
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
    let wishes = Resource::new(move || (slug.clone(), refresh.get()), |(s, _)| list_wishes(s));
    view! {
        <section class="wishes">
            <Suspense fallback=|| view! { <p class="muted center">"Memuat doa & ucapan…"</p> }>
                {move || wishes.get().map(|r| match r {
                    Ok(p) => Either::Left(view! {
                        <div class="wishes__head">
                            <h3><Icon name="chat" />"Doa & Ucapan"</h3>
                            <span class="chip chip--gold chip--dot">{format!("{} Doa Terkirim", p.total)}</span>
                        </div>
                        {if p.items.is_empty() {
                            Either::Left(view! { <p class="muted center">"Jadilah yang pertama mengirim doa restu."</p> })
                        } else {
                            Either::Right(p.items.into_iter().enumerate().map(|(i, w)| view! {
                                <article class="wish card">
                                    <div class="wish__head">
                                        <span class=format!("avatar avatar--{}", i % 3)>{avatar_initials(&w.name)}</span>
                                        <div class="wish__who">
                                            <b>{w.name.clone()}</b>
                                            <small>{w.ago.clone()}</small>
                                        </div>
                                        <span class=format!("status status--{}", w.status)>
                                            {(w.status == "hadir").then(|| view! { <Icon name="check" /> })}
                                            {rsvp_label(&w.status)}
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
pub fn MusicSeek(#[prop(optional)] name: &'static str) -> impl IntoView {
    view! {
        <div class="seek" data-seek="">
            <div class="seek__head">
                <span class="field__label"><Icon name="schedule" />"Mulai lagu dari"</span>
                <output class="seek__time" data-seek-time="">"0:00"</output>
            </div>
            <input type="range" min="0" max="0" step="1" value="0" data-seek-range="" disabled aria-label="Mulai lagu dari detik" />
            <small class="seek__now" data-seek-now=""></small>
            <small class="muted" data-seek-hint="">"Putar lagu dulu, lalu geser ke bagian favorit (mis. reff). Tamu mendengar lagu mulai dari titik ini."</small>
            {(!name.is_empty()).then(|| view! { <input type="hidden" name=name value="0" data-seek-value="" /> })}
        </div>
    }
}
