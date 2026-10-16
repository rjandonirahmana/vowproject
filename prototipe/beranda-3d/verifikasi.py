"""Verifikasi mandiri prototipe — jalankan di mesin Anda:
   pip install playwright axe-playwright-python && playwright install chromium
   python verifikasi.py beranda-3d-prototipe.html
Hasil: tangkapan layar tiap titik gulir (bandingkan dengan STORYBOARD.md), uji a11y, uji gerak-dikurangi."""
import sys, pathlib
from playwright.sync_api import sync_playwright
f = pathlib.Path(sys.argv[1]).resolve().as_uri()
STOPS = [0, 12, 22, 38, 52, 68, 84, 100]          # % gulir, sama dengan STORYBOARD.md
with sync_playwright() as p:
    b = p.chromium.launch()
    for nama, kw in {"hp": dict(viewport={"width": 390, "height": 844}, device_scale_factor=2, is_mobile=True, has_touch=True),
                     "hp-gerak-dikurangi": dict(viewport={"width": 390, "height": 844}, reduced_motion="reduce")}.items():
        pg = b.new_context(**kw).new_page(); pg.goto(f)
        tinggi = pg.evaluate("document.querySelector('.pg').offsetHeight - innerHeight")
        for s in (STOPS if nama == "hp" else [0]):
            pg.evaluate(f"scrollTo(0, {tinggi}*{s}/100)"); pg.wait_for_timeout(250)
            pg.screenshot(path=f"shot-{nama}-{s:03d}.png")
        # Kriteria lulus (angka, bukan opini):
        assert pg.evaluate("document.documentElement.scrollWidth <= innerWidth"), "ada scroll horizontal"
        assert pg.evaluate("[...document.querySelectorAll('a')].every(a=>a.getBoundingClientRect().height>=40||!a.offsetParent)"), "target sentuh < 40px"
        if nama != "hp": assert pg.evaluate("getComputedStyle(document.querySelector('.sc.s1')).display==='none'"), "reduced-motion masih beranimasi"
        print(nama, "OK")
    b.close()
# Lanjutan yang perlu HP/URL nyata: Lighthouse mobile (LCP≤2.5s, CLS≤0.1, INP≤200ms), WebPageTest Moto G, 5 orang uji tugas "buka demo tema".
