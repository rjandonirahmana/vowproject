-- 039 — Tema templat `warkah` (surat bersegel): 4 kulit warna.
--
-- Bahan: bedah 73 template chungdoi.com (Okt 2026). Semua template mereka
-- satu mesin & satu susunan — beda hanya palet, ornamen, font. Templat
-- `warkah` (templat/warkah/) menulis ulang gerak khasnya dari nol: segel
-- pecah + partikel, kartu terbang, amplop & polaroid, tanggal terbelah,
-- galeri coverflow 3D, kalender berhati, garis waktu, kotak kado.
-- Kulit = warna (template_css) + ornamen (template_assets); kolom gaya lama
-- tetap diisi agar kartu katalog & /kelola ikut bertema.
-- Baris yang SUDAH ada tidak ditimpa (suntingan admin aman).

INSERT INTO themes (slug, name, category, nuansa, palette, region, description, tags, badge, rating, reviews,
                    layout, ornament, font, tokens, dark, image_url, image_mode, listed, sort_order,
                    script_font, bg_image, frame_image, card_deco, float_deco, open_anim, page_mode, scroll_anim,
                    gerak_judul, gerak_foto, ken_burns, bg_video, open_video, template, template_assets, template_css)
VALUES
       ('warkah-marun', 'Warkah Marun', 'Modern', 'Modern', 'terra', 'Surat Bersegel',
        'Surat bersegel lilin: segel emas pecah, kelopak memancar, amplop marun melepas foto polaroid — minimalis, hangat, berwibawa.',
        '["Templat Khusus", "Segel Lilin", "Galeri 3D"]', 'Baru', '', '',
        'klasik', 'none', 'cormorant',
        '{"bg": "#f5eee4", "ink": "#4a1d22", "card": "#fbf7f1", "gold": "#c19a4b", "line": "#e6dcd2", "sage": "#7d5a5c", "ink-2": "#4a1d22", "muted": "#7d5a5c", "gold-c": "#e2c98f", "primary": "#561821", "surface": "#f5eee4", "gold-deep": "#3d0f16", "gold-pale": "#e2c98f", "primary-2": "#3d0f16", "sage-mist": "#7d5a5c", "gold-light": "#e2c98f", "on-primary": "#ffffff", "surface-low": "#fbf7f1", "surface-high": "#f5eee4"}',
        FALSE, '/img/tema/ornamen/sudut-merah.svg', 'sudut', TRUE, 0,
        'great-vibes', '', '', '', 'none', 'none', 'satu', '',
        '', '', FALSE, '', '', 'warkah',
        '{"sudut": "/img/tema/ornamen/sudut-merah.svg", "ranting": "/img/tema/ornamen/ranting-merah.svg", "untaian": "/img/tema/ornamen/untaian-merah.svg"}',
        '.t-warkah{--w-deep:#561821;--w-deep-2:#3d0f16;--w-paper:#f5eee4;--w-card:#fbf7f1;--w-ink:#4a1d22;--w-ink-soft:#7d5a5c;--w-on-deep:#f1e6d6;--w-gold:#c19a4b;--w-gold-l:#e2c98f;--w-petal:#c7646f;--w-line:color-mix(in srgb, #561821 16%, transparent);--t-nav-bg:color-mix(in srgb, #fbf7f1 92%, transparent)}'),
       ('warkah-sakura', 'Warkah Sakura', 'Modern', 'Modern', 'blush', 'Surat Bersegel',
        'Amplop merah muda bersegel, kelopak sakura berguguran, galeri 3D & kalender berhati — lembut dan romantis.',
        '["Templat Khusus", "Segel Lilin", "Galeri 3D"]', 'Baru', '', '',
        'klasik', 'none', 'cormorant',
        '{"bg": "#fbf3f2", "ink": "#5a2a35", "card": "#fffafa", "gold": "#d1a77a", "line": "#e6dcd2", "sage": "#94656f", "ink-2": "#5a2a35", "muted": "#94656f", "gold-c": "#f0d6bb", "primary": "#a2465c", "surface": "#fbf3f2", "gold-deep": "#7a2f43", "gold-pale": "#f0d6bb", "primary-2": "#7a2f43", "sage-mist": "#94656f", "gold-light": "#f0d6bb", "on-primary": "#ffffff", "surface-low": "#fffafa", "surface-high": "#fbf3f2"}',
        FALSE, '/img/tema/ornamen/sudut-blush.svg', 'sudut', TRUE, 0,
        'great-vibes', '', '', '', 'none', 'none', 'satu', '',
        '', '', FALSE, '', '', 'warkah',
        '{"sudut": "/img/tema/ornamen/sudut-blush.svg", "ranting": "/img/tema/ornamen/ranting-blush.svg", "untaian": "/img/tema/ornamen/untaian-blush.svg"}',
        '.t-warkah{--w-deep:#a2465c;--w-deep-2:#7a2f43;--w-paper:#fbf3f2;--w-card:#fffafa;--w-ink:#5a2a35;--w-ink-soft:#94656f;--w-on-deep:#fff1f3;--w-gold:#d1a77a;--w-gold-l:#f0d6bb;--w-petal:#f2a9ba;--w-line:color-mix(in srgb, #a2465c 16%, transparent);--t-nav-bg:color-mix(in srgb, #fffafa 92%, transparent)}'),
       ('warkah-biru', 'Warkah Biru Kaca', 'Modern', 'Modern', 'navy', 'Surat Bersegel',
        'Kartu kaca beku di atas biru malam, segel perak-biru, bunga hortensia — sejuk, modern, elegan.',
        '["Templat Khusus", "Segel Lilin", "Galeri 3D"]', 'Baru', '', '',
        'klasik', 'none', 'cormorant',
        '{"bg": "#eef3f9", "ink": "#1f3150", "card": "#f8fbff", "gold": "#8fa9cf", "line": "#e6dcd2", "sage": "#5b6f8f", "ink-2": "#1f3150", "muted": "#5b6f8f", "gold-c": "#d6e2f3", "primary": "#1f3d6b", "surface": "#eef3f9", "gold-deep": "#12264a", "gold-pale": "#d6e2f3", "primary-2": "#12264a", "sage-mist": "#5b6f8f", "gold-light": "#d6e2f3", "on-primary": "#ffffff", "surface-low": "#f8fbff", "surface-high": "#eef3f9"}',
        FALSE, '/img/tema/ornamen/sudut-biru.svg', 'sudut', TRUE, 0,
        'great-vibes', '', '', '', 'none', 'none', 'satu', '',
        '', '', FALSE, '', '', 'warkah',
        '{"sudut": "/img/tema/ornamen/sudut-biru.svg", "ranting": "/img/tema/ornamen/ranting-biru.svg", "untaian": "/img/tema/ornamen/untaian-biru.svg"}',
        '.t-warkah{--w-deep:#1f3d6b;--w-deep-2:#12264a;--w-paper:#eef3f9;--w-card:#f8fbff;--w-ink:#1f3150;--w-ink-soft:#5b6f8f;--w-on-deep:#e8f0fb;--w-gold:#8fa9cf;--w-gold-l:#d6e2f3;--w-petal:#a9c3ea;--w-line:color-mix(in srgb, #1f3d6b 16%, transparent);--t-nav-bg:color-mix(in srgb, #f8fbff 92%, transparent)}'),
       ('warkah-emas', 'Warkah Emas', 'Modern', 'Modern', 'gold', 'Surat Bersegel',
        'Krem dan emas bergaya barok: segel emas berkilau, untaian bunga keemasan, susunan acara bergaris waktu.',
        '["Templat Khusus", "Segel Lilin", "Galeri 3D"]', 'Baru', '', '',
        'klasik', 'none', 'cormorant',
        '{"bg": "#f7f1e6", "ink": "#4a3a1e", "card": "#fdfaf3", "gold": "#c7a35a", "line": "#e6dcd2", "sage": "#80694a", "ink-2": "#4a3a1e", "muted": "#80694a", "gold-c": "#ecd9a6", "primary": "#7a5520", "surface": "#f7f1e6", "gold-deep": "#523812", "gold-pale": "#ecd9a6", "primary-2": "#523812", "sage-mist": "#80694a", "gold-light": "#ecd9a6", "on-primary": "#ffffff", "surface-low": "#fdfaf3", "surface-high": "#f7f1e6"}',
        FALSE, '/img/tema/ornamen/sudut-emas.svg', 'sudut', TRUE, 0,
        'great-vibes', '', '', '', 'none', 'none', 'satu', '',
        '', '', FALSE, '', '', 'warkah',
        '{"sudut": "/img/tema/ornamen/sudut-emas.svg", "ranting": "/img/tema/ornamen/ranting-emas.svg", "untaian": "/img/tema/ornamen/untaian-emas.svg"}',
        '.t-warkah{--w-deep:#7a5520;--w-deep-2:#523812;--w-paper:#f7f1e6;--w-card:#fdfaf3;--w-ink:#4a3a1e;--w-ink-soft:#80694a;--w-on-deep:#f6ecd7;--w-gold:#c7a35a;--w-gold-l:#ecd9a6;--w-petal:#e2c482;--w-line:color-mix(in srgb, #7a5520 16%, transparent);--t-nav-bg:color-mix(in srgb, #fdfaf3 92%, transparent)}')
ON CONFLICT (slug) DO NOTHING;
