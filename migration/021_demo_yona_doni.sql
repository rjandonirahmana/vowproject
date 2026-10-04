-- ═══════════════════════════════════════════════════════════════════════════
-- 021_demo_yona_doni — undangan demo jadi "Yona & Doni" (slug yona-doni).
-- Data diri sengaja MINIMAL: hanya nama panggilan pengantin; nama orang tua,
-- gelar, Instagram, rekening & keluarga disamarkan (nama umum/contoh).
-- Tautan lama /u/anindita-raditya & /kelola/anindita-raditya dialihkan 301 oleh
-- server/security.rs `legacy_demo`. Aman dijalankan ulang.
-- ═══════════════════════════════════════════════════════════════════════════

UPDATE invitations SET slug = 'yona-doni'
 WHERE slug = 'anindita-raditya' AND is_demo
   AND NOT EXISTS (SELECT 1 FROM invitations WHERE slug = 'yona-doni');

UPDATE invitations SET
    bride_name = 'Yona', bride_degree = '', bride_nick = 'Yona', bride_ig = '',
    bride_parents = 'Putri dari Bapak Fulan & Ibu Fulanah',
    groom_name = 'Doni', groom_degree = '', groom_nick = 'Doni', groom_ig = '',
    groom_parents = 'Putra dari Bapak Fulan & Ibu Fulanah',
    family_name = 'Keluarga Besar Kedua Mempelai',
    banks = '[{"bank":"Bank Central Asia (BCA)","number":"1234 5678 90","holder":"Doni"},
              {"bank":"Bank Mandiri","number":"1234 5678 9012 3","holder":"Yona"}]'::jsonb,
    love_story = replace(replace(love_story::text, 'Dita', 'Yona'), 'Radit', 'Doni')::jsonb
 WHERE slug = 'yona-doni' AND is_demo;

UPDATE rsvps r SET message = replace(replace(replace(replace(r.message,
         'Raditya & Anindita', 'Doni & Yona'), 'Dita & Radit', 'Yona & Doni'), 'Dita', 'Yona'), 'Radit', 'Doni')
  FROM invitations i
 WHERE r.invitation_id = i.id AND i.slug = 'yona-doni' AND (r.message LIKE '%Dita%' OR r.message LIKE '%Radit%');

UPDATE banners SET link = replace(link, '/u/anindita-raditya', '/u/yona-doni')
 WHERE link LIKE '/u/anindita-raditya%';
