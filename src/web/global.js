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
  // Pratinjau tema di kartu katalog (iframe /u/…?pv=1): mode SENYAP — tanpa
  // musik, tanpa menulis sessionStorage (dipakai bersama tab induk), tanpa
  // pemulihan posisi; gerbang dibuka & halaman digulir otomatis (lihat bawah).
  var PV=window.top!==window && /[?&]pv=1(&|$)/.test(location.search);
  // Satu MutationObserver untuk semua tugas yang perlu tahu DOM berubah
  // (dijadwalkan sekali per frame, bukan per mutasi).
  var domTasks=[], domPending=false;
  function onDom(fn){ domTasks.push(fn); }
  function runDom(){ domPending=false; domTasks.forEach(function(fn){ fn(); }); }
  function audio(){ return d.getElementById('bgm'); }
  function start(a){ var p=a.play(); if(p&&p.catch) p.catch(function(){}); }
  function sync(){ var a=audio(); d.documentElement.classList.toggle('bgm-playing', !!a && !a.paused); }
  function play(){ if(PV) return; var a=audio(); if(!a||!a.getAttribute('src')) return; start(a); try{sessionStorage.setItem(KEY,'1')}catch(e){} }
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
    var el=e.target.closest('[data-music],[data-open],[data-copy],[data-song],[data-demo-open]');
    if(!el) return;
    if(el.hasAttribute('data-demo-open')){
      // Demo tema: buka gerbang HANYA di pratinjau ini (bukan html.inv-opened).
      burst(el);
      var emb=el.closest('.inv--embed'); if(emb) emb.classList.add('is-open');
      var ph=el.closest('.demo-phone, .adm-preview__inv'); if(ph) ph.scrollTop=0;
      play(); setTimeout(rvScan, 700); return;
    }
    if(el.hasAttribute('data-open')){
      burst(el); play(); d.documentElement.classList.add('inv-opened');
      if(d.querySelector('.gate')) window.scrollTo(0,0);
      setTimeout(rvScan, 700);
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
      var ok=function(){ toast(el.getAttribute('data-copied')||'Tersalin'); };
      if(navigator.clipboard) navigator.clipboard.writeText(v).then(ok,ok);
      else { var ta=d.createElement('textarea'); ta.value=v; d.body.appendChild(ta); ta.select(); try{d.execCommand('copy')}catch(_){} ta.remove(); ok(); }
    }
  });
  // Semburan kelopak & kilau emas dari tombol "Buka Undangan".
  function burst(el){
    if(calm) return;
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
      (function loop(){
        if(!d.body.contains(v)){ stream.getTracks().forEach(function(t){t.stop()}); return; }
        if(!busy && v.readyState>=2){
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
  var blobs=[];
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
    if(!inp.matches || !inp.matches('input[type=file][data-preview]')) return;
    drop(inp);
    var box=inp.closest('[data-preview-box]'), out=box && box.querySelector('[data-preview-out]');
    if(out) out.textContent='';
    var f=inp.files && inp.files[0]; if(!f) return;
    var max=parseFloat(inp.dataset.max||'0');
    if(max && f.size>max*1048576){ toast('Berkas terlalu besar (maks '+max+' MB)'); inp.value=''; return; }
    var url=URL.createObjectURL(f), rec={el:inp,url:url,targets:[]};
    blobs.push(rec);
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
        if(el.dataset.rv){ var step=parseInt(getComputedStyle(el).getPropertyValue('--rv-stagger'),10)||90; el.style.transitionDelay=Math.min(k++, step>150?8:6)*step+'ms'; }
        el.classList.add('is-in'); io.unobserve(el); watched.delete(el);
      });
    }, {rootMargin:'0px 0px -6% 0px'});
    // Elemen yang diamati tapi sudah hilang (pindah halaman sebelum terlihat)
    // dilepas dari observer — tak ada elemen lama yang tertahan di memori.
    var watched=new Set();
    var observe=function(el){ watched.add(el); io.observe(el); };
    function scan(){
      watched.forEach(function(el){ if(!el.isConnected){ io.unobserve(el); watched.delete(el); } });
      if(!d.querySelector('.inv')) return;
      // Selama sampul/gerbang belum dibuka, jangan "habiskan" animasi isi yang
      // tersembunyi di baliknya — tunggu sampai dibuka.
      var gateClosed = d.querySelector('.gate:not(.gate--embed)') && !d.documentElement.classList.contains('inv-opened');
      d.querySelectorAll('.inv__main > *:not(.stack):not(.gate):not(.cover), .inv__isi > *, .inv__anchor > *, .inv .section > *:not(.gallery):not(.orn-layer), .inv .gallery > *, .inv .cover > *:not(.orn-layer), .inv .couple > .person, .inv--embed > *:not(.gate):not(.inv__glow)').forEach(function(el){
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
  // Katalog: infinite scroll — tombol "Tampilkan lebih banyak" ([data-load-more],
  // dipasang Leptos setelah hydrate) diklik otomatis saat ±600px dari layar.
  // Kliknya ditangani Leptos (tanpa navigasi) → halaman tak melompat ke atas.
  if('IntersectionObserver' in window){
    var moreSeen=new Set();
    var moreIO=new IntersectionObserver(function(es){
      es.forEach(function(e){ if(e.isIntersecting && e.target.isConnected && e.target.dataset.loadMore==='1') e.target.click(); });
    }, {rootMargin:'0px 0px 600px 0px'});
    var watchMore=function(){
      moreSeen.forEach(function(el){ if(!el.isConnected){ moreIO.unobserve(el); moreSeen.delete(el); } });
      d.querySelectorAll('[data-load-more="1"]').forEach(function(el){ if(!moreSeen.has(el)){ moreSeen.add(el); moreIO.observe(el); } });
    };
    watchMore();
    onDom(watchMore);
  }
  // ── Pratinjau tema di katalog ───────────────────────────────────────────
  // Kartu tema punya wadah kosong `.tcard__pv[data-pv=URL demo]` (dirender
  // Leptos tanpa anak → iframe yang disisipkan tak mengganggu hydrate).
  // Desktop: langsung saat hover. Layar sentuh: kartu yang disentuh, atau kartu
  // paling tengah di layar setelah gulir berhenti. Hanya SATU iframe hidup.
  if(PV){
    d.documentElement.classList.add('is-pv');
    // Skrip ini di akhir <body> → DOM sudah lengkap: beri tahu kartu induk
    // SEKARANG (tak menunggu gambar), buka gerbang, lalu mulai bergulir.
    try{ parent.postMessage({pv:'ready'}, location.origin); }catch(_){}
    var pvRun=function(){
      var o=d.querySelector('.gate [data-open]') || d.querySelector('[data-open]');
      if(o && !d.documentElement.classList.contains('inv-opened')) o.click();
      if(calm) return;
      // ±90 px/dtk ke bawah; di dasar jeda, kembali ke atas, ulangi.
      var y=0, last=0, wait=performance.now()+(o?900:300);
      (function run(t){
        if(t<wait){ last=0; requestAnimationFrame(run); return; }
        var max=d.documentElement.scrollHeight-innerHeight;
        if(y<0){ y=0; window.scrollTo({top:0, behavior:'smooth'}); wait=t+1500; requestAnimationFrame(run); return; }
        var dt=last ? Math.min(t-last, 50) : 16; last=t;
        y=Math.min(Math.max(max,0), y+dt*0.09);
        window.scrollTo({top:y, behavior:'instant'});
        if(max>0 && y>=max){ y=-1; wait=t+1800; }
        requestAnimationFrame(run);
      })(performance.now());
    };
    pvRun();
  } else if(!(navigator.connection && navigator.connection.saveData)){
    var pvCur=null, pvTimer=0, PW=390;
    var pvStop=function(){
      clearTimeout(pvTimer);
      if(!pvCur) return;
      var f=pvCur.querySelector('iframe'); if(f) f.remove();
      if(pvCur.parentNode) pvCur.parentNode.classList.remove('is-pv');
      pvCur=null;
    };
    var pvStart=function(h){
      if(pvCur===h && h.isConnected) return;
      pvStop(); pvCur=h;
      var art=h.parentNode, r=art.getBoundingClientRect(), s=r.width/PW;
      if(!s) return;
      var f=d.createElement('iframe');
      f.title='Pratinjau tema'; f.tabIndex=-1; f.setAttribute('aria-hidden','true');
      f.style.width=PW+'px'; f.style.height=Math.ceil(r.height/s)+'px'; f.style.transform='scale('+s+')';
      f.onload=function(){ if(pvCur===h) art.classList.add('is-pv'); };
      f.src=h.getAttribute('data-pv');
      h.appendChild(f);
    };
    // Iframe memberi tahu begitu HTML-nya terurai (lebih cepat dari onload).
    window.addEventListener('message', function(e){
      if(e.origin!==location.origin || !e.data || e.data.pv!=='ready' || !pvCur) return;
      var f=pvCur.querySelector('iframe'); if(f && e.source===f.contentWindow) pvCur.parentNode.classList.add('is-pv');
    });
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
          if(!e.isIntersecting) return; pfIO.unobserve(e.target);
          var u=e.target.getAttribute('data-pv'); if(u && !pfDone.has(u)){ pfDone.add(u); pfQ.push(u); }
        });
        pfNext();
      }, {rootMargin:'150px 0px'});
      var pfSeen=new WeakSet();
      var pfWatch=function(){ d.querySelectorAll('.tcard__pv[data-pv]').forEach(function(h){ if(!pfSeen.has(h)){ pfSeen.add(h); pfIO.observe(h); } }); };
      var pfStart=function(){ pfWatch(); onDom(pfWatch); };
      // Setelah halaman sendiri selesai dimuat — prefetch tak berebut dengan katalog.
      if(d.readyState==='complete') setTimeout(pfStart, 300); else window.addEventListener('load', function(){ setTimeout(pfStart, 300); });
    }
    if(matchMedia('(hover: hover) and (pointer: fine)').matches){
      d.addEventListener('mouseover', function(e){
        var art=e.target.closest && e.target.closest('.tcard__art'); if(!art) return;
        var h=art.querySelector('[data-pv]'); if(!h || h===pvCur) return;
        pvStart(h);
      });
      d.addEventListener('mouseout', function(e){
        var art=e.target.closest && e.target.closest('.tcard__art'); if(!art) return;
        if(e.relatedTarget && art.contains(e.relatedTarget)) return;
        if(pvCur && art.contains(pvCur)) pvStop();
      });
    } else {
      var pvPick=function(){
        var best=null, bd=1e9, mid=innerHeight/2;
        d.querySelectorAll('.tcard__art > [data-pv]').forEach(function(h){
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
  }
  new MutationObserver(function(){ if(!domPending){ domPending=true; requestAnimationFrame(runDom); } })
    .observe(d.body, {childList:true, subtree:true, attributes:true, attributeFilter:['data-load-more']});
  // Tamu yang sudah membuka undangan di tab ini: lanjutkan musik saat pindah halaman penuh.
  try{ if(!PV && sessionStorage.getItem(KEY)==='1'){ d.documentElement.classList.add('inv-opened'); var a=audio(); if(a&&a.dataset.autoplay!=='false') play(); } }catch(e){}
})();
