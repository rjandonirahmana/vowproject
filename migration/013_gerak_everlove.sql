-- ═══════════════════════════════════════════════════════════════════════════
-- 013_gerak_everlove — SEMUA tema memakai gerak ala undangan premium
-- (referensi: everlove.invisimple.id — Elementor + Animate.css):
--   • isi naik bertahap 1,25 dtk, jeda antar elemen 220 ms (gerak scroll "anggun")
--   • judul & foto "zoom masuk" dari kecil; foto sampul Ken Burns
--   • ornamen bunga di tiap bagian: sudut BERAYUN masuk (rotateInDown) jeda
--     500–1400 ms lalu bergoyang, bunga tunggal berputar pelan, untaian melayang
--
-- WAJIB setelah 012_ornamen.sql. DIBANGKITKAN scripts/ornamen/build.py.
-- Ornamen contoh lama yang BELUM pernah disunting admin (updated_at =
-- created_at) diganti set baru; ornamen hasil suntingan/unggahan admin tetap.
-- Tema yang sudah punya ornamen suntingan tidak ditambahi. Palet bunga dipilih
-- dari data tema (gelap → emas, Bali → kamboja, navy → biru, Jawa/Sunda →
-- melati, terra → merah/terra, blush → blush, lainnya → putih).
-- Setelah ini gerak tiap tema tetap bisa diubah di /admin/tema/… — JANGAN
-- jalankan ulang file ini setelah admin mengubah gerak tema (UPDATE themes
-- di bawah menimpa pilihan gerak semua tema).
-- ═══════════════════════════════════════════════════════════════════════════

INSERT INTO animations (kind, slug, name, spec, css, builtin, sort_order)
VALUES ('scroll', 'anggun', 'Anggun bertahap (ala undangan premium)', '{}'::jsonb,
        '--rv-from:translateY(60px);--rv-dur:1.25s;--rv-stagger:220;', TRUE, 15)
ON CONFLICT (kind, slug) DO NOTHING;

UPDATE themes SET scroll_anim = 'anggun', gerak_judul = 'zoom-masuk', gerak_foto = 'zoom-masuk', ken_burns = TRUE, updated_at = NOW();

DELETE FROM theme_ornaments WHERE img LIKE '/img/tema/ornamen/%' AND updated_at = created_at;

INSERT INTO theme_ornaments (theme, bagian, img, posisi, x, y, lebar, rotasi, cermin, masuk, jeda, durasi, gerak, kecepatan, depan, hp, opasitas, urutan)
SELECT t.slug, v.bagian, '/img/tema/ornamen/' || v.bentuk || '-' || CASE
        WHEN t.dark THEN 'emas'
        WHEN t.nuansa = 'Bali' THEN 'kamboja'
        WHEN t.palette = 'blush' THEN 'blush'
        WHEN t.palette = 'navy' THEN 'biru'
        WHEN t.nuansa IN ('Jawa', 'Sunda') THEN 'melati'
        WHEN t.palette = 'terra' AND t.nuansa IN ('Maluku', 'Papua', 'NTT', 'Dayak', 'Toraja', 'Modern') THEN 'terra'
        WHEN t.palette = 'terra' THEN 'merah'
        WHEN t.palette = 'gold' THEN 'melati'
        ELSE 'putih' END || '.svg',
       v.posisi, v.x, v.y, v.lebar, v.rotasi, v.cermin, v.masuk, v.jeda, v.durasi, v.gerak, v.kecepatan, v.depan, v.hp, v.opasitas, v.urutan
  FROM themes t
 CROSS JOIN (VALUES
    ('sampul', 'sudut', 'kiri-atas', -14, -12, 46, 0, FALSE, 'ayun-kiri', 600, 1300, 'goyang', 7, FALSE, TRUE, 100, 10),
    ('sampul', 'sudut', 'kanan-bawah', 14, 10, 42, 180, FALSE, 'ayun-kanan', 1400, 1300, 'goyang', 8, FALSE, TRUE, 100, 20),
    ('mempelai', 'sudut', 'kiri-atas', -18, -16, 34, 0, FALSE, 'ayun-kiri', 500, 1300, 'goyang', 7, FALSE, TRUE, 90, 30),
    ('mempelai', 'ranting', 'kanan-tengah', 34, 0, 24, 0, TRUE, 'ayun-kanan', 900, 1300, 'goyang', 6, FALSE, FALSE, 85, 40),
    ('kisah', 'bunga', 'kanan-atas', 8, 20, 22, 0, FALSE, 'zoom', 1000, 1200, 'putar', 20, FALSE, TRUE, 75, 50),
    ('galeri', 'untaian', 'tengah-atas', 0, -80, 80, 0, FALSE, 'turun', 300, 1200, 'melayang', 5, FALSE, TRUE, 95, 60),
    ('acara', 'sudut', 'kanan-atas', 16, -14, 36, 0, TRUE, 'ayun-kanan', 500, 1300, 'goyang', 7, FALSE, TRUE, 100, 70),
    ('acara', 'sudut', 'kiri-bawah', -16, 18, 30, 180, TRUE, 'ayun-kiri', 1000, 1300, 'goyang', 8, FALSE, FALSE, 85, 80),
    ('rsvp', 'bunga', 'kiri-atas', -14, -22, 24, 0, FALSE, 'zoom', 1000, 1200, 'putar', 20, FALSE, TRUE, 85, 90),
    ('rsvp', 'sudut', 'kanan-bawah', 16, 16, 30, 180, FALSE, 'ayun-kanan', 500, 1300, 'goyang', 8, FALSE, TRUE, 90, 100)
 ) AS v(bagian, bentuk, posisi, x, y, lebar, rotasi, cermin, masuk, jeda, durasi, gerak, kecepatan, depan, hp, opasitas, urutan)
 WHERE NOT EXISTS (SELECT 1 FROM theme_ornaments o WHERE o.theme = t.slug)
 ORDER BY t.slug, v.urutan;
