#!/usr/bin/env python3
"""Pembangkit lapisan PANGGUNG 3D (src/web/panggung/svg/*.svg).

Tiap adegan = 3 lapisan berkedalaman (viewBox potret 1080×1920):
  {adegan}-jauh   cakrawala jauh (paling lambat saat kamera maju)
  {adegan}-utama  bangunan utama di tengah
  {adegan}-depan  gerbang depan yang DITEMBUS kamera (bukaan transparan)
Gaya "clay render" putih: cahaya dari kiri atas → sisi kiri terang, kanan
redup, ceruk lebih gelap. Jalankan ulang setelah mengubah gambar:
    python3 scripts/gerak/panggung.py
"""
import math, os
OUT = os.path.join(os.path.dirname(__file__), '..', '..', 'src', 'web', 'panggung', 'svg')
W, H = 1080, 1920

DEFS = '''<defs>
<linearGradient id="w" x1="0" x2="1"><stop offset="0" stop-color="#ffffff"/><stop offset=".55" stop-color="#f3f4f6"/><stop offset="1" stop-color="#dfe2e7"/></linearGradient>
<linearGradient id="wv" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#ffffff"/><stop offset="1" stop-color="#e6e8ec"/></linearGradient>
<linearGradient id="s" x1="0" x2="1"><stop offset="0" stop-color="#e3e6ea"/><stop offset="1" stop-color="#c9ced5"/></linearGradient>
<linearGradient id="r" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#b9bfc8"/><stop offset="1" stop-color="#d6dade"/></linearGradient>
<radialGradient id="dome" cx=".35" cy=".3" r=".85"><stop offset="0" stop-color="#ffffff"/><stop offset=".6" stop-color="#eceef1"/><stop offset="1" stop-color="#c8cdd4"/></radialGradient>
<linearGradient id="gold" x1="0" x2="1"><stop offset="0" stop-color="#8a6a2a"/><stop offset=".45" stop-color="#f1d47c"/><stop offset="1" stop-color="#9b7a2e"/></linearGradient>
<linearGradient id="air" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#c9d3dc"/><stop offset=".5" stop-color="#e8edf1"/><stop offset="1" stop-color="#b8c4ce"/></linearGradient>
<linearGradient id="hijau" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#7d9b6a"/><stop offset="1" stop-color="#46603c"/></linearGradient>
<pattern id="kisi" width="22" height="22" patternUnits="userSpaceOnUse"><rect width="22" height="22" fill="url(#dome)"/><path d="M11 3 L19 11 L11 19 L3 11Z" fill="#b6bcc5" opacity=".55"/></pattern>
</defs>'''

def svg(name, body):
    with open(os.path.join(OUT, name + '.svg'), 'w') as f:
        f.write(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}">{DEFS}{body}</svg>\n')

def f(x): return f'{x:.1f}'.rstrip('0').rstrip('.')

# ── bentuk dasar ─────────────────────────────────────────────────────────
def lengkung_runcing(cx, y_kaki, lebar, tinggi_kaki, puncak):
    """Path lengkung runcing (ogive) tertutup: kaki di y_kaki, lebar, tinggi tiang lurus, tinggi puncak."""
    l, r = cx - lebar / 2, cx + lebar / 2
    yb = y_kaki - tinggi_kaki
    yp = yb - puncak
    return (f'M{f(l)} {f(y_kaki)}L{f(l)} {f(yb)}C{f(l)} {f(yb - puncak*.55)} {f(cx - lebar*.18)} {f(yp + puncak*.18)} {f(cx)} {f(yp)}'
            f'C{f(cx + lebar*.18)} {f(yp + puncak*.18)} {f(r)} {f(yb - puncak*.55)} {f(r)} {f(yb)}L{f(r)} {f(y_kaki)}Z')

def kubah(cx, y_dasar, lebar, tinggi, fill='url(#dome)', ujung=True):
    """Kubah bawang: melebar sedikit lalu meruncing ke atas."""
    l, r = cx - lebar / 2, cx + lebar / 2
    yt = y_dasar - tinggi
    p = (f'<path d="M{f(l)} {f(y_dasar)}C{f(l - lebar*.08)} {f(y_dasar - tinggi*.45)} {f(cx - lebar*.32)} {f(yt + tinggi*.18)} {f(cx)} {f(yt)}'
         f'C{f(cx + lebar*.32)} {f(yt + tinggi*.18)} {f(r + lebar*.08)} {f(y_dasar - tinggi*.45)} {f(r)} {f(y_dasar)}Z" fill="{fill}"/>')
    if ujung:
        p += (f'<rect x="{f(cx-2.5)}" y="{f(yt-38)}" width="5" height="40" fill="url(#gold)"/>'
              f'<circle cx="{f(cx)}" cy="{f(yt-8)}" r="7" fill="url(#gold)"/><circle cx="{f(cx)}" cy="{f(yt-44)}" r="5" fill="url(#gold)"/>')
    return p

def menara(cx, y_kaki, tinggi, lebar=46):
    """Menara (minaret): batang ber-gradien, 3 balkon, puncak kerucut + ujung emas."""
    o = []
    yt = y_kaki - tinggi
    o.append(f'<rect x="{f(cx-lebar/2)}" y="{f(yt+60)}" width="{f(lebar)}" height="{f(tinggi-60)}" fill="url(#w)"/>')
    for k in range(3):
        yb = yt + 60 + (tinggi - 60) * (0.18 + k * 0.27)
        o.append(f'<rect x="{f(cx-lebar*.85)}" y="{f(yb)}" width="{f(lebar*1.7)}" height="14" rx="3" fill="url(#w)"/>'
                 f'<rect x="{f(cx-lebar*.85)}" y="{f(yb+14)}" width="{f(lebar*1.7)}" height="5" fill="#cfd4da"/>')
        for j in range(5):
            x = cx - lebar * .65 + j * lebar * .325
            o.append(f'<rect x="{f(x-2)}" y="{f(yb-18)}" width="4" height="18" fill="#e1e4e8"/>')
    o.append(f'<path d="M{f(cx-lebar*.6)} {f(yt+62)}L{f(cx)} {f(yt-10)}L{f(cx+lebar*.6)} {f(yt+62)}Z" fill="url(#w)"/>')
    o.append(f'<rect x="{f(cx-2)}" y="{f(yt-46)}" width="4" height="40" fill="url(#gold)"/><circle cx="{f(cx)}" cy="{f(yt-50)}" r="5" fill="url(#gold)"/>')
    # jendela kecil
    for k in range(4):
        y = yt + 120 + k * (tinggi - 200) / 4
        o.append(f'<path d="{lengkung_runcing(cx, y + 34, 14, 22, 12)}" fill="url(#r)"/>')
    return ''.join(o)

# ══ MASJID ═══════════════════════════════════════════════════════════════
def masjid():
    # jauh: cakrawala kota putih berkabut + dua menara tinggi di sisi
    o = []
    for i, (x, h) in enumerate([(60, 120), (150, 90), (250, 140), (330, 80), (760, 110), (850, 150), (950, 95), (1030, 130)]):
        o.append(f'<rect x="{x-30}" y="{1310-h}" width="60" height="{h}" fill="#e8ebef"/>')
        o.append(kubah(x, 1310 - h, 54, 46, '#eef0f3', ujung=False))
    o.append(f'<rect x="0" y="1300" width="{W}" height="{H-1300}" fill="#e9ecf0"/>')
    o.append(menara(150, 1320, 900, 44))
    o.append(menara(930, 1320, 900, 44))
    svg('masjid-jauh', f'<g opacity=".92">{"".join(o)}</g>')

    # utama: paviliun segi delapan berkubah kisi, kubah-kubah kecil, kolam
    o = []
    cx = 540
    o.append(f'<rect x="0" y="1360" width="{W}" height="{H-1360}" fill="#eef0f2"/>')
    # sayap rendah kiri/kanan
    for sx in (-1, 1):
        x0 = cx + sx * 330
        o.append(f'<rect x="{f(x0-150)}" y="1150" width="300" height="210" fill="url({"#w" if sx < 0 else "#s"})"/>')
        for j in range(3):
            o.append(f'<path d="{lengkung_runcing(x0 - 90 + j * 90, 1340, 50, 90, 40)}" fill="url(#r)"/>')
        o.append(kubah(x0, 1150, 150, 120))
    # badan: muka tengah + dua muka samping (segi delapan)
    o.append(f'<path d="M{cx-250} 1300L{cx-170} 1290L{cx-170} 960L{cx-250} 975Z" fill="url(#w)"/>')
    o.append(f'<path d="M{cx+250} 1300L{cx+170} 1290L{cx+170} 960L{cx+250} 975Z" fill="url(#s)"/>')
    o.append(f'<rect x="{cx-170}" y="960" width="340" height="330" fill="url(#wv)"/>')
    for x in (cx - 170, cx + 170, cx - 250, cx + 250):
        o.append(f'<rect x="{x-9}" y="960" width="18" height="340" fill="#e7e9ed"/>')
    o.append(f'<path d="{lengkung_runcing(cx, 1290, 120, 150, 90)}" fill="url(#r)"/>')
    o.append(f'<rect x="{cx-38}" y="1170" width="76" height="120" fill="#6f625a"/>')
    for sx in (-1, 1):
        o.append(f'<path d="{lengkung_runcing(cx + sx*110, 1120, 56, 80, 44)}" fill="url(#r)"/>')
        o.append(f'<path d="{lengkung_runcing(cx + sx*210, 1130, 44, 70, 34)}" fill="#c6ccd3"/>')
    # lis atas + parapet
    o.append(f'<rect x="{cx-262}" y="940" width="524" height="26" fill="url(#w)"/>')
    for j in range(22):
        x = cx - 250 + j * 24
        o.append(f'<path d="M{x} 940l12 -18 12 18Z" fill="#f4f5f7"/>')
    # kubah kecil di sudut
    for x in (cx - 240, cx - 120, cx + 120, cx + 240):
        o.append(f'<rect x="{x-22}" y="880" width="44" height="60" fill="url(#w)"/>' + kubah(x, 880, 64, 58))
    # drum + kubah kisi besar
    o.append(f'<rect x="{cx-150}" y="830" width="300" height="110" fill="url(#w)"/>')
    for j in range(7):
        o.append(f'<path d="{lengkung_runcing(cx - 120 + j * 40, 925, 22, 40, 18)}" fill="url(#r)"/>')
    o.append(kubah(cx, 830, 330, 300, 'url(#kisi)'))
    o.append(kubah(cx, 830, 330, 300, 'url(#dome)', ujung=True).replace('fill="url(#dome)"', 'fill="url(#dome)" opacity=".35"'))
    # tangga & kolam
    for k in range(4):
        o.append(f'<rect x="{cx-300-k*20}" y="{1300+k*16}" width="{600+k*40}" height="16" fill="{"#f6f7f8" if k % 2 == 0 else "#e2e5e9"}"/>')
    o.append(f'<path d="M140 1380H940L1000 1470H80Z" fill="url(#air)"/>')
    o.append(f'<path d="M500 1470H580L640 1920H440Z" fill="#f2f3f5"/>')
    for sx in (-1, 1):
        o.append(f'<path d="M{540+sx*70} 1470L{540+sx*120} 1920H{540+sx*420}L{540+sx*300} 1470Z" fill="url(#hijau)" opacity=".85"/>')
    svg('masjid-utama', ''.join(o))

    # depan: rumah gerbang putih dengan LENGKUNG RUNCING besar (dibolongi)
    o = []
    bukaan = lengkung_runcing(540, 1560, 400, 520, 330)
    dinding = f'M150 1600V560H930V1600Z'
    o.append(f'<path fill-rule="evenodd" d="{dinding}{bukaan}" fill="url(#wv)"/>')
    # bingkai ukir di tepi bukaan (garis-garis konsentris)
    for k, c in enumerate(['#d4d8de', '#f7f8f9', '#c7ccd3']):
        o.append(f'<path d="{lengkung_runcing(540, 1560, 400 + 24 + k*22, 520, 330 + 16 + k*12)}" fill="none" stroke="{c}" stroke-width="9"/>')
    # pilaster & ceruk kecil
    for sx in (-1, 1):
        x = 540 + sx * 330
        o.append(f'<rect x="{x-55}" y="560" width="110" height="1040" fill="url({"#w" if sx < 0 else "#s"})"/>')
        for k in range(5):
            o.append(f'<path d="{lengkung_runcing(x, 760 + k*170, 44, 70, 34)}" fill="url(#r)"/>')
        o.append(f'<rect x="{x-70}" y="500" width="140" height="70" fill="url(#w)"/>' + kubah(x, 500, 120, 110))
    o.append(f'<rect x="130" y="540" width="820" height="30" fill="url(#w)"/>')
    for j in range(34):
        x = 140 + j * 24
        o.append(f'<path d="M{x} 540l12 -20 12 20Z" fill="#f2f3f5"/>')
    o.append(kubah(540, 520, 220, 190))
    # anak tangga di bawah bukaan
    for k in range(4):
        o.append(f'<rect x="{110-k*30}" y="{1600+k*22}" width="{860+k*60}" height="22" fill="{"#f6f7f8" if k % 2 == 0 else "#dde1e5"}"/>')
    o.append(f'<rect x="0" y="1688" width="{W}" height="{H-1688}" fill="#eceef1"/>')
    svg('masjid-depan', ''.join(o))


# ══ JAWA: pendopo joglo + candi bentar ══════════════════════════════════
def jawa():
    o = []
    # gunung berkabut + pepohonan kelapa
    o.append('<path d="M-40 1330L250 900Q330 840 410 900L700 1330Z" fill="#dfe3e6"/>')
    o.append('<path d="M380 1330L690 960Q760 900 830 960L1120 1330Z" fill="#e6e9ec"/>')
    for x, h in [(80, 380), (170, 300), (960, 360), (1040, 280)]:
        o.append(f'<path d="M{x} 1330Q{x+10} {1330-h*.6} {x+4} {1330-h}" stroke="#cfd4d8" stroke-width="10" fill="none"/>')
        for a in range(7):
            ang = -150 + a * 50
            dx, dy = math.cos(math.radians(ang)) * 90, math.sin(math.radians(ang)) * 50
            o.append(f'<path d="M{x+4} {1330-h}Q{f(x+4+dx*.5)} {f(1330-h+dy*.2-30)} {f(x+4+dx)} {f(1330-h+dy+20)}" stroke="#d3d8dc" stroke-width="12" fill="none" stroke-linecap="round"/>')
    o.append(f'<rect x="0" y="1320" width="{W}" height="{H-1320}" fill="#e9ecee"/>')
    svg('jawa-jauh', ''.join(o))

    o = []
    cx = 540
    o.append(f'<rect x="0" y="1380" width="{W}" height="{H-1380}" fill="#efeee9"/>')
    # umpak / lantai bertingkat
    for k in range(4):
        o.append(f'<rect x="{cx-400-k*24}" y="{1290+k*24}" width="{800+k*48}" height="24" fill="{"#fbfaf6" if k % 2 == 0 else "#e4e1d8"}"/>')
    # gebyok ukir di belakang tiang
    o.append(f'<rect x="{cx-300}" y="1070" width="600" height="220" fill="#d9cdb8"/>')
    for j in range(10):
        o.append(f'<rect x="{cx-290+j*60}" y="1080" width="50" height="200" fill="none" stroke="#c2ab84" stroke-width="3"/>')
        o.append(f'<path d="M{cx-265+j*60} 1100q-12 40 0 80q12 40 0 80" stroke="#b39566" stroke-width="3" fill="none"/>')
    o.append(f'<path d="{lengkung_runcing(cx, 1290, 110, 120, 60)}" fill="#7d6450"/>')
    # tiang (saka) gading berkepala emas
    for x in (cx - 360, cx - 220, cx - 80, cx + 80, cx + 220, cx + 360):
        o.append(f'<rect x="{x-14}" y="1050" width="28" height="240" fill="url(#w)"/><rect x="{x-20}" y="1040" width="40" height="16" fill="url(#gold)"/><rect x="{x-22}" y="1284" width="44" height="12" fill="#d9d4c7"/>')
    # atap joglo tiga susun (brunjung)
    o.append(f'<path d="M{cx-470} 1060L{cx-300} 950H{cx+300}L{cx+470} 1060Z" fill="#9b8571"/>')
    o.append(f'<path d="M{cx-470} 1060H{cx+470}V1074H{cx-470}Z" fill="url(#gold)"/>')
    o.append(f'<path d="M{cx-300} 952L{cx-150} 760H{cx+150}L{cx+300} 952Z" fill="#8a735f"/>')
    o.append(f'<path d="M{cx-150} 762L{cx-60} 650H{cx+60}L{cx+150} 762Z" fill="#7a6553"/>')
    for y, w in ((950, 600), (760, 300)):
        o.append(f'<rect x="{cx-w/2}" y="{y-4}" width="{w}" height="10" fill="url(#gold)"/>')
    # genteng: garis-garis
    for k in range(18):
        x = cx - 440 + k * 52
        o.append(f'<path d="M{x} 1058L{f(cx-290+k*34)} 954" stroke="#7f6a57" stroke-width="2"/>')
    # mustaka emas di puncak
    o.append(f'<path d="M{cx-26} 652Q{cx} 560 {cx+26} 652Z" fill="url(#gold)"/><circle cx="{cx}" cy="590" r="8" fill="url(#gold)"/>')
    o.append(f'<path d="M{cx-60} 650Q{cx-90} 620 {cx-70} 600" stroke="url(#gold)" stroke-width="6" fill="none"/><path d="M{cx+60} 650Q{cx+90} 620 {cx+70} 600" stroke="url(#gold)" stroke-width="6" fill="none"/>')
    # jalan setapak & taman
    o.append(f'<path d="M490 1386H590L660 1920H420Z" fill="#f6f4ee"/>')
    for sx in (-1, 1):
        o.append(f'<path d="M{540+sx*70} 1386L{540+sx*130} 1920H{540+sx*440}L{540+sx*320} 1386Z" fill="url(#hijau)" opacity=".8"/>')
    svg('jawa-utama', ''.join(o))

    # candi bentar: dua menara bertingkat simetris, celah di tengah = jalan masuk
    o = []
    def separuh(sx):
        q = []
        x_in, x_out = 540 + sx * 95, 540 + sx * 470   # tepi dalam (celah) & luar
        tingkat = [(1600, 1340), (1340, 1150), (1150, 990), (990, 860), (860, 760), (760, 680), (680, 620)]
        for i, (yb, yt) in enumerate(tingkat):
            menyempit = i * 38
            xo = x_out - sx * menyempit
            xi = x_in
            l, r = (xo, xi) if sx < 0 else (xi, xo)
            fill = 'url(#w)' if sx < 0 else 'url(#s)'
            q.append(f'<path d="M{f(l)} {yb}L{f(l + (sx < 0) * 0 )} {yt}H{f(r)}V{yb}Z" fill="{fill}"/>')
            q.append(f'<rect x="{f(min(l, r) - 8)}" y="{yt - 10}" width="{f(abs(r - l) + 16)}" height="14" fill="url(#gold)" opacity=".85"/>')
            # relief: ceruk & sulur
            if i < 3:
                mx = (l + r) / 2
                q.append(f'<rect x="{f(mx-50)}" y="{yt+30}" width="100" height="{yb-yt-60}" fill="url(#r)" opacity=".7"/>')
                q.append(f'<path d="M{f(mx-30)} {yt+60}q30 40 0 80q-30 40 0 80" stroke="#b9a983" stroke-width="5" fill="none"/>')
        # puncak berbentuk kuncup
        xm = (x_in + x_out) / 2 + sx * 40
        q.append(f'<path d="M{f(xm-50)} 620Q{f(xm)} 480 {f(xm+50)} 620Z" fill="url(#w)"/><circle cx="{f(xm)}" cy="500" r="9" fill="url(#gold)"/>')
        return ''.join(q)
    o.append(separuh(-1) + separuh(1))
    for k in range(4):
        o.append(f'<rect x="{60-k*30}" y="{1600+k*22}" width="{960+k*60}" height="22" fill="{"#f8f7f3" if k % 2 == 0 else "#e2dfd6"}"/>')
    o.append(f'<rect x="0" y="1688" width="{W}" height="{H-1688}" fill="#eeede8"/>')
    svg('jawa-depan', ''.join(o))

# ══ TIONGHOA: pagoda + aula + tembok GERBANG BULAN ══════════════════════
def atap_cina(cx, y, lebar, tinggi, warna='#5d4a46'):
    l, r = cx - lebar / 2, cx + lebar / 2
    return (f'<path d="M{f(l-40)} {f(y-30)}Q{f(l)} {f(y)} {f(l+60)} {f(y)}H{f(r-60)}Q{f(r)} {f(y)} {f(r+40)} {f(y-30)}'
            f'L{f(r-30)} {f(y-tinggi*.4)}Q{f(cx)} {f(y-tinggi*1.1)} {f(l+30)} {f(y-tinggi*.4)}Z" fill="{warna}"/>'
            f'<path d="M{f(l-40)} {f(y-30)}Q{f(l)} {f(y)} {f(l+60)} {f(y)}H{f(r-60)}Q{f(r)} {f(y)} {f(r+40)} {f(y-30)}" stroke="url(#gold)" stroke-width="6" fill="none"/>')

def tionghoa():
    o = []
    o.append('<path d="M-40 1330Q200 1060 460 1240Q700 1080 1120 1330Z" fill="#e2e5e8"/>')
    # pagoda 7 tingkat di kiri jauh
    px, y = 250, 1300
    for i in range(7):
        lb = 230 - i * 24
        o.append(f'<rect x="{px-lb/2+20}" y="{y-90}" width="{lb-40}" height="90" fill="#eceef0"/>')
        o.append(atap_cina(px, y - 90, lb, 50, '#d4d0cd').replace('url(#gold)', '#e3d3a6'))
        y -= 110
    o.append(f'<rect x="{px-3}" y="{y-60}" width="6" height="80" fill="#e3d3a6"/>')
    o.append(f'<rect x="0" y="1320" width="{W}" height="{H-1320}" fill="#eaedef"/>')
    svg('tionghoa-jauh', ''.join(o))

    o = []
    cx = 600
    o.append(f'<rect x="0" y="1380" width="{W}" height="{H-1380}" fill="#efefed"/>')
    for k in range(3):
        o.append(f'<rect x="{cx-380-k*24}" y="{1300+k*26}" width="{760+k*48}" height="26" fill="{"#fbfbf9" if k % 2 == 0 else "#e2e2de"}"/>')
    o.append(f'<rect x="{cx-330}" y="1040" width="660" height="260" fill="url(#wv)"/>')
    for x in range(cx - 300, cx + 301, 120):
        o.append(f'<rect x="{x-14}" y="1040" width="28" height="260" fill="#a8282d"/>')
    for x in range(cx - 240, cx + 241, 120):
        o.append(f'<rect x="{x-44}" y="1090" width="88" height="200" fill="#efe6d8" stroke="#a8282d" stroke-width="4"/>')
        for j in range(4):
            o.append(f'<path d="M{x-44} {1110+j*45}h88M{x} 1090v200" stroke="#c9a17a" stroke-width="2"/>')
    o.append(atap_cina(cx, 1045, 760, 150))
    o.append(f'<rect x="{cx-230}" y="900" width="460" height="70" fill="url(#wv)"/>')
    o.append(atap_cina(cx, 905, 540, 120))
    o.append(f'<path d="M{cx-200} 832Q{cx-230} 800 {cx-210} 780M{cx+200} 832Q{cx+230} 800 {cx+210} 780" stroke="url(#gold)" stroke-width="8" fill="none"/><circle cx="{cx}" cy="800" r="14" fill="url(#gold)"/>')
    o.append(f'<path d="M540 1386H640L700 1920H460Z" fill="#f6f6f4"/>')
    svg('tionghoa-utama', ''.join(o))

    o = []
    bulan = 'M540 840A300 300 0 1 0 540.1 840Z'
    o.append(f'<path fill-rule="evenodd" d="M0 700H1080V1640H0Z M540 860a300 300 0 1 1 -0.1 0Z" fill="url(#wv)"/>')
    o.append('<circle cx="540" cy="1160" r="312" fill="none" stroke="#cdd1d6" stroke-width="16"/><circle cx="540" cy="1160" r="330" fill="none" stroke="#f7f7f8" stroke-width="10"/>')
    # atap genteng di atas tembok
    o.append('<path d="M-40 720Q540 660 1120 720L1120 690Q540 610 -40 690Z" fill="#5d4a46"/><path d="M-40 720Q540 660 1120 720" stroke="url(#gold)" stroke-width="6" fill="none"/>')
    for x in range(0, 1081, 40):
        o.append(f'<path d="M{x} 700v-30" stroke="#463735" stroke-width="3"/>')
    # jendela kisi bulat kecil di kanan-kiri
    for x in (130, 950):
        o.append(f'<circle cx="{x}" cy="1050" r="70" fill="#c9ced4"/><path d="M{x-70} 1050h140M{x} 980v140M{x-50} 1000l100 100M{x+50} 1000l-100 100" stroke="#eef0f2" stroke-width="6"/>')
    # lampion merah tergantung
    for x in (300, 780):
        o.append(f'<path d="M{x} 700V780" stroke="#7a5b3a" stroke-width="3"/><ellipse cx="{x}" cy="830" rx="44" ry="54" fill="#c62828"/><ellipse cx="{x}" cy="830" rx="44" ry="54" fill="none" stroke="#8e1b1b" stroke-width="3"/><rect x="{x-20}" y="772" width="40" height="10" fill="url(#gold)"/><rect x="{x-20}" y="880" width="40" height="10" fill="url(#gold)"/><path d="M{x} 890v40" stroke="#c62828" stroke-width="6"/>')
    for k in range(4):
        o.append(f'<rect x="{60-k*30}" y="{1640+k*20}" width="{960+k*60}" height="20" fill="{"#f8f8f6" if k % 2 == 0 else "#dfdfdb"}"/>')
    o.append(f'<rect x="0" y="1720" width="{W}" height="{H-1720}" fill="#ededeb"/>')
    svg('tionghoa-depan', ''.join(o))

# ══ KAPEL: kapel putih bermenara + lengkung bunga ═══════════════════════
def kapel():
    o = []
    o.append('<path d="M-40 1330Q300 1120 640 1260Q860 1160 1120 1300V1330Z" fill="#e3e8e3"/>')
    for x, h in [(90, 420), (170, 330), (880, 400), (980, 460), (1050, 340)]:
        o.append(f'<ellipse cx="{x}" cy="{1330-h/2}" rx="34" ry="{h/2}" fill="#cfd8cf"/>')
    o.append(f'<rect x="0" y="1320" width="{W}" height="{H-1320}" fill="#ebeeea"/>')
    svg('kapel-jauh', ''.join(o))

    o = []
    cx = 540
    o.append(f'<rect x="0" y="1380" width="{W}" height="{H-1380}" fill="#eff1ee"/>')
    for sx in (-1, 1):
        x0 = cx + sx * 300
        o.append(f'<rect x="{x0-130}" y="1080" width="260" height="230" fill="url({"#w" if sx < 0 else "#s"})"/><path d="M{x0-150} 1090L{x0} 1000L{x0+150} 1090Z" fill="#dfe2e6"/>')
        for j in (-1, 1):
            o.append(f'<path d="{lengkung_runcing(x0 + j*55, 1260, 46, 110, 40)}" fill="url(#r)"/>')
    o.append(f'<rect x="{cx-180}" y="960" width="360" height="350" fill="url(#wv)"/>')
    o.append(f'<path d="M{cx-200} 975L{cx} 780L{cx+200} 975Z" fill="url(#w)"/><path d="M{cx-200} 975L{cx} 780L{cx+200} 975" stroke="#d7dbe0" stroke-width="10" fill="none"/>')
    # jendela mawar
    o.append(f'<circle cx="{cx}" cy="900" r="62" fill="#c7cdd5"/>')
    for a in range(12):
        x2, y2 = cx + 58 * math.cos(a * math.pi / 6), 900 + 58 * math.sin(a * math.pi / 6)
        o.append(f'<path d="M{cx} 900L{f(x2)} {f(y2)}" stroke="#eef0f3" stroke-width="5"/>')
    o.append(f'<circle cx="{cx}" cy="900" r="16" fill="#eef0f3"/>')
    o.append(f'<path d="{lengkung_runcing(cx, 1310, 150, 150, 80)}" fill="url(#r)"/><path d="M{cx} 1080V1310" stroke="#a9b0b9" stroke-width="4"/>')
    o.append(f'<rect x="{cx-70}" y="1160" width="140" height="150" fill="#7a685c"/><path d="M{cx} 1160V1310" stroke="#5b4d44" stroke-width="3"/>')
    # menara lonceng + puncak + salib
    o.append(f'<rect x="{cx-60}" y="600" width="120" height="190" fill="url(#w)"/><path d="{lengkung_runcing(cx, 760, 50, 70, 40)}" fill="url(#r)"/>')
    o.append(f'<path d="M{cx-70} 605L{cx} 380L{cx+70} 605Z" fill="url(#w)"/><path d="M{cx} 380L{cx+70} 605" stroke="#cfd4da" stroke-width="6"/>')
    o.append(f'<rect x="{cx-4}" y="300" width="8" height="84" fill="url(#gold)"/><rect x="{cx-26}" y="324" width="52" height="8" fill="url(#gold)"/>')
    for k in range(3):
        o.append(f'<rect x="{cx-200-k*24}" y="{1310+k*22}" width="{400+k*48}" height="22" fill="{"#fafbfa" if k % 2 == 0 else "#e2e5e1"}"/>')
    o.append(f'<path d="M490 1376H590L650 1920H430Z" fill="#f4f1f2"/>')
    for sx in (-1, 1):
        o.append(f'<path d="M{540+sx*70} 1376L{540+sx*130} 1920H{540+sx*440}L{540+sx*320} 1376Z" fill="url(#hijau)" opacity=".75"/>')
    svg('kapel-utama', ''.join(o))

    # lengkung bunga pernikahan (dibolongi) + bunga bergerombol
    o = []
    bukaan = 'M300 1620V1000A240 240 0 0 1 780 1000V1620Z'
    o.append(f'<path fill-rule="evenodd" d="M220 1620V1000A320 320 0 0 1 860 1000V1620Z{bukaan}" fill="url(#wv)"/>')
    import random
    random.seed(7)
    warna = ['#ffffff', '#f8e1e7', '#f3c6d3', '#fdf3e6', '#e9b8c6']
    for t in range(140):
        a = math.pi * (t / 139)
        r0 = 280 + random.uniform(-36, 36)
        x = 540 - r0 * math.cos(a); y = 1000 - r0 * math.sin(a)
        if t % 3 == 0:
            o.append(f'<ellipse cx="{f(x+random.uniform(-10,10))}" cy="{f(y+random.uniform(-10,10))}" rx="22" ry="11" fill="#8fae86" transform="rotate({random.randint(0,180)} {f(x)} {f(y)})"/>')
        o.append(f'<circle cx="{f(x)}" cy="{f(y)}" r="{random.randint(14,26)}" fill="{random.choice(warna)}" stroke="#e7cfd6" stroke-width="2"/>')
    for sx in (-1, 1):
        for k in range(10):
            x = 540 + sx * (280 + random.uniform(-30, 30)); y = 1060 + k * 52
            o.append(f'<circle cx="{f(x)}" cy="{f(y)}" r="{random.randint(12,22)}" fill="{random.choice(warna)}" stroke="#e7cfd6" stroke-width="2"/>')
    for k in range(3):
        o.append(f'<rect x="{160-k*40}" y="{1620+k*22}" width="{760+k*80}" height="22" fill="{"#faf9f9" if k % 2 == 0 else "#e6e2e3"}"/>')
    o.append(f'<path d="M430 1686H650L760 1920H320Z" fill="#f1e4e8"/>')
    o.append(f'<rect x="0" y="1686" width="{W}" height="{H-1686}" fill="#efeeee"/>')
    o.append(f'<path d="M430 1686H650L760 1920H320Z" fill="#f3dfe5"/>')
    svg('kapel-depan', ''.join(o))

# ══ LAPISAN TAMBAHAN (dirakit bebas di web/panggung.rs) ═════════════════
KAYU = '#8a6a4e'
def lantai(cx, y, lebar, n=3, terang='#fbfaf6', gelap='#e3e0d7'):
    return ''.join(f'<rect x="{f(cx-lebar/2-k*22)}" y="{y+k*22}" width="{f(lebar+k*44)}" height="22" fill="{terang if k % 2 == 0 else gelap}"/>' for k in range(n))

def tanah(y, warna='#eeede8'):
    return f'<rect x="0" y="{y}" width="{W}" height="{H-y}" fill="{warna}"/>'

def jalan(y, warna='#f5f3ed', rumput=True):
    o = f'<path d="M490 {y}H590L660 1920H420Z" fill="{warna}"/>'
    if rumput:
        for sx in (-1, 1):
            o += f'<path d="M{540+sx*70} {y}L{540+sx*130} 1920H{540+sx*440}L{540+sx*320} {y}Z" fill="url(#hijau)" opacity=".78"/>'
    return o

def tiang_panggung(cx, y_lantai, lebar, tinggi=120, n=7):
    return ''.join(f'<rect x="{f(cx-lebar/2+10+k*(lebar-20)/(n-1)-7)}" y="{y_lantai}" width="14" height="{tinggi}" fill="#cbb79c"/>' for k in range(n))

def dinding_jendela(x, y, w, h, n=4, fill='url(#wv)'):
    o = f'<rect x="{f(x)}" y="{f(y)}" width="{f(w)}" height="{f(h)}" fill="{fill}"/>'
    for k in range(n):
        cxk = x + (k + .5) * w / n
        o += f'<rect x="{f(cxk-18)}" y="{f(y+h*.25)}" width="36" height="{f(h*.5)}" fill="url(#r)"/><path d="M{f(cxk)} {f(y+h*.25)}V{f(y+h*.75)}" stroke="#eef0f2" stroke-width="3"/>'
    return o

def gadang():
    cx, o = 540, [tanah(1380)]
    o.append(tiang_panggung(cx, 1210, 760, 130, 9))
    o.append(dinding_jendela(cx-380, 1050, 760, 160, 6))
    o.append(f'<rect x="{cx-390}" y="1200" width="780" height="16" fill="url(#gold)"/>')
    # atap gonjong: 4 tanduk melengkung
    tanduk = [(-330, 640), (-150, 560), (150, 560), (330, 640)]
    o.append(f'<path d="M{cx-420} 1060Q{cx-380} 900 {cx-330} 640Q{cx-280} 860 {cx-150} 560Q{cx-60} 880 {cx} 900Q{cx+60} 880 {cx+150} 560Q{cx+280} 860 {cx+330} 640Q{cx+380} 900 {cx+420} 1060Z" fill="#5c4a3c"/>')
    o.append(f'<path d="M{cx-420} 1060Q{cx-380} 900 {cx-330} 640Q{cx-280} 860 {cx-150} 560Q{cx-60} 880 {cx} 900Q{cx+60} 880 {cx+150} 560Q{cx+280} 860 {cx+330} 640Q{cx+380} 900 {cx+420} 1060" stroke="url(#gold)" stroke-width="6" fill="none"/>')
    for x, y in tanduk:
        o.append(f'<circle cx="{cx+x}" cy="{y-6}" r="9" fill="url(#gold)"/>')
    o.append(f'<path d="M{cx-370} 1060H{cx+370}" stroke="#c9a24a" stroke-width="10"/>')
    o.append(lantai(cx, 1340, 300) + jalan(1406))
    svg('gadang-utama', ''.join(o))

def tongkonan():
    cx, o = 540, [tanah(1380)]
    o.append(tiang_panggung(cx, 1150, 420, 190, 6))
    o.append(f'<rect x="{cx-230}" y="1000" width="460" height="160" fill="#7a3b2a"/>')
    for k in range(6):
        o.append(f'<path d="M{cx-220+k*76} 1010l38 40 38-40M{cx-220+k*76} 1150l38-40 38 40" stroke="#e8d3a8" stroke-width="4" fill="none"/>')
    o.append(f'<circle cx="{cx}" cy="1080" r="34" fill="#2d1a12" stroke="#e8d3a8" stroke-width="5"/>')
    # atap perahu melengkung tajam
    o.append(f'<path d="M{cx-470} 820Q{cx-300} 990 {cx} 980Q{cx+300} 990 {cx+470} 820L{cx+420} 1010Q{cx} 1060 {cx-420} 1010Z" fill="#4e3b2e"/>')
    o.append(f'<path d="M{cx-470} 820Q{cx-300} 990 {cx} 980Q{cx+300} 990 {cx+470} 820" stroke="url(#gold)" stroke-width="7" fill="none"/>')
    o.append(f'<path d="M{cx} 980V1000" stroke="#e8d3a8" stroke-width="10"/>')
    o.append(f'<rect x="{cx-20}" y="1150" width="40" height="190" fill="#5a2f22"/>')
    o.append(lantai(cx, 1340, 300) + jalan(1406))
    svg('tongkonan-utama', ''.join(o))

def rumah_panggung():
    cx, o = 540, [tanah(1380)]
    o.append(tiang_panggung(cx, 1200, 640, 140, 8))
    o.append(dinding_jendela(cx-320, 1050, 640, 150, 5))
    o.append(f'<rect x="{cx-340}" y="1192" width="680" height="14" fill="url(#gold)"/>')
    # atap limas bersusun Melayu dengan selembayung
    o.append(f'<path d="M{cx-380} 1060L{cx-240} 900H{cx+240}L{cx+380} 1060Z" fill="#6b5442"/>')
    o.append(f'<path d="M{cx-260} 902L{cx-120} 760H{cx+120}L{cx+260} 902Z" fill="#5e4a3a"/>')
    for x in (cx - 120, cx + 120):
        sx = -1 if x < cx else 1
        o.append(f'<path d="M{x} 760q{sx*30} -50 {sx*10} -110" stroke="url(#gold)" stroke-width="10" fill="none" stroke-linecap="round"/>')
    o.append(f'<path d="M{cx-380} 1060H{cx+380}" stroke="#c9a24a" stroke-width="8"/>')
    # tangga kayu
    for k in range(6):
        o.append(f'<rect x="{cx-60}" y="{1210+k*24}" width="120" height="8" fill="#b89a74"/>')
    o.append(lantai(cx, 1350, 260) + jalan(1416))
    svg('panggung-utama', ''.join(o))

def lamin():
    cx, o = 540, [tanah(1380)]
    o.append(tiang_panggung(cx, 1190, 900, 150, 12))
    o.append(f'<rect x="{cx-450}" y="1040" width="900" height="160" fill="#f2ead8"/>')
    for k in range(9):
        x = cx - 430 + k * 100
        o.append(f'<path d="M{x} 1070c30 -20 60 20 90 0M{x} 1120c30 20 60 -20 90 0M{x+45} 1060v110" stroke="#2f4d6b" stroke-width="5" fill="none"/>')
        o.append(f'<circle cx="{x+45}" cy="1095" r="9" fill="#c62828"/>')
    o.append(f'<path d="M{cx-500} 1050L{cx-400} 900H{cx+400}L{cx+500} 1050Z" fill="#5d4a3b"/>')
    for x in (cx - 400, cx + 400):
        o.append(f'<path d="M{x} 900q{(x-cx)/8} -60 {(x-cx)/16} -110" stroke="url(#gold)" stroke-width="10" fill="none" stroke-linecap="round"/>')
    o.append(lantai(cx, 1340, 360) + jalan(1406))
    svg('lamin-utama', ''.join(o))

def honai():
    o = [tanah(1380, '#ece9e2')]
    for cx, sk in ((300, .75), (780, .75), (540, 1.0)):
        r = 190 * sk
        o.append(f'<rect x="{f(cx-r)}" y="{f(1330-170*sk)}" width="{f(2*r)}" height="{f(170*sk)}" fill="#a07a58"/>')
        for k in range(12):
            o.append(f'<path d="M{f(cx-r+k*2*r/12)} {f(1330-170*sk)}v{f(170*sk)}" stroke="#8a6646" stroke-width="3"/>')
        o.append(f'<path d="M{f(cx-r-30*sk)} {f(1335-170*sk)}Q{f(cx)} {f(1330-520*sk)} {f(cx+r+30*sk)} {f(1335-170*sk)}Z" fill="#c9b48a"/>')
        for k in range(8):
            o.append(f'<path d="M{f(cx-r+k*r/4)} {f(1340-170*sk)}Q{f(cx)} {f(1330-480*sk)} {f(cx)} {f(1330-470*sk)}" stroke="#b39c70" stroke-width="3" fill="none"/>')
        o.append(f'<rect x="{f(cx-26*sk)}" y="{f(1330-100*sk)}" width="{f(52*sk)}" height="{f(100*sk)}" fill="#4a3424"/>')
    o.append(jalan(1386, '#efe9dd'))
    svg('honai-utama', ''.join(o))

def meru():
    cx, o = 540, [tanah(1380, '#efeee8')]
    o.append(f'<rect x="{cx-170}" y="1150" width="340" height="190" fill="#e8dccb"/>')
    o.append(f'<rect x="{cx-190}" y="1140" width="380" height="20" fill="url(#gold)"/>')
    o.append(f'<path d="{lengkung_runcing(cx, 1340, 90, 90, 40)}" fill="#7d6450"/>')
    for i in range(7):
        y = 1140 - i * 72
        lb = 520 - i * 58
        o.append(f'<path d="M{f(cx-lb/2)} {y}L{f(cx-lb/2+40)} {y-50}H{f(cx+lb/2-40)}L{f(cx+lb/2)} {y}Z" fill="#3f342c"/>')
        o.append(f'<rect x="{f(cx-lb/2+46)}" y="{y-72}" width="{f(lb-92)}" height="22" fill="#d8ccb6"/>')
    o.append(f'<path d="M{cx-20} 640Q{cx} 560 {cx+20} 640Z" fill="url(#gold)"/>')
    o.append(lantai(cx, 1340, 420) + jalan(1406))
    svg('meru-utama', ''.join(o))

def kuil():
    cx, o = 540, [tanah(1380, '#efeeec')]
    o.append(f'<rect x="{cx-320}" y="1080" width="640" height="230" fill="url(#wv)"/>')
    for x in range(cx - 300, cx + 301, 100):
        o.append(f'<rect x="{x-10}" y="1080" width="20" height="230" fill="#b23a2e"/>')
    for x in range(cx - 250, cx + 251, 100):
        o.append(f'<rect x="{x-38}" y="1120" width="76" height="170" fill="#f3ece0" stroke="#7d5a40" stroke-width="3"/><path d="M{x} 1120v170M{x-38} 1205h76" stroke="#c9b49a" stroke-width="2"/>')
    o.append(atap_cina(cx, 1085, 780, 170, '#3d3a3a'))
    o.append(f'<rect x="{cx-220}" y="930" width="440" height="80" fill="url(#wv)"/>')
    o.append(atap_cina(cx, 935, 560, 140, '#3d3a3a'))
    o.append(lantai(cx, 1310, 680) + jalan(1376, '#f4f2ee', False))
    svg('kuil-utama', ''.join(o))

def wat():
    cx, o = 540, [tanah(1380, '#f1eee6')]
    o.append(f'<rect x="{cx-300}" y="1080" width="600" height="240" fill="url(#wv)"/>')
    for x in range(cx - 260, cx + 261, 104):
        o.append(f'<path d="{lengkung_runcing(x, 1300, 50, 120, 60)}" fill="#c9a24a"/>')
    for i, (lb, y, h) in enumerate([(760, 1090, 150), (600, 960, 150), (440, 830, 150)]):
        o.append(f'<path d="M{f(cx-lb/2)} {y}L{cx} {y-h}L{f(cx+lb/2)} {y}Z" fill="{["#9a2e2a", "#7a2a2a", "#9a2e2a"][i]}"/>')
        o.append(f'<path d="M{f(cx-lb/2)} {y}L{cx} {y-h}L{f(cx+lb/2)} {y}" stroke="url(#gold)" stroke-width="8" fill="none"/>')
        for sx in (-1, 1):
            o.append(f'<path d="M{f(cx+sx*lb/2)} {y}q{sx*30} -20 {sx*16} -70" stroke="url(#gold)" stroke-width="8" fill="none" stroke-linecap="round"/>')
    o.append(f'<path d="M{cx-6} 680L{cx} 560L{cx+6} 680Z" fill="url(#gold)"/>')
    o.append(lantai(cx, 1320, 640) + jalan(1386, '#f4f1ea'))
    svg('wat-utama', ''.join(o))

def stupa():
    cx, o = 540, [tanah(1380, '#efede6')]
    for k, (w, h) in enumerate([(820, 70), (700, 70), (580, 70)]):
        y = 1340 - (k + 1) * 70
        o.append(f'<rect x="{cx-w/2}" y="{y}" width="{w}" height="{h}" fill="{"url(#w)" if k % 2 == 0 else "#e3dfd5"}"/>')
        for j in range(int(w / 70)):
            o.append(f'<path d="M{cx-w/2+20+j*70} {y+60}q15 -40 30 0" fill="#cfc8b8"/>')
    o.append(f'<path d="M{cx-260} 1130C{cx-260} 920 {cx+260} 920 {cx+260} 1130Z" fill="url(#dome)"/>')
    o.append(f'<rect x="{cx-50}" y="900" width="100" height="40" fill="#e6e1d6"/>')
    for k in range(7):
        o.append(f'<rect x="{cx-40+k*5}" y="{900-(k+1)*34}" width="{80-k*10}" height="26" rx="8" fill="url(#gold)"/>')
    o.append(lantai(cx, 1340, 860, 2) + jalan(1384, '#f3f1ea'))
    svg('stupa-utama', ''.join(o))

def paviliun():
    cx, o = 540, [tanah(1380, '#eef0ec')]
    # gazebo putih: kubah, 6 tiang, lengkung di antaranya
    o.append(f'<path d="M{cx-300} 1300H{cx+300}V1330H{cx-300}Z" fill="#e5e7e3"/>')
    for k in range(6):
        x = cx - 270 + k * 108
        o.append(f'<rect x="{x-12}" y="980" width="24" height="320" fill="url(#w)"/><rect x="{x-18}" y="972" width="36" height="14" fill="url(#gold)"/>')
    for k in range(5):
        x = cx - 216 + k * 108
        o.append(f'<path d="M{x-54} 1010Q{x} 950 {x+54} 1010" stroke="#f6f7f5" stroke-width="16" fill="none"/>')
    o.append(f'<rect x="{cx-320}" y="950" width="640" height="34" fill="url(#w)"/>')
    o.append(kubah(cx, 950, 520, 260))
    # tirai & bunga
    for sx in (-1, 1):
        o.append(f'<path d="M{cx+sx*270} 990Q{cx+sx*230} 1150 {cx+sx*262} 1300" stroke="#f8eef1" stroke-width="26" fill="none" opacity=".9"/>')
    for k in range(18):
        x = cx - 300 + k * 35
        o.append(f'<circle cx="{x}" cy="{1296 + (k % 3) * 6}" r="{14 + (k % 4) * 3}" fill="{["#ffffff", "#f6dde4", "#eec3cf"][k % 3]}" stroke="#e6cfd6" stroke-width="2"/>')
    o.append(lantai(cx, 1330, 620) + jalan(1396, '#f5f2f3'))
    svg('paviliun-utama', ''.join(o))

def torii():
    o = []
    merah = '#c0392b'
    for x in (300, 780):
        o.append(f'<rect x="{x-34}" y="640" width="68" height="980" fill="{merah}"/><rect x="{x-50}" y="1580" width="100" height="40" fill="#2e2a28"/>')
    o.append(f'<path d="M110 600Q540 520 970 600L990 560Q540 470 90 560Z" fill="#2e2a28"/>')
    o.append(f'<rect x="150" y="610" width="780" height="50" fill="{merah}"/>')
    o.append(f'<rect x="210" y="760" width="660" height="40" fill="{merah}"/><rect x="500" y="660" width="80" height="100" fill="{merah}"/>')
    o.append(''.join(f'<rect x="{60-k*30}" y="{1620+k*22}" width="{960+k*60}" height="22" fill="{"#f6f4f0" if k % 2 == 0 else "#dfdbd4"}"/>' for k in range(3)))
    o.append(tanah(1686, '#efedea'))
    svg('torii-depan', ''.join(o))

def gapura():
    # gapura kayu beratap (Nusantara/Korea): dua tiang ukir + atap bertumpuk
    o = []
    for x in (250, 830):
        o.append(f'<rect x="{x-70}" y="760" width="140" height="860" fill="#f3ead9"/><rect x="{x-70}" y="760" width="140" height="860" fill="none" stroke="{KAYU}" stroke-width="10"/>')
        for k in range(6):
            y = 820 + k * 130
            o.append(f'<path d="M{x-40} {y}q40 50 80 0q-40 50 -80 100" stroke="#b08a5a" stroke-width="5" fill="none"/>')
        o.append(f'<rect x="{x-90}" y="1580" width="180" height="40" fill="#d9cdb6"/>')
    o.append(f'<rect x="170" y="700" width="740" height="70" fill="#f3ead9" stroke="{KAYU}" stroke-width="8"/>')
    o.append(f'<path d="M60 710L250 520H830L1020 710Z" fill="#5c4836"/><path d="M60 710H1020" stroke="url(#gold)" stroke-width="10"/>')
    o.append(f'<path d="M260 522L380 400H700L820 522Z" fill="#4e3d2e"/><path d="M500 400Q540 330 580 400Z" fill="url(#gold)"/>')
    o.append(''.join(f'<rect x="{60-k*30}" y="{1620+k*22}" width="{960+k*60}" height="22" fill="{"#f7f5ef" if k % 2 == 0 else "#e0dcd2"}"/>' for k in range(3)))
    o.append(tanah(1686, '#edebe5'))
    svg('gapura-depan', ''.join(o))

def pohon():
    # lengkung pepohonan: dua batang melengkung bertemu, dedaunan & bunga
    import random
    random.seed(11)
    o = []
    for sx in (-1, 1):
        o.append(f'<path d="M{540+sx*330} 1640C{540+sx*360} 1300 {540+sx*380} 900 {540+sx*60} 620" stroke="#7a5c44" stroke-width="56" fill="none" stroke-linecap="round"/>')
        o.append(f'<path d="M{540+sx*330} 1640C{540+sx*360} 1300 {540+sx*380} 900 {540+sx*60} 620" stroke="#9a7a5c" stroke-width="18" fill="none" stroke-linecap="round" opacity=".6"/>')
    for t in range(170):
        a = math.pi * (t / 169)
        r0 = 400 + random.uniform(-70, 70)
        x = 540 - r0 * math.cos(a) * .95; y = 980 - r0 * math.sin(a) * .95
        g = random.choice(['#7d9b6a', '#6a8a58', '#93b07e', '#a9c293'])
        o.append(f'<ellipse cx="{f(x)}" cy="{f(y)}" rx="{random.randint(26,46)}" ry="{random.randint(16,28)}" fill="{g}" transform="rotate({random.randint(0,180)} {f(x)} {f(y)})"/>')
        if t % 4 == 0:
            o.append(f'<circle cx="{f(x+random.uniform(-14,14))}" cy="{f(y+random.uniform(-14,14))}" r="9" fill="{random.choice(["#fff", "#f6d6df", "#fdf0d5"])}"/>')
    o.append(''.join(f'<rect x="{80-k*40}" y="{1640+k*22}" width="{920+k*80}" height="22" fill="{"#f1efe8" if k % 2 == 0 else "#dcd8cc"}"/>' for k in range(2)))
    o.append(tanah(1684, '#e8eadf'))
    svg('pohon-depan', ''.join(o))

def laut():
    o = ['<rect x="0" y="1180" width="1080" height="200" fill="#d6e2e8"/>']
    for k in range(10):
        o.append(f'<path d="M{k*120-40} {1200+k%3*40}q30 -10 60 0" stroke="#eef4f6" stroke-width="5" fill="none"/>')
    o.append('<path d="M-20 1200Q120 1080 260 1196Z" fill="#d1d9d6"/><path d="M780 1200Q900 1100 1100 1198Z" fill="#cfd8d4"/>')
    o.append('<path d="M640 1190l40-90v90ZM686 1190l22-60v60Z" fill="#e9eef0"/><path d="M620 1192h110l-14 14h-82Z" fill="#c9d2d6"/>')
    o.append(tanah(1360, '#ece9e1'))
    svg('laut-jauh', ''.join(o))

def hutan():
    o = []
    for lap, (y0, c) in enumerate([(1120, '#dfe5dc'), (1190, '#d3dccf'), (1260, '#c8d3c3')]):
        for k in range(14):
            x = k * 84 - 20 + lap * 30
            h = 180 + (k * 37 % 90)
            o.append(f'<path d="M{x} {y0+60}L{x+42} {y0-h+60}L{x+84} {y0+60}Z" fill="{c}"/>')
    o.append(tanah(1320, '#e6eae2'))
    svg('hutan-jauh', ''.join(o))

def fuji():
    o = ['<path d="M-60 1330L440 820Q540 760 640 820L1140 1330Z" fill="#dfe2e8"/>',
         '<path d="M440 820Q540 760 640 820L600 860L570 840L540 870L510 842L480 864Z" fill="#ffffff"/>']
    for x in (120, 960):
        o.append(f'<path d="M{x} 1330V1150" stroke="#d2cdd0" stroke-width="10"/><ellipse cx="{x}" cy="1120" rx="90" ry="44" fill="#f2dde3"/><ellipse cx="{x-40}" cy="1150" rx="60" ry="30" fill="#f6e6ea"/>')
    o.append(tanah(1320, '#ecebe9'))
    svg('fuji-jauh', ''.join(o))

if __name__ == '__main__':
    os.makedirs(OUT, exist_ok=True)
    masjid(); jawa(); tionghoa(); kapel()
    gadang(); tongkonan(); rumah_panggung(); lamin(); honai(); meru(); kuil(); wat(); stupa(); paviliun()
    torii(); gapura(); pohon(); laut(); hutan(); fuji()
    print('ok')
