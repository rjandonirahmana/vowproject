-- ═══════════════════════════════════════════════════════════════════════════
-- 005_tampilan — undangan yang lebih "hidup". Idempoten.
--
--   invitations: cover_photo (foto berdua di sampul), love_story (JSONB
--                [{year,title,text}]), live_url (tautan siaran langsung).
--   themes:      script_font (huruf kaligrafi judul), bg_image (ilustrasi
--                latar penuh), frame_image (bingkai foto), card_deco (hiasan
--                bunga di atas kartu), float_deco (kelopak/kupu/bintang
--                melayang), open_anim (animasi pembuka), page_mode
--                (tab | satu = satu halaman panjang).
--
-- Kode membaca baris lewat to_jsonb(), jadi kolom yang BELUM ada tidak
-- membuat halaman error — fitur barunya saja yang belum aktif.
-- ═══════════════════════════════════════════════════════════════════════════

ALTER TABLE invitations ADD COLUMN IF NOT EXISTS cover_photo TEXT  NOT NULL DEFAULT '';
ALTER TABLE invitations ADD COLUMN IF NOT EXISTS love_story  JSONB NOT NULL DEFAULT '[]';
ALTER TABLE invitations ADD COLUMN IF NOT EXISTS live_url    TEXT  NOT NULL DEFAULT '';

ALTER TABLE themes ADD COLUMN IF NOT EXISTS script_font TEXT NOT NULL DEFAULT '';
ALTER TABLE themes ADD COLUMN IF NOT EXISTS bg_image    TEXT NOT NULL DEFAULT '';
ALTER TABLE themes ADD COLUMN IF NOT EXISTS frame_image TEXT NOT NULL DEFAULT '';
ALTER TABLE themes ADD COLUMN IF NOT EXISTS card_deco   TEXT NOT NULL DEFAULT '';
ALTER TABLE themes ADD COLUMN IF NOT EXISTS float_deco  TEXT NOT NULL DEFAULT 'none';
ALTER TABLE themes ADD COLUMN IF NOT EXISTS open_anim   TEXT NOT NULL DEFAULT 'none';
ALTER TABLE themes ADD COLUMN IF NOT EXISTS page_mode   TEXT NOT NULL DEFAULT 'tab';

-- Tema unggulan contoh: memakai semua bahan baru (aset SVG buatan sendiri).
INSERT INTO themes (slug, name, category, nuansa, palette, region, description, tags, badge,
                    layout, ornament, font, tokens, dark, sort_order,
                    script_font, bg_image, frame_image, card_deco, float_deco, open_anim, page_mode)
VALUES ('lily-garden', 'Lily Garden Serenade', 'Botanical', 'Islami', 'sage', 'Motion Garden',
        'Taman lili cat air, kaligrafi anggun, tirai pembuka, dan kelopak yang berguguran perlahan.',
        '["Animasi Pembuka","Foto Sampul","Love Story"]'::jsonb, 'Baru • Motion',
        'klasik', 'none', 'cormorant',
        '{"bg":"#f6f8ef","card":"#fffdf7","primary":"#3f5a3c","on-primary":"#ffffff","gold":"#c5a059","gold-deep":"#7a5a1c","ink":"#1d241c","surface-low":"#eef3e6","surface":"#e6eddc","surface-high":"#dce6d1","primary-2":"#56744f","sage":"#6f8a62","sage-mist":"#a3b594","gold-light":"#dfc185","gold-c":"#f4dfae","gold-pale":"#f8ebcc","ink-2":"#434a41","muted":"#687264","line":"#dde5d3"}'::jsonb,
        FALSE, 5,
        'pinyon', '/img/tema/lily-bg.svg', '/img/tema/lily-frame.svg', '/img/tema/lily-garland.svg', 'kelopak', 'tirai', 'satu')
ON CONFLICT (slug) DO NOTHING;

-- Undangan demo: contoh foto sampul, kisah, galeri, siaran langsung.
UPDATE invitations SET
    cover_photo = '/img/layanan/mua-sesudah.jpg#pos=50,30,1.00',
    love_story  = '[{"year":"2019","title":"Pertama Bertemu","text":"Berkenalan di sebuah pameran seni di Jakarta — obrolan singkat tentang lukisan yang berlanjut hingga larut."},
                    {"year":"2024","title":"Lamaran","text":"Dengan restu kedua keluarga, Radit melamar Dita di Yogyakarta, tempat keluarga besar berkumpul."},
                    {"year":"2026","title":"Hari Bahagia","text":"Kami memohon doa restu untuk mengikat janji suci dan memulai perjalanan baru bersama."}]'::jsonb,
    live_url    = 'https://www.youtube.com/',
    gallery     = '["/img/layanan/mua-solo-putri.jpg","/img/layanan/dekor-galeri-altar.jpg","/img/layanan/mua-modern.jpg","/img/layanan/dekor-galeri-meja.jpg","/img/layanan/mua-melati.jpg","/img/layanan/dekor-galeri-gebyok.jpg"]'::jsonb
WHERE is_demo AND cover_photo = '';
