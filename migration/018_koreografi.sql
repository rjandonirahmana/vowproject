-- ═══════════════════════════════════════════════════════════════════════════
-- 018_koreografi — 5 tema unggulan mendapat KOREOGRAFI GERAK sendiri-sendiri
-- (sebelumnya 013 menyamakan gerak scroll/judul/foto SEMUA tema → terasa sama).
--
--   javanese-royal     gebyok-ukir   + keraton + melati-gugur  (ala everlove)
--   botanical-heritage taman-daun    + mekar   + daun-gugur
--   midnight-celestial galaksi       + kosmik  + kunang
--   bali-frangipani    candi-bentar  + ombak   + kamboja-gugur
--   minang-songket     tenun-songket + tenun   + kilau-emas
--
-- Animasi baru = BAWAAN (web/anim.rs builtins, CSS di src/web/gerak/) dan
-- diisikan server sendiri saat start — file ini hanya memasang ke tema.
-- gerak_judul/gerak_foto → 'ikut' agar koreografi yang mengatur judul & foto.
-- melati-gugur/kamboja-gugur dari 009; bila tak ada → hiasan bawaan.
-- WAJIB setelah 013/014. Aman dijalankan ulang (hanya 5 tema ini).
-- ═══════════════════════════════════════════════════════════════════════════

UPDATE themes t
   SET open_anim = v.o, scroll_anim = v.s,
       float_deco = CASE WHEN v.f IN ('melati-gugur', 'kamboja-gugur')
                          AND NOT EXISTS (SELECT 1 FROM animations a WHERE a.kind = 'hiasan' AND a.slug = v.f)
                         THEN v.cadangan ELSE v.f END,
       gerak_judul = 'ikut', gerak_foto = 'ikut', ken_burns = v.kb, updated_at = NOW()
  FROM (VALUES
    ('javanese-royal',     'gebyok-ukir',   'keraton', 'melati-gugur',  'kelopak', TRUE),
    ('botanical-heritage', 'taman-daun',    'mekar',   'daun-gugur',    'kelopak', TRUE),
    ('midnight-celestial', 'galaksi',       'kosmik',  'kunang',        'bintang', FALSE),
    ('bali-frangipani',    'candi-bentar',  'ombak',   'kamboja-gugur', 'kupu',    TRUE),
    ('minang-songket',     'tenun-songket', 'tenun',   'kilau-emas',    'bintang', FALSE)
  ) AS v(slug, o, s, f, cadangan, kb)
 WHERE t.slug = v.slug;
