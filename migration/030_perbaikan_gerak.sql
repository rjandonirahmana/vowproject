-- ═══════════════════════════════════════════════════════════════════════════
-- 030_perbaikan_gerak — perbaikan gerak tema hasil audit Playwright 8 Okt 2026
-- (aman dijalankan ulang).
--
-- 1. Buka `candi-bentar` (bali-frangipani): gapura TAK PERNAH terlihat —
--    keyframe cb-stand hanya punya `from`, sehingga nilai akhirnya mengambil
--    opacity:0 tahap akhir. Kini tiga animasi berurutan: berdiri → melangkah
--    masuk → memudar.
-- 2. Scroll `tenun` (minang-songket): kartu mempelai WANITA tak pernah muncul —
--    digeser −30px sambil dipotong clip-path selebar nol, kotak deteksi
--    IntersectionObserver jatuh di luar kolom. Kini tanpa geser samping &
--    menyisakan irisan 2px (tetap tak tampak: opacity 0).
-- 3. Scroll `mekar`: lingkaran foto 0% → 2px (pencegahan pola yang sama).
--
-- Bawaan di-seed ON CONFLICT DO NOTHING → perubahan src/web/gerak/*.css tak
-- sampai ke DB tanpa file ini. Sinkronkan bila berkas CSS itu diubah lagi.
-- ═══════════════════════════════════════════════════════════════════════════

UPDATE animations SET css = $css$
/* Candi Bentar — saat dibuka sampul naik & memudar, lalu gapura terbelah khas
   Bali berdiri dari bawah memenuhi layar dengan matahari terbit di celahnya
   dan kabut di kakinya; kemudian kedua belahan bergeser menjauh sambil
   membesar (seperti melangkah masuk pura) dan isi undangan muncul.
   Batu = --primary, ukiran = --foil-deep. */
{a} { --gate-reveal: 1900; }
{a}::before { content: ""; position: absolute; inset: 0; z-index: 0;
  background: radial-gradient(circle at 50% 66%, color-mix(in srgb, var(--gold-c, #fed488) 80%, #fff) 0 7%, color-mix(in srgb, var(--gold-c, #fed488) 40%, transparent) 14%, transparent 40%), linear-gradient(180deg, color-mix(in srgb, var(--gold-pale, #ffdea5) 45%, var(--bg)), var(--bg) 70%);
  transition: opacity .8s ease 2s, filter 1s ease .6s; animation: cb-sun 5s ease-in-out infinite alternate; }
{a} .gate__content { z-index: 3; background: color-mix(in srgb, var(--bg) 72%, transparent); -webkit-backdrop-filter: blur(3px); backdrop-filter: blur(3px); border-radius: 26px; max-width: 420px; width: calc(100% - 40px); padding: 22px 16px; box-shadow: 0 18px 40px -18px rgba(0, 0, 0, .35); }
{a} .cover__eyebrow { animation: cb-drop 1.1s cubic-bezier(.3, 1.4, .5, 1) .5s both; }
{a} .arch-photo, {a} .hero__seal { animation: cb-tilt 1.3s cubic-bezier(.2, .8, .2, 1) .7s both; }
{a} .cover__names { animation: cb-drop 1.1s cubic-bezier(.3, 1.4, .5, 1) 1s both; }
{a} .hero__date, {a} .cover .pill, {a} .guest { animation: cb-wave 1.1s ease 1.25s both; }
{a} .gate__open { animation: cb-wave 1s ease 1.5s both, gate-pulse 2.4s ease-in-out 2.5s infinite; }
/* Dua belahan gapura (kanan = cermin), berdiri dari bawah saat dimuat. */
{a} .gate__panel { top: 0; bottom: 0; width: 50%; z-index: 4; background: none; box-shadow: none; pointer-events: none; visibility: hidden; }
{a} .gate__panel--l { left: 0; transform-origin: 50% 100%; }
{a} .gate__panel--r { right: 0; left: auto; scale: -1 1; transform-origin: 50% 100%; }
{a} .gate__panel::before, {a} .gate__panel::after { content: ""; position: absolute; inset: 0; opacity: 1; }
{a} .gate__panel::before { background: linear-gradient(180deg, color-mix(in srgb, var(--primary) 80%, var(--gold)), var(--primary) 40%, color-mix(in srgb, var(--primary) 70%, #000));
  -webkit-mask: url("/img/tema/gerak/candi-bentar.svg") right bottom / contain no-repeat; mask: url("/img/tema/gerak/candi-bentar.svg") right bottom / contain no-repeat; filter: drop-shadow(6px 0 14px rgba(0, 0, 0, .3)); }
{a} .gate__panel::after { background: var(--foil-deep, var(--gold)); opacity: .75;
  -webkit-mask: url("/img/tema/gerak/candi-ukir.svg") right bottom / contain no-repeat; mask: url("/img/tema/gerak/candi-ukir.svg") right bottom / contain no-repeat; }
/* Kabut di kaki gapura: tiga gumpal melayang pelan. */
{a} .gate__fx { display: block; z-index: 2; }
.inv-opened {a}:not(.gate--embed) .gate__fx, .is-open > {a} .gate__fx { z-index: 5; }
{a} .gate__fx i { bottom: calc(-6% + var(--i) * 2%); left: calc(-30% + var(--i) * 30%); width: 90%; height: 18%; border-radius: 50%; background: radial-gradient(closest-side, color-mix(in srgb, #fff 75%, var(--bg)), transparent); opacity: .8; filter: blur(6px); animation: cb-mist calc(9s + var(--i) * 3s) ease-in-out infinite alternate; }
{a} .gate__fx i:nth-child(n+4) { display: none; }
.inv-opened {a}:not(.gate--embed), .is-open > {a} { transition: visibility 0s 3.1s; }
.inv-opened {a}:not(.gate--embed)::before, .is-open > {a}::before { opacity: 0; filter: brightness(1.15) saturate(1.2); }
.inv-opened {a}:not(.gate--embed) .gate__content, .is-open > {a} .gate__content { opacity: 0; transform: translateY(-70px) scale(.94); transition: opacity .6s ease, transform .8s cubic-bezier(.5, 0, .3, 1); }
/* Tiga tahap berurutan (dulu satu animasi + transisi: keyframe tanpa `to`
   mengambil opacity:0 tahap akhir → gapura tak pernah terlihat). Berdiri dari
   bawah → melangkah masuk (bergeser & membesar) → memudar. */
.inv-opened {a}:not(.gate--embed) .gate__panel, .is-open > {a} .gate__panel { visibility: visible;
  animation: cb-stand 1s cubic-bezier(.2, .8, .2, 1) .35s both, cb-go 1.6s cubic-bezier(.55, 0, .2, 1) 1.45s forwards, cb-fade .5s ease 2.55s forwards; }
.inv-opened {a}:not(.gate--embed) .gate__fx i, .is-open > {a} .gate__fx i { animation: none; opacity: 0; transform: translateY(40%) scale(1.6); transition: all 1.3s ease 1.5s; }
@keyframes cb-sun { to { filter: brightness(1.08) saturate(1.1); } }
@keyframes cb-stand { from { translate: 0 35%; opacity: 0; } to { translate: 0 0; opacity: 1; } }
@keyframes cb-go { to { transform: translateX(-80%) scale(1.35); } }
@keyframes cb-fade { to { opacity: 0; } }
@keyframes cb-mist { to { translate: 12% -6%; } }
@keyframes cb-drop { from { opacity: 0; transform: translateY(-50px); } }
@keyframes cb-tilt { from { opacity: 0; transform: perspective(700px) rotateY(35deg) scale(.85); } }
@keyframes cb-wave { from { opacity: 0; transform: translateX(-30px) rotate(-3deg); } }
$css$ WHERE kind = 'buka' AND slug = 'candi-bentar';

UPDATE animations SET css = $css$
/* Tenun — isi "ditenun" seperti benang songket: teks & kartu tersingkap dari
   kiri (genap dari kanan), judul terbuka dari tengah ke tepi, foto terbuka
   seperti tirai atas-bawah, kartu terurai dari atas, ikon merentang.
   ATURAN: keadaan tersembunyi tak boleh memotong elemen selebar NOL atau
   menggesernya ke samping — IntersectionObserver memotong kotak deteksi dengan
   clip-path elemen & overflow leluhurnya; irisan nol di luar kolom (dulu
   translateX(-30px) + inset 100%) = tak pernah "terlihat" → kartu mempelai
   wanita tak pernah muncul. Sisakan irisan 2px (tak tampak: opacity 0). */
{a} { --rv-stagger: 150; }
.rv-on {a} [data-rv] { transition: opacity .6s ease, transform 1.1s cubic-bezier(.7, 0, .25, 1), clip-path 1.1s cubic-bezier(.7, 0, .25, 1); clip-path: inset(-60px -60px -60px -60px); }
.rv-on {a} [data-rv]:not(.is-in) { opacity: 0; transform: none; filter: none; clip-path: inset(0 calc(100% - 2px) 0 0); }
.rv-on {a} [data-rv]:nth-child(even):not(.is-in) { clip-path: inset(0 0 0 calc(100% - 2px)); }
.rv-on {a} :is(.section__title, .cover__names, .guest__name)[data-rv]:not(.is-in) { clip-path: inset(0 calc(50% - 1px) 0 calc(50% - 1px)); }
.rv-on {a} :is(.arch-photo, .gallery__item, .hero__seal)[data-rv]:not(.is-in) { transform: scale(1.08); clip-path: inset(calc(50% - 1px) 0 calc(50% - 1px) 0); }
.rv-on {a} .couple > .person:first-child[data-rv]:not(.is-in) { clip-path: inset(0 calc(100% - 2px) 0 0); }
.rv-on {a} .couple > .person:last-child[data-rv]:not(.is-in) { clip-path: inset(0 0 0 calc(100% - 2px)); }
.rv-on {a} :is(.card, .event):not(.person)[data-rv]:not(.is-in) { transform: translateY(-12px); clip-path: inset(0 0 calc(100% - 2px) 0); }
.rv-on {a} :is(.intro__icon, .couple__amp, .eyebrow)[data-rv]:not(.is-in) { transform: scaleX(.02); clip-path: inset(-60px -60px -60px -60px); }
$css$ WHERE kind = 'scroll' AND slug = 'tenun';

UPDATE animations SET css = $css$
/* Mekar — isi "mekar" seperti bunga: judul merapat dari huruf renggang &
   samar, foto terbuka dari lingkaran kecil di tengah, mempelai mekar memantul,
   kartu terbuka seperti kelopak (rotasi 3D dari atas), ikon berputar mekar. */
{a} { --rv-stagger: 160; }
.rv-on {a} [data-rv] { transition: opacity 1.2s ease, transform 1.3s cubic-bezier(.2, .7, .2, 1), filter 1.2s ease, letter-spacing 1.4s ease, clip-path 1.6s cubic-bezier(.3, .6, .2, 1); }
.rv-on {a} [data-rv]:not(.is-in) { opacity: 0; transform: translateY(24px) scale(.92); filter: none; }
.rv-on {a} :is(.section__title, .cover__names, .guest__name)[data-rv]:not(.is-in) { transform: none; letter-spacing: .35em; filter: blur(6px); }
.rv-on {a} :is(.arch-photo, .gallery__item, .hero__seal)[data-rv] { clip-path: circle(80% at 50% 50%); }
.rv-on {a} :is(.arch-photo, .gallery__item, .hero__seal)[data-rv]:not(.is-in) { opacity: 1; transform: scale(1.12); clip-path: circle(2px at 50% 50%); }
.rv-on {a} .couple > .person[data-rv] { transition-timing-function: ease, cubic-bezier(.34, 1.56, .64, 1); }
.rv-on {a} .couple > .person:first-child[data-rv]:not(.is-in) { transform: scale(.6) rotate(-6deg); }
.rv-on {a} .couple > .person:last-child[data-rv]:not(.is-in) { transform: scale(.6) rotate(6deg); }
.rv-on {a} :is(.card, .event):not(.person)[data-rv] { transform-origin: top center; }
.rv-on {a} :is(.card, .event):not(.person)[data-rv]:not(.is-in) { transform: perspective(800px) rotateX(22deg) translateY(30px); }
.rv-on {a} :is(.intro__icon, .couple__amp)[data-rv]:not(.is-in) { transform: scale(0) rotate(-120deg); }
$css$ WHERE kind = 'scroll' AND slug = 'mekar';
