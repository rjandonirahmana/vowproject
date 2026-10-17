#!/usr/bin/env python3
"""Aset tema SUSUR 3D (web/susur.rs): benda-benda yang berdiri di dunia 3D
yang DISUSURI kamera — pohon kelapa, pisang, rumpun bambu, padi, orang-orangan
sawah, penjor, gapura, gebyok pelaminan, kincir air, perahu jukung, kursi tamu…

Tiap SVG = satu papan datar (atau sepasang papan bersilang untuk pohon), latar
transparan, kaki benda di tepi BAWAH kanvas. Bangunan (rumah, lumbung,
pendopo, mercusuar, dermaga) TIDAK di sini — dirakit dari bidang CSS 3D.
Jalankan:  python3 scripts/gerak/susur.py   (keluaran src/web/susur/svg/)
"""
import math
import os
import random

OUT = os.path.join(os.path.dirname(__file__), "..", "..", "src", "web", "susur", "svg")


def f(x):
    return f"{x:.1f}".rstrip("0").rstrip(".")


def tulis(nama, w, h, isi, defs=""):
    svg = f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}">'
    if defs:
        svg += f"<defs>{defs}</defs>"
    svg += isi + "</svg>\n"
    os.makedirs(OUT, exist_ok=True)
    with open(os.path.join(OUT, f"{nama}.svg"), "w") as fh:
        fh.write(svg)


def kuad(p0, p1, p2, t):
    a = (1 - t) ** 2
    b = 2 * (1 - t) * t
    c = t * t
    return (a * p0[0] + b * p1[0] + c * p2[0], a * p0[1] + b * p1[1] + c * p2[1])


# ───────────────────────────── POHON ─────────────────────────────
def pelepah(ox, oy, sudut, panjang, rng, warna=("#3f7d34", "#5e9b3e", "#2f6528")):
    """Satu pelepah kelapa: tulang melengkung terkulai + anak daun dua sisi."""
    a = math.radians(sudut)
    ujung = (ox + math.cos(a) * panjang, oy - math.sin(a) * panjang * .55 + panjang * .38)
    kendali = (ox + math.cos(a) * panjang * .55, oy - math.sin(a) * panjang * .55 - panjang * .18)
    out = [f'<path d="M{f(ox)} {f(oy)}Q{f(kendali[0])} {f(kendali[1])} {f(ujung[0])} {f(ujung[1])}" '
           f'fill="none" stroke="#6d7a2e" stroke-width="7" stroke-linecap="round"/>']
    n = 22
    for i in range(2, n):
        t = i / n
        x, y = kuad((ox, oy), kendali, ujung, t)
        x2, y2 = kuad((ox, oy), kendali, ujung, min(1, t + .02))
        arah = math.atan2(y2 - y, x2 - x)
        pj = (1 - t) * 70 + 20
        for sisi in (-1, 1):
            b = arah + sisi * (1.05 + rng.random() * .25)
            jatuh = 26 * t + 10
            ex = x + math.cos(b) * pj
            ey = y + math.sin(b) * pj + jatuh
            c = warna[(i + (sisi > 0)) % len(warna)]
            out.append(f'<path d="M{f(x)} {f(y)}Q{f((x+ex)/2)} {f((y+ey)/2 - 10)} {f(ex)} {f(ey)}" fill="none" '
                       f'stroke="{c}" stroke-width="{f(7 - 3*t)}" stroke-linecap="round"/>')
    return "".join(out)


def kelapa():
    rng = random.Random(7)
    w, h = 600, 1200
    # Batang melengkung dari kaki (300,1200) ke mahkota (350,250).
    kaki, kend, puncak = (290, 1200), (230, 700), (350, 250)
    kiri, kanan = [], []
    for i in range(41):
        t = i / 40
        x, y = kuad(kaki, kend, puncak, t)
        tebal = 30 - 14 * t
        kiri.append((x - tebal, y))
        kanan.append((x + tebal, y))
    d = "M" + " L".join(f"{f(x)} {f(y)}" for x, y in kiri) + " L" + " L".join(f"{f(x)} {f(y)}" for x, y in reversed(kanan)) + "Z"
    cincin = "".join(
        f'<path d="M{f(kiri[i][0])} {f(kiri[i][1])}Q{f((kiri[i][0]+kanan[i][0])/2)} {f(kiri[i][1]+6)} {f(kanan[i][0])} {f(kanan[i][1])}" '
        f'fill="none" stroke="#7a5e40" stroke-width="2.5" opacity=".55"/>'
        for i in range(2, 40, 2))
    batang = f'<path d="{d}" fill="url(#kb)"/>{cincin}'
    ox, oy = puncak
    daun = "".join(pelepah(ox, oy, s, p, rng) for s, p in
                   [(170, 300), (140, 270), (110, 230), (75, 210), (40, 250), (10, 300), (-15, 260), (195, 250), (60, 180)])
    buah = "".join(f'<circle cx="{f(ox+dx)}" cy="{f(oy+dy)}" r="17" fill="url(#kn)"/>' for dx, dy in [(-14, 22), (12, 26), (0, 40), (24, 10)])
    defs = ('<linearGradient id="kb" x2="1"><stop stop-color="#8a6d4c"/><stop offset=".55" stop-color="#b0916a"/><stop offset="1" stop-color="#7b5f40"/></linearGradient>'
            '<radialGradient id="kn" cx=".35" cy=".35"><stop stop-color="#9fbf54"/><stop offset="1" stop-color="#5b7a22"/></radialGradient>')
    tulis("kelapa", w, h, batang + buah + daun, defs)


def pisang():
    rng = random.Random(3)
    w, h = 520, 760
    out = ['<path d="M236 760C232 640 236 520 248 430L284 430C290 520 296 640 296 760Z" fill="url(#pb)"/>']
    for sudut, pj, robek in [(160, 290, 3), (128, 300, 2), (100, 320, 4), (70, 300, 2), (40, 290, 3), (12, 270, 2), (195, 220, 1)]:
        a = math.radians(sudut)
        ox, oy = 266, 440
        ux, uy = ox + math.cos(a) * pj, oy - math.sin(a) * pj * .9 + 60
        nx, ny = -math.sin(a), -math.cos(a)
        lb = 62
        p1 = (ox + (ux-ox)*.5 + nx*lb, oy + (uy-oy)*.5 + ny*lb - 40)
        p2 = (ox + (ux-ox)*.5 - nx*lb*.6, oy + (uy-oy)*.5 - ny*lb*.6 - 20)
        out.append(f'<path d="M{f(ox)} {f(oy)}Q{f(p1[0])} {f(p1[1])} {f(ux)} {f(uy)}Q{f(p2[0])} {f(p2[1])} {f(ox)} {f(oy)}Z" fill="url(#pd)"/>')
        out.append(f'<path d="M{f(ox)} {f(oy)}Q{f((p1[0]+p2[0])/2)} {f((p1[1]+p2[1])/2)} {f(ux)} {f(uy)}" fill="none" stroke="#c9dc8a" stroke-width="4"/>')
        for k in range(robek):
            t = .3 + .5 * rng.random()
            x, y = kuad((ox, oy), p1, (ux, uy), t)
            out.append(f'<path d="M{f(x)} {f(y)}l{f(-nx*28)} {f(-ny*28)}" stroke="#fdf9ee" stroke-opacity=".0" stroke-width="3"/>')
    # Tandan pisang + jantung.
    out.append('<path d="M286 470Q330 500 330 560" fill="none" stroke="#6d8a2a" stroke-width="7"/>')
    for i in range(5):
        out.append(f'<path d="M{300+i*6} {500+i*14}q22 -6 30 10q-18 4 -30 -10z" fill="#a4c442"/>')
    out.append('<path d="M330 560q-14 30 0 60q14 -30 0 -60z" fill="#7a2f46"/>')
    defs = ('<linearGradient id="pb" x2="1"><stop stop-color="#5f8a3a"/><stop offset=".5" stop-color="#8fb45a"/><stop offset="1" stop-color="#557d33"/></linearGradient>'
            '<linearGradient id="pd" y2="1"><stop stop-color="#79b04a"/><stop offset="1" stop-color="#3d7b30"/></linearGradient>')
    tulis("pisang", w, h, "".join(out), defs)


def bambu():
    rng = random.Random(11)
    w, h = 560, 1100
    out = []
    for i in range(9):
        x0 = 200 + i * 20 + rng.uniform(-6, 6)
        lean = (i - 4) * 34 + rng.uniform(-10, 10)
        top = 120 + rng.uniform(0, 160)
        c = ["#7d9a3a", "#93ad4a", "#6a8a32"][i % 3]
        out.append(f'<path d="M{f(x0)} 1100Q{f(x0+lean*.3)} 600 {f(x0+lean)} {f(top)}" fill="none" stroke="{c}" stroke-width="{f(13 - abs(i-4))}" stroke-linecap="round"/>')
        for k in range(1, 9):
            t = k / 9
            x, y = kuad((x0, 1100), (x0 + lean * .3, 600), (x0 + lean, top), t)
            out.append(f'<path d="M{f(x-7)} {f(y)}h14" stroke="#4f6a24" stroke-width="3"/>')
    for k in range(120):
        x = rng.uniform(40, 520)
        y = rng.uniform(80, 700 - abs(x - 280) * .6)
        a = rng.uniform(-.6, .6) + (math.pi * .15 if x > 280 else -math.pi * .15)
        L = rng.uniform(40, 64)
        ex, ey = x + math.sin(a) * L, y + math.cos(a) * L * .7 + 18
        c = ["#5c8f35", "#7cad46", "#477a2c", "#9cc35a"][k % 4]
        out.append(f'<path d="M{f(x)} {f(y)}Q{f(x+8)} {f((y+ey)/2)} {f(ex)} {f(ey)}Q{f(x-6)} {f((y+ey)/2)} {f(x)} {f(y)}Z" fill="{c}"/>')
    tulis("bambu", w, h, "".join(out))


def padi(kuning=False):
    """Ubin padi 160×70 (diulang mendatar): rumpun bilah + (kuning) malai menunduk."""
    rng = random.Random(5 if kuning else 4)
    w, h = 160, 70
    out = []
    for r in range(4):
        cx = 20 + r * 40 + rng.uniform(-4, 4)
        for k in range(9):
            a = (k - 4) * .16 + rng.uniform(-.06, .06)
            L = rng.uniform(46, 66)
            ex, ey = cx + math.sin(a) * L, h - math.cos(a) * L
            c = (["#c9a83a", "#d8bb4c", "#b8962e"] if kuning else ["#5ea43b", "#79bd4a", "#4a8d30"])[k % 3]
            out.append(f'<path d="M{f(cx)} {h}Q{f(cx + math.sin(a)*L*.4)} {f(h - L*.6)} {f(ex)} {f(ey)}" fill="none" stroke="{c}" stroke-width="2.6" stroke-linecap="round"/>')
        if kuning:
            for k in (-1, 1):
                x0, y0 = cx + k * 6, h - 52
                out.append(f'<path d="M{f(x0)} {f(y0)}q{f(k*12)} -4 {f(k*16)} 14" fill="none" stroke="#e7c95a" stroke-width="5" stroke-linecap="round" stroke-dasharray="3 2"/>')
    tulis("padi-kuning" if kuning else "padi", w, h, "".join(out))


def orang_orangan():
    w, h = 320, 560
    out = [
        '<rect x="152" y="150" width="14" height="410" fill="#8a6a44"/>',
        '<rect x="40" y="215" width="240" height="12" rx="6" fill="#8a6a44"/>',
        # Baju lurik + lengan terentang.
        '<path d="M70 210L250 210L262 246L214 252L206 360L114 360L106 252L58 246Z" fill="url(#lr)"/>',
        # Sarung kotak-kotak.
        '<path d="M112 356L208 356L220 470L100 470Z" fill="url(#sr)"/>',
        # Jerami di ujung lengan & bawah.
        '<path d="M58 240l-26 -10M58 246l-28 6M60 236l-24 -22M262 240l26 -10M262 246l28 6M260 236l24 -22M110 470l-8 26M130 470l-2 28M160 470l0 30M190 470l4 26M212 470l10 24" stroke="#d8b65a" stroke-width="5" stroke-linecap="round"/>',
        # Kepala karung + caping.
        '<circle cx="160" cy="176" r="38" fill="#e3cfa0"/>',
        '<path d="M146 170h6M168 170h6M150 190q10 8 20 0" stroke="#5a4026" stroke-width="4" fill="none" stroke-linecap="round"/>',
        '<path d="M60 160L160 92L260 160Z" fill="url(#cp)"/>',
        '<path d="M60 160Q160 176 260 160" fill="none" stroke="#a88a4a" stroke-width="5"/>',
        '<path d="M100 340l-10 40M222 340l10 40" stroke="#c03c3c" stroke-width="6"/>',
    ]
    defs = ('<pattern id="lr" width="14" height="14" patternUnits="userSpaceOnUse"><rect width="14" height="14" fill="#3e5f8a"/><rect width="5" height="14" fill="#5a7cab"/></pattern>'
            '<pattern id="sr" width="24" height="24" patternUnits="userSpaceOnUse"><rect width="24" height="24" fill="#9b3b36"/><rect width="12" height="24" fill="#b85a3e" opacity=".7"/><rect width="24" height="12" fill="#7a2c2a" opacity=".45"/></pattern>'
            '<linearGradient id="cp" y2="1"><stop stop-color="#efd99a"/><stop offset="1" stop-color="#cdb06a"/></linearGradient>')
    tulis("orang-orangan", w, h, "".join(out), defs)


def penjor():
    """Penjor janur: galah bambu melengkung, hiasan janur bergantung, sampian di ujung."""
    w, h = 360, 1100
    p0, p1, p2 = (120, 1100), (110, 300), (300, 120)
    out = []
    pts = [kuad(p0, p1, p2, i / 30) for i in range(31)]
    out.append('<path d="M' + " L".join(f"{f(x)} {f(y)}" for x, y in pts) + '" fill="none" stroke="#c9a65a" stroke-width="16" stroke-linecap="round"/>')
    out.append('<path d="M' + " L".join(f"{f(x)} {f(y)}" for x, y in pts) + '" fill="none" stroke="#e8cf8a" stroke-width="5" stroke-linecap="round" transform="translate(-3 0)"/>')
    # Rumbai janur di sepanjang lengkung.
    for i in range(5, 30):
        x, y = pts[i]
        L = 70 + (i % 3) * 22
        out.append(f'<path d="M{f(x)} {f(y)}q-10 {f(L*.5)} 4 {L}" fill="none" stroke="#f3e7ae" stroke-width="10" stroke-linecap="round"/>')
        out.append(f'<path d="M{f(x)} {f(y)}q14 {f(L*.45)} 22 {f(L*.85)}" fill="none" stroke="#d9c26a" stroke-width="8" stroke-linecap="round"/>')
        if i % 4 == 0:
            out.append(f'<circle cx="{f(x+8)}" cy="{f(y+L*.9)}" r="9" fill="{["#e06a7a", "#f3b13a"][i % 8 // 4]}"/>')
    # Sampian (hiasan ujung) + lamak (kain bergantung) di bawah.
    sx, sy = p2
    out.append(f'<path d="M{f(sx)} {f(sy)}q20 30 6 90q-26 -30 -6 -90z" fill="#f4e7b2" stroke="#c9a65a" stroke-width="3"/>')
    out.append(f'<circle cx="{f(sx+4)}" cy="{f(sy+96)}" r="10" fill="#e06a7a"/>')
    out.append('<path d="M96 760h48v170l-24 24l-24 -24z" fill="#f4e7b2" stroke="#c9a65a" stroke-width="3"/>')
    out.append('<path d="M104 790h32M104 830h32M104 870h32" stroke="#6aa04a" stroke-width="6"/>')
    out.append('<circle cx="120" cy="905" r="9" fill="#e9a23a"/>')
    tulis("penjor", w, h, "".join(out))


def janur_lengkung(cx, y, lebar, tinggi):
    """Janur melengkung di atas gapura: busur + rumbai."""
    out = [f'<path d="M{f(cx-lebar/2)} {f(y)}Q{f(cx)} {f(y-tinggi*2)} {f(cx+lebar/2)} {f(y)}" fill="none" stroke="#e7d38a" stroke-width="14"/>']
    for i in range(1, 24):
        t = i / 24
        x, yy = kuad((cx - lebar / 2, y), (cx, y - tinggi * 2), (cx + lebar / 2, y), t)
        out.append(f'<path d="M{f(x)} {f(yy)}l-5 30M{f(x)} {f(yy)}l6 26" stroke="#f3e6b0" stroke-width="4" stroke-linecap="round"/>')
    return "".join(out)


def gapura_sawah():
    w, h = 760, 860
    out = []
    for x in (110, 150, 610, 650):
        out.append(f'<rect x="{x-14}" y="250" width="28" height="610" rx="12" fill="url(#bb)"/>')
        for y in range(320, 860, 90):
            out.append(f'<path d="M{x-14} {y}h28" stroke="#9a8040" stroke-width="4"/>')
    out.append('<rect x="70" y="236" width="620" height="30" rx="14" fill="url(#bb)"/>')
    out.append('<rect x="70" y="300" width="620" height="22" rx="11" fill="url(#bb)"/>')
    out.append(janur_lengkung(380, 236, 640, 110))
    # Papan nama (teks dari DOM ditaruh di depannya).
    out.append('<rect x="190" y="330" width="380" height="96" rx="12" fill="#7d5a36"/><rect x="200" y="340" width="360" height="76" rx="8" fill="#f6ecd2"/>')
    # Hiasan bunga & daun kelapa di kaki.
    for x in (130, 630):
        out.append(f'<ellipse cx="{x}" cy="850" rx="70" ry="14" fill="#000" opacity=".12"/>')
        out.append(f'<path d="M{x-50} 860q10 -60 50 -70q40 10 50 70z" fill="#6aa04a"/>')
        for k, c in enumerate(["#e9638a", "#f3b13a", "#fff4d6"]):
            out.append(f'<circle cx="{x-24+k*24}" cy="{800-k%2*12}" r="12" fill="{c}"/>')
    defs = '<linearGradient id="bb" x2="1"><stop stop-color="#b9974e"/><stop offset=".5" stop-color="#e2c77e"/><stop offset="1" stop-color="#a9873e"/></linearGradient>'
    tulis("gapura-sawah", w, h, "".join(out), defs)


def gebyok():
    """Gebyok pelaminan: panel kayu ukir berbingkai emas + dua kembar mayang."""
    w, h = 1100, 700
    out = ['<path d="M150 700V180Q150 120 210 120H890Q950 120 950 180V700Z" fill="url(#gk)"/>',
           '<path d="M150 700V180Q150 120 210 120H890Q950 120 950 180V700" fill="none" stroke="#d7b45a" stroke-width="10"/>',
           '<path d="M380 700V300Q550 150 720 300V700Z" fill="#f7ecd2" stroke="#d7b45a" stroke-width="8"/>']
    rng = random.Random(9)
    for cx in (265, 835):
        for row in range(4):
            cy = 220 + row * 120
            out.append(f'<g fill="none" stroke="#e2c070" stroke-width="5" stroke-linecap="round">'
                       f'<path d="M{cx} {cy+40}c-40 -10 -50 -60 0 -70c50 10 40 60 0 70z"/>'
                       f'<path d="M{cx-60} {cy+10}q30 -30 60 0q30 30 60 0"/>'
                       f'<circle cx="{cx}" cy="{cy}" r="8" fill="#e2c070"/></g>')
    # Mahkota ukir di atas.
    out.append('<path d="M420 120Q550 20 680 120" fill="none" stroke="#d7b45a" stroke-width="12"/>')
    out.append('<path d="M500 110q50 -70 100 0" fill="#d7b45a"/><circle cx="550" cy="58" r="14" fill="#f3d98a"/>')
    # Kembar mayang (kiri & kanan): vas + rangkaian janur.
    for cx in (70, 1030):
        out.append(f'<path d="M{cx-34} 700l10 -90h48l10 90z" fill="#c9a24b"/>')
        out.append(f'<path d="M{cx} 610V330" stroke="#8a6a3a" stroke-width="6"/>')
        for k in range(12):
            a = -math.pi / 2 + (k - 5.5) * .24
            L = 110 + (k % 3) * 20
            ex, ey = cx + math.cos(a) * L, 420 + math.sin(a) * L
            out.append(f'<path d="M{cx} 440Q{f((cx+ex)/2 + (ex-cx)*.2)} {f((440+ey)/2)} {f(ex)} {f(ey)}" fill="none" stroke="{["#efe1a4", "#d6c06a"][k%2]}" stroke-width="7" stroke-linecap="round"/>')
        for k in range(7):
            out.append(f'<circle cx="{f(cx + rng.uniform(-50, 50))}" cy="{f(rng.uniform(330, 460))}" r="11" fill="{["#e9638a", "#fff4d6", "#f3b13a"][k%3]}"/>')
    defs = '<linearGradient id="gk" y2="1"><stop stop-color="#7a4a26"/><stop offset="1" stop-color="#5a321a"/></linearGradient>'
    tulis("gebyok", w, h, "".join(out), defs)


def kincir():
    w, h = 520, 520
    c = 260
    out = [f'<circle cx="{c}" cy="{c}" r="230" fill="none" stroke="#b08a4e" stroke-width="14"/>',
           f'<circle cx="{c}" cy="{c}" r="180" fill="none" stroke="#c9a462" stroke-width="8"/>']
    for k in range(12):
        a = k * math.pi / 6
        x, y = c + math.cos(a) * 230, c + math.sin(a) * 230
        out.append(f'<path d="M{c} {c}L{f(x)} {f(y)}" stroke="#a07c44" stroke-width="9"/>')
        px, py = c + math.cos(a) * 250, c + math.sin(a) * 250
        tx, ty = -math.sin(a) * 30, math.cos(a) * 30
        out.append(f'<path d="M{f(px-tx)} {f(py-ty)}L{f(px+tx)} {f(py+ty)}" stroke="#8a6436" stroke-width="16" stroke-linecap="round"/>')
    out.append(f'<circle cx="{c}" cy="{c}" r="26" fill="#7a5a32"/>')
    tulis("kincir", w, h, "".join(out))


def layangan():
    w, h = 180, 320
    out = ['<path d="M90 10L160 90L90 190L20 90Z" fill="#e2574c"/>',
           '<path d="M90 10L90 190M20 90H160" stroke="#fff3d6" stroke-width="4"/>',
           '<path d="M90 10L160 90L90 90Z" fill="#f3b13a"/>',
           '<path d="M90 190q-20 30 0 60t0 60" fill="none" stroke="#3a5f8a" stroke-width="4"/>']
    for k in range(4):
        out.append(f'<path d="M{80 + (k%2)*12} {214 + k*26}l10 -6l10 6" fill="none" stroke="#e2574c" stroke-width="5"/>')
    tulis("layangan", w, h, "".join(out))


def gunung():
    """Panorama latar 2400×520: dua gunung api pucat + bukit hijau + deret pohon."""
    w, h = 2400, 520
    out = ['<path d="M0 520V380L260 300L620 90Q700 40 780 90L1140 300L1300 250L1560 120Q1620 86 1680 122L2000 300L2400 360V520Z" fill="url(#gj)"/>',
           '<path d="M640 120Q700 70 760 120L720 150L700 130L680 150Z" fill="#f4f6f8" opacity=".85"/>',
           '<path d="M0 520V420Q300 360 600 410T1200 400T1800 390T2400 410V520Z" fill="url(#gb)"/>']
    rng = random.Random(2)
    x = 0
    while x < w:
        r = rng.uniform(14, 26)
        out.append(f'<circle cx="{f(x)}" cy="{f(452 - r*.4)}" r="{f(r)}" fill="#5f8f4a" opacity=".85"/>')
        x += rng.uniform(16, 34)
    out.append('<rect y="460" width="2400" height="60" fill="#6c9c50"/>')
    defs = ('<linearGradient id="gj" y2="1"><stop stop-color="#9fb3c8"/><stop offset="1" stop-color="#c9d6dc"/></linearGradient>'
            '<linearGradient id="gb" y2="1"><stop stop-color="#8fb47a"/><stop offset="1" stop-color="#6c9c50"/></linearGradient>')
    tulis("gunung", w, h, "".join(out), defs)


# ───────────────────────────── PESISIR ─────────────────────────────
def perahu():
    """Perahu jukung bercadik dari samping, haluan bermata, layar tergulung."""
    w, h = 960, 460
    out = ['<path d="M60 250Q120 380 480 390Q840 380 900 250L840 270Q480 300 120 270Z" fill="url(#ph)"/>',
           '<path d="M70 262Q480 310 890 262" fill="none" stroke="#fff" stroke-width="12"/>',
           '<path d="M90 290Q480 340 870 290" fill="none" stroke="#e2574c" stroke-width="10"/>',
           '<path d="M40 210Q60 250 120 270L60 250Z" fill="#2f6aa0"/><path d="M920 210Q900 250 840 270L900 250Z" fill="#2f6aa0"/>',
           '<circle cx="150" cy="300" r="16" fill="#fff"/><circle cx="154" cy="300" r="8" fill="#222"/>',
           # Cadik: dua tiang lengkung + pelampung bambu.
           '<path d="M250 280Q300 200 360 400M700 280Q650 200 600 400" fill="none" stroke="#8a6a44" stroke-width="10"/>',
           '<rect x="200" y="392" width="560" height="22" rx="11" fill="#c9a65a"/>',
           # Tiang + layar tergulung.
           '<path d="M480 300V40" stroke="#7a5a3a" stroke-width="10"/>',
           '<path d="M480 60L700 250" stroke="#7a5a3a" stroke-width="6"/>',
           '<path d="M486 70L690 240L486 240Z" fill="#f4e2c0" stroke="#d9b98a" stroke-width="4"/>',
           '<path d="M486 120L630 200M486 170L590 230" stroke="#e2574c" stroke-width="6"/>',
           '<ellipse cx="480" cy="440" rx="440" ry="18" fill="#000" opacity=".14"/>']
    defs = '<linearGradient id="ph" y2="1"><stop stop-color="#3b7cc0"/><stop offset="1" stop-color="#23527e"/></linearGradient>'
    tulis("perahu", w, h, "".join(out), defs)


def gapura_pesisir():
    w, h = 760, 860
    out = []
    for x in (120, 640):
        out.append(f'<path d="M{x-22} 860L{x-16} 230Q{x} 210 {x+16} 230L{x+22} 860Z" fill="url(#kd)"/>')
        for y in range(300, 860, 70):
            out.append(f'<path d="M{x-20} {y}q20 6 40 0" fill="none" stroke="#c9b48a" stroke-width="6"/>')
    out.append('<path d="M80 250Q380 180 680 250L680 290Q380 220 80 290Z" fill="url(#kd)"/>')
    # Tali tambang menjuntai + pelampung.
    out.append('<path d="M120 330Q380 470 640 330" fill="none" stroke="#d8c08a" stroke-width="8" stroke-dasharray="14 6"/>')
    for k, (x, y) in enumerate([(200, 382), (290, 416), (470, 416), (560, 382)]):
        out.append(f'<path d="M{x} {y}v20" stroke="#d8c08a" stroke-width="4"/><circle cx="{x}" cy="{y+40}" r="22" fill="{["#e2574c", "#fff", "#e2574c", "#fff"][k]}" stroke="#e2574c" stroke-width="6"/>')
    # Kemudi kapal di puncak.
    out.append('<g transform="translate(380 170)"><circle r="62" fill="none" stroke="#7a5230" stroke-width="12"/><circle r="16" fill="#7a5230"/>'
               + "".join(f'<path d="M0 0L{f(math.cos(k*math.pi/4)*86)} {f(math.sin(k*math.pi/4)*86)}" stroke="#7a5230" stroke-width="10" stroke-linecap="round"/>' for k in range(8))
               + '</g>')
    # Papan nama.
    out.append('<rect x="210" y="470" width="340" height="86" rx="10" fill="#6a4a2e"/><rect x="220" y="480" width="320" height="66" rx="6" fill="#f3ead6"/>')
    # Kerang & bintang laut di kaki.
    for x in (120, 640):
        out.append(f'<ellipse cx="{x}" cy="852" rx="80" ry="14" fill="#000" opacity=".12"/>')
        out.append(f'<path d="M{x-60} 852q20 -40 40 0zM{x+20} 852q20 -46 44 0z" fill="#f2b8a0"/>')
    defs = '<linearGradient id="kd" x2="1"><stop stop-color="#a68a6a"/><stop offset=".5" stop-color="#cdb594"/><stop offset="1" stop-color="#94785a"/></linearGradient>'
    tulis("gapura-pesisir", w, h, "".join(out), defs)


def kursi():
    w, h = 220, 340
    out = ['<path d="M40 40Q110 0 180 40V190H40Z" fill="#fbf7f0" stroke="#e6dccb" stroke-width="4"/>',
           '<path d="M30 190H190V230H30Z" fill="#fbf7f0" stroke="#e6dccb" stroke-width="4"/>',
           '<path d="M40 230L50 340M180 230L170 340M100 230V330M120 230V330" stroke="#e6dccb" stroke-width="10"/>',
           '<path d="M40 120H180V150H40Z" fill="url(#pt)"/>',
           '<path d="M110 135l-36 -24v48zM110 135l36 -24v48z" fill="url(#pt)"/>',
           '<circle cx="110" cy="135" r="12" fill="#f3d98a"/>']
    defs = '<linearGradient id="pt" x2="1"><stop stop-color="#e98aa6"/><stop offset="1" stop-color="#d4607e"/></linearGradient>'
    tulis("kursi", w, h, "".join(out), defs)


def pandan():
    """Semak pandan laut + batu karang di pasir."""
    rng = random.Random(8)
    w, h = 520, 420
    out = ['<path d="M30 420Q60 330 150 320Q220 300 260 360Q300 300 380 330Q470 340 500 420Z" fill="#a8a29a"/>',
           '<path d="M60 420Q90 360 160 350Q210 340 240 380" fill="none" stroke="#c9c3ba" stroke-width="10"/>']
    for k in range(26):
        a = -math.pi / 2 + rng.uniform(-1.2, 1.2)
        L = rng.uniform(120, 230)
        x0 = 260 + rng.uniform(-60, 60)
        ex, ey = x0 + math.cos(a) * L, 360 + math.sin(a) * L
        c = ["#4f8a3c", "#6aa24a", "#3e7330"][k % 3]
        out.append(f'<path d="M{f(x0)} 360Q{f((x0+ex)/2)} {f((360+ey)/2 - 30)} {f(ex)} {f(ey)}" fill="none" stroke="{c}" stroke-width="9" stroke-linecap="round"/>')
    tulis("pandan", w, h, "".join(out))


def layar():
    w, h = 240, 260
    out = ['<path d="M30 200Q120 230 210 200L196 226H44Z" fill="#7a4a2a"/>',
           '<path d="M120 200V20" stroke="#5a3a20" stroke-width="5"/>',
           '<path d="M124 30Q200 110 190 190H124Z" fill="#fbf1de"/>',
           '<path d="M116 50Q60 120 56 190H116Z" fill="#f2c7a2"/>']
    tulis("layar", w, h, "".join(out))


def bohlam():
    """Untaian bohlam melintang di atas lorong tamu: kabel melengkung
    (Bezier kuadrat) + bohlam berpendar di sepanjang lengkungnya."""
    w, h = 1280, 120
    defs = ('<radialGradient id="g"><stop offset=".2" stop-color="#fff6c8"/>'
            '<stop offset=".45" stop-color="#ffd27a" stop-opacity=".75"/>'
            '<stop offset="1" stop-color="#ffb84a" stop-opacity="0"/></radialGradient>')
    out = ['<path d="M0 10Q640 150 1280 10" fill="none" stroke="#3a2f2a" stroke-width="2"/><g fill="url(#g)">']
    for x in range(40, w, 80):
        t = x / w
        y = (1 - t) ** 2 * 10 + 2 * t * (1 - t) * 150 + t * t * 10
        out.append(f'<circle cx="{x}" cy="{f(y)}" r="18"/>')
    out.append("</g>")
    tulis("bohlam", w, h, "".join(out), defs)


if __name__ == "__main__":
    kelapa(); pisang(); bambu(); padi(); padi(True); orang_orangan(); penjor()
    gapura_sawah(); gebyok(); kincir(); layangan(); gunung()
    perahu(); gapura_pesisir(); kursi(); pandan(); layar(); bohlam()
    print("ok", sorted(os.listdir(OUT)))
