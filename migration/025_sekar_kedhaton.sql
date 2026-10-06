-- ═══════════════════════════════════════════════════════════════════════════
-- 025_sekar_kedhaton — tema "Sekar Kedhaton": desain & gerak undangan Jawa
-- premium yang dibedah dari everlove (WordPress/Elementor), dibangun ulang
-- dengan aset sendiri:
--   • pembuka  : video pintu gebyok gading berayun ke dalam, bunga sekar
--                terbelah, kamera masuk ke taman (public/video/sekar-buka.mp4,
--                sumber scripts/gerak/video/door-sekar.html)
--   • latar    : video krem — pohon sepia bergoyang, joglo, burung, kelopak
--                lembayung (sekar-latar.mp4 ← scene-sekar.html), berulang 2,6 dtk→
--   • gulir    : koreografi `sekar` (src/web/gerak/scroll-sekar.css, bawaan —
--                di-seed otomatis saat server start)
--   • ilustrasi: public/img/tema/sekar/ (scripts/gerak/sekar.py)
-- Aman dijalankan ulang (jalankan ULANG setelah tema diperbarui). Urutan disarankan: 017b → 018 … 024 → 025 (file ini
-- menambah sendiri kolom dari 017b & 022 bila belum ada).
-- ═══════════════════════════════════════════════════════════════════════════
-- Kolom dari 017b (source, motion_locked) & 022 (bg_video, open_video) —
-- ditambahkan di sini juga (idempoten, definisi sama) agar file ini tetap
-- jalan di DB yang belum menjalankan migrasi itu.
ALTER TABLE theme_ornaments ADD COLUMN IF NOT EXISTS source TEXT NOT NULL DEFAULT 'admin';
DO $$ BEGIN
    ALTER TABLE theme_ornaments ADD CONSTRAINT theme_ornaments_source_chk CHECK (source IN ('seed', 'admin', 'import'));
EXCEPTION WHEN duplicate_object THEN NULL; END $$;
ALTER TABLE themes ADD COLUMN IF NOT EXISTS motion_locked BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE themes ADD COLUMN IF NOT EXISTS bg_video   TEXT NOT NULL DEFAULT '';
ALTER TABLE themes ADD COLUMN IF NOT EXISTS open_video TEXT NOT NULL DEFAULT '';

INSERT INTO themes (slug, name, category, nuansa, palette, region, description, tags, badge, rating, reviews,
                    layout, ornament, font, tokens, dark, image_url, image_mode, listed, sort_order,
                    script_font, bg_image, frame_image, card_deco, float_deco, open_anim, page_mode, scroll_anim,
                    gerak_judul, gerak_foto, ken_burns, bg_video, open_video)
VALUES ('sekar-kedhaton', 'Sekar Kedhaton', 'Luxury', 'Jawa', 'gold', 'Keraton Surakarta',
        'Pintu gebyok gading terbuka ke taman keraton: pohon sepia bergoyang, bunga lili di sudut foto, dan setiap bagian bergerak lembut saat digulir.',
        '["Video Pembuka", "Foto Bergerak", "Panel Taupe"]', 'Baru', '', '',
        'klasik', 'none', 'cormorant',
        '{"bg": "#f4efe6", "ink": "#3d3428", "card": "#fbf8f2", "gold": "#b9a27c", "line": "#e4dacb", "sage": "#8c7b5f",
          "ink-2": "#5b4f3e", "muted": "#6f624c", "gold-c": "#e9dcc0", "primary": "#5f4c34", "surface": "#ece4d6",
          "gold-deep": "#7d6a4f", "gold-pale": "#efe4cf", "primary-2": "#7d6a4f", "sage-mist": "#a8916b",
          "gold-light": "#c9b48e", "on-primary": "#ffffff", "surface-low": "#f3ede3", "surface-high": "#e6dccb"}',
        FALSE, '/img/tema/sekar/lili.svg', 'sudut', TRUE, 0,
        'great-vibes', '', '', '', 'burung', 'video-pintu', 'satu', 'sekar',
        'ikut', 'ikut', TRUE, '/video/sekar-latar.mp4#loop=2.6', '/video/sekar-buka.mp4')
ON CONFLICT (slug) DO NOTHING;

-- Ornamen bagian (sedikit, takaran everlove): joglo krem di atas "Wedding Event" (panel taupe), lili di galeri & RSVP.
INSERT INTO theme_ornaments (theme, bagian, img, posisi, x, y, lebar, rotasi, cermin, masuk, jeda, durasi, gerak, kecepatan, depan, hp, opasitas, urutan, source)
SELECT 'sekar-kedhaton', v.bagian, '/img/tema/sekar/' || v.img, v.posisi, v.x, v.y, v.lebar, v.rotasi, v.cermin, v.masuk, v.jeda, v.durasi, v.gerak, v.kecepatan, v.depan, v.hp, v.opasitas, v.urutan, 'seed'
  FROM (VALUES
    ('acara',    'pendopo-krem.svg', 'tengah-atas',  0, -36, 70, 0, FALSE, 'naik',       200, 1500, 'none',     6, FALSE, TRUE, 85, 20),
    ('galeri',   'lili.svg',         'kanan-atas',  10, -16, 34, 0, TRUE,  'ayun-kanan', 500, 1500, 'goyang',  10, TRUE,  TRUE, 100, 30),
    ('rsvp',     'lili.svg',         'kiri-atas',  -10, -14, 32, 0, FALSE, 'ayun-kiri',  500, 1500, 'goyang',  10, TRUE,  TRUE, 100, 40)
  ) AS v(bagian, img, posisi, x, y, lebar, rotasi, cermin, masuk, jeda, durasi, gerak, kecepatan, depan, hp, opasitas, urutan)
 WHERE NOT EXISTS (SELECT 1 FROM theme_ornaments o WHERE o.theme = 'sekar-kedhaton');

-- Versi awal tema sempat memasang awan di atas judul mempelai (menabrak judul).
DELETE FROM theme_ornaments WHERE theme = 'sekar-kedhaton' AND bagian = 'mempelai' AND source = 'seed' AND img LIKE '%/sekar/awan.svg';

-- ── CSS koreografi sekar (sinkron dengan src/web/gerak/scroll-sekar.css) ──
-- Gerak bawaan di-seed ON CONFLICT DO NOTHING saat server start → DB yang
-- sudah punya baris `sekar` versi lama perlu diperbarui di sini. Belum ada
-- baris (server belum pernah start) → 0 baris, seed memasukkan versi terbaru.
UPDATE animations SET css = $css$
/* Sekar Kedhaton — desain & gerak undangan Jawa premium, dibedah dari
   everlove (WordPress + Elementor + Animate.css) lalu dibangun ulang tanpa
   library. Takaran yang diukur:
     • kelas inv-* : transisi 1,5 dtk, aktif saat elemen 150px di atas tepi
       bawah layar, DIULANG saat digulir naik-turun (--rv-repeat:1)
     • Animate.css : zoomIn (judul/foto/kartu), fadeInUp (teks), fadeInDown
       (label), rotateInDownLeft/Right (bunga sudut foto), jeda 200–1800ms
     • latar = video (pohon sepia bergoyang, burung, kelopak), foto = Ken Burns
   Desain: panel taupe selang-seling krem (layar HP: selebar layar), foto
   berbingkai KAPSUL bergaris emas + lili di sudut, kartu acara gading
   berbingkai ukir, judul kaligrafi. Isi kartu ikut beranimasi (--rv-deep). */
{a} { --rv-stagger: 0; --rv-repeat: 1; --rv-deep: 1; --gate-burst: none; --card-blur: none;
  --sk-taupe: #8c7b5f; --sk-cream: #f7f3ec;
  --card-glass: color-mix(in srgb, var(--card) 90%, transparent);
  --video-shade: linear-gradient(180deg, rgba(247, 243, 236, .58), rgba(247, 243, 236, .46) 40%, rgba(247, 243, 236, .62)); }

/* ── Sampul (gerbang video-pintu) ── */
/* Selubung TERANG (bukan gelap) di atas video latar krem. */
{a} .gate__bg::after { background: linear-gradient(180deg, rgba(255, 255, 255, .55), rgba(255, 255, 255, .15) 38%, rgba(255, 255, 255, .7)); }
/* Sampul tergulir ke atas 1,5 dtk easeInOutCubic (jQuery slideUp everlove). */
.inv-opened .inv{a} .gate--video-pintu:not(.gate--embed) .gate__content { transition: transform 1.5s cubic-bezier(.65, 0, .35, 1); }
{a} .gate .cover__names { font-family: var(--font-display); font-weight: 500; letter-spacing: .12em; text-transform: uppercase; color: var(--sk-taupe); }
{a} .gate .guest { background: none; box-shadow: none; border: 0; }

/* ── Bingkai foto: kapsul bergaris emas tipis ── */
{a} .arch-photo__clip { border-radius: 999px; box-shadow: 0 0 0 3px var(--sk-cream), 0 0 0 4px #b9a27c, 0 14px 32px rgba(95, 76, 52, .28); }
{a} .story__photo { border-radius: 14px; }
/* Lili di sudut foto: kanan-atas & kiri-bawah, berputar masuk (rotateInDown). */
{a} :is(.arch-photo, .countdown__photo)::before, {a} :is(.arch-photo, .countdown__photo)::after { content: ""; position: absolute; z-index: 2; width: 46%; aspect-ratio: 1; pointer-events: none;
  background: url("/img/tema/sekar/lili.svg") center / contain no-repeat; opacity: 0; }
{a} :is(.arch-photo, .countdown__photo)::before { right: -20%; top: -6%; transform-origin: 100% 100%; scale: -1 1; }
{a} :is(.arch-photo, .countdown__photo)::after { left: -20%; bottom: -8%; transform-origin: 0 100%; rotate: 0deg; }
{a} .countdown__photo::before { width: 30%; right: -9%; top: -14%; }
{a} .countdown__photo::after { width: 26%; left: -8%; bottom: -12%; }
/* Sampul: lili kiri naik ke tengah bingkai agar tak menutupi nama. */
{a} .gate .arch-photo::after { width: 38%; left: -24%; bottom: 14%; }
{a} .gate .arch-photo::before, .rv-on {a} :is(.arch-photo, .countdown__photo).is-in::before, {a} .person.is-in .arch-photo::before { animation: sk-rot-r 1.6s ease .9s both, sk-sway 7s ease-in-out 2.6s infinite; }
{a} .gate .arch-photo::after, .rv-on {a} :is(.arch-photo, .countdown__photo).is-in::after, {a} .person.is-in .arch-photo::after { animation: sk-rot-l 1.6s ease 1.1s both, sk-sway 8s ease-in-out 2.8s infinite reverse; }
@keyframes sk-rot-l { from { opacity: 0; transform: rotate(-45deg); } to { opacity: 1; transform: none; } }
@keyframes sk-rot-r { from { opacity: 0; transform: rotate(45deg); } to { opacity: 1; transform: none; } }
@keyframes sk-sway { 0%, 100% { rotate: 0deg; } 50% { rotate: 3deg; } }

/* ── Panel taupe selang-seling (quote, Save The Date, Love Story, Acara) ── */
{a} :is(.quote, .countdown), {a} .inv__isi > .section:has(> .story), {a} #acara {
  --ink: #fbf6ec; --ink-2: #f1e8d8; --muted: #e6dac4; --gold-deep: #fff8ea; --primary: #fffaf0; --gold: #e9dcc0; --line-gold: rgba(255, 255, 255, .35);
  background: var(--sk-taupe); color: var(--ink); border: 0; box-shadow: none; }
{a} #acara { padding: 30px 0 34px; }
{a} :is(.quote, .countdown) { border-radius: 26px; }
@media (max-width: 1099px) {
  {a} :is(.quote, .countdown), {a} .inv__isi > .section:has(> .story), {a} #acara {
    border-radius: 0; box-shadow: 0 0 0 100vmax var(--sk-taupe); clip-path: inset(-12px -100vmax); }
}
{a} .quote::before { display: none; }
{a} .quote__mark { color: var(--sk-taupe); }
{a} .countdown__tile { border-color: rgba(255, 255, 255, .7); box-shadow: none; }
{a} .countdown__tile b, {a} .countdown__tile span { color: #fff; }
{a} .countdown .btn--outline { color: #fff; border-color: rgba(255, 255, 255, .75); }
/* Kartu di atas panel taupe tetap gading berteks gelap. */
{a} :is(.event, .story, .mapcard, .live-card, .guide) { --ink: #3d3428; --ink-2: #5b4f3e; --muted: #6f624c; --gold-deep: #7d6a4f; --primary: #5f4c34; --gold: #b9a27c; --line-gold: rgba(140, 123, 95, .3); color: var(--ink); }
{a} .story { background: var(--sk-cream); }

/* ── Kartu acara: gading berbingkai ukir (atas & bawah) ── */
{a} .event { background: var(--sk-cream); border: 2px solid #c9b48e; border-radius: 22px; padding-top: 58px; padding-bottom: 54px; }
{a} .event::before { top: 6px; left: 8%; right: 8%; width: auto; height: 34px; margin: 0; transform: none; box-shadow: none; background: #a8916b;
  -webkit-mask: url("/img/tema/gerak/gebyok-mahkota-ukir.svg") center top / 100% auto no-repeat; mask: url("/img/tema/gerak/gebyok-mahkota-ukir.svg") center top / 100% auto no-repeat; }
{a} .event::after { inset: 8px; border-color: rgba(168, 145, 107, .55); }
{a} .event .btn--primary, {a} .event .btn--soft { background: #5f4c34; color: #fff; border-color: #5f4c34; }

/* ── Galeri: kisi rapi 2 kolom, foto pertama selebar penuh ── */
{a} .gallery--masonry { display: grid; columns: auto; grid-template-columns: 1fr 1fr; gap: 10px; }
{a} .gallery--masonry .gallery__item { margin: 0; aspect-ratio: 3 / 4; border-radius: 14px; box-shadow: 0 0 0 3px var(--sk-cream), 0 0 0 4px #c9b48e, 0 10px 24px rgba(95, 76, 52, .2); }
{a} .gallery--masonry .gallery__item:first-child { grid-column: 1 / -1; aspect-ratio: 4 / 3; }
{a} .gallery--masonry .gallery__item img { height: 100%; object-fit: cover; aspect-ratio: auto; }

/* ── Tanpa bagian bergaya aplikasi (kartu acara sudah punya tombol peta) ── */
{a} :is(.guide, .mapcard) { display: none; }

/* ── RSVP & ucapan: gaya kartu undangan, bukan formulir aplikasi ── */
{a} .rsvp .field__label .ms, {a} .wishes__head .ms { display: none; }
{a} .rsvp .field__label { font-family: var(--font-display); font-size: 15px; letter-spacing: .02em; color: var(--ink); }
{a} .rsvp .input { background: #fff; border: 1px solid #d9cdb6; border-radius: 12px; }
{a} .rsvp .input:focus { border-color: var(--sk-taupe); }
{a} .seg__opt { background: #fff; border: 1px solid #d9cdb6; border-radius: 999px; padding: 10px 6px; }
{a} .seg__opt:has(input:checked) { background: var(--sk-taupe); border-color: var(--sk-taupe); color: #fff; }
{a} .wishes__head h3 { font-family: var(--font-script); font-weight: 400; font-size: 34px; color: var(--gold-deep); }
{a} .wishes__head .chip { background: none; border: 1px solid #c9b48e; color: var(--gold-deep); }
{a} .wish { background: var(--sk-cream); border: 1px solid #e4dacb; box-shadow: none; }
{a} .avatar { background: var(--sk-taupe); color: #fff; font-family: var(--font-display); }
{a} .status, {a} .status--hadir, {a} .status--ragu, {a} .status--tidak { background: #efe4cf; color: #6e5a3f; }
{a} .gift { background: var(--sk-cream); }
{a} .gift__summary b { font-family: var(--font-script); font-weight: 400; font-size: 26px; line-height: 1.2; color: var(--gold-deep); }
{a} .gift__icon { display: none; }

/* ── Kartu rekening ATM bergelombang taupe ── */
{a} .bank { background:
    radial-gradient(140% 70% at 0% 110%, rgba(255, 255, 255, .22), transparent 55%),
    radial-gradient(120% 60% at 100% -10%, rgba(255, 255, 255, .18), transparent 60%),
    linear-gradient(135deg, #a8916b, #7d6a4f); }

/* ── GERAK SAAT DIGULIR (diulang naik-turun) ── */
.rv-on {a} [data-rv] { transition: opacity 1.5s ease, transform 1.5s ease; }
.rv-on {a} [data-rv]:not(.is-in) { opacity: 0; transform: translateY(min(50%, 120px)); filter: none; }
/* Wadah tidak digeser — isinya yang beranimasi. Panel (bagian, Save The
   Date, kutipan) cukup memudar masuk SEKALI (--rv-repeat:0 → pengamat sekali
   jalan) agar tak tampil kosong sebelum isinya masuk; isinya tetap diulang. */
{a} :is(.section, .countdown, .quote) { --rv-repeat: 0; }
{a} :is(.section, .countdown, .quote) > * { --rv-repeat: 1; }
.rv-on {a} :is(.inv__isi, .inv__anchor, .couple, .story)[data-rv]:not(.is-in) { opacity: 1; transform: none; }
.rv-on {a} :is(.section, .countdown, .quote)[data-rv] { transition: opacity 1.2s ease; }
.rv-on {a} :is(.section, .countdown, .quote)[data-rv]:not(.is-in) { opacity: 0; transform: none; }
.rv-on {a} .story[data-rv] { transition: none; }
/* fadeInDown — label kecil */
.rv-on {a} :is(.eyebrow, .event__badge, .chip)[data-rv]:not(.is-in) { transform: translateY(-100%); }
.rv-on {a} :is(.eyebrow, .event__badge, .chip)[data-rv].is-in { transition-delay: .2s; }
/* zoomIn (Animate.css) — judul, nama, kotak hitung mundur, kartu */
.rv-on {a} :is(.section__title, .countdown__title, .event__title, .event__num, .person__nick, .thanks__title, .thanks__names, .script)[data-rv] { transition: opacity .75s ease, transform 1.25s cubic-bezier(.2, .8, .3, 1.04); }
.rv-on {a} :is(.section__title, .countdown__title, .event__title, .event__num, .person__nick, .thanks__title, .thanks__names, .script)[data-rv]:not(.is-in) { transform: scale(.3); }
.rv-on {a} :is(.section__title, .countdown__title, .thanks__title)[data-rv].is-in { transition-delay: .3s; }
.rv-on {a} :is(.event__title, .person__nick, .thanks__names)[data-rv].is-in { transition-delay: .5s; }
.rv-on {a} .event__num[data-rv].is-in { transition-delay: .8s; }
.rv-on {a} .event[data-rv]:not(.is-in) { transform: scale(.85); }
/* inv-zoom-in — foto */
.rv-on {a} :is(.arch-photo, .gallery__item, .video-frame, .story__photo)[data-rv]:not(.is-in) { transform: scale(.9); }
.rv-on {a} :is(.arch-photo, .gallery__item, .video-frame)[data-rv].is-in { transition-delay: .4s; }
.rv-on {a} .gallery__item[data-rv]:nth-child(even).is-in { transition-delay: .6s; }
/* inv-kiri / inv-kanan — mempelai, kisah & kartu rekening bergantian */
.rv-on {a} :is(.couple > .person:first-child, .story > li:nth-child(odd), .gift .bank:nth-of-type(odd))[data-rv]:not(.is-in) { transform: translateX(-70%) scale(.93); }
.rv-on {a} :is(.couple > .person:last-child, .story > li:nth-child(even), .gift .bank:nth-of-type(even))[data-rv]:not(.is-in) { transform: translateX(70%) scale(.93); }
.rv-on {a} :is(.couple > .person, .story > li, .gift .bank)[data-rv] { transition: opacity .5s ease, transform 1.5s ease; }
/* inv-atas — teks & isi kartu, bertahap */
.rv-on {a} :is(.person__body > *, .event > p, .event__venue, .event__sessions, .countdown__date, .countdown__grid, .intro__text, .thanks__text)[data-rv].is-in { transition-delay: .6s; }
.rv-on {a} :is(.event__date, .event__time)[data-rv].is-in { transition-delay: .7s; }
.rv-on {a} :is(.event > .btn, .countdown__cal, .event__dress)[data-rv].is-in { transition-delay: 1s; }
.rv-on {a} :is(.event > .btn, .countdown__cal)[data-rv]:not(.is-in) { transform: scale(.3); }
/* inv-rotate-in — ikon & tanda & */
.rv-on {a} :is(.intro__icon, .couple__amp, .quote__mark)[data-rv] { transition: opacity 1.5s ease-out, transform 1.5s ease-out; }
.rv-on {a} :is(.intro__icon, .couple__amp, .quote__mark)[data-rv]:not(.is-in) { transform: rotate(-180deg); }

/* Ikon "gulir" (Lottie mouse everlove): muncul setelah pintu, lalu hilang. */
.inv-opened {a} .inv__isi::before { content: ""; position: fixed; z-index: 30; left: 50%; bottom: 96px; width: 26px; height: 40px; margin-left: -13px;
  border: 2px solid rgba(255, 255, 255, .95); border-radius: 14px; box-shadow: 0 2px 10px rgba(95, 76, 52, .35);
  background: radial-gradient(circle, #fff 0 3px, transparent 3.5px) 50% 8px / 8px 8px no-repeat;
  pointer-events: none; opacity: 0; animation: sk-mouse 1.4s ease-in-out 6.2s 5 both, sk-fade 8s linear 6.2s both; }
@keyframes sk-mouse { 0% { background-position: 50% 6px; } 60% { background-position: 50% 20px; } 100% { background-position: 50% 6px; } }
@keyframes sk-fade { 0% { opacity: 0; } 8%, 85% { opacity: 1; } 100% { opacity: 0; } }
$css$
WHERE kind = 'scroll' AND slug = 'sekar' AND builtin;
