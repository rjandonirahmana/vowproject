-- ═══════════════════════════════════════════════════════════════════════════
-- 023_wayang_pintu — tema Gunungan Wayang Kulit (aman dijalankan ulang).
--
-- 1. Koreografi scroll `everlove`: translateY(50%) ikut menggeser wadah
--    .inv__anchor (±2.450px) → celah kosong ±1.200px setelah galeri & kartu
--    acara tak muncul saat tamu melompat lewat nav bawah. Kini dijepit
--    min(50%, 140px) dan wadah .inv__isi/.inv__anchor tak digeser.
--    Kartu kaca 62% → 80% agar dahan pohon latar tak menembus teks.
-- 2. Animasi buka `video-pintu`: --gate-reveal 2600 → 1400 (isi masuk saat
--    pemandangan latar masih tumbuh, bukan setelah layar gelap lama).
-- 3. Video pintu baru (scripts/gerak/video/door.html): daun pintu terayun KE
--    DALAM, gunungan terbelah, cahaya blencong, kamera masuk. URL diberi ?v=2
--    agar peramban tak memakai salinan video lama.
--
-- Bawaan di-seed ON CONFLICT DO NOTHING → perubahan src/web/gerak/*.css tak
-- sampai ke DB tanpa file ini.
-- ═══════════════════════════════════════════════════════════════════════════
UPDATE animations SET css = $css$
/* Everlove — resep gerak scroll undangan premium everlove, DIUKUR langsung
   dari CSS-nya (kelas inv-*): transisi 1,5 dtk (ease), TANPA jeda bergiliran,
   aktif saat elemen 150px di atas tepi bawah layar dan DIULANG saat digulir
   naik-turun (--rv-repeat). Pemetaan peran:
     teks, kartu, mempelai  → inv-atas    translateY(50%)
     judul, foto, galeri    → inv-zoom-in scale(.9)
     ikon & tanda "&"       → inv-rotate-in rotate(-180deg) ease-out
   Tanpa portal ruangan & semburan; latar video tetap terang (selubung tipis). */
{a} { --rv-stagger: 0; --rv-repeat: 1; --gate-burst: none; --card-glass: color-mix(in srgb, var(--card) 80%, transparent); --card-blur: none;
  --video-shade: linear-gradient(180deg, rgba(0, 0, 0, .2), rgba(0, 0, 0, .06) 40%, rgba(0, 0, 0, .28)); }
.rv-on {a} [data-rv] { --rv-delay: 0ms; transition: transform 1.5s, opacity 1.5s; }
/* 50% dijepit 140px: kartu tinggi (mempelai ±900px) tak terdorong ratusan px. */
.rv-on {a} [data-rv]:not(.is-in) { opacity: 0; transform: translateY(min(50%, 140px)); filter: none; }
/* Wadah halaman (isi/acara/rsvp ±2.000px) tak ikut digeser — anak-anaknya
   sudah beranimasi sendiri. Dulu translateY(50%) wadah = celah kosong ±1.200px
   dan isinya tak pernah "masuk layar" saat tamu melompat lewat nav bawah. */
.rv-on {a} :is(.inv__isi, .inv__anchor)[data-rv]:not(.is-in) { opacity: 1; transform: none; }
.rv-on {a} :is(.section__title, .cover__names, .guest__name, .arch-photo, .gallery__item, .hero__seal, .video-frame)[data-rv]:not(.is-in) { transform: scale(.9); }
.rv-on {a} :is(.intro__icon, .couple__amp)[data-rv] { transition: transform 1.5s ease-out, opacity 1.5s ease-out; }
.rv-on {a} :is(.intro__icon, .couple__amp)[data-rv]:not(.is-in) { transform: rotate(-180deg); }
$css$
WHERE kind = 'scroll' AND slug = 'everlove' AND builtin;

UPDATE animations SET css = $css$
/* Video Pintu — cara everlove: pembuka = VIDEO (bukan animasi CSS).
   Sampul masuk ala Animate.css "slow" (2 dtk: zoomIn / fadeInUp). Saat dibuka
   sampul naik cepat (0,4 dtk) memperlihatkan video pintu (theme.open_video)
   yang diputar sekali; /app.js menandai gerbang .is-done saat video SELESAI
   → gerbang hilang, frame akhirnya menyambung ke video latar. */
{a} { --gate-reveal: 1400; }
{a} .gate__video { display: block; position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; background: var(--bg); visibility: hidden; }
{a} .gate__panel { display: none; }
{a} { background: var(--bg); }
/* Sampul bergerak: video latar (potongan ulang) + selubung agar teks terbaca. */
{a} .gate__bg { display: block; position: absolute; inset: 0; z-index: 1; }
{a} .gate__bg video { width: 100%; height: 100%; object-fit: cover; display: block; }
{a} .gate__bg::after { content: ""; position: absolute; inset: 0; background: linear-gradient(180deg, rgba(0, 0, 0, .55), rgba(0, 0, 0, .25) 35%, rgba(0, 0, 0, .6)); }
{a} .gate__video { z-index: 2; }
{a} .gate__content { z-index: 3; }
{a} .cover__eyebrow { animation: vp-zoom 2s ease .2s both; }
{a} .arch-photo, {a} .hero__seal { animation: vp-zoom 2s ease .4s both; }
{a} .cover__names { animation: vp-zoom 2s ease 1s both; }
{a} .hero__date, {a} .cover .pill { animation: vp-up 2s ease 1.2s both; }
{a} .guest { animation: vp-up 2s ease 1.4s both; }
{a} .gate__open { animation: vp-zoom 2s ease 1.8s both, gate-pulse 2.4s ease-in-out 3.8s infinite; }
/* Terbuka: tetap terlihat (video pintu berjalan) sampai .is-done. */
.inv-opened {a}:not(.gate--embed):not(.is-done), .is-open > {a}:not(.is-done) { visibility: visible; transition: none; }
.inv-opened {a}:not(.gate--embed).is-done, .is-open > {a}.is-done { visibility: hidden; transition: none; }
/* Hanya selama video berjalan — anak ber-visibility:visible mengalahkan
   gerbang yang sudah hidden, jadi WAJIB dibatasi :not(.is-done). */
.inv-opened {a}:not(.gate--embed):not(.is-done) .gate__video, .is-open > {a}:not(.is-done) .gate__video { visibility: visible; }
.inv-opened {a}:not(.gate--embed) .gate__content, .is-open > {a} .gate__content { opacity: 1; transform: translateY(-112%); transition: transform .4s cubic-bezier(.6, 0, .4, 1); }
@keyframes vp-zoom { 0% { opacity: 0; transform: scale3d(.3, .3, .3); } 50% { opacity: 1; } }
@keyframes vp-up { from { opacity: 0; transform: translate3d(0, 100%, 0); } to { opacity: 1; transform: none; } }
$css$
WHERE kind = 'buka' AND slug = 'video-pintu' AND builtin;

UPDATE themes SET open_video = '/video/wayang-buka.mp4?v=2'
WHERE open_video = '/video/wayang-buka.mp4';
