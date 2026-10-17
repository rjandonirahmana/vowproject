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
  var hidup = true,
    gulirRaf = 0,
    rafs = [],
    timers = [],
    pesan = null;

  function wadahGulir() {
    return d.querySelector(".inv") || d.scrollingElement || d.documentElement;
  }

  // Semua pekerjaan berulang milik pratinjau dapat dibatalkan ketika iframe
  // dinavigasi/dibongkar. Tanpanya, siklus gulir dan listener message tetap
  // memegang closure sampai konteks browsing lama benar-benar dikumpulkan.
  function nanti(fn, ms) {
    var id = setTimeout(function () {
      var i = timers.indexOf(id);
      if (i >= 0) timers.splice(i, 1);
      if (hidup) fn();
    }, ms);
    timers.push(id);
    return id;
  }

  function bingkai(fn) {
    var id = requestAnimationFrame(function (t) {
      var i = rafs.indexOf(id);
      if (i >= 0) rafs.splice(i, 1);
      if (hidup) fn(t);
    });
    rafs.push(id);
    return id;
  }

  function berhenti() {
    if (!hidup) return;
    hidup = false;
    rafs.forEach(cancelAnimationFrame);
    rafs.length = 0;
    gulirRaf = 0;
    timers.forEach(clearTimeout);
    timers.length = 0;
    if (pesan) removeEventListener("message", pesan);
    removeEventListener("pagehide", berhenti);
  }

  // Satu segmen gulir, ease-in-out via rAF. Gulir halus bawaan Safari
  // menempuh 55% layar dalam ±170 ms — di rekaman tampak MELOMPAT.
  function luncur(ke, ms, lalu) {
    if (!hidup) return;
    var wadah = wadahGulir(),
      dari = wadah.scrollTop,
      t0 = 0;
    var f = function (t) {
      gulirRaf = 0;
      if (!hidup) return;
      if (!t0) t0 = t;
      var k = Math.min(1, (t - t0) / ms);
      k = k < 0.5 ? 2 * k * k : 1 - Math.pow(-2 * k + 2, 2) / 2;
      wadah.scrollTop = dari + (ke - dari) * k;
      if (t - t0 < ms) gulirRaf = bingkai(f);
      else lalu();
    };
    gulirRaf = bingkai(f);
  }

  // Per SEGMEN ±50% layar (1,4 dtk) lalu jeda 1 dtk — thread induk senggang
  // di antaranya (dulu rAF tanpa henti: beranda turun ke ±30 fps).
  // Di dasar: jeda, kembali ke atas, ulangi.
  function langkah() {
    if (!hidup) return;
    var wadah = wadahGulir();
    var max = wadah.scrollHeight - wadah.clientHeight,
      y = wadah.scrollTop;
    if (max <= 0) return;
    if (y >= max - 4) {
      nanti(function () {
        luncur(0, 1800, function () {
          nanti(langkah, 1200);
        });
      }, 1400);
      return;
    }
    luncur(Math.min(max, y + innerHeight * 0.5), 1400, function () {
      nanti(langkah, 1000);
    });
  }

  function mulai(buka, opsi) {
    if (!hidup) return;
    addEventListener("pagehide", berhenti, { once: true });
    buka();
    bingkai(function () {
      if (!hidup) return;
      bingkai(function () {
        if (!hidup) return;
        try {
          parent.postMessage({ pv: "ready" }, location.origin);
        } catch (_) {}
      });
    });
    var jalan = false;
    var show = function (beku) {
      if (jalan) return;
      jalan = true;
      // Panggung 3D pratinjau mulai SEKARANG (iframe baru saja tampil) — di
      // frame berikutnya: Safari tak memulai animasi yang kelas pemicunya
      // terpasang di frame yang sama dengan pemuatan dokumen.
      bingkai(function () {
        if (hidup)
          d.querySelectorAll(".pg").forEach(function (g) {
            g.classList.add("pg--jalan");
          });
      });
      if (opsi && opsi.diam) return;
      if (beku) {
        nanti(function () {
          d.getAnimations().forEach(function (a) {
            try {
              a.pause();
            } catch (_) {}
          });
          d.querySelectorAll("video").forEach(function (v) {
            try {
              v.pause();
            } catch (_) {}
          });
        }, 1400);
        return;
      }
      // Prolog panggung 3D pratinjau = gerak waktu ±5 dtk (panggung/_dasar.css).
      nanti(langkah, 900 + (d.querySelector(".pg") ? 5200 : 0));
    };
    pesan = function (e) {
      if (e.origin === location.origin && e.data && e.data.pv === "show") show(!!e.data.beku);
    };
    addEventListener("message", pesan);
    nanti(function () {
      show(false);
    }, 1200);
  }

  return { mulai: mulai };
})();

/* undMundur — TUTUP undangan = gerak BUKA diputar mundur, piksel demi piksel.
 *
 * Dipakai dua mesin (global.js closeInv & tata.js close). Gerbang yang sudah
 * selesai dibuka tak punya animasi lagi (display:none), jadi timeline buka
 * DIBANGUN ULANG: keadaan tertutup dipasang → gaya dihitung → keadaan terbuka
 * dipasang → gaya dihitung lagi. Transisi & animasi CSS yang tercipta di dalam
 * gerbang = persis gerak buka (kurva, jeda, nilai sama). Semuanya dihentikan di
 * detik terakhir (atau di detik `t0` bila ditutup di tengah gerak buka) lalu
 * diputar dengan playbackRate −1 pada startTime yang sama → frame tutup ke-n =
 * frame buka ke-(akhir−n).
 *
 * Bolak-balik kelas dilakukan dalam SATU tugas (tanpa frame di antaranya).
 * Di luar gerbang: transisi yang sempat tercipta batal sendiri (nilai akhir =
 * nilai sekarang), animasi CSS yang dimulai ulang (mis. isi undangan yang
 * memudar masuk) dikembalikan ke waktunya semula — tak ada yang berkedip.
 *
 *   var t = undMundur.bangun(gerbang, pasangTutup, pasangBuka, { t0: ms })
 *   undMundur.jalan(t, selesai)   putar mundur; selesai() di frame terakhir
 *   undMundur.henti(t)            batalkan jadwal selesai()
 */
var undMundur = (function () {
  var d = document;
  var ada = !!(d.getAnimations && window.Element && Element.prototype.animate && Element.prototype.getAnimations);

  function nama(a) {
    return a.animationName || a.transitionProperty || "";
  }
  function akhir(a) {
    try {
      var e = a.effect.getComputedTiming().endTime;
      return isFinite(e) ? e : 0;
    } catch (_) {
      return 0;
    }
  }

  // Paksa gaya dihitung (tanpa layout — posisi gulir tak tersentuh).
  // document.getAnimations() saja tak cukup di WebKit: ia tak menghitung
  // ulang gaya, jadi keadaan tertutup tak pernah "terlihat" dan transisi
  // buka tak tercipta.
  function hitung(root) {
    void getComputedStyle(root).opacity;
    return d.getAnimations();
  }

  // `root` = elemen gerbang, atau daftar elemen (mis. templat kusuma: video
  // pintu berdiri di luar gerbang).
  function bangun(root, tutup, buka, opsi) {
    var akar = [].concat(root).filter(Boolean);
    if (!ada || !akar.length) {
      tutup();
      buka();
      return null;
    }
    var dalam = function (t) {
      for (var i = 0; i < akar.length; i++) if (akar[i].contains(t)) return true;
      return false;
    };
    root = akar[0];
    // Root dilepas dari render dulu (tugas yang sama, tak pernah tergambar):
    // gaya pertamanya setelah ini = keadaan TERTUTUP. Bila gaya "terbuka"
    // masih terhitung, bolak-balik kelas membatalkan transisinya sendiri
    // (nilai akhir = nilai sekarang) — gerak itu hilang dari timeline mundur.
    var simpan = akar.map(function (r) {
      var x = [r.style.getPropertyValue("display"), r.style.getPropertyPriority("display")];
      r.style.setProperty("display", "none", "important");
      return x;
    });
    // Catatan waktu animasi CSS di halaman sebelum kelas dibolak-balik.
    var lama = hitung(root),
      lamaSet = new Set(lama),
      catat = new Map();
    lama.forEach(function (a) {
      var e = a.effect,
        t = e && e.target;
      if (!t || !a.animationName) return;
      var l = catat.get(t);
      if (!l) catat.set(t, (l = []));
      l.push({ p: e.pseudoElement || "", n: a.animationName, t: a.currentTime, jeda: a.playState === "paused" });
    });
    akar.forEach(function (r, i) {
      if (simpan[i][0]) r.style.setProperty("display", simpan[i][0], simpan[i][1]);
      else r.style.removeProperty("display");
    });
    tutup();
    var tertutup = hitung(root); // gaya keadaan TERTUTUP dihitung
    // Gerak masuk sampul (ornamen, foto) yang dimulai ulang karena gerbang
    // tampil lagi = sudah selesai saat tamu menekan "Buka" → langsung ke
    // akhirnya; gerak tanpa ujung (denyut tombol, slideshow) dibiarkan jalan.
    tertutup.forEach(function (a) {
      var t = a.effect && a.effect.target;
      if (lamaSet.has(a) || !t || !dalam(t) || a.timeline !== d.timeline) return;
      if (akhir(a) > 0)
        try {
          a.finish();
        } catch (_) {}
    });
    tertutup = new Set(tertutup);
    buka();
    var baru = hitung(root).filter(function (a) {
      return !lamaSet.has(a) && !tertutup.has(a);
    });
    var milik = [],
      T = 0;
    baru.forEach(function (a) {
      var e = a.effect,
        t = e && e.target;
      // Animasi scroll-driven (timeline gulir) tak berwaktu — tak disentuh.
      if (!t || a.timeline !== d.timeline) return;
      if (dalam(t)) {
        milik.push(a);
        T = Math.max(T, akhir(a));
        return;
      }
      // Di luar gerbang: transisi → langsung ke nilai akhirnya (= sebelum
      // dibolak-balik); animasi CSS yang dimulai ulang → waktunya dipulihkan.
      if (a.transitionProperty) {
        try {
          a.finish();
        } catch (_) {
          a.cancel();
        }
        return;
      }
      var p = e.pseudoElement || "",
        l = catat.get(t) || [],
        c = null;
      for (var i = 0; i < l.length; i++)
        if (l[i].p === p && l[i].n === nama(a)) {
          c = l[i];
          break;
        }
      try {
        if (c && c.t != null) {
          a.currentTime = c.t;
          if (c.jeda) a.pause();
        } else if (akhir(a) > 0) a.finish();
      } catch (_) {}
    });
    var t0 = opsi && opsi.t0 != null ? Math.max(0, Math.min(T, opsi.t0)) : T;
    milik.forEach(function (a) {
      try {
        a.pause();
        a.currentTime = t0;
      } catch (_) {}
    });
    return { anim: milik, T: T, t0: t0, id: 0 };
  }

  // Semua animasi disamakan ujungnya (endDelay ditambah sampai ujung = `t`),
  // lalu play() dengan laju −1 dari `t`. Tanpa penyamaan, play() memundurkan
  // tiap animasi dari UJUNGNYA SENDIRI (yang pendek mulai mundur seketika).
  // Bukan lewat startTime di masa depan: compositor Chromium menggambar
  // animasi semacam itu di nilai AWALNYA beberapa frame (gerbang berkedip
  // tampil penuh, terukur di warkah).
  // `dari` (ms, bawaan t0) = posisi timeline buka saat ini; boleh melebihi T
  // (gerbang video: timeline buka masih "menunggu" video mundur).
  function jalan(set, selesai, dari) {
    if (!set) {
      selesai();
      return;
    }
    var t = dari != null ? dari : set.t0;
    set.anim.forEach(function (a) {
      try {
        var e = a.effect,
          ujung = akhir(a);
        if (ujung < t) e.updateTiming({ endDelay: (e.getTiming().endDelay || 0) + (t - ujung) });
        a.currentTime = t;
        a.playbackRate = -1;
        a.play();
      } catch (_) {}
    });
    henti(set);
    set.id = setTimeout(function () {
      set.id = 0;
      selesai();
    }, t + 20);
  }

  function henti(set) {
    if (set && set.id) {
      clearTimeout(set.id);
      set.id = 0;
    }
  }

  // Video pembuka mundur = berkas pasangan `<nama>-mundur.mp4` (frame dibalik
  // ffmpeg, satu folder dengan aslinya — ikut diunggah unggah-aset.sh).
  function srcMundur(v) {
    var s = (v && v.getAttribute("src")) || "";
    return /\/video\//.test(s) ? s.replace(/(\.mp4)(?=$|[?#])/, "-mundur$1") : "";
  }
  // Klon video dibuat LEPAS dari DOM (tetap mengunduh) — menyisipkannya ke
  // gerbang saat dibuka merusak hydration Leptos (node asing → panic WASM).
  // Baru dipasang saat menutup, sebagai anak TERAKHIR wadah video (urutan
  // tumpukan tetap dari z-index kelasnya).
  function klon(v, src) {
    var k = d.createElement("video");
    k.className = v.className;
    k.setAttribute("playsinline", "");
    k.setAttribute("aria-hidden", "true");
    k.setAttribute("tabindex", "-1");
    k.muted = true;
    k.preload = "auto";
    k.src = src;
    k.style.visibility = "hidden";
    return k;
  }
  function pasang(v, k) {
    if (k.parentNode !== v.parentNode) v.parentNode.appendChild(k);
    return k;
  }
  // Panaskan video mundur saat gerbang dibuka → saat ditutup frame
  // pertamanya sudah siap (tak menunggu jaringan).
  function panaskan(v) {
    var src = srcMundur(v);
    if (src && !v._mundur) v._mundur = klon(v, src);
  }
  // Putar video pembuka MUNDUR mulai detik `dari` (posisi video maju).
  // mulai(ms) = saat frame mundur pertama tampil (ms = sisa durasi mundur);
  // selesai() setelah frame pertama video asli kembali tampil. Tanpa berkas
  // mundur → video asli digeser mundur per seek (kasar, tapi tetap mundur).
  function videoMundur(v, dari, mulai, selesai) {
    var src = srcMundur(v),
      k = v._mundur || null,
      batas = 0,
      habis = false;
    v._mundur = null;
    var tutupVideo = function () {
      if (habis) return;
      habis = true;
      clearTimeout(batas);
        v.style.visibility = "";
      if (k) k.remove();
      selesai();
    };
    var seekMundur = function () {
      if (k) {
        k.remove();
        k = null;
      }
      var t0 = performance.now();
      v.pause();
      mulai(dari * 1000);
      (function langkah() {
        if (habis) return;
        var t = dari - (performance.now() - t0) / 1000;
        if (t <= 0 || !v.isConnected) return tutupVideo();
        try {
          v.currentTime = t;
        } catch (_) {
          return tutupVideo();
        }
        v.addEventListener(
          "seeked",
          function () {
            requestAnimationFrame(langkah);
          },
          { once: true }
        );
      })();
    };
    if (!src) return seekMundur();
    pasang(v, k || (k = klon(v, src)));
    var dur = isFinite(v.duration) && v.duration > 0 ? v.duration : 0;
    try {
      k.currentTime = Math.max(0, dur - dari);
    } catch (_) {}
    var gagal = function () {
      if (!habis && k && k.style.visibility === "hidden") seekMundur();
    };
    batas = setTimeout(gagal, 1500);
    k.addEventListener("error", gagal, { once: true });
    k.addEventListener(
      "playing",
      function () {
        if (habis) return;
        clearTimeout(batas);
        var tampil = function () {
          if (habis) return;
          k.style.visibility = "";
          v.style.visibility = "hidden";
          // Video asli (tersembunyi) disiapkan di frame pertama sekarang —
          // saat klon habis, keduanya identik dan ditukar tanpa kedip.
          try {
            v.currentTime = 0;
          } catch (_) {}
          var sisa = Math.max(0, ((k.duration || dur) - k.currentTime) * 1000);
          mulai(sisa);
          batas = setTimeout(tutupVideo, sisa + 1200);
        };
        if (k.requestVideoFrameCallback) k.requestVideoFrameCallback(tampil);
        else tampil();
      },
      { once: true }
    );
    k.addEventListener("ended", tutupVideo, { once: true });
    var pr = k.play();
    if (pr && pr.catch) pr.catch(gagal);
  }

  return { ada: ada, bangun: bangun, jalan: jalan, henti: henti, videoMundur: videoMundur, panaskan: panaskan };
})();
