#!/usr/bin/env python3
"""Ilustrasi tema Sekar Kedhaton (ala undangan Jawa premium everlove, gambar
sendiri) → public/img/tema/sekar/.

  pohon-1.svg, pohon-2.svg, pendopo.svg, awan.svg, pendopo-krem.svg
      versi SEPIA dari ilustrasi wayang (scripts/gerak/wayang.py) — pohon
      bergaya kuas cokelat di atas latar krem, joglo garis taupe.
  lili.svg
      semprotan bunga lili/clematis lembayung + daun + kuncup untuk sudut
      bingkai foto (dipasang kiri-bawah & kanan-atas, dicerminkan CSS).

    python3 scripts/gerak/sekar.py
"""
import math
import os
import random

ROOT = os.path.join(os.path.dirname(__file__), '..', '..')
SRC = os.path.join(ROOT, 'public', 'img', 'tema', 'wayang')
OUT = os.path.join(ROOT, 'public', 'img', 'tema', 'sekar')

# emas wayang → sepia/taupe everlove
SEPIA = {
    '#e3c27a': '#9c8461',
    '#a8792f': '#5f4c34',
    '#f6e2a8': '#c4b08c',
}
KREM = {
    '#e3c27a': '#efe4cd',
    '#f6e2a8': '#ffffff',
    '#a8792f': '#d9c9a8',
}
TAUPE = {
    '#e3c27a': '#8c7b5f',
    '#f6e2a8': '#b9a886',
    '#a8792f': '#6b5b43',
}


def recolor(name, table, out_name=None):
    with open(os.path.join(SRC, name)) as f:
        s = f.read()
    for a, b in table.items():
        s = s.replace(a, b).replace(a.upper(), b)
    with open(os.path.join(OUT, out_name or name), 'w') as f:
        f.write(s)


def petal(length, width):
    """Kelopak runcing menghadap ke atas, pangkal di (0,0)."""
    L, W = length, width
    return (f'M0 0 C{W*0.62:.1f} {-L*0.22:.1f} {W*0.58:.1f} {-L*0.72:.1f} 0 {-L:.1f} '
            f'C{-W*0.58:.1f} {-L*0.72:.1f} {-W*0.62:.1f} {-L*0.22:.1f} 0 0Z')


def leaf(length, width):
    L, W = length, width
    return (f'M0 0 C{W*0.7:.1f} {-L*0.3:.1f} {W*0.5:.1f} {-L*0.8:.1f} 0 {-L:.1f} '
            f'C{-W*0.45:.1f} {-L*0.75:.1f} {-W*0.7:.1f} {-L*0.3:.1f} 0 0Z')


def flower(cx, cy, r, rot, rng):
    """Bunga terbuka 6 kelopak + benang sari."""
    out = [f'<g transform="translate({cx} {cy}) rotate({rot})">']
    n = 6
    for i in range(n):
        a = i * 360 / n + rng.uniform(-6, 6)
        L = r * rng.uniform(0.92, 1.05)
        W = r * 0.62
        out.append(f'<g transform="rotate({a:.1f})">'
                   f'<path d="{petal(L, W)}" fill="url(#pet)" stroke="#9a78ad" stroke-opacity=".35" stroke-width="1"/>'
                   f'<path d="M0 -4 L0 {-L*0.86:.1f}" stroke="#8e64a6" stroke-opacity=".45" stroke-width="1.2" fill="none"/>'
                   f'<path d="M0 {-L*0.2:.1f} Q{W*0.18:.1f} {-L*0.5:.1f} {W*0.08:.1f} {-L*0.78:.1f}" stroke="#8e64a6" stroke-opacity=".25" stroke-width=".8" fill="none"/>'
                   f'<path d="M0 {-L*0.2:.1f} Q{-W*0.18:.1f} {-L*0.5:.1f} {-W*0.08:.1f} {-L*0.78:.1f}" stroke="#8e64a6" stroke-opacity=".25" stroke-width=".8" fill="none"/>'
                   '</g>')
    out.append(f'<circle r="{r*0.2:.1f}" fill="url(#ctr)"/>')
    for i in range(14):
        a = math.radians(i * 360 / 14 + rng.uniform(-8, 8))
        d = r * rng.uniform(0.18, 0.34)
        x, y = math.cos(a) * d, math.sin(a) * d
        out.append(f'<line x1="0" y1="0" x2="{x:.1f}" y2="{y:.1f}" stroke="#e9dca7" stroke-width="1.4"/>'
                   f'<circle cx="{x:.1f}" cy="{y:.1f}" r="2.1" fill="#6b4f7a"/>')
    out.append('</g>')
    return ''.join(out)


def bud(x, y, rot, s):
    return (f'<g transform="translate({x} {y}) rotate({rot}) scale({s})">'
            f'<path d="{petal(34, 18)}" fill="url(#bud)"/>'
            f'<path d="M-7 0 Q0 -14 7 0Z" fill="#6f8c5c"/></g>')


def lili():
    rng = random.Random(7)
    W = H = 320
    defs = (
        '<defs>'
        '<linearGradient id="pet" x1="0" y1="1" x2="0" y2="0">'
        '<stop offset="0" stop-color="#7d5394"/><stop offset=".35" stop-color="#b48fca"/>'
        '<stop offset=".8" stop-color="#e9dcf1"/><stop offset="1" stop-color="#f7f0fa"/></linearGradient>'
        '<radialGradient id="ctr"><stop offset="0" stop-color="#f3e9b8"/><stop offset="1" stop-color="#b9a25a"/></radialGradient>'
        '<linearGradient id="bud" x1="0" y1="1" x2="0" y2="0"><stop offset="0" stop-color="#9c78b3"/>'
        '<stop offset="1" stop-color="#eadcf2"/></linearGradient>'
        '<linearGradient id="lf" x1="0" y1="1" x2="1" y2="0"><stop offset="0" stop-color="#4c6a43"/>'
        '<stop offset="1" stop-color="#93ad79"/></linearGradient>'
        '</defs>'
    )
    g = []
    # Batang melengkung dari sudut kiri-bawah.
    for d in ('M8 312 C60 260 110 210 168 150', 'M8 312 C70 280 150 270 236 236',
              'M40 280 C70 220 80 160 74 96', 'M120 205 C150 190 190 170 214 118'):
        g.append(f'<path d="{d}" stroke="#5f7a4f" stroke-width="3" fill="none" stroke-linecap="round"/>')
    # Daun di sepanjang batang.
    for (x, y, rot, L) in [(60, 262, 40, 70), (92, 236, -55, 62), (140, 270, 70, 58), (190, 252, 115, 54),
                           (58, 190, -30, 58), (78, 132, 35, 50), (170, 160, 60, 46), (30, 300, -10, 44)]:
        g.append(f'<g transform="translate({x} {y}) rotate({rot})"><path d="{leaf(L, L*0.42)}" fill="url(#lf)"/>'
                 f'<path d="M0 -2 L0 {-L*0.9:.0f}" stroke="#3f5a37" stroke-opacity=".5" stroke-width="1"/></g>')
    # Kuncup.
    g.append(bud(76, 98, -8, 1.0))
    g.append(bud(214, 120, 30, 0.85))
    g.append(bud(240, 238, 75, 0.8))
    # Dua bunga terbuka + satu setengah mekar.
    g.append(flower(168, 150, 56, 12, rng))
    g.append(flower(108, 236, 44, -20, rng))
    g.append(flower(236, 236, 30, 40, rng))
    svg = (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}">'
           f'{defs}{"".join(g)}</svg>')
    with open(os.path.join(OUT, 'lili.svg'), 'w') as f:
        f.write(svg)


def main():
    os.makedirs(OUT, exist_ok=True)
    recolor('pohon-1.svg', SEPIA)
    recolor('pohon-2.svg', SEPIA)
    recolor('pendopo.svg', TAUPE)
    recolor('awan.svg', TAUPE)
    recolor('pendopo.svg', KREM, 'pendopo-krem.svg')
    lili()
    print('ok →', os.path.relpath(OUT, ROOT))


if __name__ == '__main__':
    main()
