-- ═══════════════════════════════════════════════════════════════════════════
-- 024_desain_v2 — desain undangan v2 (judul bagian kaligrafi ala undangan
-- WordPress/Elementor premium). Tema publik yang belum memilih huruf
-- kaligrafi diberi Great Vibes (tanda "&" Pinyon Script terbaca seperti "4"); tanpa itu judul "Bride & Groom", "Save The
-- Date", dst. jatuh ke huruf judul biasa. Admin tetap bisa menggantinya
-- (termasuk kembali ke "Tanpa kaligrafi") di /admin/tema.
-- Aman dijalankan ulang (hanya menyentuh yang masih kosong).
-- ═══════════════════════════════════════════════════════════════════════════
UPDATE themes SET script_font = 'great-vibes'
WHERE listed AND COALESCE(script_font, '') = '';
