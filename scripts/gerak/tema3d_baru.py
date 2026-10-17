#!/usr/bin/env python3
"""Lapisan panggung untuk empat tema 3D (migrasi 053): balon, kastil, kunang, teh.

Tiap adegan = tiga SVG potret 1080×1920 seperti adegan lain di
src/web/panggung/svg: JAUH (langit + cakrawala ±y1290), UTAMA (bangunan di
tengah, kaki ±y1440–1480), DEPAN (bingkai kiri/kanan/bawah yang ditembus
kamera). Jalankan:  python3 scripts/gerak/tema3d_baru.py
"""
import math
import os
import random

OUT = os.path.join(os.path.dirname(__file__), "..", "..", "src", "web", "panggung", "svg")


def f(x):
    return f"{x:.1f}".rstrip("0").rstrip(".")


def tulis(nama, isi, defs=""):
    svg = f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1080 1920">'
    if defs:
        svg += f"<defs>{defs}</defs>\n"
    svg += isi + "</svg>\n"
    with open(os.path.join(OUT, f"{nama}.svg"), "w") as fh:
        fh.write(svg)


def awan(cx, cy, w, fill="#fff", op=1.0, n=5, seed=0):
    """Gumpalan awan: deretan lingkaran di atas alas elips."""
    r = random.Random(seed)
    out = [f'<g fill="{fill}" opacity="{op}">', f'<ellipse cx="{f(cx)}" cy="{f(cy)}" rx="{f(w/2)}" ry="{f(w*.12)}"/>']
    for i in range(n):
        t = (i + .5) / n
        x = cx - w / 2 + w * t
        rr = w * (.13 + .12 * math.sin(math.pi * t)) * (0.85 + .3 * r.random())
        out.append(f'<circle cx="{f(x)}" cy="{f(cy - rr*.55)}" r="{f(rr)}"/>')
    out.append("</g>")
    return "".join(out)


def balon(cx, cy, s, warna, garis="#fff"):
    """Balon udara: amplop berlajur + keranjang."""
    w, h = 120 * s, 150 * s
    d = (f"M{f(cx)} {f(cy-h/2)}C{f(cx+w*.62)} {f(cy-h/2)} {f(cx+w*.62)} {f(cy+h*.12)} "
         f"{f(cx+w*.16)} {f(cy+h*.42)}L{f(cx-w*.16)} {f(cy+h*.42)}C{f(cx-w*.62)} {f(cy+h*.12)} "
         f"{f(cx-w*.62)} {f(cy-h/2)} {f(cx)} {f(cy-h/2)}z")
    lajur = "".join(
        f'<path fill="none" stroke="{garis}" stroke-width="{f(3*s)}" opacity=".6" d="M{f(cx)} {f(cy-h/2)}Q{f(cx+k*w*.36)} {f(cy)} {f(cx+k*w*.1)} {f(cy+h*.42)}"/>'
        for k in (-1, -.45, .45, 1))
    tali = (f'<path stroke="#7a5a3a" stroke-width="{f(2*s)}" d="M{f(cx-w*.15)} {f(cy+h*.42)}L{f(cx-w*.1)} {f(cy+h*.6)}'
            f'M{f(cx+w*.15)} {f(cy+h*.42)}L{f(cx+w*.1)} {f(cy+h*.6)}"/>')
    kr = f'<rect x="{f(cx-w*.12)}" y="{f(cy+h*.6)}" width="{f(w*.24)}" height="{f(h*.13)}" rx="{f(4*s)}" fill="#a87a4e"/>'
    return f'<path fill="{warna}" d="{d}"/>{lajur}{tali}{kr}'


# ─────────────────────────── BALON: pulau awan & gazebo ───────────────────────────
def balon_jauh():
    defs = ('<linearGradient id="bSky" x2="0" y2="1"><stop stop-color="#9fc4ef"/><stop offset=".5" stop-color="#f7cfe0"/>'
            '<stop offset=".8" stop-color="#ffe3cf"/><stop offset="1" stop-color="#fff4e6"/></linearGradient>'
            '<radialGradient id="bSun"><stop stop-color="#fffdf2"/><stop offset=".35" stop-color="#ffe6b8" stop-opacity=".8"/><stop offset="1" stop-color="#ffd7b0" stop-opacity="0"/></radialGradient>')
    isi = ['<path fill="url(#bSky)" d="M0 0h1080v1920H0z"/>',
           '<ellipse cx="300" cy="1120" rx="460" ry="460" fill="url(#bSun)"/><circle cx="300" cy="1150" r="74" fill="#fffbea"/>']
    for cx, cy, w, op, sd in [(180, 360, 300, .55, 1), (860, 260, 360, .5, 2), (620, 560, 240, .45, 3), (120, 820, 220, .4, 4)]:
        isi.append(awan(cx, cy, w, op=op, seed=sd))
    # balon kecil di kejauhan
    for cx, cy, s, c in [(760, 700, .38, "#e98aa6"), (900, 960, .26, "#f3b562"), (440, 820, .22, "#8fb7e8"), (180, 1040, .2, "#c59be0")]:
        isi.append(f'<g opacity=".8">{balon(cx, cy, s, c)}</g>')
    # lautan awan (cakrawala ±1290)
    isi.append('<path fill="#fff6f2" d="M0 1290h1080v630H0z"/>')
    for row, (y, op) in enumerate([(1280, .95), (1360, 1), (1460, 1), (1600, 1), (1760, 1)]):
        for i in range(-1, 6):
            x = i * 230 + (row % 2) * 115
            isi.append(awan(x, y, 300 + row * 40, fill="#ffffff" if row % 2 else "#fdf0f4", op=op, seed=row * 10 + i))
    tulis("balon-jauh", "\n".join(isi), defs)


def balon_utama():
    defs = ('<linearGradient id="bRock" x2="0" y2="1"><stop stop-color="#c9a7c9"/><stop offset="1" stop-color="#7d6a9c"/></linearGradient>'
            '<linearGradient id="bCol" x2="1"><stop stop-color="#fff"/><stop offset="1" stop-color="#ead9ea"/></linearGradient>'
            '<linearGradient id="bDome" x2="1" y2="1"><stop stop-color="#fff"/><stop offset=".6" stop-color="#f6dbe6"/><stop offset="1" stop-color="#d9a9c0"/></linearGradient>'
            '<filter id="bBlur"><feGaussianBlur stdDeviation="20"/></filter>')
    isi = ['<ellipse cx="540" cy="1470" rx="470" ry="80" fill="#c48bb0" opacity=".25" filter="url(#bBlur)"/>',
           # pulau mengapung: rumput di atas, batu meruncing ke bawah
           '<path fill="url(#bRock)" d="M150 1430c40 140 200 260 300 330 50 34 130 34 180 0 100-70 260-190 300-330z"/>',
           '<path fill="#9f86b8" opacity=".6" d="M330 1560c60 80 140 150 210 180 70-30 150-100 210-180z"/>',
           '<path fill="#9cc98a" d="M130 1440c0-36 60-50 410-50s410 14 410 50c0 22-60 34-410 34s-410-12-410-34z"/>',
           '<path fill="#7fb06e" d="M150 1450c90 20 250 26 390 26s300-6 390-26v14c-90 22-250 30-390 30s-300-8-390-30z"/>',
           # lantai gazebo
           '<path fill="#fff" stroke="#e7cfe0" stroke-width="6" d="M300 1420h480l-26-40H326z"/>',
           '<path fill="#f3e3ee" d="M330 1380h420v-20H330z"/>']
    # tiang
    for x in (350, 450, 630, 730):
        isi.append(f'<path fill="url(#bCol)" stroke="#e7cfe0" stroke-width="5" d="M{x-14} 1362V930h28v432z"/>')
    isi += ['<path fill="#fff" stroke="#e7cfe0" stroke-width="6" d="M316 940h448v-44H316z"/>',
            '<path fill="url(#bDome)" stroke="#e7cfe0" stroke-width="6" d="M330 896c0-150 94-250 210-250s210 100 210 250z"/>',
            '<path fill="none" stroke="#fff" stroke-width="10" opacity=".8" d="M400 880c6-110 60-180 140-196"/>',
            '<path fill="#f1b6c9" d="M532 646v-50h16v50z"/><circle cx="540" cy="584" r="18" fill="#f3c46b"/>']
    # tirai merah muda
    isi += ['<path fill="#fbe1ea" opacity=".95" d="M364 950c-30 140 22 280-6 412h-30V950zM716 950c30 140-22 280 6 412h30V950z"/>',
            '<path fill="none" stroke="#e98aa6" stroke-width="7" d="M364 1150c18-12 36-12 54 0M716 1150c-18-12-36-12-54 0"/>']
    # lengkung bunga di depan dome
    r = random.Random(7)
    for i in range(26):
        t = i / 25
        a = math.pi * (1 - t)
        x = 540 + 210 * math.cos(a)
        y = 940 - 70 * math.sin(a)
        c = r.choice(["#e98aa6", "#f6c2d3", "#fff", "#f3b562"])
        isi.append(f'<circle cx="{f(x)}" cy="{f(y)}" r="{f(13 + r.random()*6)}" fill="{c}"/>')
    # semak bunga di tepi pulau
    for x in list(range(170, 330, 34)) + list(range(760, 930, 34)):
        isi.append(f'<circle cx="{x}" cy="{1418 - r.random()*16}" r="{20 + r.random()*8:.0f}" fill="#8cc07a"/>')
        isi.append(f'<circle cx="{x + 6}" cy="{1404 - r.random()*16:.0f}" r="7" fill="{r.choice(["#e98aa6", "#fff", "#f3b562"])}"/>')
    # akar / sulur menggantung
    isi.append('<g fill="none" stroke="#7d6a9c" stroke-width="5" stroke-linecap="round" opacity=".7"><path d="M420 1640q-10 60 6 110M540 1760q4 50-6 90M660 1640q12 50-4 100"/></g>')
    tulis("balon-utama", "\n".join(isi), defs)


def balon_depan():
    isi = []
    # balon besar membingkai kiri-kanan (dilewati kamera)
    isi.append(balon(150, 520, 1.5, "#e98aa6"))
    isi.append(balon(960, 760, 1.25, "#f3b562"))
    isi.append(balon(90, 1180, .9, "#8fb7e8"))
    # awan tebal di bawah
    for i, (x, y, w) in enumerate([(80, 1780, 520), (420, 1880, 560), (820, 1800, 560), (1060, 1700, 360), (0, 1620, 320)]):
        isi.append(awan(x, y, w, fill="#ffffff", seed=40 + i, n=6))
    isi.append('<path fill="#fff" d="M0 1860h1080v60H0z"/>')
    tulis("balon-depan", "\n".join(isi))


# ─────────────────────────── KASTIL: kastil dongeng & lengkung mawar ───────────────────────────
def kastil_jauh():
    defs = ('<linearGradient id="kSky" x2="0" y2="1"><stop stop-color="#5b4a86"/><stop offset=".45" stop-color="#c98bb0"/>'
            '<stop offset=".8" stop-color="#f6c7b5"/><stop offset="1" stop-color="#fde6d2"/></linearGradient>'
            '<linearGradient id="kM1" x2="0" y2="1"><stop stop-color="#9a87b8"/><stop offset="1" stop-color="#b9a1c4"/></linearGradient>'
            '<linearGradient id="kM2" x2="0" y2="1"><stop stop-color="#7f9a7c"/><stop offset="1" stop-color="#5f7d60"/></linearGradient>'
            '<linearGradient id="kGr" x2="0" y2="1"><stop stop-color="#8fb07e"/><stop offset="1" stop-color="#557a52"/></linearGradient>')
    r = random.Random(3)
    bintang = "".join(f'<circle cx="{r.randrange(0,1080)}" cy="{r.randrange(0,640)}" r="{r.choice([1.6,2.2,3])}" fill="#fff" opacity="{r.choice([.5,.7,.9])}"/>' for _ in range(70))
    isi = ['<path fill="url(#kSky)" d="M0 0h1080v1920H0z"/>', bintang,
           '<circle cx="820" cy="380" r="64" fill="#fff6e8" opacity=".9"/>',
           '<path fill="url(#kM1)" d="M0 1180l140-170 120 110 170-230 170 210 120-120 180 180 180-160v320H0z"/>',
           '<path fill="url(#kM2)" d="M0 1260c160-70 300-80 440-40 140 40 290 30 440-20 80-26 150-28 200-10v140H0z"/>',
           '<path fill="url(#kGr)" d="M0 1290h1080v630H0z"/>',
           # jalan setapak menuju kastil
           '<path fill="#e9d2b6" opacity=".9" d="M512 1292h56l330 628H182z"/>',
           '<path fill="#f8e6cf" opacity=".5" d="M530 1292h20l120 628h-260z"/>']
    # pagar semak di kiri-kanan jalan
    for side in (-1, 1):
        for i in range(10):
            t = i / 9
            x = 540 + side * (60 + 420 * t)
            y = 1300 + 600 * t
            rr = 16 + 60 * t
            isi.append(f'<circle cx="{f(x)}" cy="{f(y)}" r="{f(rr)}" fill="#4f7350"/>')
            isi.append(f'<circle cx="{f(x + side*rr*.2)}" cy="{f(y - rr*.4)}" r="{f(rr*.28)}" fill="#e2708e"/>')
    tulis("kastil-jauh", "\n".join(isi), defs)


def menara_kastil(cx, y_kaki, tinggi, lebar, atap="#8a3b5e"):
    x0 = cx - lebar / 2
    jendela = "".join(
        f'<path fill="#ffd98a" d="M{f(cx-9)} {f(y)}v-26a9 9 0 0118 0v26z"/>'
        for y in range(int(y_kaki - 60), int(y_kaki - tinggi + 90), -110))
    return (f'<path fill="url(#kWall)" stroke="#fff4ea" stroke-width="5" d="M{f(x0)} {f(y_kaki)}V{f(y_kaki-tinggi)}h{f(lebar)}V{f(y_kaki)}z"/>'
            f'<path fill="#f1e1d6" d="M{f(x0-8)} {f(y_kaki-tinggi)}h{f(lebar+16)}v-22h{f(-lebar-16)}z"/>'
            f'<path fill="{atap}" d="M{f(x0-14)} {f(y_kaki-tinggi-22)}L{f(cx)} {f(y_kaki-tinggi-22-lebar*1.7)}L{f(x0+lebar+14)} {f(y_kaki-tinggi-22)}z"/>'
            f'<path stroke="#6a2a48" stroke-width="3" d="M{f(cx)} {f(y_kaki-tinggi-22-lebar*1.7)}v-40"/>'
            f'<path fill="#f3b562" d="M{f(cx)} {f(y_kaki-tinggi-62-lebar*1.7)}l34 10-34 10z"/>{jendela}')


def kastil_utama():
    defs = ('<linearGradient id="kWall" x2="1"><stop stop-color="#fff7f0"/><stop offset="1" stop-color="#e6d3cb"/></linearGradient>'
            '<filter id="kBlur"><feGaussianBlur stdDeviation="22"/></filter>')
    isi = ['<ellipse cx="540" cy="1460" rx="480" ry="80" fill="#3e2a48" opacity=".22" filter="url(#kBlur)"/>',
           '<path fill="#d9c3b4" d="M180 1470h720l-30-40H210z"/>',
           menara_kastil(230, 1440, 520, 96, "#7c3456"), menara_kastil(850, 1440, 520, 96, "#7c3456"),
           menara_kastil(370, 1440, 680, 110), menara_kastil(710, 1440, 680, 110),
           # bangunan tengah
           '<path fill="url(#kWall)" stroke="#fff4ea" stroke-width="6" d="M300 1440V980h480v460z"/>',
           '<path fill="#f1e1d6" d="M290 990h500v-34H290z"/>',
           ''.join(f'<path fill="#f1e1d6" d="M{x} 956h30v-30h-30z"/>' for x in range(296, 780, 56)),
           menara_kastil(540, 980, 300, 140, "#9a4468"),
           # pintu besar & jendela
           '<path fill="#5a3048" d="M470 1440v-170a70 70 0 01140 0v170z"/>',
           '<path fill="none" stroke="#f3b562" stroke-width="6" d="M470 1440v-170a70 70 0 01140 0v170M540 1200v240"/>',
           '<path fill="#ffd98a" d="M360 1200v-60a26 26 0 0152 0v60zM668 1200v-60a26 26 0 0152 0v60z"/>',
           '<circle cx="540" cy="1080" r="44" fill="#ffd98a" stroke="#f3b562" stroke-width="6"/>',
           '<path stroke="#f3b562" stroke-width="4" d="M540 1036v88M496 1080h88M509 1049l62 62M571 1049l-62 62"/>']
    # sulur mawar memanjat dinding
    r = random.Random(11)
    for x0, side in ((300, 1), (780, -1)):
        pts = []
        for i in range(14):
            y = 1440 - i * 34
            x = x0 + side * (10 + 18 * math.sin(i * .9))
            pts.append((x, y))
        d = "M" + " L".join(f"{f(x)} {f(y)}" for x, y in pts)
        isi.append(f'<path fill="none" stroke="#4f7350" stroke-width="6" d="{d}"/>')
        for x, y in pts[::2]:
            isi.append(f'<circle cx="{f(x)}" cy="{f(y)}" r="{f(11 + r.random()*5)}" fill="{r.choice(["#e2708e", "#c94a6e", "#f6a8bb"])}"/>')
    tulis("kastil-utama", "\n".join(isi), defs)


def kastil_depan():
    r = random.Random(21)
    isi = ['<path fill="#3f6142" d="M0 1700c160-40 340-30 540 0s380 30 540-10v230H0z"/>']
    # lengkung pergola mawar membingkai layar
    isi.append('<path fill="none" stroke="#5a4030" stroke-width="34" d="M90 1920V700C90 360 300 180 540 180s450 180 450 520v1220"/>')
    isi.append('<path fill="none" stroke="#7a5a40" stroke-width="12" d="M90 1920V700C90 360 300 180 540 180s450 180 450 520v1220"/>')
    for i in range(110):
        t = i / 109
        # sepanjang lengkung: kiri naik, atas, kanan turun
        if t < .35:
            x, y = 90, 1920 - (1220 * t / .35)
        elif t < .65:
            a = math.pi * (1 - (t - .35) / .3)
            x, y = 540 + 450 * math.cos(a), 700 - 520 * math.sin(a)
        else:
            x, y = 990, 700 + 1220 * (t - .65) / .35
        x += r.uniform(-30, 30)
        y += r.uniform(-24, 24)
        isi.append(f'<circle cx="{f(x)}" cy="{f(y)}" r="{f(r.uniform(22, 34))}" fill="{r.choice(["#3f6142", "#4f7350", "#5f8a5c"])}"/>')
        if r.random() < .55:
            rr = r.uniform(14, 24)
            c = r.choice(["#e2708e", "#c94a6e", "#f6a8bb", "#fff1f4"])
            isi.append(f'<circle cx="{f(x)}" cy="{f(y)}" r="{f(rr)}" fill="{c}"/><circle cx="{f(x)}" cy="{f(y)}" r="{f(rr*.45)}" fill="#fff" opacity=".35"/>')
    # semak depan
    for x in range(-40, 1120, 90):
        isi.append(f'<circle cx="{x}" cy="{1800 + r.randint(-20, 30)}" r="{r.randint(70, 100)}" fill="#355737"/>')
    tulis("kastil-depan", "\n".join(isi))


# ─────────────────────────── KUNANG: hutan malam & rumah pohon ───────────────────────────
def kunang_titik(r, n, x0, x1, y0, y1, rmax=5):
    return "".join(
        f'<circle cx="{r.uniform(x0, x1):.0f}" cy="{r.uniform(y0, y1):.0f}" r="{r.uniform(1.5, rmax):.1f}" fill="#fff3a0" opacity="{r.uniform(.4, .95):.2f}"/>'
        for _ in range(n))


def pohon_siluet(x, y, h, w, fill):
    return (f'<path fill="{fill}" d="M{f(x-w*.08)} {f(y)}V{f(y-h*.55)}h{f(w*.16)}V{f(y)}z"/>'
            f'<path fill="{fill}" d="M{f(x)} {f(y-h)}l{f(w*.5)} {f(h*.45)}h{f(-w*.18)}l{f(w*.3)} {f(h*.3)}h{f(-w*.24)}l{f(w*.22)} {f(h*.25)}h{f(-w*1.2)}l{f(w*.22)} {f(-h*.25)}h{f(-w*.24)}l{f(w*.3)} {f(-h*.3)}h{f(-w*.18)}z"/>')


def kunang_jauh():
    defs = ('<linearGradient id="nSky" x2="0" y2="1"><stop stop-color="#06131f"/><stop offset=".55" stop-color="#123a44"/><stop offset="1" stop-color="#2d6158"/></linearGradient>'
            '<radialGradient id="nMoon"><stop stop-color="#fdf9df"/><stop offset=".35" stop-color="#d8f0d0" stop-opacity=".5"/><stop offset="1" stop-color="#9fd8c0" stop-opacity="0"/></radialGradient>')
    r = random.Random(5)
    isi = ['<path fill="url(#nSky)" d="M0 0h1080v1920H0z"/>',
           "".join(f'<circle cx="{r.randrange(0,1080)}" cy="{r.randrange(0,700)}" r="{r.choice([1.4,2,2.6])}" fill="#fff" opacity="{r.choice([.4,.6,.85])}"/>' for _ in range(80)),
           '<circle cx="540" cy="620" r="300" fill="url(#nMoon)"/><circle cx="540" cy="620" r="92" fill="#fbf6dc"/>']
    for lap, (fill, base, hmin, hmax, step) in enumerate([("#1d4a4c", 1250, 260, 420, 90), ("#143a3d", 1300, 320, 520, 120), ("#0d2b2f", 1360, 380, 620, 150)]):
        x = -60 + lap * 40
        while x < 1140:
            isi.append(pohon_siluet(x, base, r.uniform(hmin, hmax), r.uniform(150, 240), fill))
            x += step + r.uniform(0, 40)
    isi.append('<path fill="#0b2326" d="M0 1340h1080v580H0z"/>')
    isi.append('<path fill="#2d6158" opacity=".35" d="M0 1330c200-30 400-20 540 0s380 24 540-10v40H0z"/>')
    isi.append(kunang_titik(r, 60, 0, 1080, 700, 1500))
    tulis("kunang-jauh", "\n".join(isi), defs)


def kunang_utama():
    defs = ('<linearGradient id="nTrunk" x2="1"><stop stop-color="#3b2a24"/><stop offset=".5" stop-color="#6b4c3a"/><stop offset="1" stop-color="#3b2a24"/></linearGradient>'
            '<radialGradient id="nGlow"><stop stop-color="#ffe9a0" stop-opacity=".9"/><stop offset="1" stop-color="#ffd27a" stop-opacity="0"/></radialGradient>'
            '<filter id="nBlur"><feGaussianBlur stdDeviation="20"/></filter>')
    r = random.Random(9)
    isi = ['<ellipse cx="540" cy="1470" rx="420" ry="70" fill="#000" opacity=".35" filter="url(#nBlur)"/>',
           # batang raksasa + akar
           '<path fill="url(#nTrunk)" d="M440 1470c20-120 30-300 20-520-6-140-30-260-60-340h280c-30 80-54 200-60 340-10 220 0 400 20 520z"/>',
           '<path fill="#3b2a24" d="M440 1470c-60-10-120 0-180 20 70-40 120-70 150-120zM640 1470c60-10 120 0 180 20-70-40-120-70-150-120z"/>',
           # tajuk
           '<g fill="#1f5a4a">' + "".join(f'<circle cx="{540 + 300*math.cos(a):.0f}" cy="{560 + 170*math.sin(a):.0f}" r="{r.randint(110,150)}"/>' for a in [i*math.pi/5 for i in range(10)]) + '<circle cx="540" cy="520" r="230"/></g>',
           '<g fill="#2b7560" opacity=".8">' + "".join(f'<circle cx="{540 + 220*math.cos(a):.0f}" cy="{480 + 120*math.sin(a):.0f}" r="{r.randint(70,100)}"/>' for a in [i*math.pi/4 for i in range(8)]) + '</g>',
           # rumah pohon
           '<path fill="#8a6a4a" stroke="#c9a77a" stroke-width="6" d="M320 980h440v-30H320z"/>',
           '<path fill="#a07a52" stroke="#e0c08f" stroke-width="5" d="M380 950V800h320v150z"/>',
           '<path fill="#5c3e2c" d="M350 810L540 680l190 130z"/>',
           '<path fill="#ffd98a" d="M420 900v-50h50v50zM610 900v-50h50v50z"/>',
           '<path fill="#5c3e2c" d="M515 950v-80h50v80z"/>',
           '<circle cx="540" cy="880" r="200" fill="url(#nGlow)" opacity=".6"/>',
           # pagar teras
           ''.join(f'<path stroke="#c9a77a" stroke-width="5" d="M{x} 980v-40"/>' for x in range(330, 760, 30)),
           '<path stroke="#c9a77a" stroke-width="6" d="M320 940h440"/>',
           # tangga melingkar di batang
           ''.join(f'<path fill="#c9a77a" d="M{460 + (i%2)*120 - 30} {1440 - i*46}h60v10h-60z"/>' for i in range(10)),
           # lentera gantung
           ''.join(f'<path stroke="#5c3e2c" stroke-width="3" d="M{x} 980v{h}"/><rect x="{x-12}" y="{980+h}" width="24" height="32" rx="6" fill="#ffd27a"/><circle cx="{x}" cy="{996+h}" r="40" fill="url(#nGlow)"/>' for x, h in [(350, 60), (430, 110), (650, 90), (730, 50)])]
    isi.append(kunang_titik(r, 50, 200, 880, 400, 1450, 6))
    tulis("kunang-utama", "\n".join(isi), defs)


def kunang_depan():
    r = random.Random(13)
    defs = '<linearGradient id="nFg" x2="1"><stop stop-color="#0a1c1d"/><stop offset="1" stop-color="#1c3b36"/></linearGradient>'
    isi = ['<path fill="url(#nFg)" d="M0 0h170c-20 400 10 900 40 1300 10 220 0 420-20 620H0z"/>',
           '<path fill="url(#nFg)" d="M1080 0H930c20 420-20 920-40 1320-10 200 0 400 20 600h170z"/>',
           '<path fill="#0a1c1d" d="M0 1760c180-50 360-50 540-10s360 40 540-10v180H0z"/>']
    # pakis di bawah
    for x0 in range(-20, 1120, 110):
        for k in range(5):
            a = -math.pi / 2 + (k - 2) * .45
            L = r.uniform(150, 230)
            x1, y1 = x0 + L * math.cos(a), 1900 + L * math.sin(a)
            isi.append(f'<path fill="none" stroke="#1d4a3e" stroke-width="10" stroke-linecap="round" d="M{x0} 1920Q{f((x0+x1)/2 + 30)} {f((1920+y1)/2)} {f(x1)} {f(y1)}"/>')
    # jamur menyala
    for x, y, s in [(240, 1800, 1), (300, 1830, .7), (820, 1810, .9), (880, 1840, .6)]:
        isi.append(f'<path fill="#e8dcc0" d="M{x-8*s} {y}v{-40*s}h{16*s}v{40*s}z"/><path fill="#7fe0d0" d="M{x-34*s} {y-36*s}a{34*s} {26*s} 0 0168 0z"/><circle cx="{x}" cy="{y-40*s}" r="{60*s}" fill="#7fe0d0" opacity=".15"/>')
    # dahan menjulur dari atas
    isi.append('<path fill="none" stroke="#0a1c1d" stroke-width="40" stroke-linecap="round" d="M140 160c160 40 300 20 420-60M940 300c-120 10-220-20-300-90"/>')
    isi.append('<g fill="#123a32">' + "".join(f'<ellipse cx="{x}" cy="{y}" rx="40" ry="18" transform="rotate({rot} {x} {y})"/>' for x, y, rot in [(260, 170, 20), (380, 150, -15), (500, 90, 30), (860, 290, -20), (740, 260, 15), (660, 210, -30)]) + "</g>")
    isi.append(kunang_titik(r, 40, 0, 1080, 200, 1800, 8))
    tulis("kunang-depan", "\n".join(isi), defs)


# ─────────────────────────── TEH: kebun teh berundak & pendopo kayu ───────────────────────────
def teh_jauh():
    defs = ('<linearGradient id="tSky" x2="0" y2="1"><stop stop-color="#cfe3e6"/><stop offset=".55" stop-color="#f1efe1"/><stop offset="1" stop-color="#fbf6e8"/></linearGradient>'
            '<linearGradient id="tGn" x2="0" y2="1"><stop stop-color="#8aa6b0"/><stop offset="1" stop-color="#b8cbc8"/></linearGradient>')
    r = random.Random(17)
    isi = ['<path fill="url(#tSky)" d="M0 0h1080v1920H0z"/>',
           '<circle cx="780" cy="520" r="70" fill="#fff8e6" opacity=".95"/>',
           # gunung api di kejauhan
           '<path fill="url(#tGn)" d="M0 1100l250-180 120 40 170-260 70 0 190 250 130-30 150 120v260H0z"/>',
           '<path fill="#fff" opacity=".55" d="M540 700h70l60 80-40-10-30 30-40-30-40 20z"/>']
    # bukit teh berundak: tiga bukit dengan garis kontur
    for lap, (cy, w, h, c1, c2) in enumerate([(1230, 820, 260, "#8fb36a", "#7ca35a"), (1290, 760, 220, "#7fa95b", "#6b9649"), (1350, 900, 200, "#6f9c4c", "#5b8a3c")]):
        cx = [250, 860, 520][lap]
        isi.append(f'<ellipse cx="{cx}" cy="{cy}" rx="{w/2}" ry="{h}" fill="{c1}"/>')
        for k in range(1, 9):
            ry = h * k / 9
            isi.append(f'<path fill="none" stroke="{c2}" stroke-width="{4 + lap*2}" d="M{cx - w/2*math.sqrt(1-(ry/h)**2):.0f} {cy - h + ry:.0f}Q{cx} {cy - h + ry + 18:.0f} {cx + w/2*math.sqrt(1-(ry/h)**2):.0f} {cy - h + ry:.0f}"/>')
    isi.append('<path fill="#5f8c40" d="M0 1300h1080v620H0z"/>')
    # baris teh di tanah (perspektif)
    for i in range(14):
        y = 1300 + (i ** 1.6) * 9
        isi.append(f'<path fill="none" stroke="#4d7a33" stroke-width="{3 + i*1.5:.0f}" d="M0 {y:.0f}Q540 {y + 10 + i*4:.0f} 1080 {y:.0f}"/>')
    isi.append('<path fill="#d8c4a0" opacity=".9" d="M520 1300h40l250 620H270z"/>')
    # kabut
    for y, op in [(1150, .55), (1260, .45), (1330, .35)]:
        isi.append(f'<path fill="#fff" opacity="{op}" d="M0 {y}c200-30 400-30 540 0s380 30 540 0v50c-200 20-380 10-540 0s-360-10-540 10z"/>')
    tulis("teh-jauh", "\n".join(isi), defs)


def teh_utama():
    defs = ('<linearGradient id="tWood" x2="0" y2="1"><stop stop-color="#b98a5a"/><stop offset="1" stop-color="#7d5634"/></linearGradient>'
            '<linearGradient id="tRoof" x2="0" y2="1"><stop stop-color="#6d4a32"/><stop offset="1" stop-color="#3f2a1c"/></linearGradient>'
            '<filter id="tBlur"><feGaussianBlur stdDeviation="20"/></filter>')
    isi = ['<ellipse cx="540" cy="1470" rx="460" ry="70" fill="#2f4a20" opacity=".3" filter="url(#tBlur)"/>',
           # umpak batu & lantai
           '<path fill="#d9cfbf" stroke="#f3ece0" stroke-width="6" d="M210 1470h660l-30-50H240z"/>',
           '<path fill="#bfb3a0" d="M250 1420h580v-24H250z"/>',
           # soko (tiang)
           ''.join(f'<path fill="url(#tWood)" d="M{x-12} 1396V950h24v446z"/>' for x in (290, 410, 670, 790)),
           ''.join(f'<path fill="#8a6a4a" d="M{x-12} 1396V1000h24v396z"/>' for x in (480, 600)),
           # atap limasan/joglo bersusun
           '<path fill="url(#tRoof)" d="M170 980L320 860h440l150 120z"/>',
           '<path fill="#a88760" d="M170 980h740v16H170z"/>',
           '<path fill="url(#tRoof)" d="M330 870L440 700h200l110 170z"/>',
           '<path fill="url(#tRoof)" d="M450 712L510 600h60l60 112z"/>',
           '<path fill="#e7c98e" d="M500 600h80v-10h-80z"/>',
           # ukiran tumpang sari
           '<path fill="none" stroke="#e7c98e" stroke-width="5" d="M320 860h440M440 704h200"/>',
           # gebyok pintu & meja kursi
           '<path fill="#6b4a30" stroke="#e7c98e" stroke-width="4" d="M470 1396v-210h140v210z"/>',
           '<path fill="none" stroke="#e7c98e" stroke-width="3" d="M540 1186v210M490 1220h100M490 1300h100"/>',
           # lampion & janur
           ''.join(f'<path stroke="#5c3e2c" stroke-width="3" d="M{x} 996v40"/><ellipse cx="{x}" cy="1060" rx="18" ry="24" fill="#f3c46b"/>' for x in (350, 730)),
           '<path fill="none" stroke="#d8c46a" stroke-width="6" d="M260 1000q40 60 0 140M820 1000q-40 60 0 140"/>']
    # pot bunga & semak teh di depan pendopo
    r = random.Random(23)
    for x in list(range(200, 330, 40)) + list(range(760, 900, 40)):
        isi.append(f'<circle cx="{x}" cy="{1440 - r.randint(0, 10)}" r="{r.randint(22, 30)}" fill="#5b8a3c"/>')
    tulis("teh-utama", "\n".join(isi), defs)


def teh_depan():
    r = random.Random(29)
    isi = []
    # cabang pohon dari kiri atas
    isi.append('<path fill="none" stroke="#4a3526" stroke-width="44" stroke-linecap="round" d="M-20 340c200 30 330 120 420 260"/>')
    isi.append('<path fill="none" stroke="#4a3526" stroke-width="20" stroke-linecap="round" d="M200 380c40-60 120-100 210-110M300 470c80 0 150 30 200 70"/>')
    for _ in range(46):
        x, y = r.uniform(-40, 520), r.uniform(220, 620)
        isi.append(f'<ellipse cx="{x:.0f}" cy="{y:.0f}" rx="{r.uniform(26,40):.0f}" ry="{r.uniform(12,18):.0f}" fill="{r.choice(["#3f6b2c", "#56853b", "#6d9c4a"])}" transform="rotate({r.randint(-40,40)} {x:.0f} {y:.0f})"/>')
    # perdu teh membulat di kiri-kanan bawah (bertingkat → kedalaman)
    for row, (y, rr, n) in enumerate([(1560, 70, 4), (1700, 100, 4), (1860, 140, 4)]):
        for side in (-1, 1):
            for i in range(n):
                x = 540 + side * (300 + i * rr * 1.25 - row * 40)
                isi.append(f'<ellipse cx="{x:.0f}" cy="{y}" rx="{rr*1.1:.0f}" ry="{rr*.75:.0f}" fill="{["#5b8a3c", "#4d7a33", "#3f6b2c"][row]}"/>')
                for _ in range(6):
                    dx, dy = r.uniform(-rr, rr), r.uniform(-rr*.6, 0)
                    isi.append(f'<ellipse cx="{x+dx:.0f}" cy="{y+dy:.0f}" rx="{rr*.18:.0f}" ry="{rr*.1:.0f}" fill="#8fbf5e"/>')
    isi.append('<path fill="#fff" opacity=".35" d="M0 1880c240-30 520-30 1080 0v40H0z"/>')
    tulis("teh-depan", "\n".join(isi))


# ─────────────────────────── PROPERTI LENSA (tema Jelajah 3D, web/jelajah.rs) ───────────────────────────
# Kanvas 800×800, tanah y≈760, pusat x=400 — sepasang mempelai (insan-*) berdiri
# di depannya. Warna netral (putih/kayu/bunga pastel) agar cocok di semua dunia.
def tulis_prop(nama, isi, defs=""):
    svg = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 800">'
    if defs:
        svg += f"<defs>{defs}</defs>"
    svg += isi + "</svg>\n"
    with open(os.path.join(OUT, f"{nama}.svg"), "w") as fh:
        fh.write(svg)


BUNGA = ["#f4a7b9", "#fff", "#f8d6a0", "#e98aa6", "#fbe3ec"]


def rumpun(r, cx, cy, n, sebar, rmin=10, rmax=18, daun="#7fae6a"):
    out = []
    for _ in range(n):
        x, y = cx + r.uniform(-sebar, sebar), cy + r.uniform(-sebar * .6, sebar * .6)
        out.append(f'<circle cx="{x:.0f}" cy="{y:.0f}" r="{r.uniform(rmin, rmax) * 1.3:.0f}" fill="{daun}"/>')
    for _ in range(n):
        x, y = cx + r.uniform(-sebar, sebar), cy + r.uniform(-sebar * .6, sebar * .6)
        rr = r.uniform(rmin, rmax)
        out.append(f'<circle cx="{x:.0f}" cy="{y:.0f}" r="{rr:.0f}" fill="{r.choice(BUNGA)}"/><circle cx="{x:.0f}" cy="{y:.0f}" r="{rr*.35:.0f}" fill="#f6c76b"/>')
    return "".join(out)


def jl_lengkung():
    r = random.Random(101)
    isi = ['<ellipse cx="400" cy="764" rx="300" ry="26" fill="#000" opacity=".12"/>',
           '<path fill="none" stroke="#fff" stroke-width="22" d="M170 760V420C170 250 280 150 400 150s230 100 230 270v340"/>',
           '<path fill="none" stroke="#e8dccb" stroke-width="6" d="M170 760V420C170 250 280 150 400 150s230 100 230 270v340"/>']
    for i in range(30):
        t = i / 29
        if t < .3:
            x, y = 170, 760 - 340 * t / .3
        elif t < .7:
            a = math.pi * (1 - (t - .3) / .4)
            x, y = 400 + 230 * math.cos(a), 420 - 270 * math.sin(a)
        else:
            x, y = 630, 420 + 340 * (t - .7) / .3
        if t < .2 or t > .8 or .35 < t < .65:
            isi.append(rumpun(r, x, y, 3, 26))
    isi.append(rumpun(r, 400, 160, 6, 60))
    tulis_prop("jl-lengkung", "".join(isi))


def jl_bangku():
    r = random.Random(102)
    isi = ['<ellipse cx="400" cy="764" rx="300" ry="24" fill="#000" opacity=".12"/>',
           '<path fill="#a87a4e" d="M180 600h440v26H180zM190 540h420v22H190zM190 490h420v22H190z"/>',
           '<path fill="#7d5634" d="M200 626h20v134h-20zM580 626h20v134h-20zM200 480h16v146h-16zM584 480h16v146h-16z"/>',
           # pot bunga kiri-kanan
           '<path fill="#e3c9a8" d="M70 680h90l-14 80H84zM640 680h90l-14 80h-62z"/>',
           rumpun(r, 115, 640, 6, 40), rumpun(r, 685, 640, 6, 40),
           # pohon kecil belakang
           '<path fill="#7d5634" d="M392 480V200h16v280z"/>', rumpun(r, 400, 190, 14, 120, 14, 22, "#6f9f5c")]
    tulis_prop("jl-bangku", "".join(isi))


def jl_ayunan():
    r = random.Random(103)
    isi = ['<ellipse cx="400" cy="764" rx="320" ry="26" fill="#000" opacity=".12"/>',
           '<path fill="#6b4a30" d="M120 760c20-200 10-420 40-560h40c-20 160-10 360 10 560z"/>',
           '<path fill="none" stroke="#6b4a30" stroke-width="34" stroke-linecap="round" d="M170 230c120-40 300-50 470-10"/>',
           rumpun(r, 400, 170, 22, 230, 14, 24, "#6f9f5c"),
           '<path stroke="#e8dccb" stroke-width="5" d="M300 236v360M500 230v366"/>',
           '<path fill="#fff" stroke="#e8dccb" stroke-width="4" d="M280 596h240v22H280z"/>',
           rumpun(r, 300, 400, 3, 14, 8, 12), rumpun(r, 500, 380, 3, 14, 8, 12)]
    tulis_prop("jl-ayunan", "".join(isi))


def jl_meja():
    r = random.Random(104)
    isi = ['<ellipse cx="400" cy="764" rx="300" ry="24" fill="#000" opacity=".12"/>',
           '<path fill="#fff" stroke="#eadfd2" stroke-width="4" d="M160 560h480v40H160z"/>',
           '<path fill="#fbf6ee" d="M170 600h460l-30 160H200z"/>',
           '<path fill="none" stroke="#eadfd2" stroke-width="4" d="M220 600q30 60 0 160M580 600q-30 60 0 160M400 600v160"/>',
           # kue bertingkat
           '<path fill="#fff" stroke="#f0d9c8" stroke-width="4" d="M320 560v-70h160v70zM345 490v-60h110v60zM370 430v-50h60v50z"/>',
           '<path fill="none" stroke="#f4a7b9" stroke-width="6" d="M320 520h160M345 460h110M370 404h60"/>',
           rumpun(r, 400, 372, 3, 16, 7, 10),
           # lilin & vas
           '<path fill="#f8e6c4" d="M220 560v-60h14v60zM566 560v-60h14v60z"/><path fill="#f6c76b" d="M227 494c-8-10 0-20 0-20s8 10 0 20zM573 494c-8-10 0-20 0-20s8 10 0 20z"/>',
           '<path fill="#dfe9f0" d="M250 560c-10-40 10-60 10-90h30c0 30 20 50 10 90z"/>', rumpun(r, 275, 440, 6, 34)]
    tulis_prop("jl-meja", "".join(isi))


def jl_lampu():
    isi = ['<ellipse cx="400" cy="764" rx="320" ry="24" fill="#000" opacity=".12"/>',
           '<path fill="#8a6646" d="M110 760V200h18v560zM672 760V200h18v560z"/>',
           '<path fill="none" stroke="#5a4636" stroke-width="3" d="M119 210Q400 360 681 210M119 290Q400 430 681 290"/>']
    for k, (y0, dy) in enumerate([(210, 150), (290, 140)]):
        for i in range(1, 12):
            t = i / 12
            x = 119 + 562 * t
            y = y0 + dy * 4 * t * (1 - t) * .5 * 2 * .5 * 2 * .5
            y = y0 + (dy) * (1 - (2 * t - 1) ** 2) * .5 * 2 * .5 * 2
            isi.append(f'<circle cx="{x:.0f}" cy="{y + 10:.0f}" r="9" fill="{["#ffe39a", "#ffd1dc", "#fff4c8"][(i + k) % 3]}"/><circle cx="{x:.0f}" cy="{y + 10:.0f}" r="20" fill="#ffe39a" opacity=".25"/>')
    isi.append('<path fill="#e3c9a8" d="M60 700h110l-16 60H76zM630 700h110l-16 60h-78z"/>')
    r = random.Random(105)
    isi.append(rumpun(r, 115, 670, 6, 40))
    isi.append(rumpun(r, 685, 670, 6, 40))
    tulis_prop("jl-lampu", "".join(isi))


def jl_hati():
    r = random.Random(106)
    isi = ['<ellipse cx="400" cy="764" rx="280" ry="24" fill="#000" opacity=".12"/>',
           '<path fill="#8a6646" d="M392 760V640h16v120z"/>',
           '<path fill="none" stroke="#fff" stroke-width="20" d="M400 640C250 540 150 450 150 330c0-90 70-150 140-150 50 0 90 30 110 70 20-40 60-70 110-70 70 0 140 60 140 150 0 120-100 210-250 310z"/>',
           '<path fill="none" stroke="#f0d9c8" stroke-width="5" d="M400 640C250 540 150 450 150 330c0-90 70-150 140-150 50 0 90 30 110 70 20-40 60-70 110-70 70 0 140 60 140 150 0 120-100 210-250 310z"/>']
    for a in range(0, 360, 24):
        t = math.radians(a)
        x = 400 + 16 * 15 * math.sin(t) ** 3
        y = 390 - 15 * (13 * math.cos(t) - 5 * math.cos(2 * t) - 2 * math.cos(3 * t) - math.cos(4 * t))
        isi.append(rumpun(r, x, y, 2, 14, 9, 14))
    tulis_prop("jl-hati", "".join(isi))


if __name__ == "__main__":
    for fn in (balon_jauh, balon_utama, balon_depan, kastil_jauh, kastil_utama, kastil_depan,
               kunang_jauh, kunang_utama, kunang_depan, teh_jauh, teh_utama, teh_depan,
               jl_lengkung, jl_bangku, jl_ayunan, jl_meja, jl_lampu, jl_hati):
        fn()
    print("12 lapisan + 6 properti lensa ditulis ke", os.path.normpath(OUT))
