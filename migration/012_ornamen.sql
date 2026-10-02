-- ═══════════════════════════════════════════════════════════════════════════
-- 012_ornamen — "Lapisan Ornamen & Gerak" (/admin/tema/{slug}/ornamen).
--
--   theme_ornaments : hiasan bergambar per BAGIAN undangan (sampul, mempelai,
--                     kisah, galeri, acara, rsvp) — posisi, ukuran, putaran,
--                     animasi muncul & gerak diam. Semua nilai dari daftar
--                     tetap (web/ornamen.rs), bukan HTML/CSS mentah.
--   themes.gerak_judul / gerak_foto : gerak khusus judul & foto saat muncul
--                     ('ikut' = sama dengan gerak scroll tema).
--   themes.ken_burns : foto sampul perlahan membesar-bergeser.
--
-- Idempoten. Hanya skema — contoh ornamen & gerak untuk semua tema diisi
-- 013_gerak_everlove.sql (gambar di public/img/tema/ornamen/).
-- ═══════════════════════════════════════════════════════════════════════════

ALTER TABLE themes ADD COLUMN IF NOT EXISTS gerak_judul TEXT    NOT NULL DEFAULT 'ikut';
ALTER TABLE themes ADD COLUMN IF NOT EXISTS gerak_foto  TEXT    NOT NULL DEFAULT 'ikut';
ALTER TABLE themes ADD COLUMN IF NOT EXISTS ken_burns   BOOLEAN NOT NULL DEFAULT FALSE;

CREATE TABLE IF NOT EXISTS theme_ornaments (
    id          BIGSERIAL   PRIMARY KEY,
    theme       TEXT        NOT NULL REFERENCES themes (slug) ON DELETE CASCADE ON UPDATE CASCADE,
    bagian      TEXT        NOT NULL DEFAULT 'sampul',
    img         TEXT        NOT NULL,
    posisi      TEXT        NOT NULL DEFAULT 'kiri-atas',
    x           INT         NOT NULL DEFAULT 0,
    y           INT         NOT NULL DEFAULT 0,
    lebar       INT         NOT NULL DEFAULT 40,
    rotasi      INT         NOT NULL DEFAULT 0,
    cermin      BOOLEAN     NOT NULL DEFAULT FALSE,
    masuk       TEXT        NOT NULL DEFAULT 'zoom',
    jeda        INT         NOT NULL DEFAULT 0,
    durasi      INT         NOT NULL DEFAULT 1200,
    gerak       TEXT        NOT NULL DEFAULT 'none',
    kecepatan   INT         NOT NULL DEFAULT 6,
    depan       BOOLEAN     NOT NULL DEFAULT FALSE,
    hp          BOOLEAN     NOT NULL DEFAULT TRUE,
    opasitas    INT         NOT NULL DEFAULT 100,
    urutan      INT         NOT NULL DEFAULT 100,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS theme_ornaments_theme_idx ON theme_ornaments (theme, bagian, urutan, id);
