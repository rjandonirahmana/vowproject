// web/global.js — perilaku yang HARUS jalan sebelum/tanpa WASM: musik latar,
// salin rekening, hitung mundur, toast, animasi muncul, banner, dll. Semua lewat
// delegasi di document, jadi tetap berlaku untuk elemen yang dirender ulang
// router (navigasi SPA). Dilayani di /app.js?v=hash (cache permanen) — bukan
// lagi skrip inline di tiap halaman.
//
// Aturan memori: SATU MutationObserver untuk seluruh halaman (tugas didaftar
// lewat onDom), observer/timer selalu dilepas saat elemennya hilang dari DOM.
(function(){
  var d=document, KEY='bgm-on';
  // "Kurangi animasi" dari sistem — dibaca sekali, dipakai semua fitur.
  var calm=matchMedia('(prefers-reduced-motion: reduce)').matches;
  // HP lemah / hemat kuota: gerak berulang dimatikan lewat html.motion-min
  // (main.css). deviceMemory dibulatkan browser (0.5,1,2,4,8) → <4 = ≤3 GB.
  var conn=navigator.connection;
  // Sinyal 2G/3G juga: RAM besar tak menolong saat video & gambar latar telat tiba.
  if((conn && (conn.saveData || /^(slow-2g|2g|3g)$/.test(conn.effectiveType||''))) || (navigator.deviceMemory && navigator.deviceMemory<4)) d.documentElement.classList.add('motion-min');
  // Pratinjau tema di kartu katalog (iframe /u/…?pv=1): mode SENYAP — tanpa
  // musik, tanpa menulis sessionStorage (dipakai bersama tab induk), tanpa
  // pemulihan posisi; gerbang dibuka & halaman digulir otomatis (lihat bawah).
  var PV=window.top!==window && /[?&]pv=1(&|$)/.test(location.search);
  // Satu MutationObserver untuk semua tugas yang perlu tahu DOM berubah
  // (dijadwalkan sekali per frame, bukan per mutasi).
  var domTasks=[], domPending=false;
  function onDom(fn){ domTasks.push(fn); }
  // Observer yang mengamati elemen halaman: elemen yang hilang dari DOM
  // (pindah halaman SPA) dilepas di sapuan onDom — tanpa ini IO menahan node
  // terputus beserta seluruh subpohonnya (gambar, video) di memori.
  var tracked=[];
  function track(io, el){ var t=tracked.find(function(x){ return x.io===io; }); if(!t){ t={io:io, els:new Set()}; tracked.push(t); } t.els.add(el); io.observe(el); }
  function untrack(io, el){ io.unobserve(el); tracked.forEach(function(t){ if(t.io===io) t.els.delete(el); }); }
  // Observer yang tak lagi mengamati apa pun dilepas dari daftar — callback-nya
  // (closure) sering menahan subpohon halaman lama (mis. etalase beranda).
  onDom(function(){
    tracked=tracked.filter(function(t){
      t.els.forEach(function(el){ if(!el.isConnected){ t.io.unobserve(el); t.els.delete(el); } });
      if(t.els.size) return true;
      t.io.disconnect(); return false;
    });
  });
  function runDom(){ domPending=false; domTasks.forEach(function(fn){ fn(); }); }
  function audio(){ return d.getElementById('bgm'); }
  // Undangan mana & tema demo mana (dipakai aturan tautan & gerbang).
  function slugOf(p){ var m=/^\/u\/([^\/]+)/.exec(p||''); return m ? m[1] : ''; }
  function temaOf(q){ try{ return new URLSearchParams(q||'').get('tema')||''; }catch(_){ return ''; } }
  // Gerbang boleh dilewati HANYA bila tamu masih "di dalam" undangan yang
  // sama: muat ulang, atau datang dari halaman undangan yang sama (tab Story
  // → Sampul). Klik Demo dari beranda / tema lain → selalu mulai dari sampul.
  // Dulu penanda sesi berlaku untuk SEMUA undangan & tema demo di tab ini.
  function masihDiUndangan(){
    try{
      var nav=performance.getEntriesByType('navigation')[0];
      if(nav && nav.type==='reload') return true;
      var r=d.referrer ? new URL(d.referrer) : null;
      return !!r && r.origin===location.origin && !!slugOf(location.pathname)
        && slugOf(r.pathname)===slugOf(location.pathname) && temaOf(r.search)===temaOf(location.search);
    }catch(_){ return false; }
  }
  // Bahasa undangan (atribut lang di .inv, i18n.rs): teks bawaan skrip ini
  // ikut — T('Tersalin','Copied'). Di luar undangan = Indonesia.
  function T(id, en){ var r=d.querySelector('.inv[lang]'); return r && r.getAttribute('lang')==='en' ? en : id; }
  function start(a){ var p=a.play(); if(p&&p.catch) p.catch(function(){}); }
  function sync(){ var a=audio(); d.documentElement.classList.toggle('bgm-playing', !!a && !a.paused); }
  // fade=true (tombol "Buka Undangan"): volume naik 0→1 dalam ±0,9 dtk.
  // iOS mengabaikan volume (hanya-baca) — musik langsung penuh, tak apa.
  // SATU pengatur volume: naik saat dibuka, turun saat ditutup. Token `fadeId`
  // membatalkan fade lama — buka lagi di tengah fade-turun tak ikut dijeda.
  var fadeId=0;
  function fadeTo(a, to, ms, done){
    var id=++fadeId, from=a.volume, t0=0;
    var step=function(t){
      if(id!==fadeId) return; if(!t0) t0=t;
      var k=Math.min(1,(t-t0)/ms), e=to>from ? k*k : 1-(1-k)*(1-k);
      try{ a.volume=from+(to-from)*e; }catch(_){}
      if(k<1) requestAnimationFrame(step); else if(done) done();
    };
    requestAnimationFrame(step);
  }
  function play(fade){
    if(PV) return; var a=audio(); if(!a||!a.getAttribute('src')) return;
    if(fade && a.paused){ try{ a.volume=0; }catch(_){} fadeTo(a, 1, 900); }
    else fadeId++; // jeda/putar manual menghentikan fade yang sedang jalan
    start(a); try{sessionStorage.setItem(KEY,'1')}catch(e){}
  }
  function pause(){ var a=audio(); if(a) a.pause(); if(!PV) try{sessionStorage.setItem(KEY,'0')}catch(e){} }
  d.addEventListener('play', sync, true);
  d.addEventListener('pause', sync, true);
  // Musik hanya terdengar selama halaman dilihat: pindah tab/aplikasi atau
  // layar HP dikunci → dijeda; kembali → dilanjutkan (hanya bila tadi memang
  // berputar — jeda manual oleh tamu tetap dihormati).
  var hiddenPaused=false;
  function onHide(){ var a=audio(); if(a && !a.paused){ a.pause(); hiddenPaused=true; } }
  function onShow(){ if(!hiddenPaused) return; hiddenPaused=false; var a=audio(); if(a && a.getAttribute('src')) start(a); }
  d.addEventListener('visibilitychange', function(){ if(d.hidden) onHide(); else onShow(); });
  window.addEventListener('pagehide', onHide);
  window.addEventListener('pageshow', function(e){ if(e.persisted && !d.hidden) onShow(); });
  window.toast=function(msg){
    var t=d.createElement('div'); t.className='toast'; t.textContent=msg; d.body.appendChild(t);
    setTimeout(function(){t.classList.add('toast--out')},2200); setTimeout(function(){t.remove()},2700);
  };
  // Masuk ke undangan (/u/…) dari halaman LAIN — atau ke tema demo lain
  // (?tema= beda) — SELALU dimuat penuh, bukan navigasi SPA router Leptos:
  // tema templat dirender middleware server (SPA menampilkan versi komponen
  // yang salah), CSS halaman asal (/tema.css semua tema) tak boleh terbawa
  // (tampilan bertabrakan), dan hydration undangan sengaja malas. Fase
  // capture → jalan sebelum router; pindah tab di undangan yang sama tetap SPA.
  // SATU aturan untuk semua tautan, termasuk yang ditambahkan nanti.
  d.addEventListener('click', function(e){
    if(e.defaultPrevented || e.button!==0 || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
    var a=e.target.closest && e.target.closest('a[href]');
    if(!a || a.target==='_blank' || a.hasAttribute('download')) return;
    var u; try{ u=new URL(a.href, location.href); }catch(_){ return; }
    if(u.origin!==location.origin || !/^\/u\//.test(u.pathname)) return;
    if(slugOf(location.pathname) && slugOf(location.pathname)===slugOf(u.pathname) && temaOf(location.search)===temaOf(u.search)) return;
    e.preventDefault(); e.stopImmediatePropagation();
    location.assign(u.href);
  }, true);
  d.addEventListener('click', function(e){
    // Tampilkan/sembunyikan sandi (form admin): .pw > input + [data-pw-toggle].
    var pw=e.target.closest('[data-pw-toggle]');
    if(pw){
      var box=pw.closest('.pw'), inp=box && box.querySelector('input'); if(!inp) return;
      var show=inp.type==='password'; inp.type=show?'text':'password';
      box.classList.toggle('is-shown', show); pw.setAttribute('aria-pressed', show?'true':'false');
      var t=show?'Sembunyikan sandi':'Tampilkan sandi'; pw.setAttribute('aria-label', t); pw.title=t;
      inp.focus(); return;
    }
    var el=e.target.closest('[data-music],[data-open],[data-copy],[data-song],[data-demo-open],[data-close-inv]');
    if(!el) return;
    if(el.hasAttribute('data-close-inv')){ e.preventDefault(); closeInv(); return; }
    if(el.hasAttribute('data-demo-open')){
      // Demo tema: buka gerbang HANYA di pratinjau ini (bukan html.inv-opened).
      burst(el);
      var emb=el.closest('.inv--embed'); if(emb) emb.classList.add('is-open');
      var ph=el.closest('.demo-phone, .adm-preview__inv'); if(ph) ph.scrollTop=0;
      play(true);
      if(!openGate(emb && emb.querySelector('.gate'), emb)){ playBg(emb); holdReveal(gateReveal(emb && emb.querySelector('.gate'))); }
      return;
    }
    if(el.hasAttribute('data-open')){
      // Gerbang yang benar-benar dibuka tamu = `data-live`. Gerbang lain yang
      // dirender ulang SETELAH undangan pernah dibuka di tab ini (kembali dari
      // tab Story / Acara, muat ulang) disembunyikan CSS `.inv-seen` — pintu
      // & video pembuka tak diputar lagi.
      var g0=d.querySelector('.gate:not(.gate--embed)'); if(g0) g0.setAttribute('data-live','1');
      // Dibuka lagi di tengah gerak tutup → batalkan sisa penutupan.
      if(closing){ clearTimeout(closing); closing=0; d.documentElement.classList.remove('inv-closing'); }
      if(g0) g0.classList.remove('gate--ulang');
      burst(el); play(true); d.documentElement.classList.add('inv-opened','inv-seen');
      if(d.querySelector('.gate')) window.scrollTo({top:0, behavior:'instant'});
      if(!openGate(g0, d)){ playBg(d); holdReveal(gateReveal(g0)); }
      // Setelah benar-benar tersembunyi (transisi visibility tiap animasi buka
      // berbeda; gerbang video menunggu videonya), lepas dari render.
      // SATU pemeriksa saja (dibatalkan saat ditutup) — tak menumpuk tiap buka.
      clearTimeout(offT);
      if(g0){ var off=function(){ if(!g0.isConnected || !d.documentElement.classList.contains('inv-opened')) return; if(getComputedStyle(g0).visibility==='hidden') g0.classList.add('gate--selesai'); else offT=setTimeout(off, 700); }; offT=setTimeout(off, 1600); }
    }
    if(el.dataset.music==='toggle'){ var a=audio(); if(a&&!a.paused) pause(); else play(); }
    if(el.hasAttribute('data-song')){
      e.preventDefault(); var a=audio(); if(!a) return;
      var src=el.getAttribute('data-song');
      // Lagu yang sama: cukup jeda/lanjut. Menyetel ulang `src` memuat ulang
      // audio dari 0:00 (mengabaikan posisi "mulai lagu dari").
      if(a.getAttribute('src')===src){ if(a.paused) play(); else pause(); }
      else { a.setAttribute('src',src); play(); }
      d.querySelectorAll('[data-song]').forEach(function(x){ x.classList.toggle('is-active', x===el); });
      var np=d.querySelector('[data-now-playing]'); if(np) np.textContent=el.getAttribute('data-title')||'';
    }
    if(el.hasAttribute('data-copy')){
      e.preventDefault(); var v=el.getAttribute('data-copy');
      var ok=function(){ toast(el.getAttribute('data-copied')||T('Tersalin','Copied')); };
      if(navigator.clipboard) navigator.clipboard.writeText(v).then(ok,ok);
      else { var ta=d.createElement('textarea'); ta.value=v; d.body.appendChild(ta); ta.select(); try{d.execCommand('copy')}catch(_){} ta.remove(); ok(); }
    }
  });
  // Tutup undangan → kembali ke sampul: gerbang dipasang lagi, `inv-opened`
  // dicabut, dan `.inv-closing` (main.css) memainkan gerak MENUTUP — panel
  // gerbang bertema merapat, lalu isi sampul muncul. Lama: --gate-close (ms),
  // bawaan 1400.
  var closing=0, offT=0;
  function closeInv(){
    var h=d.documentElement, g=d.querySelector('.gate:not(.gate--embed)');
    if(!g || closing || !h.classList.contains('inv-opened')) return;
    clearTimeout(offT);
    var ms=calm || h.classList.contains('motion-min') ? 0 : (parseInt(getComputedStyle(g).getPropertyValue('--gate-close'),10) || 1400);
    var a=audio(); if(a && !a.paused) fadeTo(a, 0, 500, function(){ pause(); try{ a.volume=1; }catch(_){} });
    try{ sessionStorage.setItem(KEY,'0'); }catch(_){}
    g.setAttribute('data-live','1');
    g.classList.remove('gate--selesai','is-done');
    // Video pintu kembali ke bingkai pertama (pintu tertutup); diputar lagi saat dibuka.
    g.querySelectorAll('video[data-gatevideo]').forEach(function(v){
      dropFin(v);
      v.pause(); try{ v.currentTime=0; }catch(_){}
    });
    void g.offsetWidth; // gaya "terbuka" dihitung dulu agar transisi punya titik awal
    h.classList.add('inv-closing');
    // Sampul yang muncul lagi tampil utuh sekaligus (gate-tutup-isi), bukan
    // memutar ulang gerak masuk tiap baris dengan jedanya masing-masing.
    g.classList.add('gate--ulang');
    requestAnimationFrame(function(){
      // inv-seen ikut dicabut: gerbang yang dirender ulang (pindah tab lalu
      // kembali) harus tampil lagi — undangan memang sedang tertutup.
      h.classList.remove('inv-opened','inv-seen');
      // Di sampul tema ini panel pintu tak terlihat (gebyok, candi bentar)?
      // Diukur pada keadaan sampul (inv-closing dilepas sesaat, dalam frame
      // yang sama) → panel yang baru merapat dipudarkan, bukan hilang mendadak
      // saat inv-closing dicabut.
      h.classList.remove('inv-closing');
      var pl=g.querySelector(':scope > .gate__panel'), cs=pl && getComputedStyle(pl), r=pl && pl.getBoundingClientRect();
      var tampak=!!pl && cs.display!=='none' && cs.visibility!=='hidden' && +cs.opacity>0.05 && r.right>0 && r.left<innerWidth && r.width*r.height>2000;
      g.toggleAttribute('data-tutup-pudar', !!pl && !tampak);
      h.classList.add('inv-closing');
      closing=setTimeout(function(){ closing=0; window.scrollTo({top:0, behavior:'instant'}); h.classList.remove('inv-closing'); }, ms + 80);
    });
  }
  // Semburan kelopak & kilau emas dari tombol "Buka Undangan".
  function burst(el){
    if(calm) return;
    // Koreografi bisa mematikan semburan (--gate-burst: none), mis. tema keraton.
    var g=el.closest('.inv') || d.querySelector('.inv');
    if(g && getComputedStyle(g).getPropertyValue('--gate-burst').trim()==='none') return;
    var r=el.getBoundingClientRect(), cx=r.left+r.width/2, cy=r.top+r.height/2;
    var box=d.createElement('div'); box.className='burst'; box.setAttribute('aria-hidden','true');
    for(var i=0;i<40;i++){
      var p=d.createElement('i'), a=Math.random()*Math.PI*2, dist=120+Math.random()*Math.min(420, innerWidth*0.35);
      p.className = i%3===0 ? 'burst__spark' : 'burst__petal';
      p.style.cssText='left:'+cx+'px;top:'+cy+'px;--dx:'+(Math.cos(a)*dist).toFixed(0)+'px;--dy:'+(Math.sin(a)*dist-60).toFixed(0)+'px;--r:'+(Math.random()*540-270).toFixed(0)+'deg;--t:'+(0.9+Math.random()*0.8).toFixed(2)+'s';
      box.appendChild(p);
    }
    d.body.appendChild(box); setTimeout(function(){ box.remove(); }, 1900);
  }
  function rvScan(){ if(window.__rvScan) window.__rvScan(); }
  // Video latar tema sinema (video[data-bgvideo], preload=none di HTML):
  // diputar tanpa suara setelah gerbang dibuka; mode hemat → tak dimuat
  // (poster saja). Dijeda saat tab tersembunyi, dilanjutkan saat kembali.
  // Video berulang dari detik `#loop=` (atau dari awal): dipakai latar sampul.
  function loopFrom(v, now){
    var lf=/#loop=([\d.]+)/.exec(v.getAttribute('src')||''), at=lf ? parseFloat(lf[1]) : 0;
    v.muted=true; v.loop=false;
    var go=function(){ try{ v.currentTime=at; }catch(_){} start(v); };
    v.addEventListener('ended', go);
    if(now){ if(v.readyState>=1) go(); else v.addEventListener('loadedmetadata', go, {once:true}); }
  }
  function playGateBg(){
    if(calm || d.documentElement.classList.contains('motion-min')) return;
    // Setelah `load`: video tak berebut jaringan dengan CSS, font & foto sampul
    // (poster tampil sampai itu). Video pintu dipanaskan agar siap saat dibuka.
    if(d.readyState!=='complete') return;
    d.querySelectorAll('video[data-gatevideo]').forEach(function(v){ if(v.preload!=='auto') v.preload='auto'; });
    var seen=d.documentElement.classList.contains('inv-seen');
    d.querySelectorAll('video[data-gatebg]:not([data-gb])').forEach(function(v){
      v.setAttribute('data-gb','1');
      // Gerbang lama yang dilewati (.inv-seen) tak perlu memuat videonya.
      if(seen && !v.closest('[data-live]')) return;
      loopFrom(v, true);
    });
  }
  // Undangan sudah dibuka di tab ini → gerbang baru dilewati, jadi video latar
  // isi diputar langsung (tak menunggu pintu).
  onDom(function(){
    if(!d.documentElement.classList.contains('inv-seen')) return;
    var opening=d.querySelector('.gate[data-live]:not(.is-done)');
    if(!opening && d.querySelector('video[data-bgvideo]:not([data-bv])')) playBg(d);
    // Pintu yang sedang diputar ditinggal (pindah tab) → jangan tahan animasi isi.
    if(!opening && revealAt>Date.now()) holdReveal(0);
  });
  function playBg(root){
    if(d.documentElement.classList.contains('motion-min')) return;
    (root||d).querySelectorAll('video[data-bgvideo]').forEach(function(v){
      v.setAttribute('data-bv','1');
      // Pengulangan (`#loop=`) dipasang SEKALI per video — playBg bisa dipanggil
      // berkali-kali (buka-tutup undangan, pindah tab).
      if(!v.dataset.lf){ v.dataset.lf='1'; loopFrom(v, false); }
      if(v.preload!=='auto') v.preload='auto';
      start(v);
    });
  }
  onDom(playGateBg);
  if(d.readyState==='complete') playGateBg(); else window.addEventListener('load', playGateBg, {once:true});
  onDom(function(){
    // Tanpa gerbang (atau sudah dibuka): langsung putar video baru di halaman.
    var g=d.querySelector('.gate:not(.gate--embed)');
    var closed=g && (!d.documentElement.classList.contains('inv-opened') || (g.querySelector('video[data-gatevideo]') && !g.classList.contains('is-done')));
    if(closed) return;
    d.querySelectorAll('.inv:not(.inv--embed) video[data-bgvideo]:not([data-bv])').forEach(function(v){ playBg(v.parentNode); });
  });
  d.addEventListener('visibilitychange', function(){
    d.querySelectorAll('video[data-bv]').forEach(function(v){ if(d.hidden) v.pause(); else start(v); });
  });
  // Video prewedding bersuara diputar → musik latar dijeda.
  d.addEventListener('play', function(e){ var t=e.target; if(t && t.matches && t.matches('video[data-prewed]')) pause(); }, true);
  // Kapan isi mulai dianimasikan setelah gerbang dibuka (ms): adegan pembuka
  // yang panjang (mis. pintu gebyok) menyetel --gate-reveal agar isi tak
  // "habis" teranimasi di balik pintu.
  function gateReveal(g){ var v=g && parseInt(getComputedStyle(g).getPropertyValue('--gate-reveal'),10); return v>0 ? v : 700; }
  // Isi di balik gerbang langsung ditandai (tersembunyi, tak sempat terlihat
  // sebelum waktunya) tapi animasi masuknya baru diputar setelah `ms`.
  var revealAt=0, held=[], heldTimer=0;
  function flushHeld(){
    heldTimer=0; var w=revealAt-Date.now();
    if(w>0){ heldTimer=setTimeout(flushHeld, w); return; }
    held.forEach(function(el){ el.classList.add('is-in'); }); held=[];
  }
  // Elemen yang masuk layar sebelum waktunya ditahan di `held`; jadwal bisa
  // dimundurkan/dimajukan (gerbang video: ditahan sampai videonya selesai).
  function revealNow(el){ if(revealAt>Date.now()){ held.push(el); if(!heldTimer) heldTimer=setTimeout(flushHeld, revealAt-Date.now()); } else el.classList.add('is-in'); }
  function holdReveal(ms){ revealAt=Date.now()+ms; if(heldTimer) clearTimeout(heldTimer); heldTimer=setTimeout(flushHeld, Math.max(0, ms)); rvScan(); }
  // Gerbang dengan video pembuka (video[data-gatevideo]): diputar sekali;
  // gerbang ditutup (.is-done) setelah video SELESAI (bukan timer — sinyal
  // lambat tak memotong pintu), lalu video latar diputar & isi dianimasikan
  // --gate-reveal ms kemudian. Mode hemat / galat → langsung selesai.
  // Lepas KEDUA listener penyelesai video pintu. `{once:true}` hanya melepas
  // yang terpicu — 'error' yang tak pernah terjadi tertinggal & menumpuk
  // +1 tiap buka-tutup (terukur di audit CDP Okt 2026).
  function dropFin(v){ if(!v._fin) return; v.removeEventListener('ended', v._fin); v.removeEventListener('error', v._fin); v._fin=null; }
  function openGate(g, root){
    var v=g && g.querySelector('video[data-gatevideo]'); if(!v) return false;
    // Penyelesai pembukaan SEBELUMNYA (ditutup di tengah video) dilepas —
    // tanpa ini buka lagi = dua penyelesai (video latar & isi diputar ganda).
    dropFin(v);
    // `v._fin!==fin`: pembukaan ini sudah digantikan/ditutup (cadangan 12 dtk
    // atau play() yang gagal belakangan tak boleh menyelesaikan gerbang tertutup).
    var done=false, fin=function(){ if(done || v._fin!==fin) return; done=true; dropFin(v); g.classList.add('is-done'); g.querySelectorAll('video[data-gatebg]').forEach(function(x){ x.pause(); }); playBg(root); holdReveal(gateReveal(g)); };
    v._fin=fin;
    holdReveal(120000);
    if(calm || d.documentElement.classList.contains('motion-min')){ fin(); return true; }
    v.muted=true; try{ v.currentTime=0; }catch(_){}
    v.addEventListener('ended', fin, {once:true}); v.addEventListener('error', fin, {once:true});
    var pr=v.play(); if(pr&&pr.catch) pr.catch(fin);
    setTimeout(fin, 12000);
    return true;
  }
  function pad(n){ return (n<10?'0':'')+n; }
  function tick(){
    if(d.hidden) return;
    d.querySelectorAll('[data-countdown]').forEach(function(box){
      var diff=Math.max(0, +box.getAttribute('data-countdown') - Date.now()), s=Math.floor(diff/1000);
      var v={d:Math.floor(s/86400), h:Math.floor(s%86400/3600), m:Math.floor(s%3600/60), s:s%60};
      box.querySelectorAll('[data-cd]').forEach(function(x){ var k=x.getAttribute('data-cd'); x.textContent = k==='d' ? v.d : pad(v[k]); });
    });
  }
  tick(); setInterval(tick, 1000);
  // Pemindai QR check-in (/kelola/…/scan): BarcodeDetector bawaan browser.
  d.addEventListener('click', function(e){
    var b=e.target.closest('[data-scan-start]'); if(!b) return;
    var v=d.querySelector('[data-scan-video]'), inp=d.querySelector('[data-scan-input]');
    if(!('BarcodeDetector' in window) || !navigator.mediaDevices){ toast('Browser ini belum mendukung pemindai kamera — ketik kodenya manual.'); return; }
    var det=new BarcodeDetector({formats:['qr_code']}), last='', busy=false;
    navigator.mediaDevices.getUserMedia({video:{facingMode:'environment'}}).then(function(stream){
      v.srcObject=stream; v.play(); b.hidden=true; v.hidden=false;
      var stopped=false, stop=function(){
        if(stopped) return; stopped=true;
        stream.getTracks().forEach(function(t){ t.stop(); }); v.srcObject=null; v.hidden=true; b.hidden=false;
        window.removeEventListener('pagehide', stop);
      };
      window.addEventListener('pagehide', stop);
      var next=0;
      (function loop(t){
        if(stopped) return;
        if(!d.body.contains(v)){ stop(); return; }
        if(!busy && v.readyState>=2 && (t||0)>=next){
          next=(t||0)+150;
          busy=true;
          det.detect(v).then(function(codes){
            busy=false;
            if(codes.length && codes[0].rawValue!==last){
              last=codes[0].rawValue; inp.value=last; inp.dispatchEvent(new Event('input',{bubbles:true}));
              inp.form.requestSubmit(); setTimeout(function(){ last=''; }, 4000);
            }
          }, function(){ busy=false; });
        }
        requestAnimationFrame(loop);
      })();
    }, function(){ toast('Izin kamera ditolak.'); });
  });
  // Pratinjau berkas lokal (lagu & foto) — TIDAK diunggah ke server/RustFS.
  // Berkas dibaca lewat URL blob di memori browser; URL dibuang saat berkas
  // diganti, inputnya hilang (pindah halaman SPA), atau tab ditutup/ditinggal.
  var blobs=[], slotBlobs=new Set();
  // Tombol "Batal" di slot foto: kosongkan input → pratinjau kembali ke semula.
  d.addEventListener('click', function(e){
    var b=e.target.closest && e.target.closest('[data-slot-clear]'); if(!b) return;
    e.preventDefault();
    var inp=b.closest('[data-slot-tile]').querySelector('input[type=file][data-slot]');
    if(inp){ inp.value=''; inp.dispatchEvent(new Event('change', {bubbles:true})); }
  });
  addEventListener('pagehide', function(){ slotBlobs.forEach(function(u){ URL.revokeObjectURL(u); }); slotBlobs.clear(); });
  function release(u){
    var a=audio();
    if(a && a.getAttribute('src')===u){ a.pause(); a.removeAttribute('src'); a.load(); sync(); }
    URL.revokeObjectURL(u);
  }
  function drop(el){
    blobs=blobs.filter(function(b){
      if(el && b.el!==el) return true;
      release(b.url);
      (b.targets||[]).forEach(function(t){ if(t.dataset.orig!==undefined){ t.innerHTML=t.dataset.orig; delete t.dataset.orig; } });
      return false;
    });
  }
  d.addEventListener('change', function(e){
    var inp=e.target;
    if(inp.matches && inp.matches('input[type=file][data-gallery-max]')){
      var mx=+inp.dataset.galleryMax, big=Array.prototype.some.call(inp.files, function(f){ return f.size>5*1048576; });
      if(inp.files.length>mx){ toast('Maksimal '+mx+' foto galeri — hanya '+mx+' pertama yang dikirim.'); }
      if(big){ toast('Ada foto galeri lebih dari 5 MB — perkecil dulu.'); inp.value=''; }
      return;
    }
    // Slot foto halaman sunting (galeri / mempelai / sampul): berkas pilihan
    // langsung tampil DI slotnya (tanda "Baru"/"Akan diganti"); kosongkan →
    // foto semula kembali. Belum diunggah sampai formulir disimpan.
    if(inp.matches && inp.matches('input[type=file][data-slot]')){
      var tile=inp.closest('[data-slot-tile]'), img=tile && tile.querySelector('[data-slot-img]');
      if(!tile || !img) return;
      if(tile.dataset.blob){ URL.revokeObjectURL(tile.dataset.blob); slotBlobs.delete(tile.dataset.blob); delete tile.dataset.blob; }
      if(img.dataset.orig===undefined) img.dataset.orig=img.getAttribute('src')||'';
      var sf=inp.files && inp.files[0];
      if(sf && !/^image\/(jpeg|png|webp)$/.test(sf.type)){ toast('Foto harus JPEG, PNG, atau WebP.'); inp.value=''; sf=null; }
      if(sf && sf.size>5*1048576){ toast('Foto lebih dari 5 MB — perkecil dulu.'); inp.value=''; sf=null; }
      if(!sf){ tile.classList.remove('is-new'); if(img.dataset.orig){ img.src=img.dataset.orig; } else { img.removeAttribute('src'); img.hidden=true; } return; }
      var su=URL.createObjectURL(sf); tile.dataset.blob=su; slotBlobs.add(su);
      img.src=su; img.hidden=false; tile.classList.add('is-new');
      return;
    }
    if(!inp.matches || !inp.matches('input[type=file][data-preview]')) return;
    drop(inp);
    var box=inp.closest('[data-preview-box]'), out=box && box.querySelector('[data-preview-out]');
    if(out) out.textContent='';
    var f=inp.files && inp.files[0]; if(!f) return;
    var max=parseFloat(inp.dataset.max||'0');
    if(max && f.size>max*1048576){ toast('Berkas terlalu besar (maks '+max+' MB)'); inp.value=''; return; }
    var url=URL.createObjectURL(f), rec={el:inp,url:url,targets:[]};
    blobs.push(rec);
    if(inp.dataset.preview==='video'){
      if(!out) return;
      var v=d.createElement('video'); v.src=url; v.muted=true; v.loop=true; v.autoplay=true; v.playsInline=true; v.className='preview-video';
      out.appendChild(v); start(v);
      return;
    }
    if(inp.dataset.preview==='audio'){
      if(!out) return;
      var btn=d.createElement('button'); btn.type='button'; btn.className='song';
      btn.setAttribute('data-song',url); btn.setAttribute('data-title',f.name);
      btn.innerHTML='<span class="song__play"><span class="ms when-idle" aria-hidden="true">play_arrow</span><span class="ms when-playing" aria-hidden="true">pause</span></span><span class="song__meta"><b></b><small>Pratinjau lokal • tidak diunggah</small></span>';
      btn.querySelector('b').textContent=f.name;
      out.appendChild(btn); btn.click();
    } else {
      var imgs=[], pos={x:50,y:50,z:1};
      var mk=function(){ var i=d.createElement('img'); i.src=url; i.alt='Pratinjau foto'; i.draggable=false; imgs.push(i); return i; };
      (inp.dataset.target ? d.querySelectorAll(inp.dataset.target) : []).forEach(function(t){
        if(t.dataset.orig===undefined) t.dataset.orig=t.innerHTML;
        t.textContent=''; t.appendChild(mk()); rec.targets.push(t);
      });
      if(!out) return;
      // Bingkai atur posisi: seret = geser titik fokus, penggeser = zoom.
      var crop=d.createElement('div'); crop.className='crop';
      crop.innerHTML='<div class="crop__frame" title="Seret untuk menggeser foto"></div><label class="crop__zoom"><span class="ms" aria-hidden="true">search</span><input type="range" min="1" max="3" step="0.05" value="1" aria-label="Zoom foto"></label><small>Seret foto untuk mengatur posisi</small>';
      var frame=crop.querySelector('.crop__frame'), zoom=crop.querySelector('input');
      frame.appendChild(mk());
      var hidden=null;
      if(inp.dataset.posName){ hidden=d.createElement('input'); hidden.type='hidden'; hidden.name=inp.dataset.posName; crop.appendChild(hidden); }
      var apply=function(){
        var st='object-position:'+pos.x+'% '+pos.y+'%;transform:scale('+pos.z+');transform-origin:'+pos.x+'% '+pos.y+'%';
        imgs.forEach(function(i){ i.setAttribute('style',st); });
        if(hidden) hidden.value=Math.round(pos.x)+','+Math.round(pos.y)+','+pos.z.toFixed(2);
      };
      var drag=null;
      frame.addEventListener('pointerdown', function(ev){ drag={x:ev.clientX,y:ev.clientY,px:pos.x,py:pos.y}; frame.setPointerCapture(ev.pointerId); frame.classList.add('is-drag'); });
      frame.addEventListener('pointermove', function(ev){
        if(!drag) return;
        var k=150/pos.z;
        pos.x=Math.max(0,Math.min(100, drag.px-(ev.clientX-drag.x)/frame.clientWidth*k));
        pos.y=Math.max(0,Math.min(100, drag.py-(ev.clientY-drag.y)/frame.clientHeight*k));
        apply();
      });
      var end=function(){ drag=null; frame.classList.remove('is-drag'); };
      frame.addEventListener('pointerup', end); frame.addEventListener('pointercancel', end);
      zoom.addEventListener('input', function(){ pos.z=parseFloat(zoom.value)||1; apply(); });
      out.appendChild(crop); apply();
    }
  });
  // Penggeser "mulai lagu dari": mengikuti lagu yang sedang dimuat di #bgm.
  function mmss(v){ v=Math.max(0,Math.floor(v||0)); return Math.floor(v/60)+':'+pad(v%60); }
  function seeks(){ return d.querySelectorAll('[data-seek]'); }
  function setStart(v){
    seeks().forEach(function(b){
      var r=b.querySelector('[data-seek-range]'), t=b.querySelector('[data-seek-time]'), h=b.querySelector('[data-seek-value]');
      r.value=v; t.textContent=mmss(v); if(h) h.value=Math.floor(v);
    });
  }
  var lastSrc='';
  d.addEventListener('loadstart', function(e){
    if(e.target.id!=='bgm') return;
    var src=e.target.getAttribute('src')||'';
    if(src!==lastSrc){ lastSrc=src; setStart(0); }
  }, true);
  d.addEventListener('loadedmetadata', function(e){
    var a=e.target; if(a.id!=='bgm' || !isFinite(a.duration)) return;
    seeks().forEach(function(b){
      var r=b.querySelector('[data-seek-range]'); r.max=Math.floor(a.duration); r.disabled=false;
      var hint=b.querySelector('[data-seek-hint]'); if(hint) hint.textContent='Geser ke bagian favorit (mis. reff) — durasi '+mmss(a.duration)+'. Tamu mendengar lagu mulai dari titik ini.';
    });
  }, true);
  // Mulai dari awal (putar pertama / setelah selesai) → loncat ke titik mulai.
  d.addEventListener('play', function(e){
    var a=e.target, r=d.querySelector('[data-seek-range]');
    if(a.id!=='bgm' || !r) return;
    var v=parseFloat(r.value)||0;
    if(v>0 && a.currentTime<0.5){ try{ a.currentTime=v; }catch(_){} }
  }, true);
  d.addEventListener('timeupdate', function(e){
    if(e.target.id!=='bgm') return;
    var txt='Sedang diputar '+mmss(e.target.currentTime);
    d.querySelectorAll('[data-seek-now]').forEach(function(n){ n.textContent=txt; });
  }, true);
  d.addEventListener('input', function(e){
    var r=e.target; if(!r.matches || !r.matches('[data-seek-range]')) return;
    var a=audio(), v=parseFloat(r.value)||0;
    setStart(v);
    if(a && a.getAttribute('src')){ try{ a.currentTime=v; }catch(_){} if(a.paused) play(); }
  });
  onDom(function(){
    if(blobs.length) blobs.filter(function(b){ return !b.el.isConnected; }).forEach(function(b){ drop(b.el); });
  });
  window.addEventListener('pagehide', function(){ drop(null); });
  window.addEventListener('pageshow', function(e){
    if(!e.persisted) return;
    d.querySelectorAll('input[type=file][data-preview]').forEach(function(i){ i.value=''; });
    d.querySelectorAll('[data-preview-out]').forEach(function(o){ o.textContent=''; });
  });
  // Draf formulir pesanan (/buat): disimpan di sessionStorage tab ini saat
  // submit, dipulihkan bila server mengembalikan error (?galat=), dibuang saat
  // pesanan berhasil. Berkas (foto/lagu) tidak bisa diisi ulang oleh browser.
  var DRAFT='buat-draft', restored=false;
  d.addEventListener('submit', function(e){
    var f=e.target; if(!f.matches || !f.matches('form[data-keep]')) return;
    var data=[];
    Array.prototype.forEach.call(f.elements, function(el){
      if(!el.name || el.disabled || /^(file|hidden|password|submit|button)$/.test(el.type)) return;
      if(el.type==='radio' || el.type==='checkbox') data.push([el.name, el.value, el.checked]);
      else data.push([el.name, el.value]);
    });
    try{ sessionStorage.setItem(DRAFT, JSON.stringify(data)); }catch(_){}
  }, true);
  function restoreDraft(){
    if(restored) return;
    var f=d.querySelector('form[data-keep]');
    if(!f || !/[?&]galat=/.test(location.search)) return;
    var data=null; try{ data=JSON.parse(sessionStorage.getItem(DRAFT)||'null'); }catch(_){}
    if(!data) return;
    restored=true;
    var fire=function(el,t){ el.dispatchEvent(new Event(t,{bubbles:true})); };
    data.forEach(function(it){
      Array.prototype.forEach.call(f.elements, function(el){
        if(el.name!==it[0] || el.type==='file') return;
        if(el.type==='radio' || el.type==='checkbox'){
          if(el.value===it[1] && el.checked!==it[2]){ el.checked=it[2]; fire(el,'change'); }
        } else if(it.length===2 && el.value!==it[1]){
          el.value=it[1]; fire(el,'input'); fire(el,'change');
        }
      });
    });
    var note=d.querySelector('[data-draft-note]'); if(note) note.hidden=false;
  }
  new MutationObserver(function(){
    var h=d.documentElement;
    if(h.getAttribute('data-buat-ready')==='1'){ h.removeAttribute('data-buat-ready'); restoreDraft(); }
  }).observe(d.documentElement, {attributes:true, attributeFilter:['data-buat-ready']});
  if(d.documentElement.getAttribute('data-buat-ready')==='1'){ d.documentElement.removeAttribute('data-buat-ready'); restoreDraft(); }
  // Cadangan bila WASM gagal dimuat (tanpa hydrate tak ada yang menimpa nilai).
  setTimeout(restoreDraft, 5000);
  if(/^\/kelola\//.test(location.pathname) && /[?&]baru=1/.test(location.search)){ try{ sessionStorage.removeItem(DRAFT); }catch(_){} }
  // Isi undangan muncul perlahan saat di-scroll. Tanpa JS / "kurangi
  // animasi" → semua langsung tampil (kelas rv-on tak pernah dipasang).
  if('IntersectionObserver' in window && !calm){
    d.documentElement.classList.add('rv-on');
    // Elemen yang masuk layar bersamaan muncul bergiliran: jeda --rv-stagger
    // (ms, dari gerak scroll tema; bawaan 90) per urutan dalam satu gelombang.
    var io=new IntersectionObserver(function(es){
      var k=0;
      es.forEach(function(e){
        if(!e.isIntersecting) return;
        var el=e.target;
        if(el.dataset.rv){
          // Jeda per PERAN (--rv-delay dari koreografi, mis. label 150ms → judul
          // 300ms → foto 500ms) lebih terarah daripada giliran urutan masuk IO.
          var cs=getComputedStyle(el), rd=cs.getPropertyValue('--rv-delay').trim(), ms;
          if(rd) ms=rd; else { var step=parseInt(cs.getPropertyValue('--rv-stagger'),10); if(isNaN(step)) step=90; ms=Math.min(k++, step>150?8:6)*step+'ms'; }
          el.style.transitionDelay=ms; el.style.setProperty('--rv-d', ms);
        }
        revealNow(el);
        io.unobserve(el); watched.delete(el);
      });
    }, {rootMargin:'0px 0px -6% 0px'});
    // Elemen yang diamati tapi sudah hilang (pindah halaman sebelum terlihat)
    // dilepas dari observer — tak ada elemen lama yang tertahan di memori.
    var watched=new Set();
    // Mode "terulang" (koreografi ber---rv-repeat:1, ala everlove): aktif
    // saat elemen 150px di atas tepi bawah layar, dilepas lagi bila elemen
    // kembali ke bawah → gerak diputar ulang tiap digulir naik-turun.
    var ioRep=new IntersectionObserver(function(es){
      es.forEach(function(e){
        if(e.isIntersecting) revealNow(e.target);
        else if(e.boundingClientRect.top>0) e.target.classList.remove('is-in');
      });
    }, {rootMargin:'0px 0px -150px 0px'});
    var observe=function(el){
      watched.add(el);
      if(el.dataset.rv && getComputedStyle(el).getPropertyValue('--rv-repeat').trim()==='1') ioRep.observe(el); else io.observe(el);
    };
    function scan(){
      watched.forEach(function(el){ if(!el.isConnected){ io.unobserve(el); ioRep.unobserve(el); watched.delete(el); } });
      if(!d.querySelector('.inv')) return;
      // Selama sampul/gerbang belum dibuka, jangan "habiskan" animasi isi yang
      // tersembunyi di baliknya — tunggu sampai dibuka.
      var gateClosed = d.querySelector('.gate:not(.gate--embed)') && !d.documentElement.classList.contains('inv-opened');
      // Koreografi "dalam" (tema menyetel --rv-deep:1, mis. sekar ala everlove):
      // isi kartu ikut beranimasi sendiri-sendiri — judul/tanggal/tombol kartu
      // acara, kotak hitung mundur, tiap babak kisah, kartu rekening, ucapan.
      var root=d.querySelector('.inv'), deep=root && getComputedStyle(root).getPropertyValue('--rv-deep').trim()==='1';
      var sel='.inv__main > *:not(.stack):not(.gate):not(.cover), .inv__isi > *, .inv__anchor > *, .inv .section > *:not(.gallery):not(.orn-layer), .inv .gallery > *, .inv .cover > *:not(.orn-layer), .inv .couple > .person, .inv--embed > *:not(.gate):not(.inv__glow)';
      if(deep) sel+=', .inv .countdown > *, .inv .quote > *, .inv .event > *, .inv .story > li, .inv .person > .arch-photo, .inv .person__body > *, .inv .gift .bank, .inv .wishes > *, .inv .rsvp-hero > *:not(.orn-layer)';
      d.querySelectorAll(sel).forEach(function(el){
        if(el.dataset.rv) return;
        var emb=el.closest('.inv--embed');
        if(!emb && el.closest('.gate')) return;
        if(emb ? !emb.classList.contains('is-open') : gateClosed){ if(!emb || !el.closest('.gate')) return; }
        // Gerak khusus judul / foto — hanya bila tema mengaturnya (--rvj-from / --rvf-from).
        var role = el.matches('.section__title, .cover__names, .guest__name, .cover__eyebrow') ? 'j' : el.matches('.arch-photo, .gallery__item, .hero__seal') ? 'f' : '';
        if(role && getComputedStyle(el).getPropertyValue('--rv'+role+'-from').trim()) el.dataset.rvRole = role==='j' ? 'judul' : 'foto';
        el.dataset.rv='1'; observe(el);
      });
      // Ornamen bagian: animasi masuk sendiri-sendiri (jeda dari pengaturan).
      d.querySelectorAll('[data-orn]:not([data-orn-w])').forEach(function(el){
        var emb=el.closest('.inv--embed'), inGate=el.closest('.gate');
        if(emb ? !emb.classList.contains('is-open') && !inGate : gateClosed && !inGate) return;
        el.setAttribute('data-orn-w','1'); observe(el);
      });
    }
    window.__rvScan=scan;
    scan();
    onDom(scan);
    // Foto bergerak (.fg): animasi silang-pudar/Ken Burns hanya berjalan saat
    // bingkainya terlihat — di luar layar dijeda (hemat baterai HP tamu).
    var fgIO=new IntersectionObserver(function(es){
      es.forEach(function(e){ e.target.classList.toggle('is-play', e.isIntersecting); });
    }, {rootMargin:'80px 0px'});
    var fgScan=function(){ d.querySelectorAll('.fg:not([data-fg-w])').forEach(function(el){ el.setAttribute('data-fg-w','1'); track(fgIO, el); }); };
    fgScan();
    onDom(fgScan);
    // Ornamen bagian (±10 per tema, gerak diam tak berujung) dijeda selama
    // bagiannya di luar layar — GPU tak terus menganimasikan yang tak terlihat.
    var ornIO=new IntersectionObserver(function(es){
      es.forEach(function(e){ e.target.classList.toggle('orn-diam', !e.isIntersecting); });
    }, {rootMargin:'120px 0px'});
    var ornScan=function(){ d.querySelectorAll('.orn-host:not([data-orn-io])').forEach(function(el){ el.setAttribute('data-orn-io','1'); track(ornIO, el); }); };
    ornScan();
    onDom(ornScan);
  }
  // Posisi scroll per halaman (navigasi SPA Leptos). Bawaan browser memulihkan
  // posisi lama SEBELUM halaman baru selesai dirender → halaman tema yang
  // pendek "terlempar" ke bawah (posisi katalog terpotong ke dasar halaman).
  // Kini: posisi dicatat sendiri dan dipulihkan setelah halaman cukup tinggi —
  // untuk katalog, kartu "lebih banyak" dimuat ulang dulu sampai posisinya ada.
  if(!PV && 'scrollRestoration' in history){
    history.scrollRestoration='manual';
    var SK='ily_scroll';
    var pageKey=function(){ return location.pathname+location.search; };
    var loadPos=function(){ try{ return JSON.parse(sessionStorage.getItem(SK)||'{}'); }catch(_){ return {}; } };
    var savePos=function(){
      var m=loadPos(); delete m[pageKey()]; m[pageKey()]=Math.round(scrollY);
      var ks=Object.keys(m); while(ks.length>60){ delete m[ks.shift()]; }
      try{ sessionStorage.setItem(SK, JSON.stringify(m)); }catch(_){}
    };
    var posTimer;
    window.addEventListener('scroll', function(){ clearTimeout(posTimer); posTimer=setTimeout(savePos, 150); }, {passive:true});
    // Klik tautan internal: simpan posisi SEBELUM URL berganti.
    d.addEventListener('click', function(e){ var a=e.target.closest && e.target.closest('a[href^="/"]'); if(a){ clearTimeout(posTimer); savePos(); } }, true);
    window.addEventListener('pagehide', savePos);
    var restorePos=function(){
      var m=loadPos(), k=pageKey();
      // Belum pernah dicatat: hormati #jangkar (mis. /?nuansa=jawa#katalog) atau ke atas.
      if(!(k in m)){
        var el=location.hash && d.getElementById(location.hash.slice(1));
        if(el) el.scrollIntoView(); else window.scrollTo({top:0, behavior:'instant'});
        return;
      }
      var y=m[k], t0=Date.now();
      (function step(){
        var max=d.documentElement.scrollHeight-innerHeight;
        if(max>=y || Date.now()-t0>4000){ window.scrollTo({top:Math.min(y, Math.max(max,0)), behavior:'instant'}); return; }
        var more=d.querySelector('[data-load-more="1"]'); if(more) more.click();
        setTimeout(step, more ? 120 : 60);
      })();
    };
    window.addEventListener('popstate', function(){ clearTimeout(posTimer); setTimeout(restorePos, 30); });
    var nav=performance.getEntriesByType && performance.getEntriesByType('navigation')[0];
    if(nav && nav.type==='reload') restorePos();
  }
  // Banner beranda ([data-banner]): titik/‹ › menggulir track (scroll-snap);
  // otomatis tiap 6 dtk — jeda saat di-hover/disentuh/fokus, tab tersembunyi,
  // atau "kurangi animasi".
  var initBanner=function(bn){
    if(bn.__bn) return; bn.__bn=1;
    var tr=bn.querySelector('[data-banner-track]'), dots=bn.querySelectorAll('[data-banner-dot]');
    if(!tr) return;
    var n=tr.children.length, hold=0;
    var idx=function(){ return Math.round(tr.scrollLeft/Math.max(1,tr.clientWidth)); };
    var go=function(i){ i=(i+n)%n; tr.scrollTo({left:i*tr.clientWidth, behavior:'smooth'}); };
    var mark=function(){ var i=idx(); dots.forEach(function(d,j){ d.classList.toggle('is-on', j===i); }); };
    tr.addEventListener('scroll', function(){ requestAnimationFrame(mark); }, {passive:true});
    bn.addEventListener('click', function(e){
      var t=e.target.closest('[data-banner-dot],[data-banner-prev],[data-banner-next]'); if(!t) return;
      hold=Date.now()+10000;
      if(t.hasAttribute('data-banner-dot')) go(+t.getAttribute('data-banner-dot'));
      else go(idx()+(t.hasAttribute('data-banner-next')?1:-1));
    });
    ['pointerdown','touchstart','focusin'].forEach(function(ev){ bn.addEventListener(ev, function(){ hold=Date.now()+10000; }, {passive:true}); });
    var hover=false; bn.addEventListener('mouseenter', function(){ hover=true; }); bn.addEventListener('mouseleave', function(){ hover=false; });
    if(n>1 && !calm){
      // Timer dihentikan begitu banner hilang dari halaman (navigasi SPA) —
      // tanpa ini setiap kunjungan ke beranda menambah timer baru selamanya.
      var timer=setInterval(function(){
        if(!bn.isConnected){ clearInterval(timer); return; }
        if(hover||d.hidden||Date.now()<hold) return;
        go(idx()+1);
      }, 6000);
    }
  };
  var watchBanner=function(){ d.querySelectorAll('[data-banner]').forEach(initBanner); };
  watchBanner();
  onDom(watchBanner);
  // Katalog: paginasi "Tampilkan N tema lagi" kini MANUAL (dulu diklik otomatis
  // = scroll tanpa ujung → info, CTA & footer beranda tak terjangkau).
  // [data-load-more] tetap dipakai restorePos untuk memuat ulang saat Back.
  // ── Pratinjau tema di katalog ───────────────────────────────────────────
  // Kartu tema punya wadah kosong `.tcard__pv[data-pv=URL demo]` (dirender
  // Leptos tanpa anak → iframe yang disisipkan tak mengganggu hydrate).
  // Desktop: langsung saat hover. Layar sentuh: kartu yang disentuh, atau kartu
  // paling tengah di layar setelah gulir berhenti. Hanya SATU iframe hidup.
  // Mesin pratinjau dibagi dengan etalase HP beranda (lihat di bawah).
  var pvApi=null, etaResume=null;
  if(PV){
    d.documentElement.classList.add('is-pv');
    // Jabat tangan dengan kartu induk: 'ready' → kartu memunculkan iframe →
    // balas 'show' → BARU gerbang dibuka (dengan gerak buka tema). Dulu
    // tombol ditekan pada ms ke-0 saat iframe masih tak terlihat, jadi yang
    // tampil langsung isi undangan tanpa animasi buka. Cadangan 1,2 dtk.
    try{ parent.postMessage({pv:'ready'}, location.origin); }catch(_){}
    var pvRun=function(beku){
      if(pvRun.done) return; pvRun.done=true;
      var o=d.querySelector('.gate [data-open]') || d.querySelector('[data-open]');
      if(o && !d.documentElement.classList.contains('inv-opened')) o.click();
      if(calm) return;
      // Mode BEKU (laptop etalase): gerak buka diputar, lalu semua animasi &
      // video dijeda dan tanpa gulir — halaman desktop 1280 px yang terus
      // beranimasi di thread induk menurunkan beranda ke 30 fps (diukur).
      if(beku){
        setTimeout(function(){
          d.getAnimations().forEach(function(a){ try{ a.pause(); }catch(_){} });
          d.querySelectorAll('video').forEach(function(v){ try{ v.pause(); }catch(_){} });
        }, (o ? gateReveal(d.querySelector('.gate:not(.gate--embed)')) : 0)+1400);
        return;
      }
      // Gulir setelah gerbang selesai terbuka (gerak buka utuh terlihat):
      // per SEGMEN (±55% layar tiap 2,4 dtk) dengan gulir halus bawaan
      // browser — dikerjakan compositor, thread utama senggang di antaranya.
      // Dulu scrollTo tiap frame lewat rAF: pratinjau (berbagi thread dengan
      // beranda) membuat seluruh beranda turun ke ±30 fps selama diputar.
      // Di dasar: jeda, kembali ke atas, ulangi.
      var g=d.querySelector('.gate:not(.gate--embed)');
      var langkah=function(){
        var max=d.documentElement.scrollHeight-innerHeight, y=window.scrollY;
        if(max<=0) return;
        if(y>=max-4){ setTimeout(function(){ window.scrollTo({top:0, behavior:'smooth'}); setTimeout(langkah, 2600); }, 1400); return; }
        window.scrollTo({top:Math.min(max, y+innerHeight*0.55), behavior:'smooth'});
        setTimeout(langkah, 2400);
      };
      setTimeout(langkah, o ? gateReveal(g)+1600 : 300);
    };
    window.addEventListener('message', function(e){ if(e.origin===location.origin && e.data && e.data.pv==='show') pvRun(!!e.data.beku); });
    setTimeout(pvRun, 1200);
  } else if(!(navigator.connection && navigator.connection.saveData)){
    // Mesin pratinjau: satu mesin = paling banyak SATU iframe hidup. Kartu
    // katalog & HP etalase berbagi satu mesin; laptop etalase punya mesin
    // sendiri. Lebar viewport iframe dari `data-pv-lebar` wadah (HP 390,
    // laptop 1280 = tata letak desktop), diskalakan ke ukuran wadahnya.
    var pvMesinSemua=[];
    var pvMesin=function(){
      var cur=null;
      var stop=function(){
        if(!cur) return;
        var f=cur.querySelector('iframe'); if(f){ try{ f.src='about:blank'; }catch(_){} f.remove(); }
        if(cur.parentNode) cur.parentNode.classList.remove('is-pv');
        cur._pvu=''; cur=null;
      };
      // Munculkan iframe lalu minta ia mulai membuka undangan.
      var show=function(art, f){
        if(art.classList.contains('is-pv')) return;
        art.classList.add('is-pv');
        var beku=!!(cur && cur.hasAttribute('data-pv-beku'));
        setTimeout(function(){ try{ f.contentWindow.postMessage({pv:'show', beku:beku}, location.origin); }catch(_){} }, 220);
      };
      // Wadah yang sama boleh berganti URL (layar etalase berganti tema).
      var start=function(h){
        var u=h.getAttribute('data-pv');
        if(!u || (cur===h && h.isConnected && h._pvu===u)) return;
        stop(); cur=h; h._pvu=u;
        var art=h.parentNode, r=art.getBoundingClientRect(), PW=+h.getAttribute('data-pv-lebar') || 390, s=r.width/PW;
        if(!s) return;
        var f=d.createElement('iframe');
        f.title='Pratinjau tema'; f.tabIndex=-1; f.setAttribute('aria-hidden','true');
        f.style.width=PW+'px'; f.style.height=Math.ceil(r.height/s)+'px'; f.style.transform='scale('+s+')';
        f.onload=function(){ if(cur===h) show(art, f); };
        f.src=u;
        h.appendChild(f);
      };
      var m={start:start, stop:stop, cur:function(){ return cur; },
        // Iframe memberi tahu begitu HTML-nya terurai (lebih cepat dari onload).
        siap:function(src){ if(!cur) return; var f=cur.querySelector('iframe'); if(f && src===f.contentWindow) show(cur.parentNode, f); }};
      pvMesinSemua.push(m);
      return m;
    };
    onDom(function(){ pvMesinSemua.forEach(function(m){ var c=m.cur(); if(c && !c.isConnected) m.stop(); }); });
    window.addEventListener('message', function(e){
      if(e.origin!==location.origin || !e.data || e.data.pv!=='ready') return;
      pvMesinSemua.forEach(function(m){ m.siap(e.source); });
    });
    var kartu=pvMesin(), pvTimer=0;
    var pvStart=kartu.start, pvStop=kartu.stop;
    // Prefetch HTML pratinjau kartu yang terlihat (di-cache browser 10 mnt,
    // security::is_demo_preview) → saat disentuh/di-hover iframe tampil seketika.
    // Maks 2 unduhan bersamaan; tiap URL sekali saja.
    if('IntersectionObserver' in window && window.fetch){
      var pfDone=new Set(), pfQ=[], pfRun=0;
      var pfNext=function(){
        while(pfRun<2 && pfQ.length){
          var u=pfQ.shift(); pfRun++;
          fetch(u, {credentials:'same-origin', priority:'low'}).catch(function(){}).then(function(){ pfRun--; pfNext(); });
        }
      };
      var pfIO=new IntersectionObserver(function(es){
        es.forEach(function(e){
          if(!e.isIntersecting) return; untrack(pfIO, e.target);
          var u=e.target.getAttribute('data-pv'); if(u && !pfDone.has(u)){ pfDone.add(u); pfQ.push(u); }
        });
        pfNext();
      }, {rootMargin:'150px 0px'});
      var pfSeen=new WeakSet();
      var pfWatch=function(){ d.querySelectorAll('.tcard__pv[data-pv]').forEach(function(h){ if(!pfSeen.has(h)){ pfSeen.add(h); track(pfIO, h); } }); };
      var pfStart=function(){ pfWatch(); onDom(pfWatch); };
      // Setelah halaman sendiri selesai dimuat — prefetch tak berebut dengan katalog.
      if(d.readyState==='complete') setTimeout(pfStart, 300); else window.addEventListener('load', function(){ setTimeout(pfStart, 300); });
    }
    if(matchMedia('(hover: hover) and (pointer: fine)').matches){
      d.addEventListener('mouseover', function(e){
        var art=e.target.closest && e.target.closest('.tcard__art'); if(!art) return;
        var h=art.querySelector('[data-pv]'); if(!h || h===kartu.cur()) return;
        pvStart(h);
      });
      d.addEventListener('mouseout', function(e){
        var art=e.target.closest && e.target.closest('.tcard__art'); if(!art) return;
        if(e.relatedTarget && art.contains(e.relatedTarget)) return;
        var c=kartu.cur(); if(c && art.contains(c)){ pvStop(); if(etaResume) etaResume(); }
      });
    } else {
      var pvPick=function(){
        var best=null, bd=1e9, mid=innerHeight/2;
        d.querySelectorAll('.tcard__art > [data-pv], .etalase__layar > [data-pv]').forEach(function(h){
          var r=h.parentNode.getBoundingClientRect();
          if(r.top<0 || r.bottom>innerHeight) return;
          var dd=Math.abs((r.top+r.bottom)/2-mid); if(dd<bd){ bd=dd; best=h; }
        });
        if(best) pvStart(best); else pvStop();
      };
      window.addEventListener('scroll', function(){ clearTimeout(pvTimer); pvTimer=setTimeout(pvPick, 250); }, {passive:true});
      d.addEventListener('touchstart', function(e){
        var art=e.target.closest && e.target.closest('.tcard__art'); if(!art) return;
        var h=art.querySelector('[data-pv]'); if(h) pvStart(h);
      }, {passive:true});
    }
    pvApi={start:pvStart, stop:pvStop, cur:kartu.cur, mesin:pvMesin};
  }
  // ── Etalase HP beranda ──────────────────────────────────────────────────
  // Tema berjajar (coverflow 3D); yang berhenti di tengah = "di dalam HP":
  // layar HP memutar pratinjaunya lewat mesin kartu di atas (gerak buka tema
  // → bergulir). Geser / panah / klik tema samping; maju otomatis tiap 7 dtk
  // selama terlihat & tak disentuh.
  // KEHALUSAN: iframe pratinjau sama-asal = berbagi thread utama dengan
  // halaman ini, jadi DIBONGKAR begitu etalase mulai bergeser dan baru
  // dimuat setelah geseran berhenti & thread senggang. Coverflow: browser
  // modern → CSS scroll-driven animation (compositor, nol JS per frame);
  // lainnya → tiap frame hanya membaca scrollLeft (posisi kartu di-cache —
  // tanpa layout paksa) & menulis transform/opacity kartu yang terlihat.
  // z-index hanya saat kartu tengah berganti.
  var etaInit=function(){
    d.querySelectorAll('[data-etalase]').forEach(function(tr){
      if(tr._eta) return; tr._eta=1;
      var sec=tr.closest('.etalase'), hp=sec.querySelector('.etalase__pv'), layar=hp.parentNode, info=sec.querySelector('[data-etalase-info]');
      var hitung=sec.querySelector('[data-etalase-hitung]'), progres=sec.querySelector('[data-etalase-progres]');
      // Laptop: tema yang sama di tata letak desktop, mesin pratinjau sendiri.
      var lp=sec.querySelector('.etalase__lpv'), lpLayar=lp && lp.parentNode, lpM=lp && pvApi && pvApi.mesin ? pvApi.mesin() : null, lpView=false, lpT=0;
      var items=[].slice.call(tr.querySelectorAll('.etalase__item')); if(!items.length) return;
      var act=-1, near=-1, raf=0, idleT=0, startT=0, autoT=0, userT=0, inView=false, seen=new Set();
      var dua=function(n){ return (n<10 ? '0' : '')+n; };
      // Didukung scroll-driven animation → transform/opacity diurus CSS
      // (compositor); JS hanya mencari kartu tengah & z-index.
      var sda=window.CSS && CSS.supports && CSS.supports('animation-timeline: view()');
      // sp = jarak antar pusat kartu = --sp di CSS (rumus JS & CSS identik).
      var pos=[], sp=1, last=[];
      var measure=function(){ pos=items.map(function(it){ return it.offsetLeft+it.offsetWidth/2; }); sp=(pos[1]-pos[0]) || items[0].offsetWidth || 1; };
      // Rumus coverflow untuk jarak dd (satuan kartu) dari tengah — sama dengan
      // keyframes etalase-alir (linear, dijepit ±3). `geserPx` = pergeseran
      // tambahan (luncuran tombol).
      var bentuk=function(dd, geserPx){
        var k=Math.min(Math.abs(dd), 3), sg=dd<0 ? -1 : 1;
        return {t:'translateX('+(geserPx||0).toFixed(1)+'px) perspective(1100px) translateX('+(-sg*k*24).toFixed(2)+'%) rotateY('+(-sg*k*26).toFixed(2)+'deg) scale('+(1-k*0.13).toFixed(3)+')', o:(1-k*0.2).toFixed(3)};
      };
      var lay=function(){
        raf=0; if(!pos.length) measure();
        var c=tr.scrollLeft+tr.clientWidth/2, best=0, bd=1e9;
        for(var i=0; i<items.length; i++){
          var dd=(pos[i]-c)/sp, a=Math.abs(dd);
          if(a<bd){ bd=a; best=i; }
          // Kartu jauh disembunyikan → tak memegang lapis GPU (will-change).
          // Ambang 6 (bukan 4,5): kartu |d| 4,5–6 sudah di luar track, jadi saat
          // swipe cepat di Safari — event scroll tertinggal beberapa frame dari
          // gulir compositor — kartu yang masuk sudah berbentuk & tampak.
          var key=a>6 ? 'x' : sda ? 'v' : dd.toFixed(3);
          if(last[i]===key) continue; last[i]=key;
          var st=items[i].style;
          // Hanya kartu dekat (is-dekat) yang memegang animasi scroll-driven &
          // will-change — 95 animasi yang ikut dihitung tiap frame berat di WebKit.
          if(key==='x'){ st.visibility='hidden'; items[i].classList.remove('is-dekat'); continue; }
          items[i].classList.add('is-dekat');
          if(sda){ st.visibility=''; continue; }
          var b0=bentuk(dd, 0);
          st.visibility=''; st.opacity=b0.o; st.transform=b0.t;
        }
        if(best!==near){ near=best; items.forEach(function(it, i){ it.style.zIndex=String(40-Math.abs(i-best)); }); }
        return best;
      };
      var el=function(tag, cls, txt){ var x=d.createElement(tag); if(cls) x.className=cls; if(txt!=null) x.textContent=txt; return x; };
      var tampil=function(i){
        var it=items[i], slug=it.getAttribute('data-slug');
        items.forEach(function(x, k){ x.classList.toggle('is-aktif', k===i); });
        info.textContent='';
        info.appendChild(el('b', 'etalase__judul', it.getAttribute('data-name')));
        var rg=it.getAttribute('data-region'); if(rg) info.appendChild(el('small', 'etalase__wilayah', rg));
        var bt=el('div', 'etalase__btns'), a1=el('a', 'btn btn--soft btn--sm', 'Lihat Demo'), a2=el('a', 'btn btn--primary btn--sm', 'Pilih Tema');
        a1.href=it.getAttribute('href'); a2.href='/buat?tema='+encodeURIComponent(slug);
        bt.appendChild(a1); bt.appendChild(a2); info.appendChild(bt);
        if(hitung) hitung.textContent=dua(i+1)+' / '+dua(items.length);
        hp.setAttribute('data-pv', it.getAttribute('data-pv'));
        if(lp) lp.setAttribute('data-pv', it.getAttribute('data-pv'));
      };
      // Pratinjau dimuat saat thread senggang (setelah geseran selesai).
      // Laptop menyusul ±0,9 dtk setelah HP (dua halaman tak dimuat
      // bersamaan di thread yang sama), hanya bila laptop terlihat.
      var mulaiLaptop=function(){
        clearTimeout(lpT);
        if(!lpM || !lpView) return;
        lpT=setTimeout(function(){
          var run=function(){ if(lpView && lp.isConnected && act>=0) lpM.start(lp); };
          if(window.requestIdleCallback) requestIdleCallback(run, {timeout:900}); else run();
        }, 900);
      };
      var mulai=function(){
        clearTimeout(startT);
        startT=setTimeout(function(){
          var run=function(){ if(pvApi && inView && hp.isConnected && act>=0) pvApi.start(hp); mulaiLaptop(); };
          if(window.requestIdleCallback) requestIdleCallback(run, {timeout:700}); else run();
          // Tema berikutnya diunduh duluan (cache 10 mnt) → tampil seketika.
          var nx=items[(act+1)%items.length].getAttribute('data-pv');
          if(window.fetch && !seen.has(nx)){ seen.add(nx); fetch(nx, {credentials:'same-origin', priority:'low'}).catch(function(){}); }
        }, 120);
      };
      // Bergeser / keluar layar → pratinjau dibongkar (diukur: lebih mulus
      // daripada menjedanya — menjeda ratusan animasi iframe justru mahal).
      var hidup=function(){ return !!(pvApi && pvApi.cur()===hp) || !!(lpM && lpM.cur()); };
      // Iframe pratinjau SAMA-ASAL tetap merender & menjalankan animasi/rAF
      // walau display:none (Chrome hanya menghentikan iframe beda-asal) —
      // diukur: dua pratinjau tersembunyi tetap membuat beranda 30 fps. Maka
      // saat bergeser: disembunyikan seketika (murah), luncuran (transisi
      // compositor) dikirim dulu, lalu DIBONGKAR sesudah frame itu — kerja
      // thread utama tak mengganggu transisi compositor yang sudah berjalan
      // (dulu dibongkar duluan: WebKit menahan frame pertama 77–137 ms).
      // Yang dibongkar hanya iframe yang masih tersembunyi (bukan pratinjau baru).
      var buang=function(){
        [[pvApi, hp], [lpM, lp]].forEach(function(x){
          var m=x[0], h=x[1]; if(!m || !h || m.cur()!==h) return;
          var f=h.querySelector('iframe'); if(f && f.style.display==='none') m.stop();
        });
      };
      // Di tugas klik hanya kelas `is-pv` dicabut (layar memudar lewat transisi
      // opacity compositor) — luncuran langsung dikirim. Frame berikutnya
      // iframe disembunyikan lalu dibongkar.
      var henti=function(){
        clearTimeout(startT); clearTimeout(lpT);
        if(!hidup()) return;
        layar.classList.remove('is-pv'); if(lpLayar) lpLayar.classList.remove('is-pv');
        requestAnimationFrame(function(){
          [hp, lp].forEach(function(h){ var f=h && h.querySelector('iframe'); if(f && !h.parentNode.classList.contains('is-pv')) f.style.display='none'; });
          setTimeout(buang, 0);
        });
      };
      // Putar otomatis: satu siklus 7 dtk per tema, garis progres (scaleX,
      // compositor) diputar ulang bersamaan; ditunda selama disentuh/ditunjuk.
      var jadwal=function(){
        clearTimeout(autoT);
        if(progres){ progres.classList.remove('is-jalan'); void progres.offsetWidth; if(!calm && inView) progres.classList.add('is-jalan'); }
        if(calm || !inView) return;
        autoT=setTimeout(function(){
          if(!tr.isConnected) return;
          if(d.hidden || Date.now()-userT<12000 || sec.matches(':hover')) return jadwal();
          go(act+1);
        }, 7000);
      };
      // Gulir berhenti: belum pas di tengah (selesai swipe / seret / momentum)
      // → meluncur ke kartu terdekat (snap sendiri); sudah pas → kartu aktif.
      var settle=function(){
        var b2=lay();
        if(!luncurT && Math.abs(pos[b2]-(tr.scrollLeft+tr.clientWidth/2))>1){ tujuan=b2; geser(b2); return; }
        if(b2!==act){ act=b2; tampil(b2); } mulai(); jadwal();
      };
      // Pindah tema lewat TOMBOL / klik samping / otomatis: TANPA menggulir.
      // Tiap kartu yang terlihat diberi transisi CSS transform+opacity dari
      // bentuknya sekarang ke bentuk di posisi tujuan (+ pergeseran px) —
      // dijalankan compositor (Core Animation di Safari), nol JS per frame,
      // nol event scroll. Selesai → scrollLeft diset ke tujuan & gaya inline
      // dilepas di tugas yang sama (coverflow scroll-driven / lay() menghasilkan
      // bentuk yang identik → tak terlihat). Diukur: menggulir tiap frame lewat
      // JS tersendat di WebKit bahkan tanpa pratinjau; scrollTo smooth bawaan
      // terlalu cepat (130–230 ms). Seret jari/mouse tetap memakai gulir asli.
      var tujuan=-1, luncurT=0, antre=-1, luncurIkut=[], progSL=-1;
      // Pengguna mulai menggulir sendiri di tengah luncuran → luncuran
      // dibatalkan di tempat: animasi dilepas, coverflow kembali mengikuti
      // posisi gulir asli. (Dulu diselesaikan paksa → scrollLeft diloncatkan
      // ke tujuan, melawan jari/trackpad.)
      var batalLuncur=function(){
        if(!luncurT && !luncurIkut.length) return;
        clearTimeout(luncurT); luncurT=0; antre=-1; akhir=null;
        luncurIkut.forEach(function(x){ if(x.anim) x.anim.cancel(); if(x.tahan) x.tahan.cancel(); });
        luncurIkut=[]; last=[]; near=-1; lay();
      };
      var akhir=null;
      var selesai=function(){
        if(!akhir) return; var f=akhir; akhir=null; f();
      };
      var geser=function(i){
        if(!pos.length) measure();
        var c0=tr.scrollLeft+tr.clientWidth/2, c1=pos[i], jarak=c1-c0;
        if(calm || Math.abs(jarak)<1){ progSL=Math.round(c1-tr.clientWidth/2); tr.scrollTo({left:c1-tr.clientWidth/2, behavior:'instant'}); return; }
        // Jarak satu kartu = 520 ms; snap pendek (setelah swipe) lebih singkat.
        var lama=Math.round(260+260*Math.min(1, Math.abs(jarak)/sp)), ikut=[];
        items.forEach(function(it, k){
          var d0=(pos[k]-c0)/sp, d1=(pos[k]-c1)/sp;
          if(Math.min(Math.abs(d0), Math.abs(d1))>6) return;
          ikut.push([it, bentuk(d0, 0), bentuk(d1, -jarak), d1]);
        });
        luncurIkut=ikut;
        // Web Animations: bentuk awal & akhir diberikan langsung (tanpa reflow
        // paksa), dijalankan compositor, dan otomatis MENIMPA coverflow CSS
        // scroll-driven (animasi skrip di atas animasi CSS) — tak ada kelas
        // yang mematikan/menyalakan ulang animasi 24 kartu (mahal di WebKit).
        // z-index hanya kartu yang ikut & berubah.
        ikut.forEach(function(x){
          var it=x[0], z=String(40-Math.abs(items.indexOf(it)-i));
          it.style.visibility='';
          if(it.style.zIndex!==z) it.style.zIndex=z;
          x.anim=it.animate([{transform:x[1].t, opacity:x[1].o}, {transform:x[2].t, opacity:x[2].o}],
            {duration:lama, easing:'cubic-bezier(0.22, 0.8, 0.24, 1)', fill:'forwards'});
        });
        near=i;
        // Serah terima: (1) animasi "tahan" di bentuk akhir TANPA pergeseran +
        // scrollLeft tujuan di tugas yang sama (identik di layar); (2) dua frame
        // kemudian animasi tahan dilepas — coverflow scroll-driven / lay() sudah
        // memakai posisi gulir baru (WebKit terlambat satu frame → dulu loncat).
        akhir=function(){
          ikut.forEach(function(x){
            var b1=bentuk(x[3], 0);
            x.tahan=x[0].animate([{transform:b1.t, opacity:b1.o}, {transform:b1.t, opacity:b1.o}], {duration:1, fill:'forwards'});
            if(x.anim) x.anim.cancel();
          });
          progSL=Math.round(c1-tr.clientWidth/2); tr.scrollLeft=c1-tr.clientWidth/2;
          last=[]; lay();
          requestAnimationFrame(function(){ requestAnimationFrame(function(){
            ikut.forEach(function(x){ if(x.tahan) x.tahan.cancel(); });
            if(luncurIkut===ikut) luncurIkut=[];
            if(!luncurT) settle(); // luncuran baru sudah dimulai → ia yang mengurus
          }); });
        };
        luncurT=setTimeout(function(){
          luncurT=0; selesai();
          if(antre>=0){ var n=antre; antre=-1; geser(n); }
        }, lama+40);
      };
      var go=function(i){
        tujuan=(i+items.length)%items.length;
        henti();
        // Klik saat masih meluncur → diantrikan (berlanjut dari tujuan itu).
        if(luncurT){ antre=tujuan; return; }
        geser(tujuan);
      };
      // Gulir oleh PENGGUNA (bukan scrollLeft yang diset luncuran) → luncuran
      // dibatalkan & pratinjau dibongkar. Sengaja TIDAK di pointerdown /
      // touchstart / wheel: menggulir HALAMAN naik-turun dengan jari atau
      // kursor di atas deretan tema memicu event itu tanpa track bergeser —
      // dulu pratinjau HP dibongkar dan tak pernah dimuat lagi (layar kosong),
      // dan luncuran yang sedang berjalan diloncatkan.
      tr.addEventListener('scroll', function(){
        var sl=Math.round(tr.scrollLeft);
        if(progSL>=0 && Math.abs(sl-progSL)<=1) progSL=-1;
        else { progSL=-1; if(!seret || seret.gerak){ userT=Date.now(); batalLuncur(); henti(); } }
        if(!raf) raf=requestAnimationFrame(lay);
        clearTimeout(idleT); idleT=setTimeout(settle, 140);
      }, {passive:true});
      ['pointerdown', 'touchstart'].forEach(function(t){ tr.addEventListener(t, function(){ userT=Date.now(); }, {passive:true}); });
      // Desktop: seret dengan mouse seperti di HP. Selama diseret snap
      // dimatikan; dilepas → meluncur ke kartu terdekat searah seretan.
      var seret=null;
      tr.addEventListener('pointerdown', function(e){
        if(e.pointerType!=='mouse' || e.button!==0) return;
        seret={x:e.clientX, s:tr.scrollLeft, gerak:false, id:e.pointerId};
      });
      tr.addEventListener('pointermove', function(e){
        if(!seret || e.pointerId!==seret.id) return;
        var dx=e.clientX-seret.x;
        if(!seret.gerak && Math.abs(dx)>5){ seret.gerak=true; tr.setPointerCapture(e.pointerId); tr.classList.add('is-seret'); }
        if(seret.gerak) tr.scrollLeft=seret.s-dx;
      });
      var lepasSeret=function(e){
        if(!seret || e.pointerId!==seret.id) return;
        var s0=seret; seret=null;
        if(!s0.gerak) return;
        tr.classList.remove('is-seret'); tr._abaikanKlik=true;
        var b=lay(), arah=tr.scrollLeft>s0.s ? 1 : -1;
        // Seretan pendek tetap pindah satu tema searah seretan.
        go(b===act ? act+arah : b);
      };
      tr.addEventListener('pointerup', lepasSeret);
      tr.addEventListener('pointercancel', lepasSeret);
      // Klik tema samping → bawa ke tengah (bukan membuka demo).
      tr.addEventListener('click', function(e){
        if(tr._abaikanKlik){ tr._abaikanKlik=false; e.preventDefault(); return; }
        var it=e.target.closest && e.target.closest('.etalase__item'); if(!it) return;
        var i=items.indexOf(it); if(i!==act){ e.preventDefault(); userT=Date.now(); go(i); }
      });
      tr.addEventListener('dragstart', function(e){ e.preventDefault(); });
      sec.querySelectorAll('[data-etalase-nav]').forEach(function(b){
        // Klik beruntun sebelum geseran selesai → maju dari tujuan terakhir.
        b.addEventListener('click', function(){ userT=Date.now(); go((tujuan>=0 && tujuan!==act ? tujuan : act)+(+b.getAttribute('data-etalase-nav'))); });
      });
      if('IntersectionObserver' in window){
        track(new IntersectionObserver(function(es){
          inView=es[0].isIntersecting;
          if(inView) settle(); else { henti(); clearTimeout(autoT); if(progres) progres.classList.remove('is-jalan'); }
        }, {threshold:0.45}), sec);
        if(lp) track(new IntersectionObserver(function(es){
          lpView=es[0].isIntersecting;
          if(lpView){ if(act>=0 && !(lpM && lpM.cur())) mulaiLaptop(); }
          else { clearTimeout(lpT); if(lpM) lpM.stop(); }
        }, {threshold:0.3}), lp.closest('.etalase__laptop'));
      } else { inView=true; settle(); }
      etaResume=function(){ if(inView && hp.isConnected) mulai(); };
      // Ukuran berubah (putar layar / jendela) → ukur ulang posisi kartu.
      // ResizeObserver pada track (bukan listener window) — ikut terbuang
      // bersama elemennya saat pindah halaman; timer di atas berhenti sendiri
      // (cek tr.isConnected), jadi tak ada yang menahan etalase lama.
      if(window.ResizeObserver){
        var lebar=0;
        new ResizeObserver(function(es){
          var w=Math.round(es[0].contentRect.width); if(w===lebar) return; lebar=w;
          if(pos.length){ pos=[]; last=[]; lay(); }
        }).observe(tr);
      }
      // Posisi kartu baru diukur saat etalase pertama terlihat (callback IO,
      // setelah layout selesai) — mengukur di sini memaksa layout seluruh
      // beranda yang belum selesai dimuat.
    });
  };
  etaInit(); onDom(etaInit);
  // Formulir berbahaya (hapus permanen): <form data-confirm="Pesan?">.
  d.addEventListener('submit', function(e){
    var f=e.target; if(!f.matches || !f.matches('form[data-confirm]')) return;
    if(!confirm(f.getAttribute('data-confirm'))){ e.preventDefault(); e.stopImmediatePropagation(); }
  }, true);
  // ── Story tamu (tab Story) ────────────────────────────────────────────
  // Penampil ala story e-ticketing/Instagram: progress 5 dtk (mulai setelah
  // foto termuat), tap kiri = mundur / kanan = maju, tahan = jeda, geser
  // kiri-kanan = kubus 3D pindah tamu, tarik ke bawah = tutup, ←/→/Esc.
  // Data dari <script id="story-data"> (JSON dari server). Hanya foto.
  // Sumber lain lewat [data-story-src] (story panduan beranda: #panduan-data,
  // data-seg="1" = progress bersegmen, item boleh punya link+cta = tombol aksi).
  (function(){
    var DUR=5000, AXIS=12, COMMIT=0.25, FLICK=0.55, CLOSE_PX=130, HOLD_MS=220;
    var S=null;
    function srcEl(id){ return d.getElementById(id||'story-data'); }
    function list(el){ try{ return el ? JSON.parse(el.textContent||'[]') : []; }catch(_){ return []; } }
    // Tanda "sudah dilihat" per baris story (data-story-slug) — undangan & panduan terpisah.
    function keyOf(bar){ return 'ily_story_seen_'+(bar ? bar.getAttribute('data-story-slug') : ''); }
    // Tombol di luar baris (grid story / Kelola) → baris story undangan di halaman ini.
    function barOf(b){ return b.closest('[data-story-slug]') || (b.getAttribute('data-story-src') ? null : d.querySelector('[data-story-slug]:not([data-story-slug="_panduan"])')); }
    function seen(key){ try{ return JSON.parse(localStorage.getItem(key)||'[]'); }catch(_){ return []; } }
    function markSeen(key, id){
      var a=seen(key); if(a.indexOf(id)<0){ a.push(id); try{ localStorage.setItem(key, JSON.stringify(a.slice(-300))); }catch(_){} }
      d.querySelectorAll('[data-story-id="'+id+'"]').forEach(function(b){ if(keyOf(barOf(b))===key) b.classList.add('is-seen'); });
    }
    function paintSeen(){
      d.querySelectorAll('[data-story-id]').forEach(function(b){ var a=seen(keyOf(barOf(b))); b.classList.toggle('is-seen', a.indexOf(+b.getAttribute('data-story-id'))>=0); });
    }
    // Penampil hidup di <body> (di luar akar Leptos): halaman Story ditinggal
    // (Back / pindah tab SPA) → tutup, agar gulir tak terkunci & foto dilepas.
    onDom(function(){ paintSeen(); if(S && !S.src.isConnected) close(true); });
    window.addEventListener('popstate', function(){ if(S) close(true); });
    function el(tag, cls, html){ var e=d.createElement(tag); if(cls) e.className=cls; if(html!=null) e.innerHTML=html; return e; }
    function esc(t){ return String(t==null?'':t).replace(/[&<>"']/g, function(c){ return {'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]; }); }
    function fcls(f){ return 'sf-'+(/^[a-z]+$/.test(f||'') ? f : 'normal'); }
    var DEL_SVG='<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14a2 2 0 01-2 2H8a2 2 0 01-2-2L5 6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/></svg>';
    // Hapus story milik perangkat ini (cookie pembuat) — POST biasa ke server.
    // Pengelola undangan (Kelola): data-owner → boleh menghapus SEMUA story.
    function delMine(id){
      var src=S && S.src; if(!src) return;
      var owner=src.getAttribute('data-owner')==='1';
      if(!confirm(owner ? 'Hapus story tamu ini secara permanen? Foto ikut terhapus.' : T('Hapus story Anda secara permanen?','Permanently delete your story?'))) return;
      var f=d.createElement('form'); f.method='post'; f.action=owner ? src.getAttribute('data-del-owner') : src.getAttribute('data-del');
      var fields=owner ? [['id', id], ['key', src.getAttribute('data-key')||'']] : [['id', id], ['back', src.getAttribute('data-back')||'']];
      fields.forEach(function(kv){ var i=d.createElement('input'); i.type='hidden'; i.name=kv[0]; i.value=kv[1]; f.appendChild(i); });
      close(true); d.body.appendChild(f); f.submit();
    }
    var X_SVG='<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>';
    function face(it, side){
      var f=el('div','sv-face-neighbor '+side);
      f.innerHTML='<img class="sv-media '+fcls(it.filter)+'" src="'+esc(it.photo)+'" alt=""><div class="sv-face-scrim"></div>'+
        '<div class="sv-face-id"><div class="sv-avatar-ring sv-face-avatar"><img class="sv-avatar '+fcls(it.filter)+'" src="'+esc(it.photo)+'" alt=""></div><span class="sv-face-username">'+esc(it.name)+'</span></div>';
      return f;
    }
    function open(i, srcId, key){
      var src=srcEl(srcId), items=list(src); if(!items.length) return;
      close(true);
      var segN=src.getAttribute('data-seg')==='1' ? items.length : 1, segs='';
      for(var k=0;k<segN;k++) segs+='<div class="sv-seg"><div class="sv-seg-fill"></div></div>';
      var root=el('div','sv-portal');
      root.innerHTML='<div class="sv-backdrop"></div><div class="sv-scene"><div class="sv-cube"><div class="sv-container">'+
        '<div class="sv-progress-row">'+segs+'</div>'+
        '<div class="sv-header"><div class="sv-header-left"><div class="sv-avatar-ring"><img class="sv-avatar" alt=""></div>'+
        '<div class="sv-user-info"><span class="sv-username"></span><span class="sv-meta"></span></div></div>'+
        '<div class="sv-header-right"><button type="button" class="sv-del-btn" aria-label="'+T('Hapus story saya','Delete my story')+'" hidden>'+DEL_SVG+'</button><button type="button" class="sv-close-btn" aria-label="'+T('Tutup story','Close story')+'">'+X_SVG+'</button></div></div>'+
        '<div class="sv-media-area"><div class="sv-img-shell sv-img-loading"><div class="sv-shimmer"></div><img class="sv-media" alt="" draggable="false"></div></div>'+
        '<p class="sv-caption"></p><a class="btn btn--primary sv-cta" hidden></a></div></div></div>';
      if(src.getAttribute('data-fit')==='contain') root.classList.add('sv-contain');
      d.body.appendChild(root);
      d.documentElement.classList.add('sv-open');
      S={items:items, i:-1, src:src, key:key||keyOf(null), root:root, scene:root.querySelector('.sv-scene'), cube:root.querySelector('.sv-cube'), box:root.querySelector('.sv-container'),
         bd:root.querySelector('.sv-backdrop'), fills:root.querySelectorAll('.sv-seg-fill'), fill:null, img:root.querySelector('.sv-media-area .sv-media'),
         shell:root.querySelector('.sv-img-shell'), raf:0, start:0, elapsed:0, paused:false, ready:false, settling:false, drag:null};
      S.bd.addEventListener('click', function(){ close(); });
      root.querySelector('.sv-close-btn').addEventListener('click', function(e){ e.stopPropagation(); close(); });
      root.querySelector('.sv-close-btn').addEventListener('pointerdown', function(e){ e.stopPropagation(); });
      // Tombol aksi: jangan dianggap "tap maju"; tutup penampil lalu navigasi biasa.
      var cta=root.querySelector('.sv-cta');
      cta.addEventListener('pointerdown', function(e){ e.stopPropagation(); });
      cta.addEventListener('click', function(){ close(true); });
      var del=root.querySelector('.sv-del-btn');
      del.addEventListener('pointerdown', function(e){ e.stopPropagation(); });
      del.addEventListener('click', function(e){ e.stopPropagation(); if(S) delMine(S.items[S.i].id); });
      S.img.addEventListener('load', function(){ if(!S) return; S.shell.classList.remove('sv-img-loading'); S.img.classList.add('sv-img-visible'); S.ready=true; S.start=performance.now()-S.elapsed; tick(); });
      S.img.addEventListener('error', function(){ if(S){ S.ready=true; S.start=performance.now(); tick(); } });
      S.box.addEventListener('pointerdown', down);
      S.box.addEventListener('pointermove', move);
      S.box.addEventListener('pointerup', up);
      S.box.addEventListener('pointercancel', up);
      show(i);
    }
    function show(i){
      if(!S) return;
      if(i<0) i=0;
      if(i>=S.items.length){ close(); return; }
      var it=S.items[i]; S.i=i; S.elapsed=0; S.ready=false;
      cancelAnimationFrame(S.raf);
      // Bersegmen: segmen sebelumnya penuh, sesudahnya kosong.
      if(S.fills.length>1){ S.fills.forEach(function(f, k){ f.style.transform='scaleX('+(k<i?1:0)+')'; }); S.fill=S.fills[i]; }
      else { S.fill=S.fills[0]; S.fill.style.transform='scaleX(0)'; }
      S.shell.classList.add('sv-img-loading'); S.img.classList.remove('sv-img-visible');
      S.img.className='sv-media '+fcls(it.filter);
      S.img.src=it.photo;
      var av=S.root.querySelector('.sv-avatar'); av.src=it.photo; av.className='sv-avatar '+fcls(it.filter);
      S.root.querySelector('.sv-username').textContent=it.name;
      S.root.querySelector('.sv-meta').textContent=it.ago||'';
      var cap=S.root.querySelector('.sv-caption'); cap.textContent=it.caption||''; cap.hidden=!it.caption;
      var cta=S.root.querySelector('.sv-cta'), go=it.link && /^(\/(?!\/)|https:\/\/)/.test(it.link);
      cta.hidden=!go; if(go){ cta.href=it.link; cta.textContent=it.cta||'Buka'; }
      var src=S.src, mine=(src.getAttribute('data-mine')||'').split(',');
      var owner=src.getAttribute('data-owner')==='1';
      S.root.querySelector('.sv-del-btn').hidden=!owner && mine.indexOf(String(it.id))<0;
      if(S.img.complete && S.img.naturalWidth){ S.shell.classList.remove('sv-img-loading'); S.img.classList.add('sv-img-visible'); S.ready=true; S.start=performance.now(); tick(); }
      markSeen(S.key, it.id);
      var nx=S.items[i+1]; if(nx){ var p=new Image(); p.src=nx.photo; }
    }
    function tick(){
      if(!S) return;
      cancelAnimationFrame(S.raf);
      S.raf=requestAnimationFrame(function step(now){
        if(!S || !S.ready) return;
        if(S.paused){ S.start=now-S.elapsed; S.raf=requestAnimationFrame(step); return; }
        S.elapsed=now-S.start;
        var k=Math.min(1, S.elapsed/DUR); S.fill.style.transform='scaleX('+k+')';
        if(k>=1){ show(S.i+1); return; }
        S.raf=requestAnimationFrame(step);
      });
    }
    function close(now){
      if(!S) return;
      cancelAnimationFrame(S.raf);
      var r=S.root; S=null;
      d.documentElement.classList.remove('sv-open');
      if(now){ r.remove(); return; }
      r.classList.add('is-closing'); setTimeout(function(){ r.remove(); }, 200);
    }
    function settle(fn, ms){ S.settling=true; setTimeout(function(){ if(!S) return; S.settling=false; fn(); }, ms); }
    function down(e){
      if(!S || S.settling || e.button!==0) return;
      S.drag={x:e.clientX, y:e.clientY, t:performance.now(), lx:e.clientX, ly:e.clientY, lt:performance.now(), axis:null, id:e.pointerId};
      S.paused=true;
      var w=S.cube.offsetWidth; S.w=w||400; S.h=S.cube.offsetHeight||800; S.cube.style.setProperty('--sv-w', S.w+'px');
    }
    function move(e){
      if(!S || !S.drag || S.settling) return;
      var g=S.drag, dx=e.clientX-g.x, dy=e.clientY-g.y;
      g.lx=e.clientX; g.ly=e.clientY; g.lt=performance.now();
      if(!g.axis && (Math.abs(dx)>AXIS || Math.abs(dy)>AXIS)){
        g.axis = Math.abs(dx)>Math.abs(dy) ? 'h' : (dy>0 ? 'down' : 'up');
        if(g.axis==='h'){
          try{ S.box.setPointerCapture(g.id); }catch(_){}
          S.cube.classList.add('is-3d');
          var pv=S.items[S.i-1], nx=S.items[S.i+1];
          if(pv) S.cube.insertBefore(face(pv,'sv-face-prev'), S.box);
          if(nx) S.cube.appendChild(face(nx,'sv-face-next'));
        }
      }
      if(g.axis==='h'){
        e.preventDefault();
        var deg=dx/S.w*90; if(deg>0 && S.i===0) deg*=0.25;
        deg=Math.max(-90, Math.min(90, deg));
        S.cube.style.transition='none';
        S.cube.style.transform='translateZ(calc(var(--sv-w) / -2)) rotateY('+deg.toFixed(3)+'deg)';
      } else if(g.axis==='down'){
        e.preventDefault();
        var dd=Math.max(0, dy), sc=Math.max(0.75, 1-dd/S.h*0.25);
        S.scene.style.transition='none';
        S.scene.style.transform='translateX(-50%) translateY('+dd.toFixed(1)+'px) scale('+sc.toFixed(4)+')';
        S.scene.style.borderRadius='18px';
        S.bd.style.opacity=Math.max(0.2, 1-dd/600).toFixed(3);
      }
    }
    function resetCube(){
      if(!S) return;
      S.cube.style.transition=''; S.cube.style.transform=''; S.cube.classList.remove('is-3d');
      S.cube.querySelectorAll('.sv-face-neighbor').forEach(function(f){ f.remove(); });
    }
    function up(e){
      if(!S || !S.drag || S.settling) return;
      var g=S.drag; S.drag=null; S.paused=false;
      try{ if(S.box.hasPointerCapture(g.id)) S.box.releasePointerCapture(g.id); }catch(_){}
      var dx=g.lx-g.x, dy=g.ly-g.y, dt=Math.max(1, g.lt-g.t);
      if(g.axis==='h'){
        var commit=Math.abs(dx)>S.w*COMMIT || (Math.abs(dx/dt)>FLICK && Math.abs(dx)>40), next=dx<0;
        if(commit && (next || S.i>0)){
          S.cube.style.transition='transform .26s cubic-bezier(.2,.8,.25,1)';
          S.cube.style.transform='translateZ(calc(var(--sv-w) / -2)) rotateY('+(next?-90:90)+'deg)';
          settle(function(){ resetCube(); show(S.i+(next?1:-1)); }, 280);
        } else {
          S.cube.style.transition='transform .22s cubic-bezier(.2,.8,.25,1)';
          S.cube.style.transform='translateZ(calc(var(--sv-w) / -2)) rotateY(0deg)';
          settle(resetCube, 240);
        }
      } else if(g.axis==='down'){
        if(dy>CLOSE_PX || (dy/dt>FLICK && dy>60)){
          S.scene.style.transition='transform .24s ease-in, opacity .24s ease-in';
          S.scene.style.transform='translateX(-50%) translateY(70vh) scale(.7)'; S.scene.style.opacity='0';
          S.bd.style.transition='opacity .24s ease-in'; S.bd.style.opacity='0';
          settle(function(){ close(true); }, 240);
        } else {
          S.scene.style.transition='transform .22s cubic-bezier(.2,.8,.25,1), border-radius .22s ease';
          S.scene.style.transform='translateX(-50%)'; S.scene.style.borderRadius='0px';
          S.bd.style.transition='opacity .2s ease'; S.bd.style.opacity='1';
          settle(function(){ ['transition','transform','borderRadius','opacity'].forEach(function(k){ S.scene.style[k]=''; S.bd.style[k]=''; }); }, 240);
        }
      } else if(!g.axis && dt<HOLD_MS){
        // Tap: sepertiga kiri = mundur, sisanya = maju (perilaku Instagram).
        var r=S.box.getBoundingClientRect();
        if(g.x-r.left < r.width*0.33) show(S.i-1 < 0 ? 0 : S.i-1); else show(S.i+1);
      }
    }
    d.addEventListener('click', function(e){
      var b=e.target.closest && e.target.closest('[data-story-open]'); if(!b) return;
      // Tombol/form di dalam kartu (hapus) bukan "buka story".
      var inner=e.target.closest('button, form, a'); if(inner && inner!==b && b.contains(inner)) return;
      e.preventDefault(); open(+b.getAttribute('data-story-open'), b.getAttribute('data-story-src'), keyOf(barOf(b)));
    });
    // Kartu non-tombol (figure role=button di Kelola): Enter / spasi = buka.
    d.addEventListener('keydown', function(e){
      if((e.key!=='Enter' && e.key!==' ') || !e.target.matches || !e.target.matches('[data-story-open][role=button]')) return;
      e.preventDefault(); open(+e.target.getAttribute('data-story-open'), e.target.getAttribute('data-story-src'), keyOf(barOf(e.target)));
    });
    d.addEventListener('keydown', function(e){
      if(!S) return;
      if(e.key==='Escape') close();
      else if(e.key==='ArrowRight') show(S.i+1);
      else if(e.key==='ArrowLeft') show(Math.max(0, S.i-1));
    });
    d.addEventListener('visibilitychange', function(){ if(S) S.paused=d.hidden; });
    // Formulir: pratinjau foto (blob lokal, tak diunggah sebelum dikirim) + filter.
    // URL blob dilepas saat diganti, formulirnya hilang (SPA), atau tab ditinggal.
    var blobPv=new Set();
    var dropPv=function(all){ blobPv.forEach(function(pv){ if(all || !pv.isConnected){ URL.revokeObjectURL(pv.dataset.blob); delete pv.dataset.blob; blobPv.delete(pv); } }); };
    onDom(function(){ if(blobPv.size) dropPv(false); });
    window.addEventListener('pagehide', function(){ dropPv(true); });
    d.addEventListener('change', function(e){
      var t=e.target;
      if(t.matches && t.matches('[data-story-file]')){
        var f=t.files && t.files[0], pv=t.closest('form').querySelector('[data-story-preview]'); if(!pv) return;
        if(pv.dataset.blob){ URL.revokeObjectURL(pv.dataset.blob); blobPv.delete(pv); }
        if(f && /^image\/(jpeg|png|webp)$/.test(f.type)){ var u=URL.createObjectURL(f); pv.dataset.blob=u; pv.src=u; pv.hidden=false; blobPv.add(pv); }
        else { pv.hidden=true; pv.removeAttribute('src'); if(f){ t.value=''; alert('Story hanya boleh foto JPEG/PNG/WebP — video tidak bisa.'); } }
      } else if(t.matches && t.matches('[data-story-filter]')){
        var p2=t.closest('form').querySelector('[data-story-preview]'); if(p2) p2.className='story-pick__img sf-'+t.value;
      }
    });
  })();
  new MutationObserver(function(){ if(!domPending){ domPending=true; requestAnimationFrame(runDom); } })
    .observe(d.body, {childList:true, subtree:true, attributes:true, attributeFilter:['data-load-more']});
  // Tamu yang sudah membuka undangan di tab ini: lanjutkan musik saat pindah halaman penuh.
  try{ if(!PV && sessionStorage.getItem(KEY)==='1' && masihDiUndangan()){ d.documentElement.classList.add('inv-opened','inv-seen'); var a=audio(); if(a&&a.dataset.autoplay!=='false') play(); } }catch(e){}
})();
