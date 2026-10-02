-- ═══════════════════════════════════════════════════════════════════════════
-- 004_keamanan — pengerasan sebelum produksi. Idempoten: aman dijalankan ulang.
--
--   1. Kunci Kelola tidak lagi disimpan polos: hanya SHA-256-nya
--      (manage_key_hash). Tautan lama pelanggan TETAP berlaku (hash dihitung
--      dari kunci lama), lalu kolom polos dikosongkan.
--   2. invitations.theme → FK ke themes(slug): tema yang dipakai undangan tak
--      bisa terhapus walau balapan dengan checkout.
--   3. Indeks yang belum ada: gifts(guest_id), admin_sessions(expires_at).
--
-- JALANKAN SEBELUM men-deploy kode yang membutuhkannya (repo.rs memakai
-- manage_key_hash untuk Kelola & checkout).
-- ═══════════════════════════════════════════════════════════════════════════

ALTER TABLE invitations ADD COLUMN IF NOT EXISTS manage_key_hash TEXT;
UPDATE invitations
   SET manage_key_hash = encode(sha256(convert_to(manage_key, 'UTF8')), 'hex')
 WHERE manage_key_hash IS NULL AND manage_key <> '';
UPDATE invitations SET manage_key = '' WHERE manage_key <> '';
ALTER TABLE invitations ALTER COLUMN manage_key SET DEFAULT '';
CREATE INDEX IF NOT EXISTS invitations_manage_key_hash_idx ON invitations (manage_key_hash);

-- Tema bawaan wajib ada (jadi tujuan undangan yang temanya hilang).
INSERT INTO themes (slug, name, category, nuansa, palette, layout, ornament, font)
VALUES ('botanical-heritage', 'Botanical Heritage Romance', 'Botanical', 'Islami', 'sage', 'klasik', 'daun', 'playfair')
ON CONFLICT (slug) DO NOTHING;
UPDATE invitations SET theme = 'botanical-heritage' WHERE theme NOT IN (SELECT slug FROM themes);
DO $$
BEGIN
    ALTER TABLE invitations
        ADD CONSTRAINT invitations_theme_fk FOREIGN KEY (theme) REFERENCES themes (slug);
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

CREATE INDEX IF NOT EXISTS gifts_guest_idx ON gifts (guest_id);
CREATE INDEX IF NOT EXISTS admin_sessions_expires_idx ON admin_sessions (expires_at);
