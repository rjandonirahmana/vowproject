// Render HTML beranimasi → frame JPEG (waktu dikendalikan lewat Web Animations).
import { chromium } from 'playwright-core';
import { mkdirSync, rmSync } from 'fs';
const [page, outDir, durS, fpsS, w, h] = process.argv.slice(2);
const FPS = +fpsS, N = Math.round(+durS * FPS);
rmSync(outDir, { recursive: true, force: true }); mkdirSync(outDir, { recursive: true });
const b = await chromium.launch({ executablePath: process.env.EXE! });
const p = await b.newPage({ viewport: { width: +w, height: +h } });
await p.goto('http://127.0.0.1:8765/' + page, { waitUntil: 'networkidle' }); await p.waitForTimeout(900);
await p.evaluate(() => document.getAnimations().forEach(a => a.pause()));
const el = await p.$('#v');
for (let i = 0; i < N; i++) {
  await p.evaluate(t => document.getAnimations().forEach(a => { a.currentTime = t; }), (i / FPS) * 1000);
  await el!.screenshot({ path: `${outDir}/${String(i).padStart(4, '0')}.jpg`, type: 'jpeg', quality: 90 });
}
await b.close(); console.log('frames', N);
