-- ═══════════════════════════════════════════════════════════════════════════
-- 045_tema_3d — 4 tema PANGGUNG 3D ala video undangan 3D ("white love"):
-- prolog sinematik digerakkan gulir (kamera menembus gerbang → mempelai di
-- depan bangunan → acara). Adegan dipilih dari slug (web/panggung.rs):
--   masjid-putih   Masjid Putih          gerbang lengkung, kubah kisi, menara
--   pendopo-putih  Pendopo Agung Jawa    joglo + candi bentar
--   gerbang-bulan  Gerbang Bulan         tembok gerbang bulan, aula, pagoda
--   kapel-putih    Kapel Putih           lengkung bunga, kapel bermenara
-- Tema lama bernuansa Islami/Jawa/Tionghoa/Kristiani ikut mendapat panggung
-- secara otomatis (tanpa migrasi). Idempoten.
-- ═══════════════════════════════════════════════════════════════════════════

INSERT INTO themes (slug, name, category, nuansa, palette, region, description, tags, badge, layout, ornament, font, tokens, dark, sort_order, script_font, bg_image, frame_image, card_deco, float_deco, open_anim, page_mode, scroll_anim, image_url, image_mode, gerak_judul, gerak_foto, ken_burns, listed) VALUES
('masjid-putih', 'Masjid Putih 3D', '3D Dimensi', 'Islami', 'sage', 'Islami', 'Prolog 3D sinematik: kamera terbang menembus gerbang lengkung masjid putih berkubah kisi, ayat Ar-Rum 21 di dalam lengkung, lalu mempelai berdiri di depan masjid di bawah bulan purnama.', '["3D Dimensi", "Islami", "Masjid", "Sinematik"]'::jsonb, '3D', 'gerbang', 'none', 'cormorant',
 '{"bg": "#f5f6f4", "card": "#ffffff", "primary": "#3d5a52", "on-primary": "#ffffff", "gold": "#c8a45a", "gold-deep": "#7a5e1c", "ink": "#24302c", "muted": "#66706b"}'::jsonb, FALSE, 1, 'great-vibes', '', '', '', 'bintang', 'mashrabiya', 'satu', 'anggun', '', 'pola', 'ikut', 'ikut', TRUE, TRUE),
('pendopo-putih', 'Pendopo Agung 3D', '3D Dimensi', 'Jawa', 'gold', 'Jawa', 'Prolog 3D sinematik: kamera melangkah di antara candi bentar menuju pendopo joglo berukir gebyok dengan mustaka emas, di bawah mentari pagi yang hangat.', '["3D Dimensi", "Jawa", "Joglo", "Sinematik"]'::jsonb, '3D', 'klasik', 'none', 'cinzel',
 '{"bg": "#faf7f0", "card": "#fffdf8", "primary": "#6b4f3a", "on-primary": "#ffffff", "gold": "#c9a24a", "gold-deep": "#7a5a20", "ink": "#2e241c", "muted": "#74675a"}'::jsonb, FALSE, 2, 'great-vibes', '', '', '', 'melati-gugur', 'gebyok-ukir', 'satu', 'keraton', '', 'pola', 'ikut', 'ikut', TRUE, TRUE),
('gerbang-bulan', 'Gerbang Bulan 3D', '3D Dimensi', 'Tionghoa', 'terra', 'Tionghoa', 'Prolog 3D sinematik: kamera menembus gerbang bulan berlampion merah menuju aula bertiang merah beratap lengkung emas, pagoda menjulang di kejauhan.', '["3D Dimensi", "Tionghoa", "Gerbang Bulan", "Sinematik"]'::jsonb, '3D', 'klasik', 'none', 'cinzel',
 '{"bg": "#f8f6f3", "card": "#fffdfa", "primary": "#a3161c", "on-primary": "#ffffff", "gold": "#d4a43a", "gold-deep": "#8a6a1f", "ink": "#3a0d0f", "muted": "#7a5a52"}'::jsonb, FALSE, 3, 'great-vibes', '', '', '', 'kilau-emas', 'naga-emas', 'satu', 'zoom', '', 'pola', 'ikut', 'ikut', TRUE, TRUE),
('kapel-putih', 'Kapel Putih 3D', '3D Dimensi', 'Kristiani', 'blush', 'Kristiani', 'Prolog 3D sinematik: kamera berjalan melewati lengkung bunga pernikahan menuju kapel putih bermenara dengan jendela mawar, ayat Korintus menyambut di dalam lengkung.', '["3D Dimensi", "Kristiani", "Kapel", "Sinematik"]'::jsonb, '3D', 'gerbang', 'none', 'cormorant',
 '{"bg": "#faf7f7", "card": "#ffffff", "primary": "#8a5a6b", "on-primary": "#ffffff", "gold": "#c9a46a", "gold-deep": "#7d5a2c", "ink": "#2b1d22", "muted": "#7a6268"}'::jsonb, FALSE, 4, 'parisienne', '', '', '', 'kelopak', 'katedral', 'satu', 'anggun', '', 'pola', 'ikut', 'ikut', TRUE, TRUE)
ON CONFLICT (slug) DO NOTHING;
