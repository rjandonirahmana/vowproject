-- ═══════════════════════════════════════════════════════════════════════════
-- 042_tema_alam — 6 tema bernuansa ALAM, masing-masing memamerkan satu
-- gerbang baru dengan gerak buka & TUTUP sendiri (CSS di src/web/gerak/,
-- animasinya otomatis tersinkron ke tabel animations saat server mulai):
--   curug-asri      air-terjun      tirai air tersibak / air tercurah
--   fajar-rembulan  fajar-rembulan  matahari terbit / senja & bulan naik
--   teratai-mekar   teratai-mekar   kelopak mekar / menguncup
--   taman-kupu      kupu-kupu       kawanan mengangkat / menurunkan sampul
--   samudra-biru    ombak-laut      ombak menyapu / pasang lalu surut
--   langit-awan     awan-berarak    menembus awan / awan berkumpul
-- Tanpa aset gambar (bentuk dari gradien & SVG data:). Idempoten.
-- ═══════════════════════════════════════════════════════════════════════════

INSERT INTO themes (slug, name, category, nuansa, palette, region, description, tags, badge, layout, ornament, font, tokens, dark, sort_order, script_font, bg_image, frame_image, card_deco, float_deco, open_anim, page_mode, scroll_anim, image_url, image_mode, gerak_judul, gerak_foto, ken_burns, listed) VALUES
('curug-asri', 'Curug Asri', 'Modern', 'Alam', 'sage', 'Air Terjun', 'Sampul tersembunyi di balik tirai air terjun yang jatuh tanpa henti; saat dibuka tirai air tersibak, saat ditutup air kembali tercurah dari atas.', '["Air Terjun", "Alam", "Hijau Toska", "Gerbang Bergerak"]'::jsonb, 'Baru', 'klasik', 'none', 'cormorant',
 '{"bg": "#f2f8f6", "card": "#ffffff", "primary": "#1f5f5b", "on-primary": "#ffffff", "gold": "#c9a45c", "gold-deep": "#7a5f22", "ink": "#12302e", "muted": "#5d7472"}'::jsonb, FALSE, 20, 'great-vibes', '', '', '', 'daun-gugur', 'air-terjun', 'satu', 'anggun', '', 'sudut', 'ikut', 'ikut', TRUE, TRUE),
('fajar-rembulan', 'Fajar & Rembulan', 'Modern', 'Alam', 'navy', 'Langit', 'Sampul langit malam berbulan sabit; dibuka matahari terbit dari balik bukit, ditutup senja memerah lalu bulan naik kembali.', '["Matahari Terbit", "Bulan", "Bintang", "Gerbang Bergerak"]'::jsonb, 'Baru', 'klasik', 'none', 'playfair',
 '{"bg": "#f7f3ea", "card": "#fffdf8", "primary": "#22305a", "on-primary": "#ffffff", "gold": "#d8a94a", "gold-deep": "#7d5c1d", "ink": "#1b1f33", "muted": "#626679"}'::jsonb, FALSE, 21, 'pinyon', '', '', '', 'bintang', 'fajar-rembulan', 'satu', 'kosmik', '', 'sudut', 'ikut', 'ikut', TRUE, TRUE),
('teratai-mekar', 'Teratai Mekar', 'Modern', 'Alam', 'blush', 'Bunga', 'Sekuntum teratai raksasa di belakang sampul; dibuka kelopaknya mekar merekah, ditutup menguncup kembali.', '["Teratai", "Bunga", "Merah Muda", "Gerbang Bergerak"]'::jsonb, 'Baru', 'klasik', 'none', 'marcellus',
 '{"bg": "#fdf4f4", "card": "#ffffff", "primary": "#b34f6c", "on-primary": "#ffffff", "gold": "#d6a75c", "gold-deep": "#84602a", "ink": "#3d1b26", "muted": "#7d5c66"}'::jsonb, FALSE, 22, 'parisienne', '', '', '', 'kelopak', 'teratai-mekar', 'satu', 'mekar', '', 'sudut', 'ikut', 'ikut', TRUE, TRUE),
('taman-kupu', 'Taman Kupu-kupu', 'Modern', 'Alam', 'sage', 'Satwa', 'Kupu-kupu hinggap di padang bunga; dibuka kawanan mengangkat kartu sampul ke langit, ditutup menurunkannya kembali.', '["Kupu-kupu", "Satwa", "Padang Bunga", "Gerbang Bergerak"]'::jsonb, 'Baru', 'klasik', 'none', 'lora',
 '{"bg": "#f6f7f0", "card": "#ffffff", "primary": "#6a5a9b", "on-primary": "#ffffff", "gold": "#d9b45a", "gold-deep": "#7c5f1e", "ink": "#2a2440", "muted": "#6b6680", "sage": "#7c9a6a"}'::jsonb, FALSE, 23, 'allura', '', '', '', 'kupu', 'kupu-kupu', 'satu', 'geser', '', 'sudut', 'ikut', 'ikut', TRUE, TRUE),
('samudra-biru', 'Samudra Biru', 'Modern', 'Alam', 'navy', 'Laut', 'Sampul di tepi pantai berombak; dibuka ombak besar menyapu layar, ditutup pasang naik lalu surut meninggalkan sampul.', '["Laut", "Ombak", "Pantai", "Gerbang Bergerak"]'::jsonb, 'Baru', 'klasik', 'none', 'dm-serif',
 '{"bg": "#f3f8fb", "card": "#ffffff", "primary": "#1d5c86", "on-primary": "#ffffff", "gold": "#d4a85a", "gold-deep": "#7a5a20", "ink": "#0f2a3d", "muted": "#5a6f7e"}'::jsonb, FALSE, 24, 'great-vibes', '', '', '', 'kilau-emas', 'ombak-laut', 'satu', 'ombak', '', 'sudut', 'ikut', 'ikut', TRUE, TRUE),
('langit-awan', 'Langit Berawan', 'Modern', 'Alam', 'navy', 'Langit', 'Kartu sampul melayang di antara awan; dibuka seperti terbang menembus awan, ditutup awan berkumpul lalu berarak kembali.', '["Awan", "Langit", "Biru Lembut", "Gerbang Bergerak"]'::jsonb, 'Baru', 'klasik', 'none', 'cormorant',
 '{"bg": "#f5f8fc", "card": "#ffffff", "primary": "#5b7fb0", "on-primary": "#ffffff", "gold": "#d9b46a", "gold-deep": "#80622a", "ink": "#1e2c44", "muted": "#64708a"}'::jsonb, FALSE, 25, 'alex-brush', '', '', '', 'burung', 'awan-berarak', 'satu', 'anggun', '', 'sudut', 'ikut', 'ikut', TRUE, TRUE)
ON CONFLICT (slug) DO NOTHING;
