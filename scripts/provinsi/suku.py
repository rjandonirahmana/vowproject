# 24 tema SUKU / budaya daerah yang belum punya tema sendiri (dibaca
# build.py dengan argumen `suku` → migration/034_tema_suku.sql + aset
# public/img/tema/suku/). Format kolom sama dengan wilayah.py.
#
# Pilihan: budaya di dalam provinsi yang berbeda nyata dari tema provinsinya
# (Gayo ≠ Aceh pesisir, Karo/Nias ≠ Toba, Osing/Madura ≠ Jawa keraton, Bali
# Aga ≠ Bali dataran, Manggarai/Timor ≠ Sumba …). Palet sengaja dijauhkan dari
# tema serumpun yang sudah ada; bentuk (rupa) dibagikan scripts/rupa/build.py.

T = True
F = False

KEPALA = """-- ═══════════════════════════════════════════════════════════════════════════
-- 034_tema_suku — 24 tema undangan per SUKU / budaya daerah yang belum punya
-- tema sendiri (Gayo, Nias, Karo, Mandailing, Mentawai, Melayu Deli, Lampung
-- Pesisir, Osing, Madura, Pekalongan, Lurik, Baduy, Bali Aga, Bima, Manggarai,
-- Timor, Iban, Kutai, Makassar, Bugis Bone, Sangihe, Kei, Biak, Sentani).
-- DIBANGKITKAN scripts/provinsi/build.py . suku — jangan disunting tangan.
--
-- Aset: public/img/tema/suku/. Idempoten: ON CONFLICT DO NOTHING; ornamen
-- hanya diisi untuk tema yang belum punya ornamen. Rupa (bentuk tiap bagian)
-- diisi 035_rupa_suku.sql. WAJIB setelah 031 & 033.
-- ═══════════════════════════════════════════════════════════════════════════
"""

SUKU = [
 # ── Sumatera ─────────────────────────────────────────────────────────────
 ("kerawang-gayo", "Kerawang Gayo", "Aceh Tengah — Gayo", "Aceh",
  "Kain kerawang Gayo berwarna merah-kuning-hijau di atas hitam, kopi dataran tinggi, dan umah pitu ruang di tepi Danau Laut Tawar.",
  ["Mode Gelap", "Kerawang", "Dataran Tinggi"], "#e4b740", "#e4b740", "#120f0c", T, "aceh", "tenun", "rumoh", "rumah_panggung",
  "khas:tb-naik", "tenun", "kilau-emas", "klasik", "marcellus", "allura"),
 ("omo-sebua-nias", "Omo Sebua Nias", "Nias — Sumatera Utara", "Nias",
  "Rumah adat omo sebua berkaki kayu raksasa, batu hombo yang dilompati para pemuda, dan warna merah-kuning-hitam bangsawan Nias.",
  ["Hombo Batu", "Merah Kuning", "Rumah Panggung"], "#a8141c", "#e2b22c", "#fbf3e3", F, "batak", "gigi-balang", "bolon", "rumah_panggung",
  "khas:lr-ayun", "zoom", "kilau-emas", "gerbang", "cinzel", "alex-brush"),
 ("siwaluh-jabu-karo", "Siwaluh Jabu Karo", "Tanah Karo — Sumatera Utara", "Batak",
  "Rumah siwaluh jabu beratap ijuk bertanduk, uis gara merah-magenta, dan tudung beka buluh perempuan Karo.",
  ["Uis Gara", "Karo", "Tudung"], "#7a1f4b", "#d9a441", "#fbf1f4", F, "batak", "ulos", "bolon", None,
  "khas:full-zoom", "lipat", "kelopak", "editorial", "dm-serif", "parisienne"),
 ("gordang-mandailing", "Gordang Sambilan Mandailing", "Mandailing — Sumatera Utara", "Batak",
  "Sembilan gendang gordang sambilan bertalu di bagas godang, ulos tenun Mandailing hitam-merah-emas yang anggun.",
  ["Mode Gelap", "Gordang Sambilan", "Bagas Godang"], "#d9aa4b", "#d9aa4b", "#160d0b", T, "batak", "gorga", "bolon", None,
  "gorga-batak", "bayang", "kunang", "bingkai", "cormorant", "great-vibes"),
 ("titi-mentawai", "Titi Mentawai", "Siberut — Sumatera Barat", "Mentawai",
  "Garis tato titi Mentawai, uma kayu di tengah hutan Siberut, dan bunga hutan — undangan bernuansa alam yang tenang.",
  ["Tato Titi", "Hutan Tropis", "Alam"], "#3d5a2a", "#c6a15a", "#f4f1e6", F, "alam", "lurik", "rebung", "pemandangan",
  "taman-daun", "naik", "daun-gugur", "editorial", "lora", "allura"),
 ("istana-maimun-deli", "Istana Maimun Deli", "Medan — Melayu Deli", "Melayu",
  "Istana Maimun berkubah kuning dan hijau, awan larat Melayu Deli, dan tepak sirih — kemegahan Kesultanan Deli.",
  ["Kuning Diraja", "Awan Larat", "Kesultanan"], "#7d5600", "#d4a017", "#fffaea", F, "melayu", "awan-larat", "masjid", "istana",
  "khas:lr-geser", "anggun", "daun-sirih", "bingkai", "marcellus", "great-vibes"),
 ("tapis-saibatin", "Tapis Saibatin Pesisir", "Lampung Pesisir — Saibatin", "Lampung",
  "Kain tapis bermotif kapal dan sulam emas masyarakat adat Saibatin di pesisir Lampung — biru laut dan emas.",
  ["Tapis Kapal", "Pesisir", "Sulam Emas"], "#1b3a6b", "#d7a83c", "#f5f4ef", F, "lampung", "tapis-kapal", "siger", "phinisi",
  "khas:tb-belah", "ombak", "bintang", "gerbang", "playfair", "alex-brush"),
 # ── Jawa ─────────────────────────────────────────────────────────────────
 ("gandrung-osing", "Gandrung Osing Banyuwangi", "Banyuwangi — Osing", "Jawa",
  "Penari gandrung bermahkota omprog, batik gajah oling merah-hitam, dan rumah tikel balung masyarakat Osing.",
  ["Gajah Oling", "Gandrung", "Merah Hitam"], "#a11a1a", "#e0b03c", "#fbf3ea", F, "jawa", "patra", "gunungan", "janur",
  "khas:full-naik", "zoom", "kamboja-gugur", "klasik", "cinzel", "allura"),
 ("karapan-madura", "Karapan Madura", "Pulau Madura", "Madura",
  "Batik Madura bercorak berani merah-kuning-hijau, rumah tanean lanjang, dan kemeriahan karapan sapi.",
  ["Batik Madura", "Warna Berani", "Tanean Lanjang"], "#b3261e", "#f0b429", "#fff6e6", F, "laut", "gigi-balang", "panggung", "rumah_panggung",
  "amplop", "lipat", "kelopak", "gerbang", "dm-serif", "great-vibes"),
 ("jlamprang-pekalongan", "Jlamprang Pekalongan", "Pekalongan — Batik Pesisir", "Jawa",
  "Batik pesisir Pekalongan: jlamprang geometris dan buketan bunga berwarna toska-merah muda — ceria dan modern.",
  ["Batik Pesisir", "Buketan", "Toska"], "#0f6e6e", "#d9a066", "#f2faf8", F, "modern", "bunga-tabur", "ikat", None,
  "tirai", "mekar", "kupu", "editorial", "playfair", "parisienne"),
 ("lurik-klaten", "Lurik Klaten", "Klaten — Jawa Tengah", "Jawa",
  "Kain lurik bergaris tenunan tangan Klaten — sederhana, bersahaja, dan rapi dalam nila dan putih tulang.",
  ["Lurik", "Minimalis", "Tenun Tangan"], "#2b3a67", "#c9a96e", "#f6f4ef", F, "modern", "lurik", "gunungan", None,
  "pudar", "naik", "none", "editorial", "lora", "great-vibes"),
 ("tenun-baduy", "Tenun Baduy", "Kanekes — Banten", "Sunda",
  "Tenun Baduy hitam-biru nila, rumah panggung bambu tanpa paku, dan leuit lumbung padi — hidup selaras dengan alam.",
  ["Tenun Nila", "Bambu", "Kesederhanaan"], "#1f2a44", "#b8a27a", "#f3f1ec", F, "sunda", "bambu", "kujang", "saung",
  "mega-mendung", "pudar", "daun-gugur", "klasik", "cormorant", "parisienne"),
 # ── Bali & Nusa Tenggara ─────────────────────────────────────────────────
 ("geringsing-tenganan", "Geringsing Tenganan", "Tenganan — Bali Aga", "Bali",
  "Kain geringsing tenun ikat ganda dari desa Bali Aga Tenganan — merah soga dan hitam, dipercaya menolak bala.",
  ["Mode Gelap", "Ikat Ganda", "Bali Aga"], "#d9a65a", "#d9a65a", "#1a0f0c", T, "bali", "tenun", "candi", None,
  "lawang-bali", "bayang", "kunang", "bingkai", "marcellus", "alex-brush"),
 ("tembe-mbojo", "Tembe Nggoli Mbojo", "Bima — Nusa Tenggara Barat", "Mbojo",
  "Sarung tembe nggoli tenunan Bima, Istana Asi Mbojo, dan rimpu perempuan Mbojo — hijau zamrud dan emas.",
  ["Tembe Nggoli", "Rimpu", "Kesultanan Bima"], "#155e4b", "#d8ae4c", "#f5f8f3", F, "melayu", "songket-lepus", "masjid", "istana",
  "tenun-songket", "geser", "daun-sirih", "gerbang", "cinzel", "parisienne"),
 ("songke-manggarai", "Songke Manggarai", "Manggarai — Flores", "NTT",
  "Kain songke hitam bersulam warna-warni, rumah kerucut mbaru niang Wae Rebo, dan sawah lodok berbentuk jaring laba-laba.",
  ["Mode Gelap", "Songke", "Mbaru Niang"], "#e7b847", "#e7b847", "#121014", T, "ntt", "ikat-sumba", "ikat", "honai",
  "khas:tb-naik", "lipat", "bintang", "gerbang", "dm-serif", "allura"),
 ("ikat-timor", "Ikat Timor", "Timor — Nusa Tenggara Timur", "NTT",
  "Tenun ikat Timor merah-jingga bermotif kaif, rumah bulat ume kbubu, dan sasando — kehangatan Pulau Timor.",
  ["Tenun Ikat", "Ume Kbubu", "Jingga"], "#b03a1a", "#e2a93b", "#fbf0e8", F, "ntt", "tenun", "ikat", "honai",
  "tenun-maluku", "anggun", "kelopak", "klasik", "lora", "alex-brush"),
 # ── Kalimantan ───────────────────────────────────────────────────────────
 ("pua-kumbu-iban", "Pua Kumbu Iban", "Kapuas Hulu — Dayak Iban", "Dayak",
  "Kain pua kumbu Dayak Iban yang ditenun penuh doa, rumah panjai, dan burung kenyalang — cokelat tanah dan krem.",
  ["Pua Kumbu", "Rumah Panjai", "Kenyalang"], "#7a2414", "#d4a35a", "#f7efe6", F, "dayak", "dayak", "enggang", None,
  "khas:lr-ayun", "lipat", "bulu-enggang", "bingkai", "dm-serif", "great-vibes"),
 ("ulap-doyo-kutai", "Ulap Doyo Kutai", "Tenggarong — Kutai Kartanegara", "Melayu",
  "Kain ulap doyo dari serat daun doyo, Keraton Kutai Kartanegara, dan erau — marun gelap berbintik emas.",
  ["Mode Gelap", "Ulap Doyo", "Keraton Kutai"], "#e3b44f", "#e3b44f", "#1d0f14", T, "melayu", "kaluak-paku", "rebung", "istana",
  "galaksi", "lipat", "kilau-emas", "editorial", "cormorant", "allura"),
 # ── Sulawesi ─────────────────────────────────────────────────────────────
 ("baju-bodo-makassar", "Baju Bodo Makassar", "Gowa — Makassar", "Bugis-Makassar",
  "Baju bodo merah muda tembus pandang, Balla Lompoa Kerajaan Gowa, dan lipa sabbe sutra — ceria dan berwibawa.",
  ["Baju Bodo", "Balla Lompoa", "Sutra"], "#a3195b", "#e6b450", "#fdf1f6", F, "bugis", "balo-renni", "phinisi", "rumah_panggung",
  "khas:full-zoom", "tenun", "kelopak", "klasik", "playfair", "great-vibes"),
 ("saoraja-bone", "Saoraja Bone", "Watampone — Bugis Bone", "Bugis-Makassar",
  "Istana Saoraja Kerajaan Bone, lontara Bugis, dan sarung sutra hijau tua berbenang emas.",
  ["Mode Gelap", "Lontara", "Sutra Bugis"], "#dfb04a", "#dfb04a", "#0f1a14", T, "bugis", "balo-renni", "phinisi", None,
  "gebyok-jawa", "kosmik", "kilau-emas", "bingkai", "cinzel", "parisienne"),
 ("kofo-sangihe", "Kofo Sangihe", "Kepulauan Sangihe — Sulawesi Utara", "Sangihe",
  "Kain kofo dari serat pisang abaka, perahu pelang, dan gunung api Karangetang — ungu laut dan emas.",
  ["Kain Kofo", "Kepulauan", "Bahari"], "#3b3f8f", "#d8b25a", "#f3f3fb", F, "laut", "tenun", "phinisi", "kora",
  "layar-naik", "ombak", "kupu", "gerbang", "lora", "parisienne"),
 # ── Maluku & Papua ───────────────────────────────────────────────────────
 ("ain-ni-ain-kei", "Ain ni Ain Kei", "Kepulauan Kei — Maluku", "Maluku",
  "Pasir putih terhalus di Ngurbloat, laut toska, dan falsafah ain ni ain (kita semua bersaudara) masyarakat Kei.",
  ["Pantai", "Toska", "Persaudaraan"], "#0b6e7d", "#e2b65b", "#effafa", F, "maluku", "ombak", "tifa", "kora",
  "khas:lr-geser", "geser", "bintang", "klasik", "marcellus", "allura"),
 ("tenun-tanimbar", "Tenun Tanimbar", "Kepulauan Tanimbar — Maluku", "Maluku",
  "Tenun ikat Tanimbar bermotif ulerati, perahu batu, dan tifa — marun hangat dari ujung tenggara Maluku.",
  ["Tenun Ikat", "Tifa", "Marun"], "#6e1f2b", "#d29d4a", "#fbf2ee", F, "maluku", "ikat-sumba", "tifa", None,
  "tenun-maluku", "zoom", "cengkih-gugur", "editorial", "cinzel", "great-vibes"),
 ("wor-biak", "Wor Biak", "Biak — Papua", "Papua",
  "Lagu dan tarian wor masyarakat Biak, perahu wairon bercadik, dan laut biru dalam Teluk Cenderawasih.",
  ["Bahari", "Tari Wor", "Biru Laut"], "#0d3b66", "#f0b23c", "#f2f6fb", F, "papua", "asmat", "cenderawasih", "kora",
  "khas:tb-belah", "anggun", "burung", "gerbang", "dm-serif", "parisienne"),
 ("kulit-kayu-sentani", "Kulit Kayu Sentani", "Danau Sentani — Papua", "Papua",
  "Lukisan kulit kayu khombouw bermotif ikan dan fouw dari Danau Sentani — tanah liat, jingga, dan hitam.",
  ["Lukisan Kulit Kayu", "Danau Sentani", "Tanah Liat"], "#8c3b1c", "#d9a45a", "#f8efe5", F, "papua", "asmat", "cenderawasih", "honai",
  "pudar", "mekar", "daun-gugur", "bingkai", "lora", "alex-brush"),
]
