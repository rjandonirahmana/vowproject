-- ═══════════════════════════════════════════════════════════════════════════
-- 002_themes — katalog tema pindah dari kode ke database.
--
-- Satu baris = satu "kulit" di atas mesin undangan yang sama: token warna
-- (JSONB {nama-variabel-css: "#hex"}), huruf judul, tata letak sampul,
-- ornamen, dan gambar opsional. Admin menambah/mengubah tema di /admin/tema;
-- server membangkitkan /tema.css dari tabel ini (web/skin.rs).
--
-- Kolom invitations.theme tetap menyimpan slug (tanpa FK: tema yang dihapus
-- jatuh ke tampilan bawaan, undangan tidak ikut terhapus).
-- ═══════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS themes (
    slug         TEXT        PRIMARY KEY,
    name         TEXT        NOT NULL,
    category     TEXT        NOT NULL DEFAULT '',
    nuansa       TEXT        NOT NULL DEFAULT '',
    palette      TEXT        NOT NULL DEFAULT '',
    region       TEXT        NOT NULL DEFAULT '',
    description  TEXT        NOT NULL DEFAULT '',
    tags         JSONB       NOT NULL DEFAULT '[]',
    badge        TEXT        NOT NULL DEFAULT '',
    rating       TEXT        NOT NULL DEFAULT '',
    reviews      TEXT        NOT NULL DEFAULT '',
    -- klasik | gerbang | bingkai | editorial   (web/skin.rs LAYOUTS)
    layout       TEXT        NOT NULL DEFAULT 'klasik',
    -- none | daun | melati | kawung | bintang  (web/skin.rs ORNAMENTS)
    ornament     TEXT        NOT NULL DEFAULT 'none',
    font         TEXT        NOT NULL DEFAULT 'playfair',
    tokens       JSONB       NOT NULL DEFAULT '{}',
    dark         BOOLEAN     NOT NULL DEFAULT FALSE,
    -- Gambar unggahan admin (menimpa ornamen): sudut | pola | penuh
    image_url    TEXT        NOT NULL DEFAULT '',
    image_mode   TEXT        NOT NULL DEFAULT 'sudut',
    -- FALSE = tema privat (pesanan custom), tak tampil di katalog.
    listed       BOOLEAN     NOT NULL DEFAULT TRUE,
    sort_order   INT         NOT NULL DEFAULT 100,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 6 tema lama (token = nilai CSS .th-* sebelumnya) + 3 tema contoh baru.
INSERT INTO themes (slug, name, category, nuansa, palette, region, description, tags, badge, rating, reviews,
                    layout, ornament, font, tokens, dark, sort_order)
VALUES
    ('botanical-heritage', 'Botanical Heritage Romance', 'Botanical', 'Islami', 'sage', 'Botanical Heirs', 'Emblem monogram inisial melingkar berhias daun zaitun sage green dengan cap lak emas.', '["Custom Seal", "Amplop QRIS", "Buku Tamu"]'::jsonb, 'Pilihan Editor', '5.0', '1.4k', 'klasik', 'daun', 'playfair', '{"bg":"#f4fcf0","card":"#ffffff","primary":"#273f2b","on-primary":"#ffffff","gold":"#c5a059","gold-deep":"#775a19","ink":"#161d17","surface-low":"#eef6eb","surface":"#e9f0e5","surface-high":"#e3eadf","primary-2":"#3e5641","sage":"#5b7553","sage-mist":"#879878","gold-light":"#dfbe72","gold-c":"#fed488","gold-pale":"#ffdea5","ink-2":"#434842","muted":"#657065","line":"#dde5da"}'::jsonb, FALSE, 10),
    ('javanese-royal', 'Javanese Royal Heritage', 'Adat', 'Jawa', 'gold', 'Adat Jawa Luhur', 'Harmoni megah motif kawung berpadu ornamen gunungan prada emas dan aksara Jawa.', '["Doa Aksara", "Wax Seal 3D", "QR Check-In"]'::jsonb, '', '4.99', '2.1k', 'bingkai', 'kawung', 'cinzel', '{"bg":"#f7f3e8","card":"#ffffff","primary":"#3b2f1a","on-primary":"#ffffff","gold":"#c5a059","gold-deep":"#7a5712","ink":"#161d17","surface-low":"#f1ead8","surface":"#ebe2cc","surface-high":"#e3d8bd","primary-2":"#5a4524","sage":"#6b5a2e","sage-mist":"#879878","gold-light":"#dfbe72","gold-c":"#fed488","gold-pale":"#ffdea5","ink-2":"#434842","muted":"#657065","line":"#e6dcc4"}'::jsonb, FALSE, 20),
    ('sundanese-melati', 'Sundanese Melati & Champagne', 'Adat', 'Sunda', 'blush', 'Sunda Pasundan', 'Ornamen klasik Parahyangan dipadukan ronce melati suci nan harum, lengkap sawer.', '["Doa Sawer", "Live Stream", "RSVP Sync"]'::jsonb, '', '4.9', '890', 'gerbang', 'melati', 'cormorant', '{"bg":"#fbf6f2","card":"#ffffff","primary":"#5a3d3a","on-primary":"#ffffff","gold":"#c9a26b","gold-deep":"#85602c","ink":"#161d17","surface-low":"#f7eee8","surface":"#f2e5dc","surface-high":"#ecdcd0","primary-2":"#7a524d","sage":"#9a6b62","sage-mist":"#c49a8f","gold-light":"#dfbe72","gold-c":"#fed488","gold-pale":"#ffdea5","ink-2":"#434842","muted":"#657065","line":"#efe0d6"}'::jsonb, FALSE, 30),
    ('modern-serenade', 'Modern Javanese Serenade', 'Modern', 'Modern', 'terra', 'Modern Minimal', 'Gaya editorial minimalis kontemporer dipadu aksen ukiran jati Jepara yang tenang.', '["Clean Layout", "Google Maps", "Multi RSVP"]'::jsonb, '', '4.8', '950', 'editorial', 'none', 'dm-serif', '{"bg":"#f6f3ee","card":"#ffffff","primary":"#2c2a27","on-primary":"#ffffff","gold":"#b0643f","gold-deep":"#8a4a2a","ink":"#161d17","surface-low":"#f0ebe4","surface":"#e9e2d8","surface-high":"#e1d8cb","primary-2":"#4a3f35","sage":"#b0643f","sage-mist":"#c98d6d","gold-light":"#d49a78","gold-c":"#f1d3bf","gold-pale":"#f6e1d3","ink-2":"#434842","muted":"#657065","line":"#e4ddd2"}'::jsonb, FALSE, 40),
    ('kraton-charcoal', 'Royal Kraton Gold & Charcoal', 'Luxury', 'Jawa', 'gold', 'VIP Exclusive', 'Sentuhan magis beludru gelap bertabur emas prada keraton, cocok untuk malam resepsi.', '["Dark Mode Lux", "Foil Shimmer", "BCA & QRIS"]'::jsonb, '', '4.95', '1.1k', 'bingkai', 'kawung', 'cinzel', '{"bg":"#15140f","card":"#1f1c15","primary":"#e9c176","on-primary":"#1a1710","gold":"#e9c176","gold-deep":"#e9c176","ink":"#f3ead4","surface-low":"#1c1a14","surface":"#232018","surface-high":"#2b271d","primary-2":"#c5a059","sage":"#d9b36a","sage-mist":"#a38a57","gold-light":"#dfbe72","gold-c":"#fed488","gold-pale":"#ffdea5","ink-2":"#d8ccb0","muted":"#a79d86","line":"#353025"}'::jsonb, TRUE, 50),
    ('midnight-celestial', 'Midnight Celestial Star', 'Luxury', 'Modern', 'navy', 'Evening Gala', 'Langit malam biru tua bertabur bintang emas untuk resepsi gala yang intim dan sakral.', '["Dark Mode", "Audio Jazz", "RSVP Sesi"]'::jsonb, '', '4.9', '640', 'gerbang', 'bintang', 'cormorant', '{"bg":"#0f1526","card":"#172036","primary":"#e9c176","on-primary":"#1a1710","gold":"#e9c176","gold-deep":"#e9c176","ink":"#eef0f7","surface-low":"#141c30","surface":"#1a2339","surface-high":"#212b44","primary-2":"#c5a059","sage":"#d9b36a","sage-mist":"#a38a57","gold-light":"#dfbe72","gold-c":"#fed488","gold-pale":"#ffdea5","ink-2":"#cfd4e4","muted":"#98a0b8","line":"#28324c"}'::jsonb, TRUE, 60),
    ('minang-songket', 'Minang Songket Merah Marawa', 'Adat', 'Minang', 'terra', 'Ranah Minang', 'Merah marun songket dan kilau benang emas, dibingkai ukiran rumah gadang.', '["Pantun Adat", "Amplop Digital", "QR Check-In"]'::jsonb, 'Baru', '', '', 'bingkai', 'kawung', 'cinzel', '{"bg":"#fbf4ee","card":"#fffaf6","primary":"#6e1a1a","on-primary":"#ffffff","gold":"#c9982f","gold-deep":"#8a5d0f","ink":"#161d17","surface-low":"#f6ebe2","surface":"#f0e0d4","surface-high":"#e9d4c5","primary-2":"#8c2a22","sage":"#a0452e","sage-mist":"#c98a70","gold-light":"#dfbe72","gold-c":"#fed488","gold-pale":"#ffdea5","ink-2":"#434842","muted":"#657065","line":"#ecd9cb"}'::jsonb, FALSE, 70),
    ('bali-frangipani', 'Bali Frangipani Serenity', 'Botanical', 'Bali', 'sage', 'Pulau Dewata', 'Kamboja putih, hijau tosca laguna, dan gapura candi bentar yang teduh.', '["Galeri Foto", "Peta Lokasi", "RSVP Sesi"]'::jsonb, 'Baru', '', '', 'gerbang', 'melati', 'marcellus', '{"bg":"#f2f8f6","card":"#ffffff","primary":"#16504a","on-primary":"#ffffff","gold":"#c7a262","gold-deep":"#7d5a1c","ink":"#161d17","surface-low":"#e9f3f0","surface":"#e0ede9","surface-high":"#d5e6e1","primary-2":"#236a61","sage":"#3d8a7d","sage-mist":"#7fb3a8","gold-light":"#dfbe72","gold-c":"#fed488","gold-pale":"#ffdea5","ink-2":"#434842","muted":"#657065","line":"#d6e6e1"}'::jsonb, FALSE, 80),
    ('rustic-kraft', 'Rustic Kraft Garden', 'Modern', 'Modern', 'terra', 'Intimate Garden', 'Nuansa kertas kraft, bunga liar kering, dan tulisan hangat untuk pesta kebun.', '["Love Story", "Musik Latar", "Buku Tamu"]'::jsonb, 'Baru', '', '', 'klasik', 'daun', 'lora', '{"bg":"#f5eee3","card":"#fffaf2","primary":"#4a3524","on-primary":"#ffffff","gold":"#b8894a","gold-deep":"#7a5526","ink":"#161d17","surface-low":"#efe5d6","surface":"#e8dcc9","surface-high":"#dfd0b9","primary-2":"#634a33","sage":"#7a8456","sage-mist":"#a5ab84","gold-light":"#dfbe72","gold-c":"#fed488","gold-pale":"#ffdea5","ink-2":"#434842","muted":"#657065","line":"#e3d6c2"}'::jsonb, FALSE, 90)
ON CONFLICT (slug) DO NOTHING;
