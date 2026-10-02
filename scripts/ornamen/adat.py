# Ilustrasi ornamen KHAS DAERAH per tema + migration/014_ornamen_adat.sql.
#   python3 scripts/ornamen/adat.py .            (dari akar proyek)
#
# Tiap tema mendapat 5 gambar dengan warna temanya sendiri
# (public/img/tema/ornamen/adat/{slug}-{bagian}.svg):
#   tokoh  — figur pengapit sampul (wayang kulit, ondel-ondel, penjor, pohon…)
#   ikon   — bangunan/lambang besar (gunungan, rumah gadang, tongkonan…)
#   atas   — hiasan gantung di atas galeri (janur, marawa, mega mendung…)
#   sudut  — kipas motif kain daerah di sudut
#   lambang— medali motif yang berputar pelan
# dan SET GERAK sendiri per daerah (wayang bergeser masuk seperti dimainkan
# dalang, pohon tumbuh dari bawah, burung terbang melayang, rumah naik…).
#
# Data warna tema: scripts/ornamen/themes.json (dump tabel themes:
#   psql … -Atc "select json_agg(json_build_object('slug',slug,'nuansa',nuansa,
#   'category',category,'description',description,'dark',dark,'tokens',tokens)
#   order by sort_order) from themes" > scripts/ornamen/themes.json)
# Motif kain diambil dari scripts/nusantara/motifs.py.
import json, math, os, re, sys

ROOT = sys.argv[1] if len(sys.argv) > 1 else "."
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "nusantara"))
import motifs as M  # noqa: E402
from data import THEMES as NUS, REGIONS  # noqa: E402

OUT = os.path.join(ROOT, "public/img/tema/ornamen/adat")
URL = "/img/tema/ornamen/adat"
os.makedirs(OUT, exist_ok=True)

# ── Warna ──────────────────────────────────────────────────────────────────
def rgb(h):
    h = h.lstrip("#")
    if len(h) == 3:
        h = "".join(c * 2 for c in h)
    return tuple(int(h[i:i + 2], 16) for i in (0, 2, 4))

def mix(a, b, t):
    A, B = rgb(a), rgb(b)
    return "#" + "".join(f"{round(x * (1 - t) + y * t):02x}" for x, y in zip(A, B))

def f(v):
    return f"{v:.1f}".rstrip("0").rstrip(".")

DEF = {"primary": "#273f2b", "gold": "#c5a059", "bg": "#f4fcf0", "ink": "#161d17", "card": "#ffffff"}

def palette(t, accent):
    tk = {k: v for k, v in (t.get("tokens") or {}).items() if isinstance(v, str) and v.startswith("#")}
    g = tk.get("gold", DEF["gold"])
    if t.get("dark"):
        # Tema gelap: ornamen emas-perunggu agar menyala di latar gelap.
        return dict(P=mix(g, "#000000", .42), Pd=mix(g, "#000000", .62), Pl=mix(g, "#000000", .2), G=g,
                    Gl=mix(g, "#ffffff", .45), Gd=mix(g, "#000000", .3), A=mix(accent, g, .45),
                    K=mix(g, "#000000", .72), B=mix(g, "#ffffff", .8), dark=True)
    p = tk.get("primary", DEF["primary"])
    return dict(P=p, Pd=mix(p, "#000000", .3), Pl=mix(p, "#ffffff", .35), G=g, Gl=mix(g, "#ffffff", .45),
                Gd=mix(g, "#000000", .3), A=accent, K=mix(tk.get("ink", DEF["ink"]), "#000000", .2),
                B=tk.get("bg", DEF["bg"]), dark=False)

def svg(w, h, body, defs=""):
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}">'
            f'<defs>{defs}</defs>{body}</svg>')

def lg(id_, c1, c2, x2=0, y2=1):
    return f'<linearGradient id="{id_}" x1="0" y1="0" x2="{x2}" y2="{y2}"><stop offset="0" stop-color="{c1}"/><stop offset="1" stop-color="{c2}"/></linearGradient>'

def holes(pts, r, fill, op=.8):
    return "".join(f'<circle cx="{f(x)}" cy="{f(y)}" r="{f(r)}" fill="{fill}" opacity="{op}"/>' for x, y in pts)

def tile_inner(c, tile):
    """Isi SVG ubin motif kain (dari motifs.py) + ukurannya."""
    pal = dict(primary=c["P"], gold=c["G"], bg=c["B"], card=c["B"], ink=c["K"], accent=c["A"])
    t = M.TILES[tile][0](pal)
    inner = t.split(">", 1)[1].rsplit("</svg>", 1)[0]
    vb = t.split('viewBox="')[1].split('"')[0].split()
    return inner, float(vb[2]), float(vb[3])

def pattern(c, tile, id_="m", scale=1.0):
    inner, w, h = tile_inner(c, tile)
    return (f'<pattern id="{id_}" width="{f(w*scale)}" height="{f(h*scale)}" patternUnits="userSpaceOnUse">'
            f'<g transform="scale({scale})">{inner}</g></pattern>')

# ═══ TOKOH (300×600, figur menghadap kanan; kiri = cermin) ══════════════════

def wayang(c, tile):
    """Wayang kulit: tokoh ksatria bermahkota, tangan bertuding, berlubang tatah."""
    P, G, Gd, K, A, B = c["P"], c["G"], c["Gd"], c["K"], c["A"], c["B"]
    d = lg("wb", c["Gl"], G) + lg("wk", P, c["Pd"]) + pattern(c, tile, "m", .35)
    b = f'<path d="M150 98 L150 596" stroke="{K}" stroke-width="5" stroke-linecap="round"/>'  # cempurit
    # tangan belakang
    b += f'<path d="M128 214 C104 262 96 300 92 336 C90 366 92 392 96 418" fill="none" stroke="{Gd}" stroke-width="13" stroke-linecap="round"/>'
    b += f'<path d="M128 214 C104 262 96 300 92 336 C90 366 92 392 96 418" fill="none" stroke="url(#wb)" stroke-width="9" stroke-linecap="round"/>'
    b += f'<path d="M96 418 L64 590" stroke="{K}" stroke-width="3"/>'
    # kain dodot
    b += f'<path d="M130 300 C108 342 98 402 92 470 L208 470 C204 404 194 344 176 300 Z" fill="url(#wk)" stroke="{Gd}" stroke-width="3"/>'
    b += f'<path d="M130 300 C108 342 98 402 92 470 L208 470 C204 404 194 344 176 300 Z" fill="url(#m)" opacity=".55"/>'
    b += f'<path d="M150 304 C146 360 152 420 168 470" fill="none" stroke="{G}" stroke-width="3"/>'
    b += f'<path d="M100 446 Q150 432 206 446" fill="none" stroke="{G}" stroke-width="4"/>'
    # kaki melangkah
    b += f'<path d="M118 470 L108 548 L84 560 L132 560 L136 470 Z M174 470 L190 546 L222 560 L174 560 L158 470 Z" fill="url(#wb)" stroke="{Gd}" stroke-width="2.5"/>'
    # badan
    b += f'<path d="M150 194 C120 198 110 216 116 242 C121 268 127 288 134 306 L172 306 C178 286 184 264 186 240 C190 214 180 198 150 194 Z" fill="url(#wb)" stroke="{Gd}" stroke-width="3"/>'
    b += holes([(150, 220 + i * 18) for i in range(5)] + [(134, 240), (166, 240), (138, 270), (162, 270)], 3.2, K, .55)
    b += f'<path d="M126 210 Q150 232 176 210" fill="none" stroke="{A}" stroke-width="6"/>'  # kalung
    b += holes([(132 + i * 8.8, 218 + (2 if 1 < i < 4 else 0)) for i in range(6)], 2.4, G, 1)
    # tangan depan menuding
    b += f'<path d="M174 214 C196 252 206 290 210 324 C214 352 228 372 252 384" fill="none" stroke="{Gd}" stroke-width="13" stroke-linecap="round"/>'
    b += f'<path d="M174 214 C196 252 206 290 210 324 C214 352 228 372 252 384" fill="none" stroke="url(#wb)" stroke-width="9" stroke-linecap="round"/>'
    b += f'<path d="M252 384 L282 590" stroke="{K}" stroke-width="3"/>'
    b += "".join(f'<rect x="{x-7}" y="{y-3}" width="14" height="6" rx="3" fill="{A}" transform="rotate({r} {x} {y})"/>' for x, y, r in ((97, 330, 80), (210, 318, 70), (240, 380, 30)))
    # kepala profil: hidung panjang, mata liyepan
    b += (f'<path d="M150 196 C140 190 132 178 130 164 C128 148 132 132 142 124 C158 116 172 124 178 136 L214 160 '
          f'C206 162 194 160 184 158 C186 166 182 172 176 172 C178 180 172 188 160 190 Z" fill="url(#wb)" stroke="{Gd}" stroke-width="3"/>')
    b += f'<path d="M160 146 Q170 142 178 148" fill="none" stroke="{K}" stroke-width="3" stroke-linecap="round"/>'
    # mahkota (topong) + sumping + jamang
    b += (f'<path d="M134 136 C128 108 136 86 156 76 C164 64 178 64 182 76 C192 84 190 104 182 124 C170 116 152 116 134 136 Z" '
          f'fill="{P}" stroke="{G}" stroke-width="3"/>')
    b += holes([(150, 96), (162, 88), (170, 100), (158, 108), (146, 116), (172, 114)], 3, G, 1)
    b += f'<path d="M136 130 C110 118 86 128 70 112 C84 146 112 150 132 152 Z" fill="{A}" stroke="{G}" stroke-width="2.5"/>'  # sumping
    b += f'<path d="M134 138 L184 126" stroke="{G}" stroke-width="5"/>'
    b += f'<circle cx="166" cy="70" r="7" fill="{G}"/>'
    return svg(300, 600, b, d)

def golek(c, tile):
    """Wayang golek Sunda: kepala kayu bulat, iket, baju & kain, tangan bertongkat."""
    P, G, Gd, K, A = c["P"], c["G"], c["Gd"], c["K"], c["A"]
    d = lg("gk", mix(G, "#ffffff", .25), Gd) + lg("gb", P, c["Pd"]) + pattern(c, tile, "m", .3)
    b = f'<path d="M150 150 V596" stroke="{K}" stroke-width="6" stroke-linecap="round"/>'
    b += f'<path d="M92 300 C70 360 60 450 66 540 L234 540 C240 450 230 360 208 300 Z" fill="url(#gb)" stroke="{G}" stroke-width="3"/>'
    b += f'<path d="M92 300 C70 360 60 450 66 540 L234 540 C240 450 230 360 208 300 Z" fill="url(#m)" opacity=".5"/>'
    b += f'<path d="M70 540 H230 L222 560 H78 Z" fill="{G}"/>'
    b += f'<path d="M110 236 C96 250 90 280 92 304 L208 304 C210 280 204 250 190 236 Z" fill="{A}" stroke="{G}" stroke-width="3"/>'
    b += f'<path d="M150 240 V304 M120 262 H180" stroke="{G}" stroke-width="2.5"/>'
    for s, (x2, y2, x3, y3) in ((-1, (70, 330, 40, 360)), (1, (230, 300, 262, 268))):
        b += f'<path d="M{150+s*42} 250 Q{x2} {y2-40} {x2} {y2}" fill="none" stroke="url(#gk)" stroke-width="12" stroke-linecap="round"/>'
        b += f'<path d="M{x2} {y2} L{x3} 596" stroke="{K}" stroke-width="3"/>'
    b += f'<ellipse cx="150" cy="186" rx="44" ry="52" fill="url(#gk)" stroke="{Gd}" stroke-width="3"/>'
    b += f'<path d="M128 186 q8 -6 16 0 M158 186 q8 -6 16 0" stroke="{K}" stroke-width="3" fill="none" stroke-linecap="round"/>'
    b += f'<path d="M150 192 v14 M140 216 q10 8 20 0" stroke="{K}" stroke-width="2.5" fill="none" stroke-linecap="round"/>'
    b += f'<path d="M104 166 C104 120 196 120 196 166 C186 150 114 150 104 166 Z" fill="{P}" stroke="{G}" stroke-width="3"/>'  # iket
    b += f'<path d="M190 150 C214 140 226 150 232 168 C218 160 206 160 196 166 Z" fill="{P}" stroke="{G}" stroke-width="2"/>'
    b += holes([(122, 150), (138, 142), (156, 140), (174, 144)], 3, G, 1)
    return svg(300, 600, b, d)

def ondel(c, tile):
    """Ondel-ondel Betawi: kepala bertopeng, rambut kembang kelapa, badan berkain."""
    P, G, K, A = c["P"], c["G"], c["K"], c["A"]
    face = "#c0392b" if not c["dark"] else c["A"]
    d = lg("ob", P, c["Pd"]) + pattern(c, tile, "m", .35)
    b = ""
    # kembang kelapa
    for i in range(15):
        a = math.radians(-170 + i * 11.5)
        x2, y2 = 150 + math.cos(a) * 128, 150 + math.sin(a) * 128
        col = (G, A, c["Gl"], "#e74c3c" if not c["dark"] else G)[i % 4]
        b += f'<path d="M150 150 L{f(x2)} {f(y2)}" stroke="{c["Gd"]}" stroke-width="2"/><circle cx="{f(x2)}" cy="{f(y2)}" r="9" fill="{col}"/>'
    b += f'<path d="M60 300 C40 380 36 470 44 560 L256 560 C264 470 260 380 240 300 Z" fill="url(#ob)" stroke="{G}" stroke-width="4"/>'
    b += f'<path d="M60 300 C40 380 36 470 44 560 L256 560 C264 470 260 380 240 300 Z" fill="url(#m)" opacity=".55"/>'
    b += f'<path d="M48 420 Q150 444 252 420 M44 500 Q150 524 256 500" stroke="{G}" stroke-width="5" fill="none"/>'
    b += f'<path d="M84 250 C70 270 62 290 60 304 L240 304 C238 290 230 270 216 250 Z" fill="{A}" stroke="{G}" stroke-width="3"/>'
    b += f'<path d="M100 254 Q150 300 200 254" stroke="{G}" stroke-width="7" fill="none"/>'
    b += f'<ellipse cx="150" cy="190" rx="60" ry="70" fill="{face}" stroke="{G}" stroke-width="4"/>'
    b += f'<path d="M100 150 C110 110 190 110 200 150 C180 136 120 136 100 150 Z" fill="{K}"/>'
    b += f'<ellipse cx="126" cy="184" rx="12" ry="9" fill="#fff"/><ellipse cx="174" cy="184" rx="12" ry="9" fill="#fff"/>'
    b += f'<circle cx="128" cy="185" r="5" fill="{K}"/><circle cx="176" cy="185" r="5" fill="{K}"/>'
    b += f'<path d="M150 192 l-8 22 h16 z" fill="{mix(face, "#000000", .25)}"/><path d="M128 230 Q150 244 172 230" stroke="{K}" stroke-width="4" fill="none"/>'
    b += f'<path d="M118 228 q-14 6 -10 18 M182 228 q14 6 10 18" stroke="{K}" stroke-width="3" fill="none"/>'
    return svg(300, 600, b, d)

def penjor(c, tile):
    """Penjor Bali: bambu melengkung, sampian janur di ujung, hiasan bergantung."""
    G, K, A = c["G"], c["K"], c["A"]
    jan, jand = ("#e8d78a", "#c3ab52") if not c["dark"] else (c["Gl"], c["Gd"])
    bam = "#a4823f" if not c["dark"] else c["P"]
    b = f'<path d="M150 600 C150 400 156 220 186 120 C206 60 244 34 270 40" fill="none" stroke="{bam}" stroke-width="12" stroke-linecap="round"/>'
    b += "".join(f'<path d="M{f(150+(i*1.1)**1.35)} {600-i*44} h14" stroke="{mix(bam, "#000000", .3)}" stroke-width="3"/>' for i in range(1, 10))
    # sampian di ujung
    b += f'<g transform="translate(270 40)">'
    for i in range(7):
        a = 60 + i * 14
        b += f'<path d="M0 0 C{f(math.cos(math.radians(a))*30)} {f(math.sin(math.radians(a))*30)} {f(math.cos(math.radians(a+10))*70)} {f(math.sin(math.radians(a+10))*90)} {f(math.cos(math.radians(a))*40)} {f(math.sin(math.radians(a))*130)}" fill="none" stroke="{jan if i % 2 else jand}" stroke-width="6" stroke-linecap="round"/>'
    b += f'<circle r="12" fill="{A}"/><circle r="6" fill="{G}"/></g>'
    # rumbai janur sepanjang batang
    for i, y in enumerate((180, 260, 340, 420)):
        x = 150 + (600 - y) ** 1.35 / 600 * 8 + (30 if y < 200 else 6)
        b += f'<path d="M{f(x)} {y} q-30 30 -18 70 M{f(x)} {y} q-12 34 4 74 M{f(x)} {y} q8 30 26 64" fill="none" stroke="{jan}" stroke-width="4" stroke-linecap="round"/>'
        b += f'<path d="M{f(x-14)} {y+6} l14 -10 l14 10 l-14 10 z" fill="{jand}"/>'
    b += f'<path d="M110 560 h80 l-8 40 h-64 z" fill="{K}" opacity=".7"/>'
    return svg(300, 600, b)

def pohon(c, tile, tinggi=1.0):
    """Pohon rindang: batang bercabang + tajuk dedaunan berlapis."""
    leaf = ["#2f5d3a", "#3f7a49", "#5c9a5e", "#86b77a", "#a9cf8f"] if not c["dark"] else [c["Pd"], c["P"], c["Gd"], c["G"], c["Gl"]]
    trunk = "#6b4a2b" if not c["dark"] else c["Pd"]
    b = f'<path d="M136 600 C140 520 132 460 140 400 C146 352 128 312 112 280 M140 420 C160 380 190 350 206 300 M138 470 C114 440 92 430 74 404" fill="none" stroke="{trunk}" stroke-width="20" stroke-linecap="round"/>'
    b += f'<path d="M136 600 C140 520 132 460 140 400" stroke="{mix(trunk, "#ffffff", .2)}" stroke-width="5" fill="none"/>'
    blobs = [(150, 200, 120), (90, 250, 80), (214, 250, 78), (150, 120, 88), (70, 330, 56), (226, 320, 60), (150, 290, 86)]
    import random
    rng = random.Random(4)
    for li, col in enumerate(leaf):
        for (x, y, r) in blobs:
            for k in range(3):
                rr = r * (0.75 - li * 0.1) * rng.uniform(.8, 1.1)
                dx, dy = rng.uniform(-r, r) * .45, rng.uniform(-r, r) * .4 - li * 10
                b += f'<circle cx="{f(x+dx)}" cy="{f(y+dy)}" r="{f(max(rr, 10))}" fill="{col}" opacity=".95"/>'
    b += holes([(rng.uniform(40, 260), rng.uniform(80, 360)) for _ in range(14)], 5, c["G"] if c["dark"] else "#f3d27c", .9)
    return svg(300, 600, b)

def burung(c, tile, jenis="enggang"):
    """Burung terbang: enggang (paruh bertanduk, ekor panjang) / cendrawasih (bulu jumbai)."""
    P, G, K, A = c["P"], c["G"], c["K"], c["A"]
    body = K if jenis == "enggang" and not c["dark"] else (P if jenis == "enggang" else ("#7b3f12" if not c["dark"] else c["P"]))
    d = lg("bw", body, mix(body, "#ffffff", .25), 1, 0)
    b = ""
    if jenis == "enggang":
        for i in range(4):  # ekor panjang belang
            b += f'<path d="M110 300 C80 {360+i*30} 60 {440+i*30} {40+i*14} {560-i*8}" fill="none" stroke="{"#ffffff" if i%2 else body}" stroke-width="12" stroke-linecap="round"/>'
            b += f'<path d="M{40+i*14} {560-i*8} l6 -18" stroke="{K}" stroke-width="10" stroke-linecap="round"/>'
    else:
        for i in range(9):  # jumbai bulu emas-kuning
            b += f'<path d="M118 300 C{60-i*6} {340+i*10} {40+i*8} {460+i*8} {70+i*14} {590-i*6}" fill="none" stroke="{G if i%2 else "#f1c84b" if not c["dark"] else c["Gl"]}" stroke-width="4" stroke-linecap="round" opacity="{1-i*.05:.2f}"/>'
    # sayap atas & bawah
    b += f'<path d="M150 260 C120 170 150 80 250 40 C230 110 220 170 186 250 Z" fill="url(#bw)" stroke="{G}" stroke-width="3"/>'
    b += "".join(f'<path d="M{170+i*14} {230-i*30} l{30-i*2} -26" stroke="{G}" stroke-width="2.5"/>' for i in range(5))
    b += f'<ellipse cx="148" cy="290" rx="52" ry="34" fill="{body}" stroke="{G}" stroke-width="3" transform="rotate(-24 148 290)"/>'
    b += f'<path d="M140 280 C110 240 80 240 60 220 C90 280 120 300 144 300 Z" fill="url(#bw)" stroke="{G}" stroke-width="2.5"/>'
    # kepala + paruh
    hx, hy = 196, 254
    b += f'<circle cx="{hx}" cy="{hy}" r="20" fill="{body}" stroke="{G}" stroke-width="2.5"/><circle cx="{hx+6}" cy="{hy-4}" r="4" fill="{G}"/>'
    if jenis == "enggang":
        b += f'<path d="M{hx+14} {hy-6} C{hx+50} {hy-14} {hx+80} {hy+6} {hx+92} {hy+30} C{hx+66} {hy+16} {hx+40} {hy+14} {hx+16} {hy+8} Z" fill="#f2c14e"/>'
        b += f'<path d="M{hx+10} {hy-16} C{hx+40} {hy-40} {hx+70} {hy-26} {hx+76} {hy-8} C{hx+52} {hy-20} {hx+34} {hy-16} {hx+16} {hy-6} Z" fill="{A if not c["dark"] else G}"/>'
    else:
        b += f'<path d="M{hx+16} {hy-2} l26 6 l-26 6 z" fill="{G}"/><circle cx="{hx}" cy="{hy-6}" r="14" fill="#f1c84b" opacity=".85"/>'
    return svg(300, 600, b, d)

def kain_gantung(c, tile, garis=False):
    """Kain adat tergantung (ulos/tapis/sabbe/songket) dengan rumbai."""
    G, K = c["G"], c["K"]
    d = pattern(c, tile, "m", .5)
    b = f'<path d="M40 40 H260" stroke="{c["Gd"]}" stroke-width="10" stroke-linecap="round"/><circle cx="40" cy="40" r="10" fill="{G}"/><circle cx="260" cy="40" r="10" fill="{G}"/>'
    b += f'<path d="M70 44 H230 C236 200 226 380 236 520 H64 C74 380 64 200 70 44 Z" fill="{c["P"]}" stroke="{G}" stroke-width="3"/>'
    b += f'<path d="M70 44 H230 C236 200 226 380 236 520 H64 C74 380 64 200 70 44 Z" fill="url(#m)" opacity=".85"/>'
    if garis:
        b += "".join(f'<path d="M{f(90+i*20)} 50 C{f(94+i*20)} 220 {f(86+i*20)} 380 {f(94+i*20)} 516" stroke="{c["A"] if i % 2 else G}" stroke-width="5" fill="none" opacity=".9"/>' for i in range(7))
    b += f'<path d="M66 470 H234 M68 488 H232" stroke="{G}" stroke-width="5"/>'
    b += "".join(f'<path d="M{70+i*8} 520 l{(-2 if i%2 else 2)} 44" stroke="{G if i % 3 else c["A"]}" stroke-width="3" stroke-linecap="round"/>' for i in range(21))
    return svg(300, 600, b, d)

def payung(c, tile):
    """Payung kebesaran (Melayu/Palembang): kubah berumbai bertingkat di tiang tinggi."""
    P, G, A = c["P"], c["G"], c["A"]
    top = "#e6b822" if not c["dark"] else G
    b = f'<path d="M150 80 V596" stroke="{c["Gd"]}" stroke-width="8"/>'
    b += f'<path d="M30 200 C40 110 100 70 150 70 C200 70 260 110 270 200 Z" fill="{top}" stroke="{c["Gd"]}" stroke-width="3"/>'
    b += "".join(f'<path d="M150 72 Q{f(150+(i-3)*30)} 120 {f(36+i*38)} 198" fill="none" stroke="{c["Gd"]}" stroke-width="2"/>' for i in range(7))
    for row, (y, col) in enumerate(((200, A), (232, P), (262, top))):
        b += f'<path d="M{30+row*8} {y} H{270-row*8} L{262-row*8} {y+30} H{38+row*8} Z" fill="{col}" stroke="{G}" stroke-width="2"/>'
        b += "".join(f'<path d="M{f(40+row*8+i*(220-row*16)/14)} {y+30} l0 14" stroke="{G}" stroke-width="3"/>' for i in range(15))
    b += f'<circle cx="150" cy="62" r="12" fill="{G}"/><path d="M150 40 l6 12 h-12 z" fill="{G}"/>'
    return svg(300, 600, b)

def rangkiang(c, tile):
    """Rangkiang Minang: lumbung padi beratap gonjong."""
    P, G, K, A = c["P"], c["G"], c["K"], c["A"]
    roof = K if not c["dark"] else c["Pd"]
    d = pattern(c, tile, "m", .3)
    rf = roof_curve([(18, 40), (282, 40)], 292, 236)
    b = f'<path d="{rf}" fill="{roof}" stroke="{G}" stroke-width="4"/>'
    b += "".join(f'<path d="M{x} 40 l-6 -20 l12 0 z" fill="{G}"/>' for x in (18, 282))
    b += f'<path d="M70 268 Q150 236 230 268" fill="none" stroke="{G}" stroke-width="3"/>'
    b += f'<path d="M86 290 L214 290 L226 460 L74 460 Z" fill="{A}" stroke="{G}" stroke-width="3"/>'
    b += f'<path d="M86 290 L214 290 L226 460 L74 460 Z" fill="url(#m)" opacity=".7"/>'
    b += f'<path d="M80 330 H220 M78 410 H222" stroke="{G}" stroke-width="5"/>'
    b += f'<rect x="130" y="350" width="40" height="44" fill="{K}" stroke="{G}" stroke-width="3"/>'
    b += "".join(f'<rect x="{x}" y="460" width="12" height="90" fill="{mix(roof, "#ffffff", .15)}"/>' for x in (90, 144, 198))
    b += f'<path d="M64 550 H236" stroke="{c["Gd"]}" stroke-width="6"/>'
    return svg(300, 600, b, d)

def tanduk(c, tile):
    """Tiang tongkonan bersusun tanduk kerbau (Toraja)."""
    G, K, A = c["G"], c["K"], c["A"]
    b = f'<rect x="132" y="60" width="36" height="540" fill="{c["P"]}" stroke="{G}" stroke-width="3"/>'
    b += "".join(f'<path d="M132 {80+i*40} H168" stroke="{G}" stroke-width="3"/>' for i in range(13))
    horn = "#f3ead8" if not c["dark"] else c["Gl"]
    for i in range(6):
        y = 90 + i * 52
        w = 110 - i * 6
        b += f'<path d="M150 {y+10} C{150-w*.6} {y+14} {150-w} {y-6} {150-w*1.05} {y-34} C{150-w*.8} {y-10} {150-w*.4} {y-6} 150 {y-4} Z" fill="{horn}" stroke="{K}" stroke-width="2"/>'
        b += f'<path d="M150 {y+10} C{150+w*.6} {y+14} {150+w} {y-6} {150+w*1.05} {y-34} C{150+w*.8} {y-10} {150+w*.4} {y-6} 150 {y-4} Z" fill="{horn}" stroke="{K}" stroke-width="2"/>'
    b += f'<path d="M110 30 L150 2 L190 30 Z" fill="{A}"/>'
    return svg(300, 600, b)

def cengkih(c, tile):
    """Ranting cengkih berbuah (Maluku)."""
    G = c["G"]
    stem = "#6b4a2b" if not c["dark"] else c["Pd"]
    leaf = ("#3f6e45", "#6f9c62") if not c["dark"] else (c["P"], c["Gd"])
    b = f'<path d="M140 600 C150 460 120 320 160 180 C176 120 210 70 250 40" fill="none" stroke="{stem}" stroke-width="7" stroke-linecap="round"/>'
    for i, (x, y, a) in enumerate(((146, 520, -40), (132, 440, 200), (150, 360, -30), (138, 290, 210), (162, 210, -20), (190, 130, 200))):
        b += f'<g transform="translate({x} {y}) rotate({a})"><path d="M0 0 C30 -18 70 -14 96 0 C70 14 30 18 0 0 Z" fill="{leaf[i%2]}"/><path d="M4 0 H90" stroke="{mix(leaf[0], "#000000", .2)}" stroke-width="1.5"/></g>'
    for (x, y) in ((250, 40), (176, 160), (128, 330), (150, 470)):
        for k in range(5):
            a = math.radians(-60 + k * 30)
            ex, ey = x + math.cos(a) * 34, y + math.sin(a) * 34 + 20
            b += f'<path d="M{x} {y} L{f(ex)} {f(ey)}" stroke="{stem}" stroke-width="2"/><ellipse cx="{f(ex)}" cy="{f(ey)}" rx="5" ry="9" fill="{"#c0392b" if k % 2 else "#d4762c"}" transform="rotate({f(math.degrees(a)-90)} {f(ex)} {f(ey)})"/><circle cx="{f(ex+math.cos(a)*7)}" cy="{f(ey+math.sin(a)*7)}" r="4" fill="{G}"/>'
    return svg(300, 600, b)

def lontar(c, tile):
    """Pohon lontar (NTT): batang tinggi, tajuk daun kipas."""
    trunk = "#7a5a3a" if not c["dark"] else c["Pd"]
    leaf = ("#3d6b3f", "#5f8f4f", "#86ad66") if not c["dark"] else (c["Pd"], c["P"], c["G"])
    b = f'<path d="M150 600 C146 460 158 300 150 170" stroke="{trunk}" stroke-width="18" fill="none" stroke-linecap="round"/>'
    b += "".join(f'<path d="M{f(146+math.sin(i)*3)} {590-i*28} h12" stroke="{mix(trunk, "#000000", .3)}" stroke-width="3"/>' for i in range(15))
    for i in range(11):
        a = math.radians(-180 + i * 18)
        x2, y2 = 150 + math.cos(a) * 120, 170 + math.sin(a) * 110 + 30
        col = leaf[i % 3]
        b += f'<path d="M150 170 L{f(x2)} {f(y2)}" stroke="{col}" stroke-width="5"/>'
        for k in range(6):
            t = .45 + k * .1
            px, py = 150 + (x2 - 150) * t, 170 + (y2 - 170) * t
            b += f'<path d="M{f(px)} {f(py)} l{f(math.cos(a-0.6)*22)} {f(math.sin(a-0.6)*22)} M{f(px)} {f(py)} l{f(math.cos(a+0.6)*22)} {f(math.sin(a+0.6)*22)}" stroke="{col}" stroke-width="3"/>'
    b += "".join(f'<circle cx="{x}" cy="{y}" r="9" fill="#5a3a22"/>' for x, y in ((138, 186), (160, 190), (150, 200)))
    return svg(300, 600, b)

def karang(c, tile):
    """Taman laut: karang bercabang, rumput laut melambai, ikan kecil."""
    cor = ("#e8735a", "#f2a07b", "#f6c39b") if not c["dark"] else (c["Gd"], c["G"], c["Gl"])
    weed = "#3e8a6e" if not c["dark"] else c["P"]
    b = ""
    for i, x in enumerate((60, 110, 230)):
        b += f'<path d="M{x} 600 C{x-20} 520 {x+30} 470 {x} 380 C{x-24} 320 {x+20} 280 {x+4} 220" fill="none" stroke="{weed}" stroke-width="10" stroke-linecap="round" opacity=".85"/>'
    def branch(x, y, a, l, d, col):
        nonlocal b
        if d == 0 or l < 10:
            b += f'<circle cx="{f(x)}" cy="{f(y)}" r="7" fill="{col}"/>'
            return
        x2, y2 = x + math.cos(math.radians(a)) * l, y + math.sin(math.radians(a)) * l
        b += f'<path d="M{f(x)} {f(y)} L{f(x2)} {f(y2)}" stroke="{col}" stroke-width="{f(4+d*3)}" stroke-linecap="round"/>'
        branch(x2, y2, a - 24, l * .74, d - 1, col)
        branch(x2, y2, a + 22, l * .72, d - 1, col)
    branch(160, 600, -90, 110, 5, cor[0])
    branch(220, 600, -80, 70, 4, cor[1])
    branch(90, 600, -100, 60, 4, cor[2])
    for (x, y, s) in ((80, 200, 1), (210, 140, -1), (120, 120, 1)):
        fish = c["G"] if c["dark"] else "#f2b134"
        b += f'<g transform="translate({x} {y}) scale({s} 1)"><path d="M-20 0 C-6 -14 16 -12 22 0 C16 12 -6 14 -20 0 Z M-20 0 L-34 -10 L-30 0 L-34 10 Z" fill="{fish}"/><circle cx="12" cy="-3" r="2.4" fill="#222"/></g>'
    b += "".join(f'<circle cx="{x}" cy="{y}" r="{r}" fill="none" stroke="{weed}" stroke-width="2" opacity=".6"/>' for x, y, r in ((250, 260, 6), (240, 230, 4), (256, 206, 3)))
    return svg(300, 600, b)

def pampas(c, tile):
    """Rumput pampas & daun kering (Modern / Rustic)."""
    pl = ("#e8d9bf", "#d8c19b", "#c9a87a") if not c["dark"] else (c["Gl"], c["G"], c["Gd"])
    stem = "#a88b62" if not c["dark"] else c["Gd"]
    b = ""
    for i, (x, h, a) in enumerate(((150, 520, 0), (110, 440, -14), (196, 470, 12), (80, 340, -26), (226, 360, 24))):
        ex, ey = x + math.sin(math.radians(a)) * h, 600 - math.cos(math.radians(a)) * h
        b += f'<path d="M150 600 Q{f((150+ex)/2)} {f((600+ey)/2+20)} {f(ex)} {f(ey)}" stroke="{stem}" stroke-width="3" fill="none"/>'
        for k in range(26):
            t = .5 + k * .02
            px, py = 150 + (ex - 150) * t, 600 + (ey - 600) * t
            for s in (-1, 1):
                b += f'<path d="M{f(px)} {f(py)} q{s*16} -10 {s*26} -30" stroke="{pl[(i+k) % 3]}" stroke-width="3" fill="none" stroke-linecap="round" opacity=".9"/>'
    for (x, y, a) in ((150, 600, -60), (150, 600, -120)):
        b += f'<g transform="translate({x} {y}) rotate({a})"><path d="M0 0 C40 -12 120 -10 170 0 C120 10 40 12 0 0 Z" fill="{c["P"] if not c["dark"] else c["Gd"]}" opacity=".8"/></g>'
    return svg(300, 600, b)

def bunga_tangkai(c, tile):
    """Tangkai bunga krisan (Tomohon, Minahasa)."""
    cols = ("#e85d75", "#f29bb0", "#f7c948", "#ffffff") if not c["dark"] else (c["G"], c["Gl"], c["Gd"], c["B"])
    leaf = "#4f7d46" if not c["dark"] else c["P"]
    b = ""
    for i, (x2, y2) in enumerate(((150, 120), (90, 220), (214, 200), (120, 340), (200, 330))):
        b += f'<path d="M150 600 Q{(150+x2)//2} {(600+y2)//2+30} {x2} {y2}" stroke="{leaf}" stroke-width="5" fill="none"/>'
        b += f'<g transform="translate({(150+x2)//2+10} {(600+y2)//2+20}) rotate({-40 if i%2 else 30})"><path d="M0 0 C20 -10 50 -8 64 0 C50 8 20 10 0 0Z" fill="{leaf}"/></g>'
    for i, (x, y) in enumerate(((150, 120), (90, 220), (214, 200), (120, 340), (200, 330))):
        col = cols[i % 4]
        r = 38 - i * 3
        for k in range(16):
            a = k * 22.5
            b += f'<ellipse cx="{x}" cy="{f(y-r*.55)}" rx="{f(r*.16)}" ry="{f(r*.55)}" fill="{col}" stroke="{mix(col, "#000000", .2)}" stroke-width="1" transform="rotate({a} {x} {y})"/>'
        b += f'<circle cx="{x}" cy="{y}" r="{f(r*.28)}" fill="{cols[2] if col != cols[2] else cols[0]}"/>'
    return svg(300, 600, b)

def menara(c, tile):
    """Menara masjid (Aceh/Islami): tiang bersegmen, balkon, kubah bulan sabit."""
    P, G, K = c["P"], c["G"], c["K"]
    wall = "#f6f1e6" if not c["dark"] else c["Gl"]
    b = f'<rect x="110" y="200" width="80" height="400" fill="{wall}" stroke="{G}" stroke-width="3"/>'
    b += "".join(f'<path d="M110 {y} H190" stroke="{G}" stroke-width="4"/>' for y in (300, 420, 520))
    b += "".join(f'<path d="M{x} {y} v30 a10 10 0 0 1 20 0 v-30" fill="none" stroke="{P}" stroke-width="3"/>' for x, y in ((140, 330), (140, 450)))
    b += f'<path d="M96 200 H204 L196 220 H104 Z" fill="{P}"/><path d="M96 170 H204 V200 H96 Z" fill="{G}"/>'
    b += "".join(f'<path d="M{100+i*12} 170 v30" stroke="{P}" stroke-width="2"/>' for i in range(9))
    b += f'<path d="M110 170 C110 110 150 80 150 60 C150 80 190 110 190 170 Z" fill="{K if not c["dark"] else P}" stroke="{G}" stroke-width="3"/>'
    b += f'<path d="M150 60 V30" stroke="{G}" stroke-width="3"/><path d="M144 26 a9 9 0 1 0 12 -8 a7 7 0 1 1 -12 8" fill="{G}"/>'
    return svg(300, 600, b)

def layar(c, tile):
    """Tiang & layar perahu pinisi (Bugis-Makassar)."""
    P, G, K = c["P"], c["G"], c["K"]
    sail = "#f4ead6" if not c["dark"] else c["Gl"]
    b = f'<path d="M150 20 V560" stroke="{mix(P, "#000000", .3)}" stroke-width="8"/>'
    b += f'<path d="M156 60 C220 120 250 240 240 330 L156 330 Z" fill="{sail}" stroke="{G}" stroke-width="3"/>'
    b += f'<path d="M144 90 C90 150 70 250 80 320 L144 320 Z" fill="{mix(sail, "#000000", .06)}" stroke="{G}" stroke-width="3"/>'
    b += f'<path d="M156 350 C206 380 226 440 220 500 L156 500 Z" fill="{sail}" stroke="{G}" stroke-width="3"/>'
    b += "".join(f'<path d="M156 {y} H{230 - abs(y-200)//6}" stroke="{G}" stroke-width="1.5" opacity=".6"/>' for y in range(120, 330, 40))
    b += f'<path d="M150 20 L190 32 L150 44 Z" fill="{c["A"]}"/>'
    b += f'<path d="M40 560 Q150 600 260 560 L240 590 Q150 616 60 590 Z" fill="{P}"/>'
    return svg(300, 600, b)

# ═══ IKON (600×420, bertumpu di dasar) ═════════════════════════════════════

def gunungan(c, tile):
    """Gunungan / kayon: daun kehidupan, pohon hayat, gapura, tatah berlubang."""
    P, G, Gd, K, A = c["P"], c["G"], c["Gd"], c["K"], c["A"]
    d = lg("gn", mix(P, G, .15), c["Pd"]) + pattern(c, tile, "m", .45)
    sh = "M300 6 C380 80 470 210 456 330 C452 366 440 392 430 404 L170 404 C160 392 148 366 144 330 C130 210 220 80 300 6 Z"
    b = f'<path d="{sh}" fill="url(#gn)" stroke="{G}" stroke-width="6"/>'
    b += f'<path d="{sh}" fill="url(#m)" opacity=".35"/>'
    b += f'<path d="M300 30 C368 96 444 214 432 324 C430 352 420 376 412 386 L188 386 C180 376 170 352 168 324 C156 214 232 96 300 30 Z" fill="none" stroke="{G}" stroke-width="2.5" stroke-dasharray="2 7" stroke-linecap="round"/>'
    # pohon hayat
    b += f'<path d="M300 330 V70" stroke="{G}" stroke-width="7"/>'
    for i, y in enumerate(range(100, 300, 34)):
        w = 40 + i * 16
        for s in (-1, 1):
            b += f'<path d="M300 {y+30} C{300+s*w*.4} {y+10} {300+s*w*.8} {y+10} {300+s*w} {y-10} C{300+s*w*.9} {y+16} {300+s*w*.6} {y+30} {300+s*w*.3} {y+34}" fill="none" stroke="{G}" stroke-width="3.5" stroke-linecap="round"/>'
            b += f'<circle cx="{300+s*w}" cy="{y-10}" r="6" fill="{A}" stroke="{G}" stroke-width="2"/>'
    # gapura & pintu
    b += f'<path d="M226 404 V330 H374 V404 Z" fill="{K}" stroke="{G}" stroke-width="4"/>'
    b += f'<path d="M210 330 H390 L374 310 H226 Z" fill="{G}"/><path d="M240 296 H360 L344 280 H256 Z" fill="{Gd}"/>'
    b += f'<rect x="284" y="346" width="32" height="58" fill="{A}" stroke="{G}" stroke-width="3"/>'
    for s in (-1, 1):  # sepasang kepala raksasa penjaga
        b += f'<circle cx="{300+s*58}" cy="370" r="16" fill="{A}" stroke="{G}" stroke-width="3"/><circle cx="{300+s*58}" cy="366" r="4" fill="{G}"/>'
    b += f'<path d="M168 404 H432" stroke="{G}" stroke-width="8" stroke-linecap="round"/>'
    return svg(600, 420, b, d)

def roof_curve(tips, base_y, valley):
    """Atap bergonjong: puncak-puncak melengkung tajam dengan lembah di antaranya."""
    (x0, y0) = tips[0]
    p = f"M{x0} {y0}"
    for (xa, ya), (xb, yb) in zip(tips, tips[1:]):
        mx = (xa + xb) / 2
        p += f" C{f(xa+(mx-xa)*.25)} {f(ya+50)} {f(mx-(mx-xa)*.4)} {valley} {f(mx)} {valley} C{f(mx+(xb-mx)*.4)} {valley} {f(xb-(xb-mx)*.25)} {f(yb+50)} {xb} {yb}"
    xl, xr = tips[0][0], tips[-1][0]
    p += f" C{xr-20} {f(y0+80)} {xr-50} {base_y-10} {xr-70} {base_y} L{xl+70} {base_y} C{xl+50} {base_y-10} {xl+20} {f(y0+80)} {xl} {y0} Z"
    return p

def rumah_gadang(c, tile):
    """Rumah gadang Minang: atap ijuk bergonjong, dinding ukir, berkolong."""
    G, K, A = c["G"], c["K"], c["A"]
    roof = "#2b2420" if not c["dark"] else c["Pd"]
    d = pattern(c, tile, "m", .35) + lg("rf", mix(roof, "#ffffff", .12), roof)
    tips = [(30, 30), (150, 70), (300, 96), (450, 70), (570, 30)]
    b = f'<path d="{roof_curve(tips, 230, 180)}" fill="url(#rf)" stroke="{G}" stroke-width="4"/>'
    b += "".join(f'<path d="M{x} {y} l-6 -18 l12 0 z" fill="{G}"/>' for x, y in tips)
    b += f'<path d="M100 230 H500 L520 330 H80 Z" fill="{A}" stroke="{G}" stroke-width="4"/>'
    b += f'<path d="M100 230 H500 L520 330 H80 Z" fill="url(#m)" opacity=".75"/>'
    b += f'<path d="M92 260 H508 M86 300 H514" stroke="{G}" stroke-width="5"/>'
    b += "".join(f'<rect x="{x}" y="268" width="26" height="26" rx="3" fill="{K}" stroke="{G}" stroke-width="2.5"/>' for x in (130, 190, 250, 324, 384, 444))
    b += f'<path d="M80 330 H520" stroke="{c["Gd"]}" stroke-width="6"/>'
    b += "".join(f'<rect x="{x}" y="334" width="12" height="60" fill="{mix(roof, "#ffffff", .2)}"/>' for x in range(100, 520, 48))
    b += f'<path d="M270 394 L282 334 H318 L330 394 Z" fill="{c["P"]}" stroke="{G}" stroke-width="3"/>'
    b += "".join(f'<path d="M{272+i*3} {380-i*14} H{328-i*3}" stroke="{G}" stroke-width="3"/>' for i in range(4))
    b += f'<path d="M40 400 H560" stroke="{c["Gd"]}" stroke-width="8" stroke-linecap="round"/>'
    return svg(600, 420, b, d)

def rumah_bolon(c, tile):
    """Rumah bolon Batak: atap pelana menjulang di kedua ujung, gorga merah-putih-hitam."""
    G, K = c["G"], c["K"]
    red = "#a3201b" if not c["dark"] else c["A"]
    roof = "#2a2420" if not c["dark"] else c["Pd"]
    b = f'<path d="M20 20 C120 130 480 130 580 20 C540 110 520 170 480 210 H120 C80 170 60 110 20 20 Z" fill="{roof}" stroke="{G}" stroke-width="4"/>'
    b += f'<path d="M150 210 L300 70 L450 210 Z" fill="{red}" stroke="{G}" stroke-width="4"/>'
    for i in range(3):  # gorga ulir
        y = 120 + i * 30
        b += f'<path d="M{300-(i+1)*36} {y+40} c10 -20 30 -20 30 0 c0 14 -16 16 -18 6 M{300+(i+1)*36} {y+40} c-10 -20 -30 -20 -30 0 c0 14 16 16 18 6" fill="none" stroke="#ffffff" stroke-width="3.5"/>'
    b += f'<path d="M300 100 l14 24 l-14 24 l-14 -24 z" fill="{G}"/>'
    b += f'<path d="M110 210 H490 V290 H110 Z" fill="{red}" stroke="{G}" stroke-width="4"/>'
    b += "".join(f'<path d="M{x} 230 l14 20 l-14 20 l-14 -20 z" fill="none" stroke="{"#ffffff" if i%2 else K}" stroke-width="3"/>' for i, x in enumerate(range(140, 480, 40)))
    b += f'<path d="M100 290 H500" stroke="{G}" stroke-width="6"/>'
    b += "".join(f'<rect x="{x}" y="294" width="14" height="96" fill="{mix(roof, "#ffffff", .2)}"/>' for x in range(120, 500, 44))
    b += f'<path d="M40 398 H560" stroke="{c["Gd"]}" stroke-width="8" stroke-linecap="round"/>'
    return svg(600, 420, b)

def tongkonan(c, tile):
    """Tongkonan Toraja: atap perahu melengkung tinggi, dinding ukir pa'ssura."""
    G, K = c["G"], c["K"]
    red = "#a31b1b" if not c["dark"] else c["A"]
    roof = "#3a3330" if not c["dark"] else c["Pd"]
    b = f'<path d="M10 10 C110 150 490 150 590 10 C560 120 520 170 470 190 H130 C80 170 40 120 10 10 Z" fill="{roof}" stroke="{G}" stroke-width="4"/>'
    b += f'<path d="M24 26 C120 150 480 150 576 26" fill="none" stroke="{G}" stroke-width="2" stroke-dasharray="6 6"/>'
    b += f'<path d="M140 190 H460 V290 H140 Z" fill="{red}" stroke="{G}" stroke-width="4"/>'
    for i, x in enumerate(range(170, 450, 56)):
        b += f'<circle cx="{x}" cy="240" r="18" fill="none" stroke="{"#ffffff" if not c["dark"] else G}" stroke-width="3"/><circle cx="{x}" cy="240" r="6" fill="{G}"/>'
        b += f'<path d="M{x-28} 214 l28 -14 l28 14 M{x-28} 266 l28 14 l28 -14" stroke="{K}" stroke-width="3" fill="none"/>'
    b += f'<path d="M300 150 V190" stroke="{G}" stroke-width="5"/>'
    for i in range(3):
        y = 150 - i * 0
        b += f'<path d="M300 {162+i*10} C{260-i*8} {164+i*10} {236-i*10} {150+i*10} {230-i*10} {124+i*10} M300 {162+i*10} C{340+i*8} {164+i*10} {364+i*10} {150+i*10} {370+i*10} {124+i*10}" fill="none" stroke="#f3ead8" stroke-width="6" stroke-linecap="round"/>'
    b += f'<path d="M130 290 H470" stroke="{G}" stroke-width="6"/>'
    b += "".join(f'<rect x="{x}" y="294" width="16" height="100" fill="{mix(roof, "#ffffff", .2)}"/>' for x in range(150, 460, 50))
    b += f'<path d="M40 398 H560" stroke="{c["Gd"]}" stroke-width="8" stroke-linecap="round"/>'
    return svg(600, 420, b)

def candi_bentar(c, tile):
    """Candi bentar Bali: gapura terbelah bertingkat + anak tangga + penjor kecil."""
    G, K = c["G"], c["K"]
    stone = "#8a6f58" if not c["dark"] else c["P"]
    brick = "#b5654a" if not c["dark"] else c["Pd"]
    b = ""
    for s in (-1, 1):
        o = f'<g transform="translate(300 0) scale({s} 1)">'
        o += f'<path d="M40 400 V140 L60 130 V100 L80 92 V66 L96 58 V30 L112 20 L128 30 V58 L144 66 V92 L164 100 V130 L184 140 V400 Z" fill="{brick}" stroke="{G}" stroke-width="3"/>'
        o += "".join(f'<path d="M40 {y} H184" stroke="{stone}" stroke-width="5"/>' for y in (180, 240, 300, 360))
        o += "".join(f'<path d="M{48+i*20} {196} v26" stroke="{mix(brick, "#000000", .2)}" stroke-width="2"/>' for i in range(7))
        o += f'<path d="M60 130 H164 M80 92 H144" stroke="{G}" stroke-width="3"/><circle cx="112" cy="250" r="22" fill="none" stroke="{G}" stroke-width="3"/>'
        o += "</g>"
        b += o
    b += f'<path d="M240 400 H360 L350 380 H250 Z M250 380 H350 L342 362 H258 Z" fill="{stone}" stroke="{G}" stroke-width="2"/>'
    b += f'<path d="M20 404 H580" stroke="{c["Gd"]}" stroke-width="8" stroke-linecap="round"/>'
    return svg(600, 420, b)

def masjid(c, tile):
    """Masjid berkubah (Aceh/Islami): kubah besar, kubah samping, dua menara."""
    P, G, K = c["P"], c["G"], c["K"]
    wall = "#f6f1e6" if not c["dark"] else c["Gl"]
    dome = K if not c["dark"] else c["Pd"]
    b = ""
    for x in (60, 540):
        b += f'<rect x="{x-14}" y="120" width="28" height="280" fill="{wall}" stroke="{G}" stroke-width="3"/><path d="M{x-14} 120 C{x-14} 90 {x} 70 {x} 56 C{x} 70 {x+14} 90 {x+14} 120 Z" fill="{dome}" stroke="{G}" stroke-width="2"/>'
        b += f'<path d="M{x-20} 180 H{x+20} M{x-20} 300 H{x+20}" stroke="{G}" stroke-width="4"/>'
    b += f'<rect x="110" y="250" width="380" height="150" fill="{wall}" stroke="{G}" stroke-width="3"/>'
    b += "".join(f'<path d="M{x} 400 V320 A26 26 0 0 1 {x+52} 320 V400" fill="{P}" opacity=".85" stroke="{G}" stroke-width="2"/>' for x in (134, 214, 334, 414))
    for x, r in ((170, 50), (430, 50)):
        b += f'<path d="M{x-r} 250 C{x-r} {250-r*1.3} {x} {250-r*1.6} {x} {250-r*1.9} C{x} {250-r*1.6} {x+r} {250-r*1.3} {x+r} 250 Z" fill="{dome}" stroke="{G}" stroke-width="3"/>'
    b += f'<path d="M210 250 C210 140 300 100 300 60 C300 100 390 140 390 250 Z" fill="{dome}" stroke="{G}" stroke-width="4"/>'
    b += f'<path d="M300 60 V30" stroke="{G}" stroke-width="3"/><path d="M292 26 a11 11 0 1 0 16 -10 a8 8 0 1 1 -16 10" fill="{G}"/>'
    b += f'<path d="M110 250 H490" stroke="{G}" stroke-width="5"/><path d="M40 404 H560" stroke="{c["Gd"]}" stroke-width="8" stroke-linecap="round"/>'
    return svg(600, 420, b)

def rumah_limas(c, tile):
    """Rumah limas Palembang: atap limas bertingkat dengan simbar, kekijing berundak."""
    G, K, A = c["G"], c["K"], c["A"]
    roof = mix(c["P"], "#000000", .25) if not c["dark"] else c["Pd"]
    d = pattern(c, tile, "m", .35)
    b = f'<path d="M60 200 L170 110 H430 L540 200 Z" fill="{roof}" stroke="{G}" stroke-width="4"/>'
    b += f'<path d="M170 110 L230 40 H370 L430 110 Z" fill="{mix(roof, "#ffffff", .1)}" stroke="{G}" stroke-width="4"/>'
    b += "".join(f'<path d="M{x} 40 l-10 -26 l10 8 l10 -8 z" fill="{G}"/>' for x in (230, 300, 370))
    b += f'<path d="M80 200 H520 V300 H80 Z" fill="{A}" stroke="{G}" stroke-width="4"/><path d="M80 200 H520 V300 H80 Z" fill="url(#m)" opacity=".7"/>'
    b += f'<path d="M60 300 H540 V330 H60 Z" fill="{mix(A, "#000000", .25)}" stroke="{G}" stroke-width="3"/>'
    b += "".join(f'<rect x="{x}" y="222" width="34" height="56" fill="{K}" stroke="{G}" stroke-width="3"/>' for x in (120, 200, 366, 446))
    b += f'<rect x="270" y="222" width="60" height="78" fill="{c["P"]}" stroke="{G}" stroke-width="3"/>'
    b += "".join(f'<rect x="{x}" y="330" width="12" height="64" fill="{mix(roof, "#ffffff", .2)}"/>' for x in range(80, 540, 52))
    b += f'<path d="M40 400 H560" stroke="{c["Gd"]}" stroke-width="8" stroke-linecap="round"/>'
    return svg(600, 420, b, d)

def siger_besar(c, tile):
    """Siger Lampung: mahkota emas 9 lekuk bertatah, di atas bantal hias."""
    G, Gd, A, P = c["G"], c["Gd"], c["A"], c["P"]
    d = lg("sg", c["Gl"], Gd)
    b = f'<path d="M60 330 Q300 400 540 330 L520 370 Q300 430 80 370 Z" fill="{P}" stroke="{G}" stroke-width="4"/>'
    b += f'<path d="M80 330 Q300 380 520 330" fill="none" stroke="{G}" stroke-width="3" stroke-dasharray="4 6"/>'
    for i in range(9):
        k = i - 4
        x = 300 + k * 52
        h = 260 - abs(k) * 34
        yb = 336 + abs(k) * 6
        b += f'<path d="M{x-30} {yb} C{x-26} {yb-h*.5} {x-12} {yb-h*.8} {x} {yb-h} C{x+12} {yb-h*.8} {x+26} {yb-h*.5} {x+30} {yb} Z" fill="url(#sg)" stroke="{Gd}" stroke-width="3"/>'
        b += f'<path d="M{x} {yb-h+30} V{yb-14}" stroke="{Gd}" stroke-width="2"/>'
        b += f'<circle cx="{x}" cy="{yb-h*.45}" r="8" fill="{A}" stroke="{Gd}" stroke-width="2"/>'
        b += f'<circle cx="{x}" cy="{yb-h-6}" r="7" fill="{G}" stroke="{Gd}" stroke-width="2"/>'
    return svg(600, 420, b, d)

def phinisi(c, tile):
    """Kapal pinisi berlayar di atas ombak."""
    P, G = c["P"], c["G"]
    sail = "#f4ead6" if not c["dark"] else c["Gl"]
    hull = mix(P, "#000000", .3)
    b = ""
    for mx, top in ((230, 30), (370, 50)):
        b += f'<path d="M{mx} {top} V300" stroke="{hull}" stroke-width="6"/>'
        b += f'<path d="M{mx+6} {top+20} C{mx+80} {top+70} {mx+100} {top+160} {mx+90} 250 L{mx+6} 250 Z" fill="{sail}" stroke="{G}" stroke-width="3"/>'
        b += f'<path d="M{mx-6} {top+40} C{mx-60} {top+90} {mx-76} {top+170} {mx-66} 250 L{mx-6} 250 Z" fill="{mix(sail, "#000000", .05)}" stroke="{G}" stroke-width="3"/>'
        b += f'<path d="M{mx} {top} l30 10 l-30 10 z" fill="{c["A"]}"/>'
    b += f'<path d="M110 120 L230 70 M490 150 L370 80" stroke="{hull}" stroke-width="2"/>'
    b += f'<path d="M60 290 H540 C520 330 480 350 440 354 H170 C120 350 80 330 60 290 Z" fill="{hull}" stroke="{G}" stroke-width="4"/>'
    b += f'<path d="M90 310 H510" stroke="{G}" stroke-width="4"/><path d="M540 290 L590 270 L560 300 Z" fill="{hull}"/>'
    wave = "#5b9bb8" if not c["dark"] else c["Gd"]
    b += f'<path d="M0 370 Q50 340 100 370 T200 370 T300 370 T400 370 T500 370 T600 370 V420 H0 Z" fill="{wave}" opacity=".8"/>'
    b += f'<path d="M0 392 Q50 362 100 392 T200 392 T300 392 T400 392 T500 392 T600 392" fill="none" stroke="#ffffff" stroke-width="3" opacity=".7"/>'
    return svg(600, 420, b)

def betang(c, tile):
    """Rumah betang Dayak: rumah panjang berkolong tinggi, tangga kayu, motif di dinding."""
    G, K, A = c["G"], c["K"], c["A"]
    roof = "#3b2f25" if not c["dark"] else c["Pd"]
    wood = "#8a5a35" if not c["dark"] else c["P"]
    d = pattern(c, tile, "m", .3)
    b = f'<path d="M20 160 L80 90 H520 L580 160 Z" fill="{roof}" stroke="{G}" stroke-width="4"/>'
    b += f'<path d="M60 90 L40 60 M540 90 L560 60" stroke="{G}" stroke-width="6" stroke-linecap="round"/>'
    b += f'<path d="M50 160 H550 V250 H50 Z" fill="{wood}" stroke="{G}" stroke-width="4"/><path d="M50 160 H550 V250 H50 Z" fill="url(#m)" opacity=".6"/>'
    b += "".join(f'<rect x="{x}" y="180" width="30" height="40" fill="{K}" stroke="{G}" stroke-width="2"/>' for x in range(80, 540, 70))
    b += "".join(f'<rect x="{x}" y="250" width="12" height="150" fill="{mix(wood, "#000000", .3)}"/>' for x in range(64, 550, 40))
    b += f'<path d="M300 250 L250 400" stroke="{mix(wood, "#000000", .2)}" stroke-width="14"/>'
    b += "".join(f'<path d="M{f(296-i*9.5)} {262+i*28} h18" stroke="{G}" stroke-width="3"/>' for i in range(5))
    b += f'<path d="M20 404 H580" stroke="{c["Gd"]}" stroke-width="8" stroke-linecap="round"/>'
    return svg(600, 420, b, d)

def honai(c, tile):
    """Honai Papua: rumah bulat beratap jerami, berkelompok."""
    G = c["G"]
    straw = ("#8c6a3f", "#a88550", "#c4a46a") if not c["dark"] else (c["Pd"], c["P"], c["Gd"])
    wall = "#6d4c2f" if not c["dark"] else c["Pd"]
    b = ""
    for (x, s) in ((150, .8), (450, .8), (300, 1.05)):
        w, h = 130 * s, 150 * s
        y = 400 - h * .55
        b += f'<rect x="{f(x-w*.55)}" y="{f(y)}" width="{f(w*1.1)}" height="{f(h*.55)}" fill="{wall}" stroke="{G}" stroke-width="3"/>'
        b += f'<rect x="{f(x-w*.12)}" y="{f(400-h*.32)}" width="{f(w*.24)}" height="{f(h*.32)}" fill="#2a1b10"/>'
        b += f'<path d="M{f(x-w*.75)} {f(y+8)} C{f(x-w*.7)} {f(y-h*.7)} {f(x+w*.7)} {f(y-h*.7)} {f(x+w*.75)} {f(y+8)} Z" fill="{straw[0]}" stroke="{G}" stroke-width="3"/>'
        for k in range(1, 6):
            yy = y + 6 - k * h * .1
            b += f'<path d="M{f(x-w*(.74-k*.09))} {f(yy)} Q{x} {f(yy+10)} {f(x+w*(.74-k*.09))} {f(yy)}" fill="none" stroke="{straw[k % 3]}" stroke-width="4"/>'
    b += f'<path d="M20 404 H580" stroke="{c["Gd"]}" stroke-width="8" stroke-linecap="round"/>'
    return svg(600, 420, b)

def sasando(c, tile):
    """Sasando Rote: tabung bambu berdawai dalam wadah daun lontar melengkung."""
    G, K = c["G"], c["K"]
    leaf = ("#c8a35a", "#a8853f") if not c["dark"] else (c["G"], c["Gd"])
    b = ""
    for i in range(13):
        a = math.radians(200 + i * 11.5)
        x2, y2 = 300 + math.cos(a) * 230, 330 + math.sin(a) * 300
        b += f'<path d="M300 380 Q{f((300+x2)/2+math.cos(a)*30)} {f((380+y2)/2)} {f(x2)} {f(y2)}" stroke="{leaf[i%2]}" stroke-width="22" fill="none" stroke-linecap="round" opacity=".95"/>'
        b += f'<path d="M300 380 Q{f((300+x2)/2+math.cos(a)*30)} {f((380+y2)/2)} {f(x2)} {f(y2)}" stroke="{mix(leaf[1], "#000000", .3)}" stroke-width="1.5" fill="none"/>'
    b += f'<rect x="272" y="60" width="56" height="330" rx="20" fill="{"#d9c28a" if not c["dark"] else c["Gl"]}" stroke="{G}" stroke-width="4"/>'
    b += "".join(f'<path d="M{280+i*5} 76 V376" stroke="{K}" stroke-width="1.2"/>' for i in range(9))
    b += f'<rect x="268" y="84" width="64" height="10" fill="{c["A"]}"/><rect x="268" y="356" width="64" height="10" fill="{c["A"]}"/>'
    b += f'<path d="M150 404 H450" stroke="{c["Gd"]}" stroke-width="8" stroke-linecap="round"/>'
    return svg(600, 420, b)

def kora(c, tile):
    """Perahu kora-kora Maluku dengan pendayung & umbul-umbul di atas ombak."""
    P, G, A = c["P"], c["G"], c["A"]
    hull = mix(P, "#000000", .25)
    b = f'<path d="M30 250 C120 300 480 300 570 250 C560 300 520 330 460 336 H140 C80 330 40 300 30 250 Z" fill="{hull}" stroke="{G}" stroke-width="4"/>'
    b += f'<path d="M30 250 L10 200 M570 250 L590 200" stroke="{hull}" stroke-width="10" stroke-linecap="round"/>'
    b += "".join(f'<path d="M{x} 300 l{12-((x//30)%2)*24} 70" stroke="{G}" stroke-width="4"/>' for x in range(110, 500, 36))
    b += "".join(f'<circle cx="{x}" cy="248" r="11" fill="{c["K"]}"/><path d="M{x-10} 282 V262 Q{x} 252 {x+10} 262 V282" fill="{A}"/>' for x in range(120, 500, 54))
    for x in (150, 300, 450):
        b += f'<path d="M{x} 280 V120" stroke="{hull}" stroke-width="4"/><path d="M{x} 124 C{x+30} 110 {x+50} 140 {x+80} 126 V170 C{x+50} 184 {x+30} 154 {x} 168 Z" fill="{A if x != 300 else G}"/>'
    wave = "#3c8db0" if not c["dark"] else c["Gd"]
    b += f'<path d="M0 360 Q50 330 100 360 T200 360 T300 360 T400 360 T500 360 T600 360 V420 H0 Z" fill="{wave}" opacity=".85"/>'
    b += f'<path d="M0 384 Q50 354 100 384 T200 384 T300 384 T400 384 T500 384 T600 384" fill="none" stroke="#ffffff" stroke-width="3" opacity=".7"/>'
    return svg(600, 420, b)

def rumah_panggung(c, tile, gunung=True):
    """Rumah panggung kayu (Minahasa/Bugis/Betawi) dengan gunung di belakang."""
    G, K = c["G"], c["K"]
    wood = "#8a5a35" if not c["dark"] else c["P"]
    roof = "#5b3a24" if not c["dark"] else c["Pd"]
    b = ""
    if gunung:
        m = ("#7fa07a", "#a9c39e") if not c["dark"] else (c["Pd"], c["P"])
        b += f'<path d="M0 300 L160 90 L250 190 L360 40 L600 300 Z" fill="{m[1]}" opacity=".7"/><path d="M330 76 L360 40 L392 80 L372 74 L360 86 L346 74 Z" fill="#ffffff" opacity=".8"/>'
        b += f'<path d="M0 330 L120 200 L240 300 L420 160 L600 330 Z" fill="{m[0]}" opacity=".7"/>'
    b += f'<path d="M110 170 L300 70 L490 170 Z" fill="{roof}" stroke="{G}" stroke-width="4"/>'
    b += f'<path d="M250 110 L300 84 L350 110 Z" fill="{c["A"]}"/>'
    b += f'<path d="M140 170 H460 V270 H140 Z" fill="{wood}" stroke="{G}" stroke-width="4"/>'
    b += "".join(f'<path d="M140 {y} H460" stroke="{mix(wood, "#000000", .2)}" stroke-width="2"/>' for y in range(184, 270, 14))
    b += "".join(f'<rect x="{x}" y="192" width="40" height="52" fill="{K}" stroke="{G}" stroke-width="3"/>' for x in (170, 390))
    b += f'<rect x="276" y="196" width="48" height="74" fill="{mix(wood, "#000000", .35)}" stroke="{G}" stroke-width="3"/>'
    b += "".join(f'<rect x="{x}" y="270" width="12" height="128" fill="{mix(wood, "#000000", .3)}"/>' for x in range(150, 460, 50))
    for s in (-1, 1):
        b += f'<path d="M{300+s*30} 270 L{300+s*110} 398" stroke="{mix(wood, "#000000", .2)}" stroke-width="10"/>'
    b += f'<path d="M20 404 H580" stroke="{c["Gd"]}" stroke-width="8" stroke-linecap="round"/>'
    return svg(600, 420, b)

def rumah_kebaya(c, tile):
    """Rumah kebaya Betawi: atap pelana, teras luas berpagar, lisplang gigi balang."""
    G, K, A = c["G"], c["K"], c["A"]
    roof = "#8e3b2a" if not c["dark"] else c["Pd"]
    wall = "#f3e6c8" if not c["dark"] else c["Gl"]
    b = f'<path d="M40 190 L180 70 H420 L560 190 Z" fill="{roof}" stroke="{G}" stroke-width="4"/>'
    b += "".join(f'<path d="M{x} 190 l10 18 l10 -18" fill="{A}" stroke="{G}" stroke-width="1.5"/>' for x in range(60, 540, 20))
    b += f'<path d="M110 208 H490 V360 H110 Z" fill="{wall}" stroke="{G}" stroke-width="3"/>'
    b += "".join(f'<path d="M{x} 236 h40 v80 h-40 z M{x+20} 236 v80" fill="{c["P"]}" stroke="{G}" stroke-width="3"/>' for x in (150, 410))
    b += f'<path d="M270 230 h60 v130 h-60 z" fill="{c["P"]}" stroke="{G}" stroke-width="3"/>'
    b += f'<path d="M60 360 H540 V384 H60 Z" fill="{mix(roof, "#ffffff", .2)}"/>'
    b += "".join(f'<path d="M{x} 336 v24" stroke="{K}" stroke-width="3"/>' for x in range(70, 540, 16))
    b += f'<path d="M60 336 H540" stroke="{K}" stroke-width="4"/>'
    b += f'<path d="M30 400 H570" stroke="{c["Gd"]}" stroke-width="8" stroke-linecap="round"/>'
    return svg(600, 420, b)

def istana(c, tile):
    """Istana Melayu (gaya Maimun): lengkung ala Moor & kubah-kubah kecil."""
    G, K, A = c["G"], c["K"], c["A"]
    wall = "#f2e6c4" if not c["dark"] else c["Gl"]
    dome = c["P"] if not c["dark"] else c["Pd"]
    b = f'<rect x="60" y="200" width="480" height="200" fill="{wall}" stroke="{G}" stroke-width="3"/>'
    b += f'<rect x="220" y="130" width="160" height="90" fill="{wall}" stroke="{G}" stroke-width="3"/>'
    for x, r, y in ((300, 60, 130), (110, 32, 200), (490, 32, 200)):
        b += f'<path d="M{x-r} {y} C{x-r} {y-r*1.2} {x} {y-r*1.4} {x} {y-r*1.8} C{x} {y-r*1.4} {x+r} {y-r*1.2} {x+r} {y} Z" fill="{dome}" stroke="{G}" stroke-width="3"/>'
    b += "".join(f'<path d="M{x} 400 V300 Q{x} 260 {x+24} 252 Q{x+48} 260 {x+48} 300 V400" fill="{A}" opacity=".85" stroke="{G}" stroke-width="2.5"/>' for x in (90, 170, 382, 462))
    b += f'<path d="M260 400 V280 Q260 230 300 222 Q340 230 340 280 V400" fill="{K}" stroke="{G}" stroke-width="3"/>'
    b += f'<path d="M60 220 H540" stroke="{G}" stroke-width="5"/>'
    b += f'<path d="M30 404 H570" stroke="{c["Gd"]}" stroke-width="8" stroke-linecap="round"/>'
    return svg(600, 420, b)

def saung(c, tile):
    """Saung bambu Sunda di tepi sawah berundak + pegunungan."""
    G = c["G"]
    m = ("#7fa07a", "#a9c39e", "#5e8a5a") if not c["dark"] else (c["Pd"], c["P"], c["Gd"])
    b = f'<path d="M0 230 L140 80 L260 180 L380 60 L600 240 Z" fill="{m[1]}" opacity=".8"/>'
    for i in range(4):
        y = 260 + i * 36
        b += f'<path d="M0 {y} Q300 {y-30} 600 {y} V{y+36} H0 Z" fill="{m[i % 2 * 2 if i % 2 else 0]}" opacity="{.9 - i * .1:.2f}"/>'
        b += "".join(f'<path d="M{x} {y+18} l6 -12 M{x+10} {y+18} l-4 -12" stroke="{mix(m[2], "#ffffff", .3)}" stroke-width="2"/>' for x in range(20, 600, 46))
    roof = "#7b5a2e" if not c["dark"] else c["Pd"]
    b += f'<path d="M330 200 L420 140 L510 200 Z" fill="{roof}" stroke="{G}" stroke-width="3"/>'
    b += "".join(f'<path d="M{x} 200 V290" stroke="#b58b4c" stroke-width="7"/>' for x in (350, 420, 490))
    b += f'<path d="M340 250 H500" stroke="#b58b4c" stroke-width="8"/>'
    return svg(600, 420, b)

def pemandangan(c, tile):
    """Alam: bukit berlapis, deretan pepohonan, matahari & burung."""
    G = c["G"]
    tr = ["#2f5d3a", "#3f7a49", "#5c9a5e", "#86b77a"] if not c["dark"] else [c["Pd"], c["P"], c["Gd"], c["G"]]
    b = f'<circle cx="430" cy="110" r="56" fill="{G}" opacity=".55"/>'
    b += f'<path d="M0 260 C120 160 220 200 300 170 C380 140 480 180 600 150 V420 H0 Z" fill="{tr[3]}" opacity=".55"/>'
    b += f'<path d="M0 320 C140 250 260 300 360 260 C460 220 520 270 600 250 V420 H0 Z" fill="{tr[2]}" opacity=".7"/>'
    import random
    rng = random.Random(2)
    for i in range(16):
        x = 20 + i * 38 + rng.uniform(-10, 10)
        h = rng.uniform(90, 160)
        y0 = 404
        col = tr[i % 2]
        if i % 3 == 0:  # cemara
            b += f'<path d="M{f(x)} {f(y0-h)} L{f(x-24)} {f(y0-30)} H{f(x+24)} Z" fill="{col}"/>'
        else:  # pohon bulat
            b += f'<circle cx="{f(x)}" cy="{f(y0-h*.65)}" r="{f(h*.28)}" fill="{col}"/><circle cx="{f(x-10)}" cy="{f(y0-h*.5)}" r="{f(h*.22)}" fill="{tr[(i+1) % 3]}"/>'
        b += f'<rect x="{f(x-3)}" y="{f(y0-30)}" width="6" height="30" fill="#5b3f26"/>'
    for (x, y) in ((140, 90), (176, 70), (208, 96)):
        b += f'<path d="M{x-12} {y} q6 -8 12 0 q6 -8 12 0" fill="none" stroke="{c["K"]}" stroke-width="2.5"/>'
    b += f'<path d="M0 404 H600" stroke="{c["Gd"]}" stroke-width="8"/>'
    return svg(600, 420, b)

def laut_ikon(c, tile):
    """Laut: gugusan pulau karst, perahu kecil, ombak & matahari."""
    G = c["G"]
    sea = ("#2f7fa5", "#5aa7c7", "#9fd2e3") if not c["dark"] else (c["Pd"], c["P"], c["Gd"])
    isl = "#4f8a5a" if not c["dark"] else c["Gd"]
    b = f'<circle cx="460" cy="120" r="54" fill="{G}" opacity=".55"/>'
    for (x, w, h) in ((90, 90, 170), (190, 70, 120), (480, 110, 190)):
        b += f'<path d="M{x-w} 300 C{x-w} {300-h} {x-w*.2} {300-h*1.1} {x} {300-h} C{x+w*.3} {300-h*1.05} {x+w} {300-h*.8} {x+w} 300 Z" fill="{isl}"/>'
        b += f'<path d="M{x-w*.6} {300-h*.6} q10 -16 20 0 q10 -16 20 0" stroke="{mix(isl, "#ffffff", .3)}" stroke-width="4" fill="none"/>'
    for i, col in enumerate(sea):
        y = 290 + i * 40
        b += f'<path d="M0 {y} Q75 {y-24} 150 {y} T300 {y} T450 {y} T600 {y} V420 H0 Z" fill="{col}" opacity=".9"/>'
    b += f'<path d="M300 300 C330 316 370 316 400 300 L392 316 H308 Z" fill="#7a4f2a"/><path d="M350 300 V240 L388 296 Z" fill="#f4ead6"/>'
    return svg(600, 420, b)

def modern_ikon(c, tile):
    """Modern: lengkung gapura garis tunggal, matahari, daun monoline."""
    G, P = c["G"], c["P"]
    b = f'<path d="M150 404 V200 A150 150 0 0 1 450 200 V404" fill="none" stroke="{G}" stroke-width="6"/>'
    b += f'<path d="M180 404 V210 A120 120 0 0 1 420 210 V404" fill="none" stroke="{P}" stroke-width="2" opacity=".6"/>'
    b += f'<circle cx="300" cy="230" r="54" fill="{G}" opacity=".35"/>'
    for s in (-1, 1):
        b += f'<g transform="translate({300+s*170} 404) scale({s} 1)">'
        b += f'<path d="M0 0 C-10 -80 10 -160 -20 -240" fill="none" stroke="{P}" stroke-width="3"/>'
        b += "".join(f'<path d="M{f(-k*2)} {-40-k*36} c-30 -6 -44 -26 -44 -40 c20 4 38 20 44 40 z" fill="none" stroke="{P}" stroke-width="2.5"/>' for k in range(5))
        b += "</g>"
    b += f'<path d="M80 404 H520" stroke="{G}" stroke-width="4"/>'
    return svg(600, 420, b)

# ═══ ATAS (800×220, hiasan gantung) ═════════════════════════════════════════

def rope(y0=24, sag=120):
    return f"M10 {y0} Q400 {y0+sag*2} 790 {y0}"

def on_rope(n, y0=24, sag=120):
    pts = []
    for i in range(n):
        t = (i + .5) / n
        x = (1 - t) ** 2 * 10 + 2 * (1 - t) * t * 400 + t * t * 790
        y = (1 - t) ** 2 * y0 + 2 * (1 - t) * t * (y0 + sag * 2) + t * t * y0
        pts.append((x, y))
    return pts

def janur(c, tile):
    """Janur kuning (Jawa): untaian anyaman janur melengkung + kembar mayang."""
    jan, jand = ("#e8d78a", "#c3ab52") if not c["dark"] else (c["Gl"], c["Gd"])
    b = f'<path d="{rope(30, 70)}" stroke="{jand}" stroke-width="6" fill="none"/>'
    for i, (x, y) in enumerate(on_rope(11, 30, 70)):
        for k in range(-2, 3):
            b += f'<path d="M{f(x)} {f(y)} C{f(x+k*10)} {f(y+30)} {f(x+k*18)} {f(y+50)} {f(x+k*8)} {f(y+80+abs(k)*-6)}" stroke="{jan if (k+i) % 2 else jand}" stroke-width="5" fill="none" stroke-linecap="round"/>'
        b += f'<path d="M{f(x-9)} {f(y+8)} l9 -12 l9 12 l-9 12 z" fill="{jand}"/>'
        if i in (2, 5, 8):
            b += f'<circle cx="{f(x)}" cy="{f(y+92)}" r="8" fill="{c["A"]}"/><circle cx="{f(x)}" cy="{f(y+92)}" r="3.5" fill="{c["G"]}"/>'
    return svg(800, 220, b)

def mega(c, tile):
    """Mega mendung (Cirebon/Sunda): awan berlapis gradasi, ujung melingkar."""
    P = c["P"]
    shades = [mix(P, "#ffffff", t) for t in (0, .2, .4, .6, .8)]
    b = ""
    def cloud(cx, cy, s):
        o = ""
        for k, col in enumerate(shades):
            r = (1 - k * .16) * s
            o += (f'<path d="M{f(cx-r*2.2)} {f(cy)} C{f(cx-r*2.2)} {f(cy-r*.9)} {f(cx-r*1.4)} {f(cy-r*1.1)} {f(cx-r*.9)} {f(cy-r*.7)} '
                  f'C{f(cx-r*.7)} {f(cy-r*1.5)} {f(cx+r*.5)} {f(cy-r*1.6)} {f(cx+r*.7)} {f(cy-r*.8)} '
                  f'C{f(cx+r*1.4)} {f(cy-r*1.1)} {f(cx+r*2.2)} {f(cy-r*.6)} {f(cx+r*2.2)} {f(cy)} '
                  f'C{f(cx+r*1.6)} {f(cy+r*.5)} {f(cx-r*1.6)} {f(cy+r*.5)} {f(cx-r*2.2)} {f(cy)} Z" fill="{col}" stroke="{c["G"] if k == 0 else "none"}" stroke-width="2"/>')
        o += f'<path d="M{f(cx+s*2.2)} {f(cy)} c{f(s*.5)} 0 {f(s*.6)} {f(-s*.6)} {f(s*.2)} {f(-s*.7)}" stroke="{shades[0]}" stroke-width="5" fill="none" stroke-linecap="round"/>'
        return o
    for (x, y, s) in ((150, 120, 46), (400, 90, 60), (650, 120, 46), (275, 160, 30), (525, 160, 30)):
        b += cloud(x, y, s)
    return svg(800, 220, b)

def marawa(c, tile):
    """Marawa Minang: bendera tiga warna hitam-merah-kuning berjajar di tali."""
    cols = ("#1b1b1b", "#b3201b", "#e3b021") if not c["dark"] else (c["Pd"], c["A"], c["G"])
    b = f'<path d="{rope(20, 60)}" stroke="{c["Gd"]}" stroke-width="4" fill="none"/>'
    for i, (x, y) in enumerate(on_rope(9, 20, 60)):
        w, h = 52, 110
        tilt = (i - 4) * 3
        b += f'<g transform="translate({f(x)} {f(y)}) rotate({tilt})">'
        for k, col in enumerate(cols):
            b += f'<path d="M{-w/2 + k*w/3} 0 H{-w/2 + (k+1)*w/3} V{h - (k % 2) * 10} L{-w/2 + k*w/3 + w/6} {h+12} L{-w/2 + k*w/3} {h - (k % 2) * 10} Z" fill="{col}"/>'
        b += f'<rect x="{-w/2}" y="0" width="{w}" height="8" fill="{c["G"]}"/></g>'
    return svg(800, 220, b)

def ombak_atas(c, tile):
    """Ombak bergulung + kerang/cengkih (laut/Maluku)."""
    sea = ("#2f7fa5", "#5aa7c7", "#9fd2e3") if not c["dark"] else (c["Pd"], c["P"], c["Gd"])
    b = ""
    for i, col in enumerate(sea):
        y = 60 + i * 30
        p = f"M0 {y}"
        for k in range(8):
            x = k * 100
            p += f" C{x+30} {y-40} {x+80} {y-40} {x+70} {y-10} C{x+60} {y+6} {x+46} {y-6} {x+56} {y-16}"
            p += f" M{x+100} {y}"
        b += f'<path d="{p}" fill="none" stroke="{col}" stroke-width="{8-i*2}" stroke-linecap="round"/>'
    b += "".join(f'<circle cx="{x}" cy="{y}" r="5" fill="{c["G"]}"/>' for x, y in ((120, 150), (360, 170), (600, 150), (240, 180), (500, 186)))
    return svg(800, 220, b)

def bunting(c, tile):
    """Umbul-umbul segitiga bermotif kain daerah di tali melengkung."""
    d = pattern(c, tile, "m", .4)
    b = f'<path d="{rope(20, 50)}" stroke="{c["Gd"]}" stroke-width="4" fill="none"/>'
    for i, (x, y) in enumerate(on_rope(11, 20, 50)):
        col = (c["P"], c["A"])[i % 2]
        b += f'<path d="M{f(x-30)} {f(y)} L{f(x+30)} {f(y)} L{f(x)} {f(y+90)} Z" fill="{col}" stroke="{c["G"]}" stroke-width="2.5"/>'
        b += f'<path d="M{f(x-30)} {f(y)} L{f(x+30)} {f(y)} L{f(x)} {f(y+90)} Z" fill="url(#m)" opacity=".55"/>'
        b += f'<circle cx="{f(x)}" cy="{f(y+96)}" r="5" fill="{c["G"]}"/>'
    return svg(800, 220, b, d)

def lentera(c, tile):
    """Lentera gantung (Aceh/Melayu/Islami) berderet di tali."""
    G, P = c["G"], c["P"]
    b = f'<path d="{rope(16, 40)}" stroke="{c["Gd"]}" stroke-width="3" fill="none"/>'
    for i, (x, y) in enumerate(on_rope(7, 16, 40)):
        L = 30 + (i % 2) * 34
        b += f'<path d="M{f(x)} {f(y)} V{f(y+L)}" stroke="{c["Gd"]}" stroke-width="2"/>'
        yy = y + L
        b += f'<path d="M{f(x-8)} {f(yy)} H{f(x+8)} L{f(x+20)} {f(yy+14)} V{f(yy+56)} L{f(x+8)} {f(yy+70)} H{f(x-8)} L{f(x-20)} {f(yy+56)} V{f(yy+14)} Z" fill="{P}" stroke="{G}" stroke-width="2.5"/>'
        b += f'<ellipse cx="{f(x)}" cy="{f(yy+35)}" rx="9" ry="16" fill="{G}" opacity=".9"/>'
        b += f'<path d="M{f(x)} {f(yy+70)} v14" stroke="{G}" stroke-width="3"/><circle cx="{f(x)}" cy="{f(yy+88)}" r="4" fill="{G}"/>'
    return svg(800, 220, b)

def dahan(c, tile):
    """Dahan berdaun melengkung + burung kecil (alam)."""
    leaf = ["#2f5d3a", "#3f7a49", "#5c9a5e", "#86b77a"] if not c["dark"] else [c["Pd"], c["P"], c["Gd"], c["G"]]
    b = f'<path d="M0 40 C200 70 300 150 420 120 C540 90 640 40 800 60" stroke="#6b4a2b" stroke-width="7" fill="none" stroke-linecap="round"/>'
    import random
    rng = random.Random(9)
    for i in range(46):
        t = i / 45
        x = t * 800
        y = 40 + math.sin(t * math.pi) * 90 - (20 if .6 < t < .9 else 0) + rng.uniform(-8, 8)
        a = rng.uniform(40, 140) * (1 if i % 2 else -1)
        L = rng.uniform(30, 46)
        b += f'<g transform="translate({f(x)} {f(y)}) rotate({f(a)})"><path d="M0 0 C{f(L*.3)} {f(-L*.3)} {f(L*.7)} {f(-L*.3)} {f(L)} 0 C{f(L*.7)} {f(L*.3)} {f(L*.3)} {f(L*.3)} 0 0 Z" fill="{leaf[i % 4]}"/></g>'
    for (x, y) in ((260, 90), (560, 70)):
        b += f'<g transform="translate({x} {y})"><ellipse rx="16" ry="11" fill="{c["A"] if not c["dark"] else c["G"]}"/><circle cx="14" cy="-8" r="8" fill="{c["A"] if not c["dark"] else c["G"]}"/><path d="M21 -8 l8 2 l-8 3 z" fill="#e0a12a"/><path d="M-14 0 l-14 -6 l2 10 z" fill="{c["A"] if not c["dark"] else c["G"]}"/></g>'
    return svg(800, 220, b)

def garis_modern(c, tile):
    """Modern: lengkung garis tipis berlapis + titik emas."""
    G, P = c["G"], c["P"]
    b = "".join(f'<path d="M{40+k*20} 20 Q400 {200-k*24} {760-k*20} 20" fill="none" stroke="{G if k % 2 == 0 else P}" stroke-width="{3 - k*.4:.1f}" opacity="{.9-k*.12:.2f}"/>' for k in range(5))
    b += "".join(f'<circle cx="{f(x)}" cy="{f(y+8)}" r="5" fill="{G}"/>' for x, y in on_rope(9, 20, 90))
    return svg(800, 220, b)

def untaian_bunga(c, tile):
    """Untaian bunga kecil (Minahasa / romantis)."""
    cols = ("#e85d75", "#f29bb0", "#f7c948", "#ffffff") if not c["dark"] else (c["G"], c["Gl"], c["Gd"], c["B"])
    b = f'<path d="{rope(20, 70)}" stroke="#4f7d46" stroke-width="4" fill="none"/>'
    for i, (x, y) in enumerate(on_rope(15, 20, 70)):
        col = cols[i % 4]
        b += f'<g transform="translate({f(x)} {f(y)})">' + "".join(f'<ellipse cx="0" cy="-10" rx="5" ry="10" fill="{col}" stroke="{mix(col, "#000000", .2)}" stroke-width="1" transform="rotate({k*45})"/>' for k in range(8)) + f'<circle r="5" fill="{cols[2]}"/></g>'
        b += f'<g transform="translate({f(x+20)} {f(y+6)}) rotate({30 if i % 2 else -30})"><path d="M0 0 C8 -6 20 -6 26 0 C20 6 8 6 0 0 Z" fill="#4f7d46"/></g>'
    return svg(800, 220, b)

# ═══ SUDUT & LAMBANG (motif kain) ═══════════════════════════════════════════

def sudut_motif(c, tile):
    """Kipas seperempat lingkaran bermotif kain + bingkai emas berlapis + ujung rumbai."""
    G = c["G"]
    d = pattern(c, tile, "m", .55) + f'<clipPath id="q"><path d="M0 0 H300 A300 300 0 0 1 0 300 Z"/></clipPath>'
    b = f'<path d="M0 0 H300 A300 300 0 0 1 0 300 Z" fill="{c["P"]}" opacity=".9"/>'
    b += f'<rect width="300" height="300" fill="url(#m)" clip-path="url(#q)"/>'
    for r, w in ((300, 6), (282, 2.5), (240, 2)):
        b += f'<path d="M{r} 0 A{r} {r} 0 0 1 0 {r}" fill="none" stroke="{G}" stroke-width="{w}"/>'
    for k in range(13):
        a = math.radians(k * 7.5)
        x, y = math.cos(a) * 300, math.sin(a) * 300
        b += f'<path d="M{f(x)} {f(y)} l{f(math.cos(a)*22)} {f(math.sin(a)*22)}" stroke="{G}" stroke-width="3"/><circle cx="{f(x+math.cos(a)*28)}" cy="{f(y+math.sin(a)*28)}" r="6" fill="{c["A"] if k % 2 else G}"/>'
    return svg(340, 340, b, d)

def sudut_daun(c, tile):
    """Sudut dedaunan (alam): daun-daun lebar menjulur dari pojok."""
    leaf = ["#2f5d3a", "#3f7a49", "#5c9a5e", "#86b77a"] if not c["dark"] else [c["Pd"], c["P"], c["Gd"], c["G"]]
    b = ""
    import random
    rng = random.Random(6)
    for i in range(22):
        a = rng.uniform(-5, 95)
        L = rng.uniform(120, 260)
        W = L * rng.uniform(.22, .32)
        b += f'<g transform="rotate({f(a)})"><path d="M0 0 C{f(L*.3)} {f(-W)} {f(L*.75)} {f(-W*.9)} {f(L)} 0 C{f(L*.75)} {f(W*.9)} {f(L*.3)} {f(W)} 0 0 Z" fill="{leaf[i % 4]}"/><path d="M6 0 H{f(L*.92)}" stroke="{mix(leaf[0], "#000000", .2)}" stroke-width="1.5" opacity=".6"/></g>'
    return svg(340, 340, b)

def sudut_karang(c, tile):
    """Sudut laut: kerang, bintang laut & rumput laut."""
    G = c["G"]
    weed = "#3e8a6e" if not c["dark"] else c["P"]
    b = "".join(f'<path d="M0 {y} C60 {y-30} 100 {y+30} {160+y//3} {y-20}" stroke="{weed}" stroke-width="8" fill="none" stroke-linecap="round" opacity=".8"/>' for y in (40, 90, 140))
    b += f'<g transform="translate(90 200)">' + "".join(f'<path d="M0 0 L{f(math.cos(math.radians(k*72-90))*44)} {f(math.sin(math.radians(k*72-90))*44)}" stroke="#e8735a" stroke-width="16" stroke-linecap="round"/>' for k in range(5)) + "</g>"
    b += f'<g transform="translate(220 80)"><path d="M0 40 C-40 0 -30 -40 0 -44 C30 -40 40 0 0 40 Z" fill="#f2d3b3" stroke="{G}" stroke-width="2"/>' + "".join(f'<path d="M0 36 L{-24+k*8} -36" stroke="{G}" stroke-width="1.5"/>' for k in range(7)) + "</g>"
    return svg(340, 340, b)

def sudut_garis(c, tile):
    G, P = c["G"], c["P"]
    b = "".join(f'<path d="M{20+k*22} 0 V{120-k*10} A{200-k*22} {200-k*22} 0 0 0 {220} {320-k*22}" fill="none" stroke="{G if k % 2 == 0 else P}" stroke-width="{3-k*.4:.1f}"/>' for k in range(5))
    b += "".join(f'<circle cx="{20+k*22}" cy="{124-k*10}" r="4" fill="{G}"/>' for k in range(5))
    return svg(340, 340, b)

def lambang(c, tile, emblem):
    """Medali: lingkaran motif kain + cincin emas bergerigi + lambang daerah."""
    G = c["G"]
    d = pattern(c, tile, "m", .5) + '<clipPath id="o"><circle cx="150" cy="150" r="118"/></clipPath>'
    b = f'<circle cx="150" cy="150" r="140" fill="{c["P"]}"/>'
    b += "".join(f'<path d="M150 4 l7 14 h-14 z" fill="{G}" transform="rotate({k*15} 150 150)"/>' for k in range(24))
    b += f'<rect width="300" height="300" fill="url(#m)" clip-path="url(#o)" opacity=".9"/>'
    b += f'<circle cx="150" cy="150" r="118" fill="none" stroke="{G}" stroke-width="5"/><circle cx="150" cy="150" r="132" fill="none" stroke="{G}" stroke-width="2"/>'
    b += f'<circle cx="150" cy="150" r="64" fill="{c["B"] if not c["dark"] else c["Pd"]}" stroke="{G}" stroke-width="4"/>'
    pal = dict(primary=c["P"], gold=G, bg=c["B"], card=c["B"], ink=c["K"], accent=c["A"])
    b += f'<g transform="translate(150 158) scale(.62)">{M.EMBLEMS[emblem](pal, 0, 0)}</g>'
    return svg(300, 300, b, d)

# ═══ SET PER DAERAH ═════════════════════════════════════════════════════════
# tokoh: (fungsi, letak "bawah"|"atas", masuk, gerak, lebar%)
# ikon : (fungsi, masuk, gerak)   atas: (fungsi, masuk, gerak)   sudut: fungsi
SETS = {
    "jawa":      dict(tokoh=(wayang, "bawah", "geser", "goyang", 30), ikon=(gunungan, "tumbuh", "goyang"), atas=(janur, "turun", "goyang"), sudut=sudut_motif, emblem="gunungan", tile="kawung"),
    "sunda":     dict(tokoh=(golek, "bawah", "naik", "goyang", 30), ikon=(saung, "naik", "none"), atas=(mega, "geser", "melayang"), sudut=sudut_motif, emblem="kujang", tile="mega-mendung"),
    "minang":    dict(tokoh=(rangkiang, "bawah", "naik", "none", 30), ikon=(rumah_gadang, "naik", "none"), atas=(marawa, "turun", "goyang"), sudut=sudut_motif, emblem="gonjong", tile="songket-lepus"),
    "batak":     dict(tokoh=(kain_gantung, "bawah", "turun", "goyang", 28), ikon=(rumah_bolon, "naik", "none"), atas=(bunting, "turun", "goyang"), sudut=sudut_motif, emblem="bolon", tile="gorga", garis=True),
    "melayu":    dict(tokoh=(payung, "bawah", "turun", "goyang", 30), ikon=(istana, "naik", "none"), atas=(lentera, "turun", "goyang"), sudut=sudut_motif, emblem="rebung", tile="pucuk-rebung"),
    "maluku":    dict(tokoh=(cengkih, "bawah", "tumbuh", "goyang", 32), ikon=(kora, "geser", "melayang"), atas=(ombak_atas, "geser", "melayang"), sudut=sudut_motif, emblem="tifa", tile="cengkih"),
    "betawi":    dict(tokoh=(ondel, "bawah", "naik", "goyang", 32), ikon=(rumah_kebaya, "naik", "none"), atas=(bunting, "turun", "goyang"), sudut=sudut_motif, emblem="panggung", tile="gigi-balang"),
    "bali":      dict(tokoh=(penjor, "bawah", "tumbuh", "goyang", 34), ikon=(candi_bentar, "naik", "none"), atas=(janur, "turun", "goyang"), sudut=sudut_motif, emblem="candi", tile="patra"),
    "bugis":     dict(tokoh=(layar, "bawah", "naik", "melayang", 30), ikon=(phinisi, "geser", "melayang"), atas=(bunting, "turun", "goyang"), sudut=sudut_motif, emblem="phinisi", tile="balo-renni"),
    "toraja":    dict(tokoh=(tanduk, "bawah", "naik", "none", 26), ikon=(tongkonan, "naik", "none"), atas=(bunting, "turun", "goyang"), sudut=sudut_motif, emblem="tongkonan", tile="pa-tedong"),
    "aceh":      dict(tokoh=(menara, "bawah", "naik", "none", 24), ikon=(masjid, "naik", "none"), atas=(lentera, "turun", "goyang"), sudut=sudut_motif, emblem="rumoh", tile="pintu-aceh"),
    "palembang": dict(tokoh=(payung, "bawah", "turun", "goyang", 30), ikon=(rumah_limas, "naik", "none"), atas=(bunting, "turun", "goyang"), sudut=sudut_motif, emblem="limas", tile="songket-lepus"),
    "lampung":   dict(tokoh=(kain_gantung, "bawah", "turun", "goyang", 28), ikon=(siger_besar, "zoom", "kilau"), atas=(bunting, "turun", "goyang"), sudut=sudut_motif, emblem="siger", tile="tapis-kapal"),
    "dayak":     dict(tokoh=(lambda c, t: burung(c, t, "enggang"), "atas", "geser", "melayang", 40), ikon=(betang, "naik", "none"), atas=(bunting, "turun", "goyang"), sudut=sudut_motif, emblem="enggang", tile="dayak"),
    "papua":     dict(tokoh=(lambda c, t: burung(c, t, "cendrawasih"), "atas", "geser", "melayang", 40), ikon=(honai, "naik", "none"), atas=(bunting, "turun", "goyang"), sudut=sudut_motif, emblem="cenderawasih", tile="asmat"),
    "ntt":       dict(tokoh=(lontar, "bawah", "tumbuh", "goyang", 34), ikon=(sasando, "zoom", "denyut"), atas=(bunting, "turun", "goyang"), sudut=sudut_motif, emblem="ikat", tile="ikat-sumba"),
    "minahasa":  dict(tokoh=(bunga_tangkai, "bawah", "tumbuh", "goyang", 32), ikon=(rumah_panggung, "naik", "none"), atas=(untaian_bunga, "turun", "melayang"), sudut=sudut_motif, emblem="panggung", tile="bunga-tabur"),
    "alam":      dict(tokoh=(pohon, "bawah", "tumbuh", "goyang", 44), ikon=(pemandangan, "naik", "none"), atas=(dahan, "turun", "goyang"), sudut=sudut_daun, emblem=None, tile="bunga-tabur"),
    "laut":      dict(tokoh=(karang, "bawah", "naik", "goyang", 36), ikon=(laut_ikon, "naik", "melayang"), atas=(ombak_atas, "geser", "melayang"), sudut=sudut_karang, emblem=None, tile="ombak"),
    "modern":    dict(tokoh=(pampas, "bawah", "tumbuh", "goyang", 34), ikon=(modern_ikon, "pudar", "none"), atas=(garis_modern, "pudar", "melayang"), sudut=sudut_garis, emblem=None, tile="lurik"),
}
NUANSA = {"Jawa": "jawa", "Sunda": "sunda", "Minang": "minang", "Batak": "batak", "Melayu Deli": "melayu", "Maluku": "maluku",
          "Betawi": "betawi", "Bali": "bali", "Bugis-Makassar": "bugis", "Toraja": "toraja", "Aceh": "aceh", "Palembang": "palembang",
          "Lampung": "lampung", "Dayak": "dayak", "Papua": "papua", "NTT": "ntt", "Minahasa": "minahasa", "Modern": "modern"}
ALAM = {"kebun-teh-ciwidey", "lingkung-gunung-parahyangan", "jatiluwih-subak", "rustic-kraft", "angklung-saung-bambu"}
LAUT = {"pantai-ora-biru", "mutiara-kei", "raja-ampat-biru", "bunaken-laut-teduh"}
# Tema bunga (Islami/Botanical) & kustom: tetap ornamen bunga dari 013.
SKIP = {"lily-garden", "botanical-heritage", "custom-rara-bima"}
# Ikon khusus tema tertentu (lebih tepat dari ikon rumpunnya).
IKON_KHUSUS = {"pendopo-joglo-agung": None, "rumah-kebaya-condet": rumah_kebaya, "sasando-rote": sasando, "angklung-saung-bambu": saung,
               "rumah-panggung-woloan": rumah_panggung, "danau-toba-biru": rumah_bolon, "pesisir-padang-bundo": rumah_gadang}

def slugify(s):
    s = s.lower().replace("'", "")
    return re.sub(r"-+", "-", re.sub(r"[^a-z0-9]+", "-", s)).strip("-")[:48]

# tile & emblem tiap tema Nusantara dari data.py
NUS_INFO = {}
for region, items in NUS.items():
    for it in items:
        NUS_INFO[slugify(it[0])] = (region, it[8], it[9])

THEMES = json.load(open(os.path.join(HERE, "themes.json")))

def pick_set(t):
    if t["slug"] in ALAM:
        return "alam"
    if t["slug"] in LAUT:
        return "laut"
    return NUANSA.get(t["nuansa"] or "")

# ── Penempatan & gerak per tema (10 ornamen) ───────────────────────────────
def placements(S):
    tf, tl, tm, tg, tw = S["tokoh"]
    _, im, ig = S["ikon"]
    _, am, ag = S["atas"]
    geser = lambda side: (f"geser-{side}" if tm == "geser" else tm)
    rows = []
    if tl == "bawah":
        # Sampul: dua sudut motif berayun di atas, dua tokoh mengapit di bawah.
        rows += [("sampul", "sudut", "kiri-atas", -36, -36, 30, 0, False, "ayun-kiri", 400, 1300, "goyang", 8, False, True, 85),
                 ("sampul", "sudut", "kanan-atas", 36, -36, 30, 0, True, "ayun-kanan", 700, 1300, "goyang", 9, False, True, 85),
                 ("sampul", "tokoh", "kiri-bawah", -50, 24, tw, 0, True, geser("kiri"), 1100, 1400, tg, 6, False, True, 100),
                 ("sampul", "tokoh", "kanan-bawah", 50, 24, tw, 0, False, geser("kanan"), 1500, 1400, tg, 7, False, True, 100)]
    else:
        # Burung terbang di sudut atas, sudut motif di bawah.
        rows += [("sampul", "tokoh", "kiri-atas", -30, -18, tw, 0, True, "geser-kiri", 900, 1600, tg, 5, False, True, 100),
                 ("sampul", "tokoh", "kanan-atas", 30, -10, tw - 8, 0, False, "geser-kanan", 1500, 1600, tg, 6, False, True, 100),
                 ("sampul", "sudut", "kiri-bawah", -36, 36, 30, 180, True, "ayun-kiri", 400, 1300, "goyang", 8, False, True, 85),
                 ("sampul", "sudut", "kanan-bawah", 36, 36, 30, 180, False, "ayun-kanan", 700, 1300, "goyang", 9, False, True, 85)]
    lam = "lambang" if S.get("emblem") else "sudut"
    rows += [
        ("mempelai", lam, "tengah-atas", 0, -46, 22 if lam == "lambang" else 26, 0, False, "zoom", 300, 1200, "putar" if lam == "lambang" else "goyang", 36, False, True, 90),
        ("kisah", "tokoh", "kanan-atas" if tl == "bawah" else "kiri-atas", 18 if tl == "bawah" else -18, -10, max(tw - 10, 20), 0, tl != "bawah", geser("kanan") if tl == "bawah" else "geser-kiri", 200, 1300, tg, 7, False, False, 85),
        ("galeri", "atas", "tengah-atas", 0, -78, 84, 0, False, "geser-kiri" if am == "geser" else am, 200, 1400, ag, 6, False, True, 95),
        ("acara", "ikon", "tengah-atas", 0, -6, 92, 0, False, "geser-kiri" if im == "geser" else im, 200, 1600, ig, 9, False, True, 28),
        ("acara", "sudut", "kanan-bawah", 16, 20, 28, 180, False, "ayun-kanan", 600, 1300, "goyang", 8, False, False, 85),
        ("rsvp", "ikon", "tengah-bawah", 0, 12, 80, 0, False, "geser-kanan" if im == "geser" else im, 300, 1600, ig, 9, False, True, 22),
    ]
    return rows

def q(s):
    return "'" + str(s).replace("'", "''") + "'"

def b(v):
    return "TRUE" if v else "FALSE"

vals, done, per_set, themes_done = [], 0, {}, []
for t in THEMES:
    if t["slug"] in SKIP:
        continue
    key = pick_set(t)
    if not key:
        print("lewati (tanpa set):", t["slug"], t["nuansa"])
        continue
    S = dict(SETS[key])
    region, tile, emblem = NUS_INFO.get(t["slug"], (None, S["tile"], S.get("emblem")))
    if key in ("alam", "laut", "modern"):
        tile = S["tile"]
    accent = REGIONS.get(region or "", {}).get("accent") or REGIONS.get(key, {}).get("accent") or "#9c1d1d"
    c = palette(t, accent)
    ikon_fn = IKON_KHUSUS.get(t["slug"], S["ikon"][0]) or S["ikon"][0]
    pieces = {
        "tokoh": S["tokoh"][0](c, tile) if S["tokoh"][0] is not kain_gantung else kain_gantung(c, tile, S.get("garis", False)),
        "ikon": ikon_fn(c, tile),
        "atas": S["atas"][0](c, tile),
        "sudut": S["sudut"](c, tile),
    }
    if S.get("emblem"):
        pieces["lambang"] = lambang(c, tile, emblem if emblem in M.EMBLEMS else S["emblem"])
    for name, content in pieces.items():
        with open(os.path.join(OUT, f'{t["slug"]}-{name}.svg'), "w") as fh:
            fh.write(content)
    for i, (bag, piece, pos, x, y, w, rot, mir, masuk, jeda, dur, gerak, spd, depan, hp, op) in enumerate(placements(S)):
        img = f'{URL}/{t["slug"]}-{piece}.svg'
        vals.append(f"    ({q(t['slug'])}, {q(bag)}, {q(img)}, {q(pos)}, {x}, {y}, {w}, {rot}, {b(mir)}, {q(masuk)}, {jeda}, {dur}, {q(gerak)}, {spd}, {b(depan)}, {b(hp)}, {op}, {(i + 1) * 10})")
    done += 1
    themes_done.append(t["slug"])
    per_set[key] = per_set.get(key, 0) + 1

sql = f"""-- ═══════════════════════════════════════════════════════════════════════════
-- 014_ornamen_adat — ornamen KHAS DAERAH per tema (bukan lagi bunga yang sama
-- beda warna): Jawa = wayang kulit + gunungan + janur, Minang = rumah gadang +
-- rangkiang + marawa, Bali = penjor + candi bentar, Batak = rumah bolon + ulos,
-- Toraja = tongkonan + tanduk kerbau, Betawi = ondel-ondel, Dayak/Papua =
-- enggang/cendrawasih terbang, tema alam = pepohonan, tema laut = karang, dst.
-- Tiap daerah punya GERAK sendiri (wayang bergeser masuk seperti dimainkan
-- dalang, pohon/penjor tumbuh dari bawah, burung melayang, rumah naik, awan
-- & perahu melayang, kain bergoyang).
--
-- WAJIB setelah 013. DIBANGKITKAN scripts/ornamen/adat.py (gambar di
-- public/img/tema/ornamen/adat/, warna mengikuti tiap tema).
-- Ornamen contoh yang BELUM pernah disunting admin (updated_at = created_at)
-- pada tema-tema ini diganti; tema yang ornamennya sudah disunting dilewati.
-- Tema bunga (Lily Garden, Botanical Heritage) & tema kustom tetap memakai
-- ornamen bunga dari 013.
-- ═══════════════════════════════════════════════════════════════════════════

-- Tanpa tabel sementara: aman dijalankan per pernyataan (autocommit) maupun
-- dalam satu transaksi.

DELETE FROM theme_ornaments o
 WHERE o.img LIKE '/img/tema/ornamen/%' AND o.updated_at = o.created_at
   AND o.theme IN ({", ".join(q(s) for s in themes_done)})
   AND NOT EXISTS (SELECT 1 FROM theme_ornaments e WHERE e.theme = o.theme AND e.updated_at <> e.created_at);

INSERT INTO theme_ornaments (theme, bagian, img, posisi, x, y, lebar, rotasi, cermin, masuk, jeda, durasi, gerak, kecepatan, depan, hp, opasitas, urutan)
SELECT a.theme, a.bagian, a.img, a.posisi, a.x, a.y, a.lebar, a.rotasi, a.cermin, a.masuk, a.jeda, a.durasi, a.gerak, a.kecepatan, a.depan, a.hp, a.opasitas, a.urutan
  FROM (VALUES
{",\n".join(vals)}
  ) AS a(theme, bagian, img, posisi, x, y, lebar, rotasi, cermin, masuk, jeda, durasi, gerak, kecepatan, depan, hp, opasitas, urutan)
 WHERE EXISTS (SELECT 1 FROM themes t WHERE t.slug = a.theme)
   AND NOT EXISTS (SELECT 1 FROM theme_ornaments o WHERE o.theme = a.theme)
 ORDER BY a.theme, a.urutan;
"""
with open(os.path.join(ROOT, "migration/014_ornamen_adat.sql"), "w") as fh:
    fh.write(sql)
print("tema:", done, "| per set:", per_set)
