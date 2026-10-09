# Generator RUPA (bentuk tiap bagian undangan)

Membagikan varian struktur (`src/web/rupa.rs`, CSS di `style/rupa/`) ke semua
tema tayang dan menulis **`migration/033_rupa.sql`** (tema lama) serta
**`migration/035_rupa_suku.sql`** (tema suku dari 034).

```
python3 scripts/rupa/build.py .
```

## Aturan yang dijaga

| Aturan | Nilai |
|---|---|
| Beda bagian antar-pasangan tema (dari 8) | ≥ 5 |
| Beda bagian antar-tema SERUMPUN (motif & warna mirip) | ≥ 6 |
| Pasangan (animasi buka, gerak gulir) | unik — yang kembar diganti gerak gulirnya |

Varian dipilih sesuai rumpun budaya (`AFINITAS`: kubah untuk kesultanan
Melayu/Aceh, gunungan untuk Jawa, atap runcing untuk Minang/Batak/Toraja,
ombak untuk pesisir …) lalu disebar merata. Tema templat & tema video
(koreografi sendiri) tidak diberi rupa.

## Masukan: `tema.json`

Dump tema TAYANG **sebelum** data 033/035 diterapkan (kalau sesudahnya, gerak
kembar yang sudah diganti 033 tak terdeteksi lagi dan hilang dari berkas):

```
psql "$DATABASE_URL" -Atc "select json_agg(json_build_object('slug',slug,'name',name,
  'nuansa',nuansa,'region',region,'category',category,'dark',dark,'listed',listed,
  'template',coalesce(template,''),'layout',layout,'open_anim',open_anim,
  'scroll_anim',scroll_anim,'float_deco',float_deco,'font',font,
  'script_font',script_font,'sort_order',sort_order,'bg_video',bg_video)
  order by sort_order) from themes where listed" > scripts/rupa/tema.json
```

## Menambah varian

1. Tulis `style/rupa/{huruf}-{varian}.css` — semua selektor diawali
   `.inv.rp-{huruf}-{varian}`; hanya transform/opacity bila beranimasi.
2. Daftarkan di `BAGIAN` (rupa.rs) **dan** `gaya::CSS` — test
   `varian_terdaftar_punya_css_dan_sebaliknya` gagal bila salah satu lupa.
3. Tambahkan ke `BAGIAN` di build.py, jalankan ulang.

Uji cepat tanpa compile: undangan demo menerima `?rupa=sampul:kubah,judul:pita`.

## Artefak

`033_rupa.sql` & `035_rupa_suku.sql` = ARTEFAK, jangan disunting tangan.
Idempoten: rupa ditulis hanya bila masih `{}` (belum diatur admin), gerak
hanya bila `motion_locked = FALSE`.
