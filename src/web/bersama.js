/* web/bersama.js — KODE BERSAMA pratinjau etalase (iframe ?pv=1).
 *
 * Digabung (concat! waktu kompilasi) di DEPAN dua skrip: app.js (web/global.js,
 * undangan mesin Leptos) dan tata.js (templat/tata.js, undangan templat) —
 * satu sumber, tanpa request tambahan. Tiap mesin hanya memberi tahu cara
 * "membuka" undangannya; sisanya di sini:
 *
 *   undPratinjau.mulai(buka, { diam })
 *     buka()  lewati gerbang SEBELUM iframe tampil (sampul tak pernah terlihat)
 *     → 'ready' ke kartu induk dua frame kemudian (isi sudah terlukis)
 *     → 'show' dari induk (cadangan 1,2 dtk): panggung 3D mulai (.pg--jalan),
 *         beku (laptop lama)  animasi & video dijeda
 *         biasa               prolog panggung 3D (±5 dtk) lalu gulir per segmen
 *     diam = gerak dikurangi (prefers-reduced-motion): tanpa gulir otomatis.
 */
var undPratinjau = (function () {
  var d = document;

  // Satu segmen gulir, ease-in-out via rAF. Gulir halus bawaan Safari
  // menempuh 55% layar dalam ±170 ms — di rekaman tampak MELOMPAT.
  function luncur(ke, ms, lalu) {
    var dari = window.scrollY, t0 = 0;
    var f = function (t) {
      if (!t0) t0 = t;
      var k = Math.min(1, (t - t0) / ms);
      k = k < 0.5 ? 2 * k * k : 1 - Math.pow(-2 * k + 2, 2) / 2;
      window.scrollTo(0, dari + (ke - dari) * k);
      if (t - t0 < ms) requestAnimationFrame(f); else lalu();
    };
    requestAnimationFrame(f);
  }

  // Per SEGMEN ±50% layar (1,4 dtk) lalu jeda 1 dtk — thread induk senggang
  // di antaranya (dulu rAF tanpa henti: beranda turun ke ±30 fps).
  // Di dasar: jeda, kembali ke atas, ulangi.
  function langkah() {
    var max = d.documentElement.scrollHeight - innerHeight, y = window.scrollY;
    if (max <= 0) return;
    if (y >= max - 4) {
      setTimeout(function () { luncur(0, 1800, function () { setTimeout(langkah, 1200); }); }, 1400);
      return;
    }
    luncur(Math.min(max, y + innerHeight * 0.5), 1400, function () { setTimeout(langkah, 1000); });
  }

  function mulai(buka, opsi) {
    buka();
    requestAnimationFrame(function () {
      requestAnimationFrame(function () {
        try { parent.postMessage({ pv: 'ready' }, location.origin); } catch (_) {}
      });
    });
    var jalan = false;
    var show = function (beku) {
      if (jalan) return;
      jalan = true;
      // Panggung 3D pratinjau mulai SEKARANG (iframe baru saja tampil) — di
      // frame berikutnya: Safari tak memulai animasi yang kelas pemicunya
      // terpasang di frame yang sama dengan pemuatan dokumen.
      requestAnimationFrame(function () {
        d.querySelectorAll('.pg').forEach(function (g) { g.classList.add('pg--jalan'); });
      });
      if (opsi && opsi.diam) return;
      if (beku) {
        setTimeout(function () {
          d.getAnimations().forEach(function (a) { try { a.pause(); } catch (_) {} });
          d.querySelectorAll('video').forEach(function (v) { try { v.pause(); } catch (_) {} });
        }, 1400);
        return;
      }
      // Prolog panggung 3D pratinjau = gerak waktu ±5 dtk (panggung/_dasar.css).
      setTimeout(langkah, 900 + (d.querySelector('.pg') ? 5200 : 0));
    };
    addEventListener('message', function (e) {
      if (e.origin === location.origin && e.data && e.data.pv === 'show') show(!!e.data.beku);
    });
    setTimeout(function () { show(false); }, 1200);
  }

  return { mulai: mulai };
})();
