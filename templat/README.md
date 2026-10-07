# Tema templat — HTML + CSS di database

Sejak migrasi `029_tema_templat.sql` sebuah tema bisa membawa **struktur HTML
sendiri**, bukan hanya warna/ornamen di atas komponen Leptos yang sama.

## Kenapa dua lapis (templat ≠ tema)

Hasil bedah everlove (Okt 2026, ±47 tema "3D Motion"): semua tema mereka
memakai **satu templat Elementor induk yang di-clone** — ID bagian, urutan,
dan koreografi geraknya identik. Yang membuat tiap tema terasa beda adalah
**lukisan ilustrasi, video pembuka/latar, warna, dan font**.

| Lapis | Tabel | Isi | Dibuat oleh |
|---|---|---|---|
| Templat | `theme_templates` | HTML (Jinja) + CSS + daftar aset bawaan + font | developer / admin mahir |
| Tema | `themes.template`, `template_assets`, `template_css` | pilih templat + timpa aset & variabel CSS | admin (tanpa kode) |

Jadi 10 tema Jawa cukup 1 templat + 10 set aset/warna; tema yang butuh
susunan berbeda (mis. tema "surat kabar", "tiket pesawat") mendapat templat
sendiri.

## Alur render

```
GET /u/{slug}  ──► templat::serve (middleware)
                   tema undangan punya `template`? ── tidak ──► Leptos (seperti dulu)
                   │ ya
                   ▼
                   minijinja (autoescape HTML) + data undangan
                   + <style> CSS templat + CSS tema
                   + /tata.css (pustaka gerak) + /tata.js (mesin, ber-nonce)
```

Tab `/u/{slug}/story` dan `/kelola` tetap Leptos.

## Gerak — semuanya DEKLARATIF

Templat **tidak boleh** membawa `<script>` (ditolak validator; CSP juga
menolak skrip tanpa nonce). Gerak ditulis sebagai atribut, dijalankan satu
mesin `templat/tata.js`:

| Atribut | Arti |
|---|---|
| `data-a="zoomIn"` `data-ad="400"` `data-at="1250"` | animasi **masuk sekali** saat terlihat (jeda / durasi ms). Nama: fadeIn, fadeInUp, fadeInDown, fadeInLeft, fadeInRight, zoomIn, zoomOut, rotateIn, rotateInDownLeft, rotateInDownRight, rotateInUpLeft, rotateInUpRight, slideInUp, flipInX, bounceIn |
| `data-s="atas"` | gerak **gulir yang diulang** (aktif saat top < tinggi layar − 150 px, lepas lagi bila digulir balik — persis everlove): atas, bawah, kiri, kanan, zoom-in, zoom-out, putar |
| `data-idle="goyang"` | gerak diam berulang: ayun, goyang, melayang, denyut, putar, kedip (`--poros` = titik putar) |
| `data-gate` | sampul/gerbang; konten `data-a` di luar gerbang menunggu gerbang dibuka |
| `data-pre` | bagian di luar gerbang yang animasinya jalan sejak awal (mis. panel kiri desktop) |
| `data-open` | tombol buka (musik mulai, video pembuka diputar) |
| `video[data-open-video]` | video pembuka; gerbang selesai saat video **habis** (bukan timer) |
| `video[data-bg-video] data-loop-from="2.6"` | video latar; setelah putaran pertama mengulang dari detik 2,6 |
| `[data-slides="5000"]` | anak `<img>` silang-pudar + Ken Burns |
| `[data-countdown="{{ countdown_ms }}"]` + `[data-cd=d/h/m/s]` | hitung mundur |
| `[data-copy="…"]` (+ `[data-copy-label]`) | salin ke papan klip |
| `audio[data-music]` + `[data-music-toggle]` | musik latar |
| `form[data-rsvp]` + `[data-rsvp-msg]` + `[data-wishes]` + `<template id="wish-tpl">` (`[data-f=name/status/message/ago]`) | RSVP & ucapan tanpa muat ulang (POST `{{ rsvp_action }}`; tanpa JS tetap jalan) |
| `[data-zoom]` | klik → foto layar penuh |

Kelas status di `<html>`: `t-gated`, `t-opening`, `t-video-end`, `t-open`,
`t-playing`, `t-lite` (hemat data / RAM < 4 GB), `t-pv` (pratinjau katalog).
Gerak berat (pintu 3D, pemandangan, burung) = **video/APNG**, bukan CSS.

Tips: elemen yang perlu dicerminkan sekaligus dianimasikan — pakai properti
`scale: -1 1` (bukan `transform`) atau bungkus: luar `data-a`, dalam `data-idle`.

## Data yang tersedia di templat

`tamu`, `guest` (code, vip, table_no), `couple`, `bride`/`groom` (first, nick,
name, degree, parents, ig, photo, initial), `date_label`, `date_num`,
`first`/`events[]` (title, day_name, day, month_year, date_label, time_label,
venue, address, maps, sessions), `countdown_ms`, `calendar_url`,
`quote` (text, source), `love_story[]` (year, title, text, img), `gallery[]`,
`cover_photo`, `video_url`, `banks[]` (bank, number, holder), `gift_address`,
`family_name`, `dress_code`, `dress_colors[]` (name, hex), `live_url`,
`music` (url, label), `wishes[]` (name, status, message, ago, initials),
`wish_total`, `sessions[]`, `rsvp_action`, `guest_code`, `prefill_name`,
`story_url`, `is_demo`, `preview`, `brand`, `slug`, `qs`, `a` (aset).

Filter tambahan: `|foto` (URL tanpa `#pos=…`), `|pos` (gaya object-position
dari `#pos=`), `|rupiah`, `|inisial`.

## Aturan keamanan (validator `server/templat.rs`)

- HTML: tanpa `<script>`, `<iframe>`, `<object>`, `<embed>`, `<base>`,
  `<meta>`, `<link>`, `srcdoc`, `javascript:`, atribut `on…=`; maks 200 KB.
- CSS: tanpa `<`, `@import`, `expression(`, `javascript:`; `url()` hanya
  `/lokal`, `data:image/`, atau asal RustFS/SITE_URL sendiri; maks 120 KB.
- Semua data undangan/tamu di-escape otomatis.

## Templat bawaan

Berkas `templat/<slug>/{templat.html, gaya.css, meta.json}` di-embed ke biner
dan diisikan ke DB saat server start. Baris yang **belum disunting admin**
ikut diperbarui bila berkasnya berubah; yang sudah disunting (`edited`) tidak
pernah ditimpa — tombol "Kembalikan ke bawaan" di /admin/templat
mengembalikannya.

- `kusuma` — Jawa 3D Motion (tema `kusuma-jawi`): sampul naik → pintu gebyok
  (video) → latar video; susunan & koreografi meniru undangan Jawa everlove
  (m07) dengan aset sendiri.
