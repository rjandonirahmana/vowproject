-- 054_tema_susur_3d — dua tema SUSUR 3D: tamu DIAJAK BERJALAN menyusuri
-- sebuah dunia dari gapura sampai pelaminan, berhenti di enam singgah (kartu
-- singgah = data undangan). Kamera melangkah maju di jalan berkelok, menoleh
-- ke pemandangan; rumah, lumbung, pendopo & mercusuar dirakit dari bidang CSS
-- 3D. Slug → dunia di web/susur.rs (pilih); keyframes dibangkitkan dari jalur.
--
--   desa-sawah-3d     Desa Sawah     gapura bambu → pematang & saung → kincir
--                                    & kali → lumbung → tarub janur → pendopo joglo
--   pesisir-nyiur-3d  Pesisir Nyiur  gapura nelayan → pantai & perahu jukung →
--                                    dermaga & mercusuar → rumah panggung →
--                                    lorong kursi berlampu → lengkung bunga senja
--
-- Hanya data. Idempoten (ON CONFLICT DO NOTHING).
INSERT INTO themes (
    slug, name, category, nuansa, palette, region, description, tags, badge, rating, reviews,
    layout, ornament, font, tokens, dark, image_url, image_mode, listed, sort_order,
    script_font, bg_image, frame_image, card_deco, float_deco, open_anim, page_mode, scroll_anim,
    gerak_judul, gerak_foto, ken_burns, bg_video, open_video, template, template_assets, template_css
)
VALUES
(
    'desa-sawah-3d', 'Desa Sawah 3D', '3D Dimensi', 'Jawa', 'sage', 'Pedesaan Jawa',
    'Tamu diajak berjalan menyusuri desa: melewati gapura bambu berjanur, pematang di antara petak sawah, saung & orang-orangan, kincir di tepi kali, lumbung padi, hingga tarub janur dan pelaminan di pendopo joglo.',
    '["3D Dimensi", "Jalan-jalan", "Sawah", "Joglo", "Nusantara"]', '3D', '', '',
    'klasik', 'none', 'cormorant',
    '{"bg":"#f5f2e6","card":"#fffdf5","primary":"#5b6b2e","on-primary":"#ffffff","gold":"#c9a24b","gold-deep":"#7a5a1c","gold-pale":"#efe3c2","ink":"#2c2a1c","muted":"#6a6650","line":"#e3dcc4"}',
    FALSE, '', 'sudut', TRUE, 19,
    'great-vibes', '', '', '', 'daun-gugur', 'gebyok-ukir', 'satu', 'anggun',
    'naik', 'geser-kanan', TRUE, '', '', '', '{}', ''
),
(
    'pesisir-nyiur-3d', 'Pesisir Nyiur 3D', '3D Dimensi', 'Alam', 'navy', 'Pesisir Nusantara',
    'Senja di kampung nelayan: berjalan dari gapura berpelampung, menyusuri pantai berperahu jukung, meniti dermaga kayu dekat mercusuar dan rumah panggung, lalu lorong kursi berlampu menuju lengkung bunga pelaminan.',
    '["3D Dimensi", "Jalan-jalan", "Pantai", "Senja", "Dermaga"]', '3D', '', '',
    'klasik', 'none', 'playfair',
    '{"bg":"#fbf3ec","card":"#fffaf5","primary":"#2f6a86","on-primary":"#ffffff","gold":"#e98a5a","gold-deep":"#9a4a26","gold-pale":"#fbe0cc","ink":"#24323a","muted":"#5e6e76","line":"#ecdccf"}',
    FALSE, '', 'sudut', TRUE, 20,
    'parisienne', '', '', '', 'kelopak', 'ombak-laut', 'satu', 'ombak',
    'naik', 'zoom', TRUE, '', '', '', '{}', ''
)
ON CONFLICT (slug) DO NOTHING;
