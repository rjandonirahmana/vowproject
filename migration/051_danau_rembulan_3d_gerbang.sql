-- 051_danau_rembulan_3d_gerbang — tema `danau-rembulan-3d` punya gerbang buka/tutup.
--
-- Bug: 049 mengisi open_anim = 'none', padahal undangan satu halaman hanya
-- membuat sampul-gerbang (tombol "Buka Undangan", bisa ditutup lagi lewat
-- tombol tutup) bila open_anim berisi animasi buka sungguhan (SampulPage di
-- pages/undangan.rs: `gate = single && open_anim != "none"`). Akibatnya sampul,
-- panggung, dan seluruh isi langsung terbuka sekaligus — tak ada keadaan
-- tertutup, tak ada buka/tutup.
--
-- Perbaikan: pakai 'fajar-rembulan' (matahari terbit membuka; ditutup senja
-- lalu bulan naik) — pasangan resmi 'kosmik' + 'bintang' yang sudah dipakai
-- tema ini (preset "Fajar & Rembulan" di skin.rs). Hanya menimpa bila masih
-- bawaan 049 ('none'); pilihan admin tidak disentuh.
UPDATE themes
SET open_anim = 'fajar-rembulan'
WHERE slug = 'danau-rembulan-3d'
  AND open_anim = 'none';
