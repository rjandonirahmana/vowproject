-- ═══════════════════════════════════════════════════════════════════════════
-- 046_tema_ulang_tahun — 4 tema UNDANGAN ULANG TAHUN (templat `pesta`).
--
-- Kategori "Ulang Tahun" = jenis acara ulang tahun (ThemeInfo::ulang_tahun):
-- etalase beranda menaruhnya di panel kiri "Ulang Tahun", tanpa panggung 3D
-- pernikahan, dan undangan demo yang dibuka dengan tema ini memakai data
-- pesta contoh (Aira, Ulang Tahun ke-7) — bukan Yona & Doni.
-- Templat `pesta` (templat/pesta/) di-seed otomatis saat server start.
-- Kulit = variabel --p-* (template_css); token kolom `tokens` menjaga kartu
-- katalog & /kelola ikut berwarna. Baris yang SUDAH ada tidak ditimpa.
-- ═══════════════════════════════════════════════════════════════════════════

INSERT INTO themes (slug, name, category, nuansa, palette, region, description, tags, badge, rating, reviews,
                    layout, ornament, font, tokens, dark, image_url, image_mode, listed, sort_order,
                    script_font, bg_image, frame_image, card_deco, float_deco, open_anim, page_mode, scroll_anim,
                    gerak_judul, gerak_foto, ken_burns, bg_video, open_video, template, template_assets, template_css)
VALUES
       ('pesta-balon', 'Pesta Balon Pastel', 'Ulang Tahun', 'Anak', 'blush', 'Ulang Tahun',
        'Balon pastel melayang, konfeti berjatuhan, kue dengan lilin menyala, dan kotak kado yang meletup saat diketuk — manis untuk ulang tahun anak.',
        '["Ulang Tahun", "Anak", "Balon", "Konfeti"]', 'Baru', '', '',
        'klasik', 'none', 'playfair',
        '{"bg": "#fff6f2", "card": "#ffffff", "primary": "#e2557a", "on-primary": "#ffffff", "gold": "#ffc857", "gold-deep": "#b8476a", "gold-pale": "#ffe9ef", "ink": "#3b2a33", "muted": "#7d6570", "line": "#f3d9e0"}',
        FALSE, '', 'sudut', TRUE, 0,
        '', '', '', '', 'none', 'none', 'satu', '',
        '', '', FALSE, '', '', 'pesta', '{}',
        '.t-pesta{--p-bg:#fff6f2;--p-bg2:#ffe9ef;--p-deep:#e2557a;--p-a1:#ff8fab;--p-a2:#ffc857;--p-a3:#7fd1c7;--p-ink:#3b2a33;--p-soft:#7d6570;--p-line:#f3d9e0}'),
       ('pesta-angkasa', 'Bintang Angkasa', 'Ulang Tahun', 'Anak', 'navy', 'Ulang Tahun',
        'Biru langit cerah dengan balon kuning bintang & biru muda — ceria dan rapi, cocok untuk ulang tahun anak laki-laki maupun perempuan.',
        '["Ulang Tahun", "Anak", "Biru", "Bintang"]', 'Baru', '', '',
        'klasik', 'none', 'playfair',
        '{"bg": "#eef3ff", "card": "#ffffff", "primary": "#3346a8", "on-primary": "#ffffff", "gold": "#ffcf4a", "gold-deep": "#2a3a8c", "gold-pale": "#dfe8ff", "ink": "#1b2350", "muted": "#5b6385", "line": "#d6defa"}',
        FALSE, '', 'sudut', TRUE, 0,
        '', '', '', '', 'none', 'none', 'satu', '',
        '', '', FALSE, '', '', 'pesta', '{}',
        '.t-pesta{--p-bg:#eef3ff;--p-bg2:#dfe8ff;--p-deep:#3346a8;--p-a1:#ffcf4a;--p-a2:#6ec6ff;--p-a3:#ff7aa8;--p-ink:#1b2350;--p-soft:#5b6385;--p-line:#d6defa}'),
       ('pesta-manis', 'Sweet Seventeen', 'Ulang Tahun', 'Remaja', 'blush', 'Ulang Tahun',
        'Lilac lembut berpadu merah muda & emas muda — anggun untuk sweet seventeen dan pesta ulang tahun remaja.',
        '["Ulang Tahun", "Sweet 17", "Lilac", "Remaja"]', 'Baru', '', '',
        'klasik', 'none', 'playfair',
        '{"bg": "#f7f1ff", "card": "#ffffff", "primary": "#8a4fd1", "on-primary": "#ffffff", "gold": "#f6d28b", "gold-deep": "#6b3aa8", "gold-pale": "#efe3ff", "ink": "#33224d", "muted": "#6e5f86", "line": "#e6d9fa"}',
        FALSE, '', 'sudut', TRUE, 0,
        '', '', '', '', 'none', 'none', 'satu', '',
        '', '', FALSE, '', '', 'pesta', '{}',
        '.t-pesta{--p-bg:#f7f1ff;--p-bg2:#efe3ff;--p-deep:#8a4fd1;--p-a1:#c9a7ff;--p-a2:#ffb3d1;--p-a3:#f6d28b;--p-ink:#33224d;--p-soft:#6e5f86;--p-line:#e6d9fa}'),
       ('pesta-pelangi', 'Ceria Pelangi', 'Ulang Tahun', 'Anak', 'terra', 'Ulang Tahun',
        'Kuning hangat dengan balon jingga, biru, dan merah muda menyala — riang dan penuh warna untuk pesta yang meriah.',
        '["Ulang Tahun", "Anak", "Pelangi", "Meriah"]', 'Baru', '', '',
        'klasik', 'none', 'playfair',
        '{"bg": "#fffbea", "card": "#ffffff", "primary": "#ff6b35", "on-primary": "#ffffff", "gold": "#4cc9f0", "gold-deep": "#c4471d", "gold-pale": "#fff1c7", "ink": "#2b2b2b", "muted": "#6b6458", "line": "#f6e7b4"}',
        FALSE, '', 'sudut', TRUE, 0,
        '', '', '', '', 'none', 'none', 'satu', '',
        '', '', FALSE, '', '', 'pesta', '{}',
        '.t-pesta{--p-bg:#fffbea;--p-bg2:#fff1c7;--p-deep:#ff6b35;--p-a1:#4cc9f0;--p-a2:#f72585;--p-a3:#80ed99;--p-ink:#2b2b2b;--p-soft:#6b6458;--p-line:#f6e7b4}')
ON CONFLICT (slug) DO NOTHING;
