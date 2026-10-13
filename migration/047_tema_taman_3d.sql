-- ═══════════════════════════════════════════════════════════════════════════
-- 047_tema_taman_3d — 4 tema nikah ALAM TAMAN 3D (templat `taman`).
--
-- Kategori "3D Dimensi". Bukan hanya panggung 3D (prolog adegan alam/taman,
-- web/panggung.rs: slug taman-* → "alam" untuk kulit sage, selain itu
-- "taman"): SETIAP bagian bergerak terikat gulir (CSS scroll-driven) —
-- kartu miring 3D, bunga mekar, sulur tumbuh, foto berputar masuk, galeri
-- cincin 3D yang berputar mengikuti gulir, daun & kupu-kupu ber-parallax.
-- Templat `taman` (templat/taman/) di-seed otomatis saat server start.
-- Baris yang SUDAH ada tidak ditimpa.
-- ═══════════════════════════════════════════════════════════════════════════

INSERT INTO themes (slug, name, category, nuansa, palette, region, description, tags, badge, rating, reviews,
                    layout, ornament, font, tokens, dark, image_url, image_mode, listed, sort_order,
                    script_font, bg_image, frame_image, card_deco, float_deco, open_anim, page_mode, scroll_anim,
                    gerak_judul, gerak_foto, ken_burns, bg_video, open_video, template, template_assets, template_css)
VALUES
       ('taman-mawar', 'Taman Mawar 3D', '3D Dimensi', 'Alam', 'blush', 'Taman Bunga',
        'Kamera melangkah ke taman mawar, lalu seluruh undangan ikut bergerak saat digulir: kartu miring 3D, mawar mekar, foto berputar masuk, galeri cincin 3D yang berputar.',
        '["3D Dimensi", "Taman", "Mawar", "Gerak Gulir"]', '3D', '', '',
        'klasik', 'none', 'cormorant',
        '{"bg": "#fbf4f3", "card": "#fffdfc", "primary": "#9b4d5f", "on-primary": "#ffffff", "gold": "#e8c26a", "gold-deep": "#7a3446", "gold-pale": "#f6e3e3", "ink": "#3a2a2e", "muted": "#7a666b", "line": "#efdcdc"}',
        FALSE, '', 'sudut', TRUE, 5,
        'great-vibes', '', '', '', 'none', 'none', 'satu', '',
        '', '', FALSE, '', '', 'taman', '{}',
        '.t-taman{--g-bg:#fbf4f3;--g-bg2:#f3e2e2;--g-card:#fffdfc;--g-ink:#3a2a2e;--g-soft:#7a666b;--g-deep:#9b4d5f;--g-daun:#7f9a6e;--g-daun2:#b5c7a2;--g-bunga:#e597a8;--g-bunga2:#f8d4dc;--g-putik:#e8c26a;--g-line:#efdcdc}'),
       ('taman-hijau', 'Hutan Taman 3D', '3D Dimensi', 'Alam', 'sage', 'Taman Hutan',
        'Rimbun hijau sage: kamera menembus pepohonan, daun berguguran mengikuti gulir, sulur tumbuh di tepi kartu acara, dan galeri berputar seperti cincin 3D.',
        '["3D Dimensi", "Taman", "Hutan", "Gerak Gulir"]', '3D', '', '',
        'klasik', 'none', 'cormorant',
        '{"bg": "#f2f5ee", "card": "#fdfdf8", "primary": "#46653f", "on-primary": "#ffffff", "gold": "#d9b45a", "gold-deep": "#33502d", "gold-pale": "#e3ead9", "ink": "#263222", "muted": "#5f6c58", "line": "#dbe3d1"}',
        FALSE, '', 'sudut', TRUE, 6,
        'great-vibes', '', '', '', 'none', 'none', 'satu', '',
        '', '', FALSE, '', '', 'taman', '{}',
        '.t-taman{--g-bg:#f2f5ee;--g-bg2:#e2e9d8;--g-card:#fdfdf8;--g-ink:#263222;--g-soft:#5f6c58;--g-deep:#46653f;--g-daun:#6d8c5d;--g-daun2:#a3ba8e;--g-bunga:#f0d9a8;--g-bunga2:#fbeccb;--g-putik:#d9b45a;--g-line:#dbe3d1}'),
       ('taman-lavender', 'Taman Lavender 3D', '3D Dimensi', 'Alam', 'blush', 'Taman Bunga',
        'Hamparan lavender lembut: kupu-kupu terbang menyertai gulir, angka hitung mundur membalik, dan setiap kartu mekar dari kedalaman.',
        '["3D Dimensi", "Taman", "Lavender", "Gerak Gulir"]', '3D', '', '',
        'klasik', 'none', 'cormorant',
        '{"bg": "#f6f3fa", "card": "#fffdff", "primary": "#6c5a9c", "on-primary": "#ffffff", "gold": "#e6c47a", "gold-deep": "#4f3f80", "gold-pale": "#ebe4f5", "ink": "#2e2840", "muted": "#6d6680", "line": "#e5dff0"}',
        FALSE, '', 'sudut', TRUE, 7,
        'great-vibes', '', '', '', 'none', 'none', 'satu', '',
        '', '', FALSE, '', '', 'taman', '{}',
        '.t-taman{--g-bg:#f6f3fa;--g-bg2:#e9e2f3;--g-card:#fffdff;--g-ink:#2e2840;--g-soft:#6d6680;--g-deep:#6c5a9c;--g-daun:#8a9c7a;--g-daun2:#bccab0;--g-bunga:#b8a2dc;--g-bunga2:#ddd1f0;--g-putik:#e6c47a;--g-line:#e5dff0}'),
       ('taman-senja', 'Taman Senja 3D', '3D Dimensi', 'Alam', 'terra', 'Taman Bunga',
        'Cahaya senja keemasan di taman: bunga terakota mekar satu per satu, foto mempelai berputar masuk dari kiri & kanan, galeri cincin 3D berpendar hangat.',
        '["3D Dimensi", "Taman", "Senja", "Gerak Gulir"]', '3D', '', '',
        'klasik', 'none', 'cormorant',
        '{"bg": "#faf3ea", "card": "#fffcf7", "primary": "#a85a3a", "on-primary": "#ffffff", "gold": "#e0a95a", "gold-deep": "#7d3f25", "gold-pale": "#f3e2cf", "ink": "#3a2a22", "muted": "#7a6658", "line": "#efdfcd"}',
        FALSE, '', 'sudut', TRUE, 8,
        'great-vibes', '', '', '', 'none', 'none', 'satu', '',
        '', '', FALSE, '', '', 'taman', '{}',
        '.t-taman{--g-bg:#faf3ea;--g-bg2:#f1e1cf;--g-card:#fffcf7;--g-ink:#3a2a22;--g-soft:#7a6658;--g-deep:#a85a3a;--g-daun:#8a9a66;--g-daun2:#c2c99a;--g-bunga:#e39a72;--g-bunga2:#f6d2bb;--g-putik:#e0a95a;--g-line:#efdfcd}')
ON CONFLICT (slug) DO NOTHING;
