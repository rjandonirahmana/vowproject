-- Tiga tema 3D baru dengan keluarga undangan, palet, dan adegan panggung
-- khusus yang berbeda. Slug panggung dipetakan ke adegan terpisah di
-- web/panggung.rs; tidak memakai ulang siluet tema 3D yang sudah ada.
INSERT INTO themes (
    slug, name, category, nuansa, palette, region, description, tags, badge, rating, reviews,
    layout, ornament, font, tokens, dark, image_url, image_mode, listed, sort_order,
    script_font, bg_image, frame_image, card_deco, float_deco, open_anim, page_mode, scroll_anim,
    gerak_judul, gerak_foto, ken_burns, bg_video, open_video, template, template_assets, template_css
)
VALUES
(
    'aurora-kristal-3d', 'Aurora Kristal 3D', '3D Dimensi', 'Modern', 'navy', 'Pegunungan Aurora',
    'Panggung istana kaca di tengah aurora hijau. Susunan undangan bergaya warkah bersegel dengan galeri coverflow 3D dan nuansa kristal yang terang.',
    '["3D Dimensi", "Aurora", "Kristal", "Sinematik"]', '3D', '', '',
    'gerbang', 'none', 'cinzel',
    '{"bg":"#edf7f5","card":"#f9fffc","primary":"#397b78","on-primary":"#ffffff","gold":"#8bd7bd","gold-deep":"#245b59","gold-pale":"#d9eee8","ink":"#1d353c","muted":"#607b7c","line":"#d6e8e2"}',
    FALSE, '', 'sudut', TRUE, 9,
    'great-vibes', '', '', '', 'kilau-emas', 'none', 'satu', 'anggun',
    'pudar', 'zoom', TRUE, '', '', 'warkah', '{}',
    '.t-warkah{--w-deep:#214a50;--w-deep-2:#142f3d;--w-paper:#edf7f5;--w-card:#f9fffc;--w-ink:#1d353c;--w-ink-soft:#607b7c;--w-on-deep:#effff8;--w-gold:#75c9ad;--w-gold-l:#c4f1dc;--w-line:rgba(43,105,99,.2);--w-glass:rgba(248,255,251,.86);--w-petal:#75c9ad}'
),
(
    'samudra-mutiara-3d', 'Samudra Mutiara 3D', '3D Dimensi', 'Alam', 'navy', 'Teluk Mutiara',
    'Istana mutiara di dasar laut dengan pilar karang dan cahaya yang menembus air. Undangan taman bergerak 3D diwarnai biru laguna, hijau laut, dan kilau koral.',
    '["3D Dimensi", "Samudra", "Mutiara", "Gerak Gulir"]', '3D', '', '',
    'klasik', 'none', 'cormorant',
    '{"bg":"#eaf7f5","card":"#f8fffd","primary":"#187f83","on-primary":"#ffffff","gold":"#edbd87","gold-deep":"#155d69","gold-pale":"#d3f0e9","ink":"#163d48","muted":"#52747a","line":"#c9e5e2"}',
    FALSE, '', 'sudut', TRUE, 10,
    'great-vibes', '', '', '', 'kupu', 'none', 'satu', 'ombak',
    'geser-kanan', 'geser-kiri', TRUE, '', '', 'taman', '{}',
    '.t-taman{--g-bg:#eaf7f5;--g-bg2:#cbe8e7;--g-card:#f8fffd;--g-ink:#163d48;--g-soft:#52747a;--g-deep:#187f83;--g-on-deep:#ffffff;--g-daun:#168a86;--g-daun2:#73c5b2;--g-bunga:#f1ad91;--g-bunga2:#b8eee1;--g-putik:#edbd87;--g-line:#c9e5e2}.t-taman .g-bunga{filter:drop-shadow(0 4px 12px rgba(9,94,111,.22))}'
),
(
    'danau-rembulan-3d', 'Danau Rembulan 3D', '3D Dimensi', 'Modern', 'navy', 'Danau Rembulan',
    'Paviliun terapung dan jembatan lengkung di bawah bulan purnama, dengan teratai yang muncul di depan layar. Panggung malam ini memakai siluet dan palet khusus, berbeda dari tema taman dan surat.',
    '["3D Dimensi", "Rembulan", "Danau", "Sinematik"]', '3D', '', '',
    'gerbang', 'none', 'cormorant',
    '{"bg":"#171d38","card":"#f8f4f2","primary":"#59658e","on-primary":"#ffffff","gold":"#d8c3ed","gold-deep":"#3f486e","gold-pale":"#e9dff0","ink":"#242744","muted":"#6a6c87","line":"#e4ddea"}',
    TRUE, '', 'sudut', TRUE, 11,
    'parisienne', '', '', '', 'bintang', 'none', 'satu', 'kosmik',
    'blur', 'lipat', TRUE, '', '', '', '{}', ''
)
ON CONFLICT (slug) DO NOTHING;
