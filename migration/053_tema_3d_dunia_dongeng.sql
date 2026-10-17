-- 053_tema_3d_dunia_dongeng — empat tema 3D baru, masing-masing dengan ADEGAN
-- panggung sendiri (SVG dari scripts/gerak/tema3d_baru.py) dan gerak KAMERA
-- yang belum dipakai tema lain (slug → adegan di web/panggung.rs; keyframes
-- di panggung/_dasar.css):
--
--   langit-balon-3d   Langit Balon      k11 spiral turun dari langit
--   kastil-mawar-3d   Kastil Mawar      k12 dolly-zoom "vertigo"
--   hutan-kunang-3d   Hutan Kunang      k13 terbang rendah naik-turun
--   kebun-teh-3d      Kebun Teh Puncak  k14 crane turun berundak
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
    'langit-balon-3d', 'Langit Balon 3D', '3D Dimensi', 'Modern', 'blush', 'Pulau Awan',
    'Gazebo putih di pulau yang melayang di atas lautan awan, dikelilingi balon udara warna-warni. Kamera turun berputar dari langit sebelum mendarat di depan gazebo.',
    '["3D Dimensi", "Balon Udara", "Awan", "Sinematik"]', '3D', '', '',
    'klasik', 'none', 'playfair',
    '{"bg":"#fdf3f6","card":"#fffafc","primary":"#b4416a","on-primary":"#ffffff","gold":"#e9a24a","gold-deep":"#8c5212","gold-pale":"#fbe3c6","ink":"#3a2433","muted":"#735f6c","line":"#f1d9e2"}',
    FALSE, '', 'sudut', TRUE, 15,
    'great-vibes', '', '', '', 'kelopak', 'awan-berarak', 'satu', 'mekar',
    'zoom-masuk', 'naik', TRUE, '', '', '', '{}', ''
),
(
    'kastil-mawar-3d', 'Kastil Mawar 3D', '3D Dimensi', 'Modern', 'blush', 'Kastil Dongeng',
    'Kastil dongeng bermenara runcing di ujung jalan setapak, dibingkai lengkung pergola mawar. Kamera melaju dengan efek dolly-zoom: kastil tetap, dunia di belakangnya menjauh.',
    '["3D Dimensi", "Kastil", "Mawar", "Dongeng"]', '3D', '', '',
    'gerbang', 'none', 'cinzel',
    '{"bg":"#faf1f3","card":"#fffbfc","primary":"#7c2f52","on-primary":"#ffffff","gold":"#d39a4a","gold-deep":"#7d5418","gold-pale":"#f5e1c6","ink":"#33202b","muted":"#6f5a65","line":"#ecd8df"}',
    FALSE, '', 'sudut', TRUE, 16,
    'great-vibes', '', '', '', 'kelopak', 'gerbang', 'satu', 'sekar',
    'lebar', 'zoom', TRUE, '', '', '', '{}', ''
),
(
    'hutan-kunang-3d', 'Hutan Kunang 3D', '3D Dimensi', 'Alam', 'sage', 'Hutan Kunang-kunang',
    'Rumah pohon berlentera di pohon raksasa, di tengah hutan malam yang dipenuhi kunang-kunang dan jamur bercahaya. Kamera terbang rendah naik-turun di antara pakis.',
    '["3D Dimensi", "Hutan", "Kunang-kunang", "Malam"]', '3D', '', '',
    'gerbang', 'none', 'cormorant',
    '{"bg":"#0c1d1f","card":"#14292b","primary":"#a8e6c8","on-primary":"#0c1d1f","gold":"#ffe08a","gold-deep":"#ffe08a","gold-pale":"#25403d","ink":"#e8f5ee","muted":"#a3bfb3","line":"#24413d"}',
    TRUE, '', 'sudut', TRUE, 17,
    'parisienne', '', '', '', 'kunang', 'taman-daun', 'satu', 'bayang',
    'blur', 'pudar', TRUE, '', '', '', '{}', ''
),
(
    'kebun-teh-3d', 'Kebun Teh Puncak 3D', '3D Dimensi', 'Sunda', 'sage', 'Puncak, Jawa Barat',
    'Pendopo kayu beratap joglo di tengah kebun teh berundak yang berkabut pagi, dengan gunung di kejauhan. Kamera turun undakan demi undakan menuju pendopo.',
    '["3D Dimensi", "Kebun Teh", "Pendopo", "Nusantara"]', '3D', '', '',
    'klasik', 'none', 'cormorant',
    '{"bg":"#f4f6ec","card":"#fcfdf7","primary":"#3f6b2c","on-primary":"#ffffff","gold":"#b8873e","gold-deep":"#6e4e18","gold-pale":"#efe2c4","ink":"#24301c","muted":"#5c6852","line":"#dfe5d0"}',
    FALSE, '', 'sudut', TRUE, 18,
    'great-vibes', '', '', '', 'daun-gugur', 'gebyok-ukir', 'satu', 'anggun',
    'naik', 'geser-kanan', TRUE, '', '', '', '{}', ''
)
ON CONFLICT (slug) DO NOTHING;
