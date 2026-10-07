//! pages/kelola.rs — dashboard pengantin (/kelola/:slug) dan pemindai QR buku
//! tamu (/kelola/:slug/scan). Tautan khusus `?key=…` ditukar server menjadi
//! cookie HttpOnly lalu URL dibersihkan (server/owner.rs); `?key=` di URL
//! hanya tersisa bila router berpindah halaman di klien (mis. tautan demo).
//!
//! Akses = kunci rahasia di URL (dikirim sekali saat pesan). Undangan demo
//! memakai kunci "demo" dan bersifat read-only.

use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::{use_params_map, use_query_map};

use crate::web::api::{get_dashboard, owner_stories, AddGuest, CheckIn, DeleteGuest, DeleteStory, MarkSent};
use crate::web::components::{err_msg, wa_share_text, Monogram, FloatDeco, StoryPager};
use crate::web::fmt::{self, rupiah, rupiah_ringkas};
use crate::web::icons::Icon;
use crate::web::model::*;

use super::ErrorCard;

/// `window.location.origin` setelah hydrate; SSR memakai tautan relatif.
fn use_origin() -> RwSignal<String> {
    let origin = RwSignal::new(String::new());
    Effect::new(move |_| {
        #[cfg(target_arch = "wasm32")]
        if let Some(o) = web_sys::window().and_then(|w| w.location().origin().ok()) {
            origin.set(o);
        }
    });
    origin
}

#[component]
pub fn KelolaPage() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let slug = move || params.read().get("slug").unwrap_or_default();
    let key = move || query.read().get("key").unwrap_or_default();
    let baru = move || query.read().get("baru").is_some();
    // Hasil kirim bukti transfer (POST /kelola/{slug}/bukti → ?ok= / ?galat=).
    let notice = move || {
        let q = query.read();
        q.get("galat").map(|m| (false, m)).or_else(|| q.get("ok").map(|m| (true, m)))
    };
    let tema = move || query.read().get("tema");

    let add = ServerAction::<AddGuest>::new();
    let del = ServerAction::<DeleteGuest>::new();
    let dash = Resource::new(
        move || (slug(), key(), tema(), add.version().get(), del.version().get()),
        |(s, k, t, _, _)| get_dashboard(s, k, t),
    );

    view! {
        <Title text=concat!("Kelola Undangan — ", crate::brand!()) />
        <Suspense fallback=|| view! { <div class="inv-loading"><div class="spinner"></div></div> }>
            {move || dash.get().map(|r| match r {
                Ok(d) => Either::Left(view! { <Dashboard d=d baru=baru() notice=notice() add=add del=del /> }),
                Err(e) => Either::Right(view! { <ErrorCard msg=err_msg(&e) /> }),
            })}
        </Suspense>
    }
}

#[component]
fn Dashboard(d: Dashboard, baru: bool, notice: Option<(bool, String)>, add: ServerAction<AddGuest>, del: ServerAction<DeleteGuest>) -> impl IntoView {
    let inv = d.inv.clone();
    let s = d.stats.clone();
    // Kunci terverifikasi dari server (cookie) — untuk aksi & tautan yang disalin.
    let key = d.manage_key.clone();
    let origin = use_origin();
    let search = RwSignal::new(String::new());
    let filter = RwSignal::new("semua");
    let show_add = RwSignal::new(false);
    let mark = ServerAction::<MarkSent>::new();

    let days_left = {
        let target = inv.countdown_target_ms();
        let now = now_ms();
        ((target - now) as f64 / 86_400_000.0).ceil().max(0.0) as i64
    };
    let rsvp_pct = if s.total_guests > 0 { (s.rsvp_count * 100 / s.total_guests).min(100) } else { 0 };
    let hadir_pct = if s.rsvp_count > 0 { s.hadir_count * 100 / s.rsvp_count } else { 0 };
    let venue = inv.events.last().map(|e| e.venue.clone()).unwrap_or_default();
    // Belum dibayar: "Lihat Undangan" = pratinjau pemilik (tamu masih terkunci).
    // Demo: tema yang sedang dilihat ikut dibawa ke undangan, pemindai, dst.
    let tema_qs = if inv.is_demo { format!("tema={}", fmt::url_encode(&inv.theme)) } else { String::new() };
    let with_tema = |base: String| if tema_qs.is_empty() { base } else { format!("{base}{}{tema_qs}", if base.contains('?') { "&" } else { "?" }) };
    // Pratinjau pemilik, pemindai & CSV cukup URL bersih: cookie kunci ikut.
    let base_link = with_tema(format!("/u/{}", inv.slug));
    // Tautan khusus (dengan kunci) HANYA untuk disalin/disimpan pemilik.
    let kelola_link = with_tema(format!("/kelola/{}?key={}", inv.slug, key));
    let scan_link = with_tema(format!("/kelola/{}/scan", inv.slug));
    let csv_link = format!("/kelola/{}/tamu.csv", inv.slug);

    let guests = d.guests.clone();
    let counts = (
        guests.len(),
        guests.iter().filter(|g| matches!(g.category.as_str(), "vip" | "keluarga")).count(),
        guests.iter().filter(|g| g.rsvp == "hadir").count(),
        guests.iter().filter(|g| g.rsvp.is_empty()).count(),
    );
    let visible = move || {
        let q = search.get().to_lowercase();
        let f = filter.get();
        guests
            .iter()
            .filter(|g| {
                (q.is_empty()
                    || g.name.to_lowercase().contains(&q)
                    || g.table_no.to_lowercase().contains(&q)
                    || g.code.to_lowercase().contains(&q)
                    || category_label(&g.category).to_lowercase().contains(&q))
                    && match f {
                        "vip" => matches!(g.category.as_str(), "vip" | "keluarga"),
                        "hadir" => g.rsvp == "hadir",
                        "menunggu" => g.rsvp.is_empty(),
                        _ => true,
                    }
            })
            .cloned()
            .collect::<Vec<_>>()
    };

    let inv_for_blast = inv.clone();
    let blast = move || {
        let link = format!("{}{}", origin.get(), format!("/u/{}", inv_for_blast.slug));
        format!("https://wa.me/?text={}", fmt::url_encode(&wa_share_text(&inv_for_blast, "Bapak/Ibu/Saudara/i", &link)))
    };

    let (list_inv, list_key) = (inv.clone(), key.clone());
    let (add_slug, add_key) = (inv.slug.clone(), key.clone());
    let kl_copy = kelola_link.clone();
    view! {
        // Dashboard memakai tema undangan: warna & huruf (th-…), ilustrasi latar
        // + ornamen (inv__glow), hiasan melayang, dan hiasan di atas kartu utama.
        <div class=format!("inv inv--wide th-{}", inv.theme)>
            <div class="inv__glow" aria-hidden="true"></div>
            <FloatDeco kind=d.float_deco.clone() />
            <header class="inv-top">
                <a class="inv-top__brand" href=base_link.clone()>
                    <Monogram initials=inv.initials() class="monogram--xs" />
                    <span><b>{inv.couple()}</b><small>"Kelola"</small></span>
                </a>
                <div class="inv-top__actions">
                    // URL tak lagi memuat kunci → tautan khusus bisa disalin dari sini
                    // (untuk membuka Kelola di HP/laptop lain).
                    {(!inv.is_demo).then(|| view! {
                        // HP: ikon saja (teks ≥720px) — kalau tidak, menimpa nama pasangan di header.
                        <button type="button" class="btn btn--outline btn--sm" title="Salin tautan rahasia untuk membuka Kelola di perangkat lain"
                            aria-label="Salin Tautan Kelola"
                            data-copy=move || format!("{}{}", origin.get(), kl_copy.clone()) data-copied="Tautan kelola tersalin — simpan baik-baik">
                            <Icon name="key" /><span class="hide-sm">"Salin Tautan Kelola"</span>
                        </button>
                    })}
                    <a class="btn btn--soft btn--sm" href=base_link.clone() target="_blank"><Icon name="visibility" />"Lihat Undangan"</a>
                </div>
            </header>
            <main class="inv__main">
                {baru.then(|| {
                    let pay_msg = format!(
                        concat!("Halo admin ", crate::brand!(), ", saya sudah memesan undangan /u/{} ({}) total {} via {}. Mohon info pembayaran."),
                        inv.slug, d.package_name, rupiah(d.total_price), d.payment_method
                    );
                    let wa = if d.admin_wa.is_empty() { String::new() }
                        else { format!("https://wa.me/{}?text={}", d.admin_wa, fmt::url_encode(&pay_msg)) };
                    let kl = kelola_link.clone();
                    view! {
                        <section class="card banner-new">
                            <h2><Icon name="celebration" />"Undangan berhasil dibuat!"</h2>

                            <p>"Simpan tautan halaman kelola ini baik-baik — tautan inilah kunci untuk mengatur tamu & melihat RSVP."</p>
                            <div class="copyline">
                                <code>{move || format!("{}{}", origin.get(), kl.clone())}</code>
                                <button type="button" class="btn btn--outline btn--sm"
                                    data-copy=move || format!("{}{}", origin.get(), kelola_link.clone()) data-copied="Tautan kelola tersalin">
                                    <Icon name="content_copy" />"Salin"
                                </button>
                            </div>
                            <p class="totals__grand"><span>"Total Pembayaran"</span><b>{rupiah(d.total_price)}</b></p>
                            <p class="muted small">"Status: menunggu pembayaran. Undangan TERKUNCI untuk tamu sampai admin mengkonfirmasi pembayaran — Anda tetap bisa melihat pratinjaunya."</p>
                            <a class="btn btn--primary" href="#bayar"><Icon name="account_balance_wallet" />"Lanjut ke Pembayaran"</a>
                            {(!wa.is_empty()).then(|| view! {
                                <a class="btn btn--soft btn--sm" href=wa target="_blank" rel="noopener"><Icon name="chat" />"Tanya Admin via WhatsApp"</a>
                            })}
                        </section>
                    }
                })}
                {inv.is_demo.then(|| view! {
                    <p class="notice notice--info">"Dashboard demo (read-only) — tambah/hapus tamu dinonaktifkan."</p>
                })}
                {d.minutes_left.map(|m| {
                    let preview = base_link.clone();
                    view! {
                        <section class="card lock-banner">
                            <span class="lock-banner__icon"><Icon name="lock" /></span>
                            <div>
                                <b>"Undangan masih terkunci — menunggu konfirmasi pembayaran"</b>
                                <p class="small">
                                    "Tamu yang membuka tautan hanya melihat \"Undangan belum aktif\". "
                                    <b>{format!("Bila bukti transfer belum dikirim dalam {}", durasi(m))}</b>
                                    ", undangan beserta semua foto & lagu yang diunggah DIHAPUS otomatis. Setelah bukti dikirim, undangan aman menunggu verifikasi admin."
                                </p>
                                <a class="btn btn--soft btn--sm" href=preview target="_blank"><Icon name="visibility" />"Lihat Pratinjau (khusus Anda)"</a>
                            </div>
                        </section>
                    }
                })}

                {inv.is_locked().then(|| view! {
                    <PaymentCard slug=inv.slug.clone() key=key.clone() total=d.total_price pay=d.payment.clone()
                        proof=d.payment_proof.clone() notice=notice.clone() admin_wa=d.admin_wa.clone() />
                })}

                <section class="card card--soft dash-head">
                    <div>
                        <p class="eyebrow eyebrow--gold eyebrow--dot">"Panel Pengantin & Host"</p>
                        <h1>"Manajemen Tamu & Acara"</h1>
                        <p class="muted">{format!("{} • {}", inv.date_label(), venue)}</p>
                    </div>
                    <div class="hday"><b>{format!("H-{days_left}")}</b><small>"Menuju Hari H"</small></div>
                    <div class="progress">
                        <span><Icon name="check_circle" />"Target Konfirmasi RSVP"</span>
                        <b>{format!("{rsvp_pct}% Tercapai")}</b>
                        <i style=format!("--p:{rsvp_pct}%")></i>
                    </div>
                </section>

                <div class="stats">
                    <div class="stat card">
                        <p class="stat__label">"Total Undangan"<Icon name="groups" /></p>
                        <p class="stat__num">{s.total_guests}<small>" Tamu"</small></p>
                        <p class="stat__foot"><Icon name="send" />{format!("{} terkirim", s.sent)}</p>
                    </div>
                    <div class="stat stat--dark">
                        <p class="stat__label">"Pasti Hadir"<Icon name="how_to_reg" /></p>
                        <p class="stat__num">{s.hadir_pax}<small>" Orang"</small></p>
                        <p class="stat__foot"><Icon name="trending_up" />{format!("{hadir_pct}% respons positif")}</p>
                    </div>
                    <div class="stat card">
                        <p class="stat__label">"Berhalangan"<Icon name="event_busy" /></p>
                        <p class="stat__num">{s.tidak_count}<small>" Orang"</small></p>
                        <p class="stat__foot"><Icon name="mail" />{format!("{} masih ragu", s.ragu_count)}</p>
                    </div>
                    <div class="stat stat--gold">
                        <p class="stat__label">"Tanda Kasih Masuk"<Icon name="account_balance_wallet" /></p>
                        <p class="stat__num">{rupiah_ringkas(s.gift_total)}</p>
                        <p class="stat__foot"><Icon name="verified" />{format!("{} titipan", s.gift_count)}</p>
                    </div>
                </div>

                <div class="dash-actions">
                    <button type="button" class="btn btn--gold" on:click=move |_| show_add.update(|v| *v = !*v)>
                        <Icon name="person_add" />"Tambah Tamu"
                    </button>
                    <a class="btn btn--primary" href=blast target="_blank" rel="noopener"><Icon name="share" />"Share WA Blast"</a>
                    <a class="btn btn--soft btn--wide" href=csv_link><Icon name="download" />"Export Data RSVP (CSV / Excel)"</a>
                </div>

                <Show when=move || show_add.get()>
                    <AddGuestForm slug=add_slug.clone() key=add_key.clone() add=add />
                </Show>

                <a class="scanner" href=scan_link>
                    <span class="scanner__icon"><Icon name="qr_code_scanner" /></span>
                    <span><b>"Scanner Buku Tamu"</b><small>"Check-in cepat di meja resepsionis penerima tamu"</small></span>
                    <span class="btn btn--light btn--sm"><Icon name="photo_camera" />"Buka"</span>
                </a>

                <section class="guests">
                    <div class="guests__head">
                        <h2>"Daftar Undangan"<span class="count">{counts.0}</span></h2>
                        <span class="muted small">{format!("{} sudah check-in", s.checked_in)}</span>
                    </div>
                    <label class="searchbar searchbar--sm">
                        <Icon name="search" />
                        <input placeholder="Cari nama tamu / meja / grup…" on:input=move |e| search.set(event_target_value(&e)) />
                    </label>
                    <div class="chips-row chips-row--scroll">
                        {[("semua", "Semua", counts.0), ("vip", "VIP", counts.1), ("hadir", "Hadir", counts.2), ("menunggu", "Menunggu", counts.3)]
                            .into_iter().map(|(k, label, n)| view! {
                                <button type="button" class="chip-link" class:is-on=move || filter.get() == k on:click=move |_| filter.set(k)>
                                    {format!("{label} ({n})")}
                                </button>
                            }).collect_view()}
                    </div>
                    {
                        let (inv, key) = (list_inv, list_key);
                        move || {
                            let list = visible();
                            if list.is_empty() {
                                return Either::Left(view! { <p class="muted center">"Belum ada tamu. Tekan “Tambah Tamu” untuk mulai."</p> });
                            }
                            Either::Right(list.into_iter().map(|g| view! {
                                <GuestCard g=g inv=inv.clone() key=key.clone() origin=origin mark=mark del=del />
                            }).collect_view())
                        }
                    }
                </section>

                <StoryModeration slug=inv.slug.clone() key=key.clone() demo=inv.is_demo />

                <section class="card activity">
                    <div class="guests__head">
                        <h2><Icon name="notifications_active" />"Aktivitas Real-Time"</h2>
                        <span class="muted small">"Live Feed"</span>
                    </div>
                    {if d.activity.is_empty() {
                        Either::Left(view! { <p class="muted">"Belum ada aktivitas."</p> })
                    } else {
                        Either::Right(d.activity.into_iter().map(|a| view! {
                            <div class="activity__item"><i class="dot"></i><div><p>{a.text}</p><small>{a.ago}</small></div></div>
                        }).collect_view())
                    }}
                </section>
            </main>
        </div>
    }
}

/// Langkah pembayaran (satu-satunya cara: transfer ke rekening Konten
/// `pembayaran`) + unggah bukti → POST /kelola/{slug}/bukti (form biasa,
/// jalan tanpa WASM) → server menyimpan gambar & mengirim WA ke admin.
#[component]
fn PaymentCard(
    slug: String,
    key: String,
    total: i64,
    pay: crate::web::konten::Pembayaran,
    proof: Option<(String, String)>,
    notice: Option<(bool, String)>,
    admin_wa: String,
) -> impl IntoView {
    let nomor_salin: String = pay.nomor.chars().filter(|c| c.is_ascii_digit()).collect();
    let sudah = proof.is_some();
    let tanya = (!admin_wa.is_empty()).then(|| {
        let msg = format!("Halo admin, saya ada kendala pembayaran undangan /u/{slug}.");
        format!("https://wa.me/{admin_wa}?text={}", fmt::url_encode(&msg))
    });
    view! {
        <section class="card pay-card" id="bayar">
            <h2><Icon name="account_balance_wallet" />"Pembayaran"</h2>
            {notice.map(|(ok, m)| view! { <p class=if ok { "notice notice--ok" } else { "notice notice--err" }>{m}</p> })}
            <p class="totals__grand"><span>"Total yang harus ditransfer"</span><b>{rupiah(total)}</b></p>
            <div class="pay-card__acc">
                <small>{format!("Transfer ke {}", pay.metode)}</small>
                <b class="pay-card__no">{pay.nomor.clone()}</b>
                <span>{format!("a/n {}", pay.atas_nama)}</span>
                <button type="button" class="btn btn--outline btn--sm" data-copy=nomor_salin data-copied="Nomor tujuan tersalin">
                    <Icon name="content_copy" />"Salin Nomor"
                </button>
            </div>
            <ol class="pay-card__steps">
                <li>{format!("Transfer {} ke {} di atas (a/n {}).", rupiah(total), pay.metode, pay.atas_nama)}</li>
                <li>"Ambil tangkapan layar bukti transfer yang berhasil."</li>
                <li>"Unggah gambarnya di bawah — admin menerima notifikasi WhatsApp & mengaktifkan undangan Anda."</li>
            </ol>
            {(!pay.catatan.is_empty()).then(|| view! { <p class="muted small">{pay.catatan.clone()}</p> })}
            {proof.map(|(url, ago)| view! {
                <div class="pay-card__done">
                    <Icon name="hourglass_top" />
                    <span><b>"Bukti terkirim"</b>{format!(" {ago} — menunggu verifikasi admin. Undangan tidak akan dihapus otomatis.")}</span>
                    {(!url.is_empty()).then(|| { let href = url.clone(); view! {
                        <a href=href target="_blank" rel="noopener"><img src=url alt="Bukti transfer yang dikirim" loading="lazy" /></a>
                    } })}
                </div>
            })}
            <form class="pay-card__form" method="post" action=format!("/kelola/{slug}/bukti") enctype="multipart/form-data">
                <input type="hidden" name="key" value=key />
                <label class="pay-card__file">
                    <Icon name="photo_camera" />
                    <span>"Pilih gambar bukti transfer (JPG/PNG, maks 5 MB)"</span>
                    <input type="file" name="bukti" accept="image/jpeg,image/png,image/webp" required />
                </label>
                <button class="btn btn--primary" type="submit">
                    <Icon name="send" />{if sudah { "Kirim Ulang Bukti" } else { "Kirim Bukti Transfer" }}
                </button>
            </form>
            {tanya.map(|href| view! {
                <a class="btn btn--soft btn--sm" href=href target="_blank" rel="noopener"><Icon name="chat" />"Ada kendala? Chat admin"</a>
            })}
        </section>
    }
}

#[component]
fn AddGuestForm(slug: String, key: String, add: ServerAction<AddGuest>) -> impl IntoView {
    view! {
        <ActionForm action=add attr:class="card add-guest">
            <h3>"Tambah Tamu Undangan"</h3>
            <input type="hidden" name="slug" value=slug />
            <input type="hidden" name="key" value=key />
            <div class="field-row">
                <input class="input" name="name" required maxlength="80" placeholder="Nama tamu (mis. Dimas & Partner)" />
                <input class="input" name="phone" inputmode="tel" placeholder="No. WhatsApp" />
            </div>
            <div class="field-row field-row--3">
                <select class="input" name="category">
                    {CATEGORIES.iter().map(|(k, l)| view! { <option value=*k>{*l}</option> }).collect_view()}
                </select>
                <input class="input" name="session" placeholder="Sesi (mis. Resepsi 11.30)" />
                <input class="input" name="table_no" placeholder="Meja" />
            </div>
            <div class="field-row">
                <input class="input" name="pax" type="number" min="1" max="10" value="2" />
                <button class="btn btn--primary" type="submit" disabled=move || add.pending().get()><Icon name="add" />"Simpan Tamu"</button>
            </div>
            {move || add.value().get().map(|r| match r {
                Ok(m) => view! { <p class="notice notice--ok">{m}</p> }.into_any(),
                Err(e) => view! { <p class="notice notice--err">{err_msg(&e)}</p> }.into_any(),
            })}
        </ActionForm>
    }
}

#[component]
fn GuestCard(
    g: GuestRow,
    inv: Invitation,
    key: String,
    origin: RwSignal<String>,
    mark: ServerAction<MarkSent>,
    del: ServerAction<DeleteGuest>,
) -> impl IntoView {
    let path = format!("/u/{}?g={}", inv.slug, g.code);
    let vip = matches!(g.category.as_str(), "vip" | "keluarga");
    let meta = [g.session.clone(), if g.table_no.is_empty() { String::new() } else { format!("Meja {}", g.table_no) }]
        .into_iter()
        .filter(|x| !x.is_empty())
        .collect::<Vec<_>>()
        .join(" • ");
    let status_text = if g.rsvp == "hadir" { format!("Hadir ({} Pax)", g.rsvp_pax) } else { rsvp_label(&g.rsvp).to_string() };
    let wa = {
        let (path, name, phone, inv) = (path.clone(), g.name.clone(), g.phone.clone(), inv.clone());
        move || {
            let text = wa_share_text(&inv, &name, &format!("{}{}", origin.get(), path));
            format!("https://wa.me/{}?text={}", phone, fmt::url_encode(&text))
        }
    };
    let (slug, code) = (inv.slug.clone(), g.code.clone());
    let (slug2, code2, key2) = (slug.clone(), code.clone(), key.clone());
    let foot = if g.checked_in {
        view! { <span class="gc__foot-text ok"><Icon name="how_to_reg" />"Sudah check-in"</span> }.into_any()
    } else if g.gift_amount > 0 {
        view! { <span class="gc__foot-text gold"><Icon name="redeem" />{format!("Amplop Digital: {}", rupiah(g.gift_amount))}</span> }.into_any()
    } else if !g.sent_ago.is_empty() {
        view! { <span class="gc__foot-text">{format!("Terkirim via WA ({})", g.sent_ago)}</span> }.into_any()
    } else {
        view! { <span class="gc__foot-text"><Icon name="qr_code_2" />{format!("QR Check-in: {}", g.code)}</span> }.into_any()
    };
    view! {
        <article class="gc card">
            <div class="gc__top">
                <span class=format!("avatar avatar--{}", if vip { 1 } else { 0 })>{avatar_initials(&g.name)}</span>
                <div class="gc__who">
                    <b>{g.name.clone()}</b>
                    <small>{meta}</small>
                </div>
                <div class="gc__chips">
                    <span class=if vip { "chip chip--gold chip--xs" } else { "chip chip--xs" }>{category_label(&g.category)}</span>
                    <span class=format!("status status--{}", if g.rsvp.is_empty() { "none" } else { g.rsvp.as_str() })>{status_text}</span>
                </div>
            </div>
            <div class="gc__foot">
                {foot}
                <div class="gc__actions">
                    <a class="btn btn--primary btn--xs" href=wa target="_blank" rel="noopener"
                        on:click=move |_| { mark.dispatch(MarkSent { slug: slug.clone(), key: key.clone(), code: code.clone() }); }>
                        <Icon name="send" />{if g.sent_ago.is_empty() { "Kirim WA" } else { "Ingatkan WA" }}
                    </a>
                    <button type="button" class="icon-btn icon-btn--sm" title="Salin tautan pribadi"
                        data-copy=move || format!("{}{}", origin.get(), path.clone()) data-copied="Tautan tamu tersalin">
                        <Icon name="content_copy" />
                    </button>
                    <button type="button" class="icon-btn icon-btn--sm" title="Hapus tamu"
                        on:click=move |_| {
                            if confirm("Hapus tamu ini dari daftar undangan?") {
                                del.dispatch(DeleteGuest { slug: slug2.clone(), key: key2.clone(), code: code2.clone() });
                            }
                        }>
                        <Icon name="delete" />
                    </button>
                </div>
            </div>
        </article>
    }
}

fn confirm(_msg: &str) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        return web_sys::window().and_then(|w| w.confirm_with_message(_msg).ok()).unwrap_or(false);
    }
    #[allow(unreachable_code)]
    false
}

fn now_ms() -> i64 {
    #[cfg(target_arch = "wasm32")]
    {
        return js_now();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
    }
}

#[cfg(target_arch = "wasm32")]
fn js_now() -> i64 {
    #[wasm_bindgen::prelude::wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = Date, js_name = now)]
        fn date_now() -> f64;
    }
    date_now() as i64
}

// ── Pemindai QR ────────────────────────────────────────────────────────────

#[component]
pub fn ScanPage() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let slug = params.read_untracked().get("slug").unwrap_or_default();
    let key = query.read_untracked().get("key").unwrap_or_default();
    let action = ServerAction::<CheckIn>::new();
    let tema = query.read_untracked().get("tema");
    let back = match &tema {
        Some(t) => format!("/kelola/{slug}?tema={}", fmt::url_encode(t)),
        None => format!("/kelola/{slug}"),
    };
    // Tema undangan untuk tampilan pemindai (demo: tema yang sedang dilihat).
    let dash = Resource::new({ let (s, k, t) = (slug.clone(), key.clone(), tema.clone()); move || (s.clone(), k.clone(), t.clone()) }, |(s, k, t)| get_dashboard(s, k, t));
    let theme = move || {
        dash.get().and_then(|r| r.ok()).map(|d| d.inv.theme).unwrap_or_else(|| crate::web::skin::DEFAULT_THEME.to_string())
    };
    // Kunci terverifikasi (dari cookie) untuk check-in; cadangan: ?key= URL.
    let key = move || dash.get().and_then(|r| r.ok()).map(|d| d.manage_key).filter(|k| !k.is_empty()).unwrap_or_else(|| key.clone());
    view! {
        <Title text=concat!("Scanner Buku Tamu — ", crate::brand!()) />
        <Suspense fallback=|| view! { <div class="inv-loading"><div class="spinner"></div></div> }>
        {move || { let theme = theme(); let (slug, key, back) = (slug.clone(), key(), back.clone()); view! {
        <div class=format!("inv th-{theme}")>
            <div class="inv__glow" aria-hidden="true"></div>
            <header class="inv-top">
                <a class="inv-top__brand" href=back.clone()>
                    <span class="icon-btn"><Icon name="arrow_back" /></span>
                    <span><b>"Scanner Buku Tamu"</b><small>"Check-in resepsi"</small></span>
                </a>
            </header>
            <main class="inv__main">
                <section class="card scan">
                    <video data-scan-video="" playsinline="" muted="" hidden></video>
                    <button type="button" class="btn btn--primary btn--block btn--lg" data-scan-start="">
                        <Icon name="photo_camera" />"Nyalakan Kamera & Pindai QR"
                    </button>
                    <p class="muted small center">"Arahkan kamera ke QR di halaman “Doa & RSVP” milik tamu. Tanpa kamera? Ketik kodenya di bawah."</p>
                    <ActionForm action=action attr:class="scan__form">
                        <input type="hidden" name="slug" value=slug />
                        <input type="hidden" name="key" value=key />
                        <input class="input scan__input" name="code" data-scan-input="" required maxlength="200"
                            placeholder="KODE TAMU" autocomplete="off" autocapitalize="characters" />
                        <button class="btn btn--gold" type="submit" disabled=move || action.pending().get()>
                            <Icon name="how_to_reg" />"Check-in"
                        </button>
                    </ActionForm>
                    {move || action.value().get().map(|r| match r {
                        Ok(m) => view! { <p class="notice notice--ok scan__result">{m}</p> }.into_any(),
                        Err(e) => view! { <p class="notice notice--err scan__result">{err_msg(&e)}</p> }.into_any(),
                    })}
                </section>
                <a class="btn btn--soft btn--block" href=back>"Kembali ke Dashboard"</a>
            </main>
        </div>
        } }}
        </Suspense>
    }
}

/// Story tamu (tab Story undangan) — hanya pemegang kunci Kelola (pembeli
/// undangan). Berhalaman (STORY_PER_PAGE); hapus = permanen (baris + foto).
#[component]
fn StoryModeration(slug: String, key: String, demo: bool) -> impl IntoView {
    let del = ServerAction::<DeleteStory>::new();
    let page = RwSignal::new(1i64);
    let stories = Resource::new(
        {
            let (slug, key) = (slug.clone(), key.clone());
            move || (slug.clone(), key.clone(), page.get(), del.version().get())
        },
        |(slug, key, page, _)| owner_stories(slug, key, page),
    );
    // Hasil hapus dari penampil (POST /kelola/{slug}/story/hapus → ?story_ok=/story_galat=).
    let query = leptos_router::hooks::use_query_map();
    let story_notice = move || {
        let q = query.read();
        q.get("story_galat").map(|m| (false, m)).or_else(|| q.get("story_ok").map(|m| (true, m)))
    };
    let del_url = format!("/kelola/{slug}/story/hapus");
    view! {
        <section class="card story-mod" id="story">
            <div class="guests__head">
                <h2><Icon name="photo_camera" />"Story Tamu"</h2>
                <a class="muted small" href=format!("/u/{slug}/story") target="_blank" rel="noopener">"Lihat tab Story"</a>
            </div>
            <p class="muted small">"Ketuk story untuk melihatnya. Story yang kurang pas bisa Anda hapus permanen (foto ikut terhapus) — dari tombol 🗑 di kartu atau di dalam penampil. Pembuat story juga bisa menghapus story-nya sendiri."</p>
            {move || story_notice().map(|(ok, m)| view! { <p class=if ok { "notice notice--ok" } else { "notice notice--err" }>{m}</p> })}
            {move || del.value().get().and_then(|r| r.err()).map(|e| view! { <p class="notice notice--err">{err_msg(&e)}</p> })}
            <Transition fallback=|| view! { <p class="muted">"Memuat story…"</p> }>
                {move || stories.get().map(|r| match r {
                    Err(e) => Either::Left(view! { <p class="notice notice--err">{err_msg(&e)}</p> }),
                    Ok(pg) => Either::Right({
                        let (cur, pages, total) = (pg.page, pg.pages, pg.total);
                        // Data penampil story (global.js) untuk halaman ini: nomor pengirim
                        // tersamar ikut tampil di header penampil (hanya untuk pengelola).
                        let json = serde_json::to_string(&pg.items.iter().map(|s| serde_json::json!({
                            "id": s.id, "name": s.name, "photo": s.photo, "filter": s.filter,
                            "caption": s.caption, "ago": format!("{} • {}", s.phone, s.ago),
                        })).collect::<Vec<_>>()).unwrap_or_else(|_| "[]".into()).replace("</", "<\\/");
                        let (del_url, key_attr) = (del_url.clone(), key.clone());
                        view! {
                            <script type="application/json" id="story-data" inner_html=json
                                data-owner=(!demo).then_some("1") data-del-owner=del_url data-key=key_attr></script>
                            {(total == 0).then(|| view! { <p class="muted">"Belum ada story dari tamu."</p> })}
                            <div class="story-mod__grid">
                                {pg.items.into_iter().enumerate().map(|(i, s)| {
                                    let (slug, key) = (slug.clone(), key.clone());
                                    let id = s.id;
                                    let label = format!("Lihat story {}", s.name);
                                    view! {
                                        <figure class="story-mod__item" data-story-open=i.to_string() role="button" tabindex="0" aria-label=label>
                                            <img class=format!("sf-{}", s.filter) src=s.photo alt="" loading="lazy" decoding="async" />
                                            <figcaption>
                                                <b>{s.name}</b>
                                                <small class="muted">{s.phone}</small>
                                                <small>{s.ago}</small>
                                            </figcaption>
                                            // Tombol hapus jelas di tiap kartu (selalu tampil; demo = nonaktif).
                                            <button type="button" class="btn btn--sm story-mod__hapus"
                                                title=if demo { "Dashboard demo — story contoh tidak bisa dihapus" } else { "Hapus story ini permanen" }
                                                disabled=move || demo || del.pending().get()
                                                on:click=move |_| {
                                                    if confirm("Hapus story tamu ini secara permanen? Foto ikut terhapus.") { del.dispatch(DeleteStory { slug: slug.clone(), key: key.clone(), id }); }
                                                }>
                                                <Icon name="delete" />"Hapus"
                                            </button>
                                        </figure>
                                    }
                                }).collect_view()}
                            </div>
                            {(total > 0).then(|| view! { <StoryPager page=page cur=cur pages=pages total=total per=STORY_PER_PAGE anchor="story" /> })}
                        }
                    }),
                })}
            </Transition>
        </section>
    }
}
