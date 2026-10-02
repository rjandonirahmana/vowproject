# Generator 100 tema adat Nusantara

Membangkitkan **aset SVG** di `public/img/tema/nusantara/` dan
**`migration/009_tema_nusantara.sql`** (100 tema + 11 animasi khas daerah).

```
cd scripts/nusantara && python3 build.py ../..
```

| Berkas | Isi |
|---|---|
| `data.py` | Daftar tema per daerah (nama, deskripsi, tag, warna, motif, lambang) + konfigurasi rumpun (huruf, animasi). **Ubah/tambah tema di sini.** |
| `motifs.py` | Gambar SVG: ubin motif, lambang daerah, hiasan kartu, bingkai, latar, daun pintu animasi, hiasan melayang. |
| `build.py` | Menurunkan token warna (dengan cek kontras otomatis — gagal = berhenti), menulis SVG & SQL. |

## Aturan penting

- **SVG & `009_tema_nusantara.sql` = ARTEFAK.** Jangan sunting tangan: menjalankan
  `build.py` lagi menimpanya. Ubah `data.py`/`motifs.py` lalu bangkitkan ulang.
- Migrasi memakai `ON CONFLICT DO NOTHING`: tema/animasi yang **sudah disunting admin
  di database tidak ditimpa**. Untuk menerapkan perubahan generator ke tema yang
  sudah ada, hapus/sunting barisnya lewat admin dulu, atau buat migrasi baru.
- Setelah tayang, sumber kebenaran tiap tema adalah **database** (Admin → Tema /
  Animasi); generator hanya untuk membuat tema massal.
- Ukuran saat ini ±1,2 MB (305 SVG) dan ikut ke image Docker lewat `public/`.
