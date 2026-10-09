-- ═══════════════════════════════════════════════════════════════════════════
-- 035_rupa_suku — rupa (bentuk tiap bagian) untuk 25 tema suku dari 034.
-- DIBANGKITKAN scripts/rupa/build.py bersama 033 (jarak dihitung terhadap
-- SEMUA tema) — jangan disunting tangan. Aman dijalankan ulang.
-- WAJIB setelah 033 & 034.
-- ═══════════════════════════════════════════════════════════════════════════

-- ── Rupa per tema (hanya bila admin belum mengaturnya) ──
UPDATE themes SET rupa = '{"sampul": "medali", "judul": "bingkai", "mempelai": "potret", "kisah": "angka", "kartu": "tegas", "pemisah": "wajik"}'::jsonb WHERE slug = 'kerawang-gayo' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "penuh", "judul": "tumpal", "mempelai": "medali", "kisah": "polaroid", "acara": "tiket"}'::jsonb WHERE slug = 'omo-sebua-nias' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "jendela", "judul": "samping", "kisah": "angka", "acara": "tiket", "galeri": "kolom", "kartu": "bertepi", "pemisah": "ganda"}'::jsonb WHERE slug = 'siwaluh-jabu-karo' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "wajik", "judul": "bingkai", "mempelai": "potret", "kisah": "kartu", "acara": "tegas", "kartu": "kaca", "pemisah": "ganda"}'::jsonb WHERE slug = 'gordang-mandailing' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "jendela", "judul": "samping", "mempelai": "zigzag", "acara": "tiket", "galeri": "polaroid"}'::jsonb WHERE slug = 'titi-mentawai' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "gunungan", "judul": "tumpal", "mempelai": "medali", "kisah": "polaroid", "galeri": "polaroid", "kartu": "potong", "pemisah": "tumpal"}'::jsonb WHERE slug = 'istana-maimun-deli' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "medali", "judul": "tumpal", "mempelai": "sejajar", "acara": "kalender", "kartu": "bertepi", "pemisah": "ombak"}'::jsonb WHERE slug = 'tapis-saibatin' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"judul": "pita", "mempelai": "panel", "kisah": "polaroid", "acara": "lontar", "galeri": "kolom", "pemisah": "wajik"}'::jsonb WHERE slug = 'gandrung-osing' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "gunungan", "mempelai": "zigzag", "kisah": "selang", "acara": "tiket", "galeri": "mozaik", "kartu": "cetak"}'::jsonb WHERE slug = 'karapan-madura' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "penuh", "judul": "samping", "kisah": "angka", "acara": "lontar", "galeri": "polaroid", "pemisah": "titik"}'::jsonb WHERE slug = 'jlamprang-pekalongan' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "jendela", "mempelai": "panel", "kisah": "angka", "galeri": "kolom", "kartu": "potong"}'::jsonb WHERE slug = 'lurik-klaten' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "gunungan", "mempelai": "sejajar", "kisah": "polaroid", "acara": "lontar", "galeri": "mozaik", "kartu": "bertepi", "pemisah": "ombak"}'::jsonb WHERE slug = 'tenun-baduy' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"mempelai": "medali", "kisah": "tengah", "acara": "lontar", "galeri": "polaroid", "kartu": "cetak", "pemisah": "titik"}'::jsonb WHERE slug = 'geringsing-tenganan' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "atap", "judul": "pita", "acara": "kalender", "galeri": "kolom", "kartu": "cetak", "pemisah": "ganda"}'::jsonb WHERE slug = 'tembe-mbojo' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "wajik", "judul": "tumpal", "mempelai": "zigzag", "kisah": "kartu", "acara": "tiket", "galeri": "mozaik", "kartu": "bertepi", "pemisah": "titik"}'::jsonb WHERE slug = 'songke-manggarai' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "wajik", "judul": "tumpal", "kisah": "polaroid", "acara": "tegas", "galeri": "film", "kartu": "tegas", "pemisah": "tumpal"}'::jsonb WHERE slug = 'ikat-timor' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "gunungan", "judul": "kapital", "mempelai": "panel", "kisah": "selang", "acara": "tegas", "galeri": "kolom", "kartu": "potong", "pemisah": "titik"}'::jsonb WHERE slug = 'pua-kumbu-iban' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "jendela", "judul": "kapital", "mempelai": "medali", "kisah": "selang", "kartu": "kaca", "pemisah": "wajik"}'::jsonb WHERE slug = 'ulap-doyo-kutai' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "atap", "judul": "pita", "mempelai": "potret", "acara": "tegas", "kartu": "bertepi", "pemisah": "wajik"}'::jsonb WHERE slug = 'baju-bodo-makassar' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "atap", "judul": "kapital", "mempelai": "zigzag", "acara": "lontar", "galeri": "film", "kartu": "kaca", "pemisah": "ombak"}'::jsonb WHERE slug = 'saoraja-bone' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "kubah", "judul": "pita", "mempelai": "medali", "kisah": "kartu", "acara": "kalender", "galeri": "polaroid", "kartu": "tegas", "pemisah": "ombak"}'::jsonb WHERE slug = 'kofo-sangihe' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "kubah", "mempelai": "sejajar", "kisah": "selang", "acara": "linimasa", "galeri": "film", "kartu": "kaca", "pemisah": "ombak"}'::jsonb WHERE slug = 'ain-ni-ain-kei' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "penuh", "judul": "bingkai", "mempelai": "sejajar", "kisah": "tengah", "acara": "tiket", "galeri": "film", "kartu": "cetak"}'::jsonb WHERE slug = 'tenun-tanimbar' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"judul": "pita", "kisah": "tengah", "acara": "kalender", "galeri": "mozaik", "kartu": "tegas", "pemisah": "tumpal"}'::jsonb WHERE slug = 'wor-biak' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "lingkaran", "judul": "kapital", "mempelai": "sejajar", "kisah": "angka", "galeri": "polaroid", "kartu": "tegas", "pemisah": "tumpal"}'::jsonb WHERE slug = 'kulit-kayu-sentani' AND rupa = '{}'::jsonb;

-- ── Gerak gulir kembar diganti (pasangan buka+gulir kini unik) ──
