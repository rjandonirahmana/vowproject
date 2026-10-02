-- ═══════════════════════════════════════════════════════════════════════════
-- 008_animasi_semua — SEMUA animasi undangan (bawaan + buatan admin) di tabel
-- `animations`. Idempoten; boleh dijalankan walau 007 belum.
--
--   * Kunci tema = slug apa adanya (dulu buatan admin berawalan "k-": baris
--     lama diganti slug-nya menjadi "k-…" sehingga tema yang memakainya tetap cocok).
--   * Kunci unik per JENIS (kind, slug) — "pudar" ada di buka & scroll.
--   * css     : CSS lanjutan (penanda {a} = kelas animasi); kosong = dari spec.
--   * builtin : animasi bawaan — diisi server saat start bila belum ada
--               (isi pabriknya di web/anim.rs), tak bisa dihapus, bisa
--               "Kembalikan ke bawaan" dari /admin/animasi.
-- ═══════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS animations (
    kind        TEXT        NOT NULL,
    slug        TEXT        NOT NULL,
    name        TEXT        NOT NULL,
    spec        JSONB       NOT NULL DEFAULT '{}',
    css         TEXT        NOT NULL DEFAULT '',
    builtin     BOOLEAN     NOT NULL DEFAULT FALSE,
    sort_order  INT         NOT NULL DEFAULT 100,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (kind, slug)
);

-- Tabel versi 007 (slug PRIMARY KEY, tanpa kolom builtin) → naikkan sekali.
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name = 'animations' AND column_name = 'builtin') THEN
        ALTER TABLE animations ADD COLUMN builtin BOOLEAN NOT NULL DEFAULT FALSE;
        UPDATE animations SET slug = 'k-' || slug WHERE slug NOT LIKE 'k-%';
        ALTER TABLE animations DROP CONSTRAINT IF EXISTS animations_pkey;
        ALTER TABLE animations ADD PRIMARY KEY (kind, slug);
    END IF;
END $$;

ALTER TABLE animations ADD COLUMN IF NOT EXISTS css TEXT NOT NULL DEFAULT '';
ALTER TABLE animations ADD COLUMN IF NOT EXISTS sort_order INT NOT NULL DEFAULT 100;
