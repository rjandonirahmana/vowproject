-- ═══════════════════════════════════════════════════════════════════════════
-- 027_pustaka_lagu — musik latar HANYA dari pustaka yang diunggah admin.
-- Pengantin memilih lagu dari daftar ini di /buat (tak bisa mengunggah lagu
-- sendiri lagi); lagu yang tak ada di daftar diminta lewat WhatsApp admin.
-- Admin mengelola di /admin/lagu (unggah MP3/M4A ke RustFS
-- musik/pustaka/{judul}.mp3, aktif/nonaktif, urutan).
--
-- Undangan menyimpan SALINAN url lagu (kolom music_url), jadi menghapus /
-- menonaktifkan lagu di pustaka tak memutus undangan yang sudah memakainya.
-- Isi awal: lagu demo (Teman Hidup – Tulus) — lagu bawaan lama di kode tak
-- pernah punya berkas audio. Aman dijalankan ulang.
-- ═══════════════════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS songs (
    id         BIGSERIAL   PRIMARY KEY,
    title      TEXT        NOT NULL,
    artist     TEXT        NOT NULL DEFAULT '',
    duration   TEXT        NOT NULL DEFAULT '',     -- "03:42" (tampilan saja)
    tag        TEXT        NOT NULL DEFAULT '',     -- mis. "Terpopuler", "Tradisional"
    url        TEXT        NOT NULL,
    aktif      BOOLEAN     NOT NULL DEFAULT TRUE,
    urutan     INT         NOT NULL DEFAULT 100,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS songs_list_idx ON songs (aktif, urutan, id);

INSERT INTO songs (title, artist, tag, url, urutan)
SELECT 'Teman Hidup', 'Tulus', 'Lagu Demo', 'https://image.ulalaapi.store/undangan/musik/TULUS-Teman-Hidup.mp3', 10
WHERE NOT EXISTS (SELECT 1 FROM songs);
