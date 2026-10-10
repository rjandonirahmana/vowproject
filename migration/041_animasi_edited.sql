-- 041 — Animasi bawaan ikut diperbarui dari kode (seperti templat, migrasi 029).
--
-- Dulu animasi bawaan disemai `ON CONFLICT DO NOTHING` & versi DB menang →
-- perbaikan CSS gerak di repo tak pernah sampai ke DB: tamu melihat animasi
-- LAMA sampai admin menekan "Kembalikan ke bawaan". Kini baris bawaan yang
-- BELUM disunting admin (`edited = FALSE`) diperbarui setiap server start.
ALTER TABLE animations ADD COLUMN IF NOT EXISTS edited BOOLEAN NOT NULL DEFAULT FALSE;

-- Baris lama: anggap disunting admin bila diubah >1 menit setelah dibuat
-- (penyemaian mengisi created_at = updated_at) — suntingan admin tetap aman.
UPDATE animations SET edited = TRUE
 WHERE builtin AND NOT edited AND updated_at > created_at + INTERVAL '1 minute';
