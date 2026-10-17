//! rupa.rs — benda dunia (dunia.rs, kelas CSS-nya) → jaring segitiga WebGL +
//! BAHAN shader. Corak (papan kayu, genteng, anyaman bilik, ubin) dilukis
//! shader dari beberapa angka → nol memori tekstur; hanya gambar SVG (pohon,
//! gapura, mempelai…) yang masuk atlas, SATU salinan per jenis (CSS 3D
//! merasterisasi tiap pohon sendiri-sendiri).
//!
//! Warna & ukuran corak disalin dari susur/_dasar.css, _sawah.css,
//! _pesisir.css — kalau rupa di sana diubah, ubah juga di sini.

use crate::dunia::{Benda, Dunia};

/// Jenis bahan (harus sama dengan `JENIS_*` di shader lib.rs).
pub const PADAT: f32 = 0.0;
pub const GAMBAR: f32 = 1.0;
pub const GARIS: f32 = 2.0;
pub const TENGAH: f32 = 3.0;
pub const TURUN: f32 = 4.0;
pub const CATUR: f32 = 5.0;
pub const LAMPU: f32 = 6.0;
pub const TIANG: f32 = 7.0;
pub const PAGAR: f32 = 8.0;
pub const SOROT: f32 = 9.0;

/// Gerak waktu (atribut obj.z; shader).
const PUTAR_Z: f32 = 1.0;
const PUTAR_Y: f32 = 2.0;
const KELIP_LAMPU: f32 = 3.0;
const KELIP_BOHLAM: f32 = 4.0;

/// Satu titik jaring (124 byte, lihat `pasang_atribut` di lib.rs).
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Titik {
    /// Titik pangkal benda di dunia.
    pub pos: [f32; 3],
    /// Simpangan dari pangkal, SEBELUM diputar `ry`.
    pub loc: [f32; 3],
    /// ry (rad), hadap kamera (0/1), gerak waktu, % gulir saat dilewati.
    pub obj: [f32; 4],
    pub uv: [f32; 2],
    /// Koordinat px di permukaan (corak) & ukuran permukaan.
    pub px: [f32; 2],
    pub ukuran: [f32; 2],
    /// Jenis bahan + 3 parameter.
    pub bahan: [f32; 4],
    /// Redup (sisi bangunan), bingkai px (− = kiri/kanan saja), ujung bulat, tempelan.
    pub fx: [f32; 4],
    /// Kotak gambar di atlas (0..1).
    pub rect: [f32; 4],
    pub c1: [u8; 4],
    pub c2: [u8; 4],
    pub c3: [u8; 4],
}
pub const LANGKAH: i32 = std::mem::size_of::<Titik>() as i32;

#[derive(Clone, Copy)]
pub struct Bahan {
    jenis: f32,
    p: [f32; 3],
    c1: [u8; 4],
    c2: [u8; 4],
    c3: [u8; 4],
    redup: f32,
    bingkai: f32,
    bulat: bool,
    tempel: bool,
    rect: [f32; 4],
    /// Gambar berulang mendatar (padi) & dibalik (penjor kanan).
    ulang: f32,
    balik: bool,
}

const fn w(hex: u32) -> [u8; 4] {
    [(hex >> 16) as u8, (hex >> 8) as u8, hex as u8, 255]
}
fn wa(hex: u32, a: f32) -> [u8; 4] {
    let mut c = w(hex);
    c[3] = (a * 255.0).round() as u8;
    c
}

impl Bahan {
    fn padat(c: [u8; 4]) -> Self {
        Bahan { jenis: PADAT, p: [0.0; 3], c1: c, c2: c, c3: c, redup: 0.0, bingkai: 0.0, bulat: false, tempel: false, rect: [0.0; 4], ulang: 1.0, balik: false }
    }
    /// Garis berulang: `per` px, garis c2 setebal `tebal`; sumbu 0 = sepanjang
    /// tinggi/panjang (gradien CSS 0deg), 1 = mendatar (90deg), 2 = diagonal.
    fn garis(c1: u32, c2: u32, per: f32, tebal: f32, sumbu: f32) -> Self {
        Bahan { jenis: GARIS, p: [per, tebal, sumbu], c2: w(c2), ..Self::padat(w(c1)) }
    }
    fn dua(jenis: f32, c1: u32, c2: u32) -> Self {
        Bahan { jenis, c2: w(c2), ..Self::padat(w(c1)) }
    }
    fn redup(mut self, r: f32) -> Self {
        self.redup = r;
        self
    }
    fn bingkai(mut self, px: f32, c: u32) -> Self {
        self.bingkai = px;
        self.c3 = w(c);
        self
    }
    fn bulat(mut self) -> Self {
        self.bulat = true;
        self
    }
}

fn ada(kelas: &str, t: &str) -> bool {
    kelas.split(' ').any(|k| k == t)
}

// ───────────────────────────── gambar (atlas) ─────────────────────────────

/// Kunci gambar atlas untuk papan bergambar + ukuran CSS satu salinannya.
pub fn gambar(b: &Benda) -> Option<(&'static str, f32, f32)> {
    let k = b.kelas;
    if !ada(k, "ss-papan") {
        return None;
    }
    let kunci = if ada(k, "ss-papan-nama") {
        if b.teks == 1 { "teks-nama" } else { "teks-salam" }
    } else if ada(k, "ss-padi") {
        let h = if ada(k, "ss-padi--muda") { b.h * 0.6 } else { b.h };
        return Some((if ada(k, "ss-padi--kuning") { "padi-kuning" } else { "padi" }, 136.0, h));
    } else if ada(k, "ss-gapura") {
        if ada(k, "ss-gapura--pesisir") { "gapura-pesisir" } else { "gapura-sawah" }
    } else if ada(k, "ss-orang") {
        "orang-orangan"
    } else {
        ["kelapa", "pisang", "bambu", "pandan", "perahu", "kursi", "penjor", "gebyok", "kincir", "bohlam", "insan", "lengkung"]
            .into_iter()
            .find(|n| k.split(' ').any(|t| t.strip_prefix("ss-") == Some(n)))?
    };
    Some((kunci, b.w, b.h))
}

// ───────────────────────────── perakit jaring ─────────────────────────────

/// Satu benda tegak di daftar gambar: rentang indeksnya & titik untuk diurut.
pub struct Potong {
    pub awal: u32,
    pub n: u32,
    pub x: f32,
    pub z: f32,
    /// % gulir saat dilewati (1000 = tak pernah) — tak usah digambar lagi.
    pub off: f32,
    /// Tembus pandang (gambar, pendar): digambar SESUDAH benda padat, jauh →
    /// dekat, tanpa menulis kedalaman — kalau menulis, tepi transparannya
    /// melubangi atap/dinding yang digambar sesudahnya.
    pub tembus: bool,
}

pub struct Jaring {
    pub titik: Vec<Titik>,
    pub indeks: Vec<u16>,
    /// Jumlah indeks bidang TANAH di awal `indeks` (digambar berurutan tanpa
    /// uji kedalaman — urutan lapis = urutan tetap, tak pernah z-fighting).
    pub tanah: u32,
    pub tegak: Vec<Potong>,
}

struct Rakit<'a> {
    t: &'a mut Vec<Titik>,
    i: &'a mut Vec<u16>,
    pos: [f32; 3],
    obj: [f32; 4],
}

impl Rakit<'_> {
    fn titik(&self, loc: [f32; 3], uv: [f32; 2], px: [f32; 2], ukuran: [f32; 2], b: &Bahan) -> Titik {
        Titik {
            pos: self.pos,
            loc,
            obj: self.obj,
            uv,
            px,
            ukuran,
            bahan: [b.jenis, b.p[0], b.p[1], b.p[2]],
            fx: [b.redup, b.bingkai, b.bulat as u8 as f32, b.tempel as u8 as f32],
            rect: b.rect,
            c1: b.c1,
            c2: b.c2,
            c3: b.c3,
        }
    }

    /// Segi empat: sudut kiri-bawah, kanan-bawah, kanan-atas, kiri-atas;
    /// `ukuran` = lebar × tinggi permukaan (px).
    fn kuad(&mut self, s: [[f32; 3]; 4], ukuran: [f32; 2], b: &Bahan) {
        let (u0, u1) = if b.balik { (b.ulang, 0.0) } else { (0.0, b.ulang) };
        let (lw, lh) = (ukuran[0], ukuran[1]);
        let n = self.t.len() as u16;
        let isi = [([u0, 1.0], [0.0, lh]), ([u1, 1.0], [lw, lh]), ([u1, 0.0], [lw, 0.0]), ([u0, 0.0], [0.0, 0.0])];
        for (k, (uv, px)) in isi.into_iter().enumerate() {
            let v = self.titik(s[k], uv, px, ukuran, b);
            self.t.push(v);
        }
        self.i.extend_from_slice(&[n, n + 1, n + 2, n, n + 2, n + 3]);
    }

    /// Segitiga dengan koordinat px tiap sudut.
    fn tiga(&mut self, s: [[f32; 3]; 3], px: [[f32; 2]; 3], ukuran: [f32; 2], b: &Bahan) {
        let n = self.t.len() as u16;
        for k in 0..3 {
            let uv = [px[k][0] / ukuran[0].max(1.0), px[k][1] / ukuran[1].max(1.0)];
            let v = self.titik(s[k], uv, px[k], ukuran, b);
            self.t.push(v);
        }
        self.i.extend_from_slice(&[n, n + 1, n + 2]);
    }
}

/// Putar titik lokal mengelilingi sumbu tegak (sama dengan CSS rotateY).
fn putar_y(p: [f32; 3], deg: f32) -> [f32; 3] {
    let (s, c) = deg.to_radians().sin_cos();
    [c * p[0] + s * p[2], p[1], -s * p[0] + c * p[2]]
}

/// Rakit seluruh dunia. `atlas(kunci)` = kotak gambar di atlas; `off[i]` =
/// % gulir saat benda i dilewati (None = tak pernah).
pub fn rakit(d: &Dunia, benda: &[Benda], off: &[Option<f32>], atlas: &dyn Fn(&str) -> [f32; 4]) -> Jaring {
    let mut j = Jaring { titik: Vec::new(), indeks: Vec::new(), tanah: 0, tegak: Vec::new() };
    // Tanah dulu, lapis bawah (y besar) lebih dulu; urutan asli bila setara.
    let mut tanah: Vec<usize> = (0..benda.len()).filter(|&i| benda[i].datar).collect();
    tanah.sort_by(|&a, &b| benda[b].y.total_cmp(&benda[a].y));
    for &i in &tanah {
        rakit_satu(d, &benda[i], None, atlas, &mut j);
    }
    j.tanah = j.indeks.len() as u32;
    for (i, b) in benda.iter().enumerate().filter(|(_, b)| !b.datar) {
        let awal = j.indeks.len() as u32;
        rakit_satu(d, b, off[i], atlas, &mut j);
        let n = j.indeks.len() as u32 - awal;
        if n > 0 {
            let padat = ["ss-kotak", "ss-limas", "ss-tiang", "ss-kubah", "ss-pagar"].iter().any(|t| ada(b.kelas, t))
                || (ada(b.kelas, "ss-prisma") && !ada(b.kelas, "ss-mercu-kaca"));
            j.tegak.push(Potong { awal, n, x: b.x, z: b.z, off: off[i].unwrap_or(1000.0), tembus: !padat });
        }
    }
    j
}

fn rakit_satu(d: &Dunia, b: &Benda, off: Option<f32>, atlas: &dyn Fn(&str) -> [f32; 4], j: &mut Jaring) {
    let k = b.kelas;
    let hadap = ada(k, "ss-hadap");
    let mut r = Rakit {
        t: &mut j.titik,
        i: &mut j.indeks,
        pos: [b.x, b.y, b.z],
        obj: [b.ry.to_radians(), hadap as u8 as f32, 0.0, off.unwrap_or(1000.0)],
    };
    let (w2, h, d2) = (b.w / 2.0, b.h, b.d / 2.0);
    let pesisir = d.slug == "pesisir";

    if b.datar {
        let bahan = bahan_tanah(k, pesisir);
        let s = [[-w2, 0.0, d2], [w2, 0.0, d2], [w2, 0.0, -d2], [-w2, 0.0, -d2]];
        r.kuad(s, [b.w, b.d], &bahan);
    } else if ada(k, "ss-kotak") {
        kotak(&mut r, k, b, pesisir);
    } else if ada(k, "ss-limas") {
        limas(&mut r, k, b);
    } else if ada(k, "ss-prisma") {
        prisma(&mut r, k, b);
    } else if let Some((kunci, _, _)) = gambar(b) {
        let mut bahan = Bahan { jenis: GAMBAR, rect: atlas(kunci), ..Bahan::padat([255; 4]) };
        let mut hh = h;
        if ada(k, "ss-padi") {
            bahan.ulang = b.w / 136.0;
            if ada(k, "ss-padi--muda") {
                hh = h * 0.6;
                bahan.c1 = [255, 255, 255, 230];
            }
        }
        bahan.balik = ada(k, "ss-penjor--balik");
        if ada(k, "ss-bohlam") {
            r.obj[2] = KELIP_BOHLAM;
        }
        if ada(k, "ss-kincir") {
            // Berputar di pusatnya: pangkal dipindah ke tengah papan.
            r.obj[2] = PUTAR_Z;
            r.pos[1] -= h / 2.0;
            let hh2 = h / 2.0;
            r.kuad([[-w2, hh2, 0.0], [w2, hh2, 0.0], [w2, -hh2, 0.0], [-w2, -hh2, 0.0]], [b.w, h], &bahan);
            return;
        }
        r.kuad([[-w2, 0.0, 0.0], [w2, 0.0, 0.0], [w2, -hh, 0.0], [-w2, -hh, 0.0]], [b.w, hh], &bahan);
    } else if ada(k, "ss-lampu") {
        // Bohlam + pendar (box-shadow 46px 16px) dalam satu segi empat.
        let g = 62.0;
        r.obj[2] = KELIP_LAMPU;
        let bahan = Bahan { jenis: LAMPU, p: [w2, h / 2.0, g], c2: w(0xf2a33a), c3: wa(0xffc460, 0.6), ..Bahan::padat(w(0xfff6d0)) };
        r.kuad([[-w2 - g, g, 0.0], [w2 + g, g, 0.0], [w2 + g, -h - g, 0.0], [-w2 - g, -h - g, 0.0]], [b.w + 2.0 * g, h + 2.0 * g], &bahan);
    } else if ada(k, "ss-sinar") {
        // Sorot mercusuar: elips berpendar, berputar mengelilingi tiang.
        r.obj[2] = PUTAR_Y;
        r.pos[1] -= h / 2.0;
        let bahan = Bahan { jenis: SOROT, c2: wa(0xffecaa, 0.5), ..Bahan::padat(wa(0xfffadc, 0.9)) };
        let hh2 = h / 2.0;
        r.kuad([[-w2, hh2, 0.0], [w2, hh2, 0.0], [w2, -hh2, 0.0], [-w2, -hh2, 0.0]], [b.w, h], &bahan);
    } else if ada(k, "ss-kubah") {
        // Atap kerucut lampu mercusuar: dua belahan segitiga, terang & teduh.
        let (kiri, kanan) = (Bahan::padat(w(0xa3322a)), Bahan::padat(w(0x82261f)));
        let uk = [b.w, h];
        r.tiga([[-w2, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, -h, 0.0]], [[0.0, h], [w2, h], [w2, 0.0]], uk, &kiri);
        r.tiga([[0.0, 0.0, 0.0], [w2, 0.0, 0.0], [0.0, -h, 0.0]], [[w2, h], [b.w, h], [w2, 0.0]], uk, &kanan);
    } else if ada(k, "ss-pagar") {
        let bahan = Bahan::dua(PAGAR, 0xc9a65a, 0xb8964e);
        r.kuad([[-w2, 0.0, 0.0], [w2, 0.0, 0.0], [w2, -h, 0.0], [-w2, -h, 0.0]], [b.w, h], &bahan);
    } else if ada(k, "ss-tiang") {
        let bahan = if ada(k, "ss-tiang--lentera") {
            Bahan::padat(w(0x3a2f2a))
        } else if ada(k, "ss-tiang--saka") {
            Bahan { p: [26.0, 0.0, 0.0], c3: w(0xc9a24b), ..Bahan::dua(TIANG, 0x4e2f18, 0x7a5030) }
        } else if ada(k, "ss-tiang--dermaga") {
            Bahan::dua(TIANG, 0x4a3220, 0x6f4c2e)
        } else {
            Bahan::dua(TIANG, 0x5c3f28, 0x8f6a46)
        };
        r.kuad([[-w2, 0.0, 0.0], [w2, 0.0, 0.0], [w2, -h, 0.0], [-w2, -h, 0.0]], [b.w, h], &bahan);
    }
}

fn bahan_tanah(k: &str, pesisir: bool) -> Bahan {
    let c = |hex| Bahan::padat(w(hex));
    if ada(k, "ss-petak4") {
        c(if ada(k, "ss-p4--b") { 0xc8b046 } else if ada(k, "ss-p4--c") { 0x7fb64e } else if ada(k, "ss-p4--d") { 0x9cc2c9 } else { 0x6faa45 })
    } else if ada(k, "ss-petak") {
        c(if ada(k, "ss-petak--kuning") { 0xc8b046 } else if ada(k, "ss-petak--air") { 0xa9c6cf } else { 0x6faa45 })
    } else if ada(k, "ss-jalan") {
        c(0xb49a6c)
    } else if ada(k, "ss-latar") {
        c(0xc9b48c)
    } else if ada(k, "ss-karpet") {
        if pesisir { c(0xf6eee2).bingkai(10.0, 0xe98aa6).bulat() } else { c(0xa3303a).bingkai(10.0, 0xd9b45a).bulat() }
    } else if ada(k, "ss-kali") {
        c(0x7fb3c6)
    } else if ada(k, "ss-jembatan") {
        Bahan::garis(0xd5b874, 0xb8964e, 22.0, 4.0, 0.0).bingkai(-6.0, 0x9a7a3a)
    } else if ada(k, "ss-pasir--karang") {
        c(0xb9ad98)
    } else if ada(k, "ss-pasir") {
        c(0xecd3a2)
    } else if ada(k, "ss-buih") {
        Bahan::padat(wa(0xffffff, 0.5))
    } else if ada(k, "ss-papan-jalan") {
        Bahan::garis(0xc79d6a, 0xa87e4c, 28.0, 4.0, 0.0).bulat()
    } else if ada(k, "ss-dermaga") {
        Bahan::garis(0x9b7048, 0x6f4c2e, 30.0, 4.0, 0.0).bingkai(-8.0, 0x5a3c22).bulat()
    } else {
        c(0x8a7a5a)
    }
}

/// Kotak dinding: depan, belakang, kiri, kanan (+ tutup bila 5 anak).
fn kotak(r: &mut Rakit, k: &str, b: &Benda, pesisir: bool) {
    let (w2, h, d2) = (b.w / 2.0, b.h, b.d / 2.0);
    let (dinding, tutup) = if ada(k, "ss-k--bilik") {
        let m = Bahan::garis(0xdcc38c, 0xc2a265, 18.0, 9.0, 2.0);
        (m, m)
    } else if ada(k, "ss-k--lantai") {
        if pesisir {
            (Bahan::dua(TENGAH, 0x6f4c2e, 0x8f6a46), Bahan::garis(0xa87e4c, 0x8a6436, 28.0, 2.0, 1.0))
        } else {
            (Bahan::dua(TENGAH, 0x7a5530, 0x9a7044), Bahan::garis(0xb48a56, 0x9c7444, 28.0, 2.0, 1.0))
        }
    } else if ada(k, "ss-k--umpak") {
        (Bahan::dua(TURUN, 0xcfc6b4, 0xa9a08c), Bahan { p: [40.0, 0.0, 0.0], ..Bahan::dua(CATUR, 0xefe8d8, 0xe2d8c2) })
    } else if ada(k, "ss-k--gebyok") {
        let m = Bahan::garis(0x8a5a32, 0x74482a, 62.0, 4.0, 1.0);
        (m, m)
    } else if ada(k, "ss-k--papan") {
        let m = Bahan::garis(0xe9f1f4, 0xc9dbe2, 33.0, 3.0, 0.0);
        (m, m)
    } else {
        let m = Bahan::padat(w(0xc9a77a));
        (m, m)
    };
    let sisi = dinding.redup(0.14);
    // Depan (+z), belakang (−z), kiri (−x), kanan (+x).
    r.kuad([[-w2, 0.0, d2], [w2, 0.0, d2], [w2, -h, d2], [-w2, -h, d2]], [b.w, h], &dinding);
    r.kuad([[w2, 0.0, -d2], [-w2, 0.0, -d2], [-w2, -h, -d2], [w2, -h, -d2]], [b.w, h], &dinding);
    r.kuad([[-w2, 0.0, -d2], [-w2, 0.0, d2], [-w2, -h, d2], [-w2, -h, -d2]], [b.d, h], &sisi);
    r.kuad([[w2, 0.0, d2], [w2, 0.0, -d2], [w2, -h, -d2], [w2, -h, d2]], [b.d, h], &sisi);
    if b.anak >= 5 {
        r.kuad([[-w2, -h, d2], [w2, -h, d2], [w2, -h, -d2], [-w2, -h, -d2]], [b.w, b.d], &tutup);
    }
    // Tempelan di dinding (pintu, jendela) — digeser sedikit ke arah kamera.
    let tempel = |c: u32| Bahan { tempel: true, ..Bahan::padat(w(c)) };
    let muka = |r: &mut Rakit, z: f32, x0: f32, x1: f32, y0: f32, y1: f32, m: &Bahan| {
        // x0..x1 dari kiri dinding depan, y dari bawah (px).
        let s = if z > 0.0 {
            [[-w2 + x0, -y0, z], [-w2 + x1, -y0, z], [-w2 + x1, -y1, z], [-w2 + x0, -y1, z]]
        } else {
            [[w2 - x0, -y0, z], [w2 - x1, -y0, z], [w2 - x1, -y1, z], [w2 - x0, -y1, z]]
        };
        r.kuad(s, [x1 - x0, y1 - y0], m);
    };
    if ada(k, "ss-k--papan") {
        // Pintu biru 110 px × 72 % + dua jendela 80 × 70 (dinding depan).
        let (pw, ph) = (110.0, 0.72 * h);
        muka(r, d2, w2 - pw / 2.0, w2 + pw / 2.0, 0.0, ph, &tempel(0x2f6aa0));
        for f in [0.18, 0.82] {
            let x0 = f * (b.w - 80.0);
            let y1 = h - 0.4 * (h - 70.0);
            muka(r, d2, x0, x0 + 80.0, y1 - 70.0, y1, &tempel(0xa8d0e6));
        }
    } else if ada(k, "ss-k--gebyok") {
        // Ambang pintu gelap 180 px × 74 % di muka depan & belakang.
        for z in [d2, -d2] {
            muka(r, z, w2 - 90.0, w2 + 90.0, 0.0, 0.74 * h, &tempel(0x4a2a14));
        }
    }
}

/// Atap limasan: trapesium depan/belakang + segitiga samping (bubungan
/// --t = lebar → pelana). 2 anak = depan/belakang saja (joglo).
fn limas(r: &mut Rakit, k: &str, b: &Benda) {
    let (w2, h, d2, t2) = (b.w / 2.0, b.h, b.d / 2.0, b.t / 2.0);
    let (muka, samping) = if ada(k, "ss-a--brunjung") {
        (Bahan::garis(0x5d4030, 0x45301f, 26.0, 4.0, 0.0), w(0x3d2a1d))
    } else if ada(k, "ss-a--sirap") {
        (Bahan::garis(0x6a4a32, 0x4f3524, 30.0, 4.0, 0.0), w(0x45301f))
    } else if ada(k, "ss-a--genteng") {
        (Bahan::garis(0xa8573a, 0x7f3f2a, 34.0, 4.0, 0.0), w(0x763a28))
    } else if ada(k, "ss-a--merah") {
        (Bahan::garis(0xb0372f, 0x8f2a24, 26.0, 4.0, 1.0), w(0x82261f))
    } else if ada(k, "ss-a--ijuk") {
        (Bahan::garis(0x3d3226, 0x514230, 11.0, 5.0, 1.0), w(0x33291d))
    } else {
        (Bahan::padat(w(0x7a4a32)), w(0x5e3824))
    };
    let sf = h.hypot(d2);
    let uk = [b.w, sf];
    let px = [[0.0, sf], [b.w, sf], [w2 + t2, 0.0], [w2 - t2, 0.0]];
    for z in [d2, -d2] {
        let x = if z > 0.0 { 1.0 } else { -1.0 };
        let s = [[-w2 * x, 0.0, z], [w2 * x, 0.0, z], [t2 * x, -h, 0.0], [-t2 * x, -h, 0.0]];
        r.tiga([s[0], s[1], s[2]], [px[0], px[1], px[2]], uk, &muka);
        r.tiga([s[0], s[2], s[3]], [px[0], px[2], px[3]], uk, &muka);
    }
    if b.anak >= 4 {
        let m = Bahan::padat(samping);
        let sa = h.hypot(w2 - t2);
        for x in [-1.0f32, 1.0] {
            let s = [[w2 * x, 0.0, -d2 * x], [w2 * x, 0.0, d2 * x], [t2 * x, -h, 0.0]];
            r.tiga(s, [[0.0, sa], [b.d, sa], [d2, 0.0]], [b.d, sa], &m);
        }
    }
}

/// Prisma segi enam (mercusuar): sisi lebar w, apotema t; sisi berbayang.
fn prisma(r: &mut Rakit, k: &str, b: &Benda) {
    let (w2, h, t) = (b.w / 2.0, b.h, b.t);
    let kaca = ada(k, "ss-mercu-kaca");
    let dasar = if kaca { Bahan::dua(TURUN, 0xffe9a0, 0xffd060) } else { Bahan::garis(0xf4f1ea, 0xd2453c, 240.0, 120.0, 0.0) };
    for (i, redup) in [0.0, 0.1, 0.22, 0.3, 0.22, 0.1].into_iter().enumerate() {
        let a = i as f32 * 60.0;
        let s = [[-w2, 0.0, t], [w2, 0.0, t], [w2, -h, t], [-w2, -h, t]].map(|p| putar_y(p, a));
        r.kuad(s, [b.w, h], &dasar.redup(redup));
    }
    if kaca {
        // Pendar lampu (box-shadow 30px 8px) — papan menghadap kamera.
        r.obj[1] = 1.0;
        let m = Bahan { jenis: SOROT, c2: wa(0xffdc78, 0.45), ..Bahan::padat(wa(0xffecaa, 0.8)) };
        let (gw, gh, cy) = (170.0, 150.0, -h / 2.0);
        r.kuad([[-gw, cy + gh, 0.0], [gw, cy + gh, 0.0], [gw, cy - gh, 0.0], [-gw, cy - gh, 0.0]], [2.0 * gw, 2.0 * gh], &m);
    }
}
