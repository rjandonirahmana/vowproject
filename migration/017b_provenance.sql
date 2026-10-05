-- ═══════════════════════════════════════════════════════════════════════════
-- 017b_provenance — asal-usul data gerak, agar migrasi seed TAK PERNAH menimpa
-- hasil kerja admin (dulu bergantung heuristik updated_at = created_at).
--
--   theme_ornaments.source  'seed' = diisi migrasi/skrip, 'admin' = dibuat
--                           atau disunting dari /admin (bawaan kolom = admin;
--                           repo::save_ornament menandai 'admin' saat sunting).
--   themes.motion_locked    TRUE begitu admin mengubah gerak tema (buka/scroll/
--                           hiasan/judul/foto/Ken Burns) — seed melewatinya.
--
-- Dinamai 017b agar berjalan SEBELUM 018 (yang sudah memakainya). Aman
-- dijalankan ulang. Kode server tetap jalan sebelum file ini dijalankan.
-- ═══════════════════════════════════════════════════════════════════════════

ALTER TABLE theme_ornaments ADD COLUMN IF NOT EXISTS source TEXT NOT NULL DEFAULT 'admin';
DO $$ BEGIN
    ALTER TABLE theme_ornaments ADD CONSTRAINT theme_ornaments_source_chk CHECK (source IN ('seed', 'admin', 'import'));
EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- Isi awal: aset bawaan (/img/tema/…) yang belum pernah disunting = seed.
-- Unggahan admin (URL RustFS) atau yang pernah disunting tetap 'admin'.
UPDATE theme_ornaments SET source = 'seed'
 WHERE source = 'admin' AND img LIKE '/img/tema/%' AND updated_at = created_at;

CREATE INDEX IF NOT EXISTS theme_ornaments_seed_idx ON theme_ornaments (theme) WHERE source = 'seed';

ALTER TABLE themes ADD COLUMN IF NOT EXISTS motion_locked BOOLEAN NOT NULL DEFAULT FALSE;
