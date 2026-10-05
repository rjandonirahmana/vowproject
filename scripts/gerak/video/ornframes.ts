import { chromium } from 'playwright-core';
import { mkdirSync, rmSync } from 'fs';
const [kind, outDir, durS, fpsS] = process.argv.slice(2);
const FPS = +fpsS, N = Math.round(+durS * FPS);
rmSync(outDir, { recursive: true, force: true }); mkdirSync(outDir, { recursive: true });
const b = await chromium.launch({ executablePath: process.env.EXE! });
const p = await b.newPage({ viewport: { width: 500, height: 700 } });
await p.goto('http://127.0.0.1:8765/orn.html', { waitUntil: 'networkidle' });
await p.evaluate(k => document.body.dataset.k = k, kind); await p.waitForTimeout(700);
await p.evaluate(() => document.getAnimations().forEach(a => a.pause()));
const el = await p.$('#' + kind);
for (let i = 0; i < N; i++) {
  await p.evaluate(t => document.getAnimations().forEach(a => { a.currentTime = t; }), (i / FPS) * 1000);
  await el!.screenshot({ path: `${outDir}/${String(i).padStart(4, '0')}.png`, omitBackground: true });
}
await b.close(); console.log(kind, 'frames', N);
