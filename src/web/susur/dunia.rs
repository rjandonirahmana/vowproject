//! web/susur/dunia.rs — DATA & MATEMATIKA dunia susur 3D, tanpa dependensi:
//! benda, jalur Catmull-Rom, pose kamera per % gulir, `--off` (kapan benda
//! dilewati), dan perakit dua dunia (desa sawah, pesisir). Satu sumber
//! kebenaran bagi DUA perender: CSS 3D (web/susur.rs — HTML cadangan &
//! keyframes) dan WebGL2 (crate susur-gl/, menyertakan berkas ini lewat
//! `#[path]`). JANGAN memakai `crate::`/`super::` di sini.

/// Satu benda di dunia. Titik (x, z) = kaki benda di tanah (px dunia; z
/// negatif = menjauh dari gapura), `y` negatif = terangkat (mis. atap).
#[derive(Clone, Debug, PartialEq)]
pub struct Benda {
    /// Kelas CSS (primitif + varian), mis. "ss-kotak ss-k--bilik".
    pub kelas: &'static str,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    /// Putaran badan benda (derajat, sumbu tegak).
    pub ry: f32,
    pub w: f32,
    pub h: f32,
    pub d: f32,
    /// Panjang bubungan atap limas / apotema prisma.
    pub t: f32,
    /// Jumlah anak `<i>` (dinding, bidang atap, sisi prisma…).
    pub anak: u8,
    /// Tulisan di papan: 0 = tanpa, 1 = nama mempelai, 2 = salam datang.
    pub teks: u8,
    /// Bidang tanah (rebah) — dilewati saat SELURUHNYA di luar layar bawah.
    pub datar: bool,
}

impl Benda {
    fn new(kelas: &'static str, x: f32, z: f32) -> Self {
        Benda { kelas, x, y: 0.0, z, ry: 0.0, w: 0.0, h: 0.0, d: 0.0, t: 0.0, anak: 0, teks: 0, datar: false }
    }
    fn y(mut self, y: f32) -> Self {
        self.y = y;
        self
    }
    fn ry(mut self, ry: f32) -> Self {
        self.ry = ry;
        self
    }
    fn teks(mut self, teks: u8) -> Self {
        self.teks = teks;
        self
    }
}

/// Papan tegak bergambar (pohon, gapura, perahu…), kaki di (x, z).
fn papan(kelas: &'static str, x: f32, z: f32, w: f32, h: f32) -> Benda {
    Benda { w, h, ..Benda::new(kelas, x, z) }
}
/// Bidang rebah di tanah (petak sawah, jalan, pasir…), pusat di (x, z).
fn datar(kelas: &'static str, x: f32, z: f32, w: f32, d: f32) -> Benda {
    Benda { w, d, datar: true, ..Benda::new(kelas, x, z) }
}
/// Kotak dinding (4 sisi; `atas` = tambah tutup) w × d, tinggi h.
fn kotak(kelas: &'static str, x: f32, z: f32, w: f32, d: f32, h: f32, atas: bool) -> Benda {
    Benda { w, d, h, anak: if atas { 5 } else { 4 }, ..Benda::new(kelas, x, z) }
}
/// Atap limasan (alas w × d, tinggi h, bubungan t; t = w → atap pelana).
fn limas(kelas: &'static str, x: f32, z: f32, w: f32, d: f32, h: f32, t: f32) -> Benda {
    Benda { w, d, h, t, anak: 4, ..Benda::new(kelas, x, z) }
}
/// Tiang: papan tipis yang selalu menghadap kamera (dua bidang bersilang
/// saling memotong → dipecah ulang browser tiap frame).
fn tiang(kelas: &'static str, x: f32, z: f32, w: f32, h: f32) -> Benda {
    papan(kelas, x, z, w, h)
}

/// Satu dunia susur.
pub struct Dunia {
    pub slug: &'static str,
    /// Keluarga sepasang mempelai di pelaminan (lapisan /panggung/insan-*.svg).
    pub insan: &'static str,
    /// Nama enam singgah (id, en) — label kartu & peta perjalanan.
    pub singgah: [(&'static str, &'static str); 6],
    /// Titik kendali jalan setapak (x, z); kurva Catmull-Rom melewatinya.
    jalur: &'static [(f32, f32)],
    /// Indeks titik kendali tempat kamera BERHENTI di tiap singgah.
    pos: [usize; 6],
    /// Arah menoleh (derajat, + = kanan) saat berhenti di tiap singgah.
    lihat: [f32; 6],
}

pub const DUNIA: &[Dunia] = &[
    Dunia {
        slug: "sawah",
        insan: "insan-adat",
        singgah: [
            ("Gapura Desa", "Village Gate"),
            ("Pematang Sawah", "Rice-field Dyke"),
            ("Kincir & Kali", "Waterwheel Creek"),
            ("Lumbung Padi", "Rice Barn"),
            ("Tarub Janur", "Palm-leaf Arch"),
            ("Pendopo Joglo", "Joglo Pavilion"),
        ],
        jalur: &[(0.0, 0.0), (50.0, -950.0), (-60.0, -1900.0), (40.0, -2700.0), (120.0, -3250.0), (100.0, -4050.0), (-80.0, -4700.0), (0.0, -5600.0), (0.0, -6900.0), (0.0, -7400.0)],
        pos: [0, 2, 4, 6, 7, 8],
        lihat: [0.0, 26.0, -40.0, -38.0, 0.0, 0.0],
    },
    Dunia {
        slug: "pesisir",
        insan: "insan-modern",
        singgah: [
            ("Gapura Nelayan", "Fishermen's Gate"),
            ("Pantai Nyiur", "Palm Beach"),
            ("Dermaga Kayu", "Wooden Pier"),
            ("Rumah Panggung", "Stilt House"),
            ("Lorong Tamu", "Guest Aisle"),
            ("Pelaminan Senja", "Sunset Altar"),
        ],
        jalur: &[(0.0, 0.0), (40.0, -900.0), (-40.0, -1700.0), (0.0, -2400.0), (0.0, -2900.0), (0.0, -4100.0), (0.0, -4800.0), (0.0, -5300.0), (0.0, -6050.0), (0.0, -6500.0)],
        pos: [0, 2, 4, 5, 7, 8],
        lihat: [0.0, 26.0, -30.0, 44.0, 0.0, 0.0],
    },
];

/// Dunia susur untuk tema; "" = bukan tema susur.
pub fn pilih(slug: &str) -> &'static str {
    match slug {
        "desa-sawah-3d" => "sawah",
        "pesisir-nyiur-3d" => "pesisir",
        _ => "",
    }
}

pub fn dunia(slug: &str) -> Option<&'static Dunia> {
    DUNIA.iter().find(|d| d.slug == slug)
}

// ───────────────────────────── jalur & kamera ─────────────────────────────

/// Sampel jalur setapak (polilinie rapat) + panjang lintasan kumulatif.
pub struct Jalur {
    titik: Vec<(f32, f32)>,
    s: Vec<f32>,
    /// Panjang lintasan di tiap titik kendali.
    kendali: Vec<f32>,
}

const SAMPEL: usize = 24;

impl Jalur {
    pub fn new(k: &[(f32, f32)]) -> Self {
        let mut titik = Vec::new();
        for i in 0..k.len() - 1 {
            let p0 = k[i.saturating_sub(1)];
            let (p1, p2) = (k[i], k[i + 1]);
            let p3 = k[(i + 2).min(k.len() - 1)];
            for n in 0..SAMPEL {
                let t = n as f32 / SAMPEL as f32;
                let (t2, t3) = (t * t, t * t * t);
                let c = |a: f32, b: f32, c: f32, d: f32| 0.5 * (2.0 * b + (-a + c) * t + (2.0 * a - 5.0 * b + 4.0 * c - d) * t2 + (-a + 3.0 * b - 3.0 * c + d) * t3);
                titik.push((c(p0.0, p1.0, p2.0, p3.0), c(p0.1, p1.1, p2.1, p3.1)));
            }
        }
        titik.push(*k.last().unwrap());
        let mut s = vec![0.0];
        for w in titik.windows(2) {
            s.push(s.last().unwrap() + ((w[1].0 - w[0].0).powi(2) + (w[1].1 - w[0].1).powi(2)).sqrt());
        }
        let kendali = (0..k.len()).map(|i| s[i * SAMPEL]).collect();
        Jalur { titik, s, kendali }
    }

    pub fn panjang(&self) -> f32 {
        *self.s.last().unwrap()
    }

    /// Titik di panjang lintasan `s`.
    pub fn di(&self, s: f32) -> (f32, f32) {
        let s = s.clamp(0.0, self.panjang());
        let i = self.s.partition_point(|&v| v < s).clamp(1, self.s.len() - 1);
        let (s0, s1) = (self.s[i - 1], self.s[i]);
        let u = if s1 > s0 { (s - s0) / (s1 - s0) } else { 0.0 };
        let (a, b) = (self.titik[i - 1], self.titik[i]);
        (a.0 + (b.0 - a.0) * u, a.1 + (b.1 - a.1) * u)
    }

    /// Arah jalan (derajat; 0 = lurus menjauh/−z, + = berbelok kanan) di `s`,
    /// dihaluskan dengan melihat sedikit ke depan.
    pub fn arah(&self, s: f32) -> f32 {
        let a = self.di(s - 40.0);
        let b = self.di(s + 160.0);
        (b.0 - a.0).atan2(-(b.1 - a.1)).to_degrees()
    }

    pub fn titik(&self) -> &[(f32, f32)] {
        &self.titik
    }
}

/// Pose kamera pada satu persen gulir.
#[derive(Clone, Copy, Debug)]
pub struct Pose {
    pub x: f32,
    pub z: f32,
    /// Arah pandang (derajat, + = kanan).
    pub a: f32,
    /// Langkah: naik-turun (px) & goyang (derajat).
    pub angguk: f32,
    pub goyang: f32,
}

fn halus(u: f32) -> f32 {
    let u = u.clamp(0.0, 1.0);
    u * u * u * (u * (u * 6.0 - 15.0) + 10.0)
}

/// Pusat singgah i (% gulir) — DIAM di [pusat − 5, pusat + 5]; teks singgah
/// memakai rentang yang sama (pages/undangan.rs JL_RENTANG).
pub const DIAM: f32 = 5.0;
pub fn pusat(i: usize) -> f32 {
    i as f32 * 20.0
}

impl Dunia {
    pub fn jalur(&self) -> Jalur {
        Jalur::new(self.jalur)
    }

    /// Pose kamera pada `p` % gulir prolog.
    pub fn pose(&self, j: &Jalur, p: f32) -> Pose {
        let s_pos = |i: usize| j.kendali[self.pos[i]];
        let diam = |i: usize| {
            let s = s_pos(i);
            let (x, z) = j.di(s);
            Pose { x, z, a: j.arah(s) + self.lihat[i], angguk: 0.0, goyang: 0.0 }
        };
        let p = p.clamp(0.0, 100.0);
        // Singgah tempat p sedang diam, atau ruas jalan di antara dua singgah.
        for i in 0..6 {
            if (p - pusat(i)).abs() <= DIAM {
                return diam(i);
            }
        }
        let i = ((p - DIAM) / 20.0).floor().clamp(0.0, 4.0) as usize;
        let u = (p - pusat(i) - DIAM) / (20.0 - 2.0 * DIAM);
        let e = halus(u);
        let (s0, s1) = (s_pos(i), s_pos(i + 1));
        let s = s0 + (s1 - s0) * e;
        let (x, z) = j.di(s);
        // Menoleh ke singgah: memudar saat berangkat, menguat saat tiba.
        let a = j.arah(s) + self.lihat[i] * (1.0 - halus(u / 0.4)) + self.lihat[i + 1] * halus((u - 0.6) / 0.4);
        // Langkah ±110 px; selubung sin(πu) → mulai & berhenti tanpa sentak.
        let langkah = ((s1 - s0) / 110.0).round().max(2.0);
        let fase = std::f32::consts::PI * langkah * e;
        let selubung = (std::f32::consts::PI * u).sin();
        Pose { x, z, a, angguk: 7.0 * fase.sin().abs() * selubung, goyang: 0.45 * fase.sin() * selubung }
    }

    /// Titik-titik % gulir untuk keyframes: tepi tiap diam + rapat di ruas jalan.
    pub fn kunci(&self) -> Vec<f32> {
        let mut out = Vec::new();
        for i in 0..6 {
            let (a, b) = ((pusat(i) - DIAM).max(0.0), (pusat(i) + DIAM).min(100.0));
            out.push(a);
            if b > a {
                out.push(b);
            }
            if i < 5 {
                let mut p = b + 0.4;
                while p < pusat(i + 1) - DIAM - 0.01 {
                    out.push(p);
                    p += 0.4;
                }
            }
        }
        out
    }

    /// Semua benda dunia ini.
    pub fn benda(&self, j: &Jalur) -> Vec<Benda> {
        match self.slug {
            "sawah" => sawah(j),
            _ => pesisir(j),
        }
    }
}

/// Kedalaman titik (x, z) di depan mata untuk pose `k` (px; negatif = di belakang).
pub fn dalam(k: &Pose, x: f32, z: f32) -> f32 {
    let (rx, rz) = (x - k.x, z - k.z);
    let a = k.a.to_radians();
    -(-rx * a.sin() + rz * a.cos())
}

/// Tabel pose per 0,1 % gulir (dipakai menghitung `--off`).
pub fn tabel(d: &Dunia, j: &Jalur) -> Vec<Pose> {
    (0..=1000).map(|i| d.pose(j, i as f32 / 10.0)).collect()
}

/// % gulir saat benda dilewati kamera (None = tak pernah). Benda tegak:
/// begitu kakinya < ±150 px di depan mata (sudah di luar layar karena di
/// samping/atas jalan). Bidang tanah: begitu sudut TERJAUHNYA < 300 px —
/// tanah sedekat itu sudah di bawah tepi layar.
pub fn off(tabel: &[Pose], b: &Benda) -> Option<f32> {
    let sudut: Vec<(f32, f32)> = if b.datar {
        let (s, c) = b.ry.to_radians().sin_cos();
        [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
            .iter()
            .map(|(u, v)| {
                let (lx, lz) = (u * b.w / 2.0, v * b.d / 2.0);
                (b.x + lx * c + lz * s, b.z - lx * s + lz * c)
            })
            .collect()
    } else {
        vec![(b.x, b.z)]
    };
    let batas = if b.datar { 300.0 } else { 150.0 + b.w.max(b.d) * 0.15 };
    tabel
        .iter()
        .position(|k| sudut.iter().map(|&(x, z)| dalam(k, x, z)).fold(f32::MIN, f32::max) < batas)
        .map(|i| i as f32 / 10.0)
}

// ───────────────────────────── perakit dunia ─────────────────────────────

/// Acak tetap (tanpa crate rand; sama di server & WASM).
struct Acak(u32);
impl Acak {
    fn f(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 % 10_000) as f32 / 10_000.0
    }
}

/// x jalan pada kedalaman z (jalur selalu maju ke −z).
fn x_jalan(j: &Jalur, z: f32) -> f32 {
    let t = j.titik();
    let i = t.iter().position(|p| p.1 <= z).unwrap_or(t.len() - 1).max(1);
    let (a, b) = (t[i - 1], t[i]);
    if (a.1 - b.1).abs() < 0.01 { b.0 } else { a.0 + (b.0 - a.0) * (a.1 - z) / (a.1 - b.1) }
}

/// Bidang jalan setapak sepanjang lintasan (ruas ±380 px, tumpang tindih).
fn setapak(j: &Jalur, kelas: &'static str, lebar: f32, dari: f32, sampai: f32) -> Vec<Benda> {
    let mut out = Vec::new();
    let mut s = dari;
    while s < sampai {
        let (x, z) = j.di(s + 190.0);
        out.push(Benda { ry: -j.arah(s + 190.0), ..datar(kelas, x, z, lebar, 450.0) }.y(1.0));
        s += 380.0;
    }
    out
}

/// Rakitan rumah: kotak dinding + atap (limasan/pelana) + tiang kolong
/// (+ lantai panggung `alas` = (lebar, dalam, tebal) di atas tiang). Tak ada
/// bidang yang saling menembus (tiang berhenti di bawah lantai, atap duduk di
/// puncak dinding) — bidang berpotongan dipecah ulang browser tiap frame.
#[allow(clippy::too_many_arguments)]
fn rumah(out: &mut Vec<Benda>, dinding: &'static str, atap: &'static str, x: f32, z: f32, ry: f32, w: f32, d: f32, h: f32, kolong: f32, atap_h: f32, bubungan: f32, emper: f32, alas: Option<(f32, f32, f32)>) {
    let tebal = alas.map_or(0.0, |a| a.2);
    if let Some((aw, ad, at)) = alas {
        out.push(kotak("ss-kotak ss-k--lantai", x, z, aw, ad, at, true).y(-(kolong - at)).ry(ry));
    }
    if kolong > 0.0 {
        let (s, c) = ry.to_radians().sin_cos();
        for (u, v) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
            let (lx, lz) = (u * (w / 2.0 - 16.0), v * (d / 2.0 - 16.0));
            out.push(tiang("ss-papan ss-hadap ss-tiang", x + lx * c + lz * s, z - lx * s + lz * c, 22.0, kolong - tebal));
        }
    }
    if h > 0.0 {
        out.push(kotak(dinding, x, z, w, d, h, false).y(-kolong).ry(ry));
    }
    out.push(limas(atap, x, z, w + emper, d + emper, atap_h, bubungan).y(-kolong - h).ry(ry));
}

fn pohon(out: &mut Vec<Benda>, kelas: &'static str, x: f32, z: f32, w: f32, h: f32) {
    out.push(papan(kelas, x, z, w, h));
}

fn sawah(j: &Jalur) -> Vec<Benda> {
    let mut out = Vec::new();
    let mut r = Acak(0x5a3f_1c27);
    // Petak sawah (bidang polos → murah) + rumpun padi tegak di petak dekat jalan.
    // Petak berderet mengikuti kelok jalan: petak pertama tepat di balik
    // pematang (petak polos + rumpun padi), petak luar digabung 2×2 jadi satu
    // bidang berlukis (empat warna + pematang) — separuh jumlah lapisan.
    let (pw, pd) = (460.0, 400.0);
    let bebas = |x: f32, z: f32, r: f32| {
        (z + 3850.0).abs() >= 150.0 + r
            && (x + 760.0).hypot(z + 5500.0) > 480.0
            && (x - 540.0).hypot(z + 2750.0) > 400.0
            && !(z < -5300.0 && x.abs() < 820.0)
    };
    let mut z = -470.0;
    let mut baris = 0;
    while z > -6300.0 {
        let xj = x_jalan(j, z);
        for sisi in [-1.0f32, 1.0] {
            let x = xj + sisi * (120.0 + pw / 2.0);
            if bebas(x, z, pd / 2.0) {
                let v = r.f();
                let (kelas, padi) = if v < 0.16 {
                    ("ss-datar ss-petak ss-petak--air", "ss-papan ss-padi ss-padi--muda")
                } else if v < 0.4 {
                    ("ss-datar ss-petak ss-petak--kuning", "ss-papan ss-padi ss-padi--kuning")
                } else {
                    ("ss-datar ss-petak", "ss-papan ss-padi")
                };
                out.push(datar(kelas, x, z, pw - 30.0, pd - 30.0).y(3.0));
                out.push(papan(padi, x, z + pd / 2.0 - 26.0, pw - 40.0, 58.0));
            }
            if baris % 2 == 0 {
                let zm = z - pd / 2.0;
                let xm = x_jalan(j, zm) + sisi * (120.0 + 2.0 * pw);
                if bebas(xm, zm, pd) {
                    const RAGAM: [&str; 4] = ["ss-datar ss-petak4", "ss-datar ss-petak4 ss-p4--b", "ss-datar ss-petak4 ss-p4--c", "ss-datar ss-petak4 ss-p4--d"];
                    out.push(datar(RAGAM[(r.f() * 4.0) as usize % 4], xm, zm, 2.0 * pw - 30.0, 2.0 * pd - 30.0).y(3.0));
                }
            }
        }
        z -= pd;
        baris += 1;
    }
    // Jalan setapak (pematang lebar) dari gapura ke tarub, lalu karpet ke pendopo.
    out.extend(setapak(j, "ss-datar ss-jalan", 170.0, 0.0, j.kendali[7] + 600.0));
    out.push(datar("ss-datar ss-latar", 0.0, -7700.0, 2600.0, 2600.0).y(3.0));
    out.push(datar("ss-datar ss-karpet", 0.0, -6900.0, 190.0, 1100.0).y(1.0));
    // Kali melintang + jembatan bambu.
    out.push(Benda { ry: 6.0, ..datar("ss-datar ss-kali", 0.0, -3850.0, 4800.0, 300.0) }.y(2.0));
    let (bx, bz) = j.di(j.kendali[5] - 200.0);
    let ba = -j.arah(j.kendali[5] - 200.0);
    out.push(Benda { ry: ba, ..datar("ss-datar ss-jembatan", bx, bz, 200.0, 520.0) }.y(-24.0));
    for sisi in [-1.0f32, 1.0] {
        let (s, c) = ba.to_radians().sin_cos();
        out.push(papan("ss-papan ss-pagar", bx + sisi * 100.0 * c, bz - sisi * 100.0 * s, 520.0, 110.0).y(-24.0).ry(ba + 90.0));
    }
    // Kincir air di tepi kali.
    out.push(papan("ss-papan ss-kincir", -600.0, -3900.0, 420.0, 420.0).y(-2.0).ry(40.0));
    out.push(tiang("ss-papan ss-hadap ss-tiang", -670.0, -3850.0, 18.0, 260.0));
    out.push(tiang("ss-papan ss-hadap ss-tiang", -530.0, -3950.0, 18.0, 260.0));
    // Gapura bambu + penjor (singgah 0) — papan nama "Sugeng Rawuh".
    out.push(papan("ss-papan ss-gapura", 40.0, -800.0, 680.0, 770.0));
    out.push(papan("ss-papan ss-papan-nama", 40.0, -798.0, 330.0, 66.0).y(-388.0).teks(2));
    out.push(papan("ss-papan ss-penjor ss-hadap", -360.0, -760.0, 230.0, 700.0));
    out.push(papan("ss-papan ss-penjor ss-penjor--balik ss-hadap", 440.0, -760.0, 230.0, 700.0));
    // Saung (gubuk sawah) + orang-orangan (singgah 1).
    let (sx, sz, sr) = (540.0, -2750.0, -30.0);
    rumah(&mut out, "", "ss-limas ss-a--ijuk", sx, sz, sr, 340.0, 280.0, 0.0, 300.0, 170.0, 340.0 + 120.0, 120.0, None);
    out.push(kotak("ss-kotak ss-k--lantai", sx, sz, 280.0, 220.0, 22.0, true).y(-70.0).ry(sr));
    out.push(papan("ss-papan ss-orang ss-hadap", -360.0, -2380.0, 190.0, 330.0));
    // Lumbung padi di atas kolong + pisang & kelapa (singgah 3).
    rumah(&mut out, "ss-kotak ss-k--bilik", "ss-limas ss-a--ijuk", -760.0, -5500.0, 30.0, 300.0, 260.0, 190.0, 170.0, 330.0, 300.0 + 120.0, 120.0, None);
    pohon(&mut out, "ss-papan ss-pisang ss-hadap", -420.0, -5700.0, 330.0, 480.0);
    pohon(&mut out, "ss-papan ss-pisang ss-hadap", -980.0, -5000.0, 360.0, 520.0);
    // Tarub janur (singgah 4) — papan nama mempelai.
    out.push(papan("ss-papan ss-gapura ss-gapura--tarub", 0.0, -6500.0, 700.0, 790.0));
    out.push(papan("ss-papan ss-papan-nama", 0.0, -6498.0, 340.0, 70.0).y(-398.0).teks(1));
    out.push(papan("ss-papan ss-penjor ss-hadap", -420.0, -6460.0, 230.0, 700.0));
    out.push(papan("ss-papan ss-penjor ss-penjor--balik ss-hadap", 420.0, -6460.0, 230.0, 700.0));
    // Pendopo joglo: umpak + saka guru + atap penanggap & brunjung; pelaminan.
    let pz = -7650.0;
    out.push(kotak("ss-kotak ss-k--umpak", 0.0, pz, 1100.0, 1000.0, 46.0, true));
    for (x, z, h) in [(-300.0, -280.0, 430.0), (300.0, -280.0, 430.0), (-300.0, 280.0, 430.0), (300.0, 280.0, 430.0)] {
        out.push(tiang("ss-papan ss-hadap ss-tiang ss-tiang--saka", x, pz + z, 30.0, h).y(-46.0));
    }
    for (x, z) in [(-500.0, -450.0), (500.0, -450.0), (-500.0, 450.0), (500.0, 450.0), (0.0, 450.0 + 1.0)] {
        if x != 0.0 {
            out.push(tiang("ss-papan ss-hadap ss-tiang ss-tiang--saka", x, pz + z, 24.0, 430.0).y(-46.0));
        }
    }
    // Atap joglo dari depan/belakang saja (trapesium): jurai samping limasan
    // memotong bidang muka → dipecah ulang tiap frame; kamera selalu dari depan.
    out.push(Benda { anak: 2, ..limas("ss-limas ss-a--sirap", 0.0, pz, 1320.0, 1220.0, 230.0, 640.0) }.y(-476.0));
    out.push(Benda { anak: 2, ..limas("ss-limas ss-a--sirap ss-a--brunjung", 0.0, pz, 700.0, 640.0, 420.0, 150.0) }.y(-704.0));
    out.push(papan("ss-papan ss-gebyok", 0.0, pz - 300.0, 820.0, 520.0).y(-46.0));
    out.push(papan("ss-papan ss-insan", 0.0, pz - 120.0, 330.0, 330.0).y(-46.0));
    for (x, z) in [(-200.0, -150.0), (200.0, -150.0), (0.0, 120.0)] {
        out.push(papan("ss-papan ss-lampu", x, pz + z, 54.0, 64.0).y(-440.0));
    }
    // Dalem (rumah induk) di belakang pendopo.
    rumah(&mut out, "ss-kotak ss-k--gebyok", "ss-limas ss-a--genteng", 0.0, -9150.0, 0.0, 1400.0, 800.0, 340.0, 0.0, 400.0, 1400.0 + 160.0, 160.0, None);
    // Pepohonan: kelapa, bambu, pisang di sela sawah.
    for (x, z, s) in [
        (-760.0, -1250.0, 1.0), (700.0, -1550.0, 1.1), (-1050.0, -2050.0, 0.9), (980.0, -2950.0, 1.0), (-900.0, -3300.0, 1.1),
        (860.0, -4300.0, 0.95), (-1250.0, -4500.0, 1.0), (760.0, -5400.0, 1.1), (-800.0, -6300.0, 1.0), (820.0, -6600.0, 0.9),
        (-960.0, -7300.0, 1.15), (980.0, -7600.0, 1.05), (-1150.0, -8600.0, 1.0), (1200.0, -8800.0, 1.1), (-1600.0, -6900.0, 0.9),
    ] {
        pohon(&mut out, "ss-papan ss-kelapa ss-hadap", x, z, 520.0 * s, 1040.0 * s);
    }
    for (x, z) in [(-1150.0, -1500.0), (1150.0, -3700.0), (700.0, -4750.0), (-1500.0, -8000.0)] {
        pohon(&mut out, "ss-papan ss-bambu ss-hadap", x, z, 460.0, 900.0);
    }
    out
}

fn pesisir(j: &Jalur) -> Vec<Benda> {
    let mut out = Vec::new();
    // Dua daratan pasir (bidang polos persegi — ubin warna tunggal tak butuh
    // tekstur GPU) di atas laut (latar), pulau karang mercusuar = dua bujur
    // sangkar bersilang 45° (segi delapan).
    out.push(datar("ss-datar ss-pasir", 0.0, -1250.0, 3800.0, 3000.0).y(3.0));
    out.push(datar("ss-datar ss-pasir", 0.0, -6300.0, 3400.0, 3200.0).y(3.0));
    for ry in [0.0, 45.0] {
        out.push(datar("ss-datar ss-pasir ss-pasir--karang", -1250.0, -4500.0, 620.0, 620.0).y(3.0).ry(ry));
    }
    for (z, w) in [(-2750.0, 3800.0), (-4700.0, 3400.0)] {
        out.push(datar("ss-datar ss-buih", 0.0, z, w, 50.0).y(2.0));
    }
    // Jalan papan di pasir + dermaga kayu di atas laut.
    out.extend(setapak(j, "ss-datar ss-papan-jalan", 150.0, 0.0, j.kendali[3]));
    out.extend(setapak(j, "ss-datar ss-dermaga", 230.0, j.kendali[3], j.kendali[6]).into_iter().map(|b| b.y(-30.0)));
    out.extend(setapak(j, "ss-datar ss-karpet", 170.0, j.kendali[7] - 300.0, j.kendali[9]).into_iter().map(|b| b.y(1.0)));
    let mut z = -2700.0;
    while z > -4700.0 {
        for x in [-125.0, 125.0] {
            out.push(tiang("ss-papan ss-hadap ss-tiang ss-tiang--dermaga", x, z, 20.0, 140.0).y(60.0));
        }
        z -= 380.0;
    }
    for (x, z) in [(-135.0, -3560.0), (135.0, -3600.0)] {
        out.push(tiang("ss-papan ss-hadap ss-tiang ss-tiang--lentera", x, z, 14.0, 260.0).y(-30.0));
        out.push(papan("ss-papan ss-lampu", x, z, 46.0, 56.0).y(-300.0));
    }
    // Gapura nelayan (singgah 0) — papan "Selamat Datang".
    out.push(papan("ss-papan ss-gapura ss-gapura--pesisir", 30.0, -800.0, 680.0, 770.0));
    out.push(papan("ss-papan ss-papan-nama", 30.0, -797.0, 290.0, 56.0).y(-282.0).teks(2));
    // Perahu jukung & semak pandan (singgah 1).
    out.push(papan("ss-papan ss-perahu", 760.0, -2700.0, 860.0, 410.0).ry(-30.0));
    out.push(papan("ss-papan ss-perahu", -900.0, -2500.0, 700.0, 330.0).ry(30.0));
    for (x, z) in [(-420.0, -1400.0), (330.0, -1100.0), (-260.0, -2400.0), (900.0, -1900.0)] {
        out.push(papan("ss-papan ss-pandan ss-hadap", x, z, 340.0, 270.0));
    }
    // Mercusuar di pulau karang (singgah 2): prisma segi enam + kubah lampu.
    let (mx, mz) = (-1250.0, -4500.0);
    out.push(Benda { w: 128.0, h: 820.0, t: 110.0, anak: 6, ..Benda::new("ss-prisma ss-mercu", mx, mz) });
    out.push(Benda { w: 92.0, h: 140.0, t: 80.0, anak: 6, ..Benda::new("ss-prisma ss-mercu-kaca", mx, mz) }.y(-820.0));
    out.push(papan("ss-papan ss-hadap ss-kubah", mx, mz, 250.0, 120.0).y(-960.0));
    out.push(papan("ss-papan ss-sinar", mx, mz, 900.0, 160.0).y(-810.0));
    out.push(papan("ss-papan ss-pandan ss-hadap", mx + 260.0, mz + 160.0, 300.0, 240.0));
    // Rumah panggung di atas air (singgah 3).
    rumah(&mut out, "ss-kotak ss-k--papan", "ss-limas ss-a--merah", 950.0, -4850.0, -35.0, 520.0, 400.0, 270.0, 300.0, 230.0, 640.0, 120.0, Some((640.0, 520.0, 24.0)));
    // Lorong tamu: kursi berbaris + tiang lampu gantung (singgah 4–5).
    for row in 0..4 {
        let z = -5650.0 - row as f32 * 210.0;
        for x in [-470.0, -290.0, 290.0, 470.0] {
            out.push(papan("ss-papan ss-kursi", x, z, 140.0, 216.0));
        }
    }
    for z in [-5500.0, -6150.0] {
        for x in [-640.0, 640.0] {
            out.push(tiang("ss-papan ss-hadap ss-tiang ss-tiang--lentera", x, z, 14.0, 520.0));
        }
        out.push(papan("ss-papan ss-bohlam", 0.0, z, 1280.0, 120.0).y(-420.0));
    }
    // Lengkung bunga pelaminan + mempelai (singgah 5).
    out.push(papan("ss-papan ss-lengkung", 0.0, -7050.0, 700.0, 860.0));
    out.push(papan("ss-papan ss-insan", 0.0, -6960.0, 280.0, 280.0));
    // Nyiur di tepi pantai.
    for (x, z, s) in [
        (-700.0, -350.0, 1.0), (760.0, -600.0, 1.1), (-980.0, -1200.0, 1.05), (1050.0, -1500.0, 0.95), (-620.0, -2050.0, 1.1),
        (1350.0, -2350.0, 1.0), (-1500.0, -1700.0, 0.9), (-760.0, -5200.0, 1.05), (800.0, -5500.0, 1.1), (-1100.0, -6400.0, 1.0),
        (1100.0, -6700.0, 0.95), (-700.0, -7300.0, 1.15), (760.0, -7400.0, 1.0), (-1450.0, -4350.0, 0.85),
    ] {
        pohon(&mut out, "ss-papan ss-kelapa ss-hadap", x, z, 520.0 * s, 1040.0 * s);
    }
    out
}
