-- ═══════════════════════════════════════════════════════════════════════════
-- 026_story — Story tamu di undangan (tab "Story" di sebelah Doa & RSVP).
--
--   story_keys          kunci spesial 6 digit yang dikirim lewat WhatsApp
--                       (WAHA) setelah tamu memasukkan nomor HP. Hanya HASH
--                       SHA-256 yang disimpan; berlaku 15 menit, maks 5 kali
--                       coba, sekali pakai.
--   invitation_stories  satu story FOTO per nomor HP per undangan (UNIQUE).
--                       Foto di RustFS foto/{slug}/story-….webp. Pemilik
--                       undangan bisa menghapusnya dari Kelola.
--
-- + banner beranda "Story Undangan" dan beberapa story contoh untuk undangan
--   demo (foto contoh situs sendiri; demo tak menerima unggahan tamu).
-- Aman dijalankan ulang. Kode tetap jalan sebelum file ini dijalankan (tab
-- Story menampilkan "belum ada story").
-- ═══════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS story_keys (
    id            BIGSERIAL   PRIMARY KEY,
    invitation_id BIGINT      NOT NULL REFERENCES invitations (id) ON DELETE CASCADE,
    phone         TEXT        NOT NULL,             -- 62… (fmt::wa_number)
    name          TEXT        NOT NULL DEFAULT '',
    key_hash      TEXT        NOT NULL,             -- SHA-256 hex kunci 6 digit
    attempts      INT         NOT NULL DEFAULT 0,
    used_at       TIMESTAMPTZ,
    expires_at    TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '15 minutes',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS story_keys_lookup_idx ON story_keys (invitation_id, phone, created_at DESC);

CREATE TABLE IF NOT EXISTS invitation_stories (
    id            BIGSERIAL   PRIMARY KEY,
    invitation_id BIGINT      NOT NULL REFERENCES invitations (id) ON DELETE CASCADE,
    phone         TEXT        NOT NULL,
    name          TEXT        NOT NULL,
    photo_url     TEXT        NOT NULL,
    filter        TEXT        NOT NULL DEFAULT 'normal',
    caption       TEXT        NOT NULL DEFAULT '',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (invitation_id, phone)
);
CREATE INDEX IF NOT EXISTS invitation_stories_list_idx ON invitation_stories (invitation_id, created_at DESC);

-- Story contoh untuk undangan demo (nomor fiktif 6200000000…, tak bisa
-- menerima WA). Dilewati bila demo sudah punya story.
INSERT INTO invitation_stories (invitation_id, phone, name, photo_url, filter, caption, created_at)
SELECT i.id, v.phone, v.name, v.photo, v.filter, v.caption, NOW() - v.ago
  FROM invitations i,
       (VALUES
         ('620000000001', 'Dimas Wicaksono', '/img/layanan/dekor-galeri-altar.jpg',  'clarendon', 'Dekorasinya cantik banget! Sampai jumpa di hari H 🤍', INTERVAL '3 hours'),
         ('620000000002', 'Siti Sarah',      '/img/layanan/mua-modern.jpg',          'gingham',   'Siap jadi bridesmaid paling heboh ✨',               INTERVAL '2 hours'),
         ('620000000003', 'Budi Santoso',    '/img/layanan/venue-lawu-park.jpg',     'crema',     'Venue-nya adem, cocok buat keluarga besar.',          INTERVAL '90 minutes'),
         ('620000000004', 'Rian & Arini',    '/img/layanan/dekor-galeri-meja.jpg',   'juno',      'Selamat Yona & Doni! Doa terbaik dari kami.',         INTERVAL '40 minutes'),
         ('620000000005', 'Keluarga Fulan',  '/img/layanan/dekor-galeri-gebyok.jpg', 'reyes',     'Barakallahu lakuma 🤲',                                INTERVAL '10 minutes')
       ) AS v(phone, name, photo, filter, caption, ago)
 WHERE i.is_demo
   AND NOT EXISTS (SELECT 1 FROM invitation_stories s WHERE s.invitation_id = i.id);

-- Banner beranda (butuh 011). Gambar tanpa teks; teks dari kolom di bawah.
INSERT INTO banners (judul, sub, cta, link, img, img_hp, aktif, urutan)
SELECT 'Baru! Story Undangan — Tamu Berbagi Momen',
       'Tamu cukup masukkan nomor WhatsApp, terima kunci spesial, lalu unggah satu foto story. Semua tamu lain bisa melihatnya seperti story Instagram.',
       'Lihat Story Demo', '/u/{demo}/story',
       '/img/banner/banner-5-story.webp', '/img/banner/banner-5-story-hp.webp',
       TRUE, 5
WHERE NOT EXISTS (SELECT 1 FROM banners WHERE img = '/img/banner/banner-5-story.webp');
