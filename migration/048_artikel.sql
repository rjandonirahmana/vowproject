-- Artikel editorial untuk SEO. Hanya status "published" yang dilayani publik.
CREATE TABLE IF NOT EXISTS articles (
    id               BIGSERIAL PRIMARY KEY,
    title            TEXT        NOT NULL,
    slug             TEXT        NOT NULL UNIQUE
        CHECK (slug ~ '^[a-z0-9]+(-[a-z0-9]+)*$'),
    excerpt          TEXT        NOT NULL DEFAULT '',
    content          TEXT        NOT NULL DEFAULT '',
    seo_title        TEXT        NOT NULL DEFAULT '',
    seo_description  TEXT        NOT NULL DEFAULT '',
    cover_image      TEXT        NOT NULL DEFAULT '',
    status           TEXT        NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'published')),
    author_id        BIGINT REFERENCES admin_users(id) ON DELETE SET NULL,
    published_at     TIMESTAMPTZ,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS articles_published_at_idx
    ON articles (published_at DESC, id DESC)
    WHERE status = 'published';
