-- 050_perbaiki_danau_rembulan_3d — perbaikan tema `danau-rembulan-3d` (049).
--
-- Bug: tema ditandai gelap (dark = TRUE, latar #171d38) tetapi token teks
-- `ink` (#242744), `gold-deep` (#3f486e) & `primary` (#59658e) dibuat GELAP →
-- kontras teks di atas latar/panggung hanya 1,14:1 (ink) / 1,9:1 (gold-deep):
-- nama mempelai, "The Wedding of", tanggal, label & ikon nyaris tak terbaca.
-- Kartu terang (#f8f4f2) juga bertabrakan dengan --side-ink/--surface-* tema
-- gelap yang diturunkan dari `ink`.
--
-- Perbaikan: palet malam yang konsisten (latar, kartu, & permukaan gelap; teks,
-- judul & aksen terang), pola sama dengan `midnight-celestial`. Kontras (WCAG):
-- ink/latar 14:1, ink/kartu 11,8:1, muted/kartu 6:1, judul/latar 10:1.
--
-- 049 TIDAK disunting (migrasi tercatat dengan checksum — berkas yang berubah
-- tak dijalankan ulang), jadi perbaikan lewat UPDATE. Hanya menimpa bila
-- token masih persis bawaan 049: warna yang sudah disesuaikan admin dibiarkan.
UPDATE themes
SET tokens = '{"bg":"#171d38","card":"#212a4d","primary":"#d8c3ed","on-primary":"#1c2142","gold":"#e6cfa0","gold-deep":"#e6cfa0","gold-pale":"#3a4170","ink":"#eeeaf6","muted":"#aaa6c8","line":"#343c63"}'::jsonb
WHERE slug = 'danau-rembulan-3d'
  AND tokens = '{"bg":"#171d38","card":"#f8f4f2","primary":"#59658e","on-primary":"#ffffff","gold":"#d8c3ed","gold-deep":"#3f486e","gold-pale":"#e9dff0","ink":"#242744","muted":"#6a6c87","line":"#e4ddea"}'::jsonb;
