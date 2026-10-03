-- ═══════════════════════════════════════════════════════════════════════════
-- 017_bukti_bayar — bukti transfer pembayaran yang diunggah pemesan di Kelola.
-- Idempoten.
--
--   payment_proof    : URL gambar bukti di RustFS ('' bila RustFS tak dipakai —
--                      gambar tetap dikirim ke WA admin lewat WAHA)
--   payment_proof_at : kapan bukti terakhir dikirim. NOT NULL = pemesan sudah
--                      mengaku membayar → pesanan TIDAK dihapus otomatis
--                      (server/cleanup.rs) walau admin belum sempat memeriksa.
-- ═══════════════════════════════════════════════════════════════════════════

ALTER TABLE invitations ADD COLUMN IF NOT EXISTS payment_proof    TEXT NOT NULL DEFAULT '';
ALTER TABLE invitations ADD COLUMN IF NOT EXISTS payment_proof_at TIMESTAMPTZ;

-- Metode lama (qris/va/kartu) tak pernah benar-benar diproses — samakan.
UPDATE invitations SET payment_method = 'shopeepay'
 WHERE payment_method IN ('qris', 'va', 'kartu') AND status = 'menunggu_pembayaran';
