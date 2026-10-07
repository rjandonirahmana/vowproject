// Banner beranda "Story Undangan" (migrasi 026): HTML → PNG (Playwright) → WebP via canvas.
//   bun scripts/banner/story.ts   (butuh playwright-core + chrome-headless-shell Playwright)
import { chromium } from "playwright-core";
const exe = process.env.HOME + "/Library/Caches/ms-playwright/chromium_headless_shell-1243/chrome-headless-shell-mac-arm64/chrome-headless-shell";
const PUB = new URL("../../public/", import.meta.url).pathname;
const L = "file://" + PUB + "img/layanan/";
const ring = (img: string, name: string, seen = false) => `<div class="it"><div class="ring${seen ? " seen" : ""}"><img src="${L}${img}"></div><span>${name}</span></div>`;
const html = (W: number, H: number, hp: boolean) => `<!doctype html><html><head><meta charset="utf-8"><style>
*{box-sizing:border-box}html,body{margin:0}
#v{position:relative;width:${W}px;height:${H}px;overflow:hidden;font-family:-apple-system,"Helvetica Neue",Arial,sans-serif;
 background:radial-gradient(60% 140% at ${hp?78:82}% 50%,#7a3a62 0,transparent 70%),linear-gradient(90deg,#1c0f22 0%,#2b1631 ${hp?40:45}%,#4a2140 100%)}
.dots{position:absolute;inset:0;background-image:radial-gradient(rgba(255,214,150,.35) 1px,transparent 1.5px);background-size:46px 46px;opacity:.35;
 -webkit-mask:linear-gradient(90deg,transparent ${hp?35:40}%,#000)}
.glow{position:absolute;right:${hp?6:10}%;top:50%;width:${hp?700:1100}px;height:${hp?700:1100}px;margin-top:-${hp?350:550}px;border-radius:50%;background:radial-gradient(closest-side,rgba(255,196,140,.22),transparent)}
.rings{position:absolute;${hp?"display:none":"display:flex"};gap:${hp?22:34}px;${hp?"right:40px;bottom:46px":"right:850px;top:50%;transform:translateY(-50%)"}}
.it{display:flex;flex-direction:column;align-items:center;gap:${hp?8:12}px}
.ring{width:${hp?108:150}px;height:${hp?108:150}px;border-radius:50%;padding:${hp?4:5}px;background:conic-gradient(#f3d18a,#c86b8a,#f6e3b0,#f3d18a)}
.ring.seen{background:rgba(255,255,255,.28)}
.ring img{width:100%;height:100%;object-fit:cover;border-radius:50%;border:${hp?4:5}px solid #2b1631;display:block}
.it span{color:#fbeee0;font-size:${hp?20:26}px;font-weight:600;opacity:.92}
.card{position:absolute;${hp?"right:250px;top:34px;width:170px;height:280px":"right:640px;top:22px;width:168px;height:276px"};border-radius:20px;overflow:hidden;
 box-shadow:0 18px 40px rgba(0,0,0,.45),0 0 0 3px rgba(255,255,255,.85);transform:rotate(-6deg);background:#000}
.card img{width:100%;height:100%;object-fit:cover}
.card .bar{position:absolute;left:10px;right:10px;top:10px;height:4px;border-radius:2px;background:rgba(255,255,255,.35)}
.card .bar i{display:block;width:62%;height:100%;border-radius:2px;background:#fff}
.card .who{position:absolute;left:10px;top:22px;display:flex;align-items:center;gap:8px;color:#fff;font-size:${hp?15:13}px;font-weight:700}
.card .who b{width:${hp?28:24}px;height:${hp?28:24}px;border-radius:50%;border:2px solid #fff;background:url(${L}mua-modern.jpg) center/cover}
.card2{${hp?"right:70px;top:46px":"right:470px;top:40px"};transform:rotate(5deg)}
.wa{position:absolute;${hp?"right:150px;bottom:26px":"right:70px;top:54px"};display:flex;align-items:center;gap:14px;padding:${hp?"14px 20px":"16px 24px"};border-radius:18px;background:#fff;
 box-shadow:0 14px 34px rgba(0,0,0,.35);font-size:${hp?18:22}px;color:#1f2a1f}
.wa .ic{width:${hp?40:46}px;height:${hp?40:46}px;border-radius:50%;background:#25d366;display:grid;place-items:center;color:#fff;font-weight:800;font-size:${hp?20:22}px}
.wa small{display:block;color:#667;font-size:${hp?14:16}px;font-weight:500}
.wa b{letter-spacing:.32em;font-size:${hp?24:28}px}
.chip{position:absolute;${hp?"display:none":"right:90px;bottom:52px"};padding:12px 22px;border-radius:999px;background:rgba(255,255,255,.14);border:1px solid rgba(255,255,255,.35);color:#fbeee0;font-size:22px;font-weight:600}
.spark{position:absolute;width:6px;height:6px;border-radius:50%;background:#ffe2a0;box-shadow:0 0 12px #ffd27a}
</style></head><body><div id="v"><div class="dots"></div><div class="glow"></div>
<div class="card"><img src="${L}dekor-galeri-altar.jpg"><div class="bar"><i></i></div><div class="who"><b></b>Siti</div></div>
<div class="card card2"><img src="${L}mua-sesudah.jpg"><div class="bar"><i style="width:28%"></i></div><div class="who"><b style="background-image:url(${L}mua-solo-putri.jpg)"></b>Dimas</div></div>
<div class="rings">${hp ? ring("mua-modern.jpg","Siti")+ring("dekor-galeri-meja.jpg","Rian")+ring("venue-lawu-park.jpg","Budi",true) : ring("dekor-galeri-meja.jpg","Rian & Arini")+ring("venue-lawu-park.jpg","Budi",true)}</div>
<div class="wa"><span class="ic">✓</span><span><small>Kunci spesial via WhatsApp</small><b>482913</b></span></div>
<div class="chip">1 nomor HP = 1 story foto</div>
<i class="spark" style="right:${hp?120:1500}px;top:40px"></i><i class="spark" style="right:${hp?520:860}px;top:${hp?360:270}px"></i><i class="spark" style="right:${hp?300:60}px;top:${hp?30:150}px"></i>
</div></body></html>`;
const b = await chromium.launch({ executablePath: exe, args: ["--allow-file-access-from-files"] });
for (const [W, H, hp, out] of [[2880, 320, false, "banner-5-story.webp"], [1080, 405, true, "banner-5-story-hp.webp"]] as const) {
  const p = await b.newPage({ viewport: { width: W, height: H } });
  await Bun.write("/tmp/_bns.html", html(W, H, hp));
  await p.goto("file://" + "/tmp/_bns.html"); await p.waitForTimeout(900);
  const png = await p.screenshot({ type: "png", clip: { x: 0, y: 0, width: W, height: H } });
  const webp = await p.evaluate(async (b64) => { const i = new Image(); i.src = "data:image/png;base64," + b64; await i.decode(); const c = document.createElement("canvas"); c.width = i.width; c.height = i.height; c.getContext("2d")!.drawImage(i, 0, 0); return c.toDataURL("image/webp", 0.86).split(",")[1]; }, png.toString("base64"));
  await Bun.write(PUB + "img/banner/" + out, Buffer.from(webp, "base64"));
  await p.close();
}
await b.close();
