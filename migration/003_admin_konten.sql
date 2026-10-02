-- ═══════════════════════════════════════════════════════════════════════════
-- 003_admin_konten — akun admin berperan + konten situs yang bisa disunting.
--
--   admin_users     akun panel /admin. role: admin (semua) | editor (tema &
--                   konten). Sandi argon2id; akun pertama dibuat lewat kode
--                   setup ADMIN_TOKEN.
--   admin_sessions  sesi masuk; cookie berisi token acak, DB hanya menyimpan
--                   hash SHA-256-nya.
--   site_content    satu baris = satu bagian konten (JSONB), mis. paket MUA,
--                   harga paket digital, galeri dekorasi. Bagian yang belum
--                   pernah disunting memakai isi bawaan di kode (web/konten.rs).
-- ═══════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS admin_users (
    id             BIGSERIAL   PRIMARY KEY,
    username       TEXT        NOT NULL UNIQUE,
    name           TEXT        NOT NULL DEFAULT '',
    password_hash  TEXT        NOT NULL,
    role           TEXT        NOT NULL DEFAULT 'editor',
    active         BOOLEAN     NOT NULL DEFAULT TRUE,
    last_login_at  TIMESTAMPTZ,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS admin_sessions (
    token_hash  TEXT        PRIMARY KEY,
    user_id     BIGINT      NOT NULL REFERENCES admin_users(id) ON DELETE CASCADE,
    expires_at  TIMESTAMPTZ NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS admin_sessions_user_idx ON admin_sessions (user_id);

CREATE TABLE IF NOT EXISTS site_content (
    key         TEXT        PRIMARY KEY,
    data        JSONB       NOT NULL,
    updated_by  TEXT        NOT NULL DEFAULT '',
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
