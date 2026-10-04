#!/usr/bin/env python3
"""scripts/gerak/wayang.py — ilustrasi tema Gunungan Wayang Kulit (berwarna, prada emas).

Keluaran public/img/tema/wayang/*.svg : wayang ksatria & putri (bertatah tembus,
bertangkai cempurit), pohon emas (dua bentuk), pendopo joglo, awan mega mendung,
sudut bingkai ukir, latar kawung. Gambar karya sendiri (bukan salinan referensi).

    python3 scripts/gerak/wayang.py
"""
import math, os, random

OUT = os.path.join(os.path.dirname(__file__), '..', '..', 'public', 'img', 'tema', 'wayang')
os.makedirs(OUT, exist_ok=True)

GOLD = '#e3c27a'; GOLD_L = '#f6e2a8'; GOLD_D = '#a8792f'; INK = '#2a1a0c'; RED = '#8e2a1f'; RED_D = '#5e1810'; SKIN = '#1b1410'

def f(v): return f'{v:.1f}'.rstrip('0').rstrip('.')
def i0(v): return str(int(round(v)))

def save(name, w, h, body):
    svg = f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}">{body}</svg>'
    open(os.path.join(OUT, name), 'w').write(svg)
    print(f'{name:24s} {len(svg)//1024:4d} KB')

PRADA = (f'<linearGradient id="pr" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="{GOLD_L}"/>'
         f'<stop offset=".5" stop-color="{GOLD}"/><stop offset="1" stop-color="{GOLD_D}"/></linearGradient>')

def spiral(cx, cy, r, turns=1.6, flip=1, rot=0, n=30):
    pts = []
    for i in range(n + 1):
        t = i / n; a = rot + flip * t * turns * 2 * math.pi; rr = r * (1 - .82 * t)
        pts.append(f'{f(cx + rr * math.cos(a))},{f(cy + rr * math.sin(a))}')
    return 'M' + ' L'.join(pts)

def dots_in(poly_fn, n, r, seed):
    """Titik tatahan acak di dalam bentuk (poly_fn(x,y)->bool)."""
    random.seed(seed); out = []; tries = 0
    while len(out) < n and tries < n * 60:
        tries += 1
        x, y = random.uniform(0, 300), random.uniform(0, 640)
        if poly_fn(x, y) and all((x-a)**2 + (y-b)**2 > (r*3.2)**2 for a, b in out):
            out.append((x, y))
    return ''.join(f'<circle cx="{f(x)}" cy="{f(y)}" r="{r}"/>' for x, y in out)

# ── Wayang ──────────────────────────────────────────────────────────────────
def wayang(name, putri=False):
    W, H = 300, 640
    st = f'stroke="{INK}" stroke-width="2.2" stroke-linejoin="round"'
    if putri:
        P = dict(
            face='M152 92 C164 92 172 98 178 106 L212 120 C202 124 194 125 188 126 C190 132 186 136 180 137 C176 146 168 150 158 150 C150 140 146 124 148 106Z',
            hair='<circle cx="118" cy="110" r="24" fill="#1b1410" stroke="#2a1a0c" stroke-width="2.2"/><path d="M118 86 C136 80 150 84 154 96 L148 140 C138 132 128 128 118 134" fill="#1b1410" stroke="#2a1a0c" stroke-width="2.2"/>'
                 '<circle cx="118" cy="110" r="9" fill="url(#pr)" stroke="#2a1a0c" stroke-width="1.5"/><circle cx="104" cy="96" r="5" fill="#f6e2a8"/><circle cx="100" cy="122" r="5" fill="#f6e2a8"/><circle cx="128" cy="90" r="4" fill="#8e2a1f"/>',
            neck='M150 148 L166 148 L168 176 L146 176Z',
            torso='M120 178 C138 172 176 172 200 180 C198 206 190 232 178 260 L140 260 C130 232 122 206 120 178Z',
            arm_f='M194 186 C212 214 226 246 234 280 C228 314 220 344 214 374 L205 371 C210 342 217 314 222 284 C212 254 198 226 186 200Z',
            arm_b='M126 188 C110 218 98 250 94 284 C100 318 108 346 116 380 L124 377 C117 346 111 318 106 286 C112 254 122 226 134 200Z',
            kain='M136 260 L182 260 C192 340 200 450 204 586 L118 586 C120 450 126 340 136 260Z',
            drape='M174 268 C194 330 210 400 218 500 L194 506 C190 420 182 340 168 280Z',
            legs='M118 582 L204 582 L218 596 L110 596Z',
            hands=((214, 374), (116, 380)), knees=None, bk=None, nose_eye=(168, 110))
    else:
        P = dict(
            face='M150 86 C164 86 174 92 182 102 L226 120 C214 125 202 126 194 127 C196 133 192 138 186 139 C182 149 172 156 160 156 C150 146 146 128 148 106Z',
            hair=('<path d="M150 88 C144 60 154 38 176 28 C172 42 176 52 186 58 C172 58 162 68 166 86 C150 60 120 40 96 50 C112 58 118 70 116 84 C102 78 90 86 94 102 C106 98 120 108 128 126 C136 134 144 150 150 158Z" fill="url(#pr)" stroke="#2a1a0c" stroke-width="2.2"/>'
                  '<path d="M104 70 C118 60 138 64 148 80" fill="none" stroke="#2a1a0c" stroke-width="2.2"/><path d="M110 92 C122 84 136 88 144 98" fill="none" stroke="#2a1a0c" stroke-width="2"/>'
                  '<circle cx="170" cy="48" r="5" fill="#8e2a1f" stroke="#2a1a0c" stroke-width="1.5"/><circle cx="126" cy="104" r="7" fill="#8e2a1f" stroke="#f6e2a8" stroke-width="2"/>'),
            neck='M150 154 L168 156 L172 186 L144 186Z',
            torso='M112 190 C138 182 184 182 212 192 C210 228 198 256 182 282 L142 282 C126 254 114 224 112 190Z',
            arm_f='M204 198 C224 228 240 258 248 292 C242 326 232 356 226 386 L216 383 C222 354 230 326 236 296 C226 266 210 240 194 214Z',
            arm_b='M120 200 C102 230 90 262 88 296 C94 330 102 358 110 392 L119 389 C112 358 106 330 100 298 C106 266 118 238 130 214Z',
            kain='M140 280 L184 280 C192 330 196 390 196 460 L124 460 C122 390 128 330 140 280Z',
            drape='M178 286 C208 340 232 396 246 456 L208 464 C202 404 190 346 170 300Z',
            legs=('M134 456 L156 456 C152 500 142 540 130 580 L100 592 L158 592 L154 580 C164 540 170 500 172 456Z '
                  'M168 456 L192 456 C198 500 204 540 208 580 L196 592 L256 592 L228 580 C220 540 210 500 194 456Z'),
            hands=((226, 386), (110, 392)), knees=True,
            bk='M132 272 C98 262 84 296 102 316 C110 300 122 292 136 292Z', nose_eye=(172, 106))
    def in_kain(x, y):
        if putri: return 268 < y < 576 and abs(x - 160) < 20 + (y - 260) * .1
        return 288 < y < 452 and abs(x - 160) < 18 + (y - 280) * .12
    random.seed(7 if putri else 2); holes = []
    while len(holes) < (80 if putri else 60):
        x, y = random.uniform(100, 220), random.uniform(260, 590)
        if in_kain(x, y) and all((x-a)**2 + (y-b)**2 > 49 for a, b in holes): holes.append((x, y))
    hole_svg = ''.join(f'<path d="M{i0(x)} {i0(y-3)} l3 3 l-3 3 l-3 -3z"/>' for x, y in holes)
    ty = 210 if putri else 222
    hole_svg += ''.join(f'<path d="M{128 + i*10} {ty + (i%2)*5} l4 -7 l4 7z"/>' for i in range(8))
    sx = 212 if putri else 222
    hole_svg += f'<path d="{spiral(sx, 420 if putri else 390, 9, 1.5, 1, 0)}" stroke="#000" stroke-width="2.2" fill="none"/>'
    (hfx, hfy), (hbx, hby) = P['hands']
    ex, ey = P['nose_eye']
    out = (f'<defs>{PRADA}<mask id="t"><rect width="{W}" height="{H}" fill="#fff"/>{hole_svg}</mask></defs>'
           f'<path d="M{hfx} {hfy} L{hfx + 16} 636 M{hbx} {hby} L{hbx - 26} 636" stroke="{INK}" stroke-width="2.6" stroke-linecap="round"/>'
           f'<path d="M150 30 C152 200 150 420 152 640" stroke="{INK}" stroke-width="5" stroke-linecap="round"/>'
           f'<path d="M150 30 C152 200 150 420 152 640" stroke="{GOLD_D}" stroke-width="2" stroke-linecap="round"/>'
           '<g mask="url(#t)">'
           + (f'<path d="{P["bk"]}" fill="url(#pr)" {st}/>' if P['bk'] else '') +
           f'<path d="{P["arm_b"]}" fill="url(#pr)" {st}/>'
           f'<path d="{P["kain"]}" fill="{RED}" {st}/>'
           f'<path d="{P["legs"]}" fill="url(#pr)" {st}/>'
           f'<path d="{P["torso"]}" fill="url(#pr)" {st}/>'
           f'<path d="{P["drape"]}" fill="url(#pr)" {st}/>'
           f'<path d="{P["drape"]}" fill="none" stroke="{RED}" stroke-width="1.6" stroke-dasharray="6 4" transform="translate(-3 0)"/>'
           f'<path d="{P["neck"]}" fill="{SKIN}" {st}/>'
           f'<path d="{P["face"]}" fill="{SKIN}" {st}/>'
           f'{P["hair"]}'
           f'<path d="{P["arm_f"]}" fill="url(#pr)" {st}/>'
           '</g>'
           f'<path d="M{ex} {ey} C{ex+6} {ey-4} {ex+14} {ey-3} {ex+18} {ey+2} C{ex+12} {ey+3} {ex+6} {ey+3} {ex} {ey}Z" fill="{GOLD_L}"/>'
           f'<path d="M{ex-8} {ey-8} C{ex+2} {ey-14} {ex+16} {ey-12} {ex+24} {ey-4}" fill="none" stroke="{GOLD}" stroke-width="1.6"/>'
           f'<circle cx="152" cy="{128 if putri else 124}" r="6" fill="{GOLD}" stroke="{INK}" stroke-width="1.6"/>'
           f'<path d="M{122 if putri else 116} {184 if putri else 196} C150 {200 if putri else 212} 178 {200 if putri else 212} {200 if putri else 210} {184 if putri else 196}" fill="none" stroke="{GOLD_L}" stroke-width="3"/>'
           f'<path d="M{122 if putri else 116} {184 if putri else 196} C150 {200 if putri else 212} 178 {200 if putri else 212} {200 if putri else 210} {184 if putri else 196}" fill="none" stroke="{INK}" stroke-width="1" stroke-dasharray="2 4"/>'
           # kelat bahu & gelang
           f'<circle cx="{208 if putri else 218}" cy="{214 if putri else 226}" r="6" fill="{RED}" stroke="{GOLD_L}" stroke-width="2"/>'
           f'<circle cx="{hfx + 6} " cy="{hfy - 30}" r="5" fill="{RED}" stroke="{INK}" stroke-width="1.4"/><circle cx="{hbx - 6}" cy="{hby - 30}" r="5" fill="{RED}" stroke="{INK}" stroke-width="1.4"/>'
           f'<rect x="{hfx - 8}" y="{hfy - 4}" width="12" height="8" rx="4" fill="{SKIN}"/><rect x="{hbx - 4}" y="{hby - 4}" width="12" height="8" rx="4" fill="{SKIN}"/>')
    save(name, W, H, out)

# ── Pohon emas (bonsai / cemara Jawa) ────────────────────────────────────────
def pohon(name, seed, W=420, H=700):
    random.seed(seed)
    branches, pads = [], []
    def grow(x, y, ang, length, width, depth):
        if depth == 0 or length < 14:
            pads.append((x, y, length)); return
        cx = x + math.cos(ang) * length * .5 + random.uniform(-14, 14)
        cy = y + math.sin(ang) * length * .5 + random.uniform(-10, 10)
        nx = x + math.cos(ang) * length; ny = y + math.sin(ang) * length
        branches.append((f'M{f(x)} {f(y)} Q{f(cx)} {f(cy)} {f(nx)} {f(ny)}', width))
        k = 2 if depth > 1 else 3
        for i in range(k):
            grow(nx, ny, ang + random.uniform(-.75, .75) + (i - (k-1)/2) * .55, length * random.uniform(.62, .8), width * .62, depth - 1)
        if depth <= 3 and random.random() < .7:
            pads.append((nx, ny, length * .9))
    # batang meliuk dari kiri bawah ke kanan
    grow(30, H - 10, -1.25, 190, 26, 6)
    leaves = []
    for (x, y, L) in pads:
        rx, ry = 26 + L * .55, 11 + L * .2
        for j in range(int(10 + L * .22)):
            a = random.uniform(0, 2 * math.pi); r = random.random() ** .6
            px, py = x + math.cos(a) * rx * r, y + math.sin(a) * ry * r - ry * .3
            ang = random.uniform(-40, 40) - 90 * (px < x) + 30
            c = random.choice([GOLD, GOLD_L, GOLD_D, GOLD])
            leaves.append(f'<ellipse cx="{i0(px)}" cy="{i0(py)}" rx="{i0(random.uniform(8, 14))}" ry="3" transform="rotate({i0(ang)} {i0(px)} {i0(py)})" fill="{c}"/>')
        leaves.append(f'<ellipse cx="{f(x)}" cy="{f(y + ry*.4)}" rx="{f(rx*.9)}" ry="{f(ry*.35)}" fill="{GOLD_D}" opacity=".35"/>')
    br = ''.join(f'<path d="{d}" stroke="url(#bk)" stroke-width="{f(w)}" stroke-linecap="round" fill="none"/>' for d, w in branches)
    xs = [x for x, _, L in pads] + [30]; ys = [y for _, y, L in pads] + [H]
    x0, x1, y0, y1 = min(xs) - 60, max(xs) + 60, min(ys) - 40, max(ys) + 10
    k = min(W / (x1 - x0), H / (y1 - y0))
    body = (f'<g transform="scale({k:.3f}) translate({-x0:.0f} {-y0:.0f})">'f'<defs><linearGradient id="bk" x1="0" y1="1" x2="1" y2="0"><stop offset="0" stop-color="{GOLD_D}"/><stop offset="1" stop-color="{GOLD}"/></linearGradient></defs>'
            f'{br}{"".join(leaves)}</g>')
    body = body.replace('<g transform', '<defs><linearGradient id="bk" x1="0" y1="1" x2="1" y2="0"><stop offset="0" stop-color="' + GOLD_D + '"/><stop offset="1" stop-color="' + GOLD + '"/></linearGradient></defs><g transform', 1)
    save(name, W, H, body)

# ── Pendopo joglo ───────────────────────────────────────────────────────────
def pendopo():
    W, H = 600, 300
    g = []
    roof = [(240, 360, 40, 112), (150, 450, 112, 168), (60, 540, 168, 206)]
    for i, (x0, x1, y0, y1) in enumerate(roof):
        inset = 60 if i == 0 else 42
        g.append(f'<path d="M{x0 + inset} {y0} H{x1 - inset} L{x1} {y1} H{x0}Z" fill="{GOLD}" fill-opacity="{.18 + i*.05}" stroke="{GOLD}" stroke-width="2.4"/>')
        for k in range(int((x1 - x0) / 16)):
            xx = x0 + 8 + k * 16
            g.append(f'<path d="M{f(xx)} {y1} L{f(300 + (xx - 300) * .82)} {y0 + 4}" stroke="{GOLD}" stroke-width=".8" opacity=".45"/>')
    g.append(f'<path d="M290 40 Q300 20 310 40" fill="none" stroke="{GOLD}" stroke-width="3"/><circle cx="300" cy="22" r="5" fill="{GOLD}"/>')
    for x in (110, 210, 390, 490):
        g.append(f'<rect x="{x - 5}" y="206" width="10" height="70" fill="{GOLD}" fill-opacity=".55"/>')
    g.append(f'<rect x="40" y="276" width="520" height="12" fill="{GOLD}" fill-opacity=".5"/><rect x="20" y="288" width="560" height="10" fill="{GOLD}" fill-opacity=".35"/>')
    g.append(f'<path d="M60 206 H540" stroke="{GOLD_L}" stroke-width="3"/>')
    save('pendopo.svg', W, H, ''.join(g))

# ── Awan mega mendung ───────────────────────────────────────────────────────
def awan():
    W, H = 360, 150
    g = []
    for k, (c, op) in enumerate([(GOLD_D, .9), (GOLD, .9), (GOLD_L, .8), (GOLD, .7)]):
        s = 1 - k * .2; cx, cy = 180, 92 + k * 6
        bumps = [(-120, 0, 40), (-70, -30, 48), (-10, -46, 54), (55, -30, 46), (110, -2, 38)]
        d = f'M{f(cx - 160 * s)} {f(cy + 30 * s)} '
        for bx, by, r in bumps:
            d += f'A{f(r*s)} {f(r*s)} 0 0 1 {f(cx + (bx + r*.9) * s)} {f(cy + (by + 10) * s)} '
        d += f'L{f(cx + 160 * s)} {f(cy + 30 * s)}Z'
        g.append(f'<path d="{d}" fill="none" stroke="{c}" stroke-width="{3.4 - k*.5}" opacity="{op}"/>')
    g.append(f'<path d="{spiral(64, 110, 16, 1.5, 1, 0)}" fill="none" stroke="{GOLD}" stroke-width="2.4"/>')
    g.append(f'<path d="{spiral(300, 112, 14, 1.5, -1, math.pi)}" fill="none" stroke="{GOLD}" stroke-width="2.4"/>')
    save('awan.svg', W, H, ''.join(g))

# ── Sudut bingkai ukir ──────────────────────────────────────────────────────
def sudut():
    W = H = 320
    g = [f'<path d="M14 300 V40 Q14 14 40 14 H300" fill="none" stroke="url(#pr)" stroke-width="6"/>',
         f'<path d="M30 300 V56 Q30 30 56 30 H300" fill="none" stroke="{GOLD}" stroke-width="1.6"/>']
    for i in range(6):
        t = 52 + i * 42
        g.append(f'<circle cx="22" cy="{t}" r="2.6" fill="{GOLD_L}"/><circle cx="{t}" cy="22" r="2.6" fill="{GOLD_L}"/>')
    for (cx, cy, r, fl, rot) in [(70, 70, 34, 1, 0), (130, 52, 20, -1, 1), (52, 130, 20, 1, 2), (180, 46, 14, 1, 0), (46, 180, 14, -1, 3)]:
        g.append(f'<path d="{spiral(cx, cy, r, 1.7, fl, rot)}" fill="none" stroke="url(#pr)" stroke-width="{3 + r/12:.1f}" stroke-linecap="round"/>')
    for (cx, cy, a) in [(104, 92, -20), (92, 104, 110), (156, 70, -10), (70, 156, 100), (210, 40, 0), (40, 210, 90)]:
        g.append(f'<path d="M{cx} {cy} q14 -10 30 0 q-14 10 -30 0z" fill="url(#pr)" stroke="{INK}" stroke-width=".8" transform="rotate({a} {cx} {cy})"/>')
    g.append(f'<circle cx="70" cy="70" r="8" fill="{RED}" stroke="{GOLD_L}" stroke-width="2"/>')
    save('sudut.svg', W, H, f'<defs>{PRADA}</defs>' + ''.join(g))

# ── Latar satu layar: kawung samar + kabut emas di bawah ───────────────────
def latar():
    W, H = 600, 1200
    tile = []
    for (cx, cy) in [(30, 30), (90, 30), (30, 90), (90, 90), (60, 60), (0, 60), (120, 60), (60, 0), (60, 120)]:
        for a in (0, 90):
            tile.append(f'<ellipse cx="{cx}" cy="{cy}" rx="11" ry="25" transform="rotate({a + 45} {cx} {cy})" fill="none" stroke="{GOLD}" stroke-width="1"/>')
        tile.append(f'<circle cx="{cx}" cy="{cy}" r="2" fill="{GOLD}"/>')
    body = (f'<defs><pattern id="k" width="120" height="120" patternUnits="userSpaceOnUse">{"".join(tile)}</pattern>'
            f'<radialGradient id="v" cx=".5" cy=".45" r=".7"><stop offset=".4" stop-color="#fff"/><stop offset="1" stop-color="#fff" stop-opacity=".2"/></radialGradient>'
            f'<mask id="m"><rect width="{W}" height="{H}" fill="url(#v)"/></mask>'
            f'<linearGradient id="g" x1="0" y1="0" x2="0" y2="1"><stop offset=".6" stop-color="{GOLD}" stop-opacity="0"/><stop offset="1" stop-color="{GOLD}" stop-opacity=".16"/></linearGradient></defs>'
            f'<rect width="{W}" height="{H}" fill="url(#k)" opacity=".09" mask="url(#m)"/><rect width="{W}" height="{H}" fill="url(#g)"/>')
    save('latar.svg', W, H, body)

wayang('wayang-ksatria.svg')
wayang('wayang-putri.svg', putri=True)
pohon('pohon-1.svg', 4)
pohon('pohon-2.svg', 19)
pendopo(); awan(); sudut(); latar()
