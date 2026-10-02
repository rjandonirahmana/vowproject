-- ═══════════════════════════════════════════════════════════════════════════
-- 010_musik_demo — lagu bawaan semua contoh/demo undangan (web/themes.rs
-- DEMO_SONG). Idempoten. Tidak menyentuh undangan pembeli (hanya is_demo).
-- ═══════════════════════════════════════════════════════════════════════════

UPDATE invitations
   SET music_title  = 'Teman Hidup',
       music_artist = 'Tulus',
       music_url    = 'https://image.ulalaapi.store/undangan/musik/TULUS-Teman-Hidup.mp3',
       updated_at   = NOW()
 WHERE is_demo
   AND music_url IS DISTINCT FROM 'https://image.ulalaapi.store/undangan/musik/TULUS-Teman-Hidup.mp3';
