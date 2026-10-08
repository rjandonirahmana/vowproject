# Generator 38 tema provinsi

Membangkitkan **aset SVG** di `public/img/tema/provinsi/` dan
**`migration/031_tema_provinsi.sql`** (38 tema — satu per provinsi — beserta
20 animasi pintu khas dan 380 ornamen).

```
python3 scripts/provinsi/build.py .
```

| Berkas | Isi |
|---|---|
| `wilayah.py` | Daftar 38 provinsi: nama tema, rumpun budaya (filter katalog), deskripsi, tag, warna, set ornamen, ubin motif, lambang, ikon khusus, animasi buka, koreografi gulir, hiasan melayang, tata letak, huruf. **Ubah/tambah tema di sini.** |
| `build.py` | Menurunkan token warna (cek kontras — gagal = berhenti), menggambar SVG dalam warna tiap tema, menulis SQL. |

Gambar dipakai ulang dari generator lama (keduanya kini aman diimpor, keluarannya
sendiri tidak berubah):

- `scripts/nusantara/motifs.py` — ubin motif, lambang, hiasan kartu, bingkai,
  latar, daun pintu animasi buka;
- `scripts/ornamen/adat.py` — ornamen khas daerah (tokoh pengapit, bangunan
  adat, hiasan gantung, kipas sudut, lambang) + penempatan & gerak 10 ornamen.

## Supaya tiap tema benar-benar beda

Jangan hanya mengganti warna (katalog lama dipangkas karena "sama beda warna").
Tiap baris memilih kombinasi sendiri: set ornamen + ubin + lambang + ikon,
animasi buka (`khas:<varian>` = pintu baru bermotif tema ini, lihat
`VARIAN_BUKA`), koreografi gulir, hiasan, tata letak, dan huruf.

## Aturan

- **SVG & `031_tema_provinsi.sql` = ARTEFAK.** Jangan sunting tangan.
- Migrasi idempoten (`ON CONFLICT DO NOTHING`; ornamen hanya untuk tema yang belum
  punya). Setelah tayang, sumber kebenaran tiap tema = **database** (Admin → Tema).
- Koreografi gulir baru: keadaan tersembunyi jangan memotong elemen selebar nol
  sambil menggesernya ke samping (lihat `migration/030_perbaikan_gerak.sql`).
