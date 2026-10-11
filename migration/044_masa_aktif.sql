-- ═══════════════════════════════════════════════════════════════════════════
-- 044_masa_aktif — tanggal undangan DIAKTIFKAN admin, dasar masa aktif paket
-- (Paket.aktif_hari di konten: Silver 365 hari, lainnya 0 = selamanya).
-- Undangan aktif lama: diisi dari waktu bukti bayar, atau waktu dibuat.
-- Idempoten.
-- ═══════════════════════════════════════════════════════════════════════════
ALTER TABLE invitations ADD COLUMN IF NOT EXISTS activated_at TIMESTAMPTZ;

UPDATE invitations
   SET activated_at = COALESCE(payment_proof_at, created_at)
 WHERE status = 'aktif' AND activated_at IS NULL AND NOT is_demo;
