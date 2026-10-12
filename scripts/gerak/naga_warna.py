#!/usr/bin/env python3
"""Ilustrasi naga BERWARNA untuk sampul gerbang naga-emas (gate__orn):
tubuh meliuk-S dari ruas bersisik (emas, perut krem, sirip merah), dua kaki
bercakar, ekor berapi, kepala (sama dengan pendamping naga-kepala) mengejar
mutiara api. Keluaran: public/img/tema/dunia/naga-warna.svg
    python3 scripts/gerak/naga_warna.py
"""
import math, os, re
R = os.path.join(os.path.dirname(__file__), '..', '..')
W, H = 1300, 360
def f(x): return f'{x:.1f}'
kepala = open(os.path.join(R, 'src/web/pendamping/svg/naga-kepala.svg')).read()
isi_kepala = re.sub(r'^<svg[^>]*>|</svg>\s*$', '', kepala.strip())
isi_kepala = isi_kepala.replace('id="e"', 'id="ke"').replace('url(#e)', 'url(#ke)').replace('id="m"', 'id="km"').replace('url(#m)', 'url(#km)')

def titik(t):
    """Lintasan tubuh: t 0 (ekor, kiri) → 1 (leher, kanan)."""
    x = 70 + t * 820
    y = 190 + 78 * math.sin(t * math.pi * 2.2 + .4) * (0.55 + 0.45 * t)
    return x, y

o = ['<defs>'
     '<linearGradient id="e" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#fff0b8"/><stop offset=".45" stop-color="#e6b440"/><stop offset="1" stop-color="#9a6a12"/></linearGradient>'
     '<linearGradient id="m" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#ff7a52"/><stop offset="1" stop-color="#a3161c"/></linearGradient>'
     '<radialGradient id="mutiara" cx=".38" cy=".35" r=".7"><stop offset="0" stop-color="#ffffff"/><stop offset=".5" stop-color="#ffe7a6"/><stop offset="1" stop-color="#d49a2a"/></radialGradient>'
     '</defs>']
N = 46
seg = []
for k in range(N):
    t = k / (N - 1)
    x, y = titik(t)
    x2, y2 = titik(min(1, t + .01))
    ang = math.degrees(math.atan2(y2 - y, x2 - x))
    r = 14 + 32 * t ** .8
    seg.append((x, y, ang, r, k))
# ekor berapi (di belakang ruas pertama)
x0, y0, a0, _, _ = seg[0]
o.append(f'<g transform="translate({f(x0)} {f(y0)}) rotate({f(a0)})"><path d="M10 0C-20 -30 -50 -46 -80 -44C-58 -30 -50 -16 -52 -4C-66 -14 -80 -12 -92 -2C-78 4 -70 10 -66 18C-82 22 -90 34 -96 50C-62 44 -30 26 10 8Z" fill="url(#m)"/></g>')
# kaki: dipasang pada dua ruas
kaki = {14: 1, 30: 1}
for x, y, ang, r, k in seg:
    g = [f'<g transform="translate({f(x)} {f(y)}) rotate({f(ang)})">']
    if k in kaki:
        g.append(f'<g transform="translate({f(-r*.2)} {f(r*.6)}) rotate(28)"><path d="M0 0C10 4 14 14 12 30L10 46C9 52 2 52 2 46L0 30C-4 18 -8 8 0 0Z" fill="url(#e)" stroke="#8a5a10" stroke-width="1.6"/><path d="M6 46C12 56 20 60 30 58M4 48C4 60 2 66 -4 72M8 44C18 46 28 44 34 38" stroke="#fff3cc" stroke-width="4" fill="none" stroke-linecap="round"/><path d="M-6 6C-16 14 -16 24 -10 30C-10 22 -6 16 2 12Z" fill="#c62828"/></g>')
    if k % 2 == 0:
        g.append(f'<path d="M{f(-r*.9)} {f(-r*.55)}C{f(-r*.5)} {f(-r*1.7)} {f(r*.2)} {f(-r*1.8)} {f(r*.7)} {f(-r*1.35)}C{f(r*.2)} {f(-r*1.1)} {f(0)} {f(-r*.8)} {f(r*.4)} {f(-r*.5)}Z" fill="url(#m)"/>')
    g.append(f'<ellipse cx="0" cy="0" rx="{f(r*1.18)}" ry="{f(r)}" fill="url(#e)" stroke="#8a5a10" stroke-width="1.4"/>')
    g.append(f'<path d="M{f(-r*1.05)} {f(r*.35)}C{f(-r*.4)} {f(r*1.05)} {f(r*.4)} {f(r*1.05)} {f(r*1.05)} {f(r*.35)}C{f(r*.6)} {f(r*.75)} {f(-r*.6)} {f(r*.75)} {f(-r*1.05)} {f(r*.35)}Z" fill="#f7e7b6"/>')
    g.append(f'<path d="M{f(-r*.7)} {f(-r*.15)}q{f(r*.35)} {f(-r*.45)} {f(r*.7)} 0q{f(r*.35)} {f(-r*.45)} {f(r*.7)} 0" stroke="#a87414" stroke-width="1.6" fill="none" opacity=".8"/>')
    g.append('</g>')
    o.append(''.join(g))
# kepala di ujung leher, menghadap kanan mengikuti arah lintasan
xh, yh, ah, rh, _ = seg[-1]
o.append(f'<g transform="translate({f(xh-30)} {f(yh-118)}) rotate({f(ah*.6)} 40 118) scale(1.55)">{isi_kepala}</g>')
# mutiara api di depan mulut
mx, my = xh + 330, yh - 30
o.append(f'<g transform="translate({f(mx)} {f(my)})"><path d="M0 -44C14 -30 30 -30 34 -12C44 -4 40 14 30 22C34 36 18 46 4 40C-10 50 -28 40 -28 26C-42 18 -40 -2 -28 -8C-30 -26 -14 -40 0 -44Z" fill="url(#m)" opacity=".85"/><circle r="24" fill="url(#mutiara)"/><circle cx="-8" cy="-9" r="6" fill="#fff" opacity=".9"/></g>')
svg = f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}">{"".join(o)}</svg>\n'
out = os.path.join(R, 'public/img/tema/dunia/naga-warna.svg')
open(out, 'w').write(svg)
print(out, len(svg))
