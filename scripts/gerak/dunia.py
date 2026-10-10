#!/usr/bin/env python3
"""scripts/gerak/dunia.py — aset tema BUDAYA DUNIA & RELIGI (migrasi 043).

MASK (hitam di atas transparan, dipakai lewat CSS `mask-image` → warnanya
mengikuti token tema): naga, lengkung & kisi Arab, gotik & jendela mawar,
mandala 3 lapis, torii, kisi & dancheong Korea, atap kuil Thai, stupa.
POLA (berwarna, dipakai `image_mode = pola`): satu ubin per tema.
Keluaran: public/img/tema/dunia/*.svg

    python3 scripts/gerak/dunia.py
"""
import math, os

OUT = os.path.join(os.path.dirname(__file__), '..', '..', 'public', 'img', 'tema', 'dunia')
os.makedirs(OUT, exist_ok=True)


def f(v):
    return f'{v:.1f}'.rstrip('0').rstrip('.')


def save(name, w, h, body, aspect=None):
    par = f' preserveAspectRatio="{aspect}"' if aspect else ''
    svg = f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}"{par}>{body}</svg>'
    with open(os.path.join(OUT, name), 'w') as fh:
        fh.write(svg)
    print(f'{name:26s} {len(svg):6d} B')


def poly(pts):
    return 'M' + ' L'.join(f'{f(x)} {f(y)}' for x, y in pts) + 'Z'


def bez(p0, p1, p2, p3, t):
    u = 1 - t
    return (u**3 * p0[0] + 3 * u * u * t * p1[0] + 3 * u * t * t * p2[0] + t**3 * p3[0],
            u**3 * p0[1] + 3 * u * u * t * p1[1] + 3 * u * t * t * p2[1] + t**3 * p3[1])


# ── Naga (Tionghoa) ──────────────────────────────────────────────────────────
# Tubuh = rangkaian kurva Bézier; tebal mengecil ke ekor; sirip punggung,
# sisik (garis lengkung), kaki bercakar, kepala bertanduk & berkumis.
def naga():
    # Tubuh tebal meliuk (S ganda), lebar mengecil ke ekor.
    segs = [((150, 120), (190, 40), (240, 210), (300, 120)),
            ((300, 120), (350, 40), (430, 200), (490, 110)),
            ((490, 110), (530, 60), (560, 80), (578, 64))]
    pts = []
    for s_ in segs:
        for k in range(36):
            pts.append(bez(*s_, k / 36))
    pts.append(segs[-1][3])
    n = len(pts)
    left, right, spines, scales, belly = [], [], [], [], []
    for i, (x, y) in enumerate(pts):
        a_ = pts[max(i - 1, 0)]; b_ = pts[min(i + 1, n - 1)]
        dx, dy = b_[0] - a_[0], b_[1] - a_[1]; L = math.hypot(dx, dy) or 1
        tx, ty, nx, ny = dx / L, dy / L, -dy / L, dx / L
        t = i / (n - 1)
        w = 26 * (1 - t) ** 0.8 + 3          # leher tebal → ekor runcing
        left.append((x + nx * w, y + ny * w)); right.append((x - nx * w, y - ny * w))
        if 3 < i < n - 6 and i % 6 == 0:     # sirip PUNGGUNG (-normal = atas), melengkung ke belakang
            spines.append(poly([(x - nx * w * .85 - tx * 7, y - ny * w * .85 - ty * 7),
                                (x - nx * (w + 18) + tx * 12, y - ny * (w + 18) + ty * 12),
                                (x - nx * w * .85 + tx * 7, y - ny * w * .85 + ty * 7)]))
        if 2 < i < n - 8 and i % 4 == 2:     # sisik: busur di sepanjang tubuh
            for o in (-.35, .35):
                cx_, cy_ = x + nx * w * o, y + ny * w * o
                scales.append(f'M{f(cx_ - ny * 5)} {f(cy_ + nx * 5)} Q{f(cx_ + tx * 7)} {f(cy_ + ty * 7)} {f(cx_ + ny * 5)} {f(cy_ - nx * 5)}')
        if 2 < i < n - 10 and i % 3 == 0:    # garis perut (+normal = bawah)
            belly.append(f'M{f(x + nx * w * .55)} {f(y + ny * w * .55)} l{f(nx * w * .4)} {f(ny * w * .4)}')
    body = poly(left + right[::-1])
    legs = []
    for idx, side in ((14, -1), (36, -1), (64, -1), (84, -1)):  # semua di sisi perut
        x, y = pts[idx]
        a_ = pts[idx - 1]; b_ = pts[idx + 1]
        dx, dy = b_[0] - a_[0], b_[1] - a_[1]; L = math.hypot(dx, dy) or 1
        tx, ty, nx, ny = dx / L, dy / L, -dy / L * side, dx / L * side
        w = 26 * (1 - idx / (n - 1)) ** 0.8
        ex, ey = x - nx * (w + 14) - tx * 10, y - ny * (w + 14) - ty * 10     # siku
        kx, ky = ex - nx * 16 + tx * 14, ey - ny * 16 + ty * 14               # pergelangan
        legs.append(f'M{f(x - nx * w * .6)} {f(y - ny * w * .6)} L{f(ex)} {f(ey)} L{f(kx)} {f(ky)}')
        for c in (-1, 0, 1):                                                  # tiga cakar
            legs.append(f'M{f(kx)} {f(ky)} q{f(-nx * 6 + tx * 6 * c)} {f(-ny * 6 + ty * 6 * c)} {f(-nx * 10 + tx * 9 * c)} {f(-ny * 10 + ty * 9 * c)}')
    # Kepala menghadap kiri: rahang atas & bawah terbuka bergigi, surai api di
    # tengkuk, tanduk bercabang, kumis panjang, mata — dan MUTIARA API di depan
    # mulut (ciri naga Tionghoa).
    hx, hy = pts[0]
    up = (f'M{f(hx + 18)} {f(hy - 26)} C{f(hx - 6)} {f(hy - 44)} {f(hx - 44)} {f(hy - 38)} {f(hx - 62)} {f(hy - 24)} '
          f'L{f(hx - 86)} {f(hy - 20)} L{f(hx - 80)} {f(hy - 8)} L{f(hx - 30)} {f(hy - 6)} C{f(hx - 14)} {f(hy - 4)} {f(hx + 2)} {f(hy + 2)} {f(hx + 18)} {f(hy + 6)}Z')
    lo = (f'M{f(hx + 16)} {f(hy + 10)} C{f(hx - 4)} {f(hy + 6)} {f(hx - 28)} {f(hy + 2)} {f(hx - 40)} {f(hy + 6)} '
          f'L{f(hx - 70)} {f(hy + 20)} L{f(hx - 62)} {f(hy + 28)} C{f(hx - 34)} {f(hy + 30)} {f(hx - 2)} {f(hy + 32)} {f(hx + 20)} {f(hy + 24)}Z')
    teeth = ''.join(f'<path d="M{f(hx - 36 - k * 10)} {f(hy - 6)} l4 7 l4 -7Z M{f(hx - 30 - k * 9)} {f(hy + 5 + k * 3)} l4 -6 l4 6Z"/>' for k in range(3))
    mane = ''.join(f'<path d="M{f(hx + 6 + k * 12)} {f(hy - 28 + k * 2)} q{f(10 + k * 2)} -18 {f(26 + k * 3)} -20 q-8 8 -6 18Z"/>' for k in range(4))
    pearl_x, pearl_y = hx - 118, hy + 4
    flames = ''.join(f'<path d="M{f(pearl_x + 13 * math.cos(math.radians(a)))} {f(pearl_y + 13 * math.sin(math.radians(a)))} '
                     f'q{f(10 * math.cos(math.radians(a + 25)))} {f(10 * math.sin(math.radians(a + 25)))} {f(14 * math.cos(math.radians(a)))} {f(14 * math.sin(math.radians(a)))} '
                     f'q{f(-6 * math.cos(math.radians(a - 30)))} {f(-6 * math.sin(math.radians(a - 30)))} {f(-12 * math.cos(math.radians(a)) + 4)} {f(-12 * math.sin(math.radians(a)))}Z"/>' for a in range(0, 360, 45))
    head = (f'<path d="{up}"/><path d="{lo}"/>' + teeth + mane +
            f'<path d="M{f(hx - 18)} {f(hy - 36)} q-4 -26 14 -46 m-8 22 q14 -2 22 -16 M{f(hx - 2)} {f(hy - 38)} q6 -26 30 -36 m-14 14 q12 4 22 -4" fill="none" stroke="#000" stroke-width="5" stroke-linecap="round"/>'
            f'<path d="M{f(hx - 80)} {f(hy - 18)} q-34 -2 -50 -30 q-6 -12 4 -16 M{f(hx - 66)} {f(hy + 24)} q-26 22 -26 48 q2 10 12 8" fill="none" stroke="#000" stroke-width="3" stroke-linecap="round"/>'
            f'<circle cx="{f(hx - 34)}" cy="{f(hy - 24)}" r="6" fill="#fff"/><circle cx="{f(hx - 36)}" cy="{f(hy - 24)}" r="3"/>'
            f'<circle cx="{f(pearl_x)}" cy="{f(pearl_y)}" r="11"/>' + flames)
    tx_, ty_ = pts[-1]
    tail = poly([(tx_ - 6, ty_ + 4), (tx_ + 26, ty_ - 26), (tx_ + 14, ty_ + 2), (tx_ + 30, ty_ + 18)])
    save('naga.svg', 640, 240,
         f'<path d="{body}"/>' + ''.join(f'<path d="{s_}"/>' for s_ in spines) +
         f'<path d="{tail}"/>' + head +
         f'<path d="{" ".join(legs)}" fill="none" stroke="#000" stroke-width="6" stroke-linecap="round" stroke-linejoin="round"/>' +
         f'<path d="{" ".join(scales)}" fill="none" stroke="#fff" stroke-opacity=".5" stroke-width="1.8"/>' +
         f'<path d="{" ".join(belly)}" fill="none" stroke="#fff" stroke-opacity=".35" stroke-width="1.4"/>')


# ── Arab / Islami ────────────────────────────────────────────────────────────
def arab():
    # Bingkai lengkung ogee: persegi dikurangi bukaan lengkung (evenodd).
    W, H = 400, 640
    o = (f'M0 0H{W}V{H}H0Z M60 {H}V250 C60 170 120 130 200 60 C280 130 340 170 340 250V{H}Z')
    inner = f'M80 {H}V255 C80 185 130 150 200 88 C270 150 320 185 320 255V{H}Z'
    save('lengkung-arab.svg', W, H, f'<path fill-rule="evenodd" d="{o}"/><path fill="none" stroke="#000" stroke-width="5" d="{inner}"/>', 'none')
    # Ubin kisi bintang delapan (dua persegi bertumpuk) — garis.
    s, c, r = 60, 30, 20
    sq1 = poly([(c - r, c - r), (c + r, c - r), (c + r, c + r), (c - r, c + r)])
    sq2 = poly([(c, c - r * 1.414), (c + r * 1.414, c), (c, c + r * 1.414), (c - r * 1.414, c)])
    save('kisi-arab.svg', s, s, f'<g fill="none" stroke="#000" stroke-width="3"><path d="{sq1}"/><path d="{sq2}"/>'
         f'<path d="M0 0L{f(c - r)} {f(c - r)} M{s} 0L{f(c + r)} {f(c - r)} M0 {s}L{f(c - r)} {f(c + r)} M{s} {s}L{f(c + r)} {f(c + r)}"/></g>')
    # Bulan sabit & bintang.
    save('sabit.svg', 100, 100, '<path d="M62 12 A40 40 0 1 0 62 88 A32 32 0 1 1 62 12Z"/>'
         + f'<path d="{poly([(72 + 13 * (1 if k % 2 == 0 else .42) * math.cos(math.radians(-90 + 36 * k)), 44 + 13 * (1 if k % 2 == 0 else .42) * math.sin(math.radians(-90 + 36 * k))) for k in range(10)])}"/>')


# ── Kristiani ────────────────────────────────────────────────────────────────
def katedral():
    W, H = 400, 640
    o = f'M0 0H{W}V{H}H0Z M70 {H}V300 Q70 150 200 60 Q330 150 330 300V{H}Z'
    save('gotik.svg', W, H, f'<path fill-rule="evenodd" d="{o}"/>'
         f'<path fill="none" stroke="#000" stroke-width="6" d="M90 {H}V305 Q90 170 200 86 Q310 170 310 305V{H}"/>', 'none')
    # Jendela mawar: cincin, 12 kelopak, jari-jari, pusat (garis tracery).
    c, R = 100, 96
    g = [f'<circle cx="{c}" cy="{c}" r="{R}" fill="none" stroke="#000" stroke-width="6"/>',
         f'<circle cx="{c}" cy="{c}" r="22" fill="none" stroke="#000" stroke-width="5"/>',
         f'<circle cx="{c}" cy="{c}" r="62" fill="none" stroke="#000" stroke-width="3"/>']
    for k in range(12):
        a = math.radians(k * 30)
        g.append(f'<line x1="{f(c + 22 * math.cos(a))}" y1="{f(c + 22 * math.sin(a))}" x2="{f(c + R * math.cos(a))}" y2="{f(c + R * math.sin(a))}" stroke="#000" stroke-width="3"/>')
        ax = math.radians(k * 30 + 15)
        g.append(f'<circle cx="{f(c + 79 * math.cos(ax))}" cy="{f(c + 79 * math.sin(ax))}" r="11" fill="none" stroke="#000" stroke-width="3"/>')
        g.append(f'<ellipse cx="{f(c + 42 * math.cos(ax))}" cy="{f(c + 42 * math.sin(ax))}" rx="15" ry="6" transform="rotate({k * 30 + 15} {f(c + 42 * math.cos(ax))} {f(c + 42 * math.sin(ax))})" fill="none" stroke="#000" stroke-width="2.5"/>')
    save('mawar-kaca.svg', 200, 200, ''.join(g))
    save('merpati.svg', 60, 40, '<path d="M4 22 Q18 14 28 20 Q34 6 52 2 Q44 14 40 22 Q50 22 58 26 Q44 30 32 28 Q24 36 10 34 Q18 28 16 26Z"/>')


# ── Hindu: mandala 3 lapis ───────────────────────────────────────────────────
def mandala():
    c = 200
    def kelopak(n, r0, r1, w, rot=0):
        out = []
        for k in range(n):
            a = math.radians(rot + k * 360 / n)
            b = math.radians(rot + k * 360 / n + 90)
            x0, y0 = c + r0 * math.cos(a), c + r0 * math.sin(a)
            x1, y1 = c + r1 * math.cos(a), c + r1 * math.sin(a)
            mx, my = (x0 + x1) / 2, (y0 + y1) / 2
            out.append(f'M{f(x0)} {f(y0)} Q{f(mx + w * math.cos(b))} {f(my + w * math.sin(b))} {f(x1)} {f(y1)} Q{f(mx - w * math.cos(b))} {f(my - w * math.sin(b))} {f(x0)} {f(y0)}Z')
        return ' '.join(out)
    def titik(n, r, rr, rot=0):
        return ''.join(f'<circle cx="{f(c + r * math.cos(math.radians(rot + k * 360 / n)))}" cy="{f(c + r * math.sin(math.radians(rot + k * 360 / n)))}" r="{rr}"/>' for k in range(n))
    save('mandala-1.svg', 400, 400, f'<path d="{kelopak(24, 150, 198, 14)}"/>' + titik(48, 140, 3) +
         f'<circle cx="{c}" cy="{c}" r="132" fill="none" stroke="#000" stroke-width="3"/>')
    save('mandala-2.svg', 400, 400, f'<path d="{kelopak(16, 70, 128, 20, 11.25)}"/>' + titik(32, 62, 2.6) +
         f'<circle cx="{c}" cy="{c}" r="56" fill="none" stroke="#000" stroke-width="3"/>')
    save('mandala-3.svg', 400, 400, f'<path d="{kelopak(8, 14, 52, 14)}"/>' + f'<circle cx="{c}" cy="{c}" r="10"/>' + titik(8, 30, 3, 22.5))


# ── Jepang ───────────────────────────────────────────────────────────────────
def jepang():
    save('torii.svg', 400, 300,
         '<path d="M10 40 Q200 0 390 40 L384 62 Q200 30 16 62Z"/>'      # kasagi melengkung
         '<rect x="44" y="70" width="312" height="18" rx="3"/>'          # nuki
         '<rect x="186" y="62" width="28" height="30"/>'                 # gakuzuka
         '<path d="M88 60 H116 L122 300 H82Z M284 60 H312 L318 300 H278Z"/>')  # tiang condong


# ── Korea ────────────────────────────────────────────────────────────────────
def korea():
    # Ubin kisi pintu hanok (ddi-sal: garis rapat atas-bawah + palang).
    save('kisi-korea.svg', 40, 60, '<g fill="none" stroke="#000" stroke-width="2.4">'
         '<path d="M0 1H40 M0 59H40 M1 0V60 M39 0V60 M20 0V60 M0 20H40 M0 40H40"/>'
         '<path d="M10 0V20 M30 0V20 M10 40V60 M30 40V60"/></g>')
    # Pita dancheong: motif bunga & kelopak berulang (diwarnai CSS gradien).
    g = []
    for k in range(4):
        x = 30 + k * 60
        g.append(f'<circle cx="{x}" cy="30" r="13"/>')
        for j in range(8):
            a = math.radians(j * 45)
            g.append(f'<ellipse cx="{f(x + 19 * math.cos(a))}" cy="{f(30 + 19 * math.sin(a))}" rx="6" ry="3.4" transform="rotate({j * 45} {f(x + 19 * math.cos(a))} {f(30 + 19 * math.sin(a))})"/>')
    save('dancheong.svg', 240, 60, '<rect x="0" y="0" width="240" height="5"/><rect x="0" y="55" width="240" height="5"/>' + ''.join(g))


# ── Thailand: atap kuil bertingkat dengan chofa ──────────────────────────────
def thai():
    # Tiga tingkat atap bertumpuk (belakang lebih kecil & tinggi), tiap ujung
    # bawah ber-hang hong (kepala naga melengkung), puncak ber-chofa.
    W, H = 400, 320
    out = []
    for k, (half, top, base) in enumerate(((200, 40, 320), (150, 0, 236), (100, -30, 160))):
        y0 = top + 60
        roof = poly([(200 - half, base), (200, y0), (200 + half, base), (200 + half - 26, base), (200, y0 + 34), (200 - half + 26, base)])
        out.append(f'<path d="{roof}" fill-opacity="{1 - k * .18:.2f}"/>')
        for sgn in (-1, 1):
            x = 200 + sgn * (half - 13)
            out.append(f'<path d="M{f(x)} {base} q{f(sgn * 10)} -16 {f(sgn * 24)} -12 q{f(-sgn * 8)} 4 {f(-sgn * 10)} 12Z"/>')
        out.append(f'<path d="M200 {y0} C206 {y0 - 18} 214 {y0 - 28} 226 {y0 - 34} C218 {y0 - 22} 212 {y0 - 12} 210 {y0 - 2}Z"/>')
    out.append('<path d="M200 140 L150 320 H250Z" fill-opacity=".5"/>')
    save('atap-thai.svg', W, H, ''.join(out))


def stupa():
    save('stupa.svg', 300, 400,
         '<path d="M150 10 L156 60 H144Z"/>'
         '<path d="M136 60 H164 L170 120 H130Z"/>'
         + ''.join(f'<rect x="{150 - 14 - k * 3}" y="{66 + k * 9}" width="{28 + k * 6}" height="5" rx="2"/>' for k in range(6)) +
         '<path d="M70 230 C70 160 230 160 230 230Z"/>'
         '<rect x="56" y="228" width="188" height="16" rx="3"/><rect x="36" y="246" width="228" height="18" rx="3"/>'
         '<rect x="16" y="266" width="268" height="20" rx="3"/><rect x="0" y="288" width="300" height="112"/>')


# ── POLA (berwarna) per tema — ubin kecil berulang ───────────────────────────
def pola():
    # Tionghoa: awan xiangyun + koin.
    save('cina-motif.svg', 120, 90,
         '<g fill="none" stroke="#d4a43a" stroke-width="2" opacity=".55">'
         '<path d="M10 50 q0 -14 14 -14 q4 -12 18 -10 q12 -8 22 4 q14 0 14 14 q-10 8 -22 2 q-10 10 -24 2 q-12 6 -22 2z"/>'
         '<circle cx="92" cy="22" r="10"/><rect x="88" y="18" width="8" height="8"/></g>')
    # Arab: bintang delapan.
    r, c = 14, 30
    star = poly([(c + (r if k % 2 == 0 else r * .62) * math.cos(math.radians(k * 22.5)), c + (r if k % 2 == 0 else r * .62) * math.sin(math.radians(k * 22.5))) for k in range(16)])
    save('arab-motif.svg', 60, 60, f'<path d="{star}" fill="none" stroke="#c9a24a" stroke-width="1.6" opacity=".6"/><circle cx="0" cy="0" r="4" fill="#c9a24a" opacity=".35"/><circle cx="60" cy="60" r="4" fill="#c9a24a" opacity=".35"/>')
    # Kristiani: quatrefoil.
    save('katedral-motif.svg', 70, 70, '<g fill="none" stroke="#b9975b" stroke-width="1.5" opacity=".5">'
         '<circle cx="35" cy="25" r="10"/><circle cx="35" cy="45" r="10"/><circle cx="25" cy="35" r="10"/><circle cx="45" cy="35" r="10"/></g>')
    # Hindu: titik mandala kecil + paisley.
    save('hindu-motif.svg', 80, 80, '<g opacity=".55"><path d="M20 50 C6 40 14 16 32 22 C44 26 40 44 28 42 C22 41 24 34 28 34" fill="none" stroke="#c2410c" stroke-width="1.8"/>'
         + ''.join(f'<circle cx="{f(60 + 9 * math.cos(math.radians(k * 45)))}" cy="{f(22 + 9 * math.sin(math.radians(k * 45)))}" r="2" fill="#d97706"/>' for k in range(8)) + '</g>')
    # Jepang: seigaiha (gelombang setengah lingkaran).
    g = []
    for row in range(3):
        for col in range(3):
            x = col * 40 + (20 if row % 2 else 0); y = row * 20 + 20
            for rr in (18, 12, 6):
                g.append(f'<path d="M{x - rr} {y} A{rr} {rr} 0 0 1 {x + rr} {y}"/>')
    save('jepang-motif.svg', 80, 40, f'<g fill="none" stroke="#c0392b" stroke-width="1.2" opacity=".35">{"".join(g)}</g>')
    # Korea: kisi geometris.
    save('korea-motif.svg', 60, 60, '<g fill="none" stroke="#2f6f62" stroke-width="1.4" opacity=".4"><rect x="10" y="10" width="40" height="40"/><path d="M30 10V50 M10 30H50 M20 10V20 M40 40V50"/></g>')
    # Thailand: kanok (lidah api) sederhana.
    save('thai-motif.svg', 60, 70, '<g fill="none" stroke="#c99b2e" stroke-width="1.6" opacity=".55"><path d="M30 60 C14 46 18 26 30 10 C42 26 46 46 30 60Z"/><path d="M30 46 C24 38 26 30 30 24 C34 30 36 38 30 46Z"/></g>')
    # Vietnam: teratai.
    save('vietnam-motif.svg', 80, 70, '<g fill="none" stroke="#b83b5e" stroke-width="1.5" opacity=".45"><path d="M40 56 C30 46 30 30 40 16 C50 30 50 46 40 56Z"/><path d="M40 56 C24 52 16 40 18 28 C30 32 38 42 40 56Z M40 56 C56 52 64 40 62 28 C50 32 42 42 40 56Z"/></g>')
    # Buddhis: roda dharma kecil.
    sp = ''.join(f'<line x1="30" y1="30" x2="{f(30 + 12 * math.cos(math.radians(k * 45)))}" y2="{f(30 + 12 * math.sin(math.radians(k * 45)))}"/>' for k in range(8))
    save('vihara-motif.svg', 60, 60, f'<g fill="none" stroke="#c98a1a" stroke-width="1.5" opacity=".5"><circle cx="30" cy="30" r="12"/><circle cx="30" cy="30" r="3"/>{sp}</g>')


if __name__ == '__main__':
    naga(); arab(); katedral(); mandala(); jepang(); korea(); thai(); stupa(); pola()
