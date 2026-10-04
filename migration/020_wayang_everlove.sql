-- ═══════════════════════════════════════════════════════════════════════════
-- 020_wayang_everlove — tema Gunungan Wayang Kulit disusun ala undangan
-- premium Jawa (referensi everlove): sampul tergulir → pintu gebyok ukir naik
-- & berayun → menembus kusen; pohon emas bergoyang di kiri-kanan, wayang kulit
-- ksatria & putri mengapit, pendopo joglo, awan mega mendung, sudut ukir,
-- burung melintas; isi masuk bertahap (koreografi keraton). Gerak berulang
-- hemat (takaran everlove): hanya pohon yang bergoyang pelan; wayang & awan
-- cukup masuk lalu diam.
-- Ilustrasi: public/img/tema/wayang/ (scripts/gerak/wayang.py).
-- Ornamen lama tema ini DIGANTI seluruhnya. WAJIB setelah 019.
-- ═══════════════════════════════════════════════════════════════════════════

UPDATE themes SET open_anim = 'gebyok-ukir', scroll_anim = 'keraton', float_deco = 'burung', layout = 'klasik',
       bg_image = '/img/tema/wayang/latar.svg', gerak_judul = 'ikut', gerak_foto = 'ikut', ken_burns = TRUE, updated_at = NOW()
 WHERE slug = 'gunungan-wayang-kulit';

DELETE FROM theme_ornaments WHERE theme = 'gunungan-wayang-kulit';

INSERT INTO theme_ornaments (theme, bagian, img, posisi, x, y, lebar, rotasi, cermin, masuk, jeda, durasi, gerak, kecepatan, depan, hp, opasitas, urutan)
SELECT 'gunungan-wayang-kulit', v.bagian, '/img/tema/wayang/' || v.img, v.posisi, v.x, v.y, v.lebar, v.rotasi, v.cermin, v.masuk, v.jeda, v.durasi, v.gerak, v.kecepatan, v.depan, v.hp, v.opasitas, v.urutan
  FROM (VALUES
    -- Sampul (tata letak klasik = tanpa kartu): sudut & awan di atas, pohon besar
    -- di kiri-kanan, pendopo samar di kaki, wayang ksatria & putri mengapit.
    ('sampul',   'awan.svg',           'tengah-atas',  0, -22, 72,  0, FALSE, 'turun',      300, 1400, 'none',     6, FALSE, TRUE, 50, 10),
    ('sampul',   'sudut.svg',          'kiri-atas',  -10, -10, 30,  0, FALSE, 'ayun-kiri',  500, 1300, 'none',     6, TRUE,  TRUE, 100, 20),
    ('sampul',   'sudut.svg',          'kanan-atas',  10, -10, 30,  0, TRUE,  'ayun-kanan', 600, 1300, 'none',     6, TRUE,  TRUE, 100, 30),
    ('sampul',   'pohon-1.svg',        'kiri-tengah', -50, -4, 78,  0, FALSE, 'tumbuh',     300, 1600, 'goyang',  14, FALSE, TRUE, 70, 40),
    ('sampul',   'pohon-2.svg',        'kanan-tengah', 50,  0, 78,  0, TRUE,  'tumbuh',     450, 1600, 'goyang',  14, FALSE, TRUE, 70, 50),
    ('sampul',   'pendopo.svg',        'tengah-bawah', 0,  12, 96,  0, FALSE, 'naik',       800, 1400, 'none',     6, FALSE, TRUE, 28, 60),
    ('sampul',   'wayang-ksatria.svg', 'kiri-tengah', -100, 30, 38,  0, FALSE, 'geser-kiri',1000, 1300, 'none',     6, FALSE, TRUE, 95, 70),
    ('sampul',   'wayang-putri.svg',   'kanan-tengah', 100, 32, 36,  0, TRUE,  'geser-kanan',1100,1300, 'none',     6, FALSE, TRUE, 95, 80),
    -- Mempelai: pohon membingkai, awan di atas.
    ('mempelai', 'awan.svg',           'tengah-atas',  0, -30, 64,  0, FALSE, 'turun',      200, 1300, 'none',     6, FALSE, TRUE, 45, 90),
    ('mempelai', 'pohon-2.svg',        'kiri-tengah', -55, 0, 60,  0, FALSE, 'tumbuh',     300, 1500, 'goyang',  14, FALSE, TRUE, 75, 100),
    ('mempelai', 'pohon-1.svg',        'kanan-tengah', 55, 10, 60,  0, TRUE,  'tumbuh',     500, 1500, 'goyang',  14, FALSE, TRUE, 75, 110),
    -- Kisah: wayang ksatria menengok, pendopo samar.
    ('kisah',    'wayang-ksatria.svg', 'kanan-atas',  24, -12, 26,  0, TRUE,  'geser-kanan', 400,1300, 'none',     6, FALSE, TRUE, 85, 120),
    ('kisah',    'pendopo.svg',        'tengah-bawah', 0,  20, 90,  0, FALSE, 'naik',       300, 1400, 'none',     6, FALSE, TRUE, 35, 130),
    -- Galeri: awan & pohon.
    ('galeri',   'awan.svg',           'tengah-atas',  0, -40, 70,  0, FALSE, 'turun',      200, 1300, 'none',     6, FALSE, TRUE, 55, 140),
    ('galeri',   'pohon-1.svg',        'kiri-atas',  -45, -20, 50,  0, FALSE, 'tumbuh',     400, 1500, 'goyang',  14, FALSE, TRUE, 70, 150),
    -- Acara: gunungan tumbuh di tengah atas, sudut ukir, pohon.
    ('acara',    'sudut.svg',          'kiri-atas',   -4,  -4, 30,  0, FALSE, 'ayun-kiri',  500, 1300, 'none',     6, TRUE,  TRUE, 100, 160),
    ('acara',    'sudut.svg',          'kanan-atas',   4,  -4, 30,  0, TRUE,  'ayun-kanan', 600, 1300, 'none',     6, TRUE,  TRUE, 100, 170),
    ('acara',    'pohon-2.svg',        'kanan-tengah', 50, 10, 56,  0, TRUE,  'tumbuh',     300, 1500, 'goyang',  14, FALSE, TRUE, 70, 180),
    -- RSVP: dua wayang berhadapan & pendopo.
    ('rsvp',     'wayang-ksatria.svg', 'kiri-bawah', -26,  10, 26,  0, FALSE, 'geser-kiri', 500, 1300, 'none',     6, FALSE, TRUE, 90, 190),
    ('rsvp',     'wayang-putri.svg',   'kanan-bawah', 26,  10, 24,  0, TRUE,  'geser-kanan', 600,1300, 'none',     6, FALSE, TRUE, 90, 200),
    ('rsvp',     'awan.svg',           'tengah-atas',  0, -30, 60,  0, FALSE, 'turun',      200, 1300, 'none',     6, FALSE, TRUE, 45, 210)
  ) AS v(bagian, img, posisi, x, y, lebar, rotasi, cermin, masuk, jeda, durasi, gerak, kecepatan, depan, hp, opasitas, urutan)
 WHERE EXISTS (SELECT 1 FROM themes t WHERE t.slug = 'gunungan-wayang-kulit');
