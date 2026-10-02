-- ═══════════════════════════════════════════════════════════════════════════
-- 006_animasi — gerak saat scroll per tema + tiap tema bawaan diberi paket
-- animasi yang berbeda. Idempoten.
--
--   themes.scroll_anim: naik | pudar | zoom | geser | lipat | blur | none
--
-- Catatan: tema dengan animasi pembuka (open_anim ≠ none) otomatis tampil
-- sebagai satu halaman panjang — tak perlu mengatur page_mode terpisah.
-- ═══════════════════════════════════════════════════════════════════════════

ALTER TABLE themes ADD COLUMN IF NOT EXISTS scroll_anim TEXT NOT NULL DEFAULT 'naik';

-- Paket gerak berbeda untuk tema bawaan (hanya bila belum disunting admin).
UPDATE themes t SET open_anim = v.o, scroll_anim = v.s, float_deco = v.f
FROM (VALUES
    ('botanical-heritage', 'tirai',   'naik',  'kelopak'),
    ('javanese-royal',     'gerbang', 'zoom',  'none'),
    ('sundanese-melati',   'amplop',  'geser', 'kelopak'),
    ('modern-serenade',    'pudar',   'pudar', 'none'),
    ('kraton-charcoal',    'gerbang', 'lipat', 'bintang'),
    ('midnight-celestial', 'pudar',   'blur',  'bintang'),
    ('minang-songket',     'gerbang', 'zoom',  'none'),
    ('bali-frangipani',    'amplop',  'naik',  'kupu'),
    ('rustic-kraft',       'tirai',   'geser', 'kelopak')
) AS v(slug, o, s, f)
WHERE t.slug = v.slug AND t.open_anim = 'none' AND t.scroll_anim = 'naik';
