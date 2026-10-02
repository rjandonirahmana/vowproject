-- ═══════════════════════════════════════════════════════════════════════════
-- 001_init — skema awal undangan-ily.
--
--   invitations  satu undangan = satu pasangan; konten fleksibel di JSONB
--                (acara, rekening, galeri, warna busana) supaya formulir bisa
--                tumbuh tanpa migrasi baru.
--   guests       daftar undangan milik pengantin (kode unik → link pribadi,
--                QR check-in, status buka/kirim/hadir).
--   rsvps        konfirmasi kehadiran + doa & ucapan (buku tamu digital).
--   gifts        laporan tanda kasih / amplop digital dari tamu.
--
-- Tema TIDAK disimpan di DB: katalog tema = kode (src/web/themes.rs + CSS),
-- kolom `theme` hanya menyimpan slug-nya.
-- ═══════════════════════════════════════════════════════════════════════════

CREATE TABLE invitations (
    id              BIGSERIAL PRIMARY KEY,
    slug            TEXT        NOT NULL UNIQUE,
    -- Kunci halaman /kelola — dikirim sekali ke pemesan, jangan ditampilkan ke tamu.
    manage_key      TEXT        NOT NULL,
    theme           TEXT        NOT NULL DEFAULT 'botanical-heritage',
    package         TEXT        NOT NULL DEFAULT 'gold',
    -- menunggu_pembayaran | aktif | nonaktif
    status          TEXT        NOT NULL DEFAULT 'menunggu_pembayaran',
    is_demo         BOOLEAN     NOT NULL DEFAULT FALSE,

    bride_name      TEXT        NOT NULL,
    bride_degree    TEXT        NOT NULL DEFAULT '',
    bride_nick      TEXT        NOT NULL DEFAULT '',
    bride_parents   TEXT        NOT NULL DEFAULT '',
    bride_ig        TEXT        NOT NULL DEFAULT '',
    bride_photo     TEXT        NOT NULL DEFAULT '',
    groom_name      TEXT        NOT NULL,
    groom_degree    TEXT        NOT NULL DEFAULT '',
    groom_nick      TEXT        NOT NULL DEFAULT '',
    groom_parents   TEXT        NOT NULL DEFAULT '',
    groom_ig        TEXT        NOT NULL DEFAULT '',
    groom_photo     TEXT        NOT NULL DEFAULT '',

    -- [{kind,title,badge,tag,date,time_start,time_end,sessions:[{label,time}],venue,address,maps_url}]
    events          JSONB       NOT NULL DEFAULT '[]',
    dress_code      TEXT        NOT NULL DEFAULT '',
    -- [{name,hex}]
    dress_colors    JSONB       NOT NULL DEFAULT '[]',
    quote_text      TEXT        NOT NULL DEFAULT '',
    quote_source    TEXT        NOT NULL DEFAULT '',

    music_title     TEXT        NOT NULL DEFAULT '',
    music_artist    TEXT        NOT NULL DEFAULT '',
    music_url       TEXT        NOT NULL DEFAULT '',
    music_autoplay  BOOLEAN     NOT NULL DEFAULT TRUE,
    music_loop      BOOLEAN     NOT NULL DEFAULT TRUE,

    -- [{bank,number,holder}]
    banks           JSONB       NOT NULL DEFAULT '[]',
    gift_address    TEXT        NOT NULL DEFAULT '',
    -- [url, ...]
    gallery         JSONB       NOT NULL DEFAULT '[]',
    family_name     TEXT        NOT NULL DEFAULT '',

    -- Pesanan: rincian harga dibekukan saat checkout.
    addons          JSONB       NOT NULL DEFAULT '[]',
    coupon          TEXT        NOT NULL DEFAULT '',
    total_price     BIGINT      NOT NULL DEFAULT 0,
    payment_method  TEXT        NOT NULL DEFAULT '',
    contact_phone   TEXT        NOT NULL DEFAULT '',

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE guests (
    id              BIGSERIAL PRIMARY KEY,
    invitation_id   BIGINT      NOT NULL REFERENCES invitations(id) ON DELETE CASCADE,
    -- Kode pendek di link pribadi (?g=KODE) dan di QR check-in.
    code            TEXT        NOT NULL UNIQUE,
    name            TEXT        NOT NULL,
    phone           TEXT        NOT NULL DEFAULT '',
    -- vip | keluarga | sahabat | kantor | umum
    category        TEXT        NOT NULL DEFAULT 'umum',
    session         TEXT        NOT NULL DEFAULT '',
    table_no        TEXT        NOT NULL DEFAULT '',
    pax             INT         NOT NULL DEFAULT 1,
    sent_at         TIMESTAMPTZ,
    opened_at       TIMESTAMPTZ,
    checked_in_at   TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX guests_invitation_idx ON guests (invitation_id, created_at DESC);

CREATE TABLE rsvps (
    id              BIGSERIAL PRIMARY KEY,
    invitation_id   BIGINT      NOT NULL REFERENCES invitations(id) ON DELETE CASCADE,
    guest_id        BIGINT      REFERENCES guests(id) ON DELETE SET NULL,
    name            TEXT        NOT NULL,
    phone           TEXT        NOT NULL DEFAULT '',
    -- hadir | ragu | tidak
    status          TEXT        NOT NULL,
    pax             INT         NOT NULL DEFAULT 1,
    session         TEXT        NOT NULL DEFAULT '',
    message         TEXT        NOT NULL DEFAULT '',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX rsvps_invitation_idx ON rsvps (invitation_id, created_at DESC);
-- Satu tamu terdaftar = satu RSVP (dikirim ulang → diperbarui).
CREATE UNIQUE INDEX rsvps_guest_uniq ON rsvps (guest_id) WHERE guest_id IS NOT NULL;

CREATE TABLE gifts (
    id              BIGSERIAL PRIMARY KEY,
    invitation_id   BIGINT      NOT NULL REFERENCES invitations(id) ON DELETE CASCADE,
    guest_id        BIGINT      REFERENCES guests(id) ON DELETE SET NULL,
    name            TEXT        NOT NULL,
    amount          BIGINT      NOT NULL DEFAULT 0,
    channel         TEXT        NOT NULL DEFAULT '',
    note            TEXT        NOT NULL DEFAULT '',
    verified        BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX gifts_invitation_idx ON gifts (invitation_id, created_at DESC);

-- ── Seed: undangan demo (dipakai katalog "Demo" & tab Kelola read-only) ────
INSERT INTO invitations (
    slug, manage_key, theme, package, status, is_demo,
    bride_name, bride_degree, bride_nick, bride_parents, bride_ig,
    groom_name, groom_degree, groom_nick, groom_parents, groom_ig,
    events, dress_code, dress_colors, quote_text, quote_source,
    music_title, music_artist, music_url,
    banks, gift_address, family_name
) VALUES (
    'anindita-raditya', 'demo', 'botanical-heritage', 'gold', 'aktif', TRUE,
    'Anindita Kirana', 'S.Ds.', 'Dita', 'Putri pertama dari Bpk. Ir. Bambang Soedibyo & Ibu Dra. Hj. Ratna Kusuma', 'aninditakirana',
    'Raditya Pratama', 'M.B.A.', 'Radit', 'Putra kedua dari Bpk. Prof. Dr. Hendra Pratama & Ibu Hj. Siti Wardani', 'radityapratama',
    '[
      {"kind":"akad","title":"Akad Nikah","badge":"Pemberkatan & Akad","tag":"Sesi Khidmat",
       "date":"2026-10-24","time_start":"08:00","time_end":"10:00","sessions":[],
       "venue":"Masjid Agung Al-Ikhlas & Plataran Pavilion",
       "address":"Jl. Kemang Raya No. 45, Bangka, Mampang Prapatan, Jakarta Selatan",
       "maps_url":"https://maps.google.com/?q=Masjid+Agung+Al-Ikhlas+Kemang"},
      {"kind":"resepsi","title":"Resepsi Pernikahan","badge":"Resepsi Agung","tag":"Selebrasi",
       "date":"2026-10-24","time_start":"11:30","time_end":"21:00",
       "sessions":[{"label":"Sesi Siang","time":"11.30 – 14.30 WIB"},{"label":"Sesi Malam","time":"18.30 – 21.00 WIB"}],
       "venue":"The Glass House Ballroom, Plataran Dharmawangsa",
       "address":"Jl. Dharmawangsa VIII No. 12, Kebayoran Baru, Jakarta Selatan",
       "maps_url":"https://maps.google.com/?q=Plataran+Dharmawangsa"}
    ]',
    'Batik / Formal Attire — sentuhan warna Sage Green, Earthy Olive, atau Champagne Gold.',
    '[{"name":"Forest Sage","hex":"#3E5641"},{"name":"Olive","hex":"#5B7553"},{"name":"Champagne","hex":"#F3D9A4"}]',
    'Dan di antara tanda-tanda (kebesaran)-Nya ialah Dia menciptakan pasangan-pasangan untukmu dari jenismu sendiri, agar kamu cenderung dan merasa tenteram kepadanya, dan Dia menjadikan di antaramu rasa kasih dan sayang.',
    'QS. Ar-Rum: 21',
    'Kisah Romantis', 'Instrumental Piano', '/music/kisah-romantis.mp3',
    '[{"bank":"Bank Central Asia (BCA)","number":"8829 4910 2931","holder":"Raditya Pratama"},
      {"bank":"Bank Mandiri","number":"1370 0192 8412 9","holder":"Anindita Kirana"}]',
    'Jl. Dharmawangsa VIII No. 12, Kebayoran Baru, Jakarta Selatan 12160',
    'Keluarga Besar Soedibyo & Pratama'
);

INSERT INTO guests (invitation_id, code, name, phone, category, session, table_no, pax, sent_at, opened_at)
SELECT id, g.code, g.name, g.phone, g.category, g.session, g.table_no, g.pax, NOW() - g.sent, g.opened
FROM invitations,
(VALUES
  ('DEMO01', 'Dimas Wicaksono & Partner', '6281298421092', 'vip',      'Sesi 1 (Akad • 08.00 WIB)',    'A-03', 2, INTERVAL '3 days', NOW() - INTERVAL '2 minutes'),
  ('DEMO02', 'Prof. Dr. Irwan Syahputra',  '6281300000002', 'keluarga', 'Sesi 2 (Resepsi • 11.30 WIB)', 'V-01', 2, INTERVAL '3 days', NOW() - INTERVAL '1 day'),
  ('DEMO03', 'Maya Anggraini',             '6281300000003', 'kantor',   'Sesi 2 (Resepsi • 13.00 WIB)', '',     1, INTERVAL '3 days', NULL::timestamptz)
) AS g(code, name, phone, category, session, table_no, pax, sent, opened)
WHERE slug = 'anindita-raditya';

INSERT INTO rsvps (invitation_id, guest_id, name, status, pax, session, message, created_at)
SELECT i.id, gu.id, r.name, r.status, r.pax, r.session, r.message, NOW() - r.ago
FROM invitations i
JOIN (VALUES
  ('Siti Sarah & Suami', NULL, 'hadir', 2, 'Sesi Siang', 'Barakallahu lakum wa baraka alaikuma wa jama''a bainakuma fii khoir. Selamat menempuh hidup baru Dita & Radit! Semoga senantiasa diberikan cinta yang abadi dan sakinah mawaddah warahmah dunia akhirat. Aamiin!', INTERVAL '10 minutes'),
  ('Budi Santoso',       NULL, 'hadir', 1, 'Sesi Siang', 'Happy wedding Raditya & Anindita! Lancar sampai hari H yaa. Doa terbaik untuk babak baru kalian berdua, langgeng sampai kakek nenek!', INTERVAL '45 minutes'),
  ('Rian & Arini',       NULL, 'hadir', 2, 'Sesi Malam', 'Selamat berbahagia untuk sahabat kami berdua. Semoga lancar seluruh rangkaian acaranya hingga selesai. See you at the wedding!', INTERVAL '2 hours'),
  ('Dimas Wicaksono & Partner', 'DEMO01', 'hadir', 2, 'Sesi 1', 'Selamat Dita & Radit, sampai jumpa di akad!', INTERVAL '2 minutes')
) AS r(name, gcode, status, pax, session, message, ago) ON TRUE
LEFT JOIN guests gu ON gu.code = r.gcode
WHERE i.slug = 'anindita-raditya';

INSERT INTO gifts (invitation_id, guest_id, name, amount, channel, verified)
SELECT i.id, g.id, g.name, 1500000, 'BCA', TRUE
FROM invitations i JOIN guests g ON g.invitation_id = i.id AND g.code = 'DEMO02'
WHERE i.slug = 'anindita-raditya';
