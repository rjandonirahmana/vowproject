# Bangkitkan 38 tema provinsi: aset SVG + migration/031_tema_provinsi.sql.
#   python3 scripts/provinsi/build.py .            (dari akar proyek)
#
# Sumber data: wilayah.py. Gambar dipakai ulang dari dua generator lama:
#   scripts/nusantara/motifs.py  — ubin motif, lambang, hiasan kartu, bingkai,
#                                  latar, daun pintu animasi buka;
#   scripts/ornamen/adat.py      — ornamen khas daerah (tokoh, ikon, hiasan
#                                  gantung, kipas sudut, lambang) + penempatan
#                                  & gerak 10 ornamen per tema.
# Warna tiap berkas mengikuti tema masing-masing (token + cek kontras dari
# scripts/nusantara/build.py — gagal kontras = berhenti).
#
# SVG & 031_tema_provinsi.sql = ARTEFAK: jangan sunting tangan, ubah wilayah.py
# lalu jalankan ulang. Migrasi idempoten (ON CONFLICT DO NOTHING; ornamen hanya
# untuk tema yang belum punya ornamen) — suntingan admin tak ditimpa.
import json, os, sys

ROOT = os.path.abspath(sys.argv[1] if len(sys.argv) > 1 else ".")
# Data kedua: `suku` (scripts/provinsi/suku.py → 034_tema_suku.sql).
DATA = sys.argv[2] if len(sys.argv) > 2 else "wilayah"
HERE = os.path.dirname(os.path.abspath(__file__))
# Generator lama membaca sys.argv[1] sebagai akar proyek saat diimpor.
sys.argv = [sys.argv[0], ROOT]
sys.path[:0] = [os.path.join(HERE, "..", "nusantara"), os.path.join(HERE, "..", "ornamen")]
import motifs as M  # noqa: E402
from build import tokens, palette_key, q  # noqa: E402  (scripts/nusantara/build.py)
import adat  # noqa: E402
sys.path.insert(0, HERE)
from wilayah import PROVINSI, VARIAN_BUKA, AKSEN  # noqa: E402

if DATA == "suku":
    from suku import SUKU as PROVINSI, KEPALA  # noqa: E402
    MAP, SQL, SORT, PREFIX, ANIM_SORT = "suku", "034_tema_suku.sql", 400, "suku", 130
else:
    KEPALA = None
    MAP, SQL, SORT, PREFIX, ANIM_SORT = "provinsi", "031_tema_provinsi.sql", 300, "prov", 90
OUT = os.path.join(ROOT, "public/img/tema", MAP)
URL = "/img/tema/" + MAP
os.makedirs(os.path.join(OUT, "anim"), exist_ok=True)

VALID_OPEN = {"tirai", "gerbang", "amplop", "pudar", "gebyok-ukir", "taman-daun", "galaksi", "candi-bentar", "pagelaran-wayang",
              "layar-naik", "tenun-songket", "gebyok-jawa", "mega-mendung", "songket-minang", "gorga-batak", "tenun-maluku", "lawang-bali"}
VALID_SCROLL = {"naik", "pudar", "zoom", "geser", "lipat", "blur", "anggun", "keraton", "mekar", "kosmik", "ombak", "tenun", "bayang"}
VALID_FLOAT = {"none", "kelopak", "kupu", "bintang", "daun-gugur", "kunang", "burung", "kilau-emas", "melati-gugur", "kamboja-gugur",
               "cengkih-gugur", "bulu-enggang", "daun-sirih"}
VALID_LAYOUT = {"klasik", "bingkai", "gerbang", "editorial"}
VALID_FONT = {"playfair", "cormorant", "cinzel", "marcellus", "dm-serif", "lora"}
VALID_SCRIPT = {"great-vibes", "parisienne", "allura", "alex-brush"}


def write(name, content):
    with open(os.path.join(OUT, name), "w") as fh:
        fh.write(content)
    return f"{URL}/{name}"


def b(v):
    return "TRUE" if v else "FALSE"


def main():
    anim_rows, theme_rows, orn_rows, seen = [], [], [], set()
    for n, (slug, name, prov, nuansa, desc, tags, primary, gold, bg, dark, set_key, tile, emblem, ikon,
            buka, gulir, hiasan, layout, font, script) in enumerate(PROVINSI):
        assert slug not in seen, slug
        seen.add(slug)
        assert tile in M.TILES and emblem in M.EMBLEMS, (slug, tile, emblem)
        assert gulir in VALID_SCROLL and hiasan in VALID_FLOAT and layout in VALID_LAYOUT, slug
        assert font in VALID_FONT and script in VALID_SCRIPT and set_key in adat.SETS, slug
        tk = tokens(primary, gold, bg, dark)
        accent = AKSEN.get(set_key, primary)
        col = dict(primary=tk["primary"] if not dark else gold, gold=gold, bg=bg, card=tk["card"], ink=tk["ink"], accent=accent)

        # ── Animasi buka ────────────────────────────────────────────────────
        if buka.startswith("khas:"):
            spec = dict(VARIAN_BUKA[buka[5:]])
            spec.update(fill="image", border=True,
                        panel_image=write(f"anim/pintu-{slug}.svg", M.panel(col, tile, emblem)),
                        orn_image=write(f"anim/lambang-{slug}.svg", M.svg(200, 200, M.EMBLEMS[emblem](col, 100, 120))))
            open_anim = f"{PREFIX}-{slug}"[:48]
            anim_rows.append((open_anim, f"Pintu {name}", spec, ANIM_SORT + n))
        else:
            assert buka in VALID_OPEN, (slug, buka)
            open_anim = buka

        # ── Aset tema (kartu katalog, hiasan kartu, bingkai, latar) ─────────
        if n % 3 == 2:
            image_url, mode = write(f"{slug}-sudut.svg", M.corner(col, tile)), "sudut"
        else:
            image_url, mode = write(f"{slug}-motif.svg", M.tile_svg(col, tile)), "pola"
        deco = write(f"{slug}-hiasan.svg", M.card_deco(col, emblem, tile))
        frame = write(f"{slug}-bingkai.svg", M.frame(col, tile)) if layout != "bingkai" and n % 2 == 0 else ""
        bg_img = write(f"{slug}-latar.svg", M.background(col, tile, emblem)) if n % 3 == 0 else ""

        # ── Ornamen khas daerah (warna tema ini) ─────────────────────────────
        S = dict(adat.SETS[set_key])
        c = adat.palette({"tokens": tk, "dark": dark}, accent)
        ikon_fn = getattr(adat, ikon) if ikon else S["ikon"][0]
        tf = S["tokoh"][0]
        pieces = {
            "tokoh": tf(c, tile) if tf is not adat.kain_gantung else adat.kain_gantung(c, tile, S.get("garis", False)),
            "ikon": ikon_fn(c, tile),
            "atas": S["atas"][0](c, tile),
            "sudut": S["sudut"](c, tile),
        }
        if S.get("emblem"):
            pieces["lambang"] = adat.lambang(c, tile, emblem)
        urls = {k: write(f"{slug}-orn-{k}.svg", v) for k, v in pieces.items()}
        for i, (bag, piece, pos, x, y, w, rot, mir, masuk, jeda, dur, gerak, spd, depan, hp, op) in enumerate(adat.placements(S)):
            orn_rows.append(f"    ({q(slug)}, {q(bag)}, {q(urls[piece])}, {q(pos)}, {x}, {y}, {w}, {rot}, {b(mir)}, {q(masuk)}, {jeda}, {dur}, "
                            f"{q(gerak)}, {spd}, {b(depan)}, {b(hp)}, {op}, {(i + 1) * 10})")

        theme_rows.append(dict(
            slug=slug, name=name, category="Luxury" if dark else "Adat", nuansa=nuansa, palette=palette_key(primary, dark, gold),
            region=prov, description=desc, tags=tags + [f"Provinsi {prov}"], badge="Baru", layout=layout, ornament="none",
            font=font, tokens=tk, dark=dark, sort_order=SORT + n, script_font=script, bg_image=bg_img, frame_image=frame,
            card_deco=deco, float_deco=hiasan, open_anim=open_anim, page_mode="satu", scroll_anim=gulir, image_url=image_url,
            image_mode=mode, gerak_judul="zoom-masuk", gerak_foto="zoom-masuk", ken_burns=True, listed=True,
        ))

    cols = ["slug", "name", "category", "nuansa", "palette", "region", "description", "tags", "badge", "layout", "ornament", "font",
            "tokens", "dark", "sort_order", "script_font", "bg_image", "frame_image", "card_deco", "float_deco", "open_anim",
            "page_mode", "scroll_anim", "image_url", "image_mode", "gerak_judul", "gerak_foto", "ken_burns", "listed"]

    def val(v):
        if isinstance(v, (list, dict)):
            return q(json.dumps(v, ensure_ascii=False)) + "::jsonb"
        if isinstance(v, bool):
            return b(v)
        if isinstance(v, int):
            return str(v)
        return q(v)

    out = [KEPALA] if KEPALA else ["""-- ═══════════════════════════════════════════════════════════════════════════
-- 031_tema_provinsi — 38 tema undangan, SATU PER PROVINSI Indonesia (Aceh s/d
-- Papua Selatan). DIBANGKITKAN scripts/provinsi/build.py — jangan disunting
-- tangan; ubah tema lewat /admin/tema setelah tayang.
--
-- Tiap tema berbeda bentuk & gerak, bukan sekadar warna: ornamen khas daerah
-- (tokoh pengapit, bangunan adat, hiasan gantung, kipas motif, lambang
-- berputar — 10 ornamen per tema, warna tema sendiri), ubin motif kain,
-- animasi BUKA (20 pintu khas baru bermotif tema + animasi daerah yang sudah
-- ada), koreografi gulir, hiasan melayang, tata letak & huruf.
--
-- Aset: public/img/tema/provinsi/ (ikut image Docker lewat public/).
-- Idempoten: ON CONFLICT DO NOTHING; ornamen hanya diisi untuk tema yang
-- belum punya ornamen — suntingan admin tak pernah ditimpa.
-- WAJIB setelah 029 (kolom tema terbaru) & 030.
-- ═══════════════════════════════════════════════════════════════════════════
"""]
    out.append("-- ── Animasi buka khas provinsi ──" if DATA != "suku" else "-- ── Animasi buka khas suku ──")
    for slug, nm, spec, so in anim_rows:
        out.append(f"INSERT INTO animations (kind, slug, name, spec, css, builtin, sort_order) VALUES ('buka', {q(slug)}, {q(nm)}, "
                   f"{q(json.dumps(spec, ensure_ascii=False))}::jsonb, '', FALSE, {so}) ON CONFLICT (kind, slug) DO NOTHING;")
    out.append("\n-- ── Tema ──")
    for r in theme_rows:
        out.append(f"-- {r['region']}: {r['name']}")
        out.append(f"INSERT INTO themes ({', '.join(cols)})\nVALUES ({', '.join(val(r[c]) for c in cols)})\nON CONFLICT (slug) DO NOTHING;")
    out.append("""
-- ── Ornamen khas daerah (hanya tema yang belum punya ornamen) ──
INSERT INTO theme_ornaments (theme, bagian, img, posisi, x, y, lebar, rotasi, cermin, masuk, jeda, durasi, gerak, kecepatan, depan, hp, opasitas, urutan, source)
SELECT a.theme, a.bagian, a.img, a.posisi, a.x, a.y, a.lebar, a.rotasi, a.cermin, a.masuk, a.jeda, a.durasi, a.gerak, a.kecepatan, a.depan, a.hp, a.opasitas, a.urutan, 'seed'
  FROM (VALUES
""" + ",\n".join(orn_rows) + """
  ) AS a(theme, bagian, img, posisi, x, y, lebar, rotasi, cermin, masuk, jeda, durasi, gerak, kecepatan, depan, hp, opasitas, urutan)
 WHERE EXISTS (SELECT 1 FROM themes t WHERE t.slug = a.theme)
   AND NOT EXISTS (SELECT 1 FROM theme_ornaments o WHERE o.theme = a.theme)
 ORDER BY a.theme, a.urutan;
""")
    with open(os.path.join(ROOT, "migration", SQL), "w") as fh:
        fh.write("\n".join(out))

    from collections import Counter
    print("tema:", len(theme_rows), "| animasi buka baru:", len(anim_rows), "| ornamen:", len(orn_rows))
    print("buka:", dict(Counter(r["open_anim"] if not r["open_anim"].startswith(PREFIX + "-") else PREFIX + "-*" for r in theme_rows)))
    print("gulir:", dict(Counter(r["scroll_anim"] for r in theme_rows)))
    print("hiasan:", dict(Counter(r["float_deco"] for r in theme_rows)))
    print("berkas SVG:", sum(len(fs) for _, _, fs in os.walk(OUT)))


if __name__ == "__main__":
    main()
