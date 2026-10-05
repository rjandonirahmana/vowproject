-- ═══════════════════════════════════════════════════════════════════════════
-- 022_sinema — tema VIDEO: video prewedding diputar penuh di belakang isi
-- undangan (tanpa suara, berulang), isi melayang transparan di atasnya.
--
--   themes.bg_video        video latar bawaan tema (tak kosong = tema sinema)
--   themes.open_video      video pembuka gerbang (cara everlove: pintu = video)
--   invitations.video_url  video prewedding pasangan (unggah /buat atau tautan
--                          .mp4/.webm) — menggantikan video bawaan & tampil di
--                          bagian "Video Prewedding" di semua tema
--
-- Tema baru "Sinema Kenangan": sampul polaroid naik seperti layar bioskop
-- (buka: layar-naik), isi tenang (scroll: sinema). Video demo bawaan
-- public/video/sinema-demo.webm dirender dari foto contoh situs sendiri.
-- WAJIB setelah 021. Aman dijalankan ulang. Kode tetap jalan sebelum file ini
-- dijalankan (kolom dibaca lewat to_jsonb; simpan video dilewati).
-- ═══════════════════════════════════════════════════════════════════════════

ALTER TABLE themes      ADD COLUMN IF NOT EXISTS bg_video  TEXT NOT NULL DEFAULT '';
ALTER TABLE invitations ADD COLUMN IF NOT EXISTS video_url TEXT NOT NULL DEFAULT '';
-- Video pembuka di gerbang (animasi buka video-pintu), diputar sekali.
ALTER TABLE themes      ADD COLUMN IF NOT EXISTS open_video TEXT NOT NULL DEFAULT '';

INSERT INTO themes (slug, name, category, nuansa, palette, region, description, tags, badge, layout, ornament, font, tokens, dark,
                    sort_order, script_font, bg_image, frame_image, card_deco, float_deco, open_anim, page_mode, scroll_anim,
                    image_url, image_mode, listed, bg_video, gerak_judul, gerak_foto, ken_burns)
VALUES ('sinema-kenangan', 'Sinema Kenangan', 'Modern', 'Modern', 'navy', 'Video Prewedding',
        'Video prewedding Anda diputar penuh di layar undangan — sampul polaroid terangkat seperti layar bioskop, isi melayang anggun di atas momen bergerak.',
        '["Video Latar", "Sampul Polaroid", "Mode Gelap"]'::jsonb, 'Baru • Video', 'klasik', 'none', 'cormorant',
        '{"bg": "#0f3640", "card": "#0b2930", "primary": "#d9b46c", "on-primary": "#0c2026", "gold": "#d9b46c", "gold-deep": "#e8cb8f",
          "gold-light": "#f0dba8", "gold-c": "#f6e2b4", "gold-pale": "#f8ecd0", "ink": "#f6f0e4", "muted": "#c6bfb1",
          "surface-low": "#12404b", "surface": "#164a56", "surface-high": "#1b5562", "sage": "#8fb3b0", "sage-mist": "#6f9592", "line": "#2b5964"}'::jsonb,
        TRUE, 11, 'great-vibes', '', '', '', 'none', 'layar-naik', 'satu', 'sinema',
        '', 'sudut', TRUE, '/video/sinema-demo.webm', 'ikut', 'ikut', FALSE)
ON CONFLICT (slug) DO NOTHING;

-- Gunungan Wayang Kulit ala everlove: pembuka = VIDEO pintu gebyok, latar =
-- VIDEO pemandangan (tersusun sekali lalu bagian ambient berulang dari detik
-- 2,6), gerak scroll persis everlove. Video dirender dari aset sendiri
-- (scripts/gerak/video/, img2mp4.swift). Pohon/pendopo/awan per bagian sudah
-- ada di video latar → ornamen SEED itu dibuang (sampul tetap).
UPDATE themes SET open_anim = 'video-pintu', open_video = '/video/wayang-buka.mp4',
       bg_video = '/video/wayang-latar.mp4#loop=2.6', scroll_anim = 'everlove', float_deco = 'none', bg_image = '',
       updated_at = NOW()
 WHERE slug = 'gunungan-wayang-kulit' AND NOT motion_locked;

DELETE FROM theme_ornaments
 WHERE theme = 'gunungan-wayang-kulit' AND source = 'seed' AND bagian <> 'sampul'
   AND (img LIKE '%/pohon-%' OR img LIKE '%/pendopo.svg' OR img LIKE '%/awan.svg');

-- Gambar BERGERAK (APNG transparan, scripts/gerak/video/orn.html → img2apng.swift)
-- menggantikan hiasan diam: wayang bergoyang di tangkai, sudut ukir berkilau,
-- kupu-kupu & kawanan burung. Pohon/awan/pendopo sampul dibuang — sudah ada di
-- video latar yang kini juga diputar di belakang sampul.
DELETE FROM theme_ornaments
 WHERE theme = 'gunungan-wayang-kulit' AND source = 'seed' AND bagian = 'sampul'
   AND (img LIKE '%/pohon-%' OR img LIKE '%/pendopo.svg' OR img LIKE '%/awan.svg');
UPDATE theme_ornaments SET img = '/img/tema/wayang/gerak/' || CASE
           WHEN img LIKE '%wayang-ksatria.svg' THEN 'ksatria.png'
           WHEN img LIKE '%wayang-putri.svg' THEN 'putri.png'
           ELSE 'sudut.png' END,
       gerak = 'none', updated_at = created_at
 WHERE theme = 'gunungan-wayang-kulit' AND source = 'seed'
   AND (img LIKE '%/wayang/wayang-ksatria.svg' OR img LIKE '%/wayang/wayang-putri.svg' OR img LIKE '%/wayang/sudut.svg');
INSERT INTO theme_ornaments (theme, bagian, img, posisi, x, y, lebar, rotasi, cermin, masuk, jeda, durasi, gerak, kecepatan, depan, hp, opasitas, urutan, source)
SELECT 'gunungan-wayang-kulit', v.bagian, '/img/tema/wayang/gerak/' || v.img, v.posisi, v.x, v.y, v.lebar, 0, v.cermin, v.masuk, v.jeda, 1500, 'none', 6, FALSE, TRUE, 100, v.urutan, 'seed'
  FROM (VALUES
    ('sampul',   'burung.png', 'tengah-atas',   10,  40, 46, FALSE, 'pudar',      1800, 300),
    ('mempelai', 'kupu.png',   'kanan-atas',    10,   0, 16, FALSE, 'pudar',       600, 310),
    ('kisah',    'burung.png', 'kiri-atas',    -10, -20, 40, FALSE, 'geser-kiri',  300, 320),
    ('galeri',   'kupu.png',   'kiri-atas',     -6, -30, 14, TRUE,  'pudar',       400, 330),
    ('acara',    'burung.png', 'kanan-atas',    10, -10, 40, TRUE,  'geser-kanan', 300, 340),
    ('rsvp',     'kupu.png',   'kanan-atas',     6, -20, 15, FALSE, 'pudar',       500, 350)
  ) AS v(bagian, img, posisi, x, y, lebar, cermin, masuk, jeda, urutan)
 WHERE EXISTS (SELECT 1 FROM themes WHERE slug = 'gunungan-wayang-kulit')
   AND NOT EXISTS (SELECT 1 FROM theme_ornaments o WHERE o.theme = 'gunungan-wayang-kulit' AND o.img LIKE '%/gerak/kupu.png');
