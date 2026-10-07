//! pages/admin.rs — panel admin: /admin, /admin/tema, /admin/tema/:slug
//! (`baru` = tema baru, `?dari=slug` = duplikat), /admin/undangan.
//!
//! Masuk dengan ADMIN_TOKEN (POST /admin/masuk → cookie HttpOnly). Semua
//! penyimpanan lewat form POST biasa ke handler axum (server/handlers.rs),
//! jadi tetap jalan tanpa WASM; setelah hydrate, editor tema menampilkan
//! pratinjau langsung memakai fungsi yang sama dengan /tema.css (web/skin.rs).

use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::hooks::{use_params_map, use_query_map};

use crate::web::anim::{self, AnimInfo, AnimSpec};
use crate::web::api::{admin_accounts, admin_animations, admin_banners, admin_invitations, admin_konten_meta, admin_session, admin_songs, admin_stories, admin_themes, get_konten, get_theme, AdminDeleteStory};
use crate::web::components::Monogram;
use crate::web::fmt::rupiah;
use crate::web::icons::Icon;
use crate::web::konten::{input_name, section, Kind, SECTIONS};
use crate::web::model::{AdminAnim, AdminSessionInfo, AdminTheme, AdminUser, ADMIN_ROLES, INV_STATUSES};
use crate::web::skin::{
    self, ThemeInfo, FONTS, IMAGE_MODES, LAYOUTS, MOTION_PRESETS, ORNAMENTS, PAGE_MODES, PALETTES, SCRIPT_FONTS, TOKENS,
};
use crate::web::components::{scroll_class, FloatDeco, GateFx, Ornamen, OrnLayer};
use crate::web::ornamen::{self as orn, Ornament, GERAK_ELEMEN};

/// Kerangka: cek sesi → form masuk / setup akun pertama, atau isi halaman +
/// navigasi sesuai peran. `admin_only` = halaman khusus peran Admin.
#[component]
fn AdminShell(
    active: &'static str,
    title: &'static str,
    #[prop(optional)] admin_only: bool,
    children: ChildrenFn,
) -> impl IntoView {
    let session = Resource::new(|| (), |_| admin_session());
    let q = use_query_map();
    let ok = move || q.read().get("ok").unwrap_or_default();
    let galat = move || q.read().get("galat").unwrap_or_default();
    let user = move || session.get().and_then(|r| r.ok()).and_then(|s| s.user);
    view! {
        <Title text=format!(concat!("{} — Admin ", crate::brand!()), title) />
        <Meta name="robots" content="noindex, nofollow" />
        <div class="site adm">
            <header class="adm-top">
                <a class="site-top__brand" href="/admin">
                    <Monogram initials=crate::web::BRAND_MONOGRAM class="monogram--xs" />
                    <span><b>{crate::brand!()}</b><small>"Panel Admin"</small></span>
                </a>
                <Suspense fallback=|| ()>
                    {move || user().map(|u| {
                        let admin = u.is_admin();
                        view! {
                            <nav class="adm-nav">
                                <a href="/admin/tema" class:is-active=active == "tema"><Icon name="palette" />"Tema"</a>
                                <a href="/admin/animasi" class:is-active=active == "animasi"><Icon name="animation" />"Animasi"</a>
                                <a href="/admin/banner" class:is-active=active == "banner"><Icon name="view_carousel" />"Banner"</a>
                                <a href="/admin/lagu" class:is-active=active == "lagu"><Icon name="library_music" />"Lagu"</a>
                                <a href="/admin/story" class:is-active=active == "story"><Icon name="photo_camera" />"Story"</a>
                                <a href="/admin/konten" class:is-active=active == "konten"><Icon name="edit_note" />"Konten & Harga"</a>
                                {admin.then(|| view! {
                                    <a href="/admin/undangan" class:is-active=active == "undangan"><Icon name="shopping_bag" />"Pesanan"</a>
                                    <a href="/admin/akun" class:is-active=active == "akun"><Icon name="groups" />"Akun"</a>
                                })}
                                <a href="/" target="_blank"><Icon name="visibility" />"Lihat situs"</a>
                                <a href="/admin/profil" class:is-active=active == "profil" class="adm-nav__me">
                                    <Icon name="person" />{u.display()}<span class="tag">{role_label(&u.role)}</span>
                                </a>
                                <form method="post" action="/admin/keluar"><button class="adm-nav__out" type="submit"><Icon name="logout" />"Keluar"</button></form>
                            </nav>
                        }
                    })}
                </Suspense>
            </header>
            <main class="adm-main">
                {move || (!ok().is_empty()).then(|| view! { <p class="notice notice--ok">{ok()}</p> })}
                {move || (!galat().is_empty()).then(|| view! { <p class="notice notice--err">{galat()}</p> })}
                <Suspense fallback=|| view! { <div class="inv-loading"><div class="spinner"></div></div> }>
                    {
                        let children = children.clone();
                        move || session.get().map(|r| {
                            let info = r.unwrap_or_default();
                            match info.user.clone() {
                                Some(u) if admin_only && !u.is_admin() => view! {
                                    <p class="notice notice--err">"Halaman ini khusus peran Admin. Minta Admin untuk menaikkan peran akun Anda bila perlu."</p>
                                }.into_any(),
                                Some(u) => {
                                    provide_context(u);
                                    children().into_any()
                                }
                                None => view! { <Login info=info /> }.into_any(),
                            }
                        })
                    }
                </Suspense>
            </main>
        </div>
    }
}

fn role_label(role: &str) -> &'static str {
    ADMIN_ROLES.iter().find(|r| r.0 == role).map(|r| r.1).unwrap_or("?")
}

#[component]
fn Login(info: AdminSessionInfo) -> impl IntoView {
    let body = if !info.db_ready {
        view! {
            <p class="muted">"Tabel akun admin belum ada. Jalankan "<code>"migration/003_admin_konten.sql"</code>" di database, lalu muat ulang halaman ini."</p>
        }
        .into_any()
    } else if info.needs_setup && !info.setup_code_set {
        view! {
            <p class="muted">"Belum ada akun admin. Isi "<code>"ADMIN_TOKEN"</code>" (minimal 12 karakter acak) di .env server, restart, lalu buat akun pertama di sini."</p>
        }
        .into_any()
    } else if info.needs_setup {
        view! {
            <p class="muted small">"Belum ada akun. Buat akun Admin pertama memakai kode setup "<code>"ADMIN_TOKEN"</code>" dari .env server. Setelah itu kode ini tidak dipakai lagi untuk masuk."</p>
            <form method="post" action="/admin/setup" class="stack adm-login__form">
                <label class="field"><span class="field__label">"Kode setup (ADMIN_TOKEN)"</span>
                    <span class="pw"><input class="input" type="password" name="code" required autocomplete="off" /><button type="button" class="pw__btn" data-pw-toggle="" aria-label="Tampilkan sandi" aria-pressed="false" title="Tampilkan sandi"><Icon name="visibility" class="pw__on" /><Icon name="visibility_off" class="pw__off" /></button></span></label>
                <label class="field"><span class="field__label">"Nama tampilan"</span>
                    <input class="input" name="name" maxlength="60" placeholder="Rina (Owner)" /></label>
                <label class="field"><span class="field__label">"Username"</span>
                    <input class="input" name="username" required minlength="3" maxlength="32" pattern="[a-z0-9._-]+" autocomplete="username" placeholder="rina" /></label>
                <label class="field"><span class="field__label">"Sandi (min. 8 karakter)"</span>
                    <span class="pw"><input class="input" type="password" name="password" required minlength="8" autocomplete="new-password" /><button type="button" class="pw__btn" data-pw-toggle="" aria-label="Tampilkan sandi" aria-pressed="false" title="Tampilkan sandi"><Icon name="visibility" class="pw__on" /><Icon name="visibility_off" class="pw__off" /></button></span></label>
                <label class="field"><span class="field__label">"Ulangi sandi"</span>
                    <span class="pw"><input class="input" type="password" name="password2" required minlength="8" autocomplete="new-password" /><button type="button" class="pw__btn" data-pw-toggle="" aria-label="Tampilkan sandi" aria-pressed="false" title="Tampilkan sandi"><Icon name="visibility" class="pw__on" /><Icon name="visibility_off" class="pw__off" /></button></span></label>
                <button class="btn btn--primary btn--block" type="submit">"Buat Akun Admin & Masuk"</button>
            </form>
        }
        .into_any()
    } else {
        view! {
            <form method="post" action="/admin/masuk" class="stack adm-login__form">
                <label class="field"><span class="field__label">"Username"</span>
                    <input class="input" name="username" required autocomplete="username" /></label>
                <label class="field"><span class="field__label">"Sandi"</span>
                    <span class="pw"><input class="input" type="password" name="password" required autocomplete="current-password" /><button type="button" class="pw__btn" data-pw-toggle="" aria-label="Tampilkan sandi" aria-pressed="false" title="Tampilkan sandi"><Icon name="visibility" class="pw__on" /><Icon name="visibility_off" class="pw__off" /></button></span></label>
                <button class="btn btn--primary btn--block" type="submit">"Masuk"</button>
            </form>
            <p class="muted small">"Lupa sandi? Minta Admin lain mengganti sandi Anda di menu Akun."</p>
        }
        .into_any()
    };
    view! {
        <section class="card adm-login">
            <Icon name="lock" class="empty-page__icon" />
            <h1>{if info.needs_setup { "Buat Akun Admin Pertama" } else { "Masuk Panel Admin" }}</h1>
            {body}
        </section>
    }
}

// ── /admin ─────────────────────────────────────────────────────────────────

#[component]
pub fn AdminHome() -> impl IntoView {
    let themes = Resource::new(|| (), |_| admin_themes());
    let invs = Resource::new(|| (), |_| admin_invitations(String::new()));
    view! {
        <AdminShell active="" title="Beranda">
            <h1 class="adm-h1">"Selamat datang, Admin"</h1>
            <div class="adm-stats">
                <Suspense fallback=|| ()>
                    {move || themes.get().and_then(|r| r.ok()).map(|list| {
                        let listed = list.iter().filter(|t| t.theme.listed).count();
                        view! {
                            <a class="card adm-stat" href="/admin/tema">
                                <Icon name="palette" />
                                <b>{list.len()}</b>
                                <span>{format!("tema • {listed} tampil di katalog")}</span>
                            </a>
                        }
                    })}
                    {move || invs.get().and_then(|r| r.ok()).map(|list| {
                        let wait = list.iter().filter(|i| i.status == "menunggu_pembayaran" && !i.is_demo).count();
                        view! {
                            <a class="card adm-stat" class:adm-stat--warn={wait > 0} href="/admin/undangan">
                                <Icon name="hourglass_top" />
                                <b>{wait}</b>
                                <span>"pesanan menunggu aktivasi"</span>
                            </a>
                        }
                    })}
                </Suspense>
            </div>
            <section class="card adm-help">
                <h2><Icon name="auto_awesome" />"Mengelola situs tanpa ngoding"</h2>
                <ol>
                    <li>"Harga, paket, foto dekorasi & MUA, galeri, dan testimoni ada di menu "<a href="/admin/konten">"Konten & Harga"</a>"."</li>
                    <li>"Buka "<a href="/admin/tema">"Tema"</a>" → "<b>"Duplikat"</b>" tema yang paling mirip (atau "<b>"Tema Baru"</b>")."</li>
                    <li>"Atur warna, huruf judul, tata letak sampul, dan ornamen — pratinjau di kanan langsung berubah."</li>
                    <li>"Opsional: unggah gambar ornamen/latar (PNG transparan untuk hiasan sudut)."</li>
                    <li>"Isi kategori & nuansa (mis. "<i>"Adat"</i>" / "<i>"Minang"</i>") — filter katalog terbentuk otomatis."</li>
                    <li>"Simpan → tema langsung tayang di katalog. Hapus centang "<b>"Tampil di katalog"</b>" untuk tema custom pesanan satu pelanggan, lalu kirim tautan "<code>"/buat?tema=kode"</code>" ke pelanggan itu."</li>
                </ol>
            </section>
        </AdminShell>
    }
}

// ── /admin/tema ────────────────────────────────────────────────────────────

#[component]
pub fn AdminThemes() -> impl IntoView {
    let themes = Resource::new(|| (), |_| admin_themes());
    view! {
        <AdminShell active="tema" title="Tema">
            <div class="adm-head">
                <h1 class="adm-h1">"Katalog Tema"</h1>
                <a class="btn btn--primary" href="/admin/tema/baru"><Icon name="add" />"Tema Baru"</a>
            </div>
            <Suspense fallback=|| ()>
                {move || themes.get().and_then(|r| r.ok()).map(|list| view! {
                    <div class="adm-themes">
                        {list.into_iter().map(|AdminTheme { theme: t, used, .. }| view! {
                            <article class="card adm-tcard">
                                <div class=format!("adm-tcard__art tcard__art th-{}", t.slug)>
                                    <div class="mini">
                                        <p class="mini__eyebrow">"The Wedding Of"</p>
                                        <p class="mini__names">"Yona & Doni"</p>
                                        <span class="mini__btn">"Buka"</span>
                                    </div>
                                </div>
                                <div class="adm-tcard__body">
                                    <b>{t.name.clone()}</b>
                                    <small><code>{t.slug.clone()}</code>{format!(" • {} • {}", t.category, t.nuansa)}</small>
                                    <p>
                                        {if t.listed {
                                            Either::Left(view! { <span class="status status--hadir">"Tampil"</span> })
                                        } else {
                                            Either::Right(view! { <span class="status status--ragu">"Privat"</span> })
                                        }}
                                        <span class="status">{format!("{used} undangan")}</span>
                                    </p>
                                    <div class="adm-tcard__btns">
                                        <a class="btn btn--primary btn--xs" href=format!("/admin/tema/{}", t.slug)>"Sunting"</a>
                                        <a class="btn btn--soft btn--xs" href=format!("/admin/tema/baru?dari={}", t.slug)>"Duplikat"</a>
                                        <a class="btn btn--soft btn--xs" href=format!("/tema/{}", t.slug) target="_blank">"Demo"</a>
                                    </div>
                                </div>
                            </article>
                        }).collect_view()}
                    </div>
                })}
            </Suspense>
        </AdminShell>
    }
}

// ── /admin/tema/:slug ──────────────────────────────────────────────────────

#[component]
pub fn AdminThemeEdit() -> impl IntoView {
    let params = use_params_map();
    let q = use_query_map();
    let slug = move || params.read().get("slug").unwrap_or_default();
    let themes = Resource::new(|| (), |_| admin_themes());
    let anims = Resource::new(|| (), |_| admin_animations());
    // Ornamen tema ini (untuk pratinjau sampul) — katalog admin tak membawanya.
    let full = Resource::new(slug, get_theme);
    view! {
        <AdminShell active="tema" title="Sunting Tema">
            <Suspense fallback=|| ()>
                {move || themes.get().and_then(|r| r.ok()).zip(anims.get()).zip(full.get()).map(|((list, anims), full)| {
                    let orns = full.ok().flatten().map(|t| t.ornaments).unwrap_or_default();
                    let anims: Vec<AnimInfo> = anims.unwrap_or_default().into_iter().map(|a| a.anim).collect();
                    let s = slug();
                    let is_new = s == "baru";
                    let src = if is_new { q.read_untracked().get("dari").unwrap_or_default() } else { s.clone() };
                    let found = list.iter().find(|a| a.theme.slug == src).map(|a| (a.theme.clone(), a.used));
                    let categories = uniq(list.iter().map(|a| a.theme.category.clone()));
                    let nuansa = uniq(list.iter().map(|a| a.theme.nuansa.clone()));
                    match (is_new, found) {
                        (false, None) => Either::Left(view! { <p class="notice notice--err">"Tema tidak ditemukan."</p> }),
                        (true, found) => {
                            let mut t = found.map(|f| f.0).unwrap_or_else(ThemeInfo::fallback);
                            if !src.is_empty() {
                                t.name = format!("{} (salinan)", t.name);
                            }
                            t.slug.clear();
                            t.image_url.clear();
                            t.badge.clear();
                            t.rating.clear();
                            t.reviews.clear();
                            t.sort_order = 100;
                            Either::Right(view! { <ThemeForm t=t used=0 is_new=true categories=categories nuansa=nuansa anims=anims /> })
                        }
                        (false, Some((t, used))) => Either::Right(view! { <ThemeForm t=crate::web::skin::ThemeInfo { ornaments: orns, ..t } used=used is_new=false categories=categories nuansa=nuansa anims=anims /> }),
                    }
                })}
            </Suspense>
        </AdminShell>
    }
}

fn uniq(it: impl Iterator<Item = String>) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    for s in it {
        if !s.is_empty() && !v.contains(&s) {
            v.push(s);
        }
    }
    v
}

/// `#abc` / `#aabbccdd` → `#aabbcc` untuk `<input type=color>`.
fn hex6(v: &str) -> String {
    let h = v.trim_start_matches('#');
    match h.len() {
        3 | 4 => format!("#{}", h.chars().take(3).flat_map(|c| [c, c]).collect::<String>()),
        6 | 8 => format!("#{}", &h[..6]),
        _ => "#000000".into(),
    }
}

#[component]
fn ThemeForm(t: ThemeInfo, used: i64, is_new: bool, categories: Vec<String>, nuansa: Vec<String>, anims: Vec<AnimInfo>) -> impl IntoView {
    // Pilihan animasi per jenis dari katalog (tabel animations): (kunci, nama).
    let opts_of = |kind: &str, none: Option<&str>| -> Vec<(String, String)> {
        none.map(|l| (anim::NONE.to_string(), l.to_string()))
            .into_iter()
            .chain(anims.iter().filter(|a| a.kind == kind).map(|a| (a.key(), a.name.clone())))
            .collect()
    };
    let (o_open, o_scroll, o_float) =
        (opts_of("buka", Some("Tanpa (sampul biasa)")), opts_of("scroll", None), opts_of("hiasan", Some("Tanpa hiasan")));
    let options = move |opts: Vec<(String, String)>, cur: String| {
        opts.into_iter().map(|(k, l)| { let sel = k == cur; view! { <option value=k selected=sel>{l}</option> } }).collect_view()
    };
    // Paket gaya hanya yang ketiga animasinya ada di katalog.
    let has = {
        let anims = anims.clone();
        move |kind: &str, k: &str| (k == anim::NONE && kind != "scroll") || anims.iter().any(|a| a.kind == kind && a.slug == k)
    };
    let presets: Vec<_> = MOTION_PRESETS.iter().filter(|p| has("buka", p.3) && has("scroll", p.4) && has("hiasan", p.5)).collect();
    let anims_sv = StoredValue::new(anims.clone());
    let has_adv = TOKENS.iter().any(|k| !k.main && t.tokens.contains_key(k.key));
    let adv = RwSignal::new(has_adv);
    // Ditambah setiap animasi diubah / "Coba animasi" → pratinjau dirender ulang.
    let replay = RwSignal::new(0u32);
    let draft = RwSignal::new({
        let mut t = t.clone();
        // Warna utama selalu terisi (input warna tak bisa kosong).
        for k in TOKENS.iter().filter(|k| k.main) {
            t.tokens.entry(k.key.to_string()).or_insert_with(|| k.default.to_string());
        }
        t
    });
    let preview_style = move || {
        let mut d = draft.get();
        if !adv.get() {
            d.tokens.retain(|k, _| TOKENS.iter().any(|t| t.main && t.key == k));
        }
        anims_sv.with_value(|a| skin::theme_vars_with(&d, a))
    };
    let set = move |f: fn(&mut ThemeInfo, String)| move |e: leptos::ev::Event| draft.update(|t| f(t, event_target_value(&e)));
    let fonts_href = format!(
        "https://fonts.googleapis.com/css2?family={}&display=swap",
        FONTS.iter().map(|f| f.3).chain(SCRIPT_FONTS.iter().map(|f| f.3)).filter(|f| !f.is_empty()).collect::<Vec<_>>().join("&family=")
    );
    let tok_input = move |k: &'static skin::Token| {
        let init = hex6(&draft.get_untracked().color(k.key));
        view! {
            <label class="adm-color">
                <input type="color" name=format!("tok_{}", k.key) value=init.clone()
                    on:input=move |e| { let v = event_target_value(&e); draft.update(|t| { t.tokens.insert(k.key.to_string(), v); }) } />
                <span>{k.label}<small>{format!("--{}", k.key)}</small></span>
            </label>
        }
    };
    let title = if is_new { "Tema Baru".to_string() } else { format!("Sunting: {}", t.name) };
    let slug = t.slug.clone();
    provide_context(OrnLayer(Signal::stored(t.ornaments.clone())));

    view! {
        <link rel="stylesheet" href=fonts_href />
        <div class="adm-head">
            <h1 class="adm-h1">{title}</h1>
            <a class="btn btn--soft btn--sm" href="/admin/tema"><Icon name="arrow_back" />"Semua tema"</a>
        </div>
        <div class="adm-edit">
            <form class="adm-form" method="post" action="/admin/tema/simpan" enctype="multipart/form-data">
                <input type="hidden" name="orig" value=if is_new { String::new() } else { slug.clone() } />

                <section class="card fsec">
                    <h2 class="adm-sec"><Icon name="edit_note" />"Identitas & katalog"</h2>
                    <div class="field-row">
                        <label class="field">
                            <span class="field__label">"Nama tema"</span>
                            <input class="input" name="name" required maxlength="80" value=t.name.clone() on:input=set(|t, v| t.name = v) />
                        </label>
                        <label class="field">
                            <span class="field__label">"Kode (slug)"</span>
                            {if is_new {
                                Either::Left(view! { <input class="input" name="slug" required minlength="3" maxlength="48" pattern="[a-z0-9-]+" placeholder="minang-songket" /> })
                            } else {
                                Either::Right(view! { <input class="input" value=slug.clone() disabled /> })
                            }}
                        </label>
                    </div>
                    <div class="field-row field-row--3">
                        <label class="field">
                            <span class="field__label">"Kategori"</span>
                            <input class="input" name="category" list="adm-cat" maxlength="40" value=t.category.clone() placeholder="Adat" />
                        </label>
                        <label class="field">
                            <span class="field__label">"Nuansa / budaya"</span>
                            <input class="input" name="nuansa" list="adm-nuansa" maxlength="40" value=t.nuansa.clone() placeholder="Minang" />
                        </label>
                        <label class="field">
                            <span class="field__label">"Palet (filter)"</span>
                            <select class="input" name="palette">
                                {PALETTES.iter().map(|(k, l, _)| view! { <option value=*k selected=t.palette == *k>{*l}</option> }).collect_view()}
                            </select>
                        </label>
                    </div>
                    <datalist id="adm-cat">{categories.into_iter().map(|c| view! { <option value=c></option> }).collect_view()}</datalist>
                    <datalist id="adm-nuansa">{nuansa.into_iter().map(|c| view! { <option value=c></option> }).collect_view()}</datalist>
                    <div class="field-row">
                        <label class="field">
                            <span class="field__label">"Label pojok kartu"</span>
                            <input class="input" name="region" maxlength="40" value=t.region.clone() placeholder="Ranah Minang" on:input=set(|t, v| t.region = v) />
                        </label>
                        <label class="field">
                            <span class="field__label">"Tag (pisahkan koma, maks 5)"</span>
                            <input class="input" name="tags" maxlength="200" value=t.tags.join(", ") placeholder="Amplop Digital, QR Check-In" />
                        </label>
                    </div>
                    <label class="field">
                        <span class="field__label">"Deskripsi singkat"</span>
                        <textarea class="input" name="description" maxlength="240" rows="2">{t.description.clone()}</textarea>
                    </label>
                    <div class="field-row field-row--3">
                        <label class="field">
                            <span class="field__label">"Lencana (opsional)"</span>
                            <input class="input" name="badge" maxlength="30" value=t.badge.clone() placeholder="Baru" on:input=set(|t, v| t.badge = v) />
                        </label>
                        <label class="field">
                            <span class="field__label">"Rating / ulasan (opsional)"</span>
                            <div class="field-row field-row--tight">
                                <input class="input" name="rating" maxlength="6" value=t.rating.clone() placeholder="4.9" />
                                <input class="input" name="reviews" maxlength="10" value=t.reviews.clone() placeholder="120" />
                            </div>
                        </label>
                        <label class="field">
                            <span class="field__label">"Urutan (kecil = depan)"</span>
                            <input class="input" name="sort_order" type="number" min="0" max="9999" value=t.sort_order.to_string() />
                        </label>
                    </div>
                    <label class="check"><input type="checkbox" name="listed" value="1" checked=t.listed />"Tampil di katalog publik (hapus centang = tema privat/custom pelanggan)"</label>
                </section>

                <section class="card fsec">
                    <h2 class="adm-sec"><Icon name="brush" />"Tata letak & huruf"</h2>
                    <div class="adm-layouts">
                        {LAYOUTS.iter().map(|(k, l, d, _)| view! {
                            <label class="adm-opt">
                                <input type="radio" name="layout" value=*k checked=t.layout == *k
                                    on:change=move |_| draft.update(|t| t.layout = k.to_string()) />
                                <span class=format!("adm-opt__art adm-lay-{k}")><i></i></span>
                                <b>{*l}</b>
                                <small>{*d}</small>
                            </label>
                        }).collect_view()}
                    </div>
                    <div class="field-row">
                        <label class="field">
                            <span class="field__label">"Huruf judul"</span>
                            <select class="input" name="font" on:change=set(|t, v| t.font = v)>
                                {FONTS.iter().map(|(k, l, fam, _)| view! {
                                    <option value=*k selected=t.font == *k style=format!("font-family:{fam}")>{*l}</option>
                                }).collect_view()}
                            </select>
                        </label>
                        <label class="field">
                            <span class="field__label">"Ornamen bawaan"</span>
                            <select class="input" name="ornament" on:change=set(|t, v| t.ornament = v)>
                                {ORNAMENTS.iter().map(|(k, l)| view! { <option value=*k selected=t.ornament == *k>{*l}</option> }).collect_view()}
                            </select>
                        </label>
                    </div>
                </section>

                <section class="card fsec">
                    <h2 class="adm-sec"><Icon name="palette" />"Warna"</h2>
                    <label class="check"><input type="checkbox" name="dark" value="1" checked=t.dark
                        on:change=move |e| draft.update(|t| t.dark = event_target_checked(&e)) />"Tema gelap (latar gelap, teks terang)"</label>
                    <div class="adm-colors">
                        {TOKENS.iter().filter(|k| k.main).map(tok_input).collect_view()}
                    </div>
                    <details class="adm-adv" open=has_adv>
                        <summary>"Warna lanjutan (opsional)"<Icon name="expand_more" /></summary>
                        <label class="check"><input type="checkbox" name="adv" value="1" checked=has_adv
                            on:change=move |e| adv.set(event_target_checked(&e)) />"Atur manual — tanpa centang, warna ini dihitung otomatis dari warna utama"</label>
                        <div class="adm-colors">
                            {TOKENS.iter().filter(|k| !k.main).map(tok_input).collect_view()}
                        </div>
                    </details>
                </section>

                <section class="card fsec">
                    <h2 class="adm-sec"><Icon name="play_arrow" />"Animasi undangan"</h2>
                    <div class="adm-anim-help">
                        <p><b>"1. Cara membuka"</b>" — yang dilihat tamu pertama kali: sampul tertutup, lalu terbuka saat menekan \"Buka Undangan\"."</p>
                        <p><b>"2. Gerak saat scroll"</b>" — cara tiap bagian (ayat, mempelai, acara, galeri…) muncul ketika tamu menggulir."</p>
                        <p><b>"3. Hiasan melayang"</b>" — dekorasi halus yang bergerak di latar selama undangan dibuka."</p>
                        <p class="muted small">"Pilih paket gaya untuk mengisi ketiganya sekaligus, lalu ubah satu per satu bila perlu. Tekan \"▶ Coba animasi\" di pratinjau untuk melihat hasilnya sebelum menyimpan."</p>
                    </div>
                    <p class="field__label">"Paket gaya (sekali klik)"</p>
                    <div class="adm-presets">
                        {presets.into_iter().map(|(_, name, desc, o, sc, f)| {
                            let (o, sc, f) = (*o, *sc, *f);
                            view! {
                                <button type="button" class="adm-preset"
                                    class:is-on=move || draft.with(|t| t.open_anim == o && t.scroll_anim == sc && t.float_deco == f)
                                    on:click=move |_| {
                                        draft.update(|t| { t.open_anim = o.into(); t.scroll_anim = sc.into(); t.float_deco = f.into(); });
                                        replay.update(|n| *n += 1);
                                    }>
                                    <b>{*name}</b>
                                    <small>{*desc}</small>
                                </button>
                            }
                        }).collect_view()}
                    </div>
                    <div class="field-row field-row--3">
                        <label class="field">
                            <span class="field__label">"1. Cara membuka"</span>
                            <select class="input" name="open_anim" prop:value=move || draft.with(|t| t.open_anim.clone())
                                on:change=move |e| { let v = event_target_value(&e); draft.update(|t| t.open_anim = v); replay.update(|n| *n += 1); }>
                                {options(o_open, t.open_anim.clone())}
                            </select>
                        </label>
                        <label class="field">
                            <span class="field__label">"2. Gerak saat scroll"</span>
                            <select class="input" name="scroll_anim" prop:value=move || draft.with(|t| t.scroll_anim.clone())
                                on:change=move |e| { let v = event_target_value(&e); draft.update(|t| t.scroll_anim = v); replay.update(|n| *n += 1); }>
                                {options(o_scroll, t.scroll_anim.clone())}
                            </select>
                        </label>
                        <label class="field">
                            <span class="field__label">"3. Hiasan melayang"</span>
                            <select class="input" name="float_deco" prop:value=move || draft.with(|t| t.float_deco.clone())
                                on:change=move |e| { let v = event_target_value(&e); draft.update(|t| t.float_deco = v); replay.update(|n| *n += 1); }>
                                {options(o_float, t.float_deco.clone())}
                            </select>
                        </label>
                    </div>
                    <div class="field-row field-row--3">
                        <label class="field">
                            <span class="field__label">"Gerak judul"</span>
                            <select class="input" name="gerak_judul"
                                on:change=move |e| { let v = event_target_value(&e); draft.update(|t| t.gerak_judul = v); replay.update(|n| *n += 1); }>
                                {GERAK_ELEMEN.iter().map(|g| view! { <option value=g.0 selected=t.gerak_judul == g.0 || (t.gerak_judul.is_empty() && g.0 == "ikut")>{g.1}</option> }).collect_view()}
                            </select>
                        </label>
                        <label class="field">
                            <span class="field__label">"Gerak foto"</span>
                            <select class="input" name="gerak_foto"
                                on:change=move |e| { let v = event_target_value(&e); draft.update(|t| t.gerak_foto = v); replay.update(|n| *n += 1); }>
                                {GERAK_ELEMEN.iter().map(|g| view! { <option value=g.0 selected=t.gerak_foto == g.0 || (t.gerak_foto.is_empty() && g.0 == "ikut")>{g.1}</option> }).collect_view()}
                            </select>
                        </label>
                        <label class="check adm-kb">
                            <input type="checkbox" name="ken_burns" value="1" checked=t.ken_burns
                                on:change=move |e| draft.update(|t| t.ken_burns = event_target_checked(&e)) />
                            "Foto sampul bergerak pelan (Ken Burns)"
                        </label>
                    </div>
                    <p class="adm-anim-note">
                        <Icon name="local_florist" />
                        {if is_new {
                            Either::Left(view! { <span>"Ornamen bergambar per bagian (bunga di sudut sampul, galeri, dll.) bisa diatur setelah tema disimpan."</span> })
                        } else {
                            Either::Right(view! { <span>"Hiasan bergambar per bagian — rangkaian bunga di sudut, animasi muncul & bergoyang: "<a href=format!("/admin/tema/{}/ornamen", t.slug)>"Atur Ornamen & Gerak →"</a></span> })
                        }}
                    </p>
                    <p class="adm-anim-note">
                        <Icon name="add_circle" />
                        <span>"Ingin mengubah atau menambah gaya? Semua animasi ada di menu "<a href="/admin/animasi" target="_blank">"Animasi"</a>" — setelah disimpan, muat ulang halaman ini."</span>
                    </p>
                    <p class="adm-anim-note">
                        <Icon name="info" />
                        {move || if draft.with(|t| t.open_anim.is_empty() || t.open_anim == "none") {
                            "Tanpa cara membuka: undangan memakai susunan yang dipilih (tab atau satu halaman)."
                        } else {
                            "Cara membuka dipilih: undangan otomatis tampil sebagai satu halaman panjang dengan sampul tertutup."
                        }}
                    </p>
                </section>

                <section class="card fsec">
                    <h2 class="adm-sec"><Icon name="auto_awesome" />"Kesan mewah: kaligrafi & ilustrasi"</h2>
                    <div class="field-row">
                        <label class="field">
                            <span class="field__label">"Huruf kaligrafi (judul seksi & nama panggilan)"</span>
                            <select class="input" name="script_font" on:change=set(|t, v| t.script_font = v)>
                                {SCRIPT_FONTS.iter().map(|(k, l, fam, _)| view! {
                                    <option value=*k selected=t.script_font == *k style=format!("font-family:{fam}")>{*l}</option>
                                }).collect_view()}
                            </select>
                        </label>
                        <label class="field">
                            <span class="field__label">"Susunan halaman"</span>
                            <select class="input" name="page_mode">
                                {PAGE_MODES.iter().map(|(k, l)| view! { <option value=*k selected=t.page_mode == *k>{*l}</option> }).collect_view()}
                            </select>
                            <small class="muted">"Bila \"Cara membuka\" di bagian Animasi dipilih, undangan otomatis satu halaman."</small>
                        </label>
                    </div>
                    {[
                        ("bg_image", "bg_file", "Ilustrasi latar penuh", "Lukisan/ilustrasi seluruh layar (JPG/WebP ±1080×1920). Kartu otomatis dibuat agak tembus pandang.", t.bg_image.clone()),
                        ("frame_image", "frame_file", "Bingkai foto", "PNG/SVG transparan berbentuk lengkung dengan bunga — dipasang di atas foto sampul & foto mempelai, boleh melebihi tepi foto.", t.frame_image.clone()),
                        ("card_deco", "deco_file", "Hiasan bunga di atas kartu", "PNG/SVG transparan melebar (±600×170) — rangkaian bunga di tepi atas tiap kartu.", t.card_deco.clone()),
                    ].into_iter().map(|(name, file, label, help, val)| view! {
                        <div class="field adm-wide">
                            <span class="field__label">{label}</span>
                            <div class="adm-img">
                                {(!val.is_empty()).then(|| view! { <img src=val.clone() alt="" loading="lazy" decoding="async" /> })}
                                <div class="adm-img__in">
                                    <input class="input" name=name maxlength="500" value=val placeholder="/img/tema/… atau https://…"
                                        on:input=move |e| { let v = event_target_value(&e); draft.update(|t| match name {
                                            "bg_image" => t.bg_image = v, "frame_image" => t.frame_image = v, _ => t.card_deco = v,
                                        }) } />
                                    <input class="input" type="file" name=file accept="image/png,image/jpeg,image/webp" />
                                </div>
                            </div>
                            <small class="muted">{help}</small>
                        </div>
                    }).collect_view()}
                    <label class="field adm-wide">
                        <span class="field__label">"Video latar (tema sinema)"</span>
                        <input class="input" name="bg_video" maxlength="500" value=t.bg_video.clone() placeholder="/video/… atau https://…/latar.mp4"
                            on:input=set(|t, v| t.bg_video = v) />
                        <small class="muted">"Diisi = video diputar penuh di belakang isi undangan (tanpa suara, berulang). Video prewedding pasangan otomatis menggantikannya. MP4 (H.264) paling kompatibel; WebM juga didukung. Akhiri URL dengan #loop=2.6 agar pengulangan mulai dari detik 2,6 (bagian pembuka video hanya tampil sekali)."</small>
                    </label>
                    <label class="field adm-wide">
                        <span class="field__label">"Video pembuka (gerbang)"</span>
                        <input class="input" name="open_video" maxlength="500" value=t.open_video.clone() placeholder="/video/…-buka.mp4"
                            on:input=set(|t, v| t.open_video = v) />
                        <small class="muted">"Diputar sekali saat tamu menekan \"Buka Undangan\" (pakai cara membuka \"Video pintu\"). Frame terakhirnya sebaiknya sama dengan frame pertama video latar agar sambungannya mulus."</small>
                    </label>
                </section>

                <section class="card fsec">
                    <h2 class="adm-sec"><Icon name="photo_camera" />"Gambar ornamen / latar (opsional)"</h2>
                    <p class="muted small">"Menimpa ornamen bawaan. PNG/WebP transparan maks 5 MB — hiasan sudut paling cantik dengan gambar ±600px."</p>
                    <div class="field-row">
                        <label class="field">
                            <span class="field__label">"Unggah gambar"</span>
                            <input class="input" type="file" name="image_file" accept="image/png,image/jpeg,image/webp" />
                        </label>
                        <label class="field">
                            <span class="field__label">"…atau URL gambar"</span>
                            <input class="input" name="image_url" maxlength="500" value=t.image_url.clone() placeholder="https://…/ornamen.png"
                                on:input=set(|t, v| t.image_url = v) />
                        </label>
                    </div>
                    <div class="field-row">
                        <label class="field">
                            <span class="field__label">"Cara tampil"</span>
                            <select class="input" name="image_mode" on:change=set(|t, v| t.image_mode = v)>
                                {IMAGE_MODES.iter().map(|(k, l)| view! { <option value=*k selected=t.image_mode == *k>{*l}</option> }).collect_view()}
                            </select>
                        </label>
                        {(!t.image_url.is_empty()).then(|| view! {
                            <label class="check adm-clear"><input type="checkbox" name="image_clear" value="1" />"Hapus gambar"</label>
                        })}
                    </div>
                </section>

                <div class="adm-save">
                    <button class="btn btn--primary btn--lg" type="submit"><Icon name="check" />{if is_new { "Simpan & Tayangkan" } else { "Simpan Perubahan" }}</button>
                    {(!is_new).then(|| view! {
                        <a class="btn btn--soft" href=format!("/tema/{}", slug) target="_blank"><Icon name="visibility" />"Buka demo"</a>
                        <a class="btn btn--soft" href=format!("/admin/tema/{}/ornamen", slug)><Icon name="local_florist" />"Ornamen"</a>
                    })}
                </div>
            </form>

            <aside class="adm-preview">
                <p class="eyebrow">"Pratinjau langsung"</p>
                <button type="button" class="btn btn--soft btn--sm adm-replay" on:click=move |_| replay.update(|n| *n += 1)>
                    <Icon name="play_arrow" />"Coba animasi"
                </button>
                <div class="adm-preview__inv" style=preview_style>
                    // Di-key: tiap "Coba animasi" / ganti animasi membuat DOM BARU.
                    // Tanpa key, Leptos memakai ulang elemen lama (kelas is-open &
                    // tanda animasi scroll dari skrip tetap menempel → tak ada yang diputar ulang).
                    <For
                        each=move || { let (a, d, sc) = draft.with(|t| (t.open_anim.clone(), t.float_deco.clone(), t.scroll_anim.clone())); vec![(replay.get(), a, d, sc)] }
                        key=|k| k.clone()
                        children=move |(_, anim, deco, scroll)| {
                        let gate = !anim.is_empty() && anim != "none";
                        let cover = || view! {
                            <section class="hero cover orn-host">
                                <Ornamen bagian="sampul" />
                                <p class="script cover__eyebrow">"The Wedding Of"</p>
                                <div class="arch-photo arch-photo--cover">
                                    <span class="arch-photo__clip"><img src="/img/layanan/mua-sesudah.jpg" alt="" /></span>
                                    <span class="arch-photo__frame" aria-hidden="true"></span>
                                </div>
                                <h1 class="cover__names">"Yona & Doni"</h1>
                                <p class="hero__date"><Icon name="local_florist" />"Sabtu, 24 Oktober 2026"</p>
                            </section>
                        };
                        view! {
                            <div class=format!("inv inv--embed adm-prev{}", scroll_class(&scroll)) class:is-open=!gate>
                                <div class="inv__glow" aria-hidden="true"></div>
                                <FloatDeco kind=deco />
                                {gate.then(|| view! {
                                    <div class=format!("gate gate--{anim} gate--embed")>
                                        <div class="gate__panel gate__panel--l" aria-hidden="true"></div>
                                        <div class="gate__panel gate__panel--r" aria-hidden="true"></div>
                                        <div class="gate__orn" aria-hidden="true"></div>
                                        <GateFx />
                                        <div class="gate__content">
                                            {cover()}
                                            <button type="button" class="btn btn--gold btn--lg gate__open" data-demo-open="1">
                                                <Icon name="drafts" />"Buka Undangan"
                                            </button>
                                        </div>
                                    </div>
                                })}
                                <div class="inv__main">
                                    {(!gate).then(cover)}
                                    <section class="quote card">
                                        <span class="quote__mark">"99"</span>
                                        <blockquote>"“Dan di antara tanda-tanda kebesaran-Nya ialah Dia menciptakan pasangan untukmu…”"</blockquote>
                                    </section>
                                    <h2 class="section__title">"Love Story"</h2>
                                    <section class="guest card">
                                        <p class="muted">"Kepada Yth."</p>
                                        <h2 class="guest__name">"Bapak Sandiaga & Keluarga"</h2>
                                        <span class="chip chip--gold">{move || { let b = draft.get().badge; if b.is_empty() { "Tamu VIP".to_string() } else { b } }}</span>
                                    </section>
                                    <div class="event card">
                                        <span class="event__tag">"Akad Nikah"</span>
                                        <h3 class="event__title">"Masjid Agung"</h3>
                                        <p class="event__meta"><Icon name="schedule" />"08.00 – 10.00 WIB"</p>
                                    </div>
                                    <div class="event card">
                                        <span class="event__tag">"Resepsi"</span>
                                        <h3 class="event__title">"Gedung Serbaguna"</h3>
                                        <p class="event__meta"><Icon name="schedule" />"11.00 – 14.00 WIB"</p>
                                    </div>
                                </div>
                            </div>
                        }
                    } />
                </div>
                <p class="muted small">{move || { let d = draft.get(); format!("Kartu katalog: {} • {}", d.name, d.region) }}</p>
                {(!is_new).then(|| view! {
                    <details class="adm-danger">
                        <summary>"Hapus tema"</summary>
                        {if used > 0 {
                            Either::Left(view! { <p class="small">{format!("Dipakai {used} undangan — tidak bisa dihapus. Sembunyikan dari katalog saja.")}</p> })
                        } else {
                            Either::Right(view! {
                                <form method="post" action="/admin/tema/hapus">
                                    <input type="hidden" name="slug" value=slug.clone() />
                                    <p class="small">"Tema belum dipakai undangan mana pun. Penghapusan permanen."</p>
                                    <button class="btn btn--sm adm-btn-danger" type="submit"><Icon name="delete" />"Ya, hapus permanen"</button>
                                </form>
                            })
                        }}
                    </details>
                })}
            </aside>
        </div>
    }
}

// ── /admin/animasi ─────────────────────────────────────────────────────────

/// Kode pratinjau — kelas CSS terpisah dari animasi tersimpan di /tema.css.
const PREVIEW_SLUG: &str = "pratinjau-admin";

fn kind_label(kind: &str) -> &'static str {
    anim::KINDS.iter().find(|k| k.0 == kind).map(|k| k.1).unwrap_or("Animasi")
}

fn kind_icon(kind: &str) -> &'static str {
    match kind {
        "buka" => "drafts",
        "scroll" => "swipe_up",
        _ => "auto_awesome",
    }
}

#[component]
pub fn AdminAnims() -> impl IntoView {
    let anims = Resource::new(|| (), |_| admin_animations());
    view! {
        <AdminShell active="animasi" title="Animasi">
            <div class="adm-head">
                <h1 class="adm-h1">"Animasi Undangan"</h1>
            </div>
            <div class="adm-anim-help">
                <p>"Semua animasi undangan online tersimpan di database: cara membuka, gerak saat scroll, dan hiasan melayang. Animasi bawaan pun bisa diubah, dan animasi baru bisa ditambah tanpa kode."</p>
                <p class="muted small">"Perubahan langsung tayang di semua tema yang memakainya. Pilih animasi untuk tiap tema di "<a href="/admin/tema">"editor tema"</a>" (bagian \"Animasi undangan\")."</p>
            </div>
            <Suspense fallback=|| ()>
                {move || anims.get().map(|r| {
                    let list = r.unwrap_or_default();
                    anim::KINDS.iter().map(|(kind, label, desc)| {
                        let kind = *kind;
                        let mine: Vec<AdminAnim> = list.iter().filter(|a| a.anim.kind == kind).cloned().collect();
                        view! {
                            <section class="card fsec adm-anims">
                                <h2 class="adm-sec"><Icon name=kind_icon(kind) />{*label}</h2>
                                <p class="muted small">{*desc}</p>
                                <div class="adm-anims__list">
                                    {mine.into_iter().map(|AdminAnim { anim: a, used_by }| view! {
                                        <a class="adm-anims__item" href=format!("/admin/animasi/{}/{}", a.kind, a.slug)>
                                            <b>{a.name.clone()}</b>
                                            <small>
                                                <code>{a.key()}</code>
                                                {if a.builtin { " • bawaan" } else { " • buatan admin" }}
                                                {if a.css.trim().is_empty() { " • pengaturan" } else { " • CSS" }}
                                            </small>
                                            <small class="muted">{if used_by.is_empty() { "Belum dipakai tema".to_string() } else { format!("Dipakai: {}", used_by.join(", ")) }}</small>
                                        </a>
                                    }).collect_view()}
                                </div>
                                <p class="field__label">"Tambah baru dari templat"</p>
                                <div class="adm-presets">
                                    {anim::templates().into_iter().filter(|t| t.0 == kind).map(|(_, key, name, _)| view! {
                                        <a class="adm-preset" href=format!("/admin/animasi/baru?jenis={kind}&templat={key}")>
                                            <b><Icon name="add" />{name}</b>
                                        </a>
                                    }).collect_view()}
                                    <a class="adm-preset" href=format!("/admin/animasi/baru?jenis={kind}&mode=css")>
                                        <b><Icon name="code" />"Dari CSS sendiri"</b>
                                    </a>
                                </div>
                            </section>
                        }
                    }).collect_view()
                })}
            </Suspense>
        </AdminShell>
    }
}

/// /admin/animasi/baru (templat / duplikat / CSS) & /admin/animasi/:kind/:slug.
#[component]
pub fn AdminAnimEdit() -> impl IntoView {
    let params = use_params_map();
    let q = use_query_map();
    let anims = Resource::new(|| (), |_| admin_animations());
    view! {
        <AdminShell active="animasi" title="Sunting Animasi">
            <Suspense fallback=|| ()>
                {move || anims.get().map(|r| {
                    let list = r.unwrap_or_default();
                    let p = params.read();
                    let q = q.read_untracked();
                    match (p.get("kind"), p.get("slug")) {
                        (Some(kind), Some(slug)) => match list.into_iter().find(|a| a.anim.kind == kind && a.anim.slug == slug) {
                            Some(AdminAnim { anim: a, used_by }) => Either::Left(view! { <AnimForm a=a used_by=used_by is_new=false /> }),
                            None => Either::Right(view! { <p class="notice notice--err">"Animasi tidak ditemukan."</p> }),
                        },
                        _ => {
                            let jenis = q.get("jenis").filter(|j| anim::KINDS.iter().any(|k| k.0 == j)).unwrap_or_else(|| "buka".to_string());
                            // Duplikat (?jenis=&dari=slug), CSS kosong (?mode=css), atau templat (?templat=).
                            let dari = q.get("dari").unwrap_or_default();
                            let a = match list.iter().find(|a| !dari.is_empty() && a.anim.kind == jenis && a.anim.slug == dari) {
                                Some(src) => AnimInfo {
                                    slug: String::new(),
                                    name: format!("{} (salinan)", src.anim.name),
                                    builtin: false,
                                    sort_order: 100,
                                    ..src.anim.clone()
                                },
                                None if q.get("mode").as_deref() == Some("css") => AnimInfo {
                                    kind: jenis.clone(),
                                    name: "Animasi baru".into(),
                                    css: css_starter(&jenis).into(),
                                    sort_order: 100,
                                    ..Default::default()
                                },
                                None => {
                                    let key = q.get("templat").unwrap_or_default();
                                    let tpls = anim::templates();
                                    let t = tpls.iter().find(|t| t.0 == jenis && t.1 == key).or_else(|| tpls.iter().find(|t| t.0 == jenis));
                                    AnimInfo {
                                        kind: jenis.clone(),
                                        name: t.map(|t| format!("{} (baru)", t.2)).unwrap_or_default(),
                                        spec: t.map(|t| t.3.clone()).unwrap_or_default(),
                                        sort_order: 100,
                                        ..Default::default()
                                    }
                                }
                            };
                            Either::Left(view! { <AnimForm a=a used_by=vec![] is_new=true /> })
                        }
                    }
                })}
            </Suspense>
        </AdminShell>
    }
}

/// Kerangka awal mode CSS lanjutan per jenis.
fn css_starter(kind: &str) -> &'static str {
    match kind {
        "scroll" => "--rv-from:translateY(40px) scale(0.96);--rv-dur:0.9s;",
        "hiasan" => "{a} i { width: 14px; height: 14px; border-radius: 50%; background: var(--gold-light); }",
        _ => ".inv-opened {a}:not(.gate--embed) .gate__panel, .is-open > {a} .gate__panel { transition: transform 1.2s cubic-bezier(0.7, 0, 0.2, 1) 0.25s; }\n.inv-opened {a}:not(.gate--embed) .gate__panel--l, .is-open > {a} .gate__panel--l { transform: translateX(-101%); }\n.inv-opened {a}:not(.gate--embed) .gate__panel--r, .is-open > {a} .gate__panel--r { transform: translateX(101%); }",
    }
}

type SpecGet<T> = fn(&AnimSpec) -> T;
type SpecSet<T> = fn(&mut AnimSpec, T);

/// Penggeser angka (nama field = nama kolom spec).
fn range_field(
    draft: RwSignal<AnimInfo>,
    name: &'static str,
    label: &'static str,
    (min, max, step): (i32, i32, i32),
    unit: &'static str,
    get: SpecGet<i32>,
    set: SpecSet<i32>,
) -> impl IntoView {
    let init = get(&draft.get_untracked().spec);
    view! {
        <label class="field adm-range">
            <span class="field__label">{label}<b>{move || format!("{}{unit}", draft.with(|a| get(&a.spec)))}</b></span>
            <input type="range" name=name min=min.to_string() max=max.to_string() step=step.to_string() value=init.to_string()
                on:input=move |e| { let v = event_target_value(&e).parse().unwrap_or(init); draft.update(|a| set(&mut a.spec, v)); } />
        </label>
    }
}

fn select_field(
    draft: RwSignal<AnimInfo>,
    name: &'static str,
    label: &'static str,
    opts: Vec<(&'static str, &'static str)>,
    get: SpecGet<String>,
    set: SpecSet<String>,
) -> impl IntoView {
    let init = get(&draft.get_untracked().spec);
    view! {
        <label class="field">
            <span class="field__label">{label}</span>
            <select class="input" name=name prop:value=move || draft.with(|a| get(&a.spec))
                on:change=move |e| { let v = event_target_value(&e); draft.update(|a| set(&mut a.spec, v)); }>
                {opts.into_iter().map(|(k, l)| view! { <option value=k selected=init == k>{l}</option> }).collect_view()}
            </select>
        </label>
    }
}

fn check_field(draft: RwSignal<AnimInfo>, name: &'static str, label: &'static str, get: SpecGet<bool>, set: SpecSet<bool>) -> impl IntoView {
    view! {
        <label class="check">
            <input type="checkbox" name=name value="1" checked=get(&draft.get_untracked().spec)
                on:change=move |e| { let v = event_target_checked(&e); draft.update(|a| set(&mut a.spec, v)); } />
            {label}
        </label>
    }
}

/// Gambar: URL (langsung terlihat di pratinjau) atau unggah (tampil setelah disimpan).
fn image_field(
    draft: RwSignal<AnimInfo>,
    name: &'static str,
    file: &'static str,
    label: &'static str,
    help: &'static str,
    get: SpecGet<String>,
    set: SpecSet<String>,
) -> impl IntoView {
    let val = get(&draft.get_untracked().spec);
    view! {
        <div class="field adm-wide">
            <span class="field__label">{label}</span>
            <div class="adm-img">
                {move || { let v = draft.with(|a| get(&a.spec)); (!v.is_empty()).then(|| view! { <img src=v alt="" loading="lazy" decoding="async" /> }) }}
                <div class="adm-img__in">
                    <input class="input" name=name maxlength="500" value=val placeholder="/img/… atau https://… (kosong = tanpa gambar)"
                        on:input=move |e| { let v = event_target_value(&e); draft.update(|a| set(&mut a.spec, v)); } />
                    <input class="input" type="file" name=file accept="image/png,image/jpeg,image/webp" />
                </div>
            </div>
            <small class="muted">{help}</small>
        </div>
    }
}

#[component]
fn AnimForm(a: AnimInfo, used_by: Vec<String>, is_new: bool) -> impl IntoView {
    let kind = a.kind.clone();
    let slug = a.slug.clone();
    let draft = RwSignal::new(a.clone());
    let replay = RwSignal::new(0u32);
    let new_slug = RwSignal::new(crate::web::fmt::slug(&a.name, anim::KEY_MAX));
    let slug_manual = RwSignal::new(false);
    let title = if is_new { format!("{} baru", kind_label(&kind)) } else { format!("Sunting: {}", a.name) };
    // Mode: pengaturan (spec) atau CSS lanjutan.
    let css_mode = RwSignal::new(!a.css.trim().is_empty());
    let builtin = a.builtin;
    // Animasi yang dipratinjau: CSS lanjutan hanya dipakai di mode CSS.
    let effective = move || draft.with(|a| AnimInfo { slug: PREVIEW_SLUG.into(), css: if css_mode.get() { a.css.clone() } else { String::new() }, ..a.clone() });
    // CSS pratinjau: fungsi yang sama dengan /tema.css, dengan kode khusus pratinjau.
    let preview_css = move || anim::css(&effective());
    let preview_vars = move || anim::scroll_vars_of(&effective());
    let css_error = move || {
        if !css_mode.get() {
            return None;
        }
        draft.with(|a| anim::sanitize_css(&a.kind, &a.css).err().or_else(|| a.css.trim().is_empty().then(|| "CSS masih kosong.".to_string())))
    };
    let css_help = match kind.as_str() {
        "scroll" => "Isi deklarasi variabel saja (tanpa { }): --rv-from (posisi awal: transform), --rv-from-alt (elemen genap), --rv-filter, --rv-op (opasitas awal), --rv-dur (lama), --rv-origin. ATAU koreografi lengkap dengan {a} (= akar undangan), mis. .rv-on {a} .section__title[data-rv]:not(.is-in) { transform: scale(.3); } — lihat bawaan \"keraton\".",
        "hiasan" => "Tulis {a} untuk kelas hiasan ini. Tiap butir = elemen i (14 buah, variabel --i = 0…13; standarnya 9 tampil). Keyframes siap pakai: fd-fall, fd-fly, fd-twinkle.",
        _ => "Tulis {a} untuk kelas sampul ini. Bagian: .gate__panel--l / .gate__panel--r (dua pintu), .gate__content (nama & tombol), .gate__orn (ornamen), {a}::before. Keadaan TERBUKA ditulis dengan pola: .inv-opened {a}:not(.gate--embed) X, .is-open > {a} X.",
    };
    let pkey = PREVIEW_SLUG.to_string();
    let d = draft;
    let fields = match kind.as_str() {
        "buka" => view! {
            <section class="card fsec">
                <h2 class="adm-sec"><Icon name="door_open" />"Sampul / pintu"</h2>
                <div class="field-row">
                    {select_field(d, "split", "Bentuk sampul", anim::SPLITS.to_vec(), |s| s.split.clone(), |s, v| s.split = v)}
                    {select_field(d, "fill", "Isi sampul", anim::FILLS.to_vec(), |s| s.fill.clone(), |s, v| s.fill = v)}
                </div>
                {image_field(d, "panel_image", "panel_file", "Gambar sampul (bila isi = gambar)", "JPG/PNG/WebP — terbelah mengikuti bentuk sampul. Unggahan otomatis memakai isi \"Gambar unggahan\".", |s| s.panel_image.clone(), |s, v| s.panel_image = v)}
                <div class="adm-checks">
                    {check_field(d, "border", "Garis emas di tepi", |s| s.border, |s, v| s.border = v)}
                    {check_field(d, "fade", "Memudar sambil bergerak", |s| s.fade, |s, v| s.fade = v)}
                </div>
            </section>
            <section class="card fsec">
                <h2 class="adm-sec"><Icon name="animation" />"Gerakan saat dibuka"</h2>
                <div class="field-row">
                    {range_field(d, "move_pct", "Jarak geser", (0, 110, 1), "%", |s| s.move_pct, |s, v| s.move_pct = v)}
                    {range_field(d, "rotate", "Putar 3D (berayun)", (-180, 180, 5), "°", |s| s.rotate, |s, v| s.rotate = v)}
                </div>
                <div class="field-row">
                    {range_field(d, "scale", "Ukuran akhir", (50, 150, 5), "%", |s| s.scale, |s, v| s.scale = v)}
                    {select_field(d, "easing", "Irama gerak", anim::EASINGS.iter().map(|e| (e.0, e.1)).collect(), |s| s.easing.clone(), |s, v| s.easing = v)}
                </div>
                <div class="field-row">
                    {range_field(d, "duration_ms", "Lama gerak", (300, 3000, 50), " ms", |s| s.duration_ms, |s, v| s.duration_ms = v)}
                    {range_field(d, "delay_ms", "Jeda sebelum mulai", (0, 1500, 50), " ms", |s| s.delay_ms, |s, v| s.delay_ms = v)}
                </div>
                {select_field(d, "content_exit", "Nama & tombol di sampul", anim::EXITS.to_vec(), |s| s.content_exit.clone(), |s, v| s.content_exit = v)}
            </section>
            <section class="card fsec">
                <h2 class="adm-sec"><Icon name="local_florist" />"Ornamen tengah (opsional)"</h2>
                {image_field(d, "orn_image", "orn_file", "Gambar ornamen", "PNG/SVG transparan — mis. segel lilin, monogram, rangkaian bunga. Kosongkan bila tak perlu.", |s| s.orn_image.clone(), |s, v| s.orn_image = v)}
                <div class="field-row">
                    {select_field(d, "orn_effect", "Efek ornamen", anim::ORN_EFFECTS.to_vec(), |s| s.orn_effect.clone(), |s, v| s.orn_effect = v)}
                    {range_field(d, "orn_size", "Ukuran ornamen", (30, 200, 5), " px", |s| s.orn_size, |s, v| s.orn_size = v)}
                </div>
            </section>
        }
        .into_any(),
        "scroll" => view! {
            <section class="card fsec">
                <h2 class="adm-sec"><Icon name="swipe_up" />"Posisi awal sebelum muncul"</h2>
                <p class="muted small">"Tiap bagian mulai dari posisi ini lalu bergerak ke tempatnya saat terlihat."</p>
                <div class="field-row">
                    {range_field(d, "dx", "Geser mendatar", (-120, 120, 2), " px", |s| s.dx, |s, v| s.dx = v)}
                    {range_field(d, "dy", "Geser tegak (+ = dari bawah)", (-120, 120, 2), " px", |s| s.dy, |s, v| s.dy = v)}
                </div>
                <div class="field-row">
                    {range_field(d, "scale_from", "Ukuran awal", (60, 140, 2), "%", |s| s.scale_from, |s, v| s.scale_from = v)}
                    {range_field(d, "rot", "Miring awal", (-45, 45, 1), "°", |s| s.rot, |s, v| s.rot = v)}
                </div>
                <div class="field-row">
                    {range_field(d, "blur", "Buram awal", (0, 20, 1), " px", |s| s.blur, |s, v| s.blur = v)}
                    {range_field(d, "duration_ms", "Lama gerak", (300, 3000, 50), " ms", |s| s.duration_ms, |s, v| s.duration_ms = v)}
                </div>
                {check_field(d, "mirror", "Selang-seling: bagian genap datang dari arah berlawanan", |s| s.mirror, |s, v| s.mirror = v)}
            </section>
        }
        .into_any(),
        _ => view! {
            <section class="card fsec">
                <h2 class="adm-sec"><Icon name="auto_awesome" />"Hiasan melayang"</h2>
                {image_field(d, "float_image", "float_file", "Gambar hiasan", "PNG/SVG transparan kecil (kelopak, hati, daun, kupu…). Kosong = titik cahaya emas.", |s| s.float_image.clone(), |s, v| s.float_image = v)}
                <div class="field-row">
                    {select_field(d, "float_motion", "Gerakan", anim::FLOAT_MOTIONS.to_vec(), |s| s.float_motion.clone(), |s, v| s.float_motion = v)}
                    {range_field(d, "float_count", "Jumlah", (3, 14, 1), "", |s| s.float_count, |s, v| s.float_count = v)}
                </div>
                <div class="field-row">
                    {range_field(d, "float_size", "Ukuran", (8, 60, 1), " px", |s| s.float_size, |s, v| s.float_size = v)}
                    {range_field(d, "float_speed", "Lama satu putaran (makin besar makin pelan)", (4, 30, 1), " dtk", |s| s.float_speed, |s, v| s.float_speed = v)}
                </div>
            </section>
        }
        .into_any(),
    };
    let is_open_kind = kind != "buka";
    let kind_p = kind.clone();

    view! {
        <div class="adm-head">
            <h1 class="adm-h1">{title}</h1>
            <a class="btn btn--soft btn--sm" href="/admin/animasi"><Icon name="arrow_back" />"Semua animasi"</a>
        </div>
        <div class="adm-edit">
            <form class="adm-form" method="post" action="/admin/animasi/simpan" enctype="multipart/form-data">
                <input type="hidden" name="orig" value=if is_new { String::new() } else { slug.clone() } />
                <input type="hidden" name="kind" value=kind.clone() />
                <section class="card fsec">
                    <h2 class="adm-sec"><Icon name=kind_icon(&kind) />{kind_label(&kind)}</h2>
                    <div class="field-row">
                        <label class="field">
                            <span class="field__label">"Nama animasi"</span>
                            <input class="input" name="name" required maxlength="60" value=a.name.clone() placeholder="Pintu Emas Berayun"
                                on:input=move |e| {
                                    let v = event_target_value(&e);
                                    // Kode ikut nama selama belum diketik manual.
                                    if is_new && !slug_manual.get_untracked() { new_slug.set(crate::web::fmt::slug(&v, anim::KEY_MAX)); }
                                    draft.update(|a| a.name = v);
                                } />
                        </label>
                        <label class="field">
                            <span class="field__label">"Kode"</span>
                            {if is_new {
                                Either::Left(view! {
                                    <input class="input" name="slug" required maxlength="40" pattern="[a-z0-9-]+"
                                        value=crate::web::fmt::slug(&a.name, anim::KEY_MAX) prop:value=move || new_slug.get() placeholder="pintu-emas"
                                        on:input=move |e| { slug_manual.set(true); new_slug.set(event_target_value(&e)); } />
                                })
                            } else {
                                Either::Right(view! { <input class="input" value=a.key() disabled /> })
                            }}
                        </label>
                    </div>
                    <p class="muted small">"Kode tak bisa diganti setelah disimpan (dipakai tema untuk mengenali animasinya)."</p>
                </section>
                <section class="card fsec">
                    <h2 class="adm-sec"><Icon name="tune" />"Cara mengatur"</h2>
                    {builtin.then(|| view! { <p class="notice notice--info"><Icon name="info" />"Animasi bawaan — boleh diubah sesuka hati; tombol \"Kembalikan ke bawaan\" di samping memulihkan aslinya."</p> })}
                    <input type="hidden" name="mode" prop:value=move || if css_mode.get() { "css" } else { "pengaturan" } />
                    <div class="seg adm-mode">
                        <label class="seg__opt">
                            <input type="radio" name="_mode" checked=!css_mode.get_untracked() on:change=move |_| { css_mode.set(false); replay.update(|n| *n += 1); } />
                            <b>"Pengaturan"</b><small>"Geser & pilih — tanpa kode"</small>
                        </label>
                        <label class="seg__opt">
                            <input type="radio" name="_mode" checked=css_mode.get_untracked() on:change=move |_| { css_mode.set(true); replay.update(|n| *n += 1); } />
                            <b>"CSS lanjutan"</b><small>"Bebas penuh, untuk desainer"</small>
                        </label>
                    </div>
                </section>
                <div style:display=move || if css_mode.get() { "none" } else { "" }>{fields}</div>
                <section class="card fsec" style:display=move || if css_mode.get() { "" } else { "none" }>
                    <h2 class="adm-sec"><Icon name="code" />"CSS lanjutan"</h2>
                    <p class="muted small">{css_help}</p>
                    <textarea class="input adm-code" name="css" rows="16" spellcheck="false"
                        prop:value=move || draft.with(|a| a.css.clone())
                        on:input=move |e| { let v = event_target_value(&e); draft.update(|a| a.css = v); }>{a.css.clone()}</textarea>
                    {move || css_error().map(|m| view! { <p class="notice notice--err">{m}</p> })}
                    <p class="muted small">"Tidak boleh memuat karakter <, @import, atau javascript:. Pratinjau di samping langsung mengikuti CSS ini."</p>
                </section>
                <div class="adm-save">
                    <button class="btn btn--primary btn--lg" type="submit"><Icon name="check" />{if is_new { "Simpan Animasi" } else { "Simpan Perubahan" }}</button>
                    {(!is_new).then(|| view! {
                        <a class="btn btn--soft" href=format!("/admin/animasi/baru?jenis={kind}&dari={slug}")><Icon name="content_copy" />"Duplikat"</a>
                    })}
                </div>
            </form>

            <aside class="adm-preview">
                <p class="eyebrow">"Pratinjau langsung"</p>
                <style inner_html=preview_css></style>
                <button type="button" class="btn btn--soft btn--sm adm-replay" on:click=move |_| replay.update(|n| *n += 1)>
                    <Icon name="play_arrow" />"Coba animasi"
                </button>
                <div class=format!("adm-preview__inv th-{}", skin::DEFAULT_THEME) style=preview_vars>
                    // Di-key: tiap "Coba animasi" membuat DOM baru (lihat ThemeForm).
                    <For each=move || vec![replay.get()] key=|k| *k children=move |n| {
                        let open = RwSignal::new(is_open_kind);
                        // Tombol "Coba animasi" pada animasi buka → sampul terbuka sendiri.
                        #[cfg(feature = "hydrate")]
                        if n > 0 && !is_open_kind {
                            set_timeout(move || open.set(true), std::time::Duration::from_millis(600));
                        }
                        let _ = n;
                        let deco = if kind_p == "hiasan" { pkey.clone() } else { "none".to_string() };
                        let gate = kind_p == "buka";
                        let rvs = if kind_p == "scroll" { scroll_class(PREVIEW_SLUG) } else { String::new() };
                        view! {
                            <div class=format!("inv inv--embed adm-prev{rvs}") class:is-open=move || open.get()>
                                <div class="inv__glow" aria-hidden="true"></div>
                                <FloatDeco kind=deco />
                                {gate.then(|| view! {
                                    <div class=format!("gate gate--{PREVIEW_SLUG} gate--embed")>
                                        <div class="gate__panel gate__panel--l" aria-hidden="true"></div>
                                        <div class="gate__panel gate__panel--r" aria-hidden="true"></div>
                                        <div class="gate__orn" aria-hidden="true"></div>
                                        <GateFx />
                                        <div class="gate__content">
                                            <section class="hero cover">
                                                <p class="script cover__eyebrow">"The Wedding Of"</p>
                                                <h1 class="cover__names">"Yona & Doni"</h1>
                                                <p class="hero__date"><Icon name="local_florist" />"Sabtu, 24 Oktober 2026"</p>
                                            </section>
                                            <button type="button" class="btn btn--gold btn--lg gate__open" data-demo-open="1">
                                                <Icon name="drafts" />"Buka Undangan"
                                            </button>
                                        </div>
                                    </div>
                                })}
                                <div class="inv__main">
                                    <section class="hero cover">
                                        <p class="script cover__eyebrow">"The Wedding Of"</p>
                                        <h1 class="cover__names">"Yona & Doni"</h1>
                                    </section>
                                    <section class="quote card">
                                        <span class="quote__mark">"99"</span>
                                        <blockquote>"“Dan di antara tanda-tanda kebesaran-Nya ialah Dia menciptakan pasangan untukmu…”"</blockquote>
                                    </section>
                                    {["Akad Nikah", "Resepsi", "Unduh Mantu"].into_iter().map(|e| view! {
                                        <div class="event card">
                                            <span class="event__tag">{e}</span>
                                            <h3 class="event__title">"Gedung Serbaguna"</h3>
                                            <p class="event__meta"><Icon name="schedule" />"08.00 – 10.00 WIB"</p>
                                        </div>
                                    }).collect_view()}
                                </div>
                            </div>
                        }
                    } />
                </div>
                <p class="muted small">"Gambar yang diunggah baru tampil di pratinjau setelah disimpan; URL gambar langsung terlihat."</p>
                {(!is_new && builtin).then(|| {
                    let (k2, s2) = (kind.clone(), slug.clone());
                    view! {
                        <details class="adm-danger">
                            <summary>"Kembalikan ke bawaan"</summary>
                            <form method="post" action="/admin/animasi/bawaan">
                                <input type="hidden" name="kind" value=k2 />
                                <input type="hidden" name="slug" value=s2 />
                                <p class="small">"Semua perubahan pada animasi bawaan ini diganti dengan isi aslinya."</p>
                                <button class="btn btn--sm adm-btn-danger" type="submit"><Icon name="restart_alt" />"Ya, kembalikan"</button>
                            </form>
                        </details>
                    }
                })}
                {(!is_new && !builtin).then(|| {
                    let (k2, s2) = (kind.clone(), slug.clone());
                    view! {
                        <details class="adm-danger">
                            <summary>"Hapus animasi"</summary>
                            {if !used_by.is_empty() {
                                Either::Left(view! { <p class="small">{format!("Dipakai tema: {} — ganti animasi di tema itu dulu.", used_by.join(", "))}</p> })
                            } else {
                                Either::Right(view! {
                                    <form method="post" action="/admin/animasi/hapus">
                                        <input type="hidden" name="kind" value=k2 />
                                        <input type="hidden" name="slug" value=s2 />
                                        <p class="small">"Belum dipakai tema mana pun. Penghapusan permanen."</p>
                                        <button class="btn btn--sm adm-btn-danger" type="submit"><Icon name="delete" />"Ya, hapus permanen"</button>
                                    </form>
                                })
                            }}
                        </details>
                    }
                })}
            </aside>
        </div>
    }
}

// ── /admin/banner ──────────────────────────────────────────────────────────

fn banner_status(s: &str) -> (&'static str, &'static str) {
    match s {
        "tayang" => ("Tayang", "status--hadir"),
        "terjadwal" => ("Terjadwal", "status--ragu"),
        "berakhir" => ("Berakhir", "status--tidak"),
        _ => ("Nonaktif", ""),
    }
}

#[component]
pub fn AdminBanner() -> impl IntoView {
    let list = Resource::new(|| (), |_| admin_banners());
    view! {
        <AdminShell active="banner" title="Banner">
            <div class="adm-head">
                <h1 class="adm-h1">"Banner Beranda"</h1>
                <a class="btn btn--soft btn--sm" href="/" target="_blank"><Icon name="visibility" />"Lihat beranda"</a>
            </div>
            <div class="adm-anim-help">
                <p>"Strip promo tipis di atas beranda, berganti tiap 6 detik. Gambar sebaiknya TANPA teks — judul, subjudul & tombol ditulis di atasnya dari sini."</p>
                <p class="muted small">"Ukuran ideal: desktop 2880×320 px (9:1), HP 1080×405 px (8:3); ilustrasi di sisi kanan, kiri polos/gelap untuk teks. Urutan diatur dengan ↑/↓; jadwal tayang opsional (WIB)."</p>
            </div>
            <Suspense fallback=|| ()>
                {move || list.get().map(|r| match r {
                    Err(e) => view! { <p class="notice notice--err">{crate::web::components::err_msg(&e)}</p> }.into_any(),
                    Ok(items) => {
                        let n = items.len();
                        view! {
                            <div class="adm-banners">
                                {items.into_iter().enumerate().map(|(i, b)| view! { <BannerForm b=b pos=i total=n /> }).collect_view()}
                                <details class="card adm-banner adm-banner--new" open=n == 0>
                                    <summary><Icon name="add" />"Tambah banner baru"</summary>
                                    <BannerFields b=crate::web::model::Banner { aktif: true, ..Default::default() } />
                                </details>
                            </div>
                        }
                        .into_any()
                    }
                })}
            </Suspense>
        </AdminShell>
    }
}

#[component]
fn BannerForm(b: crate::web::model::Banner, pos: usize, total: usize) -> impl IntoView {
    let (label, cls) = banner_status(&b.status);
    let id = b.id.to_string();
    view! {
        <article class="card adm-banner" id=format!("banner-{}", b.id)>
            <div class="adm-banner__head">
                <span class="adm-banner__no">{pos + 1}</span>
                <b>{if b.judul.is_empty() { format!("Banner #{}", b.id) } else { b.judul.clone() }}</b>
                <span class=format!("status {cls}")>{label}</span>
                <span class="adm-banner__move">
                    <form method="post" action="/admin/banner/urut">
                        <input type="hidden" name="id" value=id.clone() /><input type="hidden" name="arah" value="naik" />
                        <button class="icon-btn" type="submit" disabled=pos == 0 aria-label="Naikkan urutan"><Icon name="arrow_upward" /></button>
                    </form>
                    <form method="post" action="/admin/banner/urut">
                        <input type="hidden" name="id" value=id.clone() /><input type="hidden" name="arah" value="turun" />
                        <button class="icon-btn" type="submit" disabled=pos + 1 == total aria-label="Turunkan urutan"><Icon name="arrow_downward" /></button>
                    </form>
                </span>
            </div>
            <div class="adm-banner__prev">
                <img src=b.img.clone() alt="" loading="lazy" decoding="async" />
                <div class="adm-banner__over">
                    <b>{b.judul.clone()}</b>
                    <small>{b.sub.clone()}</small>
                </div>
            </div>
            <details>
                <summary>"Sunting"</summary>
                <BannerFields b=b.clone() />
                <form method="post" action="/admin/banner/hapus" class="adm-banner__del">
                    <input type="hidden" name="id" value=id />
                    <button class="btn btn--sm adm-btn-danger" type="submit"><Icon name="delete" />"Hapus banner ini"</button>
                </form>
            </details>
        </article>
    }
}

#[component]
fn BannerFields(b: crate::web::model::Banner) -> impl IntoView {
    let new = b.id == 0;
    let img_field = |name: &'static str, file: &'static str, label: &'static str, help: &'static str, val: String| view! {
        <div class="field adm-wide">
            <span class="field__label">{label}</span>
            <div class="adm-img">
                {(!val.is_empty()).then(|| view! { <img src=val.clone() alt="" loading="lazy" decoding="async" /> })}
                <div class="adm-img__in">
                    <input class="input" name=name maxlength="500" value=val placeholder="/img/banner/… atau https://…" />
                    <input class="input" type="file" name=file accept="image/jpeg,image/png,image/webp" />
                </div>
            </div>
            <small class="muted">{help}</small>
        </div>
    };
    view! {
        <form class="adm-banner__form" method="post" action="/admin/banner/simpan" enctype="multipart/form-data">
            <input type="hidden" name="id" value=b.id.to_string() />
            <div class="field-row">
                <label class="field"><span class="field__label">"Judul"</span>
                    <input class="input" name="judul" maxlength="120" value=b.judul.clone() placeholder="Diskon Musim Nikah 40%" /></label>
                <label class="field"><span class="field__label">"Teks tombol (opsional)"</span>
                    <input class="input" name="cta" maxlength="40" value=b.cta.clone() placeholder="Lihat Promo" /></label>
            </div>
            <label class="field"><span class="field__label">"Subjudul (opsional, disembunyikan di HP)"</span>
                <input class="input" name="sub" maxlength="300" value=b.sub.clone() /></label>
            <label class="field"><span class="field__label">"Tautan"</span>
                <input class="input" name="link" maxlength="300" value=b.link.clone() placeholder="/buat, /dekorasi, /#katalog, /u/{demo} (undangan demo), atau https://…" /></label>
            {img_field("img", "img_file", "Gambar desktop (wajib)", "Strip 2880×320 px (9:1), JPG/PNG/WebP maks 5 MB — unggahan masuk RustFS.", b.img.clone())}
            {img_field("img_hp", "img_hp_file", "Gambar HP (opsional)", "1080×405 px (8:3). Kosong = memakai gambar desktop.", b.img_hp.clone())}
            <div class="field-row field-row--3">
                <label class="field"><span class="field__label">"Tayang mulai (opsional)"</span>
                    <input class="input" type="datetime-local" name="mulai" value=b.mulai.clone() /></label>
                <label class="field"><span class="field__label">"Tayang sampai (opsional)"</span>
                    <input class="input" type="datetime-local" name="selesai" value=b.selesai.clone() /></label>
                <label class="field"><span class="field__label">"Urutan"</span>
                    <input class="input" type="number" name="urutan" value=b.urutan.to_string() step="1" /></label>
            </div>
            <label class="check"><input type="checkbox" name="aktif" value="1" checked=b.aktif />"Aktif (tampil di beranda)"</label>
            <div class="adm-banner__save">
                <button class="btn btn--primary" type="submit"><Icon name="check" />{if new { "Tambahkan Banner" } else { "Simpan Banner" }}</button>
            </div>
        </form>
    }
}

// ── /admin/undangan ────────────────────────────────────────────────────────

#[component]
pub fn AdminUndangan() -> impl IntoView {
    let q = use_query_map();
    let term = move || q.read().get("q").unwrap_or_default();
    let invs = Resource::new(term, admin_invitations);
    let themes = Resource::new(|| (), |_| admin_themes());
    view! {
        <AdminShell active="undangan" title="Pesanan" admin_only=true>
            <div class="adm-head">
                <h1 class="adm-h1">"Pesanan Undangan"</h1>
                <form class="searchbar searchbar--sm adm-search" method="get" action="/admin/undangan">
                    <Icon name="search" />
                    <input name="q" placeholder="Cari tautan, nama, atau nomor WA…" value=term />
                </form>
            </div>
            {move || q.read().get("kelola").filter(|l| l.starts_with("/kelola/")).map(|l| view! {
                <div class="notice notice--info adm-newkey">
                    <b>"Tautan Kelola baru (tampil sekali):"</b>
                    <div class="copyline">
                        <code>{l.clone()}</code>
                        <button type="button" class="btn btn--outline btn--sm" data-copy=l.clone() data-copied="Tautan disalin">"Salin"</button>
                    </div>
                </div>
            })}
            <p class="muted small">"Pesanan yang sudah mengirim bukti transfer tampil paling atas, lalu yang menunggu pembayaran. Cek mutasi rekening, lalu ubah status ke "<b>"Aktif"</b>" — penanda pratinjau di undangan hilang. Tema bisa diganti ke tema custom (privat) yang Anda buat untuk pasangan ini."</p>
            <Suspense fallback=|| view! { <div class="inv-loading"><div class="spinner"></div></div> }>
                {move || {
                    let theme_opts: Vec<(String, String)> = themes.get().and_then(|r| r.ok()).unwrap_or_default()
                        .into_iter()
                        .map(|a| (a.theme.slug.clone(), if a.theme.listed { a.theme.name } else { format!("{} (privat)", a.theme.name) }))
                        .collect();
                    invs.get().map(|r| match r {
                        Err(e) => Either::Left(view! { <p class="notice notice--err">{e.to_string()}</p> }),
                        Ok(list) if list.is_empty() => Either::Left(view! { <p class="empty card">{"Belum ada pesanan yang cocok.".to_string()}</p> }),
                        Ok(list) => Either::Right(view! {
                            <div class="adm-orders">
                                {list.into_iter().map(|i| {
                                    let opts = theme_opts.clone();
                                    let wa = crate::web::fmt::wa_number(&i.contact_phone);
                                    view! {
                                        <form class="card adm-order" class:adm-order--wait=i.status == "menunggu_pembayaran" method="post" action="/admin/undangan/simpan">
                                            <input type="hidden" name="slug" value=i.slug.clone() />
                                            <input type="hidden" name="q" value=term() />
                                            <div class="adm-order__who">
                                                <b>{i.couple.clone()}</b>
                                                <small>
                                                    <a href=format!("/u/{}", i.slug) target="_blank">{format!("/u/{}", i.slug)}</a>
                                                    {format!(" • {} • {} • {} • {}", i.package_name, rupiah(i.total_price), i.payment_method, i.created)}
                                                </small>
                                                {(!wa.is_empty()).then(|| view! {
                                                    <a class="adm-order__wa" href=format!("https://wa.me/{wa}") target="_blank" rel="noopener"><Icon name="chat" />{format!("+{wa}")}</a>
                                                })}
                                                {i.is_demo.then(|| view! { <span class="status">"Demo — tidak bisa diubah"</span> })}
                                                // Bukti transfer dari Kelola — pesanan ini diurutkan paling atas.
                                                {(!i.proof_at.is_empty()).then(|| view! {
                                                    <span class="status status--hadir"><Icon name="check_circle" />{format!("Bukti transfer dikirim {}", i.proof_at)}</span>
                                                })}
                                                {(!i.payment_proof.is_empty()).then(|| view! {
                                                    <a class="adm-order__proof" href=i.payment_proof.clone() target="_blank" rel="noopener">
                                                        <img src=i.payment_proof.clone() alt="Bukti transfer" loading="lazy" />
                                                    </a>
                                                })}
                                                {i.minutes_left.map(|m| view! {
                                                    <span class="status status--tidak"><Icon name="hourglass_top" />{format!("Dihapus otomatis dalam {}", crate::web::model::durasi(m))}</span>
                                                })}
                                            </div>
                                            <select class="input" name="status" disabled=i.is_demo>
                                                {INV_STATUSES.iter().map(|(k, l)| view! { <option value=*k selected=i.status == *k>{*l}</option> }).collect_view()}
                                            </select>
                                            <select class="input" name="theme" disabled=i.is_demo>
                                                {opts.into_iter().map(|(k, l)| {
                                                    let sel = k == i.theme;
                                                    view! { <option value=k selected=sel>{l}</option> }
                                                }).collect_view()}
                                            </select>
                                            <button class="btn btn--primary btn--sm" type="submit" disabled=i.is_demo>"Simpan"</button>
                                            {(!i.is_demo).then(|| view! {
                                                <button class="btn btn--soft btn--xs adm-order__key" type="submit" formaction="/admin/undangan/kunci"
                                                    title="Untuk pemesan yang kehilangan tautan Kelola. Tautan lama langsung tidak berlaku.">
                                                    <Icon name="link" />"Terbitkan ulang tautan Kelola"
                                                </button>
                                            })}
                                        </form>
                                    }
                                }).collect_view()}
                            </div>
                        }),
                    })
                }}
            </Suspense>
        </AdminShell>
    }
}

// ── /admin/konten ──────────────────────────────────────────────────────────

#[component]
pub fn AdminKonten() -> impl IntoView {
    let konten = Resource::new(|| (), |_| get_konten());
    let meta = Resource::new(|| (), |_| admin_konten_meta());
    view! {
        <AdminShell active="konten" title="Konten & Harga">
            <div class="adm-head">
                <h1 class="adm-h1">"Konten & Harga"</h1>
            </div>
            <p class="muted small">"Semua yang diubah di sini langsung tayang di situs. Bagian yang belum pernah disunting memakai isi bawaan."</p>
            <Suspense fallback=|| ()>
                {move || konten.get().and_then(|r| r.ok()).map(|k| {
                    let meta: Vec<(String, String)> = meta.get().and_then(|r| r.ok()).unwrap_or_default();
                    let mut groups: Vec<&'static str> = Vec::new();
                    for sec in SECTIONS {
                        if !groups.contains(&sec.group) {
                            groups.push(sec.group);
                        }
                    }
                    groups.into_iter().map(|g| {
                        let k = k.clone();
                        let meta = meta.clone();
                        view! {
                            <section class="adm-group">
                                <h2 class="adm-sec">{g}</h2>
                                <div class="adm-sections">
                                    {SECTIONS.iter().filter(|s| s.group == g).map(|sec| {
                                        let v = k.section_json(sec.key);
                                        let count = if sec.single { String::new() } else { format!("{} item", v.as_array().map(|a| a.len()).unwrap_or(0)) };
                                        let edited = meta.iter().find(|m| m.0 == sec.key).map(|m| format!("Disunting {}", m.1)).unwrap_or_else(|| "Isi bawaan".into());
                                        view! {
                                            <a class="card adm-section" href=format!("/admin/konten/{}", sec.key)>
                                                <b>{sec.title}</b>
                                                <small>{count}</small>
                                                <small class="muted">{edited}</small>
                                            </a>
                                        }
                                    }).collect_view()}
                                </div>
                            </section>
                        }
                    }).collect_view()
                })}
            </Suspense>
        </AdminShell>
    }
}

#[component]
pub fn AdminKontenEdit() -> impl IntoView {
    let params = use_params_map();
    let key = move || params.read().get("key").unwrap_or_default();
    let konten = Resource::new(|| (), |_| get_konten());
    view! {
        <AdminShell active="konten" title="Sunting Konten">
            <Suspense fallback=|| ()>
                {move || konten.get().and_then(|r| r.ok()).map(|k| {
                    let Some(sec) = section(&key()) else {
                        return view! { <p class="notice notice--err">"Bagian konten tidak dikenal."</p> }.into_any();
                    };
                    let data = k.section_json(sec.key);
                    let items: Vec<serde_json::Value> = if sec.single { vec![data] } else { data.as_array().cloned().unwrap_or_default() };
                    let n = if sec.single { 1 } else { items.len() + 1 };
                    view! {
                        <div class="adm-head">
                            <div>
                                <p class="eyebrow">{sec.group}</p>
                                <h1 class="adm-h1">{sec.title}</h1>
                            </div>
                            <div class="adm-tcard__btns">
                                <a class="btn btn--soft btn--sm" href="/admin/konten"><Icon name="arrow_back" />"Semua bagian"</a>
                                <a class="btn btn--soft btn--sm" href=sec.page target="_blank"><Icon name="visibility" />"Lihat halaman"</a>
                            </div>
                        </div>
                        {(!sec.help.is_empty()).then(|| view! { <p class="notice notice--info">{sec.help}</p> })}
                        // Bagian dengan kolom "ikon | teks": tampilkan ikon yang tersedia
                        // (ikon di luar daftar ini diganti ikon bawaan saat tayang).
                        {sec.fields.iter().any(|f| f.help.contains("ikon")).then(|| view! {
                            <details class="card adm-icons">
                                <summary><Icon name="grid_view" />"Daftar ikon yang bisa dipakai — klik untuk menyalin nama"</summary>
                                <div class="adm-icons__grid">
                                    {crate::web::icons::ICONS.iter().map(|name| view! {
                                        <button type="button" class="adm-icon" data-copy=*name data-copied=format!("Disalin: {name}") title=*name>
                                            <span class="ms" aria-hidden="true">{*name}</span>
                                            <small>{*name}</small>
                                        </button>
                                    }).collect_view()}
                                </div>
                            </details>
                        })}
                        <form class="adm-form" method="post" action="/admin/konten/simpan" enctype="multipart/form-data">
                            <input type="hidden" name="section" value=sec.key />
                            <input type="hidden" name="n" value=n.to_string() />
                            {items.iter().enumerate().map(|(i, item)| {
                                let title = item.get(sec.title_field).and_then(|v| v.as_str()).unwrap_or("").to_string();
                                view! { <ItemFields sec_key=sec.key i=i item=item.clone() title=title open=sec.single || i == 0 /> }
                            }).collect_view()}
                            {(!sec.single).then(|| view! {
                                <ItemFields sec_key=sec.key i=n - 1 item=serde_json::Value::Null title="+ Tambah item baru".to_string() open=false is_new=true />
                            })}
                            <div class="adm-save">
                                <button class="btn btn--primary btn--lg" type="submit"><Icon name="check" />"Simpan & Tayangkan"</button>
                            </div>
                        </form>
                    }.into_any()
                })}
            </Suspense>
        </AdminShell>
    }
}

/// Satu item (atau satu objek tunggal) di editor konten.
#[component]
fn ItemFields(
    sec_key: &'static str,
    i: usize,
    item: serde_json::Value,
    title: String,
    open: bool,
    #[prop(optional)] is_new: bool,
) -> impl IntoView {
    let sec = section(sec_key).expect("bagian ada");
    let get = |k: &str| item.get(k).cloned().unwrap_or(serde_json::Value::Null);
    let fields = sec.fields.iter().map(|fld| {
        let name = input_name(i, fld.key);
        let v = get(fld.key);
        let as_str = v.as_str().unwrap_or("").to_string();
        let control = match fld.kind {
            Kind::Text => view! { <input class="input" name=name maxlength="160" value=as_str /> }.into_any(),
            Kind::Long => view! { <textarea class="input" name=name rows="3" maxlength="1200">{as_str}</textarea> }.into_any(),
            Kind::Money | Kind::Number => {
                let n = v.as_i64().unwrap_or(0);
                let hint = (fld.kind == Kind::Money && n > 0).then(|| rupiah(n));
                view! {
                    <div class="adm-num">
                        <input class="input" name=name type="number" min="0" step="1" value=n.to_string() />
                        {hint.map(|h| view! { <small class="muted">{h}</small> })}
                    </div>
                }
                .into_any()
            }
            Kind::Bool => view! {
                <label class="check"><input type="checkbox" name=name value="1" checked=v.as_bool().unwrap_or(false) />"Ya"</label>
            }
            .into_any(),
            Kind::Lines => {
                let text = v.as_array().map(|a| a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join("\n")).unwrap_or_default();
                view! { <textarea class="input" name=name rows="4">{text}</textarea> }.into_any()
            }
            Kind::Gallery => {
                let urls: Vec<String> = v.as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect()).unwrap_or_default();
                let lines = urls.join("\n");
                let n = urls.len();
                view! {
                    <div class="adm-gal">
                        {(n > 0).then(|| view! {
                            <div class="adm-gal__grid">
                                {urls.into_iter().enumerate().map(|(j, u)| view! {
                                    <label class="adm-gal__item">
                                        <img src=u.clone() alt="" loading="lazy" decoding="async" />
                                        <span class="adm-gal__no">{j + 1}</span>
                                        <span class="adm-gal__del"><input type="checkbox" name=format!("{name}__del__{j}") value=u />"Hapus"</span>
                                    </label>
                                }).collect_view()}
                            </div>
                        })}
                        <label class="adm-gal__up">
                            <Icon name="add_photo_alternate" />
                            <span><b>"Tambah foto"</b><small>{format!("Pilih beberapa sekaligus (JPG/PNG/WebP, maks 5 MB per foto, maks {} foto)", crate::web::konten::GALLERY_MAX)}</small></span>
                            <input type="file" name=format!("{name}__file") accept="image/jpeg,image/png,image/webp" multiple />
                        </label>
                        <details class="adm-gal__urls">
                            <summary>{format!("Urutan & alamat foto ({n})")}</summary>
                            <textarea class="input" name=name.clone() rows="5" placeholder="/img/layanan/… atau https://… — satu per baris">{lines}</textarea>
                            <small class="muted">"Satu foto per baris; pindahkan baris untuk mengubah urutan."</small>
                        </details>
                    </div>
                }
                .into_any()
            }
            Kind::Image => view! {
                <div class="adm-img">
                    {(!as_str.is_empty()).then(|| view! { <img src=as_str.clone() alt="" loading="lazy" decoding="async" /> })}
                    <div class="adm-img__in">
                        <input class="input" name=name.clone() maxlength="500" value=as_str placeholder="/img/layanan/… atau https://…" />
                        <input class="input" type="file" name=format!("{name}__file") accept="image/jpeg,image/png,image/webp" />
                    </div>
                </div>
            }
            .into_any(),
        };
        let help = (!fld.help.is_empty()).then(|| view! { <small class="muted">{fld.help}</small> });
        // Galeri berisi banyak input (centang, berkas) → bukan <label> agar
        // klik di dalamnya tak memicu input pertama.
        if fld.kind == Kind::Gallery {
            view! {
                <div class="field adm-wide">
                    <span class="field__label">{fld.label}</span>
                    {control}
                    {help}
                </div>
            }
            .into_any()
        } else {
            view! {
                <label class="field" class:adm-wide=matches!(fld.kind, Kind::Long | Kind::Lines | Kind::Image)>
                    <span class="field__label">{fld.label}</span>
                    {control}
                    {help}
                </label>
            }
            .into_any()
        }
    }).collect_view();
    view! {
        <details class="card adm-item" class:adm-item--new=is_new open=open>
            <summary>
                <b>{if title.is_empty() { format!("Item {}", i + 1) } else { title }}</b>
                <Icon name="expand_more" />
            </summary>
            <div class="adm-item__grid">{fields}</div>
            {(!sec.single).then(|| view! {
                <div class="adm-item__foot">
                    <label class="field adm-urut">
                        <span class="field__label">"Urutan"</span>
                        <input class="input" type="number" name=input_name(i, "_urut") value=(i * 10).to_string() />
                    </label>
                    {(!is_new).then(|| view! {
                        <label class="check adm-hapus"><input type="checkbox" name=input_name(i, "_hapus") value="1" />"Hapus item ini saat disimpan"</label>
                    })}
                </div>
            })}
        </details>
    }
}

// ── /admin/akun ────────────────────────────────────────────────────────────

#[component]
pub fn AdminAkun() -> impl IntoView {
    let accounts = Resource::new(|| (), |_| admin_accounts());
    view! {
        <AdminShell active="akun" title="Akun" admin_only=true>
            <div class="adm-head"><h1 class="adm-h1">"Akun Panel Admin"</h1></div>
            <div class="adm-roles">
                {ADMIN_ROLES.iter().map(|(_, l, d)| view! { <p><b>{*l}</b>" — "{*d}</p> }).collect_view()}
            </div>
            <Suspense fallback=|| ()>
                {move || accounts.get().and_then(|r| r.ok()).map(|list| {
                    let me = use_context::<AdminUser>().map(|u| u.id).unwrap_or(0);
                    view! {
                        <div class="adm-orders">
                            {list.into_iter().map(|u| {
                                let id = u.id.to_string();
                                view! {
                                    <div class="card adm-acc" class:adm-acc--off=!u.active>
                                        <form method="post" action="/admin/akun/simpan" class="adm-acc__row">
                                            <input type="hidden" name="aksi" value="ubah" />
                                            <input type="hidden" name="id" value=id.clone() />
                                            <div class="adm-order__who">
                                                <b>{u.username.clone()}{(u.id == me).then_some(" (Anda)")}</b>
                                                <small>{if u.last_login.is_empty() { "Belum pernah masuk".to_string() } else { format!("Terakhir masuk {}", u.last_login) }}</small>
                                            </div>
                                            <input class="input" name="name" maxlength="60" value=u.name.clone() placeholder="Nama tampilan" />
                                            <select class="input" name="role">
                                                {ADMIN_ROLES.iter().map(|(k, l, _)| view! { <option value=*k selected=u.role == *k>{*l}</option> }).collect_view()}
                                            </select>
                                            <label class="check"><input type="checkbox" name="active" value="1" checked=u.active />"Aktif"</label>
                                            <button class="btn btn--primary btn--sm" type="submit">"Simpan"</button>
                                        </form>
                                        <details class="adm-acc__pw">
                                            <summary>"Ganti sandi akun ini"</summary>
                                            <form method="post" action="/admin/akun/simpan" class="adm-acc__row">
                                                <input type="hidden" name="aksi" value="sandi" />
                                                <input type="hidden" name="id" value=id />
                                                <span class="pw"><input class="input" type="password" name="password" minlength="8" required placeholder="Sandi baru (min. 8)" autocomplete="new-password" /><button type="button" class="pw__btn" data-pw-toggle="" aria-label="Tampilkan sandi" aria-pressed="false" title="Tampilkan sandi"><Icon name="visibility" class="pw__on" /><Icon name="visibility_off" class="pw__off" /></button></span>
                                                <button class="btn btn--soft btn--sm" type="submit">"Ganti Sandi"</button>
                                            </form>
                                        </details>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }
                })}
            </Suspense>
            <section class="card adm-help">
                <h2><Icon name="person_add" />"Tambah akun"</h2>
                <form method="post" action="/admin/akun/simpan" class="adm-acc__new">
                    <input type="hidden" name="aksi" value="baru" />
                    <label class="field"><span class="field__label">"Username"</span>
                        <input class="input" name="username" required minlength="3" maxlength="32" pattern="[a-z0-9._-]+" placeholder="sekar" autocomplete="off" /></label>
                    <label class="field"><span class="field__label">"Nama tampilan"</span>
                        <input class="input" name="name" maxlength="60" placeholder="Sekar (MUA)" /></label>
                    <label class="field"><span class="field__label">"Peran"</span>
                        <select class="input" name="role">
                            {ADMIN_ROLES.iter().map(|(k, l, _)| view! { <option value=*k selected=*k == "editor">{*l}</option> }).collect_view()}
                        </select></label>
                    <label class="field"><span class="field__label">"Sandi awal (min. 8)"</span>
                        <span class="pw"><input class="input" type="password" name="password" required minlength="8" autocomplete="new-password" /><button type="button" class="pw__btn" data-pw-toggle="" aria-label="Tampilkan sandi" aria-pressed="false" title="Tampilkan sandi"><Icon name="visibility" class="pw__on" /><Icon name="visibility_off" class="pw__off" /></button></span></label>
                    <button class="btn btn--primary" type="submit"><Icon name="add" />"Buat Akun"</button>
                </form>
            </section>
        </AdminShell>
    }
}

// ── /admin/profil ──────────────────────────────────────────────────────────

#[component]
pub fn AdminProfil() -> impl IntoView {
    view! {
        <AdminShell active="profil" title="Profil">
            <section class="card adm-login">
                <Icon name="lock" class="empty-page__icon" />
                <h1>"Ganti Sandi Saya"</h1>
                {move || use_context::<AdminUser>().map(|u| view! { <p class="muted">{format!("Masuk sebagai {} • {}", u.username, role_label(&u.role))}</p> })}
                <form method="post" action="/admin/sandi" class="stack adm-login__form">
                    <label class="field"><span class="field__label">"Sandi lama"</span>
                        <span class="pw"><input class="input" type="password" name="old" required autocomplete="current-password" /><button type="button" class="pw__btn" data-pw-toggle="" aria-label="Tampilkan sandi" aria-pressed="false" title="Tampilkan sandi"><Icon name="visibility" class="pw__on" /><Icon name="visibility_off" class="pw__off" /></button></span></label>
                    <label class="field"><span class="field__label">"Sandi baru (min. 8)"</span>
                        <span class="pw"><input class="input" type="password" name="password" required minlength="8" autocomplete="new-password" /><button type="button" class="pw__btn" data-pw-toggle="" aria-label="Tampilkan sandi" aria-pressed="false" title="Tampilkan sandi"><Icon name="visibility" class="pw__on" /><Icon name="visibility_off" class="pw__off" /></button></span></label>
                    <label class="field"><span class="field__label">"Ulangi sandi baru"</span>
                        <span class="pw"><input class="input" type="password" name="password2" required minlength="8" autocomplete="new-password" /><button type="button" class="pw__btn" data-pw-toggle="" aria-label="Tampilkan sandi" aria-pressed="false" title="Tampilkan sandi"><Icon name="visibility" class="pw__on" /><Icon name="visibility_off" class="pw__off" /></button></span></label>
                    <button class="btn btn--primary btn--block" type="submit">"Simpan Sandi"</button>
                </form>
            </section>
        </AdminShell>
    }
}

// ── /admin/tema/:slug/ornamen ──────────────────────────────────────────────

type OGet<T> = fn(&Ornament) -> T;
type OSet<T> = fn(&mut Ornament, T);

/// Sinyal bersama editor ornamen: fokus (sorot di pratinjau), bagian yang
/// dipratinjau, dan "sedang memutar animasi".
#[derive(Clone, Copy)]
struct OrnUi {
    focus: RwSignal<i64>,
    bag: RwSignal<String>,
    playing: RwSignal<bool>,
}

fn o_range(o: RwSignal<Ornament>, ui: OrnUi, name: &'static str, label: &'static str, (min, max, step): (i32, i32, i32), unit: &'static str, get: OGet<i32>, set: OSet<i32>) -> impl IntoView {
    let init = get(&o.get_untracked());
    view! {
        <label class="field adm-range">
            <span class="field__label">{label}<b>{move || format!("{}{unit}", o.with(get))}</b></span>
            <input type="range" name=name min=min.to_string() max=max.to_string() step=step.to_string() value=init.to_string()
                on:input=move |e| { let v = event_target_value(&e).parse().unwrap_or(init); ui.playing.set(false); o.update(|x| set(x, v)); } />
        </label>
    }
}

fn o_select(o: RwSignal<Ornament>, ui: OrnUi, name: &'static str, label: &'static str, opts: Vec<(&'static str, &'static str)>, get: OGet<String>, set: OSet<String>) -> impl IntoView {
    let init = get(&o.get_untracked());
    view! {
        <label class="field">
            <span class="field__label">{label}</span>
            <select class="input" name=name
                on:change=move |e| { let v = event_target_value(&e); ui.playing.set(false); o.update(|x| set(x, v)); }>
                {opts.into_iter().map(|(k, l)| view! { <option value=k selected=init == k>{l}</option> }).collect_view()}
            </select>
        </label>
    }
}

fn o_check(o: RwSignal<Ornament>, ui: OrnUi, name: &'static str, label: &'static str, get: OGet<bool>, set: OSet<bool>) -> impl IntoView {
    view! {
        <label class="check">
            <input type="checkbox" name=name value="1" checked=get(&o.get_untracked())
                on:change=move |e| { let v = event_target_checked(&e); ui.playing.set(false); o.update(|x| set(x, v)); } />
            {label}
        </label>
    }
}

#[component]
pub fn AdminOrnaments() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.read().get("slug").unwrap_or_default();
    // Katalog ringan (jumlah ornamen per tema, untuk "salin dari") + satu tema lengkap.
    let themes = Resource::new(|| (), |_| admin_themes());
    let full = Resource::new(slug, get_theme);
    view! {
        <AdminShell active="tema" title="Ornamen Tema">
            <Suspense fallback=|| ()>
                {move || themes.get().zip(full.get()).map(|(list, full)| {
                    let list = list.unwrap_or_default();
                    match full.ok().flatten() {
                        None => Either::Left(view! { <p class="notice notice--err">"Tema tidak ditemukan."</p> }),
                        Some(t) => {
                            let others: Vec<(String, String, usize)> = list
                                .iter()
                                .filter(|a| a.theme.slug != t.slug && a.ornaments > 0)
                                .map(|a| (a.theme.slug.clone(), a.theme.name.clone(), a.ornaments))
                                .collect();
                            Either::Right(view! { <OrnEditor t=t others=others /> })
                        }
                    }
                })}
            </Suspense>
        </AdminShell>
    }
}

#[component]
fn OrnEditor(t: ThemeInfo, others: Vec<(String, String, usize)>) -> impl IntoView {
    let slug = t.slug.clone();
    let mut t = t;
    t.ornaments.sort_by_key(|o| (orn::bagian_index(&o.bagian), o.urutan, o.id));
    let items: Vec<RwSignal<Ornament>> = t.ornaments.iter().cloned().map(RwSignal::new).collect();
    let fresh = RwSignal::new(Ornament { theme: slug.clone(), ..Default::default() });
    let ui = OrnUi {
        focus: RwSignal::new(-1),
        bag: RwSignal::new(t.ornaments.first().map(|o| o.bagian.clone()).unwrap_or_else(|| "sampul".into())),
        playing: RwSignal::new(false),
    };
    let replay = RwSignal::new(0u32);
    {
        let items = items.clone();
        // Pratinjau = semua ornamen tersimpan (draf) + ornamen baru bila gambarnya sudah diisi.
        provide_context(OrnLayer(Signal::derive(move || {
            let mut v: Vec<Ornament> = items.iter().map(|s| s.get()).collect();
            let n = fresh.get();
            if !n.img.is_empty() {
                v.push(n);
            }
            v
        })));
    }
    let count = t.ornaments.len();
    let focus_css = move || {
        let id = ui.focus.get();
        if id < 0 { String::new() } else { format!(".adm-orn-prev [data-orn-id=\"{id}\"] .orn__img{{outline:2px dashed var(--gold);outline-offset:2px}}") }
    };
    let play = move |_| {
        ui.playing.set(true);
        replay.update(|n| *n += 1);
    };

    view! {
        <div class="adm-head">
            <h1 class="adm-h1">{format!("Ornamen & Gerak: {}", t.name)}</h1>
            <a class="btn btn--soft btn--sm" href=format!("/admin/tema/{slug}")><Icon name="arrow_back" />"Kembali ke tema"</a>
        </div>
        <div class="adm-anim-help">
            <p>"Hiasan bergambar yang ditempel di tiap bagian undangan — mis. rangkaian bunga di sudut sampul yang mekar lalu bergoyang pelan. Atur posisi, ukuran, animasi muncul, dan gerak diamnya; pratinjau di samping langsung mengikuti."</p>
            <p class="muted small">"Gambar terbaik: PNG/WebP transparan ±800px (unggahan otomatis dikompres ke WebP). Gerak judul/foto & Ken Burns ada di halaman tema → bagian Animasi."</p>
        </div>
        <div class="adm-edit">
            <div class="adm-form adm-orns">
                {items.iter().copied().enumerate().map(|(i, o)| view! { <OrnForm o=o ui=ui no=i + 1 /> }).collect_view()}
                <details class="card adm-orn adm-orn--new" open=count == 0
                    on:toggle=move |_| ui.focus.set(0)>
                    <summary><Icon name="add" />"Tambah ornamen"</summary>
                    <OrnFields o=fresh ui=ui />
                </details>
                {(!others.is_empty()).then(|| {
                    let s2 = slug.clone();
                    view! {
                        <details class="card adm-orn">
                            <summary><Icon name="content_copy" />"Salin ornamen dari tema lain"</summary>
                            <form class="adm-orn__copy" method="post" action="/admin/ornamen/salin">
                                <input type="hidden" name="theme" value=s2 />
                                <select class="input" name="dari">
                                    {others.into_iter().map(|(s, n, c)| view! { <option value=s>{format!("{n} ({c} ornamen)")}</option> }).collect_view()}
                                </select>
                                <button class="btn btn--soft" type="submit"><Icon name="content_copy" />"Salin ke tema ini"</button>
                            </form>
                            <p class="muted small">"Ornamen disalin sebagai tambahan (yang sudah ada tidak dihapus)."</p>
                        </details>
                    }
                })}
            </div>

            <aside class="adm-preview">
                <p class="eyebrow">"Pratinjau langsung"</p>
                <style inner_html=focus_css></style>
                <div class="adm-orn-bags" role="tablist">
                    {orn::BAGIAN.iter().map(|(k, _, short)| view! {
                        <button type="button" class="chip" class:is-on=move || ui.bag.get() == *k
                            on:click=move |_| { ui.bag.set(k.to_string()); ui.playing.set(false); }>
                            {*short}
                        </button>
                    }).collect_view()}
                </div>
                <button type="button" class="btn btn--soft btn--sm adm-replay" on:click=play>
                    <Icon name="play_arrow" />"Putar animasi"
                </button>
                <div class=format!("adm-preview__inv adm-orn-prev th-{slug}")>
                    <For each=move || vec![(replay.get(), ui.bag.get())] key=|k| k.clone() children=move |(_, bag)| view! {
                        <div class="inv inv--embed adm-prev is-open" class:orn-still=move || !ui.playing.get()>
                            <div class="inv__glow" aria-hidden="true"></div>
                            <div class="inv__main">
                                <OrnSample bag=bag />
                            </div>
                        </div>
                    } />
                </div>
                <p class="muted small">"Garis putus-putus = ornamen yang sedang disunting. Gambar unggahan tampil setelah disimpan; URL gambar langsung terlihat."</p>
            </aside>
        </div>
    }
}

/// Contoh isi satu bagian untuk pratinjau (markup sama dengan undangan asli).
#[component]
fn OrnSample(bag: String) -> impl IntoView {
    match bag.as_str() {
        "mempelai" => view! {
            <section class="section orn-host">
                <Ornamen bagian="mempelai" />
                <p class="eyebrow eyebrow--center">"Kedua Mempelai"</p>
                <h2 class="section__title">"Bride & Groom"</h2>
                <div class="couple">
                    {["Yona", "Doni"].into_iter().map(|n| view! {
                        <article class="person card">
                            <div class="arch-photo arch-photo--person">
                                <span class="arch-photo__clip"><span class="person__initial">{n[..1].to_string()}</span></span>
                                <span class="arch-photo__frame" aria-hidden="true"></span>
                            </div>
                            <div class="person__body"><p class="script person__nick">{n}</p><p class="person__parents">"Putri/Putra dari Bapak & Ibu"</p></div>
                        </article>
                    }).collect_view()}
                </div>
            </section>
        }.into_any(),
        "kisah" => view! {
            <section class="section orn-host">
                <Ornamen bagian="kisah" />
                <p class="eyebrow eyebrow--center">"Perjalanan Kami"</p>
                <h2 class="section__title">"Love Story"</h2>
                <ol class="story card">
                    {[("2019", "Pertama Bertemu"), ("2023", "Lamaran"), ("2026", "Hari Bahagia")].into_iter().map(|(y, h)| view! {
                        <li class="story__item">
                            <span class="story__dot" aria-hidden="true"><Icon name="favorite" /></span>
                            <div><span class="story__year">{y}</span><h3>{h}</h3><p>"Cerita singkat perjalanan kami berdua."</p></div>
                        </li>
                    }).collect_view()}
                </ol>
            </section>
        }.into_any(),
        "galeri" => view! {
            <section class="section orn-host">
                <Ornamen bagian="galeri" />
                <p class="eyebrow eyebrow--center">"Momen Kebahagiaan"</p>
                <h2 class="section__title">"Our Moments"</h2>
                <div class="gallery gallery--masonry">
                    {["/img/layanan/mua-prewed.jpg", "/img/layanan/dekor-galeri-altar.jpg", "/img/layanan/mua-sesudah.jpg", "/img/layanan/dekor-galeri-meja.jpg"].into_iter().map(|src| view! {
                        <span class="gallery__item"><img src=src alt="" loading="lazy" /></span>
                    }).collect_view()}
                </div>
            </section>
        }.into_any(),
        "acara" => view! {
            <section class="intro orn-host">
                <Ornamen bagian="acara" />
                <span class="intro__icon"><Icon name="local_florist" /></span>
                <p class="eyebrow eyebrow--gold">"Walimatul 'Ursy"</p>
                <h1 class="section__title">"Wedding Event"</h1>
                <p class="intro__text">"Dengan memohon rahmat dan ridho Allah SWT, kami mengundang Anda untuk merayakan ikatan suci kami:"</p>
            </section>
            <div class="event card">
                <span class="event__tag">"Akad Nikah"</span>
                <h3 class="event__title">"Masjid Agung"</h3>
                <p class="event__meta"><Icon name="schedule" />"08.00 – 10.00 WIB"</p>
            </div>
        }.into_any(),
        "rsvp" => view! {
            <section class="rsvp-hero card card--soft orn-host">
                <Ornamen bagian="rsvp" />
                <span class="intro__icon"><Icon name="favorite" /></span>
                <p class="eyebrow eyebrow--gold">"Buku Tamu Digital"</p>
                <h1 class="section__title">"Wedding Wishes"</h1>
                <p class="intro__text">"Kehadiran dan doa restu Anda merupakan kado terindah bagi kebahagiaan kami berdua."</p>
            </section>
        }.into_any(),
        _ => view! {
            <section class="hero cover orn-host">
                <Ornamen bagian="sampul" />
                <p class="script cover__eyebrow">"The Wedding Of"</p>
                <div class="arch-photo arch-photo--cover">
                    <span class="arch-photo__clip"><img src="/img/layanan/mua-sesudah.jpg" alt="" /></span>
                    <span class="arch-photo__frame" aria-hidden="true"></span>
                </div>
                <h1 class="cover__names">"Yona & Doni"</h1>
                <p class="hero__date"><Icon name="local_florist" />"Sabtu, 24 Oktober 2026"<Icon name="local_florist" /></p>
            </section>
        }.into_any(),
    }
}

/// Kartu satu ornamen tersimpan: ringkasan + form sunting + hapus.
#[component]
fn OrnForm(o: RwSignal<Ornament>, ui: OrnUi, no: usize) -> impl IntoView {
    let first = o.get_untracked();
    let (id, theme) = (first.id, first.theme.clone());
    view! {
        <article class="card adm-orn" id=format!("orn-{id}")>
            <div class="adm-orn__head">
                <span class="adm-banner__no">{no}</span>
                <span class="adm-orn__thumb">{move || { let src = o.with(|x| x.img.clone()); view! { <img src=src alt="" loading="lazy" /> } }}</span>
                <span class="adm-orn__sum">
                    <b>{move || o.with(|x| x.bagian_label().to_string())}</b>
                    <small>{move || o.with(|x| {
                        let p = orn::POSISI.iter().find(|p| p.0 == x.posisi).map(|p| p.1).unwrap_or("");
                        let m = orn::MASUK.iter().find(|m| m.0 == x.masuk).map(|m| m.1).unwrap_or("");
                        format!("{p} • {}% • {m}{}", x.lebar, if x.depan { " • depan" } else { "" })
                    })}</small>
                </span>
            </div>
            <details on:toggle=move |_| { ui.focus.set(id); ui.bag.set(o.with_untracked(|x| x.bagian.clone())); }>
                <summary>"Sunting"</summary>
                <OrnFields o=o ui=ui />
                <form method="post" action="/admin/ornamen/hapus" class="adm-banner__del">
                    <input type="hidden" name="theme" value=theme />
                    <input type="hidden" name="id" value=id.to_string() />
                    <button class="btn btn--sm adm-btn-danger" type="submit"><Icon name="delete" />"Hapus ornamen ini"</button>
                </form>
            </details>
        </article>
    }
}

#[component]
fn OrnFields(o: RwSignal<Ornament>, ui: OrnUi) -> impl IntoView {
    let first = o.get_untracked();
    let (id, theme) = (first.id, first.theme.clone());
    let opts = |l: &'static [(&'static str, &'static str)]| l.to_vec();
    let posisi: Vec<(&'static str, &'static str)> = orn::POSISI.iter().map(|p| (p.0, p.1)).collect();
    let focus = move || {
        ui.focus.set(id);
        let b = o.with_untracked(|x| x.bagian.clone());
        if ui.bag.get_untracked() != b {
            ui.bag.set(b);
        }
    };
    view! {
        <form class="adm-orn__form" method="post" action="/admin/ornamen/simpan" enctype="multipart/form-data"
            on:focusin=move |_| focus() on:input=move |_| focus()>
            <input type="hidden" name="theme" value=theme />
            <input type="hidden" name="id" value=id.to_string() />
            <div class="field adm-wide">
                <span class="field__label">"Gambar ornamen"</span>
                <div class="adm-img">
                    {move || { let v = o.with(|x| x.img.clone()); (!v.is_empty()).then(|| view! { <img src=v alt="" loading="lazy" decoding="async" /> }) }}
                    <div class="adm-img__in">
                        <input class="input" name="img" maxlength="500" value=first.img.clone() placeholder="/img/tema/ornamen/… atau https://…"
                            on:input=move |e| { let v = event_target_value(&e); o.update(|x| x.img = v); } />
                        <input class="input" type="file" name="img_file" accept="image/png,image/webp,image/jpeg" />
                    </div>
                </div>
            </div>
            <div class="field-row">
                {o_select(o, ui, "bagian", "Bagian", orn::BAGIAN.iter().map(|b| (b.0, b.1)).collect(), |x| x.bagian.clone(), |x, v| x.bagian = v)}
                {o_select(o, ui, "posisi", "Posisi", posisi, |x| x.posisi.clone(), |x, v| x.posisi = v)}
            </div>
            <div class="field-row">
                {o_range(o, ui, "x", "Geser mendatar", (-100, 100, 1), "%", |x| x.x, |x, v| x.x = v)}
                {o_range(o, ui, "y", "Geser tegak", (-100, 100, 1), "%", |x| x.y, |x, v| x.y = v)}
            </div>
            <div class="field-row">
                {o_range(o, ui, "lebar", "Lebar", (5, 100, 1), "%", |x| x.lebar, |x, v| x.lebar = v)}
                {o_range(o, ui, "rotasi", "Putar", (-180, 180, 1), "°", |x| x.rotasi, |x, v| x.rotasi = v)}
            </div>
            <div class="field-row">
                {o_range(o, ui, "opasitas", "Kepekatan", (10, 100, 5), "%", |x| x.opasitas, |x, v| x.opasitas = v)}
                <label class="field"><span class="field__label">"Urutan"</span>
                    <input class="input" type="number" name="urutan" value=first.urutan.to_string() step="1" min="0" max="9999" /></label>
            </div>
            <div class="adm-checks">
                {o_check(o, ui, "cermin", "Cerminkan (balik kiri-kanan)", |x| x.cermin, |x, v| x.cermin = v)}
                {o_check(o, ui, "depan", "Di depan isi", |x| x.depan, |x, v| x.depan = v)}
                {o_check(o, ui, "hp", "Tampil di HP", |x| x.hp, |x, v| x.hp = v)}
            </div>
            <div class="field-row">
                {o_select(o, ui, "masuk", "Animasi muncul", opts(orn::MASUK), |x| x.masuk.clone(), |x, v| x.masuk = v)}
                {o_select(o, ui, "gerak", "Gerak setelah muncul", opts(orn::GERAK), |x| x.gerak.clone(), |x, v| x.gerak = v)}
            </div>
            <div class="field-row field-row--3">
                {o_range(o, ui, "jeda", "Jeda", (0, 3000, 50), " ms", |x| x.jeda, |x, v| x.jeda = v)}
                {o_range(o, ui, "durasi", "Lama muncul", (200, 4000, 100), " ms", |x| x.durasi, |x, v| x.durasi = v)}
                {o_range(o, ui, "kecepatan", "Satu putaran gerak", (2, 30, 1), " dtk", |x| x.kecepatan, |x, v| x.kecepatan = v)}
            </div>
            <div class="adm-banner__save">
                <button class="btn btn--primary" type="submit"><Icon name="check" />{if id == 0 { "Tambahkan Ornamen" } else { "Simpan Ornamen" }}</button>
            </div>
        </form>
    }
}

// ── Pustaka lagu (migrasi 027) ─────────────────────────────────────────────

/// Satu-satunya sumber musik latar undangan: pengantin memilih dari daftar ini
/// di /buat; permintaan lagu baru datang lewat WhatsApp admin.
#[component]
pub fn AdminLagu() -> impl IntoView {
    let list = Resource::new(|| (), |_| admin_songs());
    view! {
        <AdminShell active="lagu" title="Pustaka Lagu">
            <div class="adm-head">
                <h1 class="adm-h1">"Pustaka Lagu"</h1>
                <a class="btn btn--soft btn--sm" href="/buat#musik" target="_blank"><Icon name="visibility" />"Lihat di formulir pesan"</a>
            </div>
            <div class="adm-anim-help">
                <p>"Musik latar undangan HANYA dari daftar ini — pengantin tidak bisa mengunggah lagu sendiri. Lagu yang diminta lewat WhatsApp ditambahkan di sini."</p>
                <p class="muted small">"MP3/M4A/OGG maks 6 MB (±128 kbps). Nonaktifkan untuk menyembunyikan dari pemesan baru; undangan yang sudah memakainya tetap berbunyi."</p>
            </div>
            <Suspense fallback=|| ()>
                {move || list.get().map(|r| match r {
                    Err(e) => view! { <p class="notice notice--err">{crate::web::components::err_msg(&e)}</p> }.into_any(),
                    Ok(items) => {
                        let n = items.len();
                        view! {
                            <div class="adm-songs">
                                {items.into_iter().enumerate().map(|(i, s)| view! { <SongRow s=s pos=i total=n /> }).collect_view()}
                                <details class="card adm-song adm-song--new" open=n == 0>
                                    <summary><Icon name="add" />"Tambah lagu"</summary>
                                    <SongFields s=crate::web::model::Song { aktif: true, ..Default::default() } />
                                </details>
                            </div>
                            <audio id="bgm" preload="none"></audio>
                        }
                        .into_any()
                    }
                })}
            </Suspense>
        </AdminShell>
    }
}

#[component]
fn SongRow(s: crate::web::model::Song, pos: usize, total: usize) -> impl IntoView {
    let id = s.id.to_string();
    view! {
        <article class="card adm-song" id=format!("lagu-{}", s.id) class:is-off=!s.aktif>
            <div class="adm-song__head">
                <button type="button" class="icon-btn song__play" data-song=s.url.clone() data-title=s.title.clone() aria-label="Dengarkan">
                    <Icon name="play_arrow" class="when-idle" /><Icon name="pause" class="when-playing" />
                </button>
                <span class="adm-song__meta">
                    <b>{s.title.clone()}</b>
                    <small>{s.meta()}</small>
                </span>
                {(!s.tag.is_empty()).then(|| view! { <span class="chip chip--gold chip--xs">{s.tag.clone()}</span> })}
                <span class=if s.aktif { "status status--hadir" } else { "status" }>{if s.aktif { "Aktif" } else { "Nonaktif" }}</span>
                <span class="adm-banner__move">
                    <form method="post" action="/admin/lagu/urut">
                        <input type="hidden" name="id" value=id.clone() /><input type="hidden" name="arah" value="naik" />
                        <button class="icon-btn" type="submit" disabled=pos == 0 aria-label="Naikkan urutan"><Icon name="arrow_upward" /></button>
                    </form>
                    <form method="post" action="/admin/lagu/urut">
                        <input type="hidden" name="id" value=id.clone() /><input type="hidden" name="arah" value="turun" />
                        <button class="icon-btn" type="submit" disabled=pos + 1 == total aria-label="Turunkan urutan"><Icon name="arrow_downward" /></button>
                    </form>
                </span>
            </div>
            <details>
                <summary>"Sunting"</summary>
                <SongFields s=s.clone() />
                <form method="post" action="/admin/lagu/hapus" class="adm-banner__del">
                    <input type="hidden" name="id" value=id />
                    <button class="btn btn--sm adm-btn-danger" type="submit"><Icon name="delete" />"Hapus dari pustaka"</button>
                </form>
            </details>
        </article>
    }
}

#[component]
fn SongFields(s: crate::web::model::Song) -> impl IntoView {
    let new = s.id == 0;
    view! {
        <form class="adm-banner__form" method="post" action="/admin/lagu/simpan" enctype="multipart/form-data">
            <input type="hidden" name="id" value=s.id.to_string() />
            <div class="field-row">
                <label class="field"><span class="field__label">"Judul"</span>
                    <input class="input" name="title" required maxlength="120" value=s.title.clone() placeholder="Teman Hidup" /></label>
                <label class="field"><span class="field__label">"Penyanyi / keterangan"</span>
                    <input class="input" name="artist" maxlength="120" value=s.artist.clone() placeholder="Tulus" /></label>
            </div>
            <div class="field-row">
                <label class="field"><span class="field__label">"Durasi (opsional)"</span>
                    <input class="input" name="duration" maxlength="8" value=s.duration.clone() placeholder="03:42" /></label>
                <label class="field"><span class="field__label">"Label (opsional)"</span>
                    <input class="input" name="tag" maxlength="30" value=s.tag.clone() placeholder="Terpopuler / Tradisional" /></label>
            </div>
            <div class="field adm-wide">
                <span class="field__label">{if new { "Berkas lagu (wajib)" } else { "Ganti berkas lagu (opsional)" }}</span>
                <input class="input" type="file" name="file" accept="audio/mpeg,audio/mp4,audio/x-m4a,audio/ogg" />
                <input class="input" name="url" maxlength="500" value=s.url.clone() placeholder="…atau alamat https://…/lagu.mp3" />
                <small class="muted">"Unggahan masuk RustFS musik/pustaka/ dan menggantikan alamat di atas."</small>
            </div>
            <label class="check"><input type="checkbox" name="aktif" value="1" checked=s.aktif /><span>"Aktif — tampil di pilihan pemesan"</span></label>
            <button class="btn btn--primary btn--sm" type="submit"><Icon name="cloud_upload" />{if new { "Tambah ke pustaka" } else { "Simpan" }}</button>
        </form>
    }
}

// ── Moderasi story tamu (semua undangan) ───────────────────────────────────

#[component]
pub fn AdminStory() -> impl IntoView {
    let del = ServerAction::<AdminDeleteStory>::new();
    let q = RwSignal::new(String::new());
    let page = RwSignal::new(1i64);
    let list = Resource::new(move || (q.get(), page.get(), del.version().get()), |(q, page, _)| admin_stories(q, page));
    view! {
        <AdminShell active="story" title="Story Tamu">
            <div class="adm-head">
                <h1 class="adm-h1">"Story Tamu"</h1>
            </div>
            <div class="adm-anim-help">
                <p>"Semua story foto tamu dari seluruh undangan, terbaru dulu. Hapus = permanen (baris & foto di RustFS)."</p>
                <p class="muted small">"Pembuat story & pengelola undangan (Kelola) juga bisa menghapus story."</p>
            </div>
            <input class="input" type="search" placeholder="Cari slug undangan / nama pengirim…"
                prop:value=move || q.get()
                on:input=move |ev| { q.set(event_target_value(&ev)); page.set(1); } />
            {move || del.value().get().and_then(|r| r.err()).map(|e| view! { <p class="notice notice--err">{crate::web::components::err_msg(&e)}</p> })}
            <Transition fallback=|| ()>
                {move || list.get().map(|r| match r {
                    Err(e) => view! { <p class="notice notice--err">{crate::web::components::err_msg(&e)}</p> }.into_any(),
                    Ok(pg) => {
                        let (cur, pages, total) = (pg.page, pg.pages, pg.total);
                        view! {
                            <div class="story-mod__grid adm-stories">
                                {pg.items.into_iter().map(|s| {
                                    let id = s.id;
                                    view! {
                                        <figure class="story-mod__item">
                                            <a href=format!("/u/{}/story", s.slug) target="_blank" rel="noopener">
                                                <img class=format!("sf-{}", s.filter) src=s.photo alt="" loading="lazy" decoding="async" />
                                            </a>
                                            <figcaption>
                                                <b>{s.name}</b>
                                                <small class="muted">{s.phone}</small>
                                                <small>{format!("{} • {}", s.couple, s.ago)}</small>
                                            </figcaption>
                                            <button type="button" class="icon-btn icon-btn--sm story-mod__del" aria-label="Hapus story permanen"
                                                disabled=move || del.pending().get()
                                                on:click=move |_| { if confirm_del() { del.dispatch(AdminDeleteStory { id }); } }>
                                                <Icon name="delete" />
                                            </button>
                                        </figure>
                                    }
                                }).collect_view()}
                            </div>
                            {(total > 0).then(|| view! { <crate::web::components::StoryPager page=page cur=cur pages=pages total=total per=crate::web::model::STORY_PER_PAGE /> })}
                        }
                        .into_any()
                    }
                })}
            </Transition>
        </AdminShell>
    }
}

fn confirm_del() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        return web_sys::window().and_then(|w| w.confirm_with_message("Hapus story ini secara permanen? Foto ikut terhapus.").ok()).unwrap_or(false);
    }
    #[allow(unreachable_code)]
    false
}
