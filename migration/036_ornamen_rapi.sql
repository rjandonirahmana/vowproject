-- ═══════════════════════════════════════════════════════════════════════════
-- 036_ornamen_rapi — ornamen bawaan yang menabrak teks (audit piksel 9 Okt
-- 2026, HP 412 px). Hanya baris SEED — ornamen yang disunting admin utuh.
-- Aman dijalankan ulang.
--
--  1. Watermark ikon bangunan di belakang judul "Wedding Event" & RSVP
--     (opasitas 22–28%) membuat judul kaligrafi tipis sulit dibaca → 14%.
--  2. Tokoh pengapit sampul (kiri/kanan-bawah, geser +24%) menabrak tanggal
--     & nama → diturunkan; kini berdiri di belakang kartu tamu.
--  3. Hiasan gantung galeri (tengah-atas, geser −78%) menjuntai ke luar
--     bagiannya & menimpa teks babak terakhir Love Story → masuk ke ruang
--     atas galeri sendiri (dicadangkan main.css).
--  Lambang/hiasan tengah-atas yang menindih label bagian diperbaiki di
--  main.css (ruang dicadangkan), tanpa mengubah data.
-- ═══════════════════════════════════════════════════════════════════════════
UPDATE theme_ornaments SET opasitas = 14
 WHERE source = 'seed' AND NOT depan AND opasitas BETWEEN 15 AND 45
   AND bagian IN ('acara', 'rsvp') AND posisi IN ('tengah-atas', 'tengah-bawah', 'tengah');

UPDATE theme_ornaments SET y = 62
 WHERE source = 'seed' AND bagian = 'sampul' AND posisi IN ('kiri-bawah', 'kanan-bawah') AND y BETWEEN 0 AND 40;

UPDATE theme_ornaments SET y = -28
 WHERE source = 'seed' AND bagian = 'galeri' AND posisi = 'tengah-atas' AND y < -50;
