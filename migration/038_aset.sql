-- 038 — Aset statis situs (public/img, public/video) dipindah ke RustFS.
--
-- Satu baris = satu berkas yang sudah diunggah `scripts/unggah-aset.sh` ke
-- bucket `undangan` di bawah kunci `aset/…`. Saat start, server memuat tabel
-- ini (server/aset.rs) lalu:
--   * menulis ulang "/img/…" & "/video/…" di HTML & CSS tema menjadi `url`;
--   * mengalihkan (301) permintaan /img/… & /video/… yang masih tersisa
--     (mis. dari WASM / main.css) ke `url`.
-- Jalur yang TIDAK ada di tabel tetap disajikan dari public/ seperti biasa.

CREATE TABLE IF NOT EXISTS aset (
    jalur      TEXT PRIMARY KEY,          -- jalur situs, mis. /img/tema/lily-bg.svg
    url        TEXT NOT NULL,             -- URL publik RustFS
    mime       TEXT NOT NULL DEFAULT '',
    ukuran     BIGINT NOT NULL DEFAULT 0, -- byte
    sha256     TEXT NOT NULL DEFAULT '',
    diunggah   TIMESTAMPTZ NOT NULL DEFAULT now()
);
