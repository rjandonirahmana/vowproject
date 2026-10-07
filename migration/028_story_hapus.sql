-- ═══════════════════════════════════════════════════════════════════════════
-- 028_story_hapus — pembuat story bisa menghapus story-nya sendiri.
--   owner_token_hash  SHA-256 token acak yang disimpan di cookie HttpOnly
--                     `ily_s_{slug}` perangkat pembuat saat story terbit →
--                     tombol "Hapus story saya" di penampil & kartunya.
--                     (Dari perangkat lain: minta kunci WA ke nomornya.)
-- Pengelola undangan menghapus dari /kelola, admin dari /admin/story.
-- Kode tetap jalan sebelum file ini (kolom dibaca lewat to_jsonb). Aman
-- dijalankan ulang. Butuh 026.
-- ═══════════════════════════════════════════════════════════════════════════
ALTER TABLE invitation_stories ADD COLUMN IF NOT EXISTS owner_token_hash TEXT NOT NULL DEFAULT '';
CREATE INDEX IF NOT EXISTS invitation_stories_recent_idx ON invitation_stories (created_at DESC);
