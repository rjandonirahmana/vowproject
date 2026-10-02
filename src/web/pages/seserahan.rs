//! pages/seserahan.rs — /seserahan: sewa wadah seserahan (kotak akrilik,
//! kayu jati ukir, rotan) + jasa hias, antar-jemput, dan kalkulator biaya.
//!
//! Semua paket, harga, layanan tambahan, tarif antar, galeri, ketentuan, dan
//! testimoni berasal dari `Konten` (bisa disunting di /admin/konten, grup
//! "Sewa Seserahan"). Formulir = GET /layanan/wa biasa; server menghitung
//! ulang estimasi dari query (web/layanan.rs `SeserahanInput`).

use leptos::prelude::*;
use leptos_meta::{Meta, Title};

use crate::web::fmt::rupiah;
use crate::web::icons::Icon;
use crate::web::konten::{icon_line, Konten};
use crate::web::layanan::SeserahanInput;
use crate::web::skin::is_safe_url;

use super::layanan::{paket_dari_query, wa_href, Quotes, SecHead, WithKonten};
use super::{SiteFooter, SiteHeader};

struct Poin {
    icon: &'static str,
    title: &'static str,
    text: &'static str,
}

const HERO_POIN: &[Poin] = &[
    Poin { icon: "inventory_2", title: "Wadah bersih & terawat", text: "Dicuci, dipoles, dan dicek retak sebelum setiap sewa" },
    Poin { icon: "local_florist", title: "Bisa sekalian dihias", text: "Isi dari Anda, kami tata rapi lengkap dengan bunga" },
    Poin { icon: "local_shipping", title: "Antar-jemput H-1", text: "Tawangmangu, Karanganyar, hingga Solo Raya" },
];

const LANGKAH: &[Poin] = &[
    Poin { icon: "fact_check", title: "1. Pilih paket & cek tanggal", text: "Hitung biaya di kalkulator, lalu kirim ke WhatsApp admin untuk cek ketersediaan." },
    Poin { icon: "event_available", title: "2. DP mengunci tanggal", text: "Tanggal Anda aman setelah DP. Kirim daftar isi seserahan bila memakai jasa hias." },
    Poin { icon: "local_shipping", title: "3. Wadah diterima H-1", text: "Diantar atau diambil di studio. Pelunasan + deposit dibayar saat wadah diterima." },
    Poin { icon: "savings", title: "4. Kembalikan, deposit kembali", text: "Wadah dijemput/dikembalikan H+1. Deposit ditransfer balik setelah dicek utuh." },
];

#[component]
pub fn SeserahanPage() -> impl IntoView {
    view! {
        <Title text=concat!("Sewa Seserahan & Hantaran Tawangmangu – Solo — ", crate::brand!()) />
        <Meta name="description" content="Sewa kotak seserahan akrilik, akrilik gold mirror, kayu jati ukir adat Jawa, dan rotan rustic. Jasa hias isi, bunga segar, antar-jemput Tawangmangu, Karanganyar & Solo Raya. Hitung biaya langsung." />
        <div class="site">
            <SiteHeader active="seserahan" />
            <WithKonten view=seserahan_body />
            <SiteFooter />
        </div>
    }
}

fn seserahan_body(k: Konten) -> impl IntoView {
    let info = k.seserahan_info.clone();
    let hero = if is_safe_url(&info.hero) { info.hero.clone() } else { String::new() };
    let min_harga = k.seserahan_paket.iter().map(|p| p.price).filter(|p| *p > 0).min();
    view! {
        <section class="lx-hero-dark" style=format!("--bg-img:url({hero})")>
            <div class="lx-hero-dark__in">
                <span class="chip chip--gold"><Icon name="redeem" />"Sewa Seserahan & Hantaran • Solo Raya"</span>
                <h1>"Sewa Seserahan Cantik, Tanpa Repot Beli Kotak"</h1>
                <p>
                    "Kotak akrilik bening, gold mirror, kayu jati ukir adat Jawa, hingga rotan rustic — lengkap dengan jasa hias isi dan antar-jemput. "
                    {min_harga.map(|m| format!("Mulai {} per set, masa sewa {}.", rupiah(m), info.lama_sewa))}
                </p>
                <div class="lx-hero__btns">
                    <a class="btn btn--gold" href="#kalkulator"><Icon name="calculate" />"Hitung Biaya Sewa"</a>
                    <a class="btn btn--glass" href="#paket-seserahan"><Icon name="expand_more" />"Lihat Pilihan Wadah"</a>
                </div>
                <div class="lx-hero-dark__feats">
                    {HERO_POIN.iter().map(|p| view! {
                        <div>
                            <span class="ms" aria-hidden="true">{p.icon}</span>
                            <span><b>{p.title}</b><small>{p.text}</small></span>
                        </div>
                    }).collect_view()}
                </div>
            </div>
        </section>

        <section class="lx-sec" id="paket-seserahan">
            <SecHead eyebrow="Pilihan Wadah Hantaran" title="Paket Sewa Seserahan"
                lead="Harga untuk satu set selama masa sewa. Deposit dikembalikan penuh setelah wadah kembali utuh." />
            <div class="lx-grid lx-grid--4">
                {k.seserahan_paket.clone().into_iter().map(|p| view! {
                    <article class="card lx-pcard sh-card" class:lx-pcard--pop=p.popular>
                        {(!p.badge.is_empty()).then(|| view! { <span class="chip chip--gold lx-pcard__flag">{p.badge.clone()}</span> })}
                        <div class="lx-pcard__img"><img src=p.img.clone() alt=p.name.clone() loading="lazy" decoding="async" /></div>
                        <div class="lx-pcard__meta">
                            <span class="eyebrow eyebrow--gold">{p.tag.clone()}</span>
                            <span class="tag">{format!("{} kotak", p.kotak)}</span>
                        </div>
                        <h3>{p.name.clone()}</h3>
                        <p class="lx-pcard__desc">{p.desc.clone()}</p>
                        <p class="lx-pcard__price"><b>{rupiah(p.price)}</b>" / set"</p>
                        <p class="sh-card__deposit"><Icon name="savings" />{format!("Deposit {} (kembali)", rupiah(p.deposit))}</p>
                        <ul class="lx-feat-list">
                            {p.features.iter().map(|f| {
                                let (icon, text) = icon_line(f, "check_circle");
                                view! { <li><span class="ms" aria-hidden="true">{icon.to_string()}</span>{text.to_string()}</li> }
                            }).collect_view()}
                        </ul>
                        <a class=if p.popular { "btn btn--primary btn--block" } else { "btn btn--soft btn--block" }
                            href=format!("/seserahan?paket={}#kalkulator", p.slug)>
                            "Pilih Paket Ini"<Icon name="arrow_forward" />
                        </a>
                    </article>
                }).collect_view()}
            </div>
        </section>

        <section class="lx-band">
            <div class="lx-band__in">
                <div>
                    <p class="eyebrow lx-cta__eyebrow">"Cara Sewa"</p>
                    <h2>"Empat Langkah, Hantaran Siap Dibawa"</h2>
                    <p>{format!("Masa sewa {}. Booking paling lambat {}.", info.lama_sewa, info.min_booking)}</p>
                </div>
                <div class="lx-band__grid">
                    {LANGKAH.iter().map(|p| view! {
                        <div class="lx-band__item">
                            <span class="ms" aria-hidden="true">{p.icon}</span>
                            <b>{p.title}</b>
                            <small>{p.text}</small>
                        </div>
                    }).collect_view()}
                </div>
            </div>
        </section>

        <Kalkulator k=k.clone() />

        {(!k.seserahan_galeri.is_empty()).then(|| view! {
            <section class="lx-sec" id="galeri">
                <SecHead eyebrow="Galeri Hantaran" title="Seserahan yang Pernah Kami Hias"
                    lead="Contoh tatanan dari pasangan sebelumnya — warna kain & bunga bisa disesuaikan tema acara Anda." />
                <div class="lx-mosaic">
                    {k.seserahan_galeri.clone().into_iter().enumerate().map(|(i, g)| {
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
        })}

        <section class="lx-sec lx-split" id="ketentuan">
            <div>
                <SecHead left=true eyebrow="Supaya Sama-sama Tenang" title="Ketentuan Sewa"
                    lead="Aturan singkat yang kami jelaskan juga saat wadah diserahkan." />
                <div class="lx-grid lx-grid--tight">
                    {info.syarat.iter().map(|l| {
                        let (icon, text) = icon_line(l, "check_circle");
                        view! {
                            <div class="lx-tile lx-tile--row">
                                <span class="ms" aria-hidden="true">{icon.to_string()}</span>
                                <span><small class="sh-rule">{text.to_string()}</small></span>
                            </div>
                        }
                    }).collect_view()}
                </div>
            </div>
            <div class="card lx-panel">
                <p class="eyebrow eyebrow--gold">"Ringkas"</p>
                <h3>"Yang Perlu Diingat"</h3>
                <div class="lx-tile lx-tile--row">
                    <span class="ms" aria-hidden="true">"schedule"</span>
                    <span><b>"Masa sewa"</b><small>{info.lama_sewa.clone()}</small></span>
                </div>
                <div class="lx-tile lx-tile--row">
                    <span class="ms" aria-hidden="true">"event_available"</span>
                    <span><b>"Batas booking"</b><small>{info.min_booking.clone()}</small></span>
                </div>
                <div class="lx-tile lx-tile--row">
                    <span class="ms" aria-hidden="true">"shopping_bag"</span>
                    <span><b>"Isi seserahan"</b><small>"Disiapkan keluarga. Pakai jasa hias bila ingin kami tata rapi di tiap kotak."</small></span>
                </div>
                <a class="btn btn--soft" href=wa_href("seserahan") target="_blank" rel="noopener"><Icon name="chat" />"Tanya Admin via WhatsApp"</a>
            </div>
        </section>

        {(!k.seserahan_testimoni.is_empty()).then(|| view! {
            <section class="lx-sec">
                <SecHead eyebrow="Kata Mereka" title="Hantaran yang Bikin Keluarga Bangga" />
                <Quotes items=k.seserahan_testimoni.clone() />
            </section>
        })}

        <section class="lx-cta">
            <div>
                <p class="eyebrow lx-cta__eyebrow">"Sekalian lengkapi hari bahagia"</p>
                <h2>"Seserahan, Dekorasi, Rias & Undangan — Satu Admin"</h2>
                <p>"Pesan beberapa layanan sekaligus agar jadwal antar H-1 lebih mudah dikoordinasikan, dan tanyakan harga bundling-nya."</p>
            </div>
            <a class="btn btn--gold btn--lg" href="#kalkulator"><Icon name="calculate" />"Hitung & Booking Seserahan"</a>
        </section>
    }
}

/// Kalkulator sewa + formulir booking. Angka hanya tampilan; server
/// menghitung ulang saat formulir dikirim ke WhatsApp.
#[component]
fn Kalkulator(k: Konten) -> impl IntoView {
    let default_paket = k.seserahan_paket.iter().find(|p| p.popular).or(k.seserahan_paket.first()).map(|p| p.slug.clone()).unwrap_or_default();
    let paket = paket_dari_query(k.seserahan_paket.iter().map(|p| p.slug.clone()).collect(), default_paket, "kalkulator");
    let opsi = RwSignal::new(Vec::<String>::new());
    let antar = RwSignal::new(k.seserahan_antar.first().map(|a| a.slug.clone()).unwrap_or_default());
    let input = Memo::new(move |_| SeserahanInput { paket: paket.get(), opsi: opsi.get(), antar: antar.get() });
    let kk = k.clone();
    let est = Memo::new(move |_| input.get().estimasi(&kk));
    let init = input.get_untracked();
    let kn = k.clone();
    let nama_paket = move || input.get().paket(&kn).map(|p| p.name.clone()).unwrap_or_default();

    view! {
        <section class="lx-sec" id="kalkulator">
            <SecHead eyebrow="Transparan Sejak Awal" title="Hitung Biaya Sewa Seserahan"
                lead="Pilih wadah, layanan tambahan, dan area antar — totalnya langsung terlihat, lalu kirim ke admin untuk cek tanggal." />
            <form class="lx-calc" action="/layanan/wa" method="get" target="_blank">
                <input type="hidden" name="layanan" value="seserahan" />
                <div class="card lx-calc__form">
                    <div class="lx-calc__title">
                        <h3>"Form Booking Cepat"</h3>
                        <span class="chip chip--xs"><Icon name="bolt" />"Hitung otomatis"</span>
                    </div>

                    <p class="field__label">"Pilih paket wadah"</p>
                    <div class="seg lx-seg sh-seg">
                        {k.seserahan_paket.clone().into_iter().map(|p| {
                            let (s1, s2, s3) = (p.slug.clone(), p.slug.clone(), p.slug.clone());
                            view! {
                                <label class="seg__opt">
                                    <input type="radio" name="paket" value=s1 checked=p.slug == init.paket
                                        prop:checked=move || paket.get() == s2 on:change=move |_| paket.set(s3.clone()) />
                                    <b>{p.name.clone()}</b>
                                    <small>{format!("{} kotak • {}", p.kotak, rupiah(p.price))}</small>
                                </label>
                            }
                        }).collect_view()}
                    </div>

                    <p class="field__label">"Layanan tambahan (opsional)"</p>
                    <div class="lx-grid lx-grid--2 lx-grid--tight">
                        {k.seserahan_opsi.clone().into_iter().map(|o| {
                            let slug = o.slug.clone();
                            let (price, per_kotak) = (o.price, o.per_kotak);
                            let harga = move || {
                                if per_kotak {
                                    format!("+{} / kotak = {}", rupiah(price), rupiah(SeserahanInput::harga_opsi(price, true, est.get().kotak)))
                                } else {
                                    format!("+{}", rupiah(price))
                                }
                            };
                            view! {
                                <label class="lx-check">
                                    <input type="checkbox" name=format!("opsi_{}", o.slug) value="1"
                                        on:change=move |e| {
                                            let on = event_target_checked(&e);
                                            opsi.update(|v| { v.retain(|x| *x != slug); if on { v.push(slug.clone()) } })
                                        } />
                                    <span>
                                        <b>{o.name.clone()}</b>
                                        {(!o.desc.is_empty()).then(|| view! { <small class="sh-opsi__desc">{o.desc.clone()}</small> })}
                                        <small class="sh-opsi__harga">{harga}</small>
                                    </span>
                                </label>
                            }
                        }).collect_view()}
                    </div>

                    <label class="field">
                        <span class="field__label">"Antar-jemput wadah"</span>
                        <select class="input" name="antar" on:change=move |e| antar.set(event_target_value(&e))>
                            {k.seserahan_antar.clone().into_iter().map(|a| {
                                let harga = if a.price > 0 { rupiah(a.price) } else { "gratis".to_string() };
                                view! { <option value=a.slug.clone()>{format!("{} ({harga})", a.name)}</option> }
                            }).collect_view()}
                        </select>
                    </label>

                    <div class="field-row">
                        <label class="field">
                            <span class="field__label">"Nama calon pengantin / keluarga"</span>
                            <input class="input" name="nama" required maxlength="80" placeholder="Contoh: Keluarga Bp. Hartono" autocomplete="name" />
                        </label>
                        <label class="field">
                            <span class="field__label">"Nomor WhatsApp aktif"</span>
                            <input class="input" name="wa" type="tel" required maxlength="20" placeholder="0812xxxxxxxx" inputmode="tel" autocomplete="tel" />
                        </label>
                    </div>
                    <label class="field">
                        <span class="field__label">"Tanggal acara (lamaran / akad)"</span>
                        <input class="input" name="tanggal" type="date" />
                    </label>
                    <label class="field">
                        <span class="field__label">"Catatan (warna tema, alamat antar, daftar isi…)"</span>
                        <textarea class="input" name="catatan" rows="3" maxlength="300"></textarea>
                    </label>
                </div>

                <aside class="lx-calc__side">
                    <div class="card lx-summary">
                        <p class="eyebrow">"Ringkasan biaya"</p>
                        <div class="lx-summary__row"><span>{move || format!("Sewa {}", nama_paket())}</span><b>{move || rupiah(est.get().sewa)}</b></div>
                        <div class="lx-summary__row"><span>"Layanan tambahan"</span><b>{move || rupiah(est.get().tambahan)}</b></div>
                        <div class="lx-summary__row"><span>"Antar-jemput"</span><b>{move || { let a = est.get().antar; if a > 0 { rupiah(a) } else { "Gratis".into() } }}</b></div>
                        <div class="lx-summary__row"><span>"Deposit (dikembalikan)"</span><span>{move || rupiah(est.get().deposit)}</span></div>
                        <div class="lx-summary__total">
                            <span><small>"Total biaya sewa"</small><b>{move || rupiah(est.get().biaya)}</b></span>
                        </div>
                        <p class="muted small sh-summary__bayar">
                            {move || { let e = est.get(); format!("Dibayar saat wadah diterima: {} (termasuk deposit {} yang kembali).", rupiah(e.bayar), rupiah(e.deposit)) }}
                        </p>
                        <button class="btn btn--primary btn--block btn--lg" type="submit"><Icon name="chat" />"Cek Tanggal & Booking via WhatsApp"</button>
                    </div>
                    <div class="lx-tile lx-tile--row lx-pack">
                        <Icon name="verified" />
                        <span>
                            <b>"Wadah steril & siap foto"</b>
                            <small>"Setiap wadah dicuci, dipoles anti-gores, dan dibungkus plastik pelindung sebelum diantar."</small>
                        </span>
                    </div>
                </aside>
            </form>
        </section>
    }
}
