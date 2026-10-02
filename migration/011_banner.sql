-- ═══════════════════════════════════════════════════════════════════════════
-- 011_banner — tabel `banners`: banner strip di atas beranda (/admin/banner).
-- Idempoten. Tabel sendiri (bukan site_content) supaya fitur banner bisa
-- bertambah lewat kolom baru tanpa mengubah struktur konten lain.
--
--   urutan        : kecil tampil dulu (diatur tombol ↑/↓ atau angka di admin)
--   mulai/selesai : jadwal tayang opsional (NULL = tanpa batas)
--   img / img_hp  : gambar desktop (strip 9:1, 2880×320) & HP (8:3, 1080×405)
-- ═══════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS banners (
    id          BIGSERIAL   PRIMARY KEY,
    judul       TEXT        NOT NULL DEFAULT '',
    sub         TEXT        NOT NULL DEFAULT '',
    cta         TEXT        NOT NULL DEFAULT '',
    link        TEXT        NOT NULL DEFAULT '',
    img         TEXT        NOT NULL DEFAULT '',
    img_hp      TEXT        NOT NULL DEFAULT '',
    aktif       BOOLEAN     NOT NULL DEFAULT TRUE,
    urutan      INT         NOT NULL DEFAULT 100,
    mulai       TIMESTAMPTZ,
    selesai     TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS banners_urutan_idx ON banners (urutan, id);

-- Pindahan dari versi sebelumnya (banner sempat disimpan di site_content).
INSERT INTO banners (judul, sub, cta, link, img, img_hp, aktif, urutan)
SELECT COALESCE(b->>'judul', ''), COALESCE(b->>'sub', ''), COALESCE(b->>'cta', ''), COALESCE(b->>'link', ''),
       COALESCE(b->>'img', ''), COALESCE(b->>'img_hp', ''), COALESCE((b->>'aktif')::boolean, TRUE), (t.ord * 10)::int
  FROM site_content sc, jsonb_array_elements(CASE WHEN jsonb_typeof(sc.data) = 'array' THEN sc.data ELSE '[]'::jsonb END) WITH ORDINALITY AS t(b, ord)
 WHERE sc.key = 'banner' AND NOT EXISTS (SELECT 1 FROM banners);

-- Isi awal (hanya bila tabel masih kosong).
INSERT INTO banners (judul, sub, cta, link, img, img_hp, aktif, urutan)
SELECT v.* FROM (VALUES
    ('100+ Tema Undangan Adat Nusantara',
     'Jawa, Sunda, Minang, Batak, Maluku, hingga Papua — lengkap dengan animasi pembuka khas daerah.',
     'Jelajahi Tema', '/#katalog', '/img/banner/banner-1-tema.webp', '/img/banner/banner-1-tema-hp.webp', TRUE, 10),
    ('Satu Admin untuk Semua Hari Bahagia',
     'Undangan digital, cetak fisik, dekorasi & sound, MUA, hingga sewa seserahan — tanya harga bundling.',
     'Lihat Layanan', '/dekorasi', '/img/banner/banner-2-layanan.webp', '/img/banner/banner-2-layanan-hp.webp', TRUE, 20),
    ('Coba Gratis, Bayar Saat Siap Disebar',
     'Buat undangan dalam 5 menit, lihat pratinjaunya dulu — RSVP, amplop digital & QR check-in sudah termasuk.',
     'Buat Undangan', '/buat', '/img/banner/banner-3-promo.webp', '/img/banner/banner-3-promo-hp.webp', TRUE, 30)
) AS v(judul, sub, cta, link, img, img_hp, aktif, urutan)
WHERE NOT EXISTS (SELECT 1 FROM banners);

DELETE FROM site_content WHERE key = 'banner';
