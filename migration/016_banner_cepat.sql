-- ═══════════════════════════════════════════════════════════════════════════
-- 016_banner_cepat — banner ke-4 beranda: situs dibangun dengan Rust (cepat).
-- Idempoten: dilewati bila banner dengan gambar yang sama sudah ada (termasuk
-- bila admin sudah menyunting teksnya). Butuh 011.
-- ═══════════════════════════════════════════════════════════════════════════

INSERT INTO banners (judul, sub, cta, link, img, img_hp, aktif, urutan)
SELECT 'Dibangun dengan Rust — Undangan Secepat Kilat',
       'Ditenagai Rust, bahasa pemrograman super cepat: undangan terbuka instan, ringan di sinyal lemah & hemat kuota tamu.',
       'Coba Demo', '/u/anindita-raditya',
       '/img/banner/banner-4-cepat.webp', '/img/banner/banner-4-cepat-hp.webp',
       TRUE, COALESCE((SELECT MAX(urutan) FROM banners), 0) + 10
WHERE NOT EXISTS (SELECT 1 FROM banners WHERE img = '/img/banner/banner-4-cepat.webp');
