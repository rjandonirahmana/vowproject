//! pages/cetak.rs — /cetak: layanan cetak undangan fisik + kalkulator biaya
//! cetak & ongkir (desain Stitch "Layanan Cetak Undangan Fisik").

use leptos::prelude::*;
use leptos_meta::{Meta, Title};

use crate::web::fmt::{ribuan, rupiah};
use crate::web::icons::Icon;
use crate::web::konten::Konten;
use crate::web::layanan::{berat, CetakInput, CETAK_QTY_MAX, CETAK_QTY_MIN};

use super::layanan::{paket_dari_query, wa_href, Quotes, SecHead, WithKonten};
use super::{SiteFooter, SiteHeader};

struct Poin {
    icon: &'static str,
    title: &'static str,
    text: &'static str,
}

const KEUNGGULAN: &[Poin] = &[
    Poin { icon: "brush", title: "Bebas custom", text: "Revisi layout & motif botanical" },
    Poin { icon: "loyalty", title: "Gratis OPP & label", text: "Plastik & stiker nama tamu" },
    Poin { icon: "redeem", title: "Gratis kartu souvenir", text: "Kartu ucapan terima kasih" },
    Poin { icon: "verified", title: "Garansi cetak ulang", text: "Jika ada cacat produksi fisik" },
];

const KERTAS: &[Poin] = &[
    Poin { icon: "auto_awesome", title: "Jasmine Glitter", text: "Kilau mutiara halus berkilau lembut di bawah cahaya." },
    Poin { icon: "texture", title: "Concorde Bertekstur", text: "Permukaan garis-garis lembut klasik khas kertas Eropa." },
    Poin { icon: "grid_view", title: "Linen Jepang Asli", text: "Serat rajut menyilang natural dengan ketahanan prima." },
    Poin { icon: "forest", title: "Kraft Paper Rustik", text: "Cokelat kayu natural yang hangat untuk tema intimate garden." },
];

const FINISHING: &[Poin] = &[
    Poin { icon: "flare", title: "Hot stamping foil emas & rose gold", text: "Lembaran foil logam dengan panas presisi tinggi agar tidak pudar." },
    Poin { icon: "layers", title: "Emboss & deboss 3D timbul", text: "Kedalaman sentuhan yang nyata saat diraba jemari." },
    Poin { icon: "mail", title: "Amplop die-cut presisi", text: "Lipatan geometri bersudut presisi dengan pisau pond khusus." },
    Poin { icon: "shield", title: "Segel lilin asli (real wax seal)", text: "Dicetak manual satu per satu dengan wax Eropa anti-retak." },
];

#[component]
pub fn CetakPage() -> impl IntoView {
    view! {
        <Title text=concat!("Cetak Undangan Fisik Eksklusif — ", crate::brand!()) />
        <Meta name="description" content="Cetak undangan pernikahan fisik: hardcover foil emas, akrilik, wax seal asli. Hitung estimasi harga & ongkir, kirim aman ke seluruh Indonesia." />
        <div class="site">
            <SiteHeader active="cetak" />
            <WithKonten view=cetak_body />
            <SiteFooter />
        </div>
    }
}

fn cetak_body(k: Konten) -> impl IntoView {
    let bonus_qty = k.umum.cetak_bonus_qty;
    view! {
            <section class="lx-hero">
                <div class="lx-hero__text">
                    <span class="chip chip--soft"><Icon name="local_florist" />"Handcrafted Botanical Stationery • Kirim Seluruh Indonesia"</span>
                    <h1>"Cetak Undangan Fisik "<em>"Eksklusif & Elegan"</em></h1>
                    <p class="lx-hero__lead">"Hadirkan keanggunan kertas bertekstur premium, foil emas timbul (emboss), dan wax seal asli. Pengiriman aman ke seluruh Nusantara via JNE, J&T Express, dan cargo terpercaya."</p>
                    <div class="lx-mini-feats">
                        {KEUNGGULAN.iter().map(|k| view! {
                            <div class="lx-mini-feat">
                                <span class="ms" aria-hidden="true">{k.icon}</span>
                                <b>{k.title}</b>
                                <small>{k.text}</small>
                            </div>
                        }).collect_view()}
                    </div>
                    <div class="lx-hero__btns">
                        <a class="btn btn--primary" href="#kalkulator"><Icon name="calculate" />"Hitung Estimasi Harga & Ongkir"</a>
                        <a class="btn btn--outline" href=wa_href("sampel") target="_blank" rel="noopener"><Icon name="mark_email_read" />"Minta Sampel Kit Fisik"</a>
                    </div>
                </div>
                <div class="lx-hero__media">
                    <img src=k.umum.cetak_hero.clone() alt="Undangan fisik hardcover sage dengan wax seal emas" fetchpriority="high" />
                    <span class="chip chip--gold lx-hero__chip"><Icon name="local_shipping" />"Garansi Anti-Penyok"</span>
                    <div class="lx-float">
                        <span class="lx-float__icon"><Icon name="star" /></span>
                        <span><b>"4,9 / 5,0"</b><small>"Dari 1.250+ pasangan bahagia se-Indonesia"</small></span>
                    </div>
                </div>
            </section>

            <section class="lx-promo">
                <div class="lx-promo__in">
                    <span class="lx-promo__icon"><Icon name="redeem" /></span>
                    <div class="lx-promo__text">
                        <p><span class="chip chip--gold chip--xs">"Spesial Musim Menikah"</span><span class="lx-promo__val">{k.umum.cetak_bonus_label.clone()}</span></p>
                        <h3>"Gratis Undangan Digital Website Botanical Heritage (1 Tahun)"</h3>
                        <p class="lx-promo__sub">{format!("Untuk pemesanan cetak fisik minimal {bonus_qty} pcs. RSVP WhatsApp, hitung mundur, petunjuk arah Maps, dan amplop digital langsung aktif.")}</p>
                    </div>
                    <a class="btn btn--gold" href="#kalkulator">"Klaim Promo Paket"<Icon name="arrow_forward" /></a>
                </div>
            </section>

            <section class="lx-sec" id="paket-cetak">
                <SecHead eyebrow="Pilihan Mahakarya Fisik" title="Katalog Paket Cetak Undangan"
                    lead="Setiap paket dikerjakan dengan presisi cetak offset Heidelberg dan finishing tangan pengrajin seni kertas kami." />
                <div class="lx-grid lx-grid--3">
                    {k.cetak_paket.clone().into_iter().map(|p| view! {
                        <article class="card lx-pcard" class:lx-pcard--pop=p.popular>
                            {p.popular.then(|| view! { <span class="chip chip--gold lx-pcard__flag">"Paling Favorit"</span> })}
                            <div class="lx-pcard__img"><img src=p.img.clone() alt=p.name.clone() loading="lazy" decoding="async" /></div>
                            <div class="lx-pcard__meta">
                                <span class="eyebrow eyebrow--gold">{p.tier.clone()}</span>
                                <span class="tag">{format!("Min {} pcs", p.min_qty)}</span>
                            </div>
                            <h3>{p.name.clone()}</h3>
                            <p class="lx-pcard__desc">{p.desc.clone()}</p>
                            <p class="lx-pcard__price"><b>{rupiah(p.price)}</b>" / pcs"</p>
                            <ul class="lx-feat-list">
                                {p.features.iter().map(|f| view! { <li><Icon name="check_circle" />{f.clone()}</li> }).collect_view()}
                            </ul>
                            <a class=if p.popular { "btn btn--primary btn--block" } else { "btn btn--soft btn--block" }
                                href=format!("/cetak?paket={}#kalkulator", p.slug)>
                                "Pilih Paket Ini"<Icon name="arrow_forward" />
                            </a>
                        </article>
                    }).collect_view()}
                </div>
            </section>

            <section class="lx-sec lx-split">
                <div>
                    <SecHead left=true eyebrow="Eksplorasi Material" title="Pilihan Kertas Premium Sensorial"
                        lead="Sentuhan pertama tamu bermula dari tekstur kertas. Kami memakai kertas ramah lingkungan bersertifikasi dengan karakter tekstur yang khas." />
                    <div class="lx-grid lx-grid--2">
                        {KERTAS.iter().map(|k| view! {
                            <div class="lx-tile">
                                <span class="ms" aria-hidden="true">{k.icon}</span>
                                <span><b>{k.title}</b><small>{k.text}</small></span>
                            </div>
                        }).collect_view()}
                    </div>
                </div>
                <div class="card lx-panel">
                    <p class="eyebrow eyebrow--gold">"Finishing Mewah Berkualitas"</p>
                    <h3>"Detail Mahakarya yang Memikat"</h3>
                    {FINISHING.iter().map(|k| view! {
                        <div class="lx-tile lx-tile--row">
                            <span class="ms" aria-hidden="true">{k.icon}</span>
                            <span><b>{k.title}</b><small>{k.text}</small></span>
                        </div>
                    }).collect_view()}
                </div>
            </section>

            <Kalkulator k=k.clone() />

            <section class="lx-sec">
                <div class="lx-head-row">
                    <SecHead left=true eyebrow="Kisah dari Pengantin Nusantara" title="Pengiriman Aman dari Sabang sampai Merauke" />
                    <span class="chip"><Icon name="verified" />"100% pesanan tiba utuh"</span>
                </div>
                <Quotes items=k.cetak_testimoni.clone() />
                <div class="lx-coverage">
                    <span><Icon name="public" />"Jangkauan pengiriman 514 kota & kabupaten di seluruh Indonesia"</span>
                    <span class="lx-coverage__ticks">
                        <span><Icon name="check" />"Resi otomatis"</span>
                        <span><Icon name="check" />"Asuransi pengiriman"</span>
                        <span><Icon name="check" />"Lacak 24 jam"</span>
                    </span>
                </div>
            </section>

            <section class="lx-cta">
                <div>
                    <p class="eyebrow lx-cta__eyebrow">"Ragu memilih tekstur kertas?"</p>
                    <h2>"Pesan \"Wedding Sample Kit\" Eksklusif"</h2>
                    <p>"Rasakan langsung sampel cetak fisik, foil emas timbul, 5 jenis kertas, dan segel wax seal asli di rumah Anda sebelum produksi massal."</p>
                </div>
                <a class="btn btn--gold btn--lg" href=wa_href("sampel") target="_blank" rel="noopener"><Icon name="shopping_bag" />"Kirim Sampel Kit ke Rumah"</a>
            </section>
    }
}

/// Kalkulator biaya cetak + ongkir. Angka di sini hanya estimasi tampilan;
/// server menghitung ulang dari query saat formulir dikirim ke WhatsApp.
#[component]
fn Kalkulator(k: Konten) -> impl IntoView {
    let default_paket = k.cetak_paket.iter().find(|p| p.popular).or(k.cetak_paket.first()).map(|p| p.slug.clone()).unwrap_or_default();
    let paket = paket_dari_query(k.cetak_paket.iter().map(|p| p.slug.clone()).collect(), default_paket, "kalkulator");
    let start_qty = k.umum.cetak_bonus_qty.clamp(CETAK_QTY_MIN, CETAK_QTY_MAX);
    let qty = RwSignal::new(start_qty);
    let finishing = RwSignal::new(Vec::<String>::new());
    let wilayah = RwSignal::new(k.cetak_wilayah.first().map(|w| w.slug.clone()).unwrap_or_default());
    let input = Memo::new(move |_| CetakInput { paket: paket.get(), qty: qty.get(), finishing: finishing.get(), wilayah: wilayah.get() });
    let kk = k.clone();
    let est = Memo::new(move |_| input.get().estimasi(&kk));
    let init = input.get_untracked();
    let kmin = k.clone();
    let min_notice = move || {
        let i = input.get();
        i.paket(&kmin).filter(|p| qty.get() < p.min_qty).map(|p| format!("{} minimal {} pcs — estimasi memakai {} pcs.", p.name, p.min_qty, p.min_qty))
    };

    view! {
        <section class="lx-sec" id="kalkulator">
            <SecHead eyebrow="Transparansi Harga & Logistik" title="Simulasi Biaya Cetak & Ongkos Kirim"
                lead="Hitung estimasi total investasi dan berat paket ke kota tujuan Anda dengan kurir terpercaya se-Indonesia." />
            <form class="lx-calc" action="/layanan/wa" method="get" target="_blank">
                <input type="hidden" name="layanan" value="cetak" />
                <div class="card lx-calc__form">
                    <div class="lx-calc__title">
                        <h3>"Form Perhitungan Cepat"</h3>
                        <span class="chip chip--xs"><Icon name="bolt" />"Hitung otomatis"</span>
                    </div>

                    <p class="field__label">"Pilih paket cetak"</p>
                    <div class="seg lx-seg">
                        {k.cetak_paket.clone().into_iter().map(|p| {
                            let (s1, s2, s3) = (p.slug.clone(), p.slug.clone(), p.slug.clone());
                            view! {
                                <label class="seg__opt">
                                    <input type="radio" name="paket" value=s1 checked=p.slug == init.paket
                                        prop:checked=move || paket.get() == s2 on:change=move |_| paket.set(s3.clone()) />
                                    <b>{if p.short.is_empty() { p.name.clone() } else { p.short.clone() }}</b>
                                    <small>{format!("{}/pcs", rupiah(p.price))}</small>
                                </label>
                            }
                        }).collect_view()}
                    </div>

                    <div class="lx-range">
                        <label class="field__label" for="qty">"Jumlah undangan (pcs)"</label>
                        <output class="lx-range__val" for="qty">{move || format!("{} pcs", ribuan(qty.get()))}</output>
                        <input id="qty" type="range" name="qty" min=CETAK_QTY_MIN.to_string() max=CETAK_QTY_MAX.to_string() step="50"
                            value=init.qty.to_string() prop:value=move || qty.get().to_string()
                            on:input=move |e| qty.set(event_target_value(&e).parse().unwrap_or(start_qty)) />
                        <div class="lx-range__scale"><span>{format!("Min. {CETAK_QTY_MIN}")}</span><span>"1.000"</span><span>{format!("{}+", ribuan(CETAK_QTY_MAX))}</span></div>
                    </div>
                    {move || min_notice().map(|m| view! { <p class="notice notice--info">{m}</p> })}

                    <p class="field__label">"Finishing tambahan (opsional)"</p>
                    <div class="lx-grid lx-grid--2 lx-grid--tight">
                        {k.cetak_finishing.clone().into_iter().map(|f| {
                            let slug = f.slug.clone();
                            view! {
                                <label class="lx-check">
                                    <input type="checkbox" name=format!("fin_{}", f.slug) value="1"
                                        on:change=move |e| {
                                            let on = event_target_checked(&e);
                                            finishing.update(|v| { v.retain(|x| *x != slug); if on { v.push(slug.clone()) } })
                                        } />
                                    <span><b>{f.name.clone()}</b><small>{format!("+{} / pcs", rupiah(f.price))}</small></span>
                                </label>
                            }
                        }).collect_view()}
                    </div>

                    <label class="field">
                        <span class="field__label">"Wilayah tujuan pengiriman"</span>
                        <select class="input" name="wilayah" on:change=move |e| wilayah.set(event_target_value(&e))>
                            {k.cetak_wilayah.clone().into_iter().map(|w| view! {
                                <option value=w.slug.clone()>{format!("{} (est. {}/kg – {})", w.name, rupiah(w.per_kg), w.kurir)}</option>
                            }).collect_view()}
                        </select>
                    </label>
                    <p class="lx-couriers">
                        <span class="muted small">"Ekspedisi:"</span>
                        {["J&T Express", "JNE (Reg/YES/Trucking)", "SiCepat", "Indah Cargo (>10 kg hemat)"].into_iter().map(|k| view! { <span class="tag">{k}</span> }).collect_view()}
                    </p>
                </div>

                <aside class="lx-calc__side">
                    <div class="card lx-summary">
                        <p class="eyebrow">"Ringkasan estimasi"</p>
                        <div class="lx-summary__row"><span>"Biaya cetak undangan"</span><b>{move || rupiah(est.get().cetak)}</b></div>
                        <div class="lx-summary__row"><span>"Estimasi berat paket"</span><span>{move || berat(est.get().gram)}</span></div>
                        <div class="lx-summary__row"><span>"Estimasi ongkos kirim"</span><b>{move || rupiah(est.get().ongkir)}</b></div>
                        <div class="lx-summary__total">
                            <span><small>"Total estimasi"</small><b>{move || rupiah(est.get().total)}</b></span>
                            {move || est.get().cargo.then(|| view! {
                                <span class="lx-summary__cargo"><small>"Hemat cargo"</small>"Tersedia opsi JTR"</span>
                            })}
                        </div>
                        {move || est.get().bonus.then(|| view! {
                            <p class="notice notice--ok lx-summary__bonus"><Icon name="verified" />"Selamat! Anda berhak atas bonus undangan digital 1 tahun."</p>
                        })}
                        <button class="btn btn--primary btn--block btn--lg" type="submit"><Icon name="chat" />"Konsultasi & Kunci Harga via WhatsApp"</button>
                    </div>
                    <div class="lx-tile lx-tile--row lx-pack">
                        <Icon name="inventory_2" />
                        <span>
                            <b>"Standar packaging anti-hujan & anti-benturan"</b>
                            <small>"Tiap 50 pcs dibungkus plastik seal kedap air, 4 lapis bubble wrap tebal, lalu boks karton double-wall bersegel stiker fragile."</small>
                        </span>
                    </div>
                </aside>
            </form>
        </section>
    }
}
