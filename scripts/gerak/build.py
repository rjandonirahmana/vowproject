#!/usr/bin/env python3
"""scripts/gerak/build.py — aset MASK untuk koreografi pembuka (web/gerak/*.css).

Semua SVG berwarna hitam di atas transparan dan dipakai lewat CSS `mask-image`,
jadi warnanya mengikuti token tema (--primary, --foil-deep, --sage, …) — satu
berkas untuk semua tema. Keluaran: public/img/tema/gerak/*.svg

    python3 scripts/gerak/build.py
"""
import math, os, random

OUT = os.path.join(os.path.dirname(__file__), '..', '..', 'public', 'img', 'tema', 'gerak')
os.makedirs(OUT, exist_ok=True)
random.seed(7)


def f(v):
    return f'{v:.1f}'.rstrip('0').rstrip('.')


def save(name, w, h, body, aspect=None):
    par = f' preserveAspectRatio="{aspect}"' if aspect else ''
    svg = f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}"{par}>{body}</svg>'
    with open(os.path.join(OUT, name), 'w') as fh:
        fh.write(svg)
    print(f'{name:28s} {len(svg):6d} B')


def spiral(cx, cy, r, turns=1.6, flip=1, rot=0, n=34):
    """Sulur (spiral) sebagai polyline — motif ukir Jawa."""
    pts = []
    for i in range(n + 1):
        t = i / n
        a = rot + flip * t * turns * 2 * math.pi
        rr = r * (1 - 0.82 * t)
        pts.append(f'{f(cx + rr * math.cos(a))},{f(cy + rr * math.sin(a))}')
    return 'M' + ' L'.join(pts)


def leaf(cx, cy, length, width, ang, vein=True):
    """Daun runcing dua lengkung + tulang daun (dilubangi lewat <mask>)."""
    a = math.radians(ang)
    ca, sa = math.cos(a), math.sin(a)
    def P(x, y):
        return f'{f(cx + x * ca - y * sa)},{f(cy + x * sa + y * ca)}'
    d = f'M{P(0, 0)} Q{P(length * .45, -width)} {P(length, 0)} Q{P(length * .45, width)} {P(0, 0)}Z'
    v = f'M{P(length * .06, 0)} L{P(length * .9, 0)}' if vein else ''
    return d, v


# ── 1. Gebyok: mahkota kusen (atas), ukirannya, tumpal daun pintu, kisi ──────
def gebyok():
    W, H = 400, 200
    # Siluet mahkota: balok atas + lengkung kalamakara bergelombang, bukaan di tengah bawah.
    edge = []
    for i in range(0, 41):
        x = 40 + i * 8
        t = (x - 200) / 160
        y = 112 + 60 * (t * t) - 22 * math.cos(t * math.pi * 3) * (1 - abs(t))
        edge.append((x, min(y, 196)))
    solid = f'M0 0 H{W} V{H} H360 ' + ' '.join(f'L{f(x)} {f(y)}' for x, y in reversed(edge)) + f' L40 {H} H0Z'
    save('gebyok-mahkota.svg', W, H, f'<path d="{solid}"/>')
    # Ukiran: bingkai manik, sulur berpasangan, roset tengah, kuncup di puncak lengkung.
    s = []
    s.append(f'<path d="M10 10 H390 M10 22 H390" stroke="#000" stroke-width="3" fill="none"/>')
    s += [f'<circle cx="{14 + i * 12}" cy="16" r="2.6"/>' for i in range(32)]
    for side in (-1, 1):
        for k in range(4):
            cx = 200 + side * (46 + k * 36)
            cy = 58 + (k % 2) * 16
            s.append(f'<path d="{spiral(cx, cy, 15 - k, 1.7, side, math.pi * (0.5 if side > 0 else 0.5))}" stroke="#000" stroke-width="3.2" fill="none" stroke-linecap="round"/>')
            d, _ = leaf(cx, cy + 8, 22, 6, 90 + side * 35, False)
            s.append(f'<path d="{d}"/>')
        # untaian lengkung mengikuti tepi bukaan
        pts = [(x, y - 10) for x, y in edge if (x - 200) * side > 8]
        s.append('<path d="M' + ' L'.join(f'{f(x)},{f(y)}' for x, y in pts) + '" stroke="#000" stroke-width="3" fill="none"/>')
        s += [f'<circle cx="{f(x)}" cy="{f(y - 20)}" r="2.4"/>' for x, y in pts[::2]]
    # roset tengah
    for i in range(8):
        d, _ = leaf(200, 58, 26, 7, i * 45, False)
        s.append(f'<path d="{d}"/>')
    s.append('<circle cx="200" cy="58" r="9" fill="none" stroke="#000" stroke-width="2.5"/><circle cx="200" cy="58" r="4"/>')
    save('gebyok-mahkota-ukir.svg', W, H, ''.join(s))

    # Tumpal / gunungan untuk daun pintu (segitiga berisi sulur & bunga).
    W, H = 200, 300
    t = []
    t.append('<path d="M100 8 L188 292 H12Z" fill="none" stroke="#000" stroke-width="6"/>')
    t.append('<path d="M100 34 L168 276 H32Z" fill="none" stroke="#000" stroke-width="2.5"/>')
    for i, (cx, cy, r) in enumerate([(100, 120, 20), (78, 190, 18), (122, 190, 18), (64, 252, 14), (100, 248, 16), (136, 252, 14)]):
        s1 = spiral(cx, cy, r, 1.6, 1 if i % 2 else -1, i)
        t.append(f'<path d="{s1}" stroke="#000" stroke-width="3" fill="none" stroke-linecap="round"/>')
    for i in range(6):
        d, _ = leaf(100, 70, 22, 6, -90 + (i - 2.5) * 22, False)
        t.append(f'<path d="{d}"/>')
    t.append('<circle cx="100" cy="66" r="6"/>')
    save('gebyok-tumpal.svg', W, H, ''.join(t))

    # Sulur tegak untuk tiang kusen (diulang vertikal).
    W, H = 60, 120
    v = ['<path d="M8 0 V120 M52 0 V120" stroke="#000" stroke-width="4"/>',
         '<path d="M30 0 C46 20 14 40 30 60 C46 80 14 100 30 120" stroke="#000" stroke-width="3" fill="none"/>']
    for cy, sd in ((30, 1), (90, -1)):
        v.append(f'<path d="{spiral(30 + sd * 9, cy, 9, 1.5, sd, 0)}" stroke="#000" stroke-width="2.6" fill="none"/>')
        d, _ = leaf(30, cy - 14, 13, 4, -90 - sd * 40, False)
        v.append(f'<path d="{d}"/>')
    save('gebyok-tiang.svg', W, H, ''.join(v))

    # Kisi-kisi berlian (diulang) — tekstur daun pintu.
    save('kisi.svg', 28, 28, '<path d="M14 1 L27 14 L14 27 L1 14Z" fill="none" stroke="#000" stroke-width="1.6"/><circle cx="14" cy="14" r="2.4"/>')


# ── 2. Candi bentar (separuh kiri; separuh kanan = dicerminkan CSS) ──────────
def candi():
    W, H = 260, 640
    # Tingkat-tingkat bertangga makin sempit ke atas; sisi dalam (kanan) tegak lurus.
    tiers = [(0, 640, 260), (12, 470, 240), (26, 400, 222), (44, 334, 200), (62, 276, 176), (82, 224, 150),
             (102, 178, 124), (122, 138, 98), (142, 104, 74), (160, 76, 52), (176, 54, 34), (190, 36, 18)]
    d = ''
    for x0, y_top, _ in tiers:
        d += f'M{x0} {y_top} H{W} V{H if x0 == 0 else y_top + 80} H{x0}Z '
    d += f'M206 0 Q214 18 222 36 H{W} V36 H200Z'
    save('candi-bentar.svg', W, H, f'<path d="{d}"/>')
    # Ukiran: garis tingkat, ceplok karang, sulur.
    u = []
    for x0, y_top, _ in tiers[1:]:
        u.append(f'<path d="M{x0 + 4} {y_top + 6} H{W}" stroke="#000" stroke-width="3"/>')
        u.append(f'<path d="M{x0 + 4} {y_top + 14} H{W}" stroke="#000" stroke-width="1.4"/>')
        for k in range(int((W - x0 - 20) / 22)):
            u.append(f'<circle cx="{x0 + 18 + k * 22}" cy="{y_top + 30}" r="4.5" fill="none" stroke="#000" stroke-width="1.6"/>')
    for cy in (540, 600):
        for k in range(6):
            u.append(f'<path d="{spiral(30 + k * 40, cy, 12, 1.4, 1 if k % 2 else -1, k)}" stroke="#000" stroke-width="2.6" fill="none"/>')
    u.append('<path d="M150 470 Q150 430 205 420 Q260 430 260 470 V640 H150Z" fill="none" stroke="#000" stroke-width="4"/>')
    save('candi-ukir.svg', W, H, ''.join(u))


# ── 3. Rimbun dedaunan (tirai taman), dua lapis ──────────────────────────────
def rimbun(name, seed, n, big):
    random.seed(seed)
    W, H = 320, 900
    leaves, veins = [], []
    for i in range(n):
        y = random.uniform(-30, H + 30)
        x = -60 + random.random() ** 1.6 * (150 + 150 * (1 - abs(y / H - .5) * 1.2))
        L = random.uniform(.6, 1) * big
        ang = random.uniform(-70, 40)
        d, v = leaf(x, y, L, L * random.uniform(.24, .34), ang)
        leaves.append(d); veins.append(v)
    # batang menjulur
    stems = ''.join(f'<path d="M-10 {y} Q{100 + k * 30} {y - 60} {170 + k * 20} {y - 140}" stroke="#000" stroke-width="5" fill="none"/>'
                    for k, y in enumerate((160, 420, 700)))
    body = ('<defs><mask id="v"><rect x="-50" y="-50" width="500" height="1000" fill="#fff"/>'
            + ''.join(f'<path d="{v}" stroke="#000" stroke-width="2.4"/>' for v in veins)
            + '</mask></defs><g mask="url(#v)">' + stems + ''.join(f'<path d="{d}"/>' for d in leaves) + '</g>')
    save(name, W, H, body)


# ── 4. Gonjong rumah gadang (atap bertanduk) ─────────────────────────────────
def gonjong():
    W, H = 400, 170
    d = ('M0 168 C40 150 30 60 18 8 C70 52 92 92 112 112 C128 92 150 60 162 30 C178 60 190 84 200 96 '
         'C210 84 222 60 238 30 C250 60 272 92 288 112 C308 92 330 52 382 8 C370 60 360 150 400 168Z')
    save('gonjong.svg', W, H, f'<path d="{d}"/>')


# ── 5. Gunungan / kayon wayang kulit (tatahan tembus = lubang di mask) ───────
def gunungan():
    W, H = 300, 470
    outer = 'M150 8 C232 110 292 236 264 382 L36 382 C8 236 68 110 150 8Z'
    holes = []
    # pita tepi: garis dalam berlubang + deret titik
    holes.append('<path d="M150 30 C220 122 270 236 246 366 L54 366 C30 236 80 122 150 30Z" fill="none" stroke="#000" stroke-width="3.5"/>')
    for i in range(46):
        t = i / 45
        # titik di sepanjang tepi kiri & kanan (aproksimasi kurva Bezier)
        for sgn in (-1, 1):
            u = t
            x = (1-u)**3*150 + 3*(1-u)**2*u*(150+sgn*72) + 3*(1-u)*u*u*(150+sgn*132) + u**3*(150+sgn*106)
            y = (1-u)**3*20 + 3*(1-u)**2*u*116 + 3*(1-u)*u*u*236 + u**3*374
            holes.append(f'<circle cx="{f(x)}" cy="{f(y)}" r="2.2"/>')
    # pohon hayat: cabang-cabang ber-sulur; daun = lubang runcing di antara cabang
    random.seed(5)
    for k in range(7):
        y = 92 + k * 30
        span = 18 + k * 13
        for sgn in (-1, 1):
            holes.append(f'<path d="{spiral(150 + sgn * span * .78, y - 4, 9 + k * .8, 1.5, sgn, math.pi)}" stroke="#000" stroke-width="2.6" fill="none"/>')
            for j in range(2 + k // 2):
                d, _ = leaf(150 + sgn * (10 + j * 15), y + 10, 13 + k, 4.2, (90 - sgn * 60) - sgn * j * 12, False)
                holes.append(f'<path d="{d}"/>')
    # roset (lingkaran + kelopak) di tengah pohon
    holes.append('<circle cx="150" cy="250" r="24" fill="none" stroke="#000" stroke-width="4"/>')
    for i in range(8):
        d, _ = leaf(150, 250, 17, 4.5, i * 45 + 22, False)
        holes.append(f'<path d="{d}"/>')
    # gapura di kaki: pintu kembar + atap bertingkat + dua tiang
    holes.append('<path d="M104 300 H196 L184 288 H116Z"/>')
    holes.append('<rect x="118" y="306" width="28" height="52" rx="3"/><rect x="154" y="306" width="28" height="52" rx="3"/>')
    holes.append('<rect x="92" y="306" width="10" height="58"/><rect x="198" y="306" width="10" height="58"/>')
    for sgn in (-1, 1):
        holes.append(f'<path d="{spiral(150 + sgn * 78, 334, 14, 1.6, sgn, 0)}" stroke="#000" stroke-width="3" fill="none"/>')
    body = ('<defs><mask id="g"><path d="' + outer + '" fill="#fff"/>'
            '<rect x="145" y="378" width="10" height="90" rx="4" fill="#fff"/><path d="M128 392 Q150 384 172 392 L170 400 Q150 393 130 400Z" fill="#fff"/>'
            + ''.join(holes) + '</mask></defs><rect width="300" height="470" mask="url(#g)"/>')
    save('gunungan.svg', W, H, body)


gunungan()
gebyok()
candi()
rimbun('rimbun-1.svg', 3, 78, 210)
rimbun('rimbun-2.svg', 11, 54, 250)
gonjong()
