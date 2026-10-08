# syntax=docker/dockerfile:1.7
# ═══════════════════════════════════════════════════════════════════════════════
# Dockerfile — ilyvowcraft (Leptos SSR + Axum, satu binary satu port)
#
#   Stage 1 (builder): rust:1.95-alpine (musl, STATIS)
#     a. cargo-leptos DIPIN — layer sendiri, rerun hanya saat toolchain berubah
#     b. pre-compile dependency dengan dummy src — cache s/d Cargo.toml/lock berubah
#     c. cargo leptos build --release --precompress → WASM + SSR + .br/.gz sekali jalan
#   Stage 2 (runtime): debian:bookworm-slim, NON-ROOT
#
# Pola sama dengan e-ticketing & ppm. TIDAK ADA TEST DI SINI — disengaja: test
# milik job CI terpisah, supaya kegagalan test tak tercampur log build Docker.
#
# Env WAJIB saat runtime:
#   DATABASE_URL
#
# Opsional: AUTO_MIGRATE (default false = migrasi manual), ADMIN_WHATSAPP,
#   RUSTFS_ENDPOINT/ACCESS_KEY/SECRET_KEY/BUCKET/PUBLIC_URL, RUST_LOG
#
# Port: app mendengar di 3000 DI DALAM container (SITE_ADDR di bawah). Di host
# produksi dipetakan ke 3600 (3000 panel WA, 3100 e-ticketing, 3200-3202 ppm,
# 3300 Gitea, 3400 lajubus, 3500 ILY). Proxy/domain BELUM diatur.
#
# Run:  docker build -t undangan .
#       docker run -p 3600:3000 --env-file .env undangan
# ═══════════════════════════════════════════════════════════════════════════════

# ── Builder ───────────────────────────────────────────────────────────────────
# Versi DIPIN. MSRV tertinggi di Cargo.lock saat ini 1.94.1 (crate aws-smithy-*);
# naikkan tag ini bila `cargo update` menarik dependency yang menuntut lebih baru —
# BERSAMAAN dengan `toolchain:` di .github/workflows/master.yml.
FROM rust:1.95-alpine AS builder

# cmake + linux-headers: jaring pengaman untuk aws-lc-sys (rustls/aws-sdk-s3) —
# versi sekarang memakai `cc`, tapi fallback-nya butuh cmake. binaryen = wasm-opt,
# brotli dipakai --precompress. TIDAK ada OpenSSL: satu-satunya stack TLS adalah
# rustls + aws-lc (Cargo.lock tanpa openssl-sys/native-tls — jaga tetap begitu).
RUN apk add --no-cache \
    musl-dev g++ make perl pkgconfig cmake linux-headers \
    zlib-dev zlib-static \
    binaryen brotli

RUN rustup target add wasm32-unknown-unknown

# cargo-leptos sebagai layer sendiri, VERSI DIPIN (= `cargo leptos --version` di
# laptop pengembang). `--locked` hanya mengunci dependensinya, bukan versinya —
# tanpa `--version`, rilis baru cargo-leptos bisa mengubah tata letak keluaran /pkg diam-diam.
#
# wasm-bindgen: cargo-leptos 0.3.9 membaca versi crate dari Cargo.lock (=0.2.122,
# dipin di Cargo.toml) lalu mengunduh CLI yang PERSIS sama — tak perlu memasang
# wasm-bindgen-cli sendiri, dan skema bindgen dijamin cocok.
RUN --mount=type=cache,id=undangan-cargo-registry,target=/usr/local/cargo/registry \
    cargo install cargo-leptos --locked --version 0.3.9

ENV PKG_CONFIG_ALLOW_CROSS=1
WORKDIR /app

# ── Pre-compile dependency ────────────────────────────────────────────────────
# Salin HANYA input yang memengaruhi dependency → layer ini invalid saat
# Cargo.toml/lock/build.rs/migration/style berubah, BUKAN saat edit src/.
COPY Cargo.toml Cargo.lock build.rs ./
# migration/ WAJIB ada sebelum compile apa pun: build.rs meng-embed setiap
# migration/*.sql ke binari (`fs::read_dir("migration").expect(...)`) — tanpa
# ini build.rs panic. Pelajaran yang sama dari e-ticketing.
COPY migration/ ./migration/
# style/ = `style-file` (CSS biasa, tanpa Tailwind), public/ = `assets-dir` (cargo-leptos gagal bila
# tak ada). public/ memuat ikon app + manifest yang disajikan dari root situs.
COPY style/ ./style/
COPY public/ ./public/

# Dummy source agar Cargo meng-compile & men-cache SELURUH dependency.
RUN mkdir -p src && \
    printf 'fn main() {}' > src/main.rs && \
    printf '' > src/lib.rs

# Dua langkah di bawah HANYA memanaskan cache — `|| true` disengaja. Build yang
# sesungguhnya (tanpa jaring pengaman) ada di bawah.
#
# Deps SSR: fitur persis seperti yang dipakai cargo-leptos untuk binari
# (`--no-default-features --features=ssr`), profil release.
RUN --mount=type=cache,id=undangan-cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=undangan-target,target=/app/target \
    cargo build --release --no-default-features --features ssr 2>&1 || true

# Deps WASM: `--profile wasm-release` (= lib-profile-release di Cargo.toml) dan
# `--no-default-features` (default = ssr; ssr+hydrate bersamaan tidak sah).
# Profil/target berbeda = direktori artefak berbeda — meniru persis perintah
# cargo-leptos adalah satu-satunya cara cache ini benar-benar terpakai.
RUN --mount=type=cache,id=undangan-cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=undangan-target,target=/app/target \
    cargo build --profile wasm-release --target wasm32-unknown-unknown \
    --no-default-features --features hydrate --lib 2>&1 || true

# ── Build final ───────────────────────────────────────────────────────────────
COPY src/ ./src/
# templat/ di-embed lewat include_str! (server/templat.rs: tata.js, tata.css,
# templat/<slug>/{templat.html,gaya.css,meta.json}) — tanpa ini compile gagal
# "couldn't read src/server/../../templat/kusuma/meta.json". Setiap berkas di
# luar src/ yang di-include_str! WAJIB disalin di sini juga.
COPY templat/ ./templat/
# Sentuh agar Cargo tahu source berubah setelah swap dummy→real.
RUN touch src/main.rs src/lib.rs

# `--precompress`: .br + .gz untuk tiap aset /pkg dibuat SEKALI di sini dan
# disajikan `ServeDir::precompressed_br()` (main.rs) — bundle WASM tak dikompresi
# ulang per klien di VPS kecil.
#
# Salinan `undangan_bg.wasm`: glue JS hasil wasm-bindgen memakai `undangan_bg.wasm` sebagai
# nama bawaan, sedangkan cargo-leptos menulis `undangan.wasm`. Server meminta
# `undangan.wasm` (LEPTOS_OUTPUT_NAME di-set saat compile), jadi ini jaring pengaman
# murah: kalau suatu saat nama itu bergeser, hydration tetap jalan alih-alih
# 404 diam-diam yang membuat semua tombol mati (pernah menimpa e-ticketing).
RUN --mount=type=cache,id=undangan-cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=undangan-target,target=/app/target \
    cargo leptos build --release --precompress \
    && cp /app/target/release/undangan /app/undangan-bin \
    && cp -r /app/target/site /app/site-out \
    && cd /app/site-out/pkg \
    && for ext in "" .br .gz; do \
         if [ -f "undangan.wasm${ext}" ] && [ ! -f "undangan_bg.wasm${ext}" ]; then \
           cp "undangan.wasm${ext}" "undangan_bg.wasm${ext}"; \
         fi; \
       done \
    && test -f undangan.wasm && test -f undangan.js && test -f undangan.css \
    && ls -la /app/site-out/pkg/

# ── Runtime ───────────────────────────────────────────────────────────────────
# Binari musl STATIS (aws-lc ikut tertaut statis) → jalan di distro mana pun. Debian dipilih
# karena curl (HEALTHCHECK) + ca-certificates (HTTPS ke RustFS) teruji.
# JANGAN ganti builder ke target glibc sambil membiarkan runtime ini apa adanya.
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

# NON-ROOT: proses memegang kredensial Postgres dan RustFS.
# UID tetap (10001) supaya kepemilikan volume ter-mount stabil antar rebuild.
RUN useradd --system --uid 10001 --create-home --shell /usr/sbin/nologin undangan

WORKDIR /app

COPY --from=builder /app/undangan-bin   ./undangan
COPY --from=builder /app/site-out  ./target/site
# Cargo.toml WAJIB saat runtime: `get_configuration(Some("Cargo.toml"))` membaca
# [package.metadata.leptos]. Hilang → panic saat start.
COPY --from=builder /app/Cargo.toml ./Cargo.toml

RUN chown -R undangan:undangan /app
USER undangan

# SITE_ADDR dibaca config.rs (default = site-addr Cargo.toml 0.0.0.0:3600 — dev lokal).
# Di container selalu 3000; pemetaan port host diatur `docker run -p`.
ENV SITE_ADDR=0.0.0.0:3000
ENV LEPTOS_SITE_ADDR=0.0.0.0:3000
ENV LEPTOS_SITE_ROOT=target/site
ENV LEPTOS_ENV=PROD
ENV RUST_LOG=info

EXPOSE 3000

# /healthz murah (tanpa query DB). start-period 45s: startup menyambung Postgres,
# menjalankan migrasi (AUTO_MIGRATE) di bawah advisory lock, dan
# memeriksa bucket RustFS (timeout 3 dtk) — jangan sampai di-restart sebelum siap.
HEALTHCHECK --interval=15s --timeout=3s --start-period=45s --retries=3 \
    CMD curl -fsS http://localhost:3000/healthz || exit 1

CMD ["./undangan"]
