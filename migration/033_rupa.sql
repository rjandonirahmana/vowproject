-- ═══════════════════════════════════════════════════════════════════════════
-- 033_rupa — RUPA: varian STRUKTUR per bagian undangan (web/rupa.rs).
--
-- Audit piksel 9 Okt 2026: 50 tema berbagi satu kerangka — tinggi halaman
-- 9906/9999 px untuk hampir semua tema, 603 dari 1275 pasangan "struktur
-- > 0,85 identik" (granit-belitung ~ raja-ampat 0,996). Kini tiap tema memilih
-- satu varian per bagian (sampul, judul, mempelai, acara, galeri, kartu,
-- pemisah) dan tiap pasangan tema berbeda di ≥ 4 bagian.
--
-- Bagian data DIBANGKITKAN scripts/rupa/build.py — jangan disunting tangan.
-- Aman dijalankan ulang: rupa hanya ditulis bila masih '{}' (belum diatur
-- admin), gerak hanya bila tema belum dikunci admin (motion_locked).
-- WAJIB setelah 025 (kolom motion_locked) & 031.
-- ═══════════════════════════════════════════════════════════════════════════
ALTER TABLE themes ADD COLUMN IF NOT EXISTS rupa JSONB NOT NULL DEFAULT '{}'::jsonb;

-- ── Rupa per tema (hanya bila admin belum mengaturnya) ──
UPDATE themes SET rupa = '{"sampul": "gunungan", "judul": "kapital", "mempelai": "medali", "kisah": "angka", "acara": "lontar", "galeri": "mozaik", "kartu": "potong", "pemisah": "wajik"}'::jsonb WHERE slug = 'javanese-royal' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "kubah", "judul": "bingkai", "mempelai": "panel", "kisah": "tengah", "acara": "tegas", "galeri": "polaroid", "kartu": "kaca", "pemisah": "wajik"}'::jsonb WHERE slug = 'botanical-heritage' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "penuh", "judul": "samping", "mempelai": "potret", "kisah": "polaroid", "acara": "tiket", "galeri": "film", "kartu": "kaca", "pemisah": "ganda"}'::jsonb WHERE slug = 'midnight-celestial' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "atap", "judul": "tumpal", "mempelai": "sejajar", "kisah": "selang", "acara": "tegas", "galeri": "kolom", "kartu": "bertepi", "pemisah": "tumpal"}'::jsonb WHERE slug = 'minang-songket' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "medali", "judul": "bingkai", "mempelai": "zigzag", "kisah": "polaroid", "acara": "lontar", "galeri": "polaroid", "kartu": "potong", "pemisah": "ombak"}'::jsonb WHERE slug = 'bali-frangipani' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "gunungan", "judul": "pita", "mempelai": "potret", "kisah": "selang", "acara": "kalender", "galeri": "film", "kartu": "cetak", "pemisah": "ombak"}'::jsonb WHERE slug = 'mega-mendung-cirebon' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "atap", "judul": "kapital", "mempelai": "zigzag", "kisah": "angka", "acara": "tegas", "galeri": "mozaik", "kartu": "tegas", "pemisah": "ganda"}'::jsonb WHERE slug = 'gorga-rumah-bolon' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "wajik", "judul": "kapital", "mempelai": "sejajar", "kisah": "kartu", "acara": "lontar", "galeri": "kolom", "kartu": "bertepi", "pemisah": "wajik"}'::jsonb WHERE slug = 'phinisi-layar-tujuh' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "lingkaran", "judul": "kapital", "mempelai": "medali", "kisah": "tengah", "acara": "linimasa", "galeri": "film", "kartu": "kaca", "pemisah": "titik"}'::jsonb WHERE slug = 'burung-enggang-borneo' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "kubah", "judul": "bingkai", "mempelai": "medali", "kisah": "angka", "acara": "tegas", "galeri": "kolom", "kartu": "potong", "pemisah": "ganda"}'::jsonb WHERE slug = 'aceh-pintu-seulanga' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "atap", "judul": "bingkai", "mempelai": "panel", "kisah": "kartu", "acara": "tiket", "galeri": "polaroid", "kartu": "tegas", "pemisah": "tumpal"}'::jsonb WHERE slug = 'ulos-ragi-hotang' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "kubah", "judul": "tumpal", "mempelai": "potret", "kisah": "polaroid", "acara": "lontar", "galeri": "mozaik", "kartu": "kaca", "pemisah": "tumpal"}'::jsonb WHERE slug = 'gonjong-rumah-gadang' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "kubah", "judul": "tumpal", "mempelai": "zigzag", "kisah": "selang", "acara": "lontar", "galeri": "polaroid", "kartu": "bertepi", "pemisah": "wajik"}'::jsonb WHERE slug = 'selembayung-riau' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "wajik", "judul": "tumpal", "mempelai": "panel", "kisah": "tengah", "galeri": "kolom", "kartu": "bertepi", "pemisah": "ombak"}'::jsonb WHERE slug = 'lancang-kuning-kepri' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "jendela", "judul": "samping", "mempelai": "sejajar", "kisah": "selang", "acara": "lontar", "galeri": "film", "kartu": "potong", "pemisah": "tumpal"}'::jsonb WHERE slug = 'durian-pecah-jambi' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "medali", "judul": "tumpal", "mempelai": "medali", "kisah": "kartu", "acara": "linimasa", "kartu": "kaca", "pemisah": "tumpal"}'::jsonb WHERE slug = 'songket-limas-palembang' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "kubah", "judul": "pita", "kisah": "selang", "galeri": "mozaik", "kartu": "potong", "pemisah": "ombak"}'::jsonb WHERE slug = 'granit-belitung' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "wajik", "judul": "bingkai", "mempelai": "potret", "kisah": "selang", "acara": "kalender", "galeri": "mozaik", "kartu": "bertepi", "pemisah": "tumpal"}'::jsonb WHERE slug = 'rafflesia-bengkulu' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "wajik", "judul": "pita", "mempelai": "panel", "kisah": "kartu", "acara": "linimasa", "galeri": "film", "kartu": "bertepi", "pemisah": "tumpal"}'::jsonb WHERE slug = 'siger-tapis-lampung' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "jendela", "judul": "pita", "mempelai": "sejajar", "kisah": "polaroid", "acara": "tiket", "galeri": "polaroid", "kartu": "cetak", "pemisah": "titik"}'::jsonb WHERE slug = 'ondel-ondel-betawi' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "lingkaran", "judul": "pita", "mempelai": "zigzag", "kisah": "polaroid", "acara": "kalender", "galeri": "kolom", "kartu": "tegas", "pemisah": "titik"}'::jsonb WHERE slug = 'kujang-pasundan' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "gunungan", "mempelai": "panel", "kisah": "angka", "acara": "kalender", "pemisah": "titik"}'::jsonb WHERE slug = 'menara-banten' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"mempelai": "sejajar", "kisah": "kartu", "acara": "kalender", "galeri": "polaroid", "kartu": "potong", "pemisah": "wajik"}'::jsonb WHERE slug = 'joglo-sidomukti' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "gunungan", "mempelai": "potret", "kisah": "kartu", "acara": "lontar", "galeri": "kolom", "kartu": "kaca", "pemisah": "titik"}'::jsonb WHERE slug = 'parang-keraton-yogya' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "medali", "mempelai": "zigzag", "kisah": "angka", "acara": "tiket", "galeri": "film", "pemisah": "wajik"}'::jsonb WHERE slug = 'gapura-majapahit' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "medali", "judul": "bingkai", "kisah": "tengah", "galeri": "mozaik", "kartu": "kaca", "pemisah": "titik"}'::jsonb WHERE slug = 'penjor-pura-bali' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "lingkaran", "judul": "tumpal", "mempelai": "medali", "acara": "lontar", "galeri": "film", "kartu": "bertepi", "pemisah": "tumpal"}'::jsonb WHERE slug = 'tenun-sasak-lombok' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "wajik", "judul": "kapital", "mempelai": "panel", "kisah": "tengah", "acara": "tiket", "kartu": "kaca", "pemisah": "tumpal"}'::jsonb WHERE slug = 'ikat-sumba-sasando' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "lingkaran", "judul": "samping", "mempelai": "potret", "acara": "tegas", "galeri": "polaroid", "kartu": "tegas", "pemisah": "titik"}'::jsonb WHERE slug = 'radakng-kanayatn' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "lingkaran", "judul": "samping", "mempelai": "sejajar", "kisah": "polaroid", "acara": "linimasa", "galeri": "mozaik", "kartu": "potong", "pemisah": "ganda"}'::jsonb WHERE slug = 'batang-garing-ngaju' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "jendela", "judul": "samping", "mempelai": "medali", "kisah": "tengah", "acara": "kalender", "galeri": "kolom", "kartu": "kaca", "pemisah": "ombak"}'::jsonb WHERE slug = 'sasirangan-banjar' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "wajik", "judul": "kapital", "mempelai": "zigzag", "kisah": "angka", "acara": "linimasa", "kartu": "potong"}'::jsonb WHERE slug = 'lamin-kenyah' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "kubah", "judul": "bingkai", "kisah": "tengah", "acara": "lontar", "galeri": "kolom", "kartu": "cetak", "pemisah": "tumpal"}'::jsonb WHERE slug = 'kayan-tidung' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "jendela", "judul": "samping", "mempelai": "panel", "kisah": "kartu", "acara": "kalender", "galeri": "polaroid", "kartu": "cetak", "pemisah": "ganda"}'::jsonb WHERE slug = 'walewangko-minahasa' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "penuh", "judul": "samping", "mempelai": "sejajar", "acara": "kalender", "galeri": "mozaik", "kartu": "kaca", "pemisah": "wajik"}'::jsonb WHERE slug = 'karawo-gorontalo' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "atap", "judul": "kapital", "mempelai": "potret", "kisah": "polaroid", "acara": "linimasa", "kartu": "cetak", "pemisah": "ganda"}'::jsonb WHERE slug = 'souraja-kaili' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "penuh", "judul": "samping", "mempelai": "medali", "kisah": "angka", "acara": "linimasa", "galeri": "film", "kartu": "tegas", "pemisah": "ombak"}'::jsonb WHERE slug = 'sandeq-mandar' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "atap", "judul": "bingkai", "kisah": "angka", "acara": "linimasa", "galeri": "film", "kartu": "kaca", "pemisah": "ganda"}'::jsonb WHERE slug = 'tongkonan-toraja' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "medali", "judul": "bingkai", "mempelai": "sejajar", "kisah": "selang", "acara": "tegas", "galeri": "polaroid", "kartu": "tegas", "pemisah": "ganda"}'::jsonb WHERE slug = 'benteng-buton' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "lingkaran", "judul": "pita", "mempelai": "zigzag", "kisah": "tengah", "acara": "linimasa", "galeri": "polaroid", "kartu": "cetak", "pemisah": "ombak"}'::jsonb WHERE slug = 'tifa-cengkih-ambon' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "lingkaran", "judul": "pita", "kisah": "selang", "acara": "tiket", "galeri": "film", "pemisah": "titik"}'::jsonb WHERE slug = 'kedaton-ternate' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "penuh", "judul": "kapital", "mempelai": "panel", "kisah": "kartu", "acara": "linimasa", "galeri": "mozaik", "kartu": "tegas", "pemisah": "titik"}'::jsonb WHERE slug = 'cendrawasih-papua' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "lingkaran", "judul": "kapital", "mempelai": "potret", "kisah": "tengah", "acara": "tegas", "galeri": "mozaik", "kartu": "potong", "pemisah": "ombak"}'::jsonb WHERE slug = 'raja-ampat' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "jendela", "judul": "samping", "mempelai": "panel", "kisah": "tengah", "acara": "tegas", "galeri": "film", "kartu": "tegas"}'::jsonb WHERE slug = 'kaki-seribu-arfak' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "penuh", "judul": "samping", "mempelai": "zigzag", "kisah": "tengah", "acara": "linimasa", "galeri": "kolom", "kartu": "potong", "pemisah": "tumpal"}'::jsonb WHERE slug = 'ukir-kamoro-mimika' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "lingkaran", "judul": "samping", "mempelai": "medali", "kisah": "kartu", "galeri": "mozaik", "kartu": "kaca"}'::jsonb WHERE slug = 'honai-baliem' AND rupa = '{}'::jsonb;
UPDATE themes SET rupa = '{"sampul": "wajik", "judul": "kapital", "mempelai": "sejajar", "acara": "tegas", "galeri": "film", "kartu": "cetak", "pemisah": "titik"}'::jsonb WHERE slug = 'asmat-merauke' AND rupa = '{}'::jsonb;

-- ── Gerak gulir kembar diganti (pasangan buka+gulir kini unik) ──
-- joglo-sidomukti: keraton → bayang (kembar dengan javanese-royal)
UPDATE themes SET scroll_anim = 'bayang' WHERE slug = 'joglo-sidomukti' AND scroll_anim = 'keraton' AND NOT motion_locked;
-- tenun-sasak-lombok: tenun → anggun (kembar dengan minang-songket)
UPDATE themes SET scroll_anim = 'anggun' WHERE slug = 'tenun-sasak-lombok' AND scroll_anim = 'tenun' AND NOT motion_locked;
-- batang-garing-ngaju: mekar → tenun (kembar dengan botanical-heritage)
UPDATE themes SET scroll_anim = 'tenun' WHERE slug = 'batang-garing-ngaju' AND scroll_anim = 'mekar' AND NOT motion_locked;
-- lamin-kenyah: keraton → tenun (kembar dengan songket-limas-palembang)
UPDATE themes SET scroll_anim = 'tenun' WHERE slug = 'lamin-kenyah' AND scroll_anim = 'keraton' AND NOT motion_locked;
-- kayan-tidung: geser → anggun (kembar dengan ondel-ondel-betawi)
UPDATE themes SET scroll_anim = 'anggun' WHERE slug = 'kayan-tidung' AND scroll_anim = 'geser' AND NOT motion_locked;
-- kedaton-ternate: keraton → naik (kembar dengan songket-limas-palembang)
UPDATE themes SET scroll_anim = 'naik' WHERE slug = 'kedaton-ternate' AND scroll_anim = 'keraton' AND NOT motion_locked;
-- raja-ampat: ombak → naik (kembar dengan granit-belitung)
UPDATE themes SET scroll_anim = 'naik' WHERE slug = 'raja-ampat' AND scroll_anim = 'ombak' AND NOT motion_locked;
