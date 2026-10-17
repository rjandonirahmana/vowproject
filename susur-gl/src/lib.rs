//! susur-gl — perender WebGL2 untuk prolog SUSUR 3D (web/susur.rs).
//!
//! Kenapa bukan CSS 3D: di bawah perspektif browser merasterisasi TIAP bidang
//! sebagai lapisan sendiri pada ukuran CSS × DPR (±150 lapisan, 45–100 MB
//! memori GPU, naik-turun mengikuti gulir). Di sini seluruh dunia = SATU
//! kanvas + SATU atlas gambar (tiap jenis pohon sekali), corak dilukis shader,
//! urutan depan-belakang oleh depth buffer → memori GPU kecil & TETAP.
//!
//! Data dunia (jalur, pose kamera, benda, `--off`) disertakan dari
//! src/web/susur/dunia.rs — satu sumber dengan server; kelas CSS benda
//! diterjemahkan ke jaring + bahan di rupa.rs.
//!
//! Kontrak DOM (pages/undangan.rs `Susur3d`): `<canvas class="ss__gl"
//! data-dunia data-nama data-salam data-insan data-v>` di dalam `.pg.ss`,
//! `<template class="ss__cadangan">` berisi dunia CSS 3D — dipasang ke
//! `.ss__dunia` bila WebGL2 tak tersedia / gagal (kanvas diberi `.is-gagal`). Kartu singgah, langit &
//! cakrawala tetap CSS (scroll-driven); kanvas hanya menggantikan dunia.
//!
//! Tanpa kebocoran: loop rAF berhenti (closure-nya dilepas) saat prolog di
//! luar layar; observer & closure memegang `Weak`; konteks hilang → semua
//! sumber daya dilepas, dipasang ulang saat konteks pulih.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    CanvasRenderingContext2d, Element, HtmlCanvasElement, HtmlImageElement, IntersectionObserver, IntersectionObserverEntry,
    IntersectionObserverInit, WebGl2RenderingContext as Gl, WebGlBuffer, WebGlProgram, WebGlTexture, WebGlUniformLocation,
};

#[path = "../../src/web/susur/dunia.rs"]
#[allow(dead_code)]
mod dunia;
mod rupa;

// ───────────────────────────── shader ─────────────────────────────

/// Kamera = transform CSS `.ss__dunia`: rotateZ(goyang) rotateY(arah)
/// translate3d(−x, angguk, −z); proyeksi = `perspective: --pp` dengan titik
/// lenyap di (50 %, --hz) dan mata setinggi (--dasar − --hz) di atas tanah.
const VS: &str = r#"#version 300 es
layout(location=0) in vec3 a_pos;
layout(location=1) in vec3 a_loc;
layout(location=2) in vec4 a_obj;
layout(location=3) in vec2 a_uv;
layout(location=4) in vec2 a_px;
layout(location=5) in vec2 a_size;
layout(location=6) in vec4 a_mat;
layout(location=7) in vec4 a_fx;
layout(location=8) in vec4 a_rect;
layout(location=9) in vec4 a_c1;
layout(location=10) in vec4 a_c2;
layout(location=11) in vec4 a_c3;
uniform vec4 u_kam;
uniform vec4 u_kam2;
uniform vec4 u_pj;
uniform vec2 u_dz;
out vec2 v_uv;
out vec2 v_px;
out float v_alfa;
flat out vec2 v_size;
flat out vec4 v_mat;
flat out vec4 v_fx;
flat out vec4 v_rect;
flat out vec4 v_c1;
flat out vec4 v_c2;
flat out vec4 v_c3;
const float TAU = 6.2831853;
void main() {
  vec3 l = a_loc;
  float t = u_kam2.z;
  v_alfa = 1.0;
  if (a_obj.z == 1.0) {
    float s = -TAU * t / 14.0;
    l.xy = vec2(cos(s) * l.x - sin(s) * l.y, sin(s) * l.x + cos(s) * l.y);
  } else if (a_obj.z == 2.0) {
    float s = TAU * t / 7.0;
    l = vec3(cos(s) * l.x + sin(s) * l.z, l.y, -sin(s) * l.x + cos(s) * l.z);
  } else if (a_obj.z == 3.0) {
    v_alfa = 1.0 - 0.22 * (0.5 - 0.5 * cos(TAU * t / 6.8));
  } else if (a_obj.z == 4.0) {
    v_alfa = 1.0 - 0.25 * (0.5 - 0.5 * cos(TAU * t / 5.2));
  }
  float ry = a_obj.y > 0.5 ? -u_kam.z : a_obj.x;
  float c = cos(ry), s = sin(ry);
  vec3 p = a_pos + vec3(c * l.x + s * l.z, l.y, -s * l.x + c * l.z);
  p += vec3(-u_kam.x, u_kam2.x, -u_kam.y);
  float ca = cos(u_kam.z), sa = sin(u_kam.z);
  p = vec3(ca * p.x + sa * p.z, p.y, -sa * p.x + ca * p.z);
  float cg = cos(u_kam.w), sg = sin(u_kam.w);
  p.xy = vec2(cg * p.x - sg * p.y, sg * p.x + cg * p.y);
  float w = -p.z;
  gl_Position = vec4(p.x * u_pj.x, p.y * u_pj.y + p.z * u_pj.z + u_pj.w, w * u_dz.x + u_dz.y, w);
  if (a_fx.w > 0.0) gl_Position.z -= 0.0005 * w;
  if (u_kam2.y >= a_obj.w) gl_Position = vec4(0.0, 0.0, 2.0, 1.0);
  v_uv = a_uv; v_px = a_px; v_size = a_size; v_mat = a_mat; v_fx = a_fx; v_rect = a_rect;
  v_c1 = a_c1; v_c2 = a_c2; v_c3 = a_c3;
}
"#;

/// Bahan (rupa.rs): 0 padat, 1 gambar atlas, 2 garis berulang, 3 gradien
/// tepi-tengah-tepi, 4 gradien turun, 5 ubin catur, 6 lampu berpendar,
/// 7 tiang (+ cincin emas), 8 pagar, 9 sorot elips. Keluaran premultiplied.
const FS: &str = r#"#version 300 es
precision highp float;
uniform sampler2D u_tex;
in vec2 v_uv;
in vec2 v_px;
in float v_alfa;
flat in vec2 v_size;
flat in vec4 v_mat;
flat in vec4 v_fx;
flat in vec4 v_rect;
flat in vec4 v_c1;
flat in vec4 v_c2;
flat in vec4 v_c3;
out vec4 o;
void main() {
  int k = int(v_mat.x + 0.5);
  vec4 c = v_c1;
  if (k == 1) {
    vec2 m = v_rect.xy + v_uv * v_rect.zw;
    vec2 f = v_rect.xy + vec2(fract(v_uv.x), v_uv.y) * v_rect.zw;
    o = textureGrad(u_tex, f, dFdx(m), dFdy(m)) * (v_c1.a * v_alfa);
    if (o.a < 0.004) discard;
    return;
  } else if (k == 2) {
    float x = v_mat.w < 0.5 ? v_px.y : (v_mat.w < 1.5 ? v_px.x : (v_px.x + v_px.y) * 0.70710678);
    c = mod(x, v_mat.y) >= v_mat.y - v_mat.z ? v_c2 : v_c1;
  } else if (k == 3) {
    c = mix(v_c1, v_c2, 1.0 - abs(2.0 * v_uv.x - 1.0));
  } else if (k == 4) {
    c = mix(v_c1, v_c2, v_uv.y);
  } else if (k == 5) {
    vec2 q = floor(v_px / v_mat.y);
    c = mod(q.x + q.y, 2.0) < 1.0 ? v_c1 : v_c2;
  } else if (k == 6) {
    vec2 q = (v_px - v_size * 0.5) / v_mat.yz;
    float d = length(q);
    if (d <= 1.0) {
      c = mix(v_c1, v_c2, clamp(length(q + vec2(0.0, 0.2)) / 1.2, 0.0, 1.0));
    } else {
      float r = (d - 1.0) * min(v_mat.y, v_mat.z) / v_mat.w;
      c = v_c3;
      c.a *= pow(max(1.0 - r, 0.0), 2.0);
    }
  } else if (k == 7) {
    c = mix(v_c1, v_c2, 1.0 - abs(2.0 * v_uv.x - 1.0));
    if (v_mat.y > 0.0 && (v_px.y < v_mat.y || v_px.y > v_size.y - v_mat.y)) c = v_c3;
  } else if (k == 8) {
    bool palang = v_px.y < 10.0 || abs(v_px.y - v_size.y * 0.5) < 4.0;
    c = palang ? v_c1 : (mod(v_px.x, 104.0) < 10.0 ? v_c2 : vec4(0.0));
  } else if (k == 9) {
    float d = length((v_uv - 0.5) * 2.0);
    c = d < 0.45 ? mix(v_c1, v_c2, d / 0.45) : v_c2 * vec4(1.0, 1.0, 1.0, max(1.0 - (d - 0.45) / 0.55, 0.0));
  }
  if (v_fx.y != 0.0) {
    float b = abs(v_fx.y);
    if (v_px.x < b || v_px.x > v_size.x - b || (v_fx.y > 0.0 && (v_px.y < b || v_px.y > v_size.y - b))) c = v_c3;
  }
  if (v_fx.z > 0.0) {
    float rx = v_size.x * 0.5, ry = v_size.y * 0.09;
    float dy = max(abs(v_px.y - v_size.y * 0.5) - (v_size.y * 0.5 - ry), 0.0);
    float dx = v_px.x - rx;
    if (dx * dx / (rx * rx) + dy * dy / (ry * ry) > 1.0) discard;
  }
  c.rgb *= 1.0 - v_fx.x;
  c.a *= v_alfa;
  if (c.a < 0.004) discard;
  o = vec4(c.rgb * c.a, c.a);
}
"#;

/// Bidang dekat & jauh kamera (px dunia).
const DEKAT: f32 = 30.0;
const JAUH: f32 = 30_000.0;
/// Atlas gambar: lebar tetap, tinggi secukupnya (≤ 2048).
const ATLAS_W: f64 = 2048.0;
const ATLAS_H_MAKS: f64 = 2048.0;
const SELA: f64 = 4.0;

// ───────────────────────────── titik masuk ─────────────────────────────

#[wasm_bindgen(start)]
pub fn mulai() {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else { return };
    let Ok(daftar) = doc.query_selector_all("canvas.ss__gl") else { return };
    for i in 0..daftar.length() {
        if let Some(k) = daftar.item(i).and_then(|n| n.dyn_into::<HtmlCanvasElement>().ok()) {
            pasang(k);
        }
    }
}

type Kursi = Rc<RefCell<Option<Rc<RefCell<Panggung>>>>>;
/// Closure rAF yang bisa melepas dirinya sendiri saat loop berhenti.
type Raf = Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>>;
/// Pengamat layar + closure-nya (dilepas bersama).
type Pengamat = (IntersectionObserver, Closure<dyn FnMut(js_sys::Array)>);

/// Pasang perender pada satu kanvas (+ penanganan konteks WebGL hilang/pulih).
fn pasang(kanvas: HtmlCanvasElement) {
    let kursi: Kursi = Rc::new(RefCell::new(None));
    match Panggung::baru(&kanvas) {
        Ok(p) => *kursi.borrow_mut() = Some(p),
        Err(_) => return cadangan(&kanvas),
    }
    // Seumur halaman (satu kanvas per undangan) → closure sengaja dilepas.
    let k = kursi.clone();
    let hilang = Closure::<dyn FnMut(web_sys::Event)>::new(move |e: web_sys::Event| {
        e.prevent_default();
        if let Some(p) = k.borrow_mut().take() {
            p.borrow_mut().matikan();
        }
    });
    let k = kursi.clone();
    let kv = kanvas.clone();
    let pulih = Closure::<dyn FnMut()>::new(move || match Panggung::baru(&kv) {
        Ok(p) => *k.borrow_mut() = Some(p),
        Err(_) => cadangan(&kv),
    });
    let _ = kanvas.add_event_listener_with_callback("webglcontextlost", hilang.as_ref().unchecked_ref());
    let _ = kanvas.add_event_listener_with_callback("webglcontextrestored", pulih.as_ref().unchecked_ref());
    hilang.forget();
    pulih.forget();
}

/// WebGL2 tak ada / gagal → pasang dunia CSS 3D dari `<template>`. Kanvas
/// DISEMBUNYIKAN, bukan dibuang: hydrate Leptos mengharapkan elemennya ada.
fn cadangan(kanvas: &HtmlCanvasElement) {
    if let Some(bag) = kanvas.closest(".pg.ss").ok().flatten() {
        if let (Ok(Some(dunia)), Ok(Some(tpl))) = (bag.query_selector(".ss__dunia"), bag.query_selector("template.ss__cadangan")) {
            dunia.set_inner_html(&tpl.inner_html());
        }
    }
    let _ = kanvas.class_list().add_1("is-gagal");
}

// ───────────────────────────── panggung ─────────────────────────────

struct Seragam {
    kam: Option<WebGlUniformLocation>,
    kam2: Option<WebGlUniformLocation>,
    pj: Option<WebGlUniformLocation>,
}

struct Panggung {
    gl: Gl,
    kanvas: HtmlCanvasElement,
    bagian: Element,
    dunia: &'static dunia::Dunia,
    jalur: dunia::Jalur,
    tegak: Vec<rupa::Potong>,
    indeks: Vec<u16>,
    urut: Vec<(f32, u32)>,
    tulis: Vec<u16>,
    tanah: i32,
    seragam: Seragam,
    _program: WebGlProgram,
    _vbo: WebGlBuffer,
    ibo: WebGlBuffer,
    tekstur: Option<WebGlTexture>,
    /// Lebar, tinggi (CSS px), DPR terakhir — ganti → ubah ukuran kanvas.
    ukuran: (i32, i32, f64),
    p_akhir: f32,
    t_akhir: f64,
    kotor: bool,
    siap: bool,
    mati: bool,
    tampak: bool,
    berjalan: bool,
    /// prefers-reduced-motion: tanpa gerak waktu (kincir, sorot, kelip).
    tenang: bool,
    /// Pratinjau etalase: gerak WAKTU 5 dtk setelah `.pg--jalan`.
    pv: bool,
    pv_mulai: Option<f64>,
    raf: Raf,
    io: Option<Pengamat>,
}

fn galat(s: &str) -> JsValue {
    JsValue::from_str(s)
}

impl Panggung {
    fn baru(kanvas: &HtmlCanvasElement) -> Result<Rc<RefCell<Panggung>>, JsValue> {
        let win = web_sys::window().ok_or_else(|| galat("window"))?;
        let doc = win.document().ok_or_else(|| galat("document"))?;
        let data = kanvas.dataset();
        let ambil = |k: &str| data.get(k).unwrap_or_default();
        let dunia = dunia::dunia(&ambil("dunia")).ok_or_else(|| galat("dunia"))?;
        let bagian = kanvas.closest(".pg.ss")?.ok_or_else(|| galat("bagian"))?;
        let akar = doc.document_element().ok_or_else(|| galat("akar"))?;
        let kelas = akar.class_list();
        let pv = kelas.contains("is-pv") || kelas.contains("t-pv");

        let opsi = js_sys::Object::new();
        for (k, v) in [("alpha", true), ("antialias", false), ("depth", true), ("stencil", false), ("premultipliedAlpha", true)] {
            js_sys::Reflect::set(&opsi, &k.into(), &v.into())?;
        }
        let gl: Gl = kanvas.get_context_with_context_options("webgl2", &opsi)?.ok_or_else(|| galat("webgl2"))?.dyn_into()?;

        // Dunia: benda, kapan dilewati, gambar yang dibutuhkan → atlas → jaring.
        let jalur = dunia.jalur();
        let tabel = dunia::tabel(dunia, &jalur);
        let benda = dunia.benda(&jalur);
        let off: Vec<Option<f32>> = benda.iter().map(|b| if b.datar { None } else { dunia::off(&tabel, b) }).collect();
        drop(tabel);
        let dpr = win.device_pixel_ratio().min(2.0);
        let atlas = Atlas::kemas(&benda, dpr);
        let jaring = rupa::rakit(dunia, &benda, &off, &|k| atlas.uv(k));
        if jaring.titik.len() >= u16::MAX as usize {
            return Err(galat("jaring terlalu besar"));
        }

        let program = program(&gl)?;
        gl.use_program(Some(&program));
        let vao = gl.create_vertex_array().ok_or_else(|| galat("vao"))?;
        gl.bind_vertex_array(Some(&vao));
        let vbo = gl.create_buffer().ok_or_else(|| galat("vbo"))?;
        gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&vbo));
        // SAFETY: tampilan sementara ke memori WASM, dipakai sebelum alokasi berikutnya.
        let byte = unsafe { std::slice::from_raw_parts(jaring.titik.as_ptr() as *const u8, jaring.titik.len() * rupa::LANGKAH as usize) };
        unsafe { gl.buffer_data_with_array_buffer_view(Gl::ARRAY_BUFFER, &js_sys::Uint8Array::view(byte), Gl::STATIC_DRAW) };
        pasang_atribut(&gl);
        let ibo = gl.create_buffer().ok_or_else(|| galat("ibo"))?;
        gl.bind_buffer(Gl::ELEMENT_ARRAY_BUFFER, Some(&ibo));
        unsafe { gl.buffer_data_with_array_buffer_view(Gl::ELEMENT_ARRAY_BUFFER, &js_sys::Uint16Array::view(&jaring.indeks), Gl::DYNAMIC_DRAW) };

        let seragam = |n: &str| gl.get_uniform_location(&program, n);
        let s = Seragam { kam: seragam("u_kam"), kam2: seragam("u_kam2"), pj: seragam("u_pj") };
        gl.uniform2f(seragam("u_dz").as_ref(), (JAUH + DEKAT) / (JAUH - DEKAT), -2.0 * JAUH * DEKAT / (JAUH - DEKAT));
        gl.uniform1i(seragam("u_tex").as_ref(), 0);
        gl.enable(Gl::BLEND);
        gl.blend_func(Gl::ONE, Gl::ONE_MINUS_SRC_ALPHA);
        gl.depth_func(Gl::LEQUAL);
        gl.clear_color(0.0, 0.0, 0.0, 0.0);

        let tenang = win.match_media("(prefers-reduced-motion: reduce)").ok().flatten().is_some_and(|m| m.matches());
        let rupa::Jaring { titik, indeks, tanah, tegak } = jaring;
        drop(titik);
        let st = Rc::new(RefCell::new(Panggung {
            gl,
            kanvas: kanvas.clone(),
            bagian,
            dunia,
            jalur,
            urut: Vec::with_capacity(tegak.len()),
            tulis: Vec::with_capacity(indeks.len()),
            tegak,
            indeks,
            tanah: tanah as i32,
            seragam: s,
            _program: program,
            _vbo: vbo,
            ibo,
            tekstur: None,
            ukuran: (0, 0, 0.0),
            p_akhir: -1.0,
            t_akhir: 0.0,
            kotor: true,
            siap: false,
            mati: false,
            tampak: false,
            berjalan: false,
            tenang,
            pv,
            pv_mulai: None,
            raf: Rc::new(RefCell::new(None)),
            io: None,
        }));
        amati(&st)?;
        let teks = [("teks-nama", ambil("nama")), ("teks-salam", ambil("salam"))];
        atlas.muat(Rc::downgrade(&st), kanvas, &teks, &ambil("insan"), &ambil("v"))?;
        Ok(st)
    }

    /// Lepas semua sumber daya (konteks hilang). Loop rAF berhenti sendiri.
    fn matikan(&mut self) {
        self.mati = true;
        if let Some((io, _cb)) = self.io.take() {
            io.disconnect();
        }
    }

    /// % gulir prolog — sama dengan rentang `contain` view-timeline CSS.
    fn kemajuan(&mut self, kini: f64) -> f32 {
        if self.pv {
            if !self.bagian.class_list().contains("pg--jalan") {
                return 0.0;
            }
            let m = *self.pv_mulai.get_or_insert(kini);
            return (((kini - m - 400.0) / 5000.0 * 100.0) as f32).clamp(0.0, 100.0);
        }
        let r = self.bagian.get_bounding_client_rect();
        let tinggi = web_sys::window().and_then(|w| w.inner_height().ok()).and_then(|v| v.as_f64()).unwrap_or(0.0);
        let rentang = r.height() - tinggi;
        if rentang < 1.0 {
            return 0.0;
        }
        ((-r.top() / rentang * 100.0) as f32).clamp(0.0, 100.0)
    }

    /// Samakan ukuran kanvas dengan layar (DPR ≤ 2) & proyeksi dengan CSS.
    fn ukur(&mut self) {
        let Some(win) = web_sys::window() else { return };
        let (cw, ch) = (self.kanvas.client_width(), self.kanvas.client_height());
        let dpr = win.device_pixel_ratio().min(2.0);
        if (cw, ch, dpr) == self.ukuran || cw == 0 || ch == 0 {
            return;
        }
        self.ukuran = (cw, ch, dpr);
        let (pw, ph) = ((cw as f64 * dpr).round() as u32, (ch as f64 * dpr).round() as u32);
        self.kanvas.set_width(pw);
        self.kanvas.set_height(ph);
        self.gl.viewport(0, 0, pw as i32, ph as i32);
        // --pp: clamp(500px, min(140vw, 72svh), 1000px); --hz 40 %; --dasar 71 %.
        let vw = win.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(cw as f64) as f32;
        let (w, h) = (cw as f32, ch as f32);
        let pp = (1.4 * vw).min(0.72 * h).clamp(500.0, 1000.0);
        let (hz, dasar) = (0.40, 0.71);
        self.gl.uniform4f(self.seragam.pj.as_ref(), 2.0 * pp / w, -2.0 * pp / h, 2.0 * (hz - 0.5), -2.0 * (dasar - hz) * pp);
        self.kotor = true;
    }

    /// Satu bingkai rAF. `false` = loop berhenti.
    fn bingkai(&mut self, kini: f64) -> bool {
        if self.mati || !self.tampak || !self.siap {
            return false;
        }
        self.ukur();
        let p = self.kemajuan(kini);
        // Diam di singgah: gambar ulang hanya untuk gerak waktu, ±30 fps.
        let gerak = !self.tenang && kini - self.t_akhir >= 33.0;
        if self.kotor || (p - self.p_akhir).abs() > 1e-4 || gerak {
            self.gambar(p, if self.tenang { 0.0 } else { kini / 1000.0 });
            self.p_akhir = p;
            self.t_akhir = kini;
            self.kotor = false;
        }
        true
    }

    fn gambar(&mut self, p: f32, t: f64) {
        let gl = &self.gl;
        let k = self.dunia.pose(&self.jalur, p);
        gl.uniform4f(self.seragam.kam.as_ref(), k.x, k.z, k.a.to_radians(), k.goyang.to_radians());
        gl.uniform4f(self.seragam.kam2.as_ref(), k.angguk, p, (t % 1.0e4) as f32, 0.0);
        gl.depth_mask(true); // clear kedalaman ikut depthMask
        gl.clear(Gl::COLOR_BUFFER_BIT | Gl::DEPTH_BUFFER_BIT);
        // Tanah: lapis tetap, tanpa uji kedalaman (tak pernah menutupi benda tegak).
        gl.disable(Gl::DEPTH_TEST);
        gl.depth_mask(false);
        gl.draw_elements_with_i32(Gl::TRIANGLES, self.tanah, Gl::UNSIGNED_SHORT, 0);
        // Benda tegak: padat dulu (menulis kedalaman), lalu tembus pandang
        // jauh → dekat tanpa menulis kedalaman.
        self.urut.clear();
        for (i, o) in self.tegak.iter().enumerate() {
            if p < o.off {
                self.urut.push((dalam_urut(o.tembus, dunia::dalam(&k, o.x, o.z)), i as u32));
            }
        }
        self.urut.sort_unstable_by(|a, b| b.0.total_cmp(&a.0));
        self.tulis.clear();
        let mut padat = 0;
        for &(_, i) in &self.urut {
            let o = &self.tegak[i as usize];
            self.tulis.extend_from_slice(&self.indeks[o.awal as usize..(o.awal + o.n) as usize]);
            if !o.tembus {
                padat = self.tulis.len() as i32;
            }
        }
        if self.tulis.is_empty() {
            return;
        }
        gl.bind_buffer(Gl::ELEMENT_ARRAY_BUFFER, Some(&self.ibo));
        // SAFETY: tampilan sementara ke `tulis`, dipakai sebelum alokasi berikutnya.
        unsafe { gl.buffer_sub_data_with_i32_and_array_buffer_view(Gl::ELEMENT_ARRAY_BUFFER, self.tanah * 2, &js_sys::Uint16Array::view(&self.tulis)) };
        gl.enable(Gl::DEPTH_TEST);
        gl.depth_mask(true);
        gl.draw_elements_with_i32(Gl::TRIANGLES, padat, Gl::UNSIGNED_SHORT, self.tanah * 2);
        gl.depth_mask(false);
        let sisa = self.tulis.len() as i32 - padat;
        gl.draw_elements_with_i32(Gl::TRIANGLES, sisa, Gl::UNSIGNED_SHORT, (self.tanah + padat) * 2);
    }
}

/// Kunci urut: semua benda padat (kunci sangat besar → paling awal) lalu
/// tembus pandang menurut kedalaman, jauh dulu.
fn dalam_urut(tembus: bool, dalam: f32) -> f32 {
    if tembus { dalam } else { 1.0e9 + dalam }
}

/// Loop rAF; berhenti (closure dilepas) begitu `bingkai` menolak.
fn jalan(st: &Rc<RefCell<Panggung>>) {
    {
        let mut p = st.borrow_mut();
        if p.berjalan || p.mati || !p.tampak || !p.siap {
            return;
        }
        p.berjalan = true;
    }
    let raf = st.borrow().raf.clone();
    let lemah = Rc::downgrade(st);
    let raf2 = raf.clone();
    *raf.borrow_mut() = Some(Closure::new(move |kini: f64| {
        let lanjut = lemah.upgrade().is_some_and(|st| {
            let mut p = st.borrow_mut();
            let l = p.bingkai(kini);
            if !l {
                p.berjalan = false;
            }
            l
        });
        if lanjut {
            minta_bingkai(&raf2);
        } else {
            let _ = raf2.borrow_mut().take();
        }
    }));
    minta_bingkai(&raf);
}

fn minta_bingkai(raf: &Raf) {
    if let (Some(w), Some(f)) = (web_sys::window(), raf.borrow().as_ref()) {
        let _ = w.request_animation_frame(f.as_ref().unchecked_ref());
    }
}

/// Loop hanya hidup selama prolog (± 120 px) ada di layar.
fn amati(st: &Rc<RefCell<Panggung>>) -> Result<(), JsValue> {
    let lemah = Rc::downgrade(st);
    let cb = Closure::<dyn FnMut(js_sys::Array)>::new(move |entri: js_sys::Array| {
        let Some(st) = lemah.upgrade() else { return };
        let tampak = entri.iter().filter_map(|e| e.dyn_into::<IntersectionObserverEntry>().ok()).any(|e| e.is_intersecting());
        st.borrow_mut().tampak = tampak;
        if tampak {
            jalan(&st);
        }
    });
    let opsi = IntersectionObserverInit::new();
    opsi.set_root_margin("120px");
    let io = IntersectionObserver::new_with_options(cb.as_ref().unchecked_ref(), &opsi)?;
    io.observe(&st.borrow().bagian);
    st.borrow_mut().io = Some((io, cb));
    Ok(())
}

fn kompilasi(gl: &Gl, jenis: u32, sumber: &str) -> Result<web_sys::WebGlShader, JsValue> {
    let s = gl.create_shader(jenis).ok_or_else(|| galat("shader"))?;
    gl.shader_source(&s, sumber);
    gl.compile_shader(&s);
    if gl.get_shader_parameter(&s, Gl::COMPILE_STATUS).as_bool() == Some(true) {
        Ok(s)
    } else {
        Err(galat(&gl.get_shader_info_log(&s).unwrap_or_default()))
    }
}

fn program(gl: &Gl) -> Result<WebGlProgram, JsValue> {
    let vs = kompilasi(gl, Gl::VERTEX_SHADER, VS)?;
    let fs = kompilasi(gl, Gl::FRAGMENT_SHADER, FS)?;
    let p = gl.create_program().ok_or_else(|| galat("program"))?;
    gl.attach_shader(&p, &vs);
    gl.attach_shader(&p, &fs);
    gl.link_program(&p);
    gl.delete_shader(Some(&vs));
    gl.delete_shader(Some(&fs));
    if gl.get_program_parameter(&p, Gl::LINK_STATUS).as_bool() == Some(true) {
        Ok(p)
    } else {
        Err(galat(&gl.get_program_info_log(&p).unwrap_or_default()))
    }
}

/// Tata letak `rupa::Titik` (124 byte).
fn pasang_atribut(gl: &Gl) {
    let f = Gl::FLOAT;
    let tata: [(u32, i32, u32, bool, i32); 12] = [
        (0, 3, f, false, 0),
        (1, 3, f, false, 12),
        (2, 4, f, false, 24),
        (3, 2, f, false, 40),
        (4, 2, f, false, 48),
        (5, 2, f, false, 56),
        (6, 4, f, false, 64),
        (7, 4, f, false, 80),
        (8, 4, f, false, 96),
        (9, 4, Gl::UNSIGNED_BYTE, true, 112),
        (10, 4, Gl::UNSIGNED_BYTE, true, 116),
        (11, 4, Gl::UNSIGNED_BYTE, true, 120),
    ];
    for (i, n, t, norm, off) in tata {
        gl.enable_vertex_attrib_array(i);
        gl.vertex_attrib_pointer_with_i32(i, n, t, norm, rupa::LANGKAH, off);
    }
}

// ───────────────────────────── atlas gambar ─────────────────────────────

/// Satu gambar di atlas: kunci, kotak px.
struct Isi {
    kunci: &'static str,
    kotak: [f64; 4],
}

struct Atlas {
    isi: Vec<Isi>,
    tinggi: f64,
    /// px atlas per px CSS.
    skala: f64,
}

impl Atlas {
    /// Kemas semua gambar (satu per jenis, ukuran salinan terbesar) dengan
    /// rak berjajar; resolusi 0,8 px atlas per px CSS di DPR 2, diperkecil
    /// sampai muat 2048 × 2048.
    fn kemas(benda: &[dunia::Benda], dpr: f64) -> Atlas {
        let mut ukuran: Vec<(&'static str, f64, f64)> = Vec::new();
        for (k, w, h) in benda.iter().filter_map(rupa::gambar) {
            match ukuran.iter_mut().find(|u| u.0 == k) {
                Some(u) => {
                    u.1 = u.1.max(w as f64);
                    u.2 = u.2.max(h as f64);
                }
                None => ukuran.push((k, w as f64, h as f64)),
            }
        }
        ukuran.sort_by(|a, b| b.2.total_cmp(&a.2));
        // 0,8 px atlas per px CSS di DPR 2 (benda hampir selalu tampil lebih
        // kecil dari ukuran CSS-nya; mipmap menangani yang jauh).
        let mut skala = 0.4 * dpr.max(1.0);
        loop {
            let (mut x, mut y, mut rak) = (0.0, 0.0, 0.0f64);
            let mut isi = Vec::with_capacity(ukuran.len());
            for &(kunci, w, h) in &ukuran {
                let (w, h) = ((w * skala).ceil(), (h * skala).ceil());
                if x + w + SELA > ATLAS_W {
                    x = 0.0;
                    y += rak + SELA;
                    rak = 0.0;
                }
                isi.push(Isi { kunci, kotak: [x + SELA, y + SELA, w, h] });
                x += w + SELA;
                rak = rak.max(h);
            }
            let tinggi = (y + rak + 2.0 * SELA).ceil();
            if tinggi <= ATLAS_H_MAKS || skala < 0.2 {
                return Atlas { isi, tinggi: tinggi.min(ATLAS_H_MAKS), skala };
            }
            skala *= 0.85;
        }
    }

    fn uv(&self, kunci: &str) -> [f32; 4] {
        self.isi.iter().find(|i| i.kunci == kunci).map_or([0.0; 4], |i| {
            let k = i.kotak;
            [(k[0] / ATLAS_W) as f32, (k[1] / self.tinggi) as f32, (k[2] / ATLAS_W) as f32, (k[3] / self.tinggi) as f32]
        })
    }

    /// Unduh & lukis semua gambar ke kanvas 2D sementara, lalu unggah sebagai
    /// SATU tekstur bermipmap; kanvas 2D & gambar dilepas sesudahnya.
    /// `teks` = (kunci papan, isi) — nama mempelai & salam, fon skrip undangan.
    fn muat(self, st: Weak<RefCell<Panggung>>, kanvas_gl: &HtmlCanvasElement, teks: &[(&str, String)], insan: &str, v: &str) -> Result<(), JsValue> {
        let doc = web_sys::window().and_then(|w| w.document()).ok_or_else(|| galat("document"))?;
        let kanvas: HtmlCanvasElement = doc.create_element("canvas")?.dyn_into()?;
        kanvas.set_width(ATLAS_W as u32);
        kanvas.set_height(self.tinggi as u32);
        let ctx: CanvasRenderingContext2d = kanvas.get_context("2d")?.ok_or_else(|| galat("2d"))?.dyn_into()?;
        let gambar: Vec<&Isi> = self.isi.iter().filter(|i| !i.kunci.starts_with("teks-")).collect();
        let tunggu = Rc::new(RefCell::new(Muat { sisa: gambar.len() + 1, kanvas, ctx, st }));

        for i in gambar {
            let url = match i.kunci {
                "insan" => insan.to_string(),
                "lengkung" => "/panggung/jl-lengkung.svg".to_string(),
                k => format!("/susur/svg/{k}.svg?v={v}"),
            };
            let img = HtmlImageElement::new()?;
            let kotak = i.kotak;
            let potong = i.kunci == "insan";
            let (t1, im1) = (tunggu.clone(), img.clone());
            let ok = Closure::once_into_js(move || {
                let m = t1.borrow();
                let [x, y, w, h] = kotak;
                if potong {
                    // Sepasang mempelai: potongan 300×300 (x430 y1125) kanvas insan 1080×1920.
                    let s = w / 300.0;
                    m.ctx.save();
                    m.ctx.begin_path();
                    m.ctx.rect(x, y, w, h);
                    m.ctx.clip();
                    let _ = m.ctx.draw_image_with_html_image_element_and_dw_and_dh(&im1, x - 430.0 * s, y - 1125.0 * s, 1080.0 * s, 1920.0 * s);
                    m.ctx.restore();
                } else {
                    let _ = m.ctx.draw_image_with_html_image_element_and_dw_and_dh(&im1, x, y, w, h);
                }
                drop(m);
                Muat::selesai_satu(&t1);
            });
            let t2 = tunggu.clone();
            let gagal = Closure::once_into_js(move || Muat::selesai_satu(&t2));
            img.set_onload(Some(ok.unchecked_ref()));
            img.set_onerror(Some(gagal.unchecked_ref()));
            img.set_src(&url);
        }

        // Teks papan nama: tunggu fon skrip undangan termuat.
        let gaya = web_sys::window()
            .and_then(|w| w.get_computed_style(kanvas_gl).ok().flatten())
            .map(|g| {
                let f = g.get_property_value("--font-script").unwrap_or_default();
                if f.trim().is_empty() { g.get_property_value("--font-display").unwrap_or_default() } else { f }
            })
            .unwrap_or_default();
        let keluarga = if gaya.trim().is_empty() { "cursive".to_string() } else { gaya.trim().to_string() };
        let tulisan: Vec<(String, [f64; 4])> = teks
            .iter()
            .filter_map(|(k, t)| self.isi.iter().find(|i| i.kunci == *k).map(|i| (t.clone(), i.kotak)))
            .collect();
        let skala = self.skala;
        let t3 = tunggu.clone();
        let fon = format!("500 {}px {keluarga}", (34.0 * skala).round());
        let lukis = Closure::once_into_js(move |_: JsValue| {
            {
                let m = t3.borrow();
                m.ctx.set_fill_style_str("#5a3a1e");
                m.ctx.set_text_align("center");
                m.ctx.set_text_baseline("middle");
                for (s, [x, y, w, h]) in &tulisan {
                    // Papan CSS: 34px, padding 10px; nama panjang diperkecil agar muat.
                    let mut px = 34.0 * skala;
                    m.ctx.set_font(&format!("500 {px}px {keluarga}"));
                    let lebar = m.ctx.measure_text(s).map(|t| t.width()).unwrap_or(0.0);
                    let muat = w - 20.0 * skala;
                    if lebar > muat && lebar > 0.0 {
                        px *= muat / lebar;
                        m.ctx.set_font(&format!("500 {px}px {keluarga}"));
                    }
                    let _ = m.ctx.fill_text(s, x + w / 2.0, y + h / 2.0);
                }
            }
            Muat::selesai_satu(&t3);
        });
        // Gagal/lambat memuat fon → tetap dilukis dengan fon cadangan.
        let janji = doc.fonts().load(&fon);
        if let Ok(then) = js_sys::Reflect::get(&janji, &"then".into()).and_then(|t| t.dyn_into::<js_sys::Function>()) {
            let _ = then.call2(&janji, &lukis, &lukis);
        }
        Ok(())
    }
}

struct Muat {
    sisa: usize,
    kanvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    st: Weak<RefCell<Panggung>>,
}

impl Muat {
    fn selesai_satu(m: &Rc<RefCell<Muat>>) {
        let sisa = {
            let mut m = m.borrow_mut();
            m.sisa = m.sisa.saturating_sub(1);
            m.sisa
        };
        if sisa > 0 {
            return;
        }
        let m = m.borrow();
        let Some(st) = m.st.upgrade() else { return };
        {
            let mut p = st.borrow_mut();
            if p.mati {
                return;
            }
            let gl = &p.gl;
            let tex = gl.create_texture();
            gl.active_texture(Gl::TEXTURE0);
            gl.bind_texture(Gl::TEXTURE_2D, tex.as_ref());
            gl.pixel_storei(Gl::UNPACK_PREMULTIPLY_ALPHA_WEBGL, 1);
            let _ = gl.tex_image_2d_with_u32_and_u32_and_html_canvas_element(Gl::TEXTURE_2D, 0, Gl::RGBA as i32, Gl::RGBA, Gl::UNSIGNED_BYTE, &m.kanvas);
            gl.generate_mipmap(Gl::TEXTURE_2D);
            gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MIN_FILTER, Gl::LINEAR_MIPMAP_LINEAR as i32);
            gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MAG_FILTER, Gl::LINEAR as i32);
            gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_S, Gl::CLAMP_TO_EDGE as i32);
            gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_T, Gl::CLAMP_TO_EDGE as i32);
            p.tekstur = tex;
            p.siap = true;
            p.kotor = true;
            let _ = p.kanvas.class_list().add_1("is-siap");
        }
        // Lepas kanvas 2D sementara (memori bitmap-nya).
        m.kanvas.set_width(0);
        m.kanvas.set_height(0);
        jalan(&st);
    }
}
