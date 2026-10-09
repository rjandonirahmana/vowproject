# Bangkitkan RUPA (varian struktur per bagian, web/rupa.rs) untuk semua tema
# + perbaikan gerak kembar → bagian data migration/033_rupa.sql.
#
#   python3 scripts/rupa/build.py .              (dari akar proyek)
#
# Masukan: scripts/rupa/tema.json — dump tema TAYANG (lihat README.md).
#
# Aturan:
#   1. Tiap pasangan tema berbeda di ≥ JARAK_MIN dari 8 bagian rupa (serumpun ≥ JARAK_SERUMPUN) — diukur
#      piksel, sebelum ini 603 dari 1275 pasangan "struktur > 0,85 identik".
#   2. Varian dipilih sesuai rumpun budaya (AFINITAS): kubah untuk kesultanan
#      Melayu/Aceh, gunungan untuk Jawa, atap runcing untuk Minang/Batak/Toraja…
#      lalu disebar merata (varian yang sudah sering dipakai diberi penalti).
#   3. Tak ada dua tema dengan pasangan (animasi buka, gerak gulir) yang sama —
#      yang kembar diganti gerak gulirnya (hanya tema yang belum dikunci admin).
#   Tema templat & tema video (koreografi khusus) tidak diberi rupa.
#
# SQL idempoten & aman bagi suntingan admin: rupa hanya ditulis bila masih
# kosong, gerak hanya bila motion_locked = FALSE.
import json, os, random, sys
from itertools import combinations

ROOT = os.path.abspath(sys.argv[1] if len(sys.argv) > 1 else ".")
HERE = os.path.dirname(os.path.abspath(__file__))
JARAK_MIN = 5          # semua pasangan (dari 8 bagian)
JARAK_SERUMPUN = 6     # pasangan serumpun (motif & warna mirip → struktur harus lebih beda)

# Harus sama dengan web/rupa.rs BAGIAN (test rust memeriksa CSS-nya).
BAGIAN = {
    "sampul": ["asli", "kubah", "gunungan", "atap", "lingkaran", "jendela", "penuh", "wajik", "medali"],
    "judul": ["asli", "kapital", "pita", "bingkai", "samping", "tumpal"],
    "mempelai": ["asli", "sejajar", "zigzag", "medali", "panel", "potret"],
    "kisah": ["asli", "kartu", "selang", "angka", "tengah", "polaroid"],
    "acara": ["asli", "tiket", "lontar", "linimasa", "kalender", "tegas"],
    "galeri": ["asli", "kolom", "film", "mozaik", "polaroid"],
    "kartu": ["asli", "tegas", "kaca", "potong", "bertepi", "cetak"],
    "pemisah": ["asli", "wajik", "ombak", "tumpal", "titik", "ganda"],
}
KUNCI = list(BAGIAN)

# Rumpun → varian yang cocok per bagian (urutan = kekuatan kecocokan).
# Kosong = bebas (tanpa preferensi).
AFINITAS = {
    "Aceh":           dict(kisah=["angka", "kartu"], sampul=["kubah", "medali"], judul=["bingkai", "kapital"], acara=["tegas", "asli"], kartu=["potong", "tegas"], pemisah=["wajik", "ganda"]),
    "Melayu":         dict(kisah=["selang", "tengah"], sampul=["kubah", "wajik", "lingkaran"], judul=["tumpal", "bingkai", "pita"], acara=["lontar", "asli"], kartu=["bertepi", "potong"], pemisah=["tumpal", "wajik", "ombak"]),
    "Palembang":      dict(sampul=["kubah", "medali"], judul=["tumpal", "kapital"], kartu=["potong", "bertepi"], pemisah=["tumpal", "ganda"]),
    "Lampung":        dict(sampul=["wajik", "medali"], judul=["tumpal", "pita"], kartu=["bertepi", "potong"], pemisah=["tumpal", "wajik"]),
    "Minang":         dict(sampul=["atap", "kubah"], judul=["tumpal", "kapital"], acara=["tegas", "lontar"], kartu=["bertepi", "potong"], pemisah=["tumpal", "wajik"]),
    "Batak":          dict(kisah=["angka", "kartu"], sampul=["atap", "wajik"], judul=["kapital", "bingkai"], acara=["tegas", "tiket"], kartu=["tegas", "bertepi"], pemisah=["ganda", "tumpal"]),
    "Jawa":           dict(kisah=["angka", "kartu"], sampul=["gunungan", "asli", "medali"], judul=["asli", "kapital", "bingkai"], acara=["lontar", "kalender", "asli"], kartu=["potong", "asli"], pemisah=["wajik", "titik"]),
    "Sunda":          dict(kisah=["selang", "polaroid"], sampul=["gunungan", "lingkaran"], judul=["pita", "asli"], acara=["kalender", "lontar"], pemisah=["ombak", "titik"]),
    "Betawi":         dict(kisah=["polaroid", "selang"], sampul=["jendela", "penuh"], judul=["pita", "samping"], acara=["tiket", "kalender"], galeri=["polaroid", "film"], kartu=["cetak", "tegas"], pemisah=["titik", "ganda"]),
    "Bali":           dict(kisah=["polaroid", "tengah"], sampul=["medali", "gunungan", "asli"], judul=["bingkai", "asli"], acara=["lontar", "asli"], galeri=["polaroid", "mozaik"], kartu=["potong", "kaca"], pemisah=["titik", "wajik"]),
    "Sasak":          dict(sampul=["wajik", "lingkaran"], judul=["tumpal", "pita"], acara=["lontar", "kalender"], kartu=["bertepi"], pemisah=["tumpal", "wajik"]),
    "NTT":            dict(sampul=["wajik", "lingkaran"], judul=["tumpal", "kapital"], acara=["tiket", "tegas"], kartu=["bertepi", "tegas"], pemisah=["tumpal", "titik"]),
    "Dayak":          dict(sampul=["lingkaran", "wajik"], judul=["kapital", "samping"], acara=["linimasa", "tegas"], kartu=["potong", "tegas"], pemisah=["titik", "tumpal"]),
    "Banjar":         dict(sampul=["kubah", "jendela"], judul=["samping", "pita"], acara=["kalender", "tiket"], kartu=["kaca", "bertepi"], pemisah=["ombak", "wajik"]),
    "Minahasa":       dict(kisah=["polaroid", "kartu"], sampul=["jendela", "lingkaran"], judul=["pita", "samping"], acara=["kalender", "tiket"], galeri=["polaroid", "mozaik"], kartu=["cetak", "kaca"], pemisah=["titik", "ganda"]),
    "Gorontalo":      dict(sampul=["kubah", "medali"], judul=["samping", "bingkai"], acara=["kalender", "lontar"], kartu=["kaca", "potong"], pemisah=["wajik", "titik"]),
    "Kaili":          dict(sampul=["atap", "medali"], judul=["kapital", "bingkai"], acara=["linimasa", "tegas"], pemisah=["ganda", "wajik"]),
    "Mandar":         dict(sampul=["penuh", "lingkaran"], judul=["samping", "kapital"], acara=["linimasa", "tiket"], galeri=["film", "kolom"], pemisah=["ombak", "titik"]),
    "Bugis-Makassar": dict(sampul=["wajik", "atap"], judul=["kapital", "tumpal"], acara=["lontar", "tegas"], kartu=["bertepi", "potong"], pemisah=["wajik", "ombak"]),
    "Toraja":         dict(kisah=["angka", "tengah"], sampul=["atap", "wajik"], judul=["kapital", "bingkai"], acara=["tegas", "linimasa"], kartu=["potong", "tegas"], pemisah=["ganda", "tumpal"]),
    "Buton":          dict(sampul=["medali", "kubah"], judul=["bingkai", "kapital"], acara=["tegas", "kalender"], kartu=["tegas", "potong"], pemisah=["ganda", "ombak"]),
    "Maluku":         dict(kisah=["tengah", "selang"], sampul=["lingkaran", "kubah", "penuh"], judul=["pita", "samping"], acara=["linimasa", "tiket"], galeri=["film", "polaroid"], kartu=["kaca", "cetak"], pemisah=["ombak", "titik"]),
    "Papua":          dict(kisah=["tengah", "kartu"], sampul=["lingkaran", "penuh", "wajik"], judul=["kapital", "samping"], acara=["linimasa", "tegas"], galeri=["mozaik", "film"], kartu=["tegas", "potong"], pemisah=["titik", "tumpal"]),
    "Islami":         dict(sampul=["kubah", "medali"], judul=["bingkai", "kapital"], pemisah=["wajik", "ganda"]),
    "Modern":         dict(kisah=["polaroid", "selang"], sampul=["penuh", "jendela", "lingkaran"], judul=["samping", "kapital"], acara=["tiket", "kalender"], galeri=["film", "mozaik"], kartu=["kaca", "cetak", "tegas"], pemisah=["ganda", "titik"]),
}
# Petunjuk tambahan dari atribut tema.
def afinitas(t):
    a = {k: list(v) for k, v in AFINITAS.get(t["nuansa"], {}).items()}
    if t.get("layout") == "editorial":
        a["judul"] = ["samping"] + a.get("judul", [])
        a["sampul"] = ["jendela", "penuh"] + a.get("sampul", [])
    if t.get("dark"):
        a["kartu"] = ["kaca"] + a.get("kartu", [])
    if t["scroll_anim"] in ("ombak",) or "Bahari" in t.get("name", "") or "laut" in t.get("name", "").lower():
        a["pemisah"] = ["ombak"] + a.get("pemisah", [])
    return a

# Gerak gulir pengganti untuk pasangan (buka, gulir) kembar — per rumpun.
GULIR_GANTI = {
    "Jawa": ["bayang", "anggun", "mekar", "lipat"], "Dayak": ["tenun", "zoom", "naik"], "Maluku": ["naik", "lipat", "pudar"],
    "Papua": ["naik", "tenun", "pudar"], "Palembang": ["tenun", "anggun", "lipat"], "Melayu": ["anggun", "lipat", "pudar"],
}
GULIR_UMUM = ["anggun", "naik", "lipat", "pudar", "tenun", "zoom", "geser", "mekar", "ombak", "bayang"]


def skor_pilihan(pref, v):
    if v in pref:
        return 6 - 2 * pref.index(v)  # 6, 4, 2 …
    return 0


def main():
    tema = json.load(open(os.path.join(HERE, "tema.json")))
    ikut = [t for t in tema if not t.get("template") and not t.get("bg_video")]
    rng = random.Random(20261009)
    rumpun = {t["slug"]: t["nuansa"] for t in tema}
    pakai = {k: {v: 0 for v in vs} for k, vs in BAGIAN.items()}
    hasil = {}

    for t in ikut:
        a = afinitas(t)
        terbaik, skor_terbaik = None, -1e9
        # Sampel kombinasi berbobot: varian afinitas lebih sering ditarik.
        for _ in range(40000):
            pilih = []
            for k in KUNCI:
                vs = BAGIAN[k]
                w = [1 + skor_pilihan(a.get(k, []), v) for v in vs]
                pilih.append(rng.choices(vs, weights=w)[0])
            pilih = tuple(pilih)
            if any(sum(x != y for x, y in zip(pilih, lain)) < (JARAK_SERUMPUN if rumpun[s2] == t["nuansa"] else JARAK_MIN)
                   for s2, lain in hasil.items()):
                continue
            s = sum(skor_pilihan(a.get(k, []), v) for k, v in zip(KUNCI, pilih))
            s -= sum(pakai[k][v] * 0.9 for k, v in zip(KUNCI, pilih))  # sebar merata
            s -= 3 * sum(v == "asli" for v in pilih)  # bawaan dipakai seperlunya
            if s > skor_terbaik:
                terbaik, skor_terbaik = pilih, s
        assert terbaik, f"tak ada kombinasi sah untuk {t['slug']} — kurangi JARAK_MIN atau tambah varian"
        hasil[t["slug"]] = terbaik
        for k, v in zip(KUNCI, terbaik):
            pakai[k][v] += 1

    # Gerak kembar: (buka, gulir) unik di antara tema tayang.
    dipakai, ganti = {}, {}
    for t in tema:
        key = (t["open_anim"], t["scroll_anim"])
        if key not in dipakai:
            dipakai[key] = t["slug"]
            continue
        pool = GULIR_GANTI.get(t["nuansa"], []) + GULIR_UMUM
        baru = next(g for g in pool if (t["open_anim"], g) not in dipakai and g != t["scroll_anim"])
        dipakai[(t["open_anim"], baru)] = t["slug"]
        ganti[t["slug"]] = (t["scroll_anim"], baru, dipakai[key])

    # Tema suku (sort ≥ 400, migrasi 034) baru ada SETELAH 033 → rupanya di 035.
    suku = {t["slug"] for t in tema if t["sort_order"] >= 400}
    jarak = [sum(x != y for x, y in zip(hasil[a], hasil[b])) for a, b in combinations(hasil, 2)]
    serumpun = [sum(x != y for x, y in zip(hasil[a], hasil[b])) for a, b in combinations(hasil, 2) if rumpun[a] == rumpun[b]]
    for nama, kepala, pilih_slug in (("033_rupa.sql", "kepala.sql", lambda s: s not in suku), ("035_rupa_suku.sql", "kepala-suku.sql", lambda s: s in suku)):
        out = [open(os.path.join(HERE, kepala)).read().rstrip(), "", "-- ── Rupa per tema (hanya bila admin belum mengaturnya) ──"]
        for slug, pilih in hasil.items():
            if not pilih_slug(slug):
                continue
            r = {k: v for k, v in zip(KUNCI, pilih) if v != "asli"}
            out.append(f"UPDATE themes SET rupa = '{json.dumps(r, ensure_ascii=False)}'::jsonb WHERE slug = '{slug}' AND rupa = '{{}}'::jsonb;")
        out += ["", "-- ── Gerak gulir kembar diganti (pasangan buka+gulir kini unik) ──"]
        for slug, (lama, baru, kembar) in ganti.items():
            if pilih_slug(slug):
                out.append(f"-- {slug}: {lama} → {baru} (kembar dengan {kembar})")
                out.append(f"UPDATE themes SET scroll_anim = '{baru}' WHERE slug = '{slug}' AND scroll_anim = '{lama}' AND NOT motion_locked;")
        open(os.path.join(ROOT, "migration", nama), "w").write("\n".join(out) + "\n")

    print(f"tema diberi rupa: {len(hasil)} | jarak antar-pasangan: min {min(jarak)}, rata {sum(jarak) / len(jarak):.2f}"
          f" | serumpun: min {min(serumpun)} ({len(serumpun)} pasangan)")
    for k in KUNCI:
        print(f"  {k:9}", " ".join(f"{v}:{n}" for v, n in pakai[k].items()))
    print("gerak diganti:", len(ganti), {s: f"{a}→{b}" for s, (a, b, _) in ganti.items()})


if __name__ == "__main__":
    main()
