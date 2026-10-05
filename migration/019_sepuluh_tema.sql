-- ═══════════════════════════════════════════════════════════════════════════
-- 019_sepuluh_tema — katalog dipangkas jadi 10 tema yang konsepnya benar-benar
-- berbeda (bukan sekadar beda warna); tiap tema akan diberi koreografi gerak
-- sendiri. Urutan katalog mengikuti daftar di bawah (wayang kulit pertama).
--
-- • Tema PUBLIK (listed) di luar daftar DIHAPUS PERMANEN (ornamennya ikut,
--   ON DELETE CASCADE) — kecuali yang masih dipakai undangan: itu hanya
--   disembunyikan (listed = FALSE) agar undangan pelanggan tetap jalan.
-- • Tema privat (listed = FALSE, pesanan custom buatan admin) tidak disentuh.
-- WAJIB setelah 018. Aman dijalankan ulang.
-- ═══════════════════════════════════════════════════════════════════════════

DELETE FROM themes t
 WHERE t.listed
   AND t.slug NOT IN ('gunungan-wayang-kulit', 'javanese-royal', 'botanical-heritage', 'midnight-celestial', 'minang-songket',
                      'bali-frangipani', 'mega-mendung-cirebon', 'gorga-rumah-bolon', 'phinisi-layar-tujuh', 'burung-enggang-borneo')
   AND NOT EXISTS (SELECT 1 FROM invitations i WHERE i.theme = t.slug);

UPDATE themes SET listed = FALSE, updated_at = NOW()
 WHERE listed
   AND slug NOT IN ('gunungan-wayang-kulit', 'javanese-royal', 'botanical-heritage', 'midnight-celestial', 'minang-songket',
                    'bali-frangipani', 'mega-mendung-cirebon', 'gorga-rumah-bolon', 'phinisi-layar-tujuh', 'burung-enggang-borneo');

UPDATE themes t SET listed = TRUE, sort_order = v.n, updated_at = NOW()
  FROM (VALUES ('gunungan-wayang-kulit', 1), ('javanese-royal', 2), ('botanical-heritage', 3), ('midnight-celestial', 4),
               ('minang-songket', 5), ('bali-frangipani', 6), ('mega-mendung-cirebon', 7), ('gorga-rumah-bolon', 8),
               ('phinisi-layar-tujuh', 9), ('burung-enggang-borneo', 10)) AS v(slug, n)
 WHERE t.slug = v.slug;

-- Tema pertama: Gunungan Wayang Kulit — koreografi pagelaran (bawaan web/anim.rs).
UPDATE themes SET open_anim = 'pagelaran-wayang', scroll_anim = 'bayang', float_deco = 'kunang',
       gerak_judul = 'ikut', gerak_foto = 'ikut', ken_burns = TRUE, updated_at = NOW()
 WHERE slug = 'gunungan-wayang-kulit' AND NOT motion_locked;
