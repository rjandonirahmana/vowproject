//! pages/buat.rs — /buat: formulir pemesanan paket + input data undangan.
//!
//! Form HTML biasa (multipart) → POST /buat/kirim (server/handlers.rs), jadi
//! tetap jalan tanpa WASM. Setelah hydrate, ringkasan harga, pratinjau mini,
//! dan cek ketersediaan tautan ikut hidup.

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;

use std::ops::Not;

use crate::web::api::{get_konten, get_theme, list_themes};
use crate::web::components::monogram_svg;
use crate::web::fmt::{self, rupiah};
use crate::web::icons::Icon;
use crate::web::model::initial;
use crate::web::skin::{ThemeInfo, DEFAULT_THEME};
use crate::web::konten::Konten;
use crate::web::themes::{PAYMENT_METHODS, QUOTES, SONGS};

use super::{SiteFooter, SiteHeader};

#[component]
pub fn BuatPage() -> impl IntoView {
    let q = use_query_map();
    let qget = move |k: &str| q.read_untracked().get(k).unwrap_or_default();

    let tema_q = qget("tema");
    let theme = RwSignal::new(if tema_q.is_empty() { DEFAULT_THEME.to_string() } else { tema_q.clone() });
    // Datang dari detail/kartu tema (?tema=slug): tema sudah dipilih → daftar
    // pilihan disembunyikan, cukup ringkasan + tombol "Ganti tema".
    let pick_open = RwSignal::new(tema_q.is_empty());
    // Katalog + tema privat dari ?tema= (pesanan custom yang dibuatkan admin).
    let theme_list = Resource::new(
        move || tema_q.clone(),
        |t| async move {
            let mut v: Vec<ThemeInfo> = list_themes().await.unwrap_or_default();
            if !t.is_empty() && !v.iter().any(|x| x.slug == t) {
                if let Ok(Some(p)) = get_theme(t).await {
                    v.insert(0, p);
                }
            }
            v
        },
    );
    let package = RwSignal::new(qget("paket"));
    // Harga, paket & add-on dari konten admin (server menghitung ulang saat kirim).
    let konten = Resource::new(|| (), |_| get_konten());
    let k = move || konten.get().and_then(|r| r.ok()).unwrap_or_default();
    let addons = RwSignal::new(Vec::<String>::new());
    let coupon = RwSignal::new(String::new());
    let bride = RwSignal::new(qget("wanita"));
    let groom = RwSignal::new(qget("pria"));
    let date = RwSignal::new(qget("tanggal"));
    let venue = RwSignal::new(String::new());
    let galat = qget("galat");

    // Tanda "sudah hydrate" untuk skrip global: draf isian (bila server menolak
    // kiriman sebelumnya) baru dipulihkan setelah sinyal halaman siap, supaya
    // nilainya tidak ditimpa `prop:value` saat hydrate.
    Effect::new(move |_| {
        if let Some(el) = crate::web::fmt::document().and_then(|d| d.document_element()) {
            let _ = el.set_attribute("data-buat-ready", "1");
        }
    });

    let totals = move || k().calc_total(&package.get(), &addons.get(), &coupon.get());
    let initials = move || format!("{}&{}", initial(&bride.get()), initial(&groom.get()));
    let toggle_addon = move |slug: String| {
        addons.update(|v| {
            if let Some(i) = v.iter().position(|x| *x == slug) { v.remove(i); } else { v.push(slug); }
        })
    };

    view! {
        <Title text=concat!("Buat Undangan — ", crate::brand!()) />
        <div class="site">
            <SiteHeader active="buat" />
            <ol class="steps">
                <li class="is-done"><span><Icon name="check" /></span><div><small>"Langkah 1"</small><b>"Paket & Add-on"</b></div></li>
                <li class="is-active"><span>"2"</span><div><small>"Sedang Aktif"</small><b>"Data Mempelai & Acara"</b></div></li>
                <li><span>"3"</span><div><small>"Langkah 3"</small><b>"Rekening Amplop"</b></div></li>
                <li><span>"4"</span><div><small>"Langkah 4"</small><b>"Konfirmasi & Bayar"</b></div></li>
            </ol>

            {(!galat.is_empty()).then(|| view! {
                <div class="notice notice--err wrap">
                    <p>{galat.clone()}</p>
                    <p class="small" data-draft-note="" hidden>"Isian Anda sudah dipulihkan — cukup perbaiki bagian di atas lalu kirim lagi. Foto & lagu yang tadi diunggah perlu dipilih ulang (browser tidak mengizinkan berkas diisi otomatis)."</p>
                </div>
            })}

            <form class="buat" method="post" action="/buat/kirim" enctype="multipart/form-data" data-keep="">
                <div class="buat__main">
                    // ── A. Tautan & tema ──
                    <section class="card fsec">
                        <FsecHead icon="link" title="Tema Undangan" sub="Pilih estetika desain utama undangan Anda" tag="Bagian A" />
                        <div class="notice notice--info buat__link-note">
                            <Icon name="lock" />
                            <span>
                                <b>"Tautan undangan dibuat otomatis & acak"</b>
                                " — nama kedua mempelai + kunci rahasia, contoh "<code>"/u/anindita-raditya-k7f3x9m2"</code>". Kunci acak membuat tautan tak bisa ditebak dan tak mungkin sama dengan pasangan lain walau namanya sama. Hanya orang yang Anda kirimi tautan yang bisa membukanya."
                            </span>
                        </div>
                        <Suspense fallback=|| view! { <p class="muted small">"Memuat tema…"</p> }>
                            {move || theme_list.get().and_then(|list| {
                                let cur = theme.get_untracked();
                                let t = list.into_iter().find(|t| t.slug == cur)?;
                                pick_open.get().not().then(|| view! {
                                    <div class="theme-chosen">
                                        <input type="hidden" name="theme" value=t.slug.clone() />
                                        <span class=format!("theme-pick__sw th-{}", t.slug)><i></i><i></i></span>
                                        <span class="theme-chosen__txt">
                                            <small>"Tema terpilih"</small>
                                            <b>{t.name.clone()}</b>
                                            <small>{if t.listed { t.region.clone() } else { "Tema custom milik Anda".to_string() }}</small>
                                        </span>
                                        <a class="btn btn--soft btn--sm" href=format!("/tema/{}", t.slug) target="_blank"><Icon name="visibility" />"Demo"</a>
                                        <button type="button" class="btn btn--outline btn--sm" on:click=move |_| pick_open.set(true)>
                                            <Icon name="swap_horiz" />"Ganti tema"
                                        </button>
                                    </div>
                                })
                            })}
                            <div class="theme-pick">
                                {move || pick_open.get().then(|| theme_list.get().map(|list| list.into_iter().map(|t| {
                                    let slug = t.slug.clone();
                                    let checked = theme.get_untracked() == t.slug;
                                    view! {
                                        <label class="theme-pick__opt">
                                            <input type="radio" name="theme" value=t.slug.clone() checked=checked
                                                on:change=move |_| theme.set(slug.clone()) />
                                            <span class=format!("theme-pick__sw th-{}", t.slug)><i></i><i></i></span>
                                            <span>
                                                <b>{t.name.clone()}</b>
                                                <small>{if t.listed { t.region.clone() } else { "Tema custom milik Anda".to_string() }}</small>
                                            </span>
                                        </label>
                                    }
                                }).collect_view()))}
                            </div>
                        </Suspense>
                    </section>

                    // ── B. Mempelai ──
                    <section class="card fsec">
                        <FsecHead icon="favorite" title="Profil Calon Mempelai" sub="Lengkapi data kedua mempelai dan silsilah keluarga terhormat" tag="Bagian B" />
                        <PersonFields prefix="bride" label="Mempelai Wanita (The Bride)" parent_label="Putri dari" name=bride />
                        <PersonFields prefix="groom" label="Mempelai Pria (The Groom)" parent_label="Putra dari" name=groom />
                        <div class="subcard">
                            <p class="eyebrow eyebrow--dot">"Foto Sampul Berdua"</p>
                            <div class="preview-row" data-preview-box="">
                                <label class="upload">
                                    <Icon name="photo_camera" />
                                    <span><b>"Foto prewedding untuk sampul"</b><small>"Tampil besar di halaman depan dalam bingkai lengkung — geser & zoom agar pas (maks 5 MB)."</small></span>
                                    <input type="file" name="cover_photo" accept="image/jpeg,image/png,image/webp" data-preview="image" data-max="5"
                                        data-pos-name="cover_photo_pos" />
                                </label>
                                <div class="preview-out preview-out--photo" data-preview-out=""></div>
                            </div>
                        </div>
                        <label class="field">
                            <span class="field__label">"Nama Keluarga Pengundang (penutup undangan)"</span>
                            <input class="input" name="family_name" maxlength="120" placeholder="Keluarga Besar Soedibyo & Pratama" />
                        </label>
                    </section>

                    // ── C. Acara ──
                    <section class="card fsec">
                        <FsecHead icon="calendar_month" title="Rincian Jadwal & Tempat Acara" sub="Atur jadwal sakral akad nikah dan jam sesi resepsi tamu" tag="Bagian C" />
                        <div class="subcard">
                            <div class="subcard__head"><span class="num">"1"</span><b>"Akad Nikah / Pemberkatan"</b><small class="gold">"Wajib"</small></div>
                            <div class="field-row">
                                <label class="field"><span class="field__label">"Hari & Tanggal"</span>
                                    <input class="input" type="date" name="akad_date" required prop:value=move || date.get()
                                        on:input=move |e| date.set(event_target_value(&e)) /></label>
                                <label class="field"><span class="field__label">"Judul Acara"</span>
                                    <select class="input" name="akad_title">
                                        <option>"Akad Nikah"</option><option>"Pemberkatan Nikah"</option><option>"Upacara Pernikahan"</option>
                                    </select></label>
                            </div>
                            <div class="field-row field-row--3">
                                <label class="field"><span class="field__label">"Mulai"</span><input class="input" type="time" name="akad_start" value="08:00" /></label>
                                <label class="field"><span class="field__label">"Selesai"</span><input class="input" type="time" name="akad_end" value="10:00" /></label>
                                <label class="field"><span class="field__label">"Zona waktu"</span>
                                    <select class="input" name="tz">
                                        {fmt::TIMEZONES.iter().map(|(z, _, _)| view! { <option value=*z>{*z}</option> }).collect_view()}
                                    </select></label>
                            </div>
                            <label class="field"><span class="field__label">"Nama Tempat"</span>
                                <input class="input" name="akad_venue" maxlength="120" placeholder="Masjid Agung Al-Ikhlas"
                                    on:input=move |e| venue.set(event_target_value(&e)) /></label>
                            <label class="field"><span class="field__label">"Alamat"</span>
                                <input class="input" name="akad_address" maxlength="200" placeholder="Jl. Kemang Raya No. 45, Jakarta Selatan" /></label>
                            <label class="field"><span class="field__label">"Link Google Maps (opsional)"</span>
                                <input class="input" name="akad_maps" type="url" maxlength="300" placeholder="https://maps.app.goo.gl/…" /></label>
                        </div>
                        <div class="subcard">
                            <div class="subcard__head"><span class="num num--gold">"2"</span><b>"Resepsi Pernikahan"</b><small>"Opsional • multi-sesi"</small></div>
                            <div class="field-row">
                                <label class="field"><span class="field__label">"Tanggal Resepsi"</span><input class="input" type="date" name="resepsi_date" /></label>
                                <label class="field"><span class="field__label">"Jam (mulai – selesai)"</span>
                                    <div class="field-row field-row--tight">
                                        <input class="input" type="time" name="resepsi_start" value="11:00" />
                                        <input class="input" type="time" name="resepsi_end" value="14:00" />
                                    </div></label>
                            </div>
                            <div class="field-row">
                                <label class="field"><span class="field__label">"Sesi 1 (Siang)"</span><input class="input" name="resepsi_s1" placeholder="11.30 – 14.30" /></label>
                                <label class="field"><span class="field__label">"Sesi 2 (Malam)"</span><input class="input" name="resepsi_s2" placeholder="18.30 – 21.00" /></label>
                            </div>
                            <label class="field"><span class="field__label">"Gedung / Venue"</span><input class="input" name="resepsi_venue" maxlength="120" placeholder="The Glass House Ballroom" /></label>
                            <label class="field"><span class="field__label">"Alamat"</span><input class="input" name="resepsi_address" maxlength="200" /></label>
                            <label class="field"><span class="field__label">"Link Google Maps (opsional)"</span><input class="input" type="url" name="resepsi_maps" maxlength="300" /></label>
                        </div>
                        <label class="field">
                            <span class="field__label"><Icon name="checkroom" />"Panduan Pakaian (Dress Code) Tamu"</span>
                            <input class="input" name="dress_code" maxlength="200" placeholder="Formal Batik Tradisional / Gaun sentuhan Sage Green & Champagne" />
                        </label>
                        <p class="field__label">"Warna busana yang dianjurkan (opsional — isi nama warna untuk menampilkannya)"</p>
                        <div class="dress-pick">
                            {[("Sage", "#8aa07d"), ("Champagne", "#f3d9a4"), ("", "#ffffff"), ("", "#2c2a27")].into_iter().enumerate().map(|(i, (n, hex))| view! {
                                <label class="dress-pick__opt">
                                    <input type="color" name=format!("dress{}_hex", i + 1) value=hex />
                                    <input class="input" name=format!("dress{}_name", i + 1) maxlength="30" value=n placeholder="Nama warna" />
                                </label>
                            }).collect_view()}
                        </div>
                        <label class="field">
                            <span class="field__label"><Icon name="smartphone" />"Tautan Live Streaming (opsional)"</span>
                            <input class="input" type="url" name="live_url" maxlength="300" placeholder="https://youtube.com/live/… atau https://instagram.com/…" />
                        </label>
                    </section>

                    // ── Kisah & galeri ──
                    <section class="card fsec">
                        <FsecHead icon="auto_stories" title="Love Story & Galeri" sub="Ceritakan perjalanan cinta dan bagikan momen terbaik (opsional)" tag="Bagian C+" />
                        {(1..=3).map(|i| view! {
                            <div class="story-row">
                                <input class="input" name=format!("story{i}_year") maxlength="20" placeholder=["2019", "2023", "2026"][i - 1] />
                                <input class="input" name=format!("story{i}_title") maxlength="80" placeholder=["Pertama Bertemu", "Lamaran", "Hari Bahagia"][i - 1] />
                                <textarea class="input" name=format!("story{i}_text") maxlength="500" rows="2" placeholder="Ceritakan singkat momen ini…"></textarea>
                            </div>
                        }).collect_view()}
                        <label class="upload">
                            <Icon name="cloud_upload" />
                            <span><b>"Foto galeri (maks 6 foto, masing-masing ≤ 5 MB)"</b><small>"Tampil sebagai kolase \"Our Moments\". Pilih beberapa foto sekaligus."</small></span>
                            <input type="file" name="gallery" accept="image/jpeg,image/png,image/webp" multiple data-gallery-max="6" />
                        </label>
                    </section>

                    // ── D. Musik & doa ──
                    <section class="card fsec">
                        <FsecHead icon="music_note" title="Media Audio & Untaian Doa" sub="Lagu latar pemikat suasana dan kata mutiara sakral" tag="Bagian D" />
                        <p class="field__label">"Pilihan Musik Latar"</p>
                        <div class="songs">
                            {SONGS.iter().enumerate().map(|(i, s)| view! {
                                <label class="song song--radio">
                                    <input type="radio" name="music_preset" value=s.slug checked=i == 0 />
                                    <span class="song__meta"><b>{s.title}</b><small>{format!("{} • {}", s.artist, s.duration)}</small></span>
                                    <span class="song__play" data-song=s.url() data-title=s.title title="Dengarkan">
                                        <Icon name="play_arrow" class="when-idle" /><Icon name="pause" class="when-playing" />
                                    </span>
                                </label>
                            }).collect_view()}
                        </div>
                        <div data-preview-box="">
                            <label class="upload">
                                <Icon name="cloud_upload" />
                                <span><b>"Upload lagu MP3 sendiri"</b><small>"Format .mp3 / .m4a, maksimal 6 MB — menggantikan pilihan di atas. Bisa langsung didengar; baru diunggah saat pesanan dikirim."</small></span>
                                <input type="file" name="music_file" accept="audio/mpeg,audio/mp4,audio/x-m4a,audio/ogg" data-preview="audio" data-max="6" />
                            </label>
                            <div class="preview-out" data-preview-out=""></div>
                        </div>
                        <crate::web::components::MusicSeek name="music_start" />
                        <label class="check"><input type="checkbox" name="music_autoplay" checked /><span>"Putar otomatis saat undangan dibuka"</span></label>
                        <label class="field">
                            <span class="field__label">"Ayat / Kutipan Doa Pembuka"</span>
                            <select class="input" name="quote">
                                {QUOTES.iter().enumerate().map(|(i, (text, src))| view! {
                                    <option value=i.to_string()>{format!("{src} — “{}…”", text.chars().take(60).collect::<String>())}</option>
                                }).collect_view()}
                            </select>
                        </label>
                        <audio id="bgm" preload="none"></audio>
                    </section>

                    // ── E. Rekening ──
                    <section class="card fsec">
                        <FsecHead icon="account_balance_wallet" title="Rekening Amplop Digital" sub="Tamu dapat menyalin nomor rekening sekali ketuk (opsional)" tag="Bagian E" />
                        {[1, 2].into_iter().map(|i| view! {
                            <div class="field-row field-row--3">
                                <input class="input" name=format!("bank{i}_name") placeholder=if i == 1 { "Bank (mis. BCA)" } else { "Bank kedua (opsional)" } />
                                <input class="input" name=format!("bank{i}_number") inputmode="numeric" placeholder="Nomor rekening" />
                                <input class="input" name=format!("bank{i}_holder") placeholder="Atas nama" />
                            </div>
                        }).collect_view()}
                    </section>
                </div>

                // ── Ringkasan (sticky) ──
                <aside class="buat__side">
                    <div class=move || format!("mini-live th-{}", theme.get())>
                        <p class="mini-live__label"><span>"Live Preview Miniatur"</span><span class="gold">"● Tersinkronisasi"</span></p>
                        <div class="mini-live__card">
                            <div class="monogram monogram--sm" inner_html=move || monogram_svg(&initials(), "THE WEDDING OF")></div>
                            <p class="mini__eyebrow">"Walimatul 'Urs"</p>
                            <h3>{move || {
                                let (b, g) = (bride.get(), groom.get());
                                let f = |x: &str| x.split_whitespace().next().unwrap_or("").to_string();
                                if b.is_empty() && g.is_empty() { "Nama & Pasangan".to_string() } else { format!("{} & {}", f(&b), f(&g)) }
                            }}</h3>
                            <p class="mini__date">{move || { let d = date.get(); if d.is_empty() { "Tanggal acara".into() } else { fmt::tanggal_panjang(&d) } }}</p>
                            <p class="muted small">{move || venue.get()}</p>
                        </div>
                    </div>

                    <section class="card summary">
                        <div class="side-card__head"><h3>"Rincian Paket"</h3></div>
                        <Suspense fallback=|| view! { <p class="muted small">"Memuat paket…"</p> }>
                            {move || konten.get().map(|r| {
                                let kk: Konten = r.unwrap_or_default();
                                let chosen = kk.package_or_default(&package.get_untracked()).slug;
                                let pk = kk.clone();
                                view! {
                                    {kk.paket.clone().into_iter().map(|p| {
                                        let (s1, s2, s3) = (p.slug.clone(), p.slug.clone(), p.slug.clone());
                                        let pk = pk.clone();
                                        view! {
                                            <label class="pkg-opt" class:is-on=move || pk.package_or_default(&package.get()).slug == s1>
                                                <input type="radio" name="package" value=s2 checked=chosen == p.slug
                                                    on:change=move |_| package.set(s3.clone()) />
                                                <span><b>{p.name.clone()}</b><small>{p.period.clone()}</small></span>
                                                <b>{rupiah(p.price)}</b>
                                            </label>
                                        }
                                    }).collect_view()}
                                    {(!kk.addon.is_empty()).then(|| view! { <p class="eyebrow">"Layanan Tambahan (Add-on)"</p> })}
                                    {kk.addon.clone().into_iter().map(|a| {
                                        let slug = a.slug.clone();
                                        view! {
                                            <label class="pkg-opt">
                                                <input type="checkbox" name="addon" value=a.slug.clone() on:change=move |_| toggle_addon(slug.clone()) />
                                                <span><b>{a.name.clone()}</b><small>{a.desc.clone()}</small></span>
                                                <b>{format!("+{}", rupiah(a.price))}</b>
                                            </label>
                                        }
                                    }).collect_view()}
                                }
                            })}
                        </Suspense>
                        <label class="field">
                            <span class="field__label">"Kupon Promo"</span>
                            <input class="input" name="coupon" maxlength="30" placeholder="Kode kupon (opsional)"
                                on:input=move |e| coupon.set(event_target_value(&e)) />
                        </label>
                        <Suspense fallback=|| ()>
                        <div class="totals">
                            <p><span>"Subtotal"</span><span>{move || rupiah(totals().0)}</span></p>
                            <Show when=move || { totals().1 > 0 }>
                                <p class="totals__disc"><span>"Kupon"</span><span>{move || format!("-{}", rupiah(totals().1))}</span></p>
                            </Show>
                            <p class="totals__grand"><span>"Total Pembayaran"</span><b>{move || rupiah(totals().2)}</b></p>
                        </div>
                        </Suspense>
                        <p class="eyebrow">"Pembayaran"</p>
                        // Satu-satunya cara bayar: transfer ke rekening Konten
                        // `pembayaran`, lalu unggah bukti di dashboard Kelola.
                        <input type="hidden" name="payment" value=PAYMENT_METHODS[0].0 />
                        <Suspense fallback=|| ()>
                            {move || { let p = k().pembayaran; view! {
                                <div class="payinfo">
                                    <span class="ms" aria-hidden="true">"account_balance_wallet"</span>
                                    <span>
                                        <small>{format!("Transfer {}", p.metode)}</small>
                                        <b>{p.nomor.clone()}</b>
                                        <small>{format!("a/n {} • bukti transfer diunggah setelah pesanan disimpan", p.atas_nama)}</small>
                                    </span>
                                </div>
                            } }}
                        </Suspense>
                        <label class="field">
                            <span class="field__label">"WhatsApp Pemesan (untuk konfirmasi)"</span>
                            <input class="input" name="contact_phone" required inputmode="tel" maxlength="20" placeholder="0812-xxxx-xxxx" />
                        </label>
                        <button type="submit" class="btn btn--primary btn--block btn--lg">
                            "Simpan Data & Lanjut Pembayaran"
                            <Icon name="arrow_forward" />
                        </button>
                        <p class="muted small center"><Icon name="lock" />" Pratinjau langsung (khusus Anda) • tamu bisa membuka setelah pembayaran • belum dibayar 24 jam = dihapus otomatis"</p>
                    </section>
                </aside>
            </form>
            <SiteFooter />
        </div>
    }
}

#[component]
fn FsecHead(icon: &'static str, title: &'static str, sub: &'static str, tag: &'static str) -> impl IntoView {
    view! {
        <div class="fsec__head">
            <span class="fsec__icon"><span class="ms" aria-hidden="true">{icon}</span></span>
            <div><h2>{title}</h2><p>{sub}</p></div>
            <span class="tag">{tag}</span>
        </div>
    }
}

#[component]
fn PersonFields(prefix: &'static str, label: &'static str, parent_label: &'static str, name: RwSignal<String>) -> impl IntoView {
    view! {
        <div class="subcard">
            <p class="eyebrow eyebrow--dot">{label}</p>
            <div class="field-row">
                <label class="field"><span class="field__label">"Nama Lengkap"</span>
                    <input class="input" name=format!("{prefix}_name") required maxlength="80" prop:value=move || name.get()
                        on:input=move |e| name.set(event_target_value(&e)) /></label>
                <label class="field"><span class="field__label">"Gelar (opsional)"</span>
                    <input class="input" name=format!("{prefix}_degree") maxlength="30" placeholder="S.Ds." /></label>
            </div>
            <div class="field-row">
                <label class="field"><span class="field__label">"Nama Panggilan"</span>
                    <input class="input" name=format!("{prefix}_nick") maxlength="30" /></label>
                <label class="field"><span class="field__label">"Username Instagram"</span>
                    <input class="input" name=format!("{prefix}_ig") maxlength="40" placeholder="@username" /></label>
            </div>
            <label class="field"><span class="field__label">{parent_label}" Orang Tua"</span>
                <input class="input" name=format!("{prefix}_parents") maxlength="200" placeholder=format!("{parent_label} Bpk. … & Ibu …") /></label>
            <div class="preview-row" data-preview-box="">
                <label class="upload">
                    <Icon name="photo_camera" />
                    <span><b>"Foto Mempelai"</b><small>"JPEG/PNG/WebP, maksimal 5 MB (opsional) — pratinjau langsung, diunggah saat pesanan dikirim"</small></span>
                    <input type="file" name=format!("{prefix}_photo") accept="image/jpeg,image/png,image/webp" data-preview="image" data-max="5"
                        data-pos-name=format!("{prefix}_photo_pos") />
                </label>
                <div class="preview-out preview-out--photo" data-preview-out=""></div>
            </div>
        </div>
    }
}
