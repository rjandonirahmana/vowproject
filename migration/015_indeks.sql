-- ═══════════════════════════════════════════════════════════════════════════
-- 015_indeks — indeks untuk tugas latar pembersihan (server/cleanup.rs).
-- Idempoten.
--
--   invitations_unpaid_idx : purge_unpaid tiap 15 menit memfilter pesanan
--                            belum dibayar berdasarkan umur — indeks parsial
--                            kecil (hanya baris menunggu pembayaran).
-- (admin_sessions.expires_at sudah diindeks di 004.)
-- ═══════════════════════════════════════════════════════════════════════════

CREATE INDEX IF NOT EXISTS invitations_unpaid_idx
    ON invitations (created_at)
    WHERE status = 'menunggu_pembayaran' AND NOT is_demo;
