-- 052_tema_3d_gulir_beda — tiga tema 3D baru, masing-masing dengan ADEGAN
-- panggung sendiri dan cara GULIR (gerak kamera) yang berbeda satu sama lain
-- (slug → adegan di web/panggung.rs; keyframes kamera di panggung/_dasar.css):
--
--   gurun-senja-3d      Gurun Senja       k6 sapu samping (dolly menyusur kiri→kanan)
--   salju-lentera-3d    Salju Lentera     k7 tilt naik (kamera naik dari tanah menengadah)
--   ladang-lavender-3d  Ladang Lavender   k8 ayun (mengayun kiri-kanan sambil mendekat)
--
-- Gerak isi (scroll_anim), pembuka (open_anim) & hiasan melayang juga dibuat
-- beda per tema. Idempoten (ON CONFLICT DO NOTHING).
INSERT INTO themes (
    slug, name, category, nuansa, palette, region, description, tags, badge, rating, reviews,
    layout, ornament, font, tokens, dark, image_url, image_mode, listed, sort_order,
    script_font, bg_image, frame_image, card_deco, float_deco, open_anim, page_mode, scroll_anim,
    gerak_judul, gerak_foto, ken_burns, bg_video, open_video, template, template_assets, template_css
)
VALUES
(
    'gurun-senja-3d', 'Gurun Senja 3D', '3D Dimensi', 'Alam', 'gold', 'Oasis Senja',
    'Istana pasir berkubah di antara bukit gurun saat matahari tenggelam, dengan siluet palem di depan layar. Kamera menyusur samping melewati paralaks bukit pasir sebelum masuk ke gerbang istana.',
    '["3D Dimensi", "Gurun", "Senja", "Sinematik"]', '3D', '', '',
    'klasik', 'none', 'cinzel',
    '{"bg":"#fbf1e4","card":"#fffaf2","primary":"#8a3b4e","on-primary":"#ffffff","gold":"#d9893c","gold-deep":"#8a4b1c","gold-pale":"#f6dcb8","ink":"#3a2230","muted":"#7a5f63","line":"#ead6c0"}',
    FALSE, '', 'sudut', TRUE, 12,
    'great-vibes', '', '', '', 'kilau-emas', 'candi-bentar', 'satu', 'geser',
    'geser-kiri', 'geser-kanan', TRUE, '', '', '', '{}', ''
),
(
    'salju-lentera-3d', 'Salju Lentera 3D', '3D Dimensi', 'Alam', 'navy', 'Lembah Salju',
    'Pondok kayu berlentera di lembah bersalju di bawah bulan purnama. Kamera mulai rendah dari hamparan salju lalu naik perlahan menengadah ke pondok yang hangat bercahaya.',
    '["3D Dimensi", "Salju", "Lentera", "Sinematik"]', '3D', '', '',
    'gerbang', 'none', 'cormorant',
    '{"bg":"#0f1b2e","card":"#182a44","primary":"#bfe0ff","on-primary":"#0f1b2e","gold":"#ffcf8a","gold-deep":"#ffcf8a","gold-pale":"#34476a","ink":"#eaf2fb","muted":"#a9bad2","line":"#2a3f60"}',
    TRUE, '', 'sudut', TRUE, 13,
    'parisienne', '', '', '', 'kunang', 'awan-berarak', 'satu', 'blur',
    'naik', 'zoom', TRUE, '', '', '', '{}', ''
),
(
    'ladang-lavender-3d', 'Ladang Lavender 3D', '3D Dimensi', 'Alam', 'blush', 'Ladang Lavender',
    'Pergola putih berhias untaian lavender di tengah ladang ungu saat senja keemasan. Kamera mengayun santai ke kiri dan kanan seperti berjalan menyusuri barisan bunga menuju pergola.',
    '["3D Dimensi", "Lavender", "Pergola", "Sinematik"]', '3D', '', '',
    'klasik', 'none', 'cormorant',
    '{"bg":"#f7f0fb","card":"#fffdff","primary":"#5e3c8f","on-primary":"#ffffff","gold":"#c99a4a","gold-deep":"#7a5a1c","gold-pale":"#f0e2c2","ink":"#2d2140","muted":"#6c6080","line":"#e3d8ef"}',
    FALSE, '', 'sudut', TRUE, 14,
    'great-vibes', '', '', '', 'kelopak', 'kupu-kupu', 'satu', 'ombak',
    'lebar', 'zoom-keluar', TRUE, '', '', '', '{}', ''
)
ON CONFLICT (slug) DO NOTHING;
