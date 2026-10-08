// templat/tata.js — mesin SATU-SATUNYA untuk semua tema templat (/u/{slug}).
//
// Templat (HTML di DB) tidak boleh membawa skrip sendiri (CSP menolak skrip
// tanpa nonce); semua perilaku ditulis DEKLARATIF lewat atribut data-* dan
// dijalankan di sini. Kosakata (lihat templat/README.md):
//
//   [data-gate]                 sampul/gerbang layar penuh (ditutup saat dibuka)
//   [data-pre]                  bagian yang animasinya jalan SEBELUM gerbang dibuka
//   [data-open]                 tombol "Buka Undangan" (musik + video pembuka)
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
  const playMusic = () => {
    if (!audio || pv) return;
    audio.volume = 0;
    audio
      .play()
      .then(() => {
        setPlaying(true);
        let v = 0;
        const t = setInterval(() => {
          v = Math.min(1, v + 0.08);
          audio.volume = v;
          if (v >= 1) clearInterval(t);
        }, 120);
      })
      .catch(() => setPlaying(false));
  };
  q("[data-music-toggle]").forEach((b) =>
    b.addEventListener("click", () => {
      if (!audio) return;
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
      if (gate) gate.hidden = true;
      if (openEl) openEl.hidden = true;
    }, 2200);
  };
  const open = (instant) => {
    if (opened || html.classList.contains("t-opening")) return;
    if (!instant) playMusic();
    if (instant || reduce || !openVideo) {
      if (instant) html.classList.add("t-instant");
      html.classList.add("t-opening");
      return void setTimeout(finish, instant ? 0 : +(data.gate_ms || 900));
    }
    html.classList.add("t-opening");
    openVideo.preload = "auto";
    let done = false;
    const end = () => {
      if (!done) {
        done = true;
        html.classList.add("t-video-end");
        setTimeout(finish, 450);
      }
    };
    openVideo.addEventListener("ended", end, { once: true });
    openVideo.addEventListener("error", end, { once: true });
    // Mulai setelah sampul mulai naik; cadangan bila video tak bisa diputar.
    setTimeout(() => openVideo.play().catch(end), +(data.video_delay_ms || 600));
    setTimeout(end, +(data.video_max_ms || 9000));
  };
  q("[data-open]").forEach((b) =>
    b.addEventListener("click", (e) => {
      e.preventDefault();
      open(false);
    }),
  );
  if (gate) {
    html.classList.add("t-gated");
    // Sudah dibuka di sesi ini (kembali dari tab Story / muat ulang) → langsung isi.
    if (store.get(KEY) === "1" && !pv) open(true);
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

  // ── Salin ────────────────────────────────────────────────────────────────
  q("[data-copy]").forEach((b) =>
    b.addEventListener("click", () => {
      const text = b.dataset.copy || "";
      const ok = () => {
        const label = b.querySelector("[data-copy-label]") || b;
        const old = label.textContent;
        label.textContent = "Tersalin";
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
    box.setAttribute("aria-label", el.alt || "Foto");
    const img = d.createElement("img");
    img.src = src;
    img.alt = el.alt || "";
    const x = d.createElement("button");
    x.type = "button";
    x.className = "t-lightbox__x";
    x.setAttribute("aria-label", "Tutup foto");
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
      el.setAttribute("aria-label", el.alt ? "Perbesar foto: " + el.alt : "Perbesar foto");
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
        .catch(() => say(false, "Gagal mengirim — periksa koneksi lalu coba lagi."))
        .finally(() => {
          if (btn) btn.disabled = false;
        });
    }),
  );

  // ── Pratinjau katalog (iframe ?pv=1): buka otomatis lalu gulir pelan ────
  if (pv) {
    html.classList.add("t-pv");
    setTimeout(() => open(true), 250);
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
    setTimeout(() => requestAnimationFrame(step), 1800);
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
