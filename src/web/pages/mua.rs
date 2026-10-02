//! pages/mua.rs — /mua: tata rias pengantin (MUA) Tawangmangu, Karanganyar &
//! Solo (desain Stitch "Layanan MUA Make Up Pengantin").

use leptos::prelude::*;
use leptos_meta::{Meta, Title};

use crate::web::icons::Icon;
use crate::web::konten::{icon_line, Konten};
use crate::web::layanan::MUA_LOKASI;

use super::layanan::{paket_dari_query, wa_href, Quotes, ReservasiForm, SecHead, VendorCard, WithKonten};
use super::{SiteFooter, SiteHeader};

struct Poin {
    icon: &'static str,
    title: &'static str,
    text: &'static str,
}

const HERO_POIN: &[Poin] = &[
    Poin { icon: "diamond", title: "Produk high-end", text: "Dior, Charlotte Tilbury, MAC, Estée Lauder & Chanel." },
    Poin { icon: "thermostat", title: "16 jam flawless", text: "Formula anti-crack khusus suhu dingin & embun Tawangmangu." },
    Poin { icon: "local_florist", title: "Melati grade A", text: "Ronce melati basah segar & hair/hijab do rapi." },
];

/// (ikon, judul, isi, catatan kaki)
const ALASAN: &[(Poin, &str)] = &[
    (
        Poin { icon: "directions_car", title: "Bebas biaya transport", text: "Tanpa biaya tambahan untuk Kota Solo, Karanganyar, Colomadu, Ngargoyoso, hingga resort puncak Tawangmangu. Tim MUA & asisten tiba tepat waktu sebelum subuh." },
        "Berlaku untuk seluruh paket pengantin utama",
    ),
    (
        Poin { icon: "ac_unit", title: "Complexion khusus udara dingin", text: "Tawangmangu sering berkabut dan dingin (14–19°C) sehingga foundation rentan cakey atau mengelupas. Kami memakai teknik layering hidrasi intensif dan setting seal water-resistant." },
        "Teruji di 140+ pernikahan villa Tawangmangu",
    ),
    (
        Poin { icon: "event_available", title: "Gratis trial / tes makeup", text: "Sesi uji coba riasan lengkap (complexion, tes tone lipstik, konsultasi busana adat) di studio kami di Karanganyar agar mempelai tenang di hari bahagia." },
        "Tersedia pada Paket All-In & Solo Basahan",
    ),
];

#[component]
pub fn MuaPage() -> impl IntoView {
    view! {
        <Title text=concat!("MUA & Rias Pengantin Tawangmangu, Karanganyar & Solo — ", crate::brand!()) />
        <Meta name="description" content="Tata rias pengantin profesional: Solo Putri, Basahan, Paes Ageng, hingga modern glowing bride. Tahan udara dingin Tawangmangu, bebas biaya transport Solo Raya." />
        <div class="site">
            <SiteHeader active="mua" />
            <WithKonten view=mua_body />
            <SiteFooter />
        </div>
    }
}

fn mua_body(k: Konten) -> impl IntoView {
    let paket = paket_dari_query(
        k.mua_paket.iter().map(|p| p.slug.clone()).collect(),
        k.mua_paket.first().map(|p| p.slug.clone()).unwrap_or_default(),
        "reservasi",
    );
    let u = k.umum.clone();
    let prof = k.mua_profil.clone();
    let g = k.mua_galeri.clone();
    view! {
            <div class="lx-strip">
                <span><Icon name="location_on" />"Tawangmangu • Karanganyar • Kota Surakarta (Solo)"</span>
                <span><Icon name="check_circle" />"Bebas biaya transportasi lokasi"</span>
            </div>

            <section class="lx-hero">
                <div class="lx-hero__text">
                    <span class="chip chip--soft"><Icon name="spa" />"Atelier Rias Pengantin Solo Raya"</span>
                    <h1>"Tata Rias Pengantin Anggun, "<em>"MUA Profesional"</em>" Tawangmangu, Karanganyar & Solo"</h1>
                    <p class="lx-hero__lead">
                        "Menampilkan kecantikan autentik dan aura pengantin yang "<em>"manglingi"</em>
                        ". Ahli rias tradisional Solo Putri / Basahan, Paes Ageng, Sunda Siger, hingga modern flawless wedding look yang tahan udara sejuk pegunungan."
                    </p>
                    <div class="lx-mini-feats lx-mini-feats--3">
                        {HERO_POIN.iter().map(|k| view! {
                            <div class="lx-mini-feat">
                                <span class="ms" aria-hidden="true">{k.icon}</span>
                                <b>{k.title}</b>
                                <small>{k.text}</small>
                            </div>
                        }).collect_view()}
                    </div>
                    <div class="lx-hero__btns">
                        <a class="btn btn--primary" href="#reservasi"><Icon name="calendar_month" />"Cek Tanggal & Konsultasi"</a>
                        <a class="btn btn--outline" href="#galeri"><Icon name="photo_camera" />"Lihat Hasil Paes & Rias"</a>
                    </div>
                </div>
                <div class="lx-hero__media lx-hero__media--tall">
                    <img src=u.mua_hero.clone() alt="Pengantin Solo Putri dengan paes dan cunduk mentul" fetchpriority="high" />
                    <span class="chip chip--gold lx-hero__chip"><Icon name="verified" />"Paes Halus"</span>
                    <div class="lx-float">
                        <span class="lx-float__icon"><Icon name="water_drop" /></span>
                        <span><b>"Tahan suhu dingin 14°C"</b><small>"Skin-prep pelembap khusus pegunungan"</small></span>
                    </div>
                </div>
            </section>

            <section class="lx-sec">
                <SecHead eyebrow="Keistimewaan Layanan Wilayah" title="Mengapa Pengantin di Lereng Lawu & Solo Memilih Kami?"
                    lead="Setiap lokasi pernikahan punya tantangan mikroklimat dan tradisi yang khas. Kami menyiapkan teknik serta dedikasi tanpa kompromi." />
                <div class="lx-grid lx-grid--3">
                    {ALASAN.iter().map(|(k, foot)| view! {
                        <article class="card lx-why">
                            <span class="lx-why__icon"><span class="ms" aria-hidden="true">{k.icon}</span></span>
                            <h3>{k.title}</h3>
                            <p>{k.text}</p>
                            <small>{*foot}</small>
                        </article>
                    }).collect_view()}
                </div>
            </section>

            <section class="lx-sec" id="paket-mua">
                <SecHead eyebrow="Katalog Paket Tata Rias" title="Pilihan Paket Rias Pengantin & Keluarga"
                    lead="Transparan, memakai kosmetik berstandar internasional, dan dikerjakan MUA bersertifikat resmi." />
                <div class="lx-grid lx-grid--4">
                    {k.mua_paket.clone().into_iter().map(|p| {
                        let href = format!("/mua?paket={}#reservasi", p.slug);
                        view! { <VendorCard p=p href=href /> }
                    }).collect_view()}
                </div>
            </section>

            <section class="lx-sec" id="galeri">
                <SecHead eyebrow="Detail & Presisi" title="Galeri Transformasi & Sentuhan Mahakarya"
                    lead="Keanggunan lahir dari garis paes yang simetris, bulu mata yang tidak memberatkan kelopak, dan kilau kulit yang memancarkan pesona." />
                <div class="lx-gallery-mua">
                    {mua_gallery(g)}
                </div>
            </section>

            <section class="lx-sec">
                <div class="card lx-profile">
                    <div class="lx-profile__img">
                        <img src=prof.img.clone() alt=prof.name.clone() loading="lazy" decoding="async" />
                        {(!prof.badge.is_empty()).then(|| view! { <span class="chip chip--gold"><Icon name="workspace_premium" />{prof.badge.clone()}</span> })}
                    </div>
                    <div class="lx-profile__body">
                        <p class="eyebrow eyebrow--gold">"Profil Lead Makeup Artist"</p>
                        <h2>{prof.name.clone()}</h2>
                        <p class="lx-profile__role">{prof.role.clone()}</p>
                        <p>{prof.bio.clone()}</p>
                        <div class="lx-grid lx-grid--2 lx-grid--tight">
                            {prof.prestasi.iter().map(|line| {
                                let (icon, rest) = icon_line(line, "verified");
                                let (title, sub) = rest.split_once('|').map(|(a, b)| (a.trim().to_string(), b.trim().to_string())).unwrap_or((rest.to_string(), String::new()));
                                view! {
                                    <div class="lx-tile lx-tile--row">
                                        <span class="ms" aria-hidden="true">{icon.to_string()}</span>
                                        <span><b>{title}</b><small>{sub}</small></span>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                        {(!prof.quote.is_empty()).then(|| view! { <blockquote class="lx-profile__quote">{format!("“{}”", prof.quote)}</blockquote> })}
                    </div>
                </div>
            </section>

            <section class="lx-sec">
                <SecHead eyebrow="Cerita & Kebahagiaan" title="Apa Kata Pengantin Kami?" />
                <Quotes items=k.mua_testimoni.clone() />
            </section>

            <section class="lx-sec lx-reserve" id="reservasi">
                <div class="lx-reserve__info">
                    <p class="eyebrow lx-cta__eyebrow">"Jadwal Tanggal Pernikahan"</p>
                    <h2>"Cek Ketersediaan Tanggal & Konsultasi Langsung"</h2>
                    <p>"Demi menjaga kualitas dan ketenangan mempelai, kami hanya menerima "<b>{format!("maksimal {} pengantin per hari", u.mua_maks_per_hari)}</b>" di seluruh wilayah Solo, Karanganyar, dan Tawangmangu."</p>
                    <div class="lx-slot">
                        <div class="lx-slot__top"><b>"Status slot musim nikah"</b><span class="chip chip--gold chip--xs">"Diperbarui berkala"</span></div>
                        <div class="lx-slot__top"><span>"Slot tersedia:"</span><b>{format!("{} tanggal", u.mua_slot_sisa)}</b></div>
                        <div class="lx-slot__bar" style=format!("--p:{}%", u.mua_slot_persen.clamp(0, 100))><i></i></div>
                        <small>{format!("{}% terisi", u.mua_slot_persen.clamp(0, 100))}</small>
                    </div>
                    <p class="lx-reserve__meta"><Icon name="schedule" />"Respon cepat WhatsApp: 08.00 – 21.00 WIB"</p>
                    <p class="lx-reserve__meta"><Icon name="location_on" />"Studio fitting di Karanganyar (alamat via WhatsApp)"</p>
                    <a class="btn btn--gold" href=wa_href("mua") target="_blank" rel="noopener"><Icon name="chat" />"Chat Langsung via WhatsApp"</a>
                </div>
                <ReservasiForm
                    layanan="mua"
                    title="Formulir Pra-Reservasi Tanggal"
                    lead="Isi singkat, penawaran resmi, katalog busana, dan konfirmasi jadwal MUA kami kirim via WhatsApp."
                    nama_label="Nama lengkap calon pengantin"
                    nama_ph="Contoh: Roro Anindita"
                    lokasi_label="Lokasi acara"
                    lokasi=MUA_LOKASI
                    pakets=k.mua_paket.clone()
                    paket=paket
                    catatan_label="Catatan khusus (konsep gaun / paes / kebutuhan tambahan)"
                    submit="Kirim Reservasi ke WhatsApp"
                />
            </section>

            <section class="lx-sec">
                <div class="lx-coverage">
                    <span><Icon name="map" />"Jangkauan rias mobile studio: hotel resort Tawangmangu, ballroom Solo, hingga rumah mempelai di seluruh eks-Karesidenan Surakarta."</span>
                </div>
            </section>
    }
}

/// Galeri rias: foto 1 = kartu besar, 2 & 3 = pasangan sebelum/sesudah,
/// sisanya kartu biasa (urutan diatur admin di /admin/konten/mua_galeri).
fn mua_gallery(g: Vec<crate::web::konten::Foto>) -> impl IntoView {
    let card = |f: crate::web::konten::Foto, side: bool| {
        view! {
            <article class="card lx-gcard" class:lx-gcard--side=side>
                <img src=f.img.clone() alt=f.judul.clone() loading="lazy" decoding="async" />
                <div>
                    <p class="eyebrow eyebrow--gold">{f.kecil.clone()}</p>
                    <h3>{f.judul.clone()}</h3>
                    <p>{f.teks.clone()}</p>
                </div>
            </article>
        }
        .into_any()
    };
    let mut it = g.into_iter();
    let mut out = Vec::new();
    if let Some(f) = it.next() {
        out.push(card(f, true));
    }
    let pair: Vec<_> = it.by_ref().take(2).collect();
    if !pair.is_empty() {
        out.push(
            view! {
                <article class="card lx-ba">
                    {pair.into_iter().map(|f| view! {
                        <figure>
                            <img src=f.img.clone() alt=f.kecil.clone() loading="lazy" decoding="async" />
                            <figcaption><b>{f.kecil.clone()}</b>{f.teks.clone()}</figcaption>
                        </figure>
                    }).collect_view()}
                </article>
            }
            .into_any(),
        );
    }
    out.extend(it.map(|f| card(f, false)));
    out
}
