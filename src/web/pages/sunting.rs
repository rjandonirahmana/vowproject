//! pages/sunting.rs — /kelola/{slug}/sunting: pemilik (kunci Kelola) atau
//! Admin menyunting ISI undangan yang sudah dipesan — mempelai, foto, acara,
//! busana, kisah & galeri, musik & video, kutipan, rekening, tema, WA pemesan.
//!
//! Form HTML biasa (multipart) → POST /kelola/{slug}/sunting/simpan
//! (server/handlers.rs update_invitation), jadi tetap jalan tanpa WASM. Nama
//! kolom SAMA dengan /buat (server memakai baca_isi yang sama). Paket & harga
//! tidak bisa diubah di sini.

use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::hooks::{use_params_map, use_query_map};

use super::buat::FsecHead;
use super::ErrorCard;
use crate::web::api::get_sunting;
use crate::web::components::{err_msg, BahasaUndangan, MusicSeek};
use crate::web::fmt;
use crate::web::icons::Icon;
use crate::web::model::{Event, Sunting};
use crate::web::skeleton::*;
use crate::web::themes::QUOTES;

#[component]
pub fn SuntingPage() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let slug = move || params.read().get("slug").unwrap_or_default();
    let key = move || query.read().get("key").unwrap_or_default();
    let notice = move || {
        let q = query.read();
        q.get("galat").map(|m| (false, m)).or_else(|| q.get("ok").map(|m| (true, m)))
    };
    let data = Resource::new(move || (slug(), key()), |(s, k)| get_sunting(s, k));
    view! {
        <Title text=concat!("Sunting Undangan — ", crate::brand!()) />
        <Meta name="robots" content="noindex, nofollow" />
        <Suspense fallback=|| view! { <div class="skl-pad"><SkelDashboard /></div> }>
            {move || data.get().map(|r| match r {
                Ok(d) => view! { <SuntingForm d=d notice=notice() /> }.into_any(),
                Err(e) => view! { <ErrorCard msg=err_msg(&e) /> }.into_any(),
            })}
        </Suspense>
    }
}

/// Nilai kolom acara (kosong bila acara itu belum ada).
fn ev<'a>(d: &'a Sunting, kind: &str) -> Option<&'a Event> {
    d.inv.events.iter().find(|e| e.kind == kind)
}

#[component]
fn SuntingForm(d: Sunting, notice: Option<(bool, String)>) -> impl IntoView {
    let inv = d.inv.clone();
    let slug = inv.slug.clone();
    let admin = d.manage_key.is_empty();
    let akad = ev(&d, "akad").cloned().or_else(|| inv.events.first().cloned()).unwrap_or_default();
    let resepsi = ev(&d, "resepsi").cloned().unwrap_or_default();
    let tz = if akad.tz.is_empty() { "WIB".to_string() } else { akad.tz.clone() };
    let sesi = |i: usize| resepsi.sessions.get(i).map(|s| s.time.clone()).unwrap_or_default();
    let (s1, s2) = (sesi(0), sesi(1));
    let akad_titles = {
        let mut v = vec!["Akad Nikah".to_string(), "Pemberkatan Nikah".to_string(), "Upacara Pernikahan".to_string()];
        if !akad.title.is_empty() && !v.contains(&akad.title) {
            v.insert(0, akad.title.clone());
        }
        v
    };
    let music_start = fmt::music_start(&inv.music_url);
    let musik_lain = d.song_id == 0 && !inv.music_url.is_empty();
    let video_luar = !inv.video_url.is_empty() && !inv.video_url.contains("/video/");
    let kembali = format!("/kelola/{slug}");

    view! {
        <div class="site sunting">
            <header class="sunting__top">
                <a class="btn btn--soft btn--sm" href=kembali.clone()><Icon name="arrow_back" />"Kembali ke Kelola"</a>
                <span class="sunting__title"><small>"Sunting Undangan"</small><b>{inv.couple()}</b></span>
                <a class="btn btn--soft btn--sm" href=format!("/u/{slug}") target="_blank" rel="external"><Icon name="visibility" />"Lihat Undangan"</a>
            </header>
            {admin.then(|| view! {
                <p class="notice notice--info dash-admin"><Icon name="admin_panel_settings" />"Mode admin — perubahan langsung tersimpan atas nama pemesan."</p>
            })}
            {notice.map(|(ok, m)| view! { <p class=if ok { "notice notice--ok dash-admin" } else { "notice notice--err dash-admin" }>{m}</p> })}

            <form class="buat buat--sunting" method="post" action=format!("/kelola/{slug}/sunting/simpan") enctype="multipart/form-data">
                <input type="hidden" name="key" value=d.manage_key.clone() />
                <div class="buat__main">
                    // ── Tema ──
                    <section class="card fsec">
                        <FsecHead icon="palette" title="Tema Undangan" sub="Ganti tampilan kapan saja — isi undangan tetap" tag="Tema" />
                        <label class="field"><span class="field__label">"Tema"</span>
                            <select class="input" name="theme">
                                {d.themes.iter().map(|(s, n, r)| view! {
                                    <option value=s.clone() selected=*s == inv.theme>{format!("{n} — {r}")}</option>
                                }).collect_view()}
                            </select>
                        </label>
                    </section>

                    // ── Mempelai ──
                    <section class="card fsec">
                        <FsecHead icon="favorite" title="Profil Mempelai" sub="Nama, gelar, orang tua, Instagram & foto" tag="Mempelai" />
                        <Orang prefix="bride" label="Mempelai Wanita" parent_label="Putri dari" name=inv.bride_name.clone() degree=inv.bride_degree.clone()
                            nick=inv.bride_nick.clone() ig=inv.bride_ig.clone() parents=inv.bride_parents.clone() photo=inv.bride_photo.clone() />
                        <Orang prefix="groom" label="Mempelai Pria" parent_label="Putra dari" name=inv.groom_name.clone() degree=inv.groom_degree.clone()
                            nick=inv.groom_nick.clone() ig=inv.groom_ig.clone() parents=inv.groom_parents.clone() photo=inv.groom_photo.clone() />
                        <div class="subcard">
                            <p class="eyebrow eyebrow--dot">"Foto Sampul Berdua"</p>
                            <Foto name="cover_photo" url=inv.cover_photo.clone() label="Foto sampul berdua" />
                        </div>
                        <label class="field"><span class="field__label">"Nama Keluarga Pengundang"</span>
                            <input class="input" name="family_name" maxlength="120" value=inv.family_name.clone() /></label>
                        <BahasaUndangan current=inv.language() />
                    </section>

                    // ── Acara ──
                    <section class="card fsec">
                        <FsecHead icon="calendar_month" title="Jadwal & Tempat Acara" sub="Akad / pemberkatan (wajib) dan resepsi" tag="Acara" />
                        <div class="subcard">
                            <div class="subcard__head"><span class="num">"1"</span><b>"Akad Nikah / Pemberkatan"</b><small class="gold">"Wajib"</small></div>
                            <div class="field-row">
                                <label class="field"><span class="field__label">"Hari & Tanggal"</span>
                                    <input class="input" type="date" name="akad_date" required value=akad.date.clone() /></label>
                                <label class="field"><span class="field__label">"Judul Acara"</span>
                                    <select class="input" name="akad_title">
                                        {akad_titles.into_iter().map(|t| { let sel = t == akad.title; view! { <option selected=sel>{t}</option> } }).collect_view()}
                                    </select></label>
                            </div>
                            <div class="field-row field-row--3">
                                <label class="field"><span class="field__label">"Mulai"</span><input class="input" type="time" name="akad_start" value=akad.time_start.clone() /></label>
                                <label class="field"><span class="field__label">"Selesai"</span><input class="input" type="time" name="akad_end" value=akad.time_end.clone() /></label>
                                <label class="field"><span class="field__label">"Zona waktu"</span>
                                    <select class="input" name="tz">
                                        {fmt::TIMEZONES.iter().map(|(z, _, _)| view! { <option value=*z selected=*z == tz>{*z}</option> }).collect_view()}
                                    </select></label>
                            </div>
                            <label class="field"><span class="field__label">"Nama Tempat"</span>
                                <input class="input" name="akad_venue" maxlength="120" value=akad.venue.clone() /></label>
                            <label class="field"><span class="field__label">"Alamat"</span>
                                <input class="input" name="akad_address" maxlength="200" value=akad.address.clone() /></label>
                            <label class="field"><span class="field__label">"Link Google Maps (opsional)"</span>
                                <input class="input" name="akad_maps" type="url" maxlength="300" value=akad.maps_url.clone() /></label>
                        </div>
                        <div class="subcard">
                            <div class="subcard__head"><span class="num num--gold">"2"</span><b>"Resepsi Pernikahan"</b><small>"Opsional — kosongkan tanggal untuk menghapus"</small></div>
                            <div class="field-row">
                                <label class="field"><span class="field__label">"Tanggal Resepsi"</span><input class="input" type="date" name="resepsi_date" value=resepsi.date.clone() /></label>
                                <label class="field"><span class="field__label">"Jam (mulai – selesai)"</span>
                                    <div class="field-row field-row--tight">
                                        <input class="input" type="time" name="resepsi_start" value=resepsi.time_start.clone() />
                                        <input class="input" type="time" name="resepsi_end" value=resepsi.time_end.clone() />
                                    </div></label>
                            </div>
                            <div class="field-row">
                                <label class="field"><span class="field__label">"Sesi 1 (Siang)"</span><input class="input" name="resepsi_s1" value=s1 placeholder="11.30 – 14.30" /></label>
                                <label class="field"><span class="field__label">"Sesi 2 (Malam)"</span><input class="input" name="resepsi_s2" value=s2 placeholder="18.30 – 21.00" /></label>
                            </div>
                            <label class="field"><span class="field__label">"Gedung / Venue"</span><input class="input" name="resepsi_venue" maxlength="120" value=resepsi.venue.clone() /></label>
                            <label class="field"><span class="field__label">"Alamat"</span><input class="input" name="resepsi_address" maxlength="200" value=resepsi.address.clone() /></label>
                            <label class="field"><span class="field__label">"Link Google Maps (opsional)"</span><input class="input" type="url" name="resepsi_maps" maxlength="300" value=resepsi.maps_url.clone() /></label>
                        </div>
                        <label class="field"><span class="field__label"><Icon name="checkroom" />"Dress Code Tamu"</span>
                            <input class="input" name="dress_code" maxlength="200" value=inv.dress_code.clone() /></label>
                        <p class="field__label">"Warna busana yang dianjurkan (kosongkan nama untuk menyembunyikan)"</p>
                        <div class="dress-pick">
                            {(0..4).map(|i| {
                                let c = inv.dress_colors.get(i).cloned().unwrap_or_default();
                                let hex = if c.hex.is_empty() { "#ffffff".to_string() } else { c.hex.clone() };
                                view! {
                                    <label class="dress-pick__opt">
                                        <input type="color" name=format!("dress{}_hex", i + 1) value=hex />
                                        <input class="input" name=format!("dress{}_name", i + 1) maxlength="30" value=c.name.clone() placeholder="Nama warna" />
                                    </label>
                                }
                            }).collect_view()}
                        </div>
                        <label class="field"><span class="field__label"><Icon name="smartphone" />"Tautan Live Streaming (opsional)"</span>
                            <input class="input" type="url" name="live_url" maxlength="300" value=inv.live_url.clone() /></label>
                    </section>

                    // ── Kisah & galeri ──
                    <section class="card fsec">
                        <FsecHead icon="auto_stories" title="Love Story & Galeri" sub="Kosongkan judul & cerita untuk menghapus satu babak" tag="Kisah" />
                        {(0..4).map(|i| {
                            let s = inv.love_story.get(i).cloned().unwrap_or_default();
                            view! {
                                <div class="story-row">
                                    <input class="input" name=format!("story{}_year", i + 1) maxlength="20" value=s.year.clone() placeholder="Tahun" />
                                    <input class="input" name=format!("story{}_title", i + 1) maxlength="80" value=s.title.clone() placeholder="Judul momen" />
                                    <textarea class="input" name=format!("story{}_text", i + 1) maxlength="500" rows="2" placeholder="Ceritakan singkat momen ini…"
                                        inner_html=crate::web::fmt::textarea_html(&s.text)></textarea>
                                </div>
                            }
                        }).collect_view()}
                        <p class="field__label">"Galeri foto (maks 6) — ganti, hapus, atau isi slot kosong; perubahan baru terjadi saat disimpan"</p>
                        <div class="slot-grid">
                            {(0..6usize).map(|i| match inv.gallery.get(i) {
                                Some(g) => view! {
                                    <SlotFoto
                                        url=g.clone()
                                        file_name=format!("galeri_ganti_{i}")
                                        hapus_name="galeri_hapus".to_string()
                                        hapus_value=i.to_string()
                                        label=format!("Foto {}", i + 1)
                                    />
                                }.into_any(),
                                None => view! {
                                    <SlotFoto url=String::new() file_name="galeri_baru".to_string() hapus_name=String::new() hapus_value=String::new() label=format!("Slot {}", i + 1) />
                                }.into_any(),
                            }).collect_view()}
                        </div>
                    </section>

                    // ── Musik, video & doa ──
                    <section class="card fsec" id="musik">
                        <FsecHead icon="music_note" title="Musik, Video & Doa" sub="Lagu latar dari pustaka, video prewedding, kutipan pembuka" tag="Media" />
                        <div class="songs">
                            {musik_lain.then(|| view! {
                                <label class="song song--radio">
                                    <input type="radio" name="music_song" value="-1" checked />
                                    <span class="song__meta"><b>{format!("Tetap: {}", if inv.music_title.is_empty() { "lagu sekarang".to_string() } else { inv.music_label() })}</b><small>"Lagu yang sedang dipakai"</small></span>
                                </label>
                            })}
                            {d.songs.iter().map(|s| view! {
                                <label class="song song--radio">
                                    <input type="radio" name="music_song" value=s.id.to_string() checked=s.id == d.song_id />
                                    <span class="song__meta"><b>{s.title.clone()}</b><small>{s.meta()}</small></span>
                                    {(!s.tag.is_empty()).then(|| view! { <span class="chip chip--gold chip--xs">{s.tag.clone()}</span> })}
                                    <span class="song__play" data-song=s.url.clone() data-title=s.title.clone() title="Dengarkan">
                                        <Icon name="play_arrow" class="when-idle" /><Icon name="pause" class="when-playing" />
                                    </span>
                                </label>
                            }).collect_view()}
                            <label class="song song--radio">
                                <input type="radio" name="music_song" value="0" checked=inv.music_url.is_empty() />
                                <span class="song__meta"><b>"Tanpa musik"</b></span>
                            </label>
                        </div>
                        <MusicSeek name="music_start" start=music_start />
                        <p class="muted small">"Memutar lagu lain mengembalikan titik mulai ke 0:00 — geser lagi bila perlu."</p>
                        <label class="check"><input type="checkbox" name="music_autoplay" checked=inv.music_autoplay /><span>"Putar otomatis saat undangan dibuka"</span></label>
                        <div class="subcard">
                            <p class="eyebrow eyebrow--dot">"Video Prewedding"</p>
                            {(!inv.video_url.is_empty()).then(|| view! {
                                <video class="sunting-video" src=inv.video_url.clone() controls preload="metadata" playsinline></video>
                                <label class="check"><input type="checkbox" name="hapus_video" /><span>"Hapus video"</span></label>
                            })}
                            <div data-preview-box="">
                                <label class="upload">
                                    <Icon name="movie" />
                                    <span><b>{if inv.video_url.is_empty() { "Unggah video (opsional)" } else { "Ganti video" }}</b><small>"MP4 / MOV / WebM, maksimal 20 MB."</small></span>
                                    <input type="file" name="video_file" accept="video/mp4,video/quicktime,video/webm" data-preview="video" data-max="20" />
                                </label>
                                <div class="preview-out" data-preview-out=""></div>
                            </div>
                            <label class="field"><span class="field__label">"…atau tautan langsung ke berkas video"</span>
                                <input class="input" name="video_link" maxlength="300" value=if video_luar { inv.video_url.clone() } else { String::new() } placeholder="https://…/prewedding.mp4" /></label>
                        </div>
                        <label class="field"><span class="field__label">"Ayat / Kutipan Doa Pembuka"</span>
                            <select class="input" name="quote">
                                {QUOTES.iter().enumerate().map(|(i, (text, src))| view! {
                                    <option value=i.to_string() selected=i == d.quote_idx>{format!("{src} — “{}…”", text.chars().take(60).collect::<String>())}</option>
                                }).collect_view()}
                            </select></label>
                        <audio id="bgm" preload="none"></audio>
                    </section>

                    // ── Rekening ──
                    <section class="card fsec">
                        <FsecHead icon="account_balance_wallet" title="Rekening Amplop Digital" sub="Kosongkan nomor untuk menghapus" tag="Amplop" />
                        {(0..2).map(|i| {
                            let b = inv.banks.get(i).cloned().unwrap_or_default();
                            view! {
                                <div class="field-row field-row--3">
                                    <input class="input" name=format!("bank{}_name", i + 1) value=b.bank.clone() placeholder="Bank (mis. BCA)" />
                                    <input class="input" name=format!("bank{}_number", i + 1) value=b.number.clone() inputmode="numeric" placeholder="Nomor rekening" />
                                    <input class="input" name=format!("bank{}_holder", i + 1) value=b.holder.clone() placeholder="Atas nama" />
                                </div>
                            }
                        }).collect_view()}
                    </section>
                </div>

                <aside class="buat__side">
                    <section class="card summary sunting__save">
                        <div class="side-card__head"><h3>"Simpan Perubahan"</h3></div>
                        <p class="muted small">"Paket & harga tidak berubah. Foto yang diganti/dihapus dibuang dari penyimpanan setelah tersimpan."</p>
                        <label class="field"><span class="field__label">"WhatsApp Pemesan"</span>
                            <input class="input" name="contact_phone" inputmode="tel" maxlength="20" value=d.contact_phone.clone() /></label>
                        <button type="submit" class="btn btn--primary btn--block btn--lg"><Icon name="check" />"Simpan Perubahan"</button>
                        <a class="btn btn--soft btn--block" href=kembali><Icon name="close" />"Batal"</a>
                    </section>
                </aside>
            </form>
        </div>
    }
}

/// Satu mempelai: kolom teks terisi + foto (tetap / ganti / hapus).
#[component]
fn Orang(
    prefix: &'static str,
    label: &'static str,
    parent_label: &'static str,
    name: String,
    degree: String,
    nick: String,
    ig: String,
    parents: String,
    photo: String,
) -> impl IntoView {
    view! {
        <div class="subcard">
            <p class="eyebrow eyebrow--dot">{label}</p>
            <div class="field-row">
                <label class="field"><span class="field__label">"Nama Lengkap"</span>
                    <input class="input" name=format!("{prefix}_name") required maxlength="80" value=name /></label>
                <label class="field"><span class="field__label">"Gelar (opsional)"</span>
                    <input class="input" name=format!("{prefix}_degree") maxlength="30" value=degree /></label>
            </div>
            <div class="field-row">
                <label class="field"><span class="field__label">"Nama Panggilan"</span>
                    <input class="input" name=format!("{prefix}_nick") maxlength="30" value=nick /></label>
                <label class="field"><span class="field__label">"Username Instagram"</span>
                    <input class="input" name=format!("{prefix}_ig") maxlength="40" value=ig placeholder="@username" /></label>
            </div>
            <label class="field"><span class="field__label">{parent_label}" Orang Tua"</span>
                <input class="input" name=format!("{prefix}_parents") maxlength="200" value=parents /></label>
            <Foto name=if prefix == "bride" { "bride_photo" } else { "groom_photo" } url=photo label=if prefix == "bride" { "Foto mempelai wanita" } else { "Foto mempelai pria" } />
        </div>
    }
}

/// Foto mempelai / sampul = satu slot (sama dengan slot galeri).
#[component]
fn Foto(name: &'static str, url: String, label: &'static str) -> impl IntoView {
    view! {
        <div class="slot-grid slot-grid--satu">
            <SlotFoto url=url file_name=name.to_string() hapus_name=format!("hapus_{name}") hapus_value="1".to_string() label=label.to_string() />
        </div>
    }
}

/// Satu slot foto: foto sekarang (atau kosong) + Ganti/Tambah (berkas dipilih
/// langsung tampil di slot — global.js data-slot) + Hapus (centang → pudar,
/// berkas dibuang dari RustFS saat disimpan). Tanda status murni CSS
/// (.is-new dari skrip, :has(:checked) untuk hapus).
#[component]
fn SlotFoto(url: String, file_name: String, hapus_name: String, hapus_value: String, label: String) -> impl IntoView {
    let src = url.split('#').next().unwrap_or("").to_string();
    let ada = !src.is_empty();
    let style = fmt::photo_style(&url);
    view! {
        <div class="slot" class:slot--kosong=!ada data-slot-tile="">
            <div class="slot__media">
                <img data-slot-img="" src=(!src.is_empty()).then(|| src.clone()) alt=label.clone() loading="lazy" style=style hidden=!ada />
                {(!ada).then(|| view! { <span class="slot__plus"><Icon name="add_photo_alternate" /></span> })}
                <span class="slot__tanda slot__tanda--baru">{if ada { "Akan diganti" } else { "Baru" }}</span>
                <span class="slot__tanda slot__tanda--hapus">"Akan dihapus"</span>
            </div>
            <div class="slot__aksi">
                <label class="slot__btn">
                    <Icon name=if ada { "swap_horiz" } else { "add" } />{if ada { "Ganti" } else { "Tambah" }}
                    <input type="file" name=file_name accept="image/jpeg,image/png,image/webp" data-slot="" />
                </label>
                <button type="button" class="slot__btn slot__btn--batal" data-slot-clear="">"Batal"</button>
                {(ada && !hapus_name.is_empty()).then(|| view! {
                    <label class="slot__btn slot__btn--hapus"><input type="checkbox" name=hapus_name value=hapus_value /><Icon name="delete" />"Hapus"</label>
                })}
            </div>
            <small class="slot__label">{label}</small>
        </div>
    }
}
