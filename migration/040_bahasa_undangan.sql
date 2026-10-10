-- 040 — Bahasa undangan untuk tamu: 'id' (Indonesia) atau 'en' (English).
--
-- Dipilih pasangan di /buat & /kelola/…/sunting; tamu tetap boleh mengganti
-- sendiri lewat tombol ID/EN (?lang=). Teks yang ditulis pasangan (nama,
-- kutipan, kisah) tidak diterjemahkan — hanya teks bawaan undangan.
ALTER TABLE invitations ADD COLUMN IF NOT EXISTS lang TEXT NOT NULL DEFAULT 'id';
DO $$ BEGIN
    ALTER TABLE invitations ADD CONSTRAINT invitations_lang_cek CHECK (lang IN ('id', 'en'));
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;
