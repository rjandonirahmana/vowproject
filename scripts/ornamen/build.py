# Bangkitkan contoh ornamen bunga (SVG) + migration/013_gerak_everlove.sql.
#   python3 scripts/ornamen/build.py .        (dari akar proyek)
#
# Ornamen = gambar transparan yang ditempel ke bagian undangan lewat tabel
# theme_ornaments (diatur di /admin/tema/{slug}/ornamen). Skrip ini hanya
# membuat CONTOH awal untuk beberapa tema; selanjutnya admin bebas mengganti
# gambar (unggah PNG/WebP) dan mengatur posisi/animasinya.
#
# Bentuk (semua digambar mengarah ke KIRI-ATAS; sudut lain = cermin/putar):
#   sudut-*   rangkaian bunga di sudut (600×600) — bunga besar dekat pojok,
#             ranting daun menjalar di sepanjang tepi atas & kiri
#   untaian-* untaian bunga melengkung (800×260) — untuk tengah-atas
#   ranting-* setangkai ranting berdaun & kuncup (300×600) — untuk sisi
#   bunga-*   satu bunga mekar (300×300) — aksen di tengah/sudut kecil
import math, os, random, sys

ROOT = sys.argv[1] if len(sys.argv) > 1 else "."
OUT = os.path.join(ROOT, "public/img/tema/ornamen")
URL = "/img/tema/ornamen"
os.makedirs(OUT, exist_ok=True)

# ── Palet: (kelopak gelap, kelopak terang, daun gelap, daun terang, aksen) ──
PAL = {
    "blush":  dict(p0="#d9868d", p1="#fbe1dc", l0="#4f6e4c", l1="#a7bf98", a="#c5a059", kind="mawar"),
    "putih":  dict(p0="#d8c9b0", p1="#fffdf8", l0="#56704f", l1="#b2c4a2", a="#c5a059", kind="mawar"),
    "merah":  dict(p0="#8e1b22", p1="#e4797a", l0="#34502f", l1="#86a271", a="#c9982f", kind="mawar"),
    "kamboja": dict(p0="#f0c35c", p1="#ffffff", l0="#2f6655", l1="#8dbfa6", a="#c7a262", kind="kamboja"),
    "emas":   dict(p0="#a77a2c", p1="#f6dd9c", l0="#8a6424", l1="#e3c27a", a="#f3d27c", kind="mawar"),
    "terra":  dict(p0="#b4693f", p1="#f0c9a4", l0="#77744a", l1="#c2bd8c", a="#b8894a", kind="mawar"),
    "melati": dict(p0="#d6cfbf", p1="#ffffff", l0="#3f5f3f", l1="#9fbf8f", a="#c9a26b", kind="melati"),
    "biru":   dict(p0="#7d98c4", p1="#eef3fb", l0="#4c6a5a", l1="#a9c2b0", a="#c9a35a", kind="mawar"),
}

def f(v):
    return f"{v:.1f}".rstrip("0").rstrip(".")

class Svg:
    def __init__(self, w, h, pal, key):
        self.w, self.h, self.p, self.key = w, h, pal, key
        self.defs, self.back, self.mid, self.front = [], [], [], []
        P = pal
        self.defs.append(f'<radialGradient id="pt-{key}" cx="50%" cy="85%" r="90%"><stop offset="0" stop-color="{P["p0"]}"/><stop offset="1" stop-color="{P["p1"]}"/></radialGradient>')
        self.defs.append(f'<radialGradient id="pc-{key}" cx="50%" cy="50%" r="60%"><stop offset="0" stop-color="{mix(P["p0"], "#000000", .25)}"/><stop offset=".6" stop-color="{P["p0"]}"/><stop offset="1" stop-color="{P["p1"]}"/></radialGradient>')
        self.defs.append(f'<linearGradient id="lf-{key}" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="{P["l1"]}"/><stop offset="1" stop-color="{P["l0"]}"/></linearGradient>')
        self.defs.append(f'<radialGradient id="km-{key}" cx="15%" cy="50%" r="85%"><stop offset="0" stop-color="{P["p0"]}"/><stop offset=".45" stop-color="{mix(P["p0"], P["p1"], .6)}"/><stop offset="1" stop-color="{P["p1"]}"/></radialGradient>')
        self.defs.append(f'<filter id="sh-{key}" x="-20%" y="-20%" width="140%" height="140%"><feDropShadow dx="0" dy="2" stdDeviation="2.2" flood-color="#000" flood-opacity=".18"/></filter>')

    # ── Unsur ──
    def stem(self, d, width=3.2):
        self.back.append(f'<path d="{d}" fill="none" stroke="{self.p["l0"]}" stroke-width="{f(width)}" stroke-linecap="round"/>')

    def leaf(self, x, y, length, ang, width=None, layer="mid"):
        w = width or length * 0.36
        L = length
        d = f"M0 0C{f(L*.25)} {f(-w)} {f(L*.7)} {f(-w*.9)} {f(L)} 0C{f(L*.7)} {f(w*.9)} {f(L*.25)} {f(w)} 0 0Z"
        g = (f'<g transform="translate({f(x)} {f(y)}) rotate({f(ang)})"><path d="{d}" fill="url(#lf-{self.key})"/>'
             f'<path d="M{f(L*.06)} 0Q{f(L*.5)} {f(-w*.12)} {f(L*.93)} 0" fill="none" stroke="{self.p["l0"]}" stroke-width="{f(max(.8, L/60))}" opacity=".55"/></g>')
        getattr(self, layer).append(g)

    def leaves_along(self, pts, every, size, side_ang=48, jitter=0, rng=None, shrink=True):
        """Daun berpasangan selang-seling sepanjang polyline `pts`."""
        rng = rng or random.Random(1)
        n = len(pts)
        for i in range(1, n, every):
            (x0, y0), (x1, y1) = pts[i - 1], pts[i]
            base = math.degrees(math.atan2(y1 - y0, x1 - x0))
            t = i / n
            s = size * ((1 - .55 * t) if shrink else 1) * (1 + rng.uniform(-jitter, jitter))
            side = 1 if (i // every) % 2 else -1
            self.leaf(x1, y1, s, base + side * (side_ang + rng.uniform(-8, 8)))

    def berry(self, x, y, r):
        self.front.append(f'<circle cx="{f(x)}" cy="{f(y)}" r="{f(r)}" fill="{self.p["a"]}"/><circle cx="{f(x - r*.3)}" cy="{f(y - r*.3)}" r="{f(r*.35)}" fill="#fff" opacity=".55"/>')

    def bud(self, x, y, r, ang):
        P = self.p
        self.front.append(
            f'<g transform="translate({f(x)} {f(y)}) rotate({f(ang)})">'
            f'<path d="M0 {f(r*1.4)}C{f(-r)} {f(r*.6)} {f(-r*.8)} {f(-r*.7)} 0 {f(-r*1.2)}C{f(r*.8)} {f(-r*.7)} {f(r)} {f(r*.6)} 0 {f(r*1.4)}Z" fill="url(#pt-{self.key})" stroke="{mix(P["p0"], "#000000", .2)}" stroke-width=".8"/>'
            f'<path d="M0 {f(r*1.5)}C{f(-r*.9)} {f(r*1.1)} {f(-r*.6)} {f(r*.2)} {f(-r*.2)} 0M0 {f(r*1.5)}C{f(r*.9)} {f(r*1.1)} {f(r*.6)} {f(r*.2)} {f(r*.2)} 0" fill="none" stroke="{P["l0"]}" stroke-width="{f(r*.28)}" stroke-linecap="round"/></g>')

    def rose(self, x, y, r, rot=0):
        """Mawar dilihat dari atas: 3 lingkar kelopak + pusat spiral."""
        P, k = self.p, self.key
        edge = mix(P["p0"], "#000000", .18)
        out = [f'<g transform="translate({f(x)} {f(y)}) rotate({f(rot)})" filter="url(#sh-{k})">']
        for ring, (n, rr, sc) in enumerate([(6, r, 1.0), (5, r * .72, .8), (4, r * .46, .62)]):
            off = ring * 23
            for j in range(n):
                a = off + j * 360 / n
                pw, ph = rr * .78 * sc / sc, rr
                d = (f"M0 0C{f(-pw)} {f(-ph*.25)} {f(-pw*.9)} {f(-ph*1.02)} 0 {f(-ph)}"
                     f"C{f(pw*.9)} {f(-ph*1.02)} {f(pw)} {f(-ph*.25)} 0 0Z")
                out.append(f'<path d="{d}" transform="rotate({f(a)})" fill="url(#pt-{k})" stroke="{edge}" stroke-width=".9" stroke-opacity=".5"/>')
        # pusat: spiral kelopak tergulung
        out.append(f'<circle r="{f(r*.3)}" fill="url(#pc-{k})"/>')
        sp = " ".join(f"{'M' if i == 0 else 'L'}{f(math.cos(t)*r*.03*t)} {f(math.sin(t)*r*.03*t)}" for i, t in enumerate([i * .35 for i in range(0, 29)]))
        out.append(f'<path d="{sp}" fill="none" stroke="{edge}" stroke-width="{f(max(1, r/28))}" stroke-linecap="round" opacity=".7"/>')
        out.append("</g>")
        self.front.append("".join(out))

    def kamboja(self, x, y, r, rot=0):
        """Kamboja/frangipani: 5 kelopak berputar, pusat kuning."""
        k = self.key
        out = [f'<g transform="translate({f(x)} {f(y)}) rotate({f(rot)})" filter="url(#sh-{k})">']
        for j in range(5):
            a = j * 72
            d = f"M0 0C{f(r*.15)} {f(-r*.55)} {f(r*.75)} {f(-r*.85)} {f(r*.98)} {f(-r*.42)}C{f(r*1.12)} {f(-r*.1)} {f(r*.55)} {f(r*.18)} 0 0Z"
            out.append(f'<path d="{d}" transform="rotate({f(a)})" fill="url(#km-{k})" stroke="{mix(self.p["p0"], "#000000", .25)}" stroke-width=".7" stroke-opacity=".35"/>')
        out.append(f'<circle r="{f(r*.12)}" fill="{mix(self.p["p0"], "#000000", .1)}"/></g>')
        self.front.append("".join(out))

    def melati(self, x, y, r, rot=0):
        """Melati: 8 kelopak runcing putih."""
        k = self.key
        stroke = mix(self.p["p0"], "#000000", .2)
        out = [f'<g transform="translate({f(x)} {f(y)}) rotate({f(rot)})" filter="url(#sh-{k})">']
        for j in range(8):
            d = f"M0 0C{f(-r*.28)} {f(-r*.45)} {f(-r*.18)} {f(-r*.9)} 0 {f(-r)}C{f(r*.18)} {f(-r*.9)} {f(r*.28)} {f(-r*.45)} 0 0Z"
            out.append(f'<path d="{d}" transform="rotate({j*45})" fill="url(#pt-{k})" stroke="{stroke}" stroke-width=".7" stroke-opacity=".6"/>')
        out.append(f'<circle r="{f(r*.14)}" fill="{self.p["a"]}"/></g>')
        self.front.append("".join(out))

    def flower(self, x, y, r, rot=0):
        {"mawar": self.rose, "kamboja": self.kamboja, "melati": self.melati}[self.p["kind"]](x, y, r, rot)

    def render(self):
        return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {self.w} {self.h}" width="{self.w}" height="{self.h}">'
                f'<defs>{"".join(self.defs)}</defs>{"".join(self.back)}{"".join(self.mid)}{"".join(self.front)}</svg>')


def mix(a, b, t):
    A = [int(a.lstrip("#")[i:i + 2], 16) for i in (0, 2, 4)]
    B = [int(b.lstrip("#")[i:i + 2], 16) for i in (0, 2, 4)]
    return "#" + "".join(f"{round(x * (1 - t) + y * t):02x}" for x, y in zip(A, B))

def bez(p0, p1, p2, p3, n=40):
    pts = []
    for i in range(n + 1):
        t = i / n
        u = 1 - t
        pts.append((u**3*p0[0] + 3*u*u*t*p1[0] + 3*u*t*t*p2[0] + t**3*p3[0],
                    u**3*p0[1] + 3*u*u*t*p1[1] + 3*u*t*t*p2[1] + t**3*p3[1]))
    return pts

def path_of(p0, p1, p2, p3):
    return f"M{f(p0[0])} {f(p0[1])}C{f(p1[0])} {f(p1[1])} {f(p2[0])} {f(p2[1])} {f(p3[0])} {f(p3[1])}"

# ── Komposisi ──────────────────────────────────────────────────────────────

def sudut(name, P):
    s = Svg(600, 600, P, name)
    rng = random.Random(7)
    small = P["kind"] != "mawar"
    # Dua sulur: menyusur tepi atas & tepi kiri, berpangkal di pojok.
    for b in [((40, 46), (200, 6), (380, 70), (585, 34)), ((46, 40), (6, 200), (70, 380), (34, 585)),
              ((60, 60), (190, 130), (260, 210), (330, 250))]:
        s.stem(path_of(*b), 3.4 if b[3][0] > 400 or b[3][1] > 400 else 2.6)
        pts = bez(*b, n=34)
        s.leaves_along(pts, 2, 92 if b[3][0] != 330 else 60, rng=rng, jitter=.18)
    # daun lebar di balik gugus bunga — memberi kesan rimbun
    for a in (-8, 18, 40, 62, 88, 112):
        s.leaf(110, 110, rng.uniform(150, 190), a + rng.uniform(-6, 6), layer="back")
    # ujung sulur: kuncup & buah
    s.bud(570, 30, 14, 80)
    s.bud(30, 570, 14, 170)
    for (x, y) in [(470, 58), (430, 92), (58, 470), (92, 430), (300, 238), (330, 232)]:
        s.berry(x, y, 6.5)
    # gugus bunga dekat pojok
    if small:
        for (x, y, r) in [(128, 120, 84), (258, 70, 60), (70, 258, 60), (220, 196, 48), (360, 44, 38), (44, 360, 38), (300, 150, 30), (150, 300, 30)]:
            s.flower(x, y, r, rng.uniform(0, 72))
    else:
        for (x, y, r) in [(128, 120, 92), (268, 72, 64), (72, 268, 64), (228, 204, 46), (360, 48, 34), (48, 360, 34)]:
            s.flower(x, y, r, rng.uniform(0, 60))
        s.bud(420, 52, 20, 75)
        s.bud(52, 420, 20, 165)
    return s.render()

def untaian(name, P):
    s = Svg(800, 260, P, name)
    rng = random.Random(11)
    # lengkung menggantung dari kiri ke kanan, bunga di tengah
    for b in [((10, 30), (220, 230), (580, 230), (790, 30))]:
        s.stem(path_of(*b), 3)
        s.leaves_along(bez(*b, n=46), 2, 52, rng=rng, jitter=.2, shrink=False, side_ang=55)
    for (x, y) in [(120, 120), (680, 120), (210, 165), (590, 165)]:
        s.berry(x, y, 6)
    s.bud(30, 48, 13, -40)
    s.bud(770, 48, 13, 40)
    if P["kind"] == "mawar":
        for (x, y, r) in [(400, 182, 62), (318, 176, 42), (482, 176, 42), (250, 150, 26), (550, 150, 26)]:
            s.flower(x, y, r, rng.uniform(0, 60))
    else:
        for (x, y, r) in [(400, 178, 56), (325, 172, 42), (475, 172, 42), (255, 152, 30), (545, 152, 30), (190, 120, 22), (610, 120, 22)]:
            s.flower(x, y, r, rng.uniform(0, 72))
    return s.render()

def ranting(name, P):
    s = Svg(300, 600, P, name)
    rng = random.Random(5)
    b = ((40, 20), (150, 180), (60, 380), (230, 585))
    s.stem(path_of(*b), 3)
    s.leaves_along(bez(*b, n=40), 3, 70, rng=rng, jitter=.2, shrink=False)
    for t_x, t_y, r in [(84, 120, 34), (120, 300, 26), (178, 470, 22)]:
        s.flower(t_x, t_y, r, rng.uniform(0, 70))
    s.bud(236, 585, 14, 150)
    s.bud(40, 22, 13, -30)
    return s.render()

def bunga(name, P):
    s = Svg(300, 300, P, name)
    rng = random.Random(3)
    for a in range(0, 360, 60):
        s.leaf(150, 150, 140, a + 30, layer="back")
    s.flower(150, 150, 96, rng.uniform(0, 60))
    return s.render()

SHAPES = {"sudut": sudut, "untaian": untaian, "ranting": ranting, "bunga": bunga}

def write(shape, pal):
    name = f"{shape}-{pal}"
    with open(os.path.join(OUT, f"{name}.svg"), "w") as fh:
        fh.write(SHAPES[shape](name, PAL[pal]))
    return f"{URL}/{name}.svg"

# ── Set gerak ala undangan premium (Everlove/Elementor + Animate.css) ──────
# Ornamen sudut masuk "berayun" (rotateInDownLeft/Right) dengan jeda 500–1400 ms
# lalu bergoyang/berputar pelan; judul & foto "zoom masuk"; teks naik bertahap.
# Kolom: bagian, bentuk, posisi, x, y, lebar, rotasi, cermin, masuk, jeda, durasi, gerak, kecepatan, depan, hp, opasitas
SET = [
    ("sampul", "sudut", "kiri-atas", -14, -12, 46, 0, False, "ayun-kiri", 600, 1300, "goyang", 7, False, True, 100),
    ("sampul", "sudut", "kanan-bawah", 14, 10, 42, 180, False, "ayun-kanan", 1400, 1300, "goyang", 8, False, True, 100),
    ("mempelai", "sudut", "kiri-atas", -18, -16, 34, 0, False, "ayun-kiri", 500, 1300, "goyang", 7, False, True, 90),
    ("mempelai", "ranting", "kanan-tengah", 34, 0, 24, 0, True, "ayun-kanan", 900, 1300, "goyang", 6, False, False, 85),
    ("kisah", "bunga", "kanan-atas", 8, 20, 22, 0, False, "zoom", 1000, 1200, "putar", 20, False, True, 75),
    ("galeri", "untaian", "tengah-atas", 0, -80, 80, 0, False, "turun", 300, 1200, "melayang", 5, False, True, 95),
    ("acara", "sudut", "kanan-atas", 16, -14, 36, 0, True, "ayun-kanan", 500, 1300, "goyang", 7, False, True, 100),
    ("acara", "sudut", "kiri-bawah", -16, 18, 30, 180, True, "ayun-kiri", 1000, 1300, "goyang", 8, False, False, 85),
    ("rsvp", "bunga", "kiri-atas", -14, -22, 24, 0, False, "zoom", 1000, 1200, "putar", 20, False, True, 85),
    ("rsvp", "sudut", "kanan-bawah", 16, 16, 30, 180, False, "ayun-kanan", 500, 1300, "goyang", 8, False, True, 90),
]

for pal in PAL:
    for shape in SHAPES:
        write(shape, pal)

# Palet bunga dipilih dari data tema (berlaku juga untuk tema buatan admin).
PALET_SQL = """CASE
        WHEN t.dark THEN 'emas'
        WHEN t.nuansa = 'Bali' THEN 'kamboja'
        WHEN t.palette = 'blush' THEN 'blush'
        WHEN t.palette = 'navy' THEN 'biru'
        WHEN t.nuansa IN ('Jawa', 'Sunda') THEN 'melati'
        WHEN t.palette = 'terra' AND t.nuansa IN ('Maluku', 'Papua', 'NTT', 'Dayak', 'Toraja', 'Modern') THEN 'terra'
        WHEN t.palette = 'terra' THEN 'merah'
        WHEN t.palette = 'gold' THEN 'melati'
        ELSE 'putih' END"""

def q(s):
    return "'" + str(s).replace("'", "''") + "'"

def b(v):
    return "TRUE" if v else "FALSE"

vals = []
for i, (bag, shape, pos, x, y, w, rot, mir, masuk, jeda, dur, gerak, spd, depan, hp, op) in enumerate(SET):
    vals.append(f"    ({q(bag)}, {q(shape)}, {q(pos)}, {x}, {y}, {w}, {rot}, {b(mir)}, {q(masuk)}, {jeda}, {dur}, {q(gerak)}, {spd}, {b(depan)}, {b(hp)}, {op}, {(i + 1) * 10})")

sql = f"""-- ═══════════════════════════════════════════════════════════════════════════
-- 013_gerak_everlove — SEMUA tema memakai gerak ala undangan premium
-- (referensi: everlove.invisimple.id — Elementor + Animate.css):
--   • isi naik bertahap 1,25 dtk, jeda antar elemen 220 ms (gerak scroll "anggun")
--   • judul & foto "zoom masuk" dari kecil; foto sampul Ken Burns
--   • ornamen bunga di tiap bagian: sudut BERAYUN masuk (rotateInDown) jeda
--     500–1400 ms lalu bergoyang, bunga tunggal berputar pelan, untaian melayang
--
-- WAJIB setelah 012_ornamen.sql. DIBANGKITKAN scripts/ornamen/build.py.
-- Ornamen contoh lama yang BELUM pernah disunting admin (updated_at =
-- created_at) diganti set baru; ornamen hasil suntingan/unggahan admin tetap.
-- Tema yang sudah punya ornamen suntingan tidak ditambahi. Palet bunga dipilih
-- dari data tema (gelap → emas, Bali → kamboja, navy → biru, Jawa/Sunda →
-- melati, terra → merah/terra, blush → blush, lainnya → putih).
-- Setelah ini gerak tiap tema tetap bisa diubah di /admin/tema/… — JANGAN
-- jalankan ulang file ini setelah admin mengubah gerak tema (UPDATE themes
-- di bawah menimpa pilihan gerak semua tema).
-- ═══════════════════════════════════════════════════════════════════════════

INSERT INTO animations (kind, slug, name, spec, css, builtin, sort_order)
VALUES ('scroll', 'anggun', 'Anggun bertahap (ala undangan premium)', '{{}}'::jsonb,
        '--rv-from:translateY(60px);--rv-dur:1.25s;--rv-stagger:220;', TRUE, 15)
ON CONFLICT (kind, slug) DO NOTHING;

UPDATE themes SET scroll_anim = 'anggun', gerak_judul = 'zoom-masuk', gerak_foto = 'zoom-masuk', ken_burns = TRUE, updated_at = NOW();

DELETE FROM theme_ornaments WHERE img LIKE '/img/tema/ornamen/%' AND updated_at = created_at;

INSERT INTO theme_ornaments (theme, bagian, img, posisi, x, y, lebar, rotasi, cermin, masuk, jeda, durasi, gerak, kecepatan, depan, hp, opasitas, urutan)
SELECT t.slug, v.bagian, '/img/tema/ornamen/' || v.bentuk || '-' || {PALET_SQL} || '.svg',
       v.posisi, v.x, v.y, v.lebar, v.rotasi, v.cermin, v.masuk, v.jeda, v.durasi, v.gerak, v.kecepatan, v.depan, v.hp, v.opasitas, v.urutan
  FROM themes t
 CROSS JOIN (VALUES
{",\n".join(vals)}
 ) AS v(bagian, bentuk, posisi, x, y, lebar, rotasi, cermin, masuk, jeda, durasi, gerak, kecepatan, depan, hp, opasitas, urutan)
 WHERE NOT EXISTS (SELECT 1 FROM theme_ornaments o WHERE o.theme = t.slug)
 ORDER BY t.slug, v.urutan;
"""
with open(os.path.join(ROOT, "migration/013_gerak_everlove.sql"), "w") as fh:
    fh.write(sql)
print("svg:", len(PAL) * len(SHAPES), "| ornamen per tema:", len(SET))
