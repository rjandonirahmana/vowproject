// templat/tata.js — mesin SATU-SATUNYA untuk semua tema templat (/u/{slug}).
//
// Templat (HTML di DB) tidak boleh membawa skrip sendiri (CSP menolak skrip
// tanpa nonce); semua perilaku ditulis DEKLARATIF lewat atribut data-* dan
// dijalankan di sini. Kosakata (lihat templat/README.md):
//
//   [data-gate]                 sampul/gerbang layar penuh (ditutup saat dibuka)
//   [data-pre]                  bagian yang animasinya jalan SEBELUM gerbang dibuka
//   [data-open]                 tombol "Buka Undangan" (musik + video pembuka)
//   [data-close]                tombol "Tutup Undangan" (dipasang platform) → kembali ke
//                               sampul; gerak mundur milik templat (.t-closing), --t-close ms
//   video[data-open-video]      video pembuka — gerbang selesai saat video HABIS
//   video[data-bg-video]        video latar berulang; data-loop-from="2.6" (detik)
//   [data-a="zoomIn"]           animasi MASUK sekali (data-ad jeda ms, data-at durasi ms)
//   [data-s="atas|kiri|…"]      gerak GULIR — aktif saat masuk layar, lepas lagi bila
//                               digulir kembali ke atas (diulang, ala everlove)
//   [data-slides="4000"]        silang-pudar anak <img> tiap N ms (+ Ken Burns CSS)
//   [data-countdown="<ms>"]     hitung mundur; anak [data-cd=d|h|m|s]
//   [data-copy="teks"]          salin ke papan klip
//   audio[data-music] + [data-music-toggle]
//   form[data-rsvp]             kirim RSVP tanpa muat ulang; [data-rsvp-msg],
//                               [data-wishes] + <template id="wish-tpl"> ([data-f=…])
//   [data-zoom]                 klik → foto layar penuh
//   [data-burst="N"]            ledakan N partikel saat [data-open]/[data-gift] diketuk
//   [data-coverflow="ms"]       galeri 3D: anak [data-cf-item], [data-cf-count],
//                               [data-cf-prev]/[data-cf-next]; geser/ketuk sisi
//   [data-gift aria-controls]   kotak kado: ketuk → meletup, isi #id muncul
(() => {
  const d = document;
  const html = d.documentElement;
  const q = (s, r = d) => Array.from(r.querySelectorAll(s));
  const data = (() => {
    try {
      return JSON.parse(d.getElementById("tata-data").textContent || "{}");
    } catch (_) {
      return {};
    }
  })();
  const params = new URLSearchParams(location.search);
  const pv = params.get("pv") === "1"; // pratinjau kartu katalog (iframe)
  const reduce = matchMedia("(prefers-reduced-motion: reduce)").matches;
  const KEY = "tata_open_" + (data.slug || "");
  const store = {
    get: (k) => {
      try {
        return sessionStorage.getItem(k);
      } catch (_) {
        return null;
      }
    },
    set: (k, v) => {
      try {
        sessionStorage.setItem(k, v);
      } catch (_) {}
    },
  };
  html.classList.add("t-js");
  // Bahasa halaman (<html lang>, dari server) — teks bawaan skrip ini ikut.
  const T = (id, en) => (html.lang === "en" ? en : id);
  if (reduce) html.classList.add("t-reduce");
  // ── Anggaran media (dihitung PALING AWAL, sebelum video apa pun dimuat) ──
  //   penuh   : video pembuka + video latar + gerak idle
  //   t-lite  : hemat data / RAM < 4 GB / koneksi 2G–3G atau < 1.5 Mbps →
  //             tanpa video (gerbang memakai transisi CSS), tanpa loop dekor
  //   t-reduce: prefers-reduced-motion → statis
  // RAM besar tak menolong di sinyal buruk — jenis koneksi ikut dihitung.
  const nav = navigator;
  const conn = nav.connection || {};
  const lite =
    !!conn.saveData ||
    (nav.deviceMemory && nav.deviceMemory < 4) ||
    /^(slow-2g|2g|3g)$/.test(conn.effectiveType || "") ||
    (conn.downlink > 0 && conn.downlink < 1.5);
  if (lite) html.classList.add("t-lite");

  // ── Animasi masuk (data-a) ───────────────────────────────────────────────
  // Durasi & jeda dipasang sebagai variabel CSS; kelas a-in memicu keyframe
  // t-{nama} (tata.css). Isi di luar gerbang menunggu gerbang dibuka.
  const gate = d.querySelector("[data-gate]");
  const prep = (el) => {
    if (el.dataset.ad) el.style.setProperty("--a-d", (+el.dataset.ad || 0) + "ms");
    if (el.dataset.at) el.style.setProperty("--a-t", (+el.dataset.at || 0) + "ms");
  };
  q("[data-a]").forEach(prep);
  // Isi gerbang & [data-pre] (mis. panel kiri desktop) beranimasi sejak awal.
  const inGate = (el) => (gate && gate.contains(el)) || !!el.closest("[data-pre]");
  let opened = !gate;
  const aIO = new IntersectionObserver(
    (es) => {
      es.forEach((e) => {
        if (!e.isIntersecting) return;
        if (!opened && !inGate(e.target)) return; // ditunda sampai gerbang dibuka
        e.target.classList.add("a-in");
        aIO.unobserve(e.target);
      });
    },
    { rootMargin: "0px 0px -8% 0px" },
  );
  const watchA = () =>
    q("[data-a]:not(.a-in)").forEach((el) => {
      aIO.unobserve(el);
      aIO.observe(el);
    });

  // ── Gerak gulir (data-s) — aktif bila top < tinggi layar − 150px ─────────
  // IntersectionObserver (batas bawah −150px) alih-alih mengukur SEMUA elemen
  // tiap frame gulir: browser hanya memberi tahu elemen yang melintasi batas.
  // Keluar lewat ATAS (top < batas) tetap aktif; keluar lewat bawah → lepas.
  const sEls = q("[data-s]");
  const sSet = (el, top, lim) => el.classList.toggle("s-in", top < lim);
  const sIO = new IntersectionObserver(
    (es) => {
      if (!opened) return;
      es.forEach((e) =>
        sSet(
          e.target,
          e.boundingClientRect.top,
          e.rootBounds ? e.rootBounds.bottom : innerHeight - 150,
        ),
      );
    },
    { rootMargin: "0px 0px -150px 0px" },
  );
  sEls.forEach((el) => sIO.observe(el));
  // Sekali saja saat gerbang dibuka (posisi awal) — bukan per frame gulir.
  const scan = () => {
    if (!opened) return;
    const lim = innerHeight - 150;
    sEls.forEach((el) => sSet(el, el.getBoundingClientRect().top, lim));
  };

  // ── Video latar: berulang dari detik tertentu (bagian awal sekali saja) ──
  const bgVideos = q("video[data-bg-video]");
  bgVideos.forEach((v) => {
    const from = parseFloat(v.dataset.loopFrom || "0") || 0;
    v.loop = !from;
    if (from)
      v.addEventListener("ended", () => {
        v.currentTime = from;
        v.play().catch(() => {});
      });
  });
  const playBg = () => {
    if (reduce || html.classList.contains("t-lite")) return;
    bgVideos.forEach((v) => {
      if (v.preload === "none") v.preload = "auto";
      v.play().catch(() => {});
    });
  };

  // ── Musik ────────────────────────────────────────────────────────────────
  const audio = d.querySelector("audio[data-music]");
  const setPlaying = (on) => html.classList.toggle("t-playing", on);
  // SATU pengatur volume (naik saat dibuka, turun saat ditutup). Token
  // `fadeRun` membatalkan fade lama: buka lagi di tengah fade-turun tak ikut dijeda.
  let fadeRun = 0;
  const fadeTo = (to, ms, done) => {
    const run = ++fadeRun;
    const from = audio.volume;
    const t0 = performance.now();
    const step = (t) => {
      if (run !== fadeRun) return;
      const k = Math.min(1, (t - t0) / ms);
      try {
        audio.volume = from + (to - from) * k;
      } catch (_) {}
      if (k < 1) requestAnimationFrame(step);
      else if (done) done();
    };
    requestAnimationFrame(step);
  };
  const playMusic = () => {
    if (!audio || pv) return;
    fadeRun++;
    audio.volume = 0;
    audio
      .play()
      .then(() => {
        setPlaying(true);
        fadeTo(1, 1500);
      })
      .catch(() => setPlaying(false));
  };
  q("[data-music-toggle]").forEach((b) =>
    b.addEventListener("click", () => {
      if (!audio) return;
      fadeRun++; // ketukan manual menghentikan fade yang sedang jalan
      if (audio.paused) {
        audio.volume = 1;
        audio
          .play()
          .then(() => setPlaying(true))
          .catch(() => {});
      } else {
        audio.pause();
        setPlaying(false);
      }
    }),
  );
  // Video bersuara (prewedding) menjeda musik; musik lanjut setelahnya.
  q("video:not([muted]):not([data-bg-video]):not([data-open-video])").forEach((v) => {
    v.addEventListener("play", () => {
      if (audio && !audio.paused) {
        audio.pause();
        v.dataset.resume = "1";
        setPlaying(false);
      }
    });
    v.addEventListener("pause", () => {
      if (v.dataset.resume) {
        delete v.dataset.resume;
        audio &&
          audio
            .play()
            .then(() => setPlaying(true))
            .catch(() => {});
      }
    });
  });

  // ── Gerbang: sampul naik → video pembuka → isi ───────────────────────────
  // t-lite / reduce: video pembuka tak pernah diunduh (templat preload="none");
  // perangkat & koneksi kuat mulai memuatnya sekarang agar siap saat dibuka.
  const openEl = d.querySelector("video[data-open-video]");
  const openVideo = lite || reduce ? null : openEl;
  if (openVideo) openVideo.preload = "auto";
  const finish = () => {
    if (opened) return;
    opened = true;
    html.classList.add("t-open");
    html.classList.remove("t-opening");
    store.set(KEY, "1");
    playBg();
    watchA();
    scan();
    setTimeout(() => {
      if (!opened) return; // sudah ditutup lagi sebelum gerbang sempat dilepas
      if (gate) gate.hidden = true;
      if (openEl) openEl.hidden = true;
    }, 2200);
  };
  // Tiap pembukaan bernomor (`openRun`); menutup menaikkan nomornya, jadi
  // timer/listener pembukaan lama yang masih tertunda tak berbuat apa-apa.
  let openRun = 0;
  let closeT = 0;
  let videoEnd = null;
  const dropVideoEnd = () => {
    if (!videoEnd || !openVideo) return;
    openVideo.removeEventListener("ended", videoEnd);
    openVideo.removeEventListener("error", videoEnd);
    videoEnd = null;
  };
  const open = (instant) => {
    if (opened || html.classList.contains("t-opening")) return;
    // Dibuka lagi di tengah gerak tutup → batalkan sisa penutupan.
    if (closeT) {
      clearTimeout(closeT);
      closeT = 0;
      html.classList.remove("t-closing");
    }
    const run = ++openRun;
    const fin = () => run === openRun && finish();
    if (!instant) playMusic();
    if (instant || reduce || !openVideo) {
      if (instant) html.classList.add("t-instant");
      html.classList.add("t-opening");
      return void setTimeout(fin, instant ? 0 : +(data.gate_ms || 900));
    }
    html.classList.add("t-opening");
    openVideo.preload = "auto";
    dropVideoEnd();
    let done = false;
    const end = () => {
      if (done || run !== openRun) return;
      done = true;
      dropVideoEnd();
      html.classList.add("t-video-end");
      setTimeout(fin, 450);
    };
    videoEnd = end;
    openVideo.addEventListener("ended", end, { once: true });
    openVideo.addEventListener("error", end, { once: true });
    // Mulai setelah sampul mulai naik; cadangan bila video tak bisa diputar.
    setTimeout(() => run === openRun && openVideo.play().catch(end), +(data.video_delay_ms || 600));
    setTimeout(end, +(data.video_max_ms || 9000));
  };
  // ── Ledakan partikel (data-burst="N") ────────────────────────────────────
  // Tombol buka / kotak kado: N partikel memancar dari tengah elemen (arah &
  // jarak acak, sedikit jatuh), berputar, lalu pudar. Bentuk & warna milik
  // templat: --burst-mask (mask-image), --burst-c1..c3, --burst-size.
  const burst = (el) => {
    if (!el || reduce) return;
    const n = Math.min(40, +el.dataset.burst || 16);
    const r = el.getBoundingClientRect();
    const box = d.createElement("div");
    box.className = "t-burst";
    box.setAttribute("aria-hidden", "true");
    box.style.left = r.left + r.width / 2 + "px";
    box.style.top = r.top + r.height / 2 + "px";
    for (let i = 0; i < n; i++) {
      const p = d.createElement("i");
      const ang = (i / n) * Math.PI * 2 + Math.random() * 0.6;
      const dist = 80 + Math.random() * 170;
      p.className = "t-burst__p";
      p.dataset.v = String(i % 3);
      p.style.setProperty("--dx", (Math.cos(ang) * dist).toFixed(1) + "px");
      p.style.setProperty("--dy", (Math.sin(ang) * dist + 70).toFixed(1) + "px");
      p.style.setProperty("--r", Math.round(Math.random() * 300 - 150) + "deg");
      p.style.setProperty("--s", (0.6 + Math.random() * 0.8).toFixed(2));
      p.style.setProperty("--bd", Math.round(Math.random() * 160) + "ms");
      box.appendChild(p);
    }
    d.body.appendChild(box);
    setTimeout(() => box.remove(), 2000);
  };
  q("[data-open]").forEach((b) =>
    b.addEventListener("click", (e) => {
      e.preventDefault();
      if (b.dataset.burst) burst(b);
      open(false);
    }),
  );
  // ── Tutup undangan (tombol platform [data-close]) ─────────────────────────
  // Kembali ke sampul: gerbang (dan video pembuka) dipasang lagi dalam gaya
  // "terbuka", lalu t-open dicabut di bawah kelas t-closing — tiap templat
  // menulis gerak mundurnya sendiri (.t-closing …). Lama: --t-close (ms).
  const close = () => {
    if (!opened || !gate || html.classList.contains("t-closing")) return;
    const ms = reduce ? 0 : parseInt(getComputedStyle(gate).getPropertyValue("--t-close"), 10) || 1200;
    if (audio && !audio.paused)
      fadeTo(0, 500, () => {
        audio.pause();
        audio.volume = 1;
        setPlaying(false);
      });
    openRun++;
    dropVideoEnd();
    store.set(KEY, "0");
    gate.hidden = false;
    if (openEl) {
      openEl.hidden = false;
      openEl.pause();
      try {
        openEl.currentTime = 0;
      } catch (_) {}
    }
    void gate.offsetWidth; // gaya "terbuka" dihitung dulu → transisi punya titik awal
    html.classList.add("t-closing");
    requestAnimationFrame(() => {
      html.classList.remove("t-open", "t-video-end", "t-instant");
      opened = false;
      closeT = setTimeout(() => {
        closeT = 0;
        html.classList.remove("t-closing");
        // Instan: html ber-scroll-behavior smooth + body sudah overflow:hidden
        // → gulir halus terhenti di tengah (sisa ±70px & tepi terang di bawah).
        scrollTo({ top: 0, behavior: "instant" });
      }, ms + 80);
    });
  };
  q("[data-close]").forEach((b) => {
    if (!gate) b.hidden = true;
    b.addEventListener("click", (e) => {
      e.preventDefault();
      close();
    });
  });
  if (gate) {
    html.classList.add("t-gated");
    // Sudah dibuka di sesi ini (kembali dari tab Story / muat ulang) → langsung isi.
    // Lewati gerbang HANYA bila masih di undangan yang sama (muat ulang /
    // kembali dari tab Story); klik Demo dari beranda / tema lain → sampul.
    // Sama dengan global.js masihDiUndangan().
    const masihDiUndangan = () => {
      try {
        const nav = performance.getEntriesByType("navigation")[0];
        if (nav && nav.type === "reload") return true;
        const r = d.referrer ? new URL(d.referrer) : null;
        const tema = (u) => new URLSearchParams(u.search).get("tema") || "";
        return !!r && r.origin === location.origin && r.pathname.split("/")[2] === location.pathname.split("/")[2] && tema(r) === tema(location);
      } catch (_) {
        return false;
      }
    };
    if (store.get(KEY) === "1" && !pv && masihDiUndangan()) open(true);
  }

  // ── Slideshow foto (silang-pudar) ────────────────────────────────────────
  const slideIO = new IntersectionObserver((es) =>
    es.forEach((e) => {
      e.target.dataset.vis = e.isIntersecting ? "1" : "";
    }),
  );
  q("[data-slides]").forEach((box) => {
    const imgs = q(":scope > img", box);
    if (!imgs.length) return;
    imgs[0].classList.add("is-on");
    if (imgs.length < 2 || reduce) return;
    slideIO.observe(box);
    let i = 0;
    setInterval(
      () => {
        if (box.dataset.vis !== "1" || d.hidden) return;
        imgs[i].classList.remove("is-on");
        i = (i + 1) % imgs.length;
        imgs[i].classList.add("is-on");
      },
      Math.max(2500, +box.dataset.slides || 4500),
    );
  });

  // ── Hitung mundur ────────────────────────────────────────────────────────
  q("[data-countdown]").forEach((box) => {
    const target = +box.dataset.countdown || 0;
    const parts = {
      d: box.querySelector("[data-cd=d]"),
      h: box.querySelector("[data-cd=h]"),
      m: box.querySelector("[data-cd=m]"),
      s: box.querySelector("[data-cd=s]"),
    };
    const pad = (n) => String(n).padStart(2, "0");
    const tick = () => {
      let s = Math.max(0, Math.floor((target - Date.now()) / 1000));
      const dd = Math.floor(s / 86400);
      s -= dd * 86400;
      const hh = Math.floor(s / 3600);
      s -= hh * 3600;
      const mm = Math.floor(s / 60);
      s -= mm * 60;
      if (parts.d) parts.d.textContent = pad(dd);
      if (parts.h) parts.h.textContent = pad(hh);
      if (parts.m) parts.m.textContent = pad(mm);
      if (parts.s) parts.s.textContent = pad(s);
    };
    if (!target) return;
    tick();
    // Berhenti setelah lewat; tab tersembunyi tak dihitung (disusul saat kembali).
    const t = setInterval(() => {
      if (d.hidden) return;
      tick();
      if (Date.now() >= target) clearInterval(t);
    }, 1000);
    d.addEventListener("visibilitychange", () => {
      if (!d.hidden) tick();
    });
  });

  // ── Galeri coverflow (data-coverflow="5000") ─────────────────────────────
  // Anak [data-cf-item] disusun 3D: tengah tegak, sisi miring (data-p = posisi
  // relatif −3…3, gayanya di tata.css). Geser, ketuk foto samping, atau
  // [data-cf-prev]/[data-cf-next]; ganti otomatis tiap N ms selama terlihat.
  // Foto tengah tetap bisa [data-zoom]; [data-cf-count] = "3 / 10".
  q("[data-coverflow]").forEach((box) => {
    const items = q("[data-cf-item]", box);
    const N = items.length;
    if (!N) return;
    const count = box.querySelector("[data-cf-count]");
    let cur = 0;
    let timer = 0;
    let vis = false;
    const show = (i) => {
      cur = (i + N) % N;
      items.forEach((el, k) => {
        let p = k - cur;
        if (p > N / 2) p -= N;
        if (p < -N / 2) p += N;
        el.dataset.p = String(Math.max(-3, Math.min(3, p)));
        el.setAttribute("aria-hidden", p ? "true" : "false");
      });
      if (count) count.textContent = cur + 1 + " / " + N;
    };
    const auto = () => {
      clearInterval(timer);
      const ms = +box.dataset.coverflow || 0;
      if (ms && vis && !reduce && !pv) timer = setInterval(() => show(cur + 1), ms);
    };
    const go = (i) => {
      show(i);
      auto();
    };
    // Fase tangkap: ketukan foto SAMPING hanya memutar (tak membuka zoom).
    items.forEach((el, k) =>
      el.addEventListener(
        "click",
        (e) => {
          if (k === cur) return;
          e.preventDefault();
          e.stopPropagation();
          go(k);
        },
        true,
      ),
    );
    q("[data-cf-prev]", box).forEach((b) => b.addEventListener("click", () => go(cur - 1)));
    q("[data-cf-next]", box).forEach((b) => b.addEventListener("click", () => go(cur + 1)));
    let x0 = null;
    box.addEventListener("touchstart", (e) => (x0 = e.touches[0].clientX), { passive: true });
    box.addEventListener(
      "touchend",
      (e) => {
        if (x0 === null) return;
        const dx = e.changedTouches[0].clientX - x0;
        x0 = null;
        if (Math.abs(dx) > 40) go(cur + (dx < 0 ? 1 : -1));
      },
      { passive: true },
    );
    new IntersectionObserver((es) => {
      vis = es[0].isIntersecting;
      auto();
    }).observe(box);
    box.classList.add("is-cf");
    show(0);
  });

  // ── Kotak kado (data-gift) ───────────────────────────────────────────────
  // Tombol [data-gift aria-controls="id"]: ketuk → kotak meletup + partikel,
  // isi (rekening/alamat) muncul. Tanpa JS isi tetap terlihat.
  q("[data-gift]").forEach((b) => {
    const body = d.getElementById(b.getAttribute("aria-controls") || "");
    const wrap = b.closest("[data-gift-wrap]") || b.parentNode;
    if (body) body.hidden = true;
    b.setAttribute("aria-expanded", "false");
    b.addEventListener("click", () => {
      const on = !wrap.classList.contains("is-open");
      wrap.classList.toggle("is-open", on);
      b.setAttribute("aria-expanded", String(on));
      if (body) body.hidden = !on;
      if (on) burst(b);
    });
  });

  // ── Salin ────────────────────────────────────────────────────────────────
  q("[data-copy]").forEach((b) =>
    b.addEventListener("click", () => {
      const text = b.dataset.copy || "";
      const ok = () => {
        const label = b.querySelector("[data-copy-label]") || b;
        const old = label.textContent;
        label.textContent = T("Tersalin", "Copied");
        b.classList.add("is-copied");
        setTimeout(() => {
          label.textContent = old;
          b.classList.remove("is-copied");
        }, 1800);
      };
      if (navigator.clipboard)
        navigator.clipboard
          .writeText(text)
          .then(ok)
          .catch(() => {});
    }),
  );

  // ── Foto layar penuh ─────────────────────────────────────────────────────
  // Pemicu bisa difokus papan ketik (Enter/Spasi); dialog modal ber-aria,
  // ditutup klik / Esc / tombol ×, fokus kembali ke foto asal.
  const zoom = (el) => {
    const src = el.dataset.zoom || el.currentSrc || el.src;
    if (!src) return;
    const box = d.createElement("div");
    box.className = "t-lightbox";
    box.setAttribute("role", "dialog");
    box.setAttribute("aria-modal", "true");
    box.setAttribute("aria-label", el.alt || T("Foto", "Photo"));
    const img = d.createElement("img");
    img.src = src;
    img.alt = el.alt || "";
    const x = d.createElement("button");
    x.type = "button";
    x.className = "t-lightbox__x";
    x.setAttribute("aria-label", T("Tutup foto", "Close photo"));
    x.textContent = "×";
    box.append(img, x);
    // Latar tak bisa digulir / difokus / dibaca pembaca layar selama dialog terbuka.
    const latar = Array.from(d.body.children).filter((n) => !n.inert);
    const overflow = d.body.style.overflow;
    latar.forEach((n) => {
      n.inert = true;
    });
    d.body.style.overflow = "hidden";
    const close = () => {
      latar.forEach((n) => {
        n.inert = false;
      });
      d.body.style.overflow = overflow;
      box.remove();
      d.removeEventListener("keydown", key, true);
      el.focus({ preventScroll: true });
    };
    const key = (e) => {
      if (e.key === "Escape") {
        e.preventDefault();
        close();
      } else if (e.key === "Tab") {
        e.preventDefault();
        x.focus();
      } // satu-satunya kontrol: fokus terkunci
    };
    box.addEventListener("click", close);
    d.addEventListener("keydown", key, true);
    d.body.appendChild(box);
    x.focus({ preventScroll: true });
  };
  q("[data-zoom]").forEach((el) => {
    if (!el.hasAttribute("tabindex")) el.tabIndex = 0;
    if (!el.hasAttribute("role")) el.setAttribute("role", "button");
    if (!el.getAttribute("aria-label"))
      el.setAttribute("aria-label", el.alt ? T("Perbesar foto: ", "Enlarge photo: ") + el.alt : T("Perbesar foto", "Enlarge photo"));
    el.addEventListener("click", () => zoom(el));
    el.addEventListener("keydown", (e) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        zoom(el);
      }
    });
  });

  // ── RSVP & ucapan (tanpa muat ulang; tanpa JS tetap jalan via 303) ───────
  q("form[data-rsvp]").forEach((f) =>
    f.addEventListener("submit", (e) => {
      e.preventDefault();
      const msg = f.querySelector("[data-rsvp-msg]") || d.querySelector("[data-rsvp-msg]");
      const btn = f.querySelector("[type=submit]");
      if (btn) btn.disabled = true;
      const say = (ok, text) => {
        if (msg) {
          msg.textContent = text;
          msg.dataset.ok = ok ? "1" : "0";
          msg.hidden = false;
        }
      };
      fetch(f.action, {
        method: "POST",
        headers: {
          Accept: "application/json",
          "Content-Type": "application/x-www-form-urlencoded",
        },
        body: new URLSearchParams(new FormData(f)),
      })
        .then((r) => r.json())
        .then((r) => {
          say(r.ok, r.msg || "");
          if (!r.ok) return;
          // Jumlah dari DB (kiriman ulang tamu terdaftar memperbarui baris yang sama).
          if (typeof r.wish_total === "number")
            q("[data-wish-total]").forEach((el) => {
              el.textContent = String(r.wish_total);
            });
          if (!r.wish) return;
          f.reset();
          const tpl = d.getElementById("wish-tpl");
          const list = d.querySelector("[data-wishes]");
          if (!tpl || !list) return;
          const node = tpl.content.firstElementChild.cloneNode(true);
          q("[data-f]", node).forEach((el) => {
            el.textContent = r.wish[el.dataset.f] || "";
          });
          // Ucapan yang DIPERBARUI menggantikan kartu kiriman sebelumnya di halaman ini.
          if (r.baru === false) q(".is-new", list).forEach((n) => n.remove());
          node.classList.add("is-new");
          list.prepend(node);
          const empty = d.querySelector("[data-wishes-empty]");
          if (empty) empty.hidden = true;
          if (typeof r.wish_total !== "number")
            q("[data-wish-total]").forEach((el) => {
              el.textContent = String((+el.textContent || 0) + 1);
            });
        })
        .catch(() => say(false, T("Gagal mengirim — periksa koneksi lalu coba lagi.", "Could not send — check your connection and try again.")))
        .finally(() => {
          if (btn) btn.disabled = false;
        });
    }),
  );

  // ── Pratinjau katalog (iframe ?pv=1): buka otomatis lalu gulir pelan ────
  if (pv) {
    html.classList.add("t-pv");
    // Sama dengan global.js: buka DENGAN gerak setelah kartu induk memunculkan
    // iframe ('show'); dulu open(true) = instan, gerak buka tak pernah tampil.
    let shown = false;
    const show = () => {
      if (shown) return;
      shown = true;
      const btn = gate && gate.querySelector("[data-open]");
      if (btn) burst(btn);
      open(false);
      setTimeout(() => requestAnimationFrame(step), +(data.gate_ms || 900) + 1600);
    };
    addEventListener("message", (e) => {
      if (e.origin === location.origin && e.data && e.data.pv === "show") show();
    });
    setTimeout(show, 1200);
    let last = 0;
    const step = (t) => {
      if (last) {
        const max = d.documentElement.scrollHeight - innerHeight;
        const y = scrollY + ((t - last) / 1000) * 90;
        scrollTo(0, y >= max ? 0 : y);
      }
      last = t;
      requestAnimationFrame(step);
    };
    try {
      parent.postMessage({ pv: "ready" }, location.origin);
    } catch (_) {}
  }

  // ── Navigasi bawah (server/templat.rs): item bagian yang sedang dibaca menyala ──
  const navLinks = q(".t-nav a[href^='#']");
  if (navLinks.length) {
    const byId = new Map();
    navLinks.forEach((a) => {
      const sec = d.getElementById(a.getAttribute("href").slice(1));
      if (sec) byId.set(sec, a);
    });
    // Garis baca 40% dari atas layar: bagian yang melintasinya = aktif.
    const navIO = new IntersectionObserver(
      (es) =>
        es.forEach((e) => {
          if (!e.isIntersecting) return;
          navLinks.forEach((a) => a.classList.toggle("is-active", a === byId.get(e.target)));
        }),
      { rootMargin: "-40% 0px -59% 0px" },
    );
    byId.forEach((_, sec) => navIO.observe(sec));
  }

  watchA();
  if (!gate) {
    playBg();
    scan();
  }
})();
