// Uji regresi memori navigasi: pergi-pulang N kali antara dua halaman, paksa
// GC tiap sampel, catat heap JS, node DOM, listener, node terlepas & memori
// WASM. Gagal (exit 1) bila node/heap akhir > AMBANG di atas sampel ke-1
// (sampel ke-0 = sebelum hydrate/pemanasan, tidak dipakai sebagai patokan).
//
//   npm i --no-save playwright-core
//   EXE=<chrome/chrome-headless-shell> node scripts/uji/memori.mjs http://127.0.0.1:3600 [N=30]
//   env: START=/  LINK='a[href^="/tema/"]'  PV=off (matikan iframe pratinjau)
//        AMBANG=0.10  NOGLOBAL=1 (blokir /app.js — memisahkan global.js vs WASM)
//        ARGS='' (nyalakan lagi bfcache & prerender)
//
// Okt 2026: navigasi SPA bocor +95…380 node/putaran (ditahan
// __wbindgen_externrefs); repro minimal Leptos tidak bocor → pemicu spesifik
// aplikasi. Halaman publik kini navigasi penuh (global.js).
import { chromium } from 'playwright-core';

const [base = 'http://127.0.0.1:3600', nStr = '30'] = process.argv.slice(2);
const N = +nStr, START = process.env.START || '/', START_PATH = new URL(START, 'http://x').pathname, LINK = process.env.LINK || 'a[href^="/tema/"]';
const AMBANG = +(process.env.AMBANG || 0.1);
// bfcache & prerender (speculationrules) dimatikan: dokumen yang disimpan
// browser ikut terhitung di Nodes/heap renderer & tumbuh sampai Chrome
// membuangnya — itu kebijakan browser, bukan kebocoran aplikasi.
const ARGS = process.env.ARGS ?? '--disable-back-forward-cache --disable-features=Prerender2,SpeculationRulesPrefetchProxy';
const browser = await chromium.launch({ executablePath: process.env.EXE || undefined, args: ARGS.split(' ').filter(Boolean) });
const ctx = await browser.newContext({ viewport: { width: 1280, height: 900 } });
await ctx.addInitScript((pvOff) => {
  // Tangkap WebAssembly.Memory saat Leptos meng-instantiate WASM.
  const wrap = (fn) => async function (...a) {
    const r = await fn.apply(this, a);
    const inst = r.instance || r;
    const mem = inst.exports && Object.values(inst.exports).find((x) => x instanceof WebAssembly.Memory);
    if (mem) window.__wasmMem = mem;
    return r;
  };
  WebAssembly.instantiate = wrap(WebAssembly.instantiate);
  WebAssembly.instantiateStreaming = wrap(WebAssembly.instantiateStreaming);
  if (pvOff) Object.defineProperty(navigator, 'connection', { value: { saveData: true }, configurable: true });
}, process.env.PV === 'off');
if (process.env.NOGLOBAL) await ctx.route(/\/app\.js/, (r) => r.fulfill({ status: 200, contentType: 'text/javascript', body: '' }));

const page = await ctx.newPage();
let cdp;
const sambung = async () => { cdp = await ctx.newCDPSession(page); await cdp.send('Performance.enable'); };
await sambung();
// Navigasi dokumen penuh mengganti target CDP di beberapa versi → sambung ulang.
page.on('framenavigated', (f) => { if (f === page.mainFrame()) sambung().catch(() => {}); });

async function sampel(i) {
  for (let k = 0; k < 3; k++) await cdp.send('HeapProfiler.collectGarbage');
  const m = Object.fromEntries((await cdp.send('Performance.getMetrics')).metrics.map((x) => [x.name, x.value]));
  let detached = -1;
  try { detached = (await cdp.send('DOM.getDetachedDomNodes')).detachedNodes.length; } catch {}
  const wasmKB = await page.evaluate(() => (window.__wasmMem ? window.__wasmMem.buffer.byteLength / 1024 : 0));
  const domNodes = await page.evaluate(() => document.getElementsByTagName('*').length);
  // Nodes/heap CDP = SELURUH dokumen di proses renderer (termasuk bfcache &
  // prerender speculationrules); domNodes = dokumen yang sedang tampil saja.
  return { i, heapKB: Math.round(m.JSHeapUsedSize / 1024), nodes: m.Nodes, documents: m.Documents, domNodes, listeners: m.JSEventListeners, detached, wasmKB };
}

await page.goto(base + START, { waitUntil: 'load' });
// Hemat data menahan hydration sampai ada interaksi → picu agar setara.
await page.evaluate(() => document.dispatchEvent(new KeyboardEvent('keydown', { bubbles: true })));
await page.waitForTimeout(2500);
const rows = [await sampel(0)];
for (let i = 1; i <= N; i++) {
  await page.locator(LINK).first().click();
  await page.waitForURL((u) => new URL(u).pathname !== START_PATH);
  await page.waitForTimeout(600);
  await page.goBack();
  await page.waitForURL((u) => new URL(u).pathname === START_PATH);
  await page.waitForTimeout(600);
  if (i === 1 || i % 5 === 0) rows.push(await sampel(i));
}
await browser.close();
console.table(rows);
const a = rows[1], z = rows[rows.length - 1];
const naik = (k) => (z[k] - a[k]) / a[k];
const hasil = { nodes: naik('nodes'), heap: naik('heapKB') };
console.log(Object.fromEntries(Object.entries(hasil).map(([k, v]) => [k, (v * 100).toFixed(1) + '%'])));
const gagal = Object.entries(hasil).filter(([, v]) => v > AMBANG);
if (gagal.length) { console.error(`GAGAL: ${gagal.map(([k]) => k).join(', ')} naik > ${AMBANG * 100}%`); process.exit(1); }
console.log('LULUS');
