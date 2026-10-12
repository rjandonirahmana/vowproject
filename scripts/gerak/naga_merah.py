#!/usr/bin/env python3
"""Naga Tionghoa MERAH (gaya ilustrasi): tubuh meliuk-S dari kurva offset
(tebal berubah), sisik (pola) di-clip ke tubuh, perut krem beruas, duri
punggung, 4 kaki bercakar, ekor berjambul, kepala detail (tanduk emas, surai,
kumis, mulut terbuka). Keluaran:
  public/img/tema/dunia/naga-warna.svg      sampul gerbang naga-emas
  src/web/pendamping/svg/naga-*.svg         ruas naga terbang (pendamping)
    python3 scripts/gerak/naga_merah.py
"""
import math, os
R = os.path.join(os.path.dirname(__file__), '..', '..')
def f(x): return f'{x:.1f}'
DEFS = ('<defs>'
 '<linearGradient id="nb" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#ff6a4d"/><stop offset=".35" stop-color="#e0291f"/><stop offset=".75" stop-color="#a3100f"/><stop offset="1" stop-color="#6e0707"/></linearGradient>'
 '<linearGradient id="nh" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#ff7a5a"/><stop offset=".5" stop-color="#d9241a"/><stop offset="1" stop-color="#8a0c0c"/></linearGradient>'
 '<linearGradient id="ne" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#fff1c4"/><stop offset=".5" stop-color="#e3b65a"/><stop offset="1" stop-color="#9a6a1c"/></linearGradient>'
 '<linearGradient id="np" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#fff6e2"/><stop offset="1" stop-color="#e8cfa0"/></linearGradient>'
 '<pattern id="sisik" width="16" height="12" patternUnits="userSpaceOnUse"><path d="M0 12a8 8 0 0 1 16 0M-8 6a8 8 0 0 1 16 0M8 6a8 8 0 0 1 16 0" fill="none" stroke="#7a0909" stroke-width="1.3" opacity=".55"/><path d="M2 11a6 5 0 0 1 12 0" fill="none" stroke="#ff8a6a" stroke-width=".9" opacity=".45"/></pattern>'
 '</defs>')

def kepala_merah():
    """Kepala menghadap +x, pangkal leher di (0,0), ±260×200 — meniru naga
    Tionghoa klasik: moncong panjang ujung hidung terangkat, dahi menonjol,
    mulut terbuka lebar bertaring, surai api berlapis tersibak ke belakang,
    sirip pipi berduri, dua tanduk beruas gading-emas, kumis panjang meliuk."""
    o = []
    import random
    rnd = random.Random(5)
    # ── surai api: lapis belakang (gelap) lalu depan (terang), tersibak ke kiri
    def helai(x0, y0, panjang, sudut, lebar, warna):
        a = math.radians(sudut)
        ex, ey = x0 + math.cos(a) * panjang, y0 + math.sin(a) * panjang
        cx1, cy1 = x0 + math.cos(a - .5) * panjang * .45, y0 + math.sin(a - .5) * panjang * .45
        cx2, cy2 = x0 + math.cos(a + .35) * panjang * .7, y0 + math.sin(a + .35) * panjang * .7
        nx, ny = -math.sin(a) * lebar, math.cos(a) * lebar
        return (f'<path d="M{f(x0 + nx)} {f(y0 + ny)}C{f(cx1 + nx)} {f(cy1 + ny)} {f(cx2)} {f(cy2)} {f(ex)} {f(ey)}'
                f'C{f(cx2 - nx * .3)} {f(cy2 - ny * .3)} {f(cx1 - nx)} {f(cy1 - ny)} {f(x0 - nx)} {f(y0 - ny)}Z" fill="{warna}"/>')
    # Surai LEBAT lebih besar dari kepala: 3 lapis (gelap → terang), helai
    # panjang tersibak ke belakang dan naik-turun seperti api.
    for lapis, warna in ((0, '#7a0808'), (1, '#c81e14'), (2, 'url(#nh)')):
        for k in range(18):
            sudut = 128 + k * 6.2 + rnd.uniform(-6, 6) + lapis * 2
            x0 = 44 + rnd.uniform(-10, 10) - lapis * 10
            y0 = -56 + k * 6 + rnd.uniform(-5, 5)
            o.append(helai(x0, y0, 120 + rnd.uniform(0, 60) - lapis * 22, sudut, 12 - lapis * 2.5, warna))
    # jambul api di atas kepala (dahi → belakang)
    for k in range(7):
        x0 = 120 - k * 14
        o.append(helai(x0, -66 + k * 2, 46 + k * 6, 200 + k * 4, 6, 'url(#nh)'))
    # ── tanduk beruas (dua): PANJANG & ramping, melengkung jauh ke belakang
    def tanduk(dx, dy, w, k_tebal):
        x0, y0 = 92 + dx, -56 + dy
        pts = []
        for i in range(21):
            t = i / 20
            x = x0 - 230 * t
            y = y0 - 70 * t - 40 * math.sin(math.pi * t * .9) + 18 * t * t
            pts.append((x, y, (1 - t) * k_tebal + 1.2))
        atas = ' L'.join(f'{f(x)} {f(y - w_)}' for x, y, w_ in pts)
        bawah = ' L'.join(f'{f(x)} {f(y + w_)}' for x, y, w_ in reversed(pts))
        q = [f'<path d="M{atas} L{bawah}Z" fill="{w}" stroke="#7a5a2a" stroke-width="1.1"/>']
        for i in range(3, 20, 2):
            x, y, w_ = pts[i]
            q.append(f'<path d="M{f(x - 2)} {f(y - w_)}L{f(x + 2)} {f(y + w_)}" stroke="#8a6a32" stroke-width="1.3"/>')
        x, y, _ = pts[8]
        q.append(f'<path d="M{f(x)} {f(y - 4)}C{f(x - 10)} {f(y - 26)} {f(x - 6)} {f(y - 44)} {f(x + 6)} {f(y - 58)}C{f(x + 4)} {f(y - 40)} {f(x + 8)} {f(y - 24)} {f(x + 10)} {f(y - 6)}Z" fill="{w}" stroke="#7a5a2a" stroke-width="1"/>')
        return ''.join(q)
    o.append(tanduk(14, -6, '#b98e4a', 7))
    o.append(tanduk(0, 0, 'url(#ne)', 8))
    # ── rahang bawah terbuka (di belakang kepala atas)
    o.append('<path d="M70 18C110 30 160 44 214 70C222 74 222 84 212 86C176 84 136 74 100 62C78 54 58 44 48 30Z" fill="url(#nh)" stroke="#5a0606" stroke-width="1.6"/>')
    o.append('<path d="M96 30L212 74L204 54C170 40 130 30 100 22Z" fill="#4a0404"/>')
    o.append('<path d="M110 34C140 42 170 52 196 64C178 66 150 58 124 48Z" fill="#e5525a"/>')
    o.append('<path d="M150 50l4 -12 5 13M172 60l4 -12 5 13M194 68l3 -11 5 13M128 42l4 -11 5 12" fill="#fff8e6" stroke="#c9b48a" stroke-width=".6"/>')
    # ── kepala atas: dahi menonjol, moncong panjang, ujung hidung terangkat
    o.append('<path d="M0 -24C14 -50 46 -66 80 -62C96 -78 124 -80 140 -64C166 -60 192 -52 214 -44C232 -40 246 -46 252 -58C262 -50 262 -32 250 -22C244 -16 236 -12 226 -12'
             'L200 -6C176 0 150 8 128 16L84 24C52 28 22 18 0 4Z" fill="url(#nh)" stroke="#5a0606" stroke-width="1.8"/>')
    o.append('<path d="M148 -58C172 -54 196 -48 214 -42M156 -46q8 -6 16 0q8 -6 16 0q8 -6 16 0" stroke="#7a0909" stroke-width="1.6" fill="none" opacity=".7"/>')
    o.append('<path d="M244 -50c10 -6 18 2 12 10c-6 6 -14 0 -9 -6" stroke="#5a0606" stroke-width="2.4" fill="none"/>')
    # gigi atas & taring
    o.append('<path d="M120 16l5 13 5 -12M146 8l5 14 5 -13M170 2l5 13 5 -12M196 -4l5 15 6 -14M214 -9l4 11 4 -11" fill="#fff8e6" stroke="#c9b48a" stroke-width=".6"/>')
    # ── alis berkobar + mata almond emas
    o.append('<path d="M96 -64C108 -86 134 -90 152 -80C140 -78 130 -72 124 -64C138 -70 150 -68 160 -60C146 -58 128 -54 112 -50Z" fill="#7a0909"/>')
    o.append('<path d="M106 -54C116 -64 136 -64 146 -52C136 -46 118 -46 106 -54Z" fill="#ffd54a" stroke="#5a0606" stroke-width="2"/><ellipse cx="127" cy="-54" rx="3.6" ry="6.5" fill="#1a0505"/><circle cx="122" cy="-57" r="1.8" fill="#fff"/>')
    # sisik dahi & pipi
    o.append('<path d="M30 -30q10 -8 20 0q10 -8 20 0q10 -8 20 0M24 -10q10 -8 20 0q10 -8 20 0q10 -8 20 0M40 8q10 -8 20 0q10 -8 20 0" fill="none" stroke="#7a0909" stroke-width="1.4" opacity=".55"/>')
    # ── sirip pipi berduri (menyapu ke belakang)
    o.append('<path d="M74 22L40 54L64 40L44 76L74 48L66 84L92 46Z" fill="#ff5a36" stroke="#8a0c0c" stroke-width="1"/>')
    # ── janggut di dagu
    o.append('<path d="M150 70C142 92 126 108 104 118C128 118 148 104 160 82ZM178 80C176 100 166 116 150 128C174 124 188 108 192 86Z" fill="url(#nh)" stroke="#6e0707" stroke-width="1"/>')
    # ── kumis panjang meliuk dari bibir atas, tendril hidung
    o.append('<g fill="none" stroke="#f3d27a" stroke-width="3" stroke-linecap="round">'
             '<path d="M232 -14C214 18 170 30 130 22C96 16 70 34 66 64C64 84 50 100 30 104"/>'
             '<path d="M222 -10C232 22 222 56 194 74C172 88 168 112 184 130"/>'
             '<path d="M250 -40C268 -60 286 -56 290 -40C292 -30 284 -24 276 -28"/></g>')
    return ''.join(o)

def tubuh(n_titik, spine, lebar):
    """Kembalikan (path tubuh, list (titik, normal, lebar))."""
    pts = []
    for k in range(n_titik):
        t = k / (n_titik - 1)
        x, y = spine(t)
        x2, y2 = spine(min(1, t + .004)); x1, y1 = spine(max(0, t - .004))
        dx, dy = x2 - x1, y2 - y1; L = math.hypot(dx, dy) or 1
        nx, ny = dy / L, -dx / L   # normal ke ATAS layar (punggung)
        pts.append((t, x, y, nx, ny, lebar(t), math.degrees(math.atan2(dy, dx))))
    atas = [(x + nx * w, y + ny * w) for (t, x, y, nx, ny, w, a) in pts]
    bawah = [(x - nx * w, y - ny * w) for (t, x, y, nx, ny, w, a) in pts]
    d = 'M' + ' L'.join(f'{f(a)} {f(b)}' for a, b in atas) + ' L' + ' L'.join(f'{f(a)} {f(b)}' for a, b in reversed(bawah)) + 'Z'
    return d, pts

def kaki(x, y, ang, s=1.0, gelap=False):
    w = '#8a0c0c' if gelap else 'url(#nh)'
    return (f'<g transform="translate({f(x)} {f(y)}) rotate({f(ang)}) scale({s})">'
            f'<path d="M-14 -6C-4 10 6 24 4 40C2 52 10 60 22 62L30 64C34 66 32 72 26 70L12 68C-2 66 -10 56 -10 42C-10 28 -22 12 -30 0Z" fill="{w}" stroke="#5a0606" stroke-width="1.3"/>'
            f'<path d="M-24 2C-34 10 -36 22 -30 30C-28 20 -22 14 -14 10Z" fill="#ff6a4d" opacity=".9"/>'
            f'<g fill="url(#ne)" stroke="#7a5212" stroke-width=".8"><path d="M26 62q14 -2 18 8q-10 -4 -18 0Z"/><path d="M22 66q10 6 8 18q-4 -10 -12 -14Z"/><path d="M14 66q2 12 -6 18q2 -10 0 -16Z"/></g></g>')

def naga_sampul():
    W, H = 1300, 440
    spine = lambda t: (110 + t * 860, 250 + 95 * math.sin(t * math.pi * 2.3 + .2) * (.55 + .45 * t))
    lebar = lambda t: (9 + 36 * min(1, t / .55) ** .8) if t < .55 else (45 - 12 * (t - .55) / .45)
    d, pts = tubuh(140, spine, lebar)
    o = [DEFS]
    # kaki jauh (di belakang tubuh, lebih gelap)
    for tk in (.3, .78):
        p = pts[int(tk * 139)]; o.append(kaki(p[1] - p[3] * p[5] * .4, p[2] - p[4] * p[5] * .4, p[6] + 10, .9, True))
    # ekor berjambul
    t0 = pts[0]
    o.append(f'<g transform="translate({f(t0[1])} {f(t0[2])}) rotate({f(t0[6])})"><path d="M8 0C-10 -26 -40 -44 -74 -46C-50 -30 -42 -16 -42 -4C-58 -14 -76 -14 -92 -4C-74 4 -64 12 -58 22C-76 26 -86 38 -92 54C-60 46 -30 28 8 6Z" fill="url(#nh)" stroke="#6e0707" stroke-width="1.2"/><path d="M-20 -22C-40 -28 -58 -28 -72 -22M-24 4C-44 6 -60 14 -72 26" stroke="#7a0909" stroke-width="2" fill="none" opacity=".6"/></g>')
    # duri punggung
    for k in range(4, 139, 4):
        t, x, y, nx, ny, w, a = pts[k]
        tx, ty = x + nx * w, y + ny * w
        tinggi = 10 + w * .55
        bx, by = -math.cos(math.radians(a)), -math.sin(math.radians(a))
        o.append(f'<path d="M{f(tx - bx * 6)} {f(ty - by * 6)}L{f(tx + nx * tinggi + bx * tinggi * .55)} {f(ty + ny * tinggi + by * tinggi * .55)}L{f(tx + bx * 8)} {f(ty + by * 8)}Z" fill="#ff5a36" stroke="#8a0c0c" stroke-width="1"/>')
    # tubuh + sisik + perut
    o.append(f'<clipPath id="tb"><path d="{d}"/></clipPath><path d="{d}" fill="url(#nb)"/><rect x="0" y="0" width="{W}" height="{H}" fill="url(#sisik)" clip-path="url(#tb)"/>')
    perut_a = [(x - nx * w * .98, y - ny * w * .98) for (t, x, y, nx, ny, w, a) in pts]
    perut_b = [(x - nx * w * .42, y - ny * w * .42) for (t, x, y, nx, ny, w, a) in pts]
    pd = 'M' + ' L'.join(f'{f(a)} {f(b)}' for a, b in perut_a) + ' L' + ' L'.join(f'{f(a)} {f(b)}' for a, b in reversed(perut_b)) + 'Z'
    o.append(f'<path d="{pd}" fill="url(#np)"/>')
    for k in range(3, 139, 3):
        (a1, b1), (a2, b2) = perut_a[k], perut_b[k]
        o.append(f'<path d="M{f(a1)} {f(b1)}L{f(a2)} {f(b2)}" stroke="#c9a46a" stroke-width="1.4"/>')
    o.append(f'<path d="{d}" fill="none" stroke="#5a0606" stroke-width="1.6"/>')
    # kaki dekat
    for tk in (.22, .7):
        p = pts[int(tk * 139)]; o.append(kaki(p[1] - p[3] * p[5] * .55, p[2] - p[4] * p[5] * .55, p[6], 1.0))
    # kepala di ujung leher
    th = pts[-1]
    o.append(f'<g transform="translate({f(th[1] - 6)} {f(th[2])}) rotate({f(th[6] * .5 - 6)}) scale(1.05)">{kepala_merah()}</g>')
    svg = f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}">{"".join(o)}</svg>\n'
    open(os.path.join(R, 'public/img/tema/dunia/naga-warna.svg'), 'w').write(svg)
    return len(svg)

def naga_pendamping():
    P = os.path.join(R, 'src/web/pendamping/svg')
    open(os.path.join(P, 'naga-kepala.svg'), 'w').write(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="-200 -180 500 330">{DEFS}{kepala_merah()}</svg>\n')
    ruas = (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 64">{DEFS}'
            '<path d="M14 22L26 4L34 20L46 2L56 20L68 4L76 22L86 10L90 26Z" fill="#ff5a36" stroke="#8a0c0c" stroke-width="1"/>'
            '<clipPath id="r"><ellipse cx="50" cy="36" rx="49" ry="21"/></clipPath><ellipse cx="50" cy="36" rx="49" ry="21" fill="url(#nb)"/><rect width="100" height="64" fill="url(#sisik)" clip-path="url(#r)"/>'
            '<path d="M6 42C28 58 72 58 94 42C90 52 72 60 50 60C28 60 10 52 6 42Z" fill="url(#np)"/>'
            '<path d="M22 52v6M36 55v5M50 56v4M64 55v5M78 52v6" stroke="#c9a46a" stroke-width="1.4"/>'
            '<ellipse cx="50" cy="36" rx="49" ry="21" fill="none" stroke="#5a0606" stroke-width="1.4"/></svg>\n')
    open(os.path.join(P, 'naga-badan.svg'), 'w').write(ruas)
    open(os.path.join(P, 'naga-ekor.svg'), 'w').write(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 64">{DEFS}<path d="M40 32C26 14 12 4 0 2C10 12 14 20 14 28C6 22 2 24 0 30C8 34 10 38 14 40C6 44 2 50 0 60C12 56 26 46 40 38Z" fill="url(#nh)" stroke="#6e0707" stroke-width="1"/><path d="M100 22C80 20 60 24 40 30C34 32 34 38 40 40C60 46 80 48 100 46Z" fill="url(#nb)" stroke="#5a0606" stroke-width="1.2"/><path d="M96 46C80 48 60 46 42 40C60 50 80 54 96 52Z" fill="url(#np)"/></svg>\n')
    open(os.path.join(P, 'naga-kaki.svg'), 'w').write(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="-40 -10 90 100">{DEFS}{kaki(0, 0, 0, 1.0)[len("<g transform=\"translate(0.0 0.0) rotate(0.0) scale(1.0)\">"):-4]}</svg>\n')

if __name__ == '__main__':
    n = naga_sampul(); naga_pendamping(); print('ok', n)
