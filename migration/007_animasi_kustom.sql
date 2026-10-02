-- (Digantikan 008_animasi_semua.sql — 008 bisa dijalankan tanpa 007.)
-- ═══════════════════════════════════════════════════════════════════════════
-- 007_animasi_kustom — animasi buatan admin (/admin/animasi). Idempoten.
--
--   animations: satu baris = satu animasi. kind: buka | scroll | hiasan.
--   spec (JSONB) = pengaturan (angka & pilihan) yang diubah jadi CSS oleh
--   web/anim.rs — admin tidak pernah menulis CSS langsung.
--   Tema memakainya lewat kunci "k-{slug}" di open_anim/scroll_anim/float_deco.
-- ═══════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS animations (
    slug        TEXT        PRIMARY KEY,
    kind        TEXT        NOT NULL,
    name        TEXT        NOT NULL,
    spec        JSONB       NOT NULL DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
