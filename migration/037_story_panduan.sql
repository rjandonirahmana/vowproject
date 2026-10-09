-- ═══════════════════════════════════════════════════════════════════════════
-- 037_story_panduan — story PANDUAN di beranda (di bawah banner) (hanya admin yang membuat,
-- /admin/story-panduan): gambar langkah memesan undangan dari memilih tema
-- sampai pembayaran dikonfirmasi admin. Ditonton seperti story Instagram.
-- Idempoten. Isi awal = 8 gambar panduan bawaan (public/img/panduan/*.webp), hanya
-- bila tabel masih kosong.
-- ═══════════════════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS site_stories (
    id          BIGSERIAL   PRIMARY KEY,
    judul       TEXT        NOT NULL DEFAULT '',   -- label lingkaran (pendek)
    teks        TEXT        NOT NULL DEFAULT '',   -- keterangan di bawah gambar
    img         TEXT        NOT NULL,              -- gambar tegak 9:16 (1080×1920)
    tautan      TEXT        NOT NULL DEFAULT '',   -- tombol aksi opsional (/… atau https://…)
    tombol      TEXT        NOT NULL DEFAULT '',   -- teks tombol
    aktif       BOOLEAN     NOT NULL DEFAULT TRUE,
    urutan      INT         NOT NULL DEFAULT 100,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS site_stories_urutan_idx ON site_stories (urutan, id);

INSERT INTO site_stories (judul, teks, img, tautan, tombol, urutan)
SELECT v.* FROM (VALUES
    ('Pilih Tema', 'Jelajahi katalog — saring menurut daerah, nuansa, dan warna favorit kalian.', '/img/panduan/langkah-1.webp', '/#katalog', 'Lihat Katalog', 10),
    ('Coba Demo', 'Buka demo tiap tema: rasakan animasi pembuka, musik, dan tampilannya di HP.', '/img/panduan/langkah-2.webp', '', '', 20),
    ('Isi Data', 'Tekan "Pilih" lalu isi nama mempelai, orang tua, dan jadwal akad & resepsi.', '/img/panduan/langkah-3.webp', '/buat', 'Buat Undangan', 30),
    ('Foto & Paket', 'Unggah foto, pilih lagu dari pustaka, lalu tentukan paket yang sesuai.', '/img/panduan/langkah-4.webp', '/paket', 'Lihat Paket', 40),
    ('Tautan Kelola', 'Setelah dikirim, tautan Kelola masuk ke WhatsApp — simpan baik-baik, itu kunci undanganmu.', '/img/panduan/langkah-5.webp', '', '', 50),
    ('Bayar', 'Transfer sesuai total, lalu unggah bukti transfer di halaman Kelola bagian Pembayaran.', '/img/panduan/langkah-6.webp', '', '', 60),
    ('Tunggu Admin', 'Admin memeriksa pembayaran. Begitu aktif, kamu dikabari lewat WhatsApp — biasanya tak lama.', '/img/panduan/langkah-7.webp', '', '', 70),
    ('Sebar!', 'Undangan aktif: kirim tautan pribadi ke tiap tamu dan pantau RSVP dari Kelola.', '/img/panduan/langkah-8.webp', '/buat', 'Mulai Sekarang', 80)
) AS v(judul, teks, img, tautan, tombol, urutan)
WHERE NOT EXISTS (SELECT 1 FROM site_stories);
