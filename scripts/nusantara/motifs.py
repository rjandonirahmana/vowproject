# Pustaka motif SVG Nusantara (stilisasi geometris, bukan replika persis).
# Semua fungsi menerima palet `c` (dict hex) dan mengembalikan string SVG.
import math

def svg(w, h, body, defs=""):
    d = f"<defs>{defs}</defs>" if defs else ""
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}">{d}{body}</svg>'

def f(x):
    return f"{x:.1f}".rstrip("0").rstrip(".")

# ── Ubin pola (image_mode "pola") ─────────────────────────────────────────

def kawung(c):
    g, p = c["gold"], c["primary"]
    b = ""
    for cx, cy in [(30, 0), (30, 60), (0, 30), (60, 30)]:
        pass
    b += f'<g fill="none" stroke="{g}" stroke-width="2">'
    b += '<ellipse cx="30" cy="14" rx="9" ry="14"/><ellipse cx="30" cy="46" rx="9" ry="14"/>'
    b += '<ellipse cx="14" cy="30" rx="14" ry="9"/><ellipse cx="46" cy="30" rx="14" ry="9"/></g>'
    b += f'<circle cx="30" cy="30" r="3" fill="{p}"/>'
    for x, y in [(0, 0), (60, 0), (0, 60), (60, 60)]:
        b += f'<circle cx="{x}" cy="{y}" r="4" fill="{g}"/>'
    return svg(60, 60, b)

def parang(c):
    g, p = c["gold"], c["primary"]
    b = f'<g fill="none" stroke="{g}" stroke-width="5" stroke-linecap="round">'
    for o in (-40, 0, 40):
        b += f'<path d="M{o} 80 C{o+10} 60 {o+30} 60 {o+40} 40 S{o+70} 20 {o+80} 0"/>'
    b += "</g>"
    b += f'<g fill="{p}" opacity=".55">'
    for o in (-20, 20, 60):
        b += f'<circle cx="{o+20}" cy="{40-(o+20)/2+20}" r="3"/>'
    b += "</g>"
    return svg(80, 80, b)

def truntum(c):
    g = c["gold"]
    def star(x, y, r):
        s = f'<g transform="translate({x} {y})" fill="none" stroke="{g}" stroke-width="1.6">'
        for i in range(8):
            a = i * math.pi / 4
            s += f'<line x1="0" y1="0" x2="{f(math.cos(a)*r)}" y2="{f(math.sin(a)*r)}"/>'
        s += f'<circle r="{f(r*0.35)}" fill="{g}"/></g>'
        return s
    return svg(70, 70, star(17, 17, 10) + star(52, 52, 10) + star(52, 17, 4) + star(17, 52, 4))

def sidomukti(c):
    g, p = c["gold"], c["primary"]
    b = f'<path d="M40 0 L80 40 L40 80 L0 40 Z" fill="none" stroke="{g}" stroke-width="1.6"/>'
    b += f'<path d="M40 22 L58 40 L40 58 L22 40 Z" fill="none" stroke="{p}" stroke-width="1.2" opacity=".6"/>'
    b += f'<g fill="{g}">' + "".join(f'<ellipse cx="40" cy="40" rx="3" ry="9" transform="rotate({a} 40 40)"/>' for a in (0, 45, 90, 135)) + "</g>"
    return svg(80, 80, b)

def lurik(c):
    g, p = c["gold"], c["primary"]
    b = f'<rect x="0" y="0" width="6" height="40" fill="{p}" opacity=".5"/><rect x="10" y="0" width="2" height="40" fill="{g}"/>'
    b += f'<rect x="16" y="0" width="2" height="40" fill="{g}"/><rect x="26" y="0" width="10" height="40" fill="{p}" opacity=".25"/>'
    return svg(40, 40, b)

def mega_mendung(c):
    p, g = c["primary"], c["gold"]
    b = ""
    for i, (x, y, s) in enumerate([(0, 20, 1), (60, 70, 0.8)]):
        b += f'<g transform="translate({x} {y}) scale({s})" fill="none" stroke-linecap="round">'
        for k, w in enumerate((10, 7, 4)):
            col = p if k % 2 == 0 else g
            r = 30 - k * 7
            b += f'<path d="M0 30 Q10 {30-r} 30 {30-r*0.9} Q42 {8-k*2} 56 {14+k*4} Q72 {4+k*5} 84 {22+k*2} Q100 {30-r*0.4} 104 30" stroke="{col}" stroke-width="{w/2.2:.1f}"/>'
        b += "</g>"
    return svg(120, 120, b)

def kujang(c):
    g, p = c["gold"], c["primary"]
    b = f'<g transform="translate(30 6)"><path d="M10 0 C24 18 20 34 12 46 C16 54 14 62 8 70 L4 70 C8 60 6 54 2 48 C-2 40 2 30 8 26 C4 18 4 8 10 0 Z" fill="{g}" opacity=".9"/>'
    b += f'<circle cx="11" cy="20" r="2" fill="{c["card"]}"/><circle cx="9" cy="34" r="2" fill="{c["card"]}"/><rect x="2" y="70" width="10" height="16" rx="3" fill="{p}"/></g>'
    return svg(90, 100, b)

def bambu(c):
    p, g = c["primary"], c["gold"]
    b = ""
    for x in (12, 42, 72):
        b += f'<rect x="{x}" y="0" width="12" height="100" rx="5" fill="{p}" opacity=".35"/>'
        for y in (20, 55, 88):
            b += f'<rect x="{x-1}" y="{y}" width="14" height="3" rx="1.5" fill="{g}"/>'
    b += f'<path d="M24 30 q18 -10 30 -2" stroke="{g}" stroke-width="2" fill="none"/>'
    return svg(96, 100, b)

def pucuk_rebung(c):
    g, p = c["gold"], c["primary"]
    b = ""
    for x in (0, 40):
        b += f'<path d="M{x} 80 L{x+20} 0 L{x+40} 80 Z" fill="none" stroke="{g}" stroke-width="2"/>'
        b += f'<path d="M{x+8} 80 L{x+20} 30 L{x+32} 80 Z" fill="{p}" opacity=".35"/>'
        b += f'<path d="M{x+20} 40 l-5 14 h10 z" fill="{g}"/>'
    return svg(80, 80, b)

def itiak(c):
    g, p = c["gold"], c["primary"]
    b = f'<path d="M0 30 L15 15 L30 30 L45 15 L60 30 L75 15 L90 30" fill="none" stroke="{g}" stroke-width="3"/>'
    b += f'<path d="M0 60 L15 45 L30 60 L45 45 L60 60 L75 45 L90 60" fill="none" stroke="{p}" stroke-width="2" opacity=".6"/>'
    for x in (15, 45, 75):
        b += f'<circle cx="{x}" cy="22" r="3" fill="{g}"/>'
    return svg(90, 75, b)

def kaluak_paku(c):
    g, p = c["gold"], c["primary"]
    def spiral(x, y, s, col):
        pts = []
        for i in range(60):
            a = i * 0.32
            r = s * (1 - i / 60)
            pts.append(f"{f(x+math.cos(a)*r)} {f(y+math.sin(a)*r)}")
        return f'<path d="M{" L".join(pts)}" fill="none" stroke="{col}" stroke-width="2"/>'
    return svg(90, 90, spiral(25, 25, 20, g) + spiral(65, 65, 20, g) + spiral(65, 25, 12, p) + spiral(25, 65, 12, p))

def gorga(c):
    r, w, k = c.get("accent", "#b3261e"), c["card"], c["ink"]
    g = c["gold"]
    def scroll(x, y, flip):
        s = -1 if flip else 1
        return (f'<path d="M{x} {y} c{12*s} -14 {28*s} -6 {26*s} 6 c{-2*s} 10 {-14*s} 10 {-14*s} 2 c0 -5 {6*s} -6 {8*s} -2" '
                f'fill="none" stroke="{r}" stroke-width="4" stroke-linecap="round"/>')
    b = scroll(10, 30, False) + scroll(90, 30, True) + scroll(10, 75, False) + scroll(90, 75, True)
    b += f'<path d="M50 5 L60 25 L50 45 L40 25 Z" fill="{g}" opacity=".85"/><path d="M50 55 L60 75 L50 95 L40 75 Z" fill="{k}" opacity=".7"/>'
    return svg(100, 100, b)

def ulos(c):
    r, g, k = c.get("accent", "#8e1b1b"), c["gold"], c["primary"]
    b = f'<rect width="60" height="10" y="0" fill="{r}" opacity=".55"/><rect width="60" height="3" y="13" fill="{g}"/>'
    b += f'<rect width="60" height="3" y="44" fill="{g}"/><rect width="60" height="10" y="50" fill="{k}" opacity=".35"/>'
    for x in (10, 30, 50):
        b += f'<path d="M{x} 22 l6 7 l-6 7 l-6 -7 z" fill="{r}" opacity=".7"/>'
    return svg(60, 60, b)

def awan_larat(c):
    g, p = c["gold"], c["primary"]
    b = f'<path d="M0 50 C20 20 40 20 50 40 S80 70 100 40 S130 10 140 50" fill="none" stroke="{g}" stroke-width="2.4"/>'
    for x, y in ((25, 30), (75, 55), (120, 28)):
        b += f'<path d="M{x} {y} c8 -10 20 -4 16 6 c-3 6 -10 4 -9 -1" fill="none" stroke="{p}" stroke-width="1.8" opacity=".7"/>'
        b += f'<ellipse cx="{x+4}" cy="{y+10}" rx="7" ry="3" fill="{g}" opacity=".6" transform="rotate(-30 {x+4} {y+10})"/>'
    return svg(140, 90, b)

def cengkih(c):
    p, g = c["primary"], c["gold"]
    b = ""
    for x, y, a in ((22, 24, -20), (66, 64, 25), (70, 18, 60), (20, 70, -60)):
        b += f'<g transform="translate({x} {y}) rotate({a})"><rect x="-2" y="-2" width="4" height="22" rx="2" fill="{p}" opacity=".7"/>'
        b += f'<circle cx="0" cy="-6" r="6" fill="{g}"/><circle cx="-5" cy="-3" r="3" fill="{g}" opacity=".8"/><circle cx="5" cy="-3" r="3" fill="{g}" opacity=".8"/></g>'
    return svg(90, 90, b)

def ombak(c):
    p, g = c["primary"], c["gold"]
    b = ""
    for i, y in enumerate((20, 45, 70)):
        col = p if i % 2 == 0 else g
        b += f'<path d="M0 {y} q15 -14 30 0 t30 0 t30 0 t30 0" fill="none" stroke="{col}" stroke-width="2.2" opacity="{0.8 - i*0.15:.2f}"/>'
    return svg(120, 90, b)

def tenun(c):
    r, g, p = c.get("accent", "#8a2c1d"), c["gold"], c["primary"]
    b = f'<rect width="80" height="80" fill="none"/>'
    for y in (6, 74):
        b += f'<rect x="0" y="{y-3}" width="80" height="6" fill="{p}" opacity=".45"/>'
    b += f'<path d="M40 14 L66 40 L40 66 L14 40 Z" fill="none" stroke="{g}" stroke-width="2.2"/>'
    b += f'<path d="M40 26 L54 40 L40 54 L26 40 Z" fill="{r}" opacity=".55"/>'
    for x, y in ((40, 14), (66, 40), (40, 66), (14, 40)):
        b += f'<path d="M{x} {y} m-6 0 h12" stroke="{g}" stroke-width="2"/>'
    return svg(80, 80, b)

def gigi_balang(c):
    g, p = c["gold"], c["primary"]
    b = ""
    for x in range(0, 80, 20):
        b += f'<path d="M{x} 0 L{x+10} 22 L{x+20} 0 Z" fill="{p}" opacity=".5"/><path d="M{x} 80 L{x+10} 58 L{x+20} 80 Z" fill="{g}"/>'
    b += f'<circle cx="40" cy="40" r="7" fill="none" stroke="{g}" stroke-width="2"/>'
    return svg(80, 80, b)

def kembang_kelapa(c):
    g, p = c["gold"], c["primary"]
    b = f'<line x1="50" y1="90" x2="50" y2="30" stroke="{p}" stroke-width="2"/>'
    for i in range(9):
        a = -math.pi / 2 + (i - 4) * 0.32
        x2, y2 = 50 + math.cos(a) * 34, 30 + math.sin(a) * 34
        b += f'<path d="M50 30 Q{f(50+math.cos(a)*18)} {f(30+math.sin(a)*14)} {f(x2)} {f(y2)}" stroke="{g}" stroke-width="1.6" fill="none"/><circle cx="{f(x2)}" cy="{f(y2)}" r="3" fill="{g}"/>'
    return svg(100, 100, b)

def patra(c):
    g, p = c["gold"], c["primary"]
    b = f'<path d="M10 70 C10 30 50 20 60 40 C68 56 46 64 40 52 C36 44 46 40 50 46" fill="none" stroke="{g}" stroke-width="3" stroke-linecap="round"/>'
    b += f'<path d="M90 30 C90 60 60 72 50 58" fill="none" stroke="{p}" stroke-width="2" opacity=".6"/>'
    b += f'<path d="M62 20 q10 -10 20 0 q-10 6 -20 0z" fill="{g}" opacity=".8"/>'
    return svg(100, 90, b)

def kamboja(c):
    g = c["gold"]
    def bloom(x, y, s):
        o = f'<g transform="translate({x} {y}) scale({s})">'
        for a in range(0, 360, 72):
            o += f'<ellipse cx="0" cy="-11" rx="7" ry="12" fill="#fffaf0" stroke="{g}" stroke-width=".8" transform="rotate({a})"/>'
        return o + f'<circle r="5" fill="#f2c14e"/></g>'
    return svg(100, 100, bloom(28, 30, 0.9) + bloom(75, 74, 0.6))

def balo_renni(c):
    p, g = c["primary"], c["gold"]
    b = ""
    for i in range(4):
        for j in range(4):
            if (i + j) % 2 == 0:
                b += f'<rect x="{i*15}" y="{j*15}" width="15" height="15" fill="{p}" opacity=".28"/>'
    b += f'<path d="M0 30 H60 M30 0 V60" stroke="{g}" stroke-width="1.4"/>'
    return svg(60, 60, b)

def pa_tedong(c):
    r, k, g = c.get("accent", "#9c1d1d"), c["ink"], c["gold"]
    b = f'<circle cx="45" cy="50" r="18" fill="none" stroke="{r}" stroke-width="5"/>'
    b += f'<path d="M27 46 C10 40 6 24 14 14 M63 46 C80 40 84 24 76 14" fill="none" stroke="{k}" stroke-width="5" stroke-linecap="round"/>'
    b += f'<circle cx="45" cy="50" r="6" fill="{g}"/>'
    return svg(90, 90, b)

def pa_barre(c):
    g, r = c["gold"], c.get("accent", "#9c1d1d")
    b = f'<circle cx="45" cy="45" r="12" fill="{g}"/>'
    for i in range(16):
        a = i * math.pi / 8
        b += f'<line x1="{f(45+math.cos(a)*17)}" y1="{f(45+math.sin(a)*17)}" x2="{f(45+math.cos(a)*30)}" y2="{f(45+math.sin(a)*30)}" stroke="{r if i%2 else g}" stroke-width="3"/>'
    return svg(90, 90, b)

def pintu_aceh(c):
    g, p = c["gold"], c["primary"]
    b = f'<rect x="10" y="10" width="60" height="60" fill="none" stroke="{g}" stroke-width="2"/>'
    b += f'<path d="M40 10 L70 40 L40 70 L10 40 Z" fill="none" stroke="{g}" stroke-width="2"/>'
    b += f'<circle cx="40" cy="40" r="9" fill="{p}" opacity=".45"/><circle cx="40" cy="40" r="4" fill="{g}"/>'
    return svg(80, 80, b)

def songket_lepus(c):
    g, p = c["gold"], c["primary"]
    b = f'<rect width="50" height="50" fill="{p}" opacity=".12"/>'
    for x, y in ((0, 0), (50, 0), (0, 50), (50, 50), (25, 25)):
        b += f'<path d="M{x} {y-10} L{x+10} {y} L{x} {y+10} L{x-10} {y} Z" fill="{g}"/>'
    b += f'<path d="M25 5 V45 M5 25 H45" stroke="{g}" stroke-width="1" opacity=".6"/>'
    return svg(50, 50, b)

def tapis_kapal(c):
    g, p = c["gold"], c["primary"]
    b = f'<path d="M10 50 Q55 70 100 50 L92 60 Q55 76 18 60 Z" fill="{g}"/>'
    b += f'<path d="M55 50 V14 M55 16 L80 44 M55 22 L34 46" stroke="{g}" stroke-width="2" fill="none"/>'
    b += f'<path d="M0 80 q14 -8 28 0 t28 0 t28 0 t28 0" stroke="{p}" stroke-width="2" fill="none" opacity=".6"/>'
    return svg(110, 90, b)

def dayak(c):
    g, k = c["gold"], c["ink"]
    b = ""
    for x, s in ((0, 1), (100, -1)):
        b += (f'<path d="M{x} 50 c{20*s} -30 {50*s} -30 {50*s} 0 c0 18 {-22*s} 22 {-28*s} 8 c{-4*s} -10 {6*s} -14 {12*s} -8" '
              f'fill="none" stroke="{g}" stroke-width="3.2" stroke-linecap="round"/>')
    b += f'<path d="M50 10 c-8 12 8 12 0 24 c-8 12 8 12 0 24" stroke="{k}" stroke-width="2" fill="none" opacity=".6"/>'
    return svg(100, 90, b)

def asmat(c):
    g, k = c["gold"], c["primary"]
    b = f'<path d="M0 20 L15 5 L30 20 L45 5 L60 20" stroke="{g}" stroke-width="3" fill="none"/>'
    b += f'<path d="M0 50 L15 35 L30 50 L45 35 L60 50" stroke="{k}" stroke-width="3" fill="none" opacity=".6"/>'
    b += f'<circle cx="15" cy="27" r="3" fill="{g}"/><circle cx="45" cy="27" r="3" fill="{g}"/>'
    return svg(60, 60, b)

def ikat_sumba(c):
    r, g, p = c.get("accent", "#8a2c1d"), c["gold"], c["primary"]
    b = f'<path d="M30 6 L54 30 L30 54 L6 30 Z" fill="none" stroke="{g}" stroke-width="2"/>'
    b += f'<path d="M30 16 L44 30 L30 44 L16 30 Z" fill="{r}" opacity=".5"/>'
    b += f'<path d="M6 30 h-6 M54 30 h6 M30 6 v-6 M30 54 v6" stroke="{p}" stroke-width="3"/>'
    return svg(60, 60, b)

def bunga_tabur(c):
    g, p = c["gold"], c["primary"]
    def fl(x, y, s, col):
        o = f'<g transform="translate({x} {y}) scale({s})">'
        for a in range(0, 360, 60):
            o += f'<ellipse cx="0" cy="-6" rx="3.5" ry="6" fill="{col}" transform="rotate({a})"/>'
        return o + f'<circle r="2.5" fill="{g}"/></g>'
    return svg(80, 80, fl(20, 22, 1, g) + fl(60, 60, 0.8, p) + fl(62, 18, 0.5, g))

TILES = {
    "kawung": (kawung, 64), "parang": (parang, 90), "truntum": (truntum, 80), "sidomukti": (sidomukti, 90),
    "lurik": (lurik, 40), "mega-mendung": (mega_mendung, 140), "kujang": (kujang, 100), "bambu": (bambu, 100),
    "pucuk-rebung": (pucuk_rebung, 80), "itiak": (itiak, 90), "kaluak-paku": (kaluak_paku, 100), "gorga": (gorga, 100),
    "ulos": (ulos, 60), "awan-larat": (awan_larat, 150), "cengkih": (cengkih, 100), "ombak": (ombak, 120),
    "tenun": (tenun, 80), "gigi-balang": (gigi_balang, 80), "kembang-kelapa": (kembang_kelapa, 110), "patra": (patra, 100),
    "kamboja": (kamboja, 110), "balo-renni": (balo_renni, 60), "pa-tedong": (pa_tedong, 90), "pa-barre": (pa_barre, 90),
    "pintu-aceh": (pintu_aceh, 80), "songket-lepus": (songket_lepus, 50), "tapis-kapal": (tapis_kapal, 120),
    "dayak": (dayak, 110), "asmat": (asmat, 60), "ikat-sumba": (ikat_sumba, 60), "bunga-tabur": (bunga_tabur, 80),
}

# ── Lambang tengah hiasan kartu (card_deco 600×170) ────────────────────────

def em_gunungan(c, x, y):
    g, p = c["gold"], c["primary"]
    return (f'<g transform="translate({x} {y})"><path d="M0 -70 C26 -40 44 -10 40 30 L-40 30 C-44 -10 -26 -40 0 -70 Z" fill="{p}" opacity=".85"/>'
            f'<path d="M0 -58 C18 -34 30 -10 28 22 L-28 22 C-30 -10 -18 -34 0 -58 Z" fill="none" stroke="{g}" stroke-width="2"/>'
            f'<path d="M0 22 V-44 M0 -10 L-16 -26 M0 -10 L16 -26 M0 6 L-20 -8 M0 6 L20 -8" stroke="{g}" stroke-width="2"/>'
            f'<rect x="-14" y="22" width="28" height="10" fill="{g}"/></g>')

def em_kujang(c, x, y):
    g = c["gold"]
    return (f'<g transform="translate({x-30} {y-60})">' +
            "".join(f'<path transform="translate({dx} 0) scale({sx} 1)" d="M0 0 C14 18 10 34 2 46 C6 54 4 62 -2 70 L-6 70 C-2 60 -4 54 -8 48 C-12 40 -8 30 -2 26 C-6 18 -6 8 0 0 Z" fill="{g}"/>' for dx, sx in ((22, 1), (38, -1)))
            + "</g>")

def em_gonjong(c, x, y):
    p, g = c["primary"], c["gold"]
    return (f'<g transform="translate({x} {y})"><path d="M-70 10 Q-60 -30 -80 -54 Q-50 -26 -30 -22 Q-20 -46 0 -60 Q20 -46 30 -22 Q50 -26 80 -54 Q60 -30 70 10 Z" fill="{p}"/>'
            f'<rect x="-62" y="10" width="124" height="22" fill="{g}"/><path d="M-62 18 h124" stroke="{p}" stroke-width="2"/>'
            + "".join(f'<rect x="{-52+i*24}" y="12" width="8" height="18" fill="{p}" opacity=".5"/>' for i in range(5)) + "</g>")

def em_bolon(c, x, y):
    r, g, k = c.get("accent", "#9c1d1d"), c["gold"], c["ink"]
    return (f'<g transform="translate({x} {y})"><path d="M-80 -50 Q0 -10 80 -50 L60 8 L-60 8 Z" fill="{k}" opacity=".85"/>'
            f'<path d="M-56 8 H56 L50 30 H-50 Z" fill="{r}"/>'
            f'<path d="M-40 18 c8 -8 16 -4 14 2 M40 18 c-8 -8 -16 -4 -14 2" stroke="{g}" stroke-width="2.4" fill="none"/>'
            f'<path d="M0 12 l6 8 l-6 8 l-6 -8 z" fill="{g}"/></g>')

def em_rebung_band(c, x, y):
    g, p = c["gold"], c["primary"]
    o = f'<g transform="translate({x-110} {y-40})">'
    for i in range(6):
        o += f'<path d="M{i*40} 70 L{i*40+20} {0 if i in (2,3) else 20} L{i*40+40} 70 Z" fill="{p if i%2 else g}" opacity=".85"/>'
    return o + "</g>"

def em_tifa(c, x, y):
    p, g = c["primary"], c["gold"]
    o = f'<g transform="translate({x} {y})"><rect x="-16" y="-50" width="32" height="80" rx="10" fill="{p}"/><ellipse cx="0" cy="-50" rx="16" ry="5" fill="{g}"/>'
    o += "".join(f'<path d="M-16 {yy} L16 {yy+14}" stroke="{g}" stroke-width="2"/>' for yy in (-38, -18, 2))
    for s in (-1, 1):
        o += f'<g transform="scale({s} 1)"><path d="M24 20 C40 0 50 -20 70 -30" stroke="{p}" stroke-width="2" fill="none"/>'
        o += "".join(f'<circle cx="{40+i*10}" cy="{-4-i*9}" r="4" fill="{g}"/>' for i in range(3)) + "</g>"
    return o + "</g>"

def em_candi(c, x, y):
    p, g = c["primary"], c["gold"]
    o = f'<g transform="translate({x} {y})">'
    for s in (-1, 1):
        o += f'<g transform="scale({s} 1)"><path d="M12 30 V-10 L20 -16 V-34 L28 -40 V-56 L34 -66 L40 -56 V30 Z" fill="{p}"/>'
        o += f'<path d="M14 0 H38 M14 -20 H38" stroke="{g}" stroke-width="2"/></g>'
    return o + f'<circle cx="0" cy="-40" r="6" fill="{g}"/></g>'

def em_phinisi(c, x, y):
    p, g = c["primary"], c["gold"]
    o = f'<g transform="translate({x} {y})"><path d="M-70 14 Q0 34 70 14 L58 30 Q0 44 -58 30 Z" fill="{p}"/>'
    for dx, h in ((-30, 60), (10, 70), (44, 50)):
        o += f'<path d="M{dx} 14 V{14-h} L{dx+26} {4} Z" fill="{g}" opacity=".9"/><line x1="{dx}" y1="14" x2="{dx}" y2="{14-h}" stroke="{p}" stroke-width="2"/>'
    return o + "</g>"

def em_tongkonan(c, x, y):
    r, k, g = c.get("accent", "#9c1d1d"), c["ink"], c["gold"]
    return (f'<g transform="translate({x} {y})"><path d="M-90 -40 Q0 0 90 -40 Q60 -6 50 4 H-50 Q-60 -6 -90 -40 Z" fill="{k}"/>'
            f'<rect x="-46" y="4" width="92" height="26" fill="{r}"/>'
            f'<circle cx="-24" cy="17" r="6" fill="none" stroke="{g}" stroke-width="2"/><circle cx="24" cy="17" r="6" fill="none" stroke="{g}" stroke-width="2"/>'
            f'<path d="M-8 17 h16" stroke="{g}" stroke-width="2"/></g>')

def em_rumoh(c, x, y):
    p, g = c["primary"], c["gold"]
    return (f'<g transform="translate({x} {y})"><path d="M-70 -6 L0 -50 L70 -6 Z" fill="{p}"/><rect x="-56" y="-6" width="112" height="30" fill="{g}" opacity=".9"/>'
            + "".join(f'<path d="M{-44+i*22} 0 l8 8 l-8 8 l-8 -8 z" fill="{p}"/>' for i in range(5)) + "</g>")

def em_limas(c, x, y):
    p, g = c["primary"], c["gold"]
    return (f'<g transform="translate({x} {y})"><path d="M-80 0 L-40 -40 L40 -40 L80 0 Z" fill="{p}"/><path d="M-40 -40 L0 -62 L40 -40 Z" fill="{g}"/>'
            f'<rect x="-64" y="0" width="128" height="26" fill="{g}" opacity=".85"/><path d="M-64 13 h128" stroke="{p}" stroke-width="2"/></g>')

def em_siger(c, x, y):
    g, p = c["gold"], c["primary"]
    o = f'<g transform="translate({x} {y})"><path d="M-80 20 Q0 40 80 20 L70 30 Q0 50 -70 30 Z" fill="{p}"/>'
    for i in range(9):
        dx = -64 + i * 16
        h = 30 + (40 - abs(i - 4) * 8)
        o += f'<path d="M{dx-7} {22+abs(i-4)} L{dx} {22-h} L{dx+7} {22+abs(i-4)} Z" fill="{g}"/><circle cx="{dx}" cy="{22-h}" r="3" fill="{g}"/>'
    return o + "</g>"

def em_enggang(c, x, y):
    k, g, r = c["ink"], c["gold"], c.get("accent", "#b3261e")
    o = f'<g transform="translate({x} {y})">'
    for s in (-1, 1):
        o += f'<g transform="scale({s} 1)"><path d="M10 0 C40 -40 80 -30 96 -10 C70 -14 50 -6 30 10 Z" fill="{k}"/>'
        o += "".join(f'<path d="M{40+i*14} {-18-i*3} l10 -4" stroke="{g}" stroke-width="2"/>' for i in range(3)) + "</g>"
    return o + f'<ellipse cx="0" cy="0" rx="12" ry="16" fill="{k}"/><path d="M0 -10 c10 -16 30 -14 34 -4 c-12 -2 -22 2 -30 10 z" fill="{r}"/><circle cx="-3" cy="-6" r="2" fill="{g}"/></g>'

def em_cenderawasih(c, x, y):
    g, p, r = c["gold"], c["primary"], c.get("accent", "#b3261e")
    o = f'<g transform="translate({x} {y})"><ellipse cx="0" cy="0" rx="12" ry="8" fill="{r}"/><circle cx="-12" cy="-6" r="6" fill="{g}"/>'
    for i in range(7):
        a = -0.2 + i * 0.08
        o += f'<path d="M8 2 C40 {10+i*4} 70 {-10+i*8} {96-i*4} {-34+i*12}" stroke="{g}" stroke-width="2" fill="none" opacity="{1-i*0.08:.2f}"/>'
    return o + "</g>"

def em_ikat_band(c, x, y):
    r, g = c.get("accent", "#8a2c1d"), c["gold"]
    o = f'<g transform="translate({x-150} {y-20})"><rect width="300" height="40" fill="{c["primary"]}" opacity=".85"/>'
    for i in range(10):
        o += f'<path d="M{15+i*30} 6 l12 14 l-12 14 l-12 -14 z" fill="{r if i%2 else g}"/>'
    return o + "</g>"

def em_rumah_panggung(c, x, y):
    p, g = c["primary"], c["gold"]
    return (f'<g transform="translate({x} {y})"><path d="M-70 -10 L0 -56 L70 -10 Z" fill="{p}"/><rect x="-54" y="-10" width="108" height="26" fill="{g}" opacity=".9"/>'
            + "".join(f'<rect x="{-48+i*24}" y="16" width="5" height="18" fill="{p}"/>' for i in range(5)) + "</g>")

def em_masjid(c, x, y):
    p, g = c["primary"], c["gold"]
    return (f'<g transform="translate({x} {y})"><path d="M-30 10 Q-30 -30 0 -44 Q30 -30 30 10 Z" fill="{p}"/><path d="M0 -44 V-60" stroke="{g}" stroke-width="2"/>'
            f'<path d="M-4 -64 a6 6 0 1 0 8 0 a4.5 4.5 0 1 1 -8 0" fill="{g}"/><rect x="-60" y="-24" width="10" height="34" fill="{p}"/><rect x="50" y="-24" width="10" height="34" fill="{p}"/>'
            f'<rect x="-64" y="10" width="128" height="16" fill="{g}"/></g>')

EMBLEMS = {
    "gunungan": em_gunungan, "kujang": em_kujang, "gonjong": em_gonjong, "bolon": em_bolon, "rebung": em_rebung_band,
    "tifa": em_tifa, "candi": em_candi, "phinisi": em_phinisi, "tongkonan": em_tongkonan, "rumoh": em_rumoh, "limas": em_limas,
    "siger": em_siger, "enggang": em_enggang, "cenderawasih": em_cenderawasih, "ikat": em_ikat_band, "panggung": em_rumah_panggung,
    "masjid": em_masjid,
}

def card_deco(c, emblem, tile):
    """Hiasan tepi atas kartu: pita bermotif + lambang daerah di tengah."""
    g, p = c["gold"], c["primary"]
    b = f'<path d="M20 120 Q300 70 580 120" fill="none" stroke="{g}" stroke-width="2.4"/>'
    b += f'<path d="M60 128 Q300 90 540 128" fill="none" stroke="{p}" stroke-width="1.2" opacity=".6"/>'
    for s in (-1, 1):
        x0 = 300 + s * 120
        for i in range(5):
            xx = x0 + s * i * 30
            b += f'<circle cx="{xx}" cy="{100 + i*4}" r="{4 - i*0.5:.1f}" fill="{g}"/>'
    b += EMBLEMS[emblem](c, 300, 96)
    return svg(600, 170, b)

def frame(c, tile):
    """Bingkai lengkung foto (transparan di tengah) dengan motif di kaki."""
    g, p = c["gold"], c["primary"]
    b = f'<path d="M60 380 V170 A120 120 0 0 1 300 170 V380" fill="none" stroke="{g}" stroke-width="6"/>'
    b += f'<path d="M44 392 V166 A136 136 0 0 1 316 166 V392" fill="none" stroke="{p}" stroke-width="2" opacity=".7"/>'
    b += f'<circle cx="180" cy="34" r="10" fill="{g}"/><path d="M150 40 Q180 10 210 40" fill="none" stroke="{g}" stroke-width="3"/>'
    for x in (52, 308):
        b += f'<g transform="translate({x} 392)"><path d="M0 0 l14 -18 l14 18 l-14 18 z" fill="{g}" transform="translate(-14 0)"/></g>'
    for i in range(7):
        a = math.pi + i * math.pi / 6
        b += f'<circle cx="{f(180+math.cos(a)*128)}" cy="{f(170+math.sin(a)*128)}" r="3.5" fill="{g}"/>'
    return svg(360, 420, b)

def background(c, tile, emblem):
    """Ilustrasi latar penuh (1080×1920): gradasi lembut + pola samar + lambang di bawah."""
    fn, size = TILES[tile]
    tile_svg = fn(c)
    inner = tile_svg.split(">", 1)[1].rsplit("</svg>", 1)[0]
    vb = tile_svg.split('viewBox="')[1].split('"')[0].split()
    tw, th = float(vb[2]), float(vb[3])
    defs = (f'<linearGradient id="g" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{c["bg"]}"/>'
            f'<stop offset=".7" stop-color="{c["card"]}"/><stop offset="1" stop-color="{c["bg"]}"/></linearGradient>'
            f'<pattern id="m" width="{tw*1.6}" height="{th*1.6}" patternUnits="userSpaceOnUse"><g transform="scale(1.6)" opacity=".16">{inner}</g></pattern>')
    b = '<rect width="1080" height="1920" fill="url(#g)"/><rect width="1080" height="1920" fill="url(#m)"/>'
    b += f'<g opacity=".22" transform="translate(540 1780) scale(3.2)">{EMBLEMS[emblem](c, 0, 0)}</g>'
    b += f'<rect x="40" y="40" width="1000" height="1840" rx="40" fill="none" stroke="{c["gold"]}" stroke-width="3" opacity=".35"/>'
    return svg(1080, 1920, b, defs)

def corner(c, tile):
    """Hiasan sudut (image_mode "sudut"): kipas motif di pojok."""
    fn, size = TILES[tile]
    t = fn(c)
    inner = t.split(">", 1)[1].rsplit("</svg>", 1)[0]
    vb = t.split('viewBox="')[1].split('"')[0].split()
    tw = float(vb[2])
    s = 120 / tw
    g = c["gold"]
    b = f'<path d="M0 0 H240 A240 240 0 0 1 0 240 Z" fill="none" stroke="{g}" stroke-width="2" opacity=".7"/>'
    b += f'<path d="M0 0 H200 A200 200 0 0 1 0 200 Z" fill="{c["primary"]}" opacity=".07"/>'
    b += f'<g transform="translate(20 20) scale({s:.3f})" opacity=".9">{inner}</g>'
    b += f'<g transform="translate(110 40) scale({s*0.6:.3f})" opacity=".6">{inner}</g>'
    return svg(260, 260, b)

def tile_svg(c, tile):
    return TILES[tile][0](c)

# ── Gambar animasi ─────────────────────────────────────────────────────────

def panel(c, tile, emblem):
    """Daun pintu gerbang (dipakai animasi buka, isi = gambar)."""
    fn, _ = TILES[tile]
    t = fn(c)
    inner = t.split(">", 1)[1].rsplit("</svg>", 1)[0]
    vb = t.split('viewBox="')[1].split('"')[0].split()
    tw, th = float(vb[2]), float(vb[3])
    defs = f'<pattern id="m" width="{tw*1.4}" height="{th*1.4}" patternUnits="userSpaceOnUse"><g transform="scale(1.4)" opacity=".5">{inner}</g></pattern>'
    g, p = c["gold"], c["primary"]
    b = f'<rect width="600" height="1600" fill="{p}"/><rect width="600" height="1600" fill="url(#m)"/>'
    b += f'<rect x="40" y="40" width="520" height="1520" fill="none" stroke="{g}" stroke-width="10"/>'
    b += f'<rect x="70" y="70" width="460" height="1460" fill="none" stroke="{g}" stroke-width="3"/>'
    b += f'<rect x="110" y="520" width="380" height="560" rx="190" fill="{c["bg"]}" opacity=".14" stroke="{g}" stroke-width="4"/>'
    b += f'<g transform="translate(300 860) scale(1.6)">{EMBLEMS[emblem](c, 0, 0)}</g>'
    return svg(600, 1600, b, defs)

def float_melati():
    o = '<g transform="translate(20 20)">'
    for a in range(0, 360, 72):
        o += f'<ellipse cx="0" cy="-8" rx="5" ry="8.5" fill="#ffffff" stroke="#e8dcc0" stroke-width=".8" transform="rotate({a})"/>'
    return svg(40, 40, o + '<circle r="3.2" fill="#f2d98c"/></g>')

def float_kamboja():
    o = '<g transform="translate(24 24)">'
    for a in range(0, 360, 72):
        o += f'<ellipse cx="0" cy="-10" rx="6.5" ry="11" fill="#fffaf0" stroke="#f0d58c" stroke-width=".8" transform="rotate({a} ) rotate(14 0 -10)"/>'
    return svg(48, 48, o + '<circle r="5" fill="#f2b632"/></g>')

def float_cengkih():
    return svg(24, 44, '<rect x="10" y="12" width="4" height="30" rx="2" fill="#7a3e1d"/><circle cx="12" cy="9" r="7" fill="#a0522d"/>'
               '<circle cx="7" cy="12" r="3.5" fill="#8b4513"/><circle cx="17" cy="12" r="3.5" fill="#8b4513"/>')

def float_bulu():
    return svg(30, 70, '<path d="M15 68 C14 40 6 26 15 2 C24 26 16 40 15 68 Z" fill="#1b1b1b"/>'
               '<path d="M15 2 C20 14 21 20 18 28 L12 28 C9 20 10 14 15 2 Z" fill="#f3f0e6"/><path d="M15 20 V66" stroke="#e8c15a" stroke-width="1"/>')

def float_sirih():
    return svg(40, 44, '<path d="M20 42 C4 30 0 14 8 6 C13 1 18 4 20 9 C22 4 27 1 32 6 C40 14 36 30 20 42 Z" fill="#4f8a3a"/>'
               '<path d="M20 10 V40 M20 22 L12 16 M20 28 L28 22" stroke="#9fd17f" stroke-width="1.2" fill="none"/>')

FLOATS = {"melati": float_melati, "kamboja": float_kamboja, "cengkih": float_cengkih, "bulu-enggang": float_bulu, "daun-sirih": float_sirih}
