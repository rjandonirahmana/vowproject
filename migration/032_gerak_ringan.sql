-- ═══════════════════════════════════════════════════════════════════════════
-- 032_gerak_ringan — "ngadet" saat membuka & menggulir undangan (audit
-- performa 8 Okt 2026: emulasi Pixel, CPU 4× lebih lambat, trace raster).
-- Aman dijalankan ulang.
--
-- Semua gerak di bawah kini HANYA opacity + transform (dikerjakan GPU):
--   * scroll kosmik, mekar, bayang, sinema — dulu mem-blur seluruh section &
--     menganimasikan letter-spacing / text-shadow (layout + lukis ulang per
--     frame tepat saat tamu menggulir);
--   * buka galaksi, taman-daun, pagelaran-wayang — isi sampul tak di-blur lagi
--     saat dibuka;
--   * buka tenun-songket — kilau 12 helai lewat transform, bukan
--     background-position tanpa henti;
--   * buka candi-bentar — matahari "bernapas" lewat opacity, bukan filter
--     layar penuh tanpa henti;
--   * buka galaksi — sampul diam tak lagi melukis ulang langit (bintang
--     background-position 2 lapis + filter bulan = ±11 dtk raster/3 dtk);
--     mask portal hanya saat dibuka;
--   * scroll tenun — clip-path hanya untuk judul & foto.
-- Portal "ruangan" (koreografi keraton) & denyut tombol ada di main.css (ikut
-- rilis aplikasi, tak perlu migrasi).
--
-- Bawaan di-seed ON CONFLICT DO NOTHING → perubahan src/web/gerak/*.css tak
-- sampai ke DB tanpa file ini.
-- ═══════════════════════════════════════════════════════════════════════════


UPDATE animations SET css = $css$
/* Kosmik — isi datang dari kedalaman langit: tiap elemen mendekat dari jauh,
   judul membesar pelan seperti bintang yang mendekat, foto terbit dari kecil,
   mempelai berbalik 3D, kartu melayang dari jauh, ikon jatuh seperti bintang.
   Hanya opacity + transform (dikerjakan GPU) — blur / text-shadow /
   letter-spacing dulu melukis ulang seluruh section tiap frame saat digulir. */
{a} { --rv-stagger: 180; }
.rv-on {a} [data-rv] { transition: opacity 1.4s ease, transform 1.5s cubic-bezier(.16, .8, .24, 1); }
.rv-on {a} [data-rv]:not(.is-in) { opacity: 0; transform: scale(1.12); }
.rv-on {a} :is(.section__title, .cover__names, .guest__name)[data-rv]:not(.is-in) { transform: scale(.9); }
.rv-on {a} :is(.arch-photo, .gallery__item, .hero__seal)[data-rv]:not(.is-in) { transform: scale(.85); }
.rv-on {a} .couple > .person:first-child[data-rv]:not(.is-in) { transform: perspective(900px) rotateY(70deg); }
.rv-on {a} .couple > .person:last-child[data-rv]:not(.is-in) { transform: perspective(900px) rotateY(-70deg); }
.rv-on {a} :is(.card, .event):not(.person)[data-rv]:not(.is-in) { transform: perspective(1000px) translateZ(-220px) translateY(40px); }
.rv-on {a} :is(.intro__icon, .couple__amp, .eyebrow)[data-rv]:not(.is-in) { transform: translateY(-80px) scale(2); }
$css$ WHERE kind = 'scroll' AND slug = 'kosmik';

UPDATE animations SET css = $css$
/* Mekar — isi "mekar" seperti bunga: judul merapat dari lebar, foto terbuka
   dari lingkaran kecil di tengah, mempelai mekar memantul, kartu terbuka
   seperti kelopak (rotasi 3D dari atas), ikon berputar mekar.
   Judul memakai scaleX (GPU) — bukan letter-spacing + blur (tata letak &
   lukis ulang tiap frame). Irisan 2px: lihat migrasi 030. */
{a} { --rv-stagger: 160; }
.rv-on {a} [data-rv] { transition: opacity 1.2s ease, transform 1.3s cubic-bezier(.2, .7, .2, 1), clip-path 1.6s cubic-bezier(.3, .6, .2, 1); }
.rv-on {a} [data-rv]:not(.is-in) { opacity: 0; transform: translateY(24px) scale(.92); }
.rv-on {a} :is(.section__title, .cover__names, .guest__name)[data-rv]:not(.is-in) { transform: scaleX(1.3); }
.rv-on {a} :is(.arch-photo, .gallery__item, .hero__seal)[data-rv] { clip-path: circle(80% at 50% 50%); }
.rv-on {a} :is(.arch-photo, .gallery__item, .hero__seal)[data-rv]:not(.is-in) { opacity: 1; transform: scale(1.12); clip-path: circle(2px at 50% 50%); }
.rv-on {a} .couple > .person[data-rv] { transition-timing-function: ease, cubic-bezier(.34, 1.56, .64, 1); }
.rv-on {a} .couple > .person:first-child[data-rv]:not(.is-in) { transform: scale(.6) rotate(-6deg); }
.rv-on {a} .couple > .person:last-child[data-rv]:not(.is-in) { transform: scale(.6) rotate(6deg); }
.rv-on {a} :is(.card, .event):not(.person)[data-rv] { transform-origin: top center; }
.rv-on {a} :is(.card, .event):not(.person)[data-rv]:not(.is-in) { transform: perspective(800px) rotateX(22deg) translateY(30px); }
.rv-on {a} :is(.intro__icon, .couple__amp)[data-rv]:not(.is-in) { transform: scale(0) rotate(-120deg); }
$css$ WHERE kind = 'scroll' AND slug = 'mekar';

UPDATE animations SET css = $css$
/* Bayang — gerak ala wayang kulit: elemen naik dari bawah kelir lalu tegak;
   mempelai masuk dari kiri & kanan seperti tokoh wayang dimainkan dalang
   (miring lalu tegak), judul mendekat dari kelir, ikon & foto bergoyang masuk
   pada porosnya. Hanya opacity + transform (dikerjakan GPU) — filter
   brightness/blur & text-shadow dulu melukis ulang tiap section saat digulir. */
{a} { --rv-stagger: 200; }
.rv-on {a} [data-rv] { transition: opacity 1.1s ease, transform 1.3s cubic-bezier(.2, .75, .25, 1); }
.rv-on {a} [data-rv]:not(.is-in) { opacity: 0; transform: translateY(30px) scale(1.04); }
.rv-on {a} :is(.card, .event)[data-rv]:not(.is-in) { transform: translateY(46px); }
.rv-on {a} :is(.section__title, .cover__names, .guest__name)[data-rv]:not(.is-in) { transform: scale(1.1); }
.rv-on {a} :is(.arch-photo, .gallery__item, .hero__seal)[data-rv] { transform-origin: 50% 100%; }
.rv-on {a} :is(.arch-photo, .gallery__item, .hero__seal)[data-rv]:not(.is-in) { transform: rotate(-9deg) translateY(30px); }
.rv-on {a} :is(.arch-photo, .gallery__item, .hero__seal)[data-rv]:nth-child(even):not(.is-in) { transform: rotate(9deg) translateY(30px); }
.rv-on {a} .couple > .person[data-rv] { transform-origin: 50% 100%; transition-timing-function: ease, cubic-bezier(.3, 1.3, .5, 1); }
.rv-on {a} .couple > .person:first-child[data-rv]:not(.is-in) { transform: translateX(-90px) rotate(-10deg); }
.rv-on {a} .couple > .person:last-child[data-rv]:not(.is-in) { transform: translateX(90px) rotate(10deg); }
.rv-on {a} :is(.intro__icon, .couple__amp)[data-rv]:not(.is-in) { transform: scale(.4) rotate(-40deg); }
$css$ WHERE kind = 'scroll' AND slug = 'bayang';

UPDATE animations SET css = $css$
/* Sinema — untuk tema video latar: isi memudar naik halus di atas video,
   jeda per PERAN (label → judul → foto → teks), tanpa blur (video sudah
   bergerak — isi cukup tenang), tanpa portal & semburan. Kartu kaca gelap. */
{a} { --rv-stagger: 0; --gate-burst: none; --card-glass: color-mix(in srgb, var(--card) 58%, transparent); --card-blur: none; }
.rv-on {a} [data-rv] { --rv-delay: 80ms; transition: opacity 1s ease, transform 1.1s cubic-bezier(.2, .7, .2, 1); }
.rv-on {a} [data-rv]:not(.is-in) { opacity: 0; transform: translateY(22px); filter: none; }
.rv-on {a} :is(.eyebrow, .cover__eyebrow, .script)[data-rv] { --rv-delay: 120ms; }
.rv-on {a} :is(.eyebrow, .cover__eyebrow, .script)[data-rv]:not(.is-in) { transform: none; }
.rv-on {a} :is(.section__title, .cover__names, .guest__name)[data-rv] { --rv-delay: 260ms; transition: opacity 1.2s ease, transform 1.2s cubic-bezier(.2, .7, .2, 1); }
.rv-on {a} :is(.section__title, .cover__names, .guest__name)[data-rv]:not(.is-in) { transform: translateY(10px) scaleX(1.06); }
.rv-on {a} :is(.arch-photo, .gallery__item, .hero__seal, .video-frame)[data-rv] { --rv-delay: 420ms; }
.rv-on {a} :is(.arch-photo, .gallery__item, .hero__seal, .video-frame)[data-rv]:not(.is-in) { transform: translateY(26px) scale(.97); }
.rv-on {a} .couple > .person:last-child[data-rv] { --rv-delay: 560ms; }
.rv-on {a} :is(.card, .event, p:not(.eyebrow, .script, .cover__eyebrow), .intro__text, .countdown, .story)[data-rv] { --rv-delay: 560ms; }
$css$ WHERE kind = 'scroll' AND slug = 'sinema';

UPDATE animations SET css = $css$
/* Tenun — isi "ditenun" seperti benang songket: judul terbuka dari tengah ke
   tepi, foto terbuka seperti tirai atas-bawah (clip-path — ciri khasnya, hanya
   untuk judul & foto), teks & kartu bergeser masuk dari kiri (genap dari
   kanan), mempelai dari sisi masing-masing, ikon merentang.
   Elemen lain cukup opacity + transform (GPU): clip-path di SEMUA elemen dulu
   melukis ulang tiap frame saat digulir (±500 ms raster di HP).
   ATURAN irisan: keadaan tersembunyi ber-clip tak boleh selebar NOL —
   IntersectionObserver memotong kotak deteksi dengan clip-path; sisakan 2px
   (tak tampak: opacity 0). Lihat migrasi 030. */
{a} { --rv-stagger: 150; }
.rv-on {a} [data-rv] { transition: opacity .6s ease, transform 1.1s cubic-bezier(.7, 0, .25, 1); }
.rv-on {a} [data-rv]:not(.is-in) { opacity: 0; transform: translateX(-16px); filter: none; }
.rv-on {a} [data-rv]:nth-child(even):not(.is-in) { transform: translateX(16px); }
.rv-on {a} :is(.section__title, .cover__names, .guest__name, .arch-photo, .gallery__item, .hero__seal)[data-rv] { transition: opacity .6s ease, transform 1.1s cubic-bezier(.7, 0, .25, 1), clip-path 1.1s cubic-bezier(.7, 0, .25, 1); clip-path: inset(-60px -60px -60px -60px); }
.rv-on {a} :is(.section__title, .cover__names, .guest__name)[data-rv]:not(.is-in) { transform: none; clip-path: inset(0 calc(50% - 1px) 0 calc(50% - 1px)); }
.rv-on {a} :is(.arch-photo, .gallery__item, .hero__seal)[data-rv]:not(.is-in) { transform: scale(1.08); clip-path: inset(calc(50% - 1px) 0 calc(50% - 1px) 0); }
.rv-on {a} .couple > .person:first-child[data-rv]:not(.is-in) { transform: translateX(-30px); }
.rv-on {a} .couple > .person:last-child[data-rv]:not(.is-in) { transform: translateX(30px); }
.rv-on {a} :is(.card, .event):not(.person)[data-rv]:not(.is-in) { transform: translateY(-12px); }
.rv-on {a} :is(.intro__icon, .couple__amp, .eyebrow)[data-rv]:not(.is-in) { transform: scaleX(.02); }
$css$ WHERE kind = 'scroll' AND slug = 'tenun';

UPDATE animations SET css = $css$
/* Galaksi — langit malam berbintang dua lapis (paralaks, berkelip). Saat
   dibuka: sampul tersedot menjauh, bintang meregang jadi garis cahaya (warp),
   lalu langit terbuka lewat portal cahaya melingkar dari tengah. */
/* Hemat: selama sampul diam TIDAK ada mask/animasi yang melukis ulang layar —
   bintang hanya berkelip (opacity), mask portal baru dipasang saat
   dibuka (keyframe gx-portal). Dulu background-position 2 lapis layar penuh +
   filter bulan = ±11 dtk raster per 3 dtk di HP. */
{a} { --gate-reveal: 1250; }
{a}::before { content: ""; position: absolute; inset: 0; z-index: 0;
  background: radial-gradient(60% 40% at 50% 110%, color-mix(in srgb, var(--gold) 35%, transparent), transparent 70%), radial-gradient(70% 50% at 15% 10%, color-mix(in srgb, var(--primary) 28%, transparent), transparent 70%), linear-gradient(180deg, color-mix(in srgb, var(--bg) 70%, #000), var(--bg)); }
{a} .gate__content { z-index: 3; }
{a} .cover__eyebrow { animation: gx-glow 1.6s ease .3s both; }
{a} .arch-photo, {a} .hero__seal { animation: gx-near 1.6s cubic-bezier(.2, .7, .2, 1) .4s both; }
{a} .cover__names { animation: gx-glow 1.8s ease .8s both; }
{a} .hero__date, {a} .cover .pill, {a} .guest { animation: gx-near 1.3s ease 1.2s both; }
{a} .gate__open { animation: gx-near 1.1s ease 1.5s both, gate-pulse 2.4s ease-in-out 2.6s infinite; }
/* Dua lapis bintang: kecil-rapat bergeser pelan, besar-jarang berkelip. */
{a} .gate__panel { inset: -10%; width: auto; z-index: 1; box-shadow: none; pointer-events: none; transform-origin: 50% 50%; }
{a} .gate__panel::after { display: none; }
{a} .gate__panel--l { background: radial-gradient(1px 1px at 20px 30px, #fff, transparent), radial-gradient(1px 1px at 90px 80px, #fff, transparent), radial-gradient(1.5px 1.5px at 140px 20px, var(--gold-light), transparent), radial-gradient(1px 1px at 60px 140px, #fff, transparent), radial-gradient(1px 1px at 170px 120px, #fff, transparent);
  background-size: 190px 170px; opacity: .8; }
{a} .gate__panel--r { background: radial-gradient(2px 2px at 40px 60px, #fff, transparent), radial-gradient(2.5px 2.5px at 230px 190px, var(--gold-c, #fed488), transparent), radial-gradient(2px 2px at 300px 40px, #fff, transparent), radial-gradient(1.6px 1.6px at 150px 280px, #fff, transparent);
  background-size: 340px 320px; animation: gx-twinkle 4s ease-in-out infinite alternate; }
/* Bulan sabit (pseudo .gate__fx) + cincin cahaya di tepi portal ({a}::after,
   ukurannya ikut lubang mask). */
{a}::after { content: ""; position: absolute; inset: 0; z-index: 5; pointer-events: none; opacity: 0;
  background: radial-gradient(closest-side, transparent 86%, color-mix(in srgb, var(--gold-c, #fed488) 85%, #fff) 93%, color-mix(in srgb, var(--gold) 50%, transparent) 97%, transparent) center / 0px 0px no-repeat; }
{a} .gate__fx { display: block; z-index: 2; }
{a} .gate__fx::before { content: ""; position: absolute; right: 9%; top: 7%; width: 54px; height: 54px; border-radius: 50%; box-shadow: -11px 6px 0 0 var(--gold-light); filter: drop-shadow(0 0 12px var(--gold-c, #fed488)); animation: gx-moon 6s ease-in-out infinite alternate; }
/* Butir warp: bintang di sekitar pusat, meregang ke luar saat dibuka. */
{a} .gate__fx i { left: 50%; top: 50%; width: 3px; height: 3px; margin: -1.5px; border-radius: 3px; background: #fff; box-shadow: 0 0 6px #fff; opacity: 0;
  transform: rotate(calc(var(--i) * 30deg + 11deg)) translateX(calc(30px + var(--i) * 7px)); transform-origin: 0 50%; }
{a} .gate__fx i:nth-child(3n) { background: var(--gold-light); }
.inv-opened {a}:not(.gate--embed), .is-open > {a} { -webkit-mask: linear-gradient(#000 0 0), radial-gradient(closest-side, #000 97%, transparent) center no-repeat; -webkit-mask-composite: xor;
  mask: linear-gradient(#000 0 0) exclude, radial-gradient(closest-side, #000 97%, transparent) center no-repeat;
  animation: gx-portal 1.2s cubic-bezier(.55, 0, .3, 1) 1s both; transition: visibility 0s 2.3s; }
.inv-opened {a}:not(.gate--embed) .gate__content, .is-open > {a} .gate__content { opacity: 0; transform: scale(.45); transition: transform 1s cubic-bezier(.6, 0, .9, .5), opacity .9s ease; }
.inv-opened {a}:not(.gate--embed) .gate__panel, .is-open > {a} .gate__panel { animation: none; transform: scale(2.6); opacity: 0; transition: transform 1.8s cubic-bezier(.6, 0, .8, .5), opacity .6s ease 1.3s; }
.inv-opened {a}:not(.gate--embed) .gate__panel--r, .is-open > {a} .gate__panel--r { transform: scale(3.6); }
.inv-opened {a}:not(.gate--embed) .gate__fx i, .is-open > {a} .gate__fx i { animation: gx-warp 1.15s cubic-bezier(.55, 0, .9, .4) calc(.15s + var(--i) * .03s) both; }
.inv-opened {a}:not(.gate--embed) .gate__fx::before, .is-open > {a} .gate__fx::before { opacity: 0; transform: translate(60px, -60px) scale(.5); transition: all .9s ease-in; animation: none; }
.inv-opened {a}:not(.gate--embed)::after, .is-open > {a}::after { opacity: 1; background-size: 340vmax 340vmax; transition: opacity .3s .9s, background-size 1.2s cubic-bezier(.55, 0, .3, 1) 1s; }
@keyframes gx-portal { from { -webkit-mask-size: 100% 100%, 0px 0px; mask-size: 100% 100%, 0px 0px; } to { -webkit-mask-size: 100% 100%, 320vmax 320vmax; mask-size: 100% 100%, 320vmax 320vmax; } }
@keyframes gx-twinkle { from { opacity: .35; } to { opacity: 1; } }
@keyframes gx-moon { to { translate: 0 8px; } }
@keyframes gx-glow { from { opacity: 0; transform: scaleX(1.25); } }
@keyframes gx-near { from { opacity: 0; transform: scale(1.25); } }
@keyframes gx-warp { 0% { opacity: 0; } 20% { opacity: 1; } 100% { opacity: 0; transform: rotate(calc(var(--i) * 30deg + 11deg)) translateX(calc(240px + var(--i) * 30px)) scaleX(70); } }
$css$ WHERE kind = 'buka' AND slug = 'galaksi';

UPDATE animations SET css = $css$
/* Taman Daun — sampul dibingkai rimbun dedaunan dua lapis yang bergoyang
   pelan; saat dibuka rimbun tersibak ke kiri-kanan (lapis depan lebih cepat =
   paralaks), daun-daun lepas beterbangan, lalu isi undangan "mekar".
   Daun gelap = --primary, daun terang = --sage. */
{a} { --gate-reveal: 900; }
{a}::before { content: ""; position: absolute; inset: 0; z-index: 0; background: var(--bg-illus, none) center / cover no-repeat, radial-gradient(90% 60% at 50% 45%, var(--card), var(--bg)); transition: opacity .9s ease .55s; }
{a} .gate__content { z-index: 3; }
{a} :is(.cover__names, .cover__eyebrow, .hero__date) { text-shadow: 0 0 10px var(--bg), 0 0 3px var(--bg); }
{a} .cover__eyebrow { animation: tm-bloom 1.1s ease .3s both; }
{a} .arch-photo, {a} .hero__seal { animation: tm-bloom 1.4s cubic-bezier(.2, .8, .2, 1) .5s both; }
{a} .cover__names { animation: tm-bloom 1.2s ease .9s both; }
{a} .hero__date, {a} .cover .pill, {a} .guest { animation: tm-rise 1.1s ease 1.2s both; }
{a} .gate__open { animation: tm-rise 1s ease 1.5s both, gate-pulse 2.4s ease-in-out 2.5s infinite; }
/* Dua rimbun daun di tepi kiri & kanan (kanan = cermin). */
{a} .gate__panel { top: -4%; bottom: -4%; width: 32%; z-index: 2; background: none; box-shadow: none; pointer-events: none; animation: tm-sway 7s ease-in-out infinite alternate; }
{a} .gate__panel--l { left: -10%; transform-origin: 0 100%; }
{a} .gate__panel--r { right: -10%; left: auto; transform-origin: 100% 100%; scale: -1 1; animation-delay: -3.5s; }
{a} .gate__panel::before, {a} .gate__panel::after { content: ""; position: absolute; inset: 0; opacity: 1; transition: transform 1.5s cubic-bezier(.6, 0, .25, 1) .15s; }
{a} .gate__panel::before { background: linear-gradient(170deg, var(--primary), color-mix(in srgb, var(--primary) 70%, #000));
  -webkit-mask: url("/img/tema/gerak/rimbun-2.svg") left center / auto 100% no-repeat; mask: url("/img/tema/gerak/rimbun-2.svg") left center / auto 100% no-repeat; filter: drop-shadow(4px 6px 10px rgba(0, 0, 0, .25)); }
{a} .gate__panel::after { background: linear-gradient(200deg, var(--sage, var(--primary)), color-mix(in srgb, var(--sage, var(--primary)) 60%, var(--gold-light)));
  -webkit-mask: url("/img/tema/gerak/rimbun-1.svg") left top / auto 100% no-repeat; mask: url("/img/tema/gerak/rimbun-1.svg") left top / auto 100% no-repeat; translate: -14% 2%; }
/* Daun lepas: 12 butir, tersebar di tepi, ikut terbang saat dibuka. */
{a} .gate__fx { display: block; z-index: 2; }
{a} .gate__fx i { width: 26px; height: 12px; border-radius: 0 100% 0 100%; background: linear-gradient(135deg, var(--sage, var(--primary)), var(--primary)); opacity: 0;
  left: calc(var(--side) + var(--i) * 1.3%); top: calc(6% + var(--i) * 7.6%); --side: 2%; rotate: calc(var(--i) * 47deg); }
{a} .gate__fx i:nth-child(even) { --side: 74%; }
.inv-opened {a}:not(.gate--embed), .is-open > {a} { transition: visibility 0s 2.3s; }
.inv-opened {a}:not(.gate--embed)::before, .is-open > {a}::before { opacity: 0; }
.inv-opened {a}:not(.gate--embed) .gate__content, .is-open > {a} .gate__content { opacity: 0; transform: scale(1.08); transition: opacity .6s ease, transform .9s ease; }
.inv-opened {a}:not(.gate--embed) .gate__panel, .is-open > {a} .gate__panel { animation: none; transition: transform 1.6s cubic-bezier(.6, 0, .25, 1) .1s, opacity .6s 1.3s; opacity: 0; }
.inv-opened {a}:not(.gate--embed) .gate__panel--l, .is-open > {a} .gate__panel--l { transform: translateX(-105%) rotate(-16deg); }
.inv-opened {a}:not(.gate--embed) .gate__panel--r, .is-open > {a} .gate__panel--r { transform: translateX(-105%) rotate(-16deg); }
.inv-opened {a}:not(.gate--embed) .gate__panel::after, .is-open > {a} .gate__panel::after { transform: translateX(-40%) rotate(-8deg); }
.inv-opened {a}:not(.gate--embed) .gate__fx i, .is-open > {a} .gate__fx i { animation: tm-fly calc(1.4s + var(--i) * .07s) cubic-bezier(.2, .6, .3, 1) calc(var(--i) * .04s) both; }
.inv-opened {a}:not(.gate--embed) .gate__fx i:nth-child(even), .is-open > {a} .gate__fx i:nth-child(even) { --dir: 1; }
@keyframes tm-sway { from { rotate: -1.6deg; } to { rotate: 1.8deg; } }
@keyframes tm-bloom { from { opacity: 0; transform: scale(.7); } }
@keyframes tm-rise { from { opacity: 0; transform: translateY(36px); } }
@keyframes tm-fly { 0% { opacity: 0; transform: none; } 15% { opacity: 1; } 100% { opacity: 0; transform: translate(calc(var(--dir, -1) * (120px + var(--i) * 14px)), calc(-60px + var(--i) * 22px)) rotate(calc(var(--dir, -1) * 540deg)); } }
$css$ WHERE kind = 'buka' AND slug = 'taman-daun';

UPDATE animations SET css = $css$
/* Pagelaran Wayang — sampul = panggung gelap dengan blencong (lampu minyak)
   berkedip. Saat dibuka: lampu panggung meredup ke sampul & kelir (layar)
   menyala hangat, gunungan ditancapkan dari bawah lalu "dikebutkan" dalang,
   berputar & disapu keluar membuka lakon; dua bayangan tokoh wayang masuk
   dari kiri-kanan, lalu kelir memudar ke isi undangan.
   Gunungan = --foil-deep (sisi emas) + bayangan gelap di kelir. */
{a} { --gate-reveal: 2900; }
{a}::before { content: ""; position: absolute; inset: 0; z-index: 0; background: radial-gradient(70% 45% at 50% 0%, color-mix(in srgb, var(--gold) 22%, transparent), transparent 70%), var(--bg-illus, none) center / cover no-repeat, var(--bg); }
/* Kelir: layar putih kekuningan disinari blencong — muncul saat dibuka. */
{a}::after { content: ""; position: absolute; inset: 0; z-index: 1; opacity: 0; pointer-events: none;
  background: radial-gradient(85% 65% at 50% 18%, #fff7e2, color-mix(in srgb, var(--gold-pale, #ffdea5) 70%, #e9d2a3) 55%, color-mix(in srgb, var(--gold) 45%, #6b4a1e) 100%);
  box-shadow: inset 0 0 0 10px color-mix(in srgb, var(--gold) 55%, #3a2510), inset 0 0 80px rgba(60, 30, 0, .45); }
{a} .gate__content { z-index: 6; }
{a} .cover__eyebrow { animation: wy-flame 1.4s ease .2s both; }
{a} .arch-photo, {a} .hero__seal { animation: wy-shadow 1.5s cubic-bezier(.2, .7, .2, 1) .4s both; }
{a} .cover__names { animation: wy-flame 1.5s ease .8s both; }
{a} .hero__date, {a} .cover .pill, {a} .guest { animation: wy-up 1.1s ease 1.2s both; }
{a} .gate__open { animation: wy-up 1s ease 1.5s both, gate-pulse 2.4s ease-in-out 2.5s infinite; }
/* Blencong: nyala lampu di puncak panggung (berkedip) + bara naik. */
{a} .gate__fx { display: block; z-index: 2; }
{a} .gate__fx::after { content: ""; position: absolute; left: 50%; top: -60px; width: 260px; height: 220px; margin-left: -130px; border-radius: 50%;
  background: radial-gradient(closest-side, color-mix(in srgb, var(--gold-c, #fed488) 70%, #fff), color-mix(in srgb, var(--gold) 35%, transparent) 45%, transparent);
  animation: wy-flicker 2.3s ease-in-out infinite alternate; }
{a} .gate__fx i { left: calc(50% + (var(--i) - 6) * 9px); top: 70px; width: 3px; height: 3px; border-radius: 50%; background: var(--gold-c, #fed488); box-shadow: 0 0 6px var(--gold-c, #fed488);
  opacity: 0; animation: wy-ember calc(3s + var(--i) * .25s) ease-out calc(var(--i) * -.6s) infinite; }
/* Bayangan gunungan di kelir (pseudo .gate__fx::before) & gunungan emas (.gate__orn). */
{a} .gate__fx::before, {a} .gate__orn { content: ""; position: absolute; left: 50%; bottom: 3%; width: min(66%, 46vh, 330px); aspect-ratio: 300 / 470; translate: -50% 0; visibility: hidden; transform-origin: 50% 96%;
  -webkit-mask: url("/img/tema/gerak/gunungan.svg") center / contain no-repeat; mask: url("/img/tema/gerak/gunungan.svg") center / contain no-repeat; }
{a} .gate__fx::before { background: #1d1309; filter: blur(3px); opacity: .55; margin-left: 22px; }
{a} .gate__orn { display: block; z-index: 4; pointer-events: none; background: var(--foil-deep, linear-gradient(135deg, #8a6a2a, #e8c877 50%, #7a5a1a)); filter: drop-shadow(0 4px 6px rgba(0, 0, 0, .4)); }
/* Bayangan tokoh wayang kiri & kanan (kanan = cermin, saling berhadapan). */
{a} .gate__panel { top: auto; bottom: 2%; width: min(46%, 30vh); height: auto; aspect-ratio: 200 / 300; z-index: 3; background: #1d1309; box-shadow: none; filter: blur(1.2px); opacity: .8; visibility: hidden; pointer-events: none;
  -webkit-mask: url("/img/tema/ornamen/adat/gunungan-wayang-kulit-tokoh.svg") center bottom / contain no-repeat; mask: url("/img/tema/ornamen/adat/gunungan-wayang-kulit-tokoh.svg") center bottom / contain no-repeat; }
{a} .gate__panel::after { display: none; }
{a} .gate__panel--l { left: 2%; }
{a} .gate__panel--r { right: 2%; left: auto; scale: -1 1; }
/* ── Dibuka ── */
.inv-opened {a}:not(.gate--embed), .is-open > {a} { opacity: 0; transition: opacity .7s ease 2.75s, visibility 0s 3.5s; }
.inv-opened {a}:not(.gate--embed) .gate__content, .is-open > {a} .gate__content { opacity: 0; transform: translateY(30px); transition: opacity .6s ease, transform .7s ease; }
.inv-opened {a}:not(.gate--embed)::after, .is-open > {a}::after { opacity: 1; transition: opacity .9s ease .25s; }
.inv-opened {a}:not(.gate--embed) .gate__fx::after, .is-open > {a} .gate__fx::after { animation: wy-flicker .5s ease-in-out infinite alternate; }
.inv-opened {a}:not(.gate--embed) .gate__orn, .is-open > {a} .gate__orn,
.inv-opened {a}:not(.gate--embed) .gate__fx::before, .is-open > {a} .gate__fx::before { visibility: visible; animation: wy-plant .75s cubic-bezier(.2, .9, .3, 1.15) .5s both, wy-kebut .8s ease-in-out 1.25s, wy-sweep .7s cubic-bezier(.6, 0, .8, .4) 2.05s forwards; }
.inv-opened {a}:not(.gate--embed) .gate__panel, .is-open > {a} .gate__panel { visibility: visible; animation: wy-enter .8s cubic-bezier(.2, .8, .3, 1) 1.95s both; }
@keyframes wy-flicker { from { opacity: .7; transform: scale(.94); } to { opacity: 1; transform: scale(1.05); } }
@keyframes wy-ember { 0% { opacity: 0; transform: none; } 20% { opacity: .9; } 100% { opacity: 0; transform: translate(calc((var(--i) - 6) * 6px), -90px); } }
@keyframes wy-flame { from { opacity: 0; transform: scale(.94); } }
@keyframes wy-shadow { from { opacity: 0; filter: brightness(0); transform: scale(1.06); } 50% { opacity: 1; filter: brightness(.25); } }
@keyframes wy-up { from { opacity: 0; transform: translateY(40px); } }
@keyframes wy-plant { from { transform: translateY(115%); } }
@keyframes wy-kebut { 0%, 100% { rotate: 0deg; } 20% { rotate: -14deg; } 45% { rotate: 12deg; } 70% { rotate: -7deg; } 88% { rotate: 3deg; } }
@keyframes wy-sweep { to { transform: translateX(-160%) rotateY(180deg) rotate(-25deg); opacity: 0; } }
@keyframes wy-enter { from { transform: translateX(-120%); opacity: 0; } }
$css$ WHERE kind = 'buka' AND slug = 'pagelaran-wayang';

UPDATE animations SET css = $css$
/* Tenun Songket — latar 12 helai songket (benang emas di atas --primary)
   yang berkilau bergantian, gonjong rumah gadang di atas sampul. Saat dibuka
   gonjong terangkat, lalu helai-helai ditarik bergantian ke kiri & kanan dari
   tengah ke tepi — seperti kain tenun diurai — memperlihatkan isi undangan. */
{a} { --gate-reveal: 750; }
{a} .gate__panel { display: none; }
{a} .gate__fx { display: block; z-index: 0; }
{a} .gate__fx i { left: -1px; right: -1px; top: calc(var(--i) * 100% / 12); height: calc(100% / 12 + 1px); overflow: hidden;
  background: linear-gradient(180deg, var(--gold) 0 2px, transparent 2px calc(100% - 2px), var(--gold) calc(100% - 2px)),
    repeating-linear-gradient(45deg, transparent 0 9px, color-mix(in srgb, var(--gold) 60%, transparent) 9px 11px),
    repeating-linear-gradient(-45deg, transparent 0 9px, color-mix(in srgb, var(--gold) 60%, transparent) 9px 11px),
    linear-gradient(180deg, color-mix(in srgb, var(--primary) 85%, #000), var(--primary) 50%, color-mix(in srgb, var(--primary) 85%, #000));
  background-position: calc(var(--i) * 7px) 0; }
/* Kilau = pseudo selebar 60% yang DIGESER (transform, dikerjakan GPU) — bukan
   background-position (lukis ulang 12 helai tiap frame). */
{a} .gate__fx i { overflow: hidden; }
{a} .gate__fx i::after { content: ""; position: absolute; top: 0; bottom: 0; left: 0; width: 60%; background: linear-gradient(100deg, transparent 35%, rgba(255, 255, 255, .28) 50%, transparent 65%); transform: translateX(-100%); animation: ts-sheen 5s ease-in-out calc(var(--i) * .18s) infinite; }
{a} .gate__fx i:nth-child(n+13) { display: none; }
{a} .gate__fx i:nth-child(1), {a} .gate__fx i:nth-child(12) { --d: 350ms; }
{a} .gate__fx i:nth-child(2), {a} .gate__fx i:nth-child(11) { --d: 280ms; }
{a} .gate__fx i:nth-child(3), {a} .gate__fx i:nth-child(10) { --d: 210ms; }
{a} .gate__fx i:nth-child(4), {a} .gate__fx i:nth-child(9) { --d: 140ms; }
{a} .gate__fx i:nth-child(5), {a} .gate__fx i:nth-child(8) { --d: 70ms; }
{a} .gate__content { z-index: 3; margin-top: max(16vh, 120px); background: color-mix(in srgb, var(--bg) 93%, transparent); border-radius: 18px; max-width: 420px; width: calc(100% - 36px); padding: 26px 16px 20px;
  box-shadow: 0 0 0 3px var(--gold), 0 0 0 6px color-mix(in srgb, var(--primary) 70%, #000), 0 0 0 7px var(--gold-light), 0 24px 50px -16px rgba(0, 0, 0, .55); }
.gate--embed{a} .gate__content { margin-top: 110px; }
{a} .gate__orn { display: block; position: absolute; left: 50%; top: max(16vh, 120px); z-index: 4; width: min(84%, 380px); aspect-ratio: 400 / 170; translate: -50% -82%; pointer-events: none;
  background: var(--foil-deep, var(--gold)); -webkit-mask: url("/img/tema/gerak/gonjong.svg") center / contain no-repeat; mask: url("/img/tema/gerak/gonjong.svg") center / contain no-repeat;
  filter: drop-shadow(0 6px 10px rgba(0, 0, 0, .35)); animation: ts-rise 1.2s cubic-bezier(.2, .8, .2, 1.1) .2s both; }
.gate--embed{a} .gate__orn { top: 110px; }
{a} .cover__eyebrow { animation: ts-wipe 1s ease .6s both; }
{a} .arch-photo, {a} .hero__seal { animation: ts-blind 1.2s cubic-bezier(.6, 0, .2, 1) .8s both; }
{a} .cover__names { animation: ts-wipe 1.1s ease 1.1s both; }
{a} .hero__date, {a} .cover .pill, {a} .guest { animation: ts-wipe 1s ease 1.35s both; }
{a} .gate__open { animation: ts-wipe 1s ease 1.6s both, gate-pulse 2.4s ease-in-out 2.6s infinite; }
.inv-opened {a}:not(.gate--embed), .is-open > {a} { transition: visibility 0s 1.9s; }
.inv-opened {a}:not(.gate--embed) .gate__content, .is-open > {a} .gate__content { opacity: 0; transform: scale(.9); transition: opacity .45s ease, transform .5s ease; }
.inv-opened {a}:not(.gate--embed) .gate__orn, .is-open > {a} .gate__orn { animation: none; transform: translateY(-160%); opacity: 0; transition: transform .8s cubic-bezier(.6, 0, .4, 1), opacity .6s ease .2s; }
.inv-opened {a}:not(.gate--embed) .gate__fx i, .is-open > {a} .gate__fx i { transform: translateX(-104%) skewX(-8deg); transition: transform .95s cubic-bezier(.7, 0, .25, 1) calc(.35s + var(--d, 0ms)); }
.inv-opened {a}:not(.gate--embed) .gate__fx i:nth-child(even), .is-open > {a} .gate__fx i:nth-child(even) { transform: translateX(104%) skewX(8deg); }
@keyframes ts-sheen { 0%, 55% { transform: translateX(-100%); } 100% { transform: translateX(270%); } }
@keyframes ts-rise { from { opacity: 0; transform: translateY(40%) scale(.8); } }
@keyframes ts-wipe { from { opacity: 0; clip-path: inset(0 100% 0 0); transform: translateX(-16px); } to { clip-path: inset(-20px -20px -20px -20px); } }
@keyframes ts-blind { from { opacity: 0; clip-path: inset(50% 0 50% 0); } 30% { opacity: 1; } to { clip-path: inset(-10px -10px -10px -10px); } }
$css$ WHERE kind = 'buka' AND slug = 'tenun-songket';

UPDATE animations SET css = $css$
/* Candi Bentar — saat dibuka sampul naik & memudar, lalu gapura terbelah khas
   Bali berdiri dari bawah memenuhi layar dengan matahari terbit di celahnya
   dan kabut di kakinya; kemudian kedua belahan bergeser menjauh sambil
   membesar (seperti melangkah masuk pura) dan isi undangan muncul.
   Batu = --primary, ukiran = --foil-deep. */
{a} { --gate-reveal: 1900; }
{a}::before { content: ""; position: absolute; inset: 0; z-index: 0;
  background: radial-gradient(circle at 50% 66%, color-mix(in srgb, var(--gold-c, #fed488) 80%, #fff) 0 7%, color-mix(in srgb, var(--gold-c, #fed488) 40%, transparent) 14%, transparent 40%), linear-gradient(180deg, color-mix(in srgb, var(--gold-pale, #ffdea5) 45%, var(--bg)), var(--bg) 70%);
  transition: opacity .8s ease 2s; animation: cb-sun 5s ease-in-out infinite alternate; }
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
.inv-opened {a}:not(.gate--embed)::before, .is-open > {a}::before { opacity: 0; animation: none; }
.inv-opened {a}:not(.gate--embed) .gate__content, .is-open > {a} .gate__content { opacity: 0; transform: translateY(-70px) scale(.94); transition: opacity .6s ease, transform .8s cubic-bezier(.5, 0, .3, 1); }
/* Tiga tahap berurutan (dulu satu animasi + transisi: keyframe tanpa `to`
   mengambil opacity:0 tahap akhir → gapura tak pernah terlihat). Berdiri dari
   bawah → melangkah masuk (bergeser & membesar) → memudar. */
.inv-opened {a}:not(.gate--embed) .gate__panel, .is-open > {a} .gate__panel { visibility: visible;
  animation: cb-stand 1s cubic-bezier(.2, .8, .2, 1) .35s both, cb-go 1.6s cubic-bezier(.55, 0, .2, 1) 1.45s forwards, cb-fade .5s ease 2.55s forwards; }
.inv-opened {a}:not(.gate--embed) .gate__fx i, .is-open > {a} .gate__fx i { animation: none; opacity: 0; transform: translateY(40%) scale(1.6); transition: all 1.3s ease 1.5s; }
/* Matahari "bernapas" lewat opacity (GPU) — dulu filter brightness layar penuh tanpa henti. */
@keyframes cb-sun { to { opacity: .82; } }
@keyframes cb-stand { from { translate: 0 35%; opacity: 0; } to { translate: 0 0; opacity: 1; } }
@keyframes cb-go { to { transform: translateX(-80%) scale(1.35); } }
@keyframes cb-fade { to { opacity: 0; } }
@keyframes cb-mist { to { translate: 12% -6%; } }
@keyframes cb-drop { from { opacity: 0; transform: translateY(-50px); } }
@keyframes cb-tilt { from { opacity: 0; transform: perspective(700px) rotateY(35deg) scale(.85); } }
@keyframes cb-wave { from { opacity: 0; transform: translateX(-30px) rotate(-3deg); } }
$css$ WHERE kind = 'buka' AND slug = 'candi-bentar';

-- Dua tema provinsi (031) memakai preset `blur` (seluruh section di-blur 10px
-- tiap muncul) — dipindah ke gerak ringan bila belum diubah admin.
UPDATE themes SET scroll_anim = 'anggun' WHERE slug = 'penjor-pura-bali' AND scroll_anim = 'blur';
UPDATE themes SET scroll_anim = 'zoom' WHERE slug = 'asmat-merauke' AND scroll_anim = 'blur';
