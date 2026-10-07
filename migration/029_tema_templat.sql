-- ═══════════════════════════════════════════════════════════════════════════
-- 029_tema_templat — tema TEMPLAT: HTML + CSS tema disimpan di DATABASE.
--
-- Hasil bedah everlove (Okt 2026): ±47 tema "3D Motion" mereka memakai SATU
-- templat Elementor induk yang di-clone (ID bagian & urutan gerak identik);
-- yang membuat tiap tema terasa beda = ilustrasi lukisan, video pembuka/latar,
-- warna & font. Maka skemanya dua lapis:
--
--   theme_templates  = STRUKTUR + GERAK: HTML (templat Jinja, dirender server
--                      dgn autoescape) + CSS + daftar aset bawaan + font.
--                      Gerak ditulis DEKLARATIF di HTML (data-a / data-s /
--                      data-idle …) dan dijalankan SATU mesin /tata.js.
--   themes           = KULIT: memilih templat (kolom `template`) lalu menimpa
--                      aset (template_assets) & CSS (template_css) sendiri.
--                      Beberapa tema boleh berbagi satu templat.
--
-- Templat bawaan (berkas templat/<slug>/ di repo) diisikan server saat start;
-- baris yang sudah disunting admin (edited = TRUE) tak pernah ditimpa.
-- Idempoten — aman dijalankan ulang.
-- ═══════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS theme_templates (
    slug        TEXT        PRIMARY KEY CHECK (slug ~ '^[a-z0-9-]{1,40}$'),
    name        TEXT        NOT NULL,
    html        TEXT        NOT NULL,
    css         TEXT        NOT NULL DEFAULT '',
    -- Nilai `family=` Google Fonts, mis. "Pinyon+Script&family=Cormorant+Infant:wght@400;600".
    fonts       TEXT        NOT NULL DEFAULT '',
    -- Aset bawaan: kunci → URL (/lokal atau https), dipanggil di HTML sebagai {{ a.kunci }}.
    assets      JSONB       NOT NULL DEFAULT '{}'::jsonb,
    -- TRUE = berasal dari berkas templat/ di repo; FALSE = dibuat admin.
    builtin     BOOLEAN     NOT NULL DEFAULT FALSE,
    -- TRUE = sudah disunting admin → isi bawaan versi baru tak menimpanya.
    edited      BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE themes ADD COLUMN IF NOT EXISTS template        TEXT  NOT NULL DEFAULT '';
ALTER TABLE themes ADD COLUMN IF NOT EXISTS template_assets JSONB NOT NULL DEFAULT '{}'::jsonb;
ALTER TABLE themes ADD COLUMN IF NOT EXISTS template_css    TEXT  NOT NULL DEFAULT '';

-- Kolom dari 017b & 022 (bila DB tertinggal) — definisi sama, idempoten.
ALTER TABLE themes ADD COLUMN IF NOT EXISTS motion_locked BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE themes ADD COLUMN IF NOT EXISTS bg_video   TEXT NOT NULL DEFAULT '';
ALTER TABLE themes ADD COLUMN IF NOT EXISTS open_video TEXT NOT NULL DEFAULT '';

-- Tema pertama berbasis templat: "Kusuma Jawi" (templat `kusuma`) — susunan &
-- gerak meniru undangan Jawa everlove (m07), seluruh aset buatan sendiri.
-- Kolom gaya lama tetap diisi agar kartu katalog & /kelola tetap bertema.
INSERT INTO themes (slug, name, category, nuansa, palette, region, description, tags, badge, rating, reviews,
                    layout, ornament, font, tokens, dark, image_url, image_mode, listed, sort_order,
                    script_font, bg_image, frame_image, card_deco, float_deco, open_anim, page_mode, scroll_anim,
                    gerak_judul, gerak_foto, ken_burns, bg_video, open_video, template)
VALUES ('kusuma-jawi', 'Kusuma Jawi', 'Luxury', 'Jawa', 'gold', 'Adat Jawa',
        'Sampul naik membuka pintu gebyok, wayang & pohon bergoyang di latar video, kartu acara berbingkai ukir — tiap bagian masuk dengan gerak sendiri.',
        '["Templat Khusus", "Video Pembuka", "3D Motion"]', 'Baru', '', '',
        'klasik', 'none', 'cormorant',
        '{"bg": "#f4efe6", "ink": "#3d3428", "card": "#fbf8f2", "gold": "#b9a27c", "line": "#e4dacb", "sage": "#8c7b5f",
          "ink-2": "#5b4f3e", "muted": "#6f624c", "gold-c": "#e9dcc0", "primary": "#88775d", "surface": "#ece4d6",
          "gold-deep": "#7d6a4f", "gold-pale": "#efe4cf", "primary-2": "#7d6a4f", "sage-mist": "#a8916b",
          "gold-light": "#c9b48e", "on-primary": "#ffffff", "surface-low": "#f3ede3", "surface-high": "#e6dccb"}',
        FALSE, '/img/tema/sekar/lili.svg', 'sudut', TRUE, 0,
        'great-vibes', '', '', '', 'none', 'none', 'satu', '',
        '', '', FALSE, '/video/sekar-latar.mp4', '/video/sekar-buka.mp4', 'kusuma')
ON CONFLICT (slug) DO UPDATE SET template = EXCLUDED.template
WHERE themes.template = '';
