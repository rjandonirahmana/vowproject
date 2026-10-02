# Bangkitkan aset SVG + migration/009_tema_nusantara.sql dari data.py.
#   cd scripts/nusantara && python3 build.py ../..
# Ubah/tambah tema di data.py, motif di motifs.py, lalu jalankan ulang.
import json, os, re, sys, colorsys
from data import REGIONS, THEMES, ANIM_OPEN, ANIM_FLOAT
import motifs as M

ROOT = sys.argv[1]
OUT_IMG = os.path.join(ROOT, "public/img/tema/nusantara")
URL = "/img/tema/nusantara"
os.makedirs(os.path.join(OUT_IMG, "anim"), exist_ok=True)

# ── Warna ──────────────────────────────────────────────────────────────────
def rgb(h):
    h = h.lstrip("#"); return tuple(int(h[i:i+2], 16) / 255 for i in (0, 2, 4))
def hexc(t):
    return "#" + "".join(f"{max(0, min(255, round(v*255))):02x}" for v in t)
def mix(a, b, t):  # t = porsi b
    A, B = rgb(a), rgb(b); return hexc(tuple(x*(1-t)+y*t for x, y in zip(A, B)))
def lum(h):
    c = [v/12.92 if v <= 0.03928 else ((v+0.055)/1.055)**2.4 for v in rgb(h)]
    return 0.2126*c[0] + 0.7152*c[1] + 0.0722*c[2]
def contrast(a, b):
    x, y = lum(a), lum(b); return (max(x, y)+0.05) / (min(x, y)+0.05)
def push(fg, bg, target, toward):
    """Geser `fg` ke arah `toward` sampai kontras ≥ target."""
    c = fg
    for _ in range(40):
        if contrast(c, bg) >= target:
            return c
        c = mix(c, toward, 0.08)
    return c

def tokens(primary, gold, bg, dark):
    if not dark:
        card = mix(bg, "#ffffff", 0.65)
        primary = push(primary, bg, 6.0, "#000000")
        ink = push(mix(primary, "#000000", 0.55), bg, 12.0, "#000000")
        on = "#ffffff" if contrast("#ffffff", primary) >= 4.5 else ink
        gold_deep = push(mix(gold, "#000000", 0.35), bg, 4.8, "#000000")
        muted = push(mix(ink, bg, 0.42), bg, 4.8, ink)
    else:
        card = mix(bg, "#ffffff", 0.07)
        ink = push(mix(gold, "#ffffff", 0.75), bg, 12.0, "#ffffff")
        primary = push(primary, bg, 7.0, "#ffffff")
        on = mix(bg, "#000000", 0.2)
        gold_deep = push(gold, bg, 6.0, "#ffffff")
        muted = push(mix(ink, bg, 0.4), bg, 4.8, ink)
    t = {"bg": bg, "card": card, "primary": primary, "on-primary": on, "gold": gold, "gold-deep": gold_deep, "ink": ink, "muted": muted}
    assert contrast(t["primary"], bg) >= 4.5 and contrast(t["gold-deep"], bg) >= 4.5 and contrast(t["muted"], card) >= 4.5, t
    assert contrast(t["on-primary"], t["primary"]) >= 4.5, t
    return t

def palette_key(primary, dark, gold):
    if dark:
        return "mono" if lum(gold) < 0.3 else "gold"
    h, l, s = colorsys.rgb_to_hls(*rgb(primary))
    deg = h * 360
    if s < 0.15:
        return "mono"
    if 70 <= deg < 175: return "sage"
    if 175 <= deg < 260: return "navy"
    if 290 <= deg < 345: return "blush"
    if 35 <= deg < 70: return "gold"
    return "terra"

def slugify(s):
    s = s.lower().replace("'", "")
    return re.sub(r"-+", "-", re.sub(r"[^a-z0-9]+", "-", s)).strip("-")[:48]

def q(s):
    return "'" + str(s).replace("'", "''") + "'"

def write(name, content):
    with open(os.path.join(OUT_IMG, name), "w") as fh:
        fh.write(content)
    return f"{URL}/{name}"

# ── Animasi khas daerah ─────────────────────────────────────────────────────
anim_rows = []
for i, (slug, (name, col, tile, emblem, extra)) in enumerate(ANIM_OPEN.items()):
    panel = write(f"anim/pintu-{slug}.svg", M.panel(col, tile, emblem))
    orn = write(f"anim/ornamen-{slug}.svg", M.svg(200, 200, M.EMBLEMS[emblem](col, 100, 120)))
    spec = dict(fill="image", panel_image=panel, border=True, orn_image=orn, **extra)
    anim_rows.append(("buka", slug, name, spec, 60 + i))
for i, (slug, (name, img, extra)) in enumerate(ANIM_FLOAT.items()):
    url = write(f"anim/{img}.svg", M.FLOATS[img]())
    anim_rows.append(("hiasan", slug, name, dict(float_image=url, **extra), 60 + i))

# ── Tema ───────────────────────────────────────────────────────────────────
LAYOUTS = ["gerbang", "bingkai", "klasik", "editorial", "gerbang", "klasik"]
rows, seen, n = [], set(), 0
for region, items in THEMES.items():
    R = REGIONS[region]
    for i, (name, region_label, desc, tags, primary, gold, bg, dark, tile, emblem) in enumerate(items):
        slug = slugify(name)
        assert slug not in seen, slug
        seen.add(slug)
        tk = tokens(primary, gold, bg, dark)
        col = dict(primary=tk["primary"] if not dark else gold, gold=gold, bg=bg, card=tk["card"], ink=tk["ink"], accent=R["accent"])
        # Aset: motif (pola/sudut), hiasan kartu, bingkai (selang-seling), latar (tiap ke-3).
        if n % 3 == 2:
            image_url, mode = write(f"{slug}-sudut.svg", M.corner(col, tile)), "sudut"
        else:
            image_url, mode = write(f"{slug}-motif.svg", M.tile_svg(col, tile)), "pola"
        deco = write(f"{slug}-hiasan.svg", M.card_deco(col, emblem, tile))
        frame = write(f"{slug}-bingkai.svg", M.frame(col, tile)) if i % 2 == 0 else ""
        bg_img = write(f"{slug}-latar.svg", M.background(col, tile, emblem)) if n % 3 == 0 else ""
        layout = "bingkai" if dark else LAYOUTS[n % len(LAYOUTS)]
        category = "Luxury" if dark else ("Modern" if "Modern" in name or "Modern" in region_label else "Adat")
        badge = "Baru • Adat" if i == 0 else ("Favorit" if i == 2 else "")
        open_anim = R["opens"][i % len(R["opens"])]
        rows.append(dict(
            slug=slug, name=name, category=category, nuansa=R["nuansa"], palette=palette_key(primary, dark, gold),
            region=region_label, description=desc, tags=tags, badge=badge, layout=layout, ornament="none",
            font=R["fonts"][i % len(R["fonts"])], tokens=tk, dark=dark, sort_order=200 + n,
            script_font=R["scripts"][i % len(R["scripts"])], bg_image=bg_img, frame_image=frame, card_deco=deco,
            float_deco=R["floats"][i % len(R["floats"])], open_anim=open_anim, page_mode="satu",
            scroll_anim=R["scrolls"][i % len(R["scrolls"])], image_url=image_url, image_mode=mode,
        ))
        n += 1

assert n == 100, n
valid_open = {"none", "tirai", "gerbang", "amplop", "pudar"} | set(ANIM_OPEN)
valid_float = {"none", "kelopak", "kupu", "bintang"} | set(ANIM_FLOAT)
valid_scroll = {"naik", "pudar", "zoom", "geser", "lipat", "blur", "none"}
for r in rows:
    assert r["open_anim"] in valid_open and r["float_deco"] in valid_float and r["scroll_anim"] in valid_scroll, r["slug"]

# ── SQL ────────────────────────────────────────────────────────────────────
cols = ["slug", "name", "category", "nuansa", "palette", "region", "description", "tags", "badge", "layout", "ornament", "font",
        "tokens", "dark", "sort_order", "script_font", "bg_image", "frame_image", "card_deco", "float_deco", "open_anim",
        "page_mode", "scroll_anim", "image_url", "image_mode", "listed"]
def val(r, c):
    v = r.get(c, True if c == "listed" else None)
    if c in ("tags", "tokens"):
        return q(json.dumps(v, ensure_ascii=False)) + "::jsonb"
    if isinstance(v, bool):
        return "TRUE" if v else "FALSE"
    if isinstance(v, int):
        return str(v)
    return q(v)

out = ["""-- ═══════════════════════════════════════════════════════════════════════════
-- 009_tema_nusantara — 100 tema undangan adat Nusantara + 11 animasi khas
-- daerah. DIBANGKITKAN scripts/nusantara/build.py — jangan disunting tangan;
-- ubah tema lewat /admin/tema & animasi lewat /admin/animasi.
--
-- WAJIB setelah 008_animasi_semua.sql (tabel animations versi baru).
-- Idempoten: ON CONFLICT DO NOTHING — tema/animasi yang sudah disunting admin
-- tidak ditimpa bila migrasi dijalankan ulang.
--
-- Aset gambar: public/img/tema/nusantara/ (motif, hiasan kartu, bingkai,
-- latar, daun pintu animasi, hiasan melayang).
-- ═══════════════════════════════════════════════════════════════════════════
""", "-- ── Animasi khas daerah ──"]
for kind, slug, name, spec, so in anim_rows:
    out.append(f"INSERT INTO animations (kind, slug, name, spec, css, builtin, sort_order) VALUES ({q(kind)}, {q(slug)}, {q(name)}, "
               f"{q(json.dumps(spec, ensure_ascii=False))}::jsonb, '', FALSE, {so}) ON CONFLICT (kind, slug) DO NOTHING;")
out.append("\n-- ── Tema ──")
for r in rows:
    out.append(f"-- {r['nuansa']}: {r['name']}")
    out.append(f"INSERT INTO themes ({', '.join(cols)})\nVALUES ({', '.join(val(r, c) for c in cols)})\nON CONFLICT (slug) DO NOTHING;")
with open(os.path.join(ROOT, "migration/009_tema_nusantara.sql"), "w") as fh:
    fh.write("\n".join(out) + "\n")

# Ringkasan
from collections import Counter
print("tema:", len(rows), "| animasi:", len(anim_rows))
print("per daerah:", dict(Counter(r["nuansa"] for r in rows)))
print("palet:", dict(Counter(r["palette"] for r in rows)))
print("kategori:", dict(Counter(r["category"] for r in rows)))
print("berkas SVG:", sum(len(fs) for _, _, fs in os.walk(OUT_IMG)))
