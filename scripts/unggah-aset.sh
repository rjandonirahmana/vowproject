#!/usr/bin/env bash
# scripts/unggah-aset.sh — unggah public/img & public/video ke RustFS lalu
# catat URL tiap berkas di tabel `aset` (migrasi 038).
#
#   RUSTFS_ENDPOINT=https://image.ulalaapi.store \
#   RUSTFS_PUBLIC_URL=https://image.ulalaapi.store \
#   RUSTFS_ACCESS_KEY=… RUSTFS_SECRET_KEY=… \
#   DATABASE_URL=postgres://…/undangan \
#   scripts/unggah-aset.sh
#
# Aman diulang: `mc mirror` hanya mengirim berkas yang berubah, baris `aset`
# di-upsert, dan baris untuk berkas yang sudah dihapus dari public/ ikut
# dibuang (objeknya di RustFS dibiarkan — mungkin masih dirujuk cache tamu).
# Server membaca tabel ini SAAT START → restart setelah skrip selesai.
# Butuh: mc (MinIO client), psql, shasum/sha256sum.
set -euo pipefail

cd "$(dirname "$0")/.."

: "${RUSTFS_ENDPOINT:?RUSTFS_ENDPOINT kosong}"
: "${RUSTFS_ACCESS_KEY:?RUSTFS_ACCESS_KEY kosong}"
: "${RUSTFS_SECRET_KEY:?RUSTFS_SECRET_KEY kosong}"
: "${DATABASE_URL:?DATABASE_URL kosong}"
BUCKET="${RUSTFS_BUCKET:-undangan}"
PUBLIC_URL="${RUSTFS_PUBLIC_URL:-$RUSTFS_ENDPOINT}"
PUBLIC_URL="${PUBLIC_URL%/}"
# Sama dengan storage.rs public_base: bucket tak ditambahkan dua kali.
case "$PUBLIC_URL" in */"$BUCKET") BASE="$PUBLIC_URL" ;; *) BASE="$PUBLIC_URL/$BUCKET" ;; esac
PREFIX="aset"
DIRS=(img video)

# Kredensial hanya di direktori konfigurasi sementara, bukan ~/.mc.
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
MC=(mc --config-dir "$TMP/mc" --quiet)
"${MC[@]}" alias set aset "$RUSTFS_ENDPOINT" "$RUSTFS_ACCESS_KEY" "$RUSTFS_SECRET_KEY" --api S3v4 >/dev/null

# Pekerja dibatasi: di uplink lambat, permintaan yang sudah ditandatangani
# tapi lama mengantre ditolak RustFS (RequestTimeTooSkewed). `mc mirror`
# tetap keluar 0 walau ada yang gagal → galat dihitung dari keluaran JSON,
# lalu diulang (berkas yang sudah sama dilewati).
for d in "${DIRS[@]}"; do
  echo "→ mirror public/$d → $BUCKET/$PREFIX/$d"
  for coba in 1 2 3 4 5; do
    gagal=$("${MC[@]}" mirror --overwrite --max-workers 4 --exclude ".DS_Store" --exclude "*.md" --json \
      "public/$d" "aset/$BUCKET/$PREFIX/$d" 2>&1 | grep -c '"status":"error"' || true)
    [ "$gagal" -eq 0 ] && break
    echo "  $gagal berkas gagal — ulang ($coba)"
  done
  [ "$gagal" -eq 0 ] || { echo "✗ public/$d belum terunggah semua" >&2; exit 1; }
done

if command -v sha256sum >/dev/null; then SHA=(sha256sum); else SHA=(shasum -a 256); fi

mime_of() {
  case "${1##*.}" in
    svg) echo image/svg+xml ;; png) echo image/png ;; jpg|jpeg) echo image/jpeg ;;
    webp) echo image/webp ;; gif) echo image/gif ;; avif) echo image/avif ;;
    mp4) echo video/mp4 ;; webm) echo video/webm ;; mov) echo video/quicktime ;;
    mp3) echo audio/mpeg ;; m4a) echo audio/mp4 ;; ogg) echo audio/ogg ;;
    *) echo application/octet-stream ;;
  esac
}

# TSV jalur, url, mime, ukuran, sha256 → \copy ke tabel sementara.
TSV="$TMP/aset.tsv"
: >"$TSV"
n=0
while IFS= read -r -d '' f; do
  rel="${f#public/}"
  size=$(wc -c <"$f" | tr -d ' ')
  sum=$("${SHA[@]}" "$f" | cut -d' ' -f1)
  printf '/%s\t%s/%s/%s\t%s\t%s\t%s\n' "$rel" "$BASE" "$PREFIX" "$rel" "$(mime_of "$f")" "$size" "$sum" >>"$TSV"
  n=$((n + 1))
done < <(find "${DIRS[@]/#/public/}" -type f ! -name .DS_Store ! -name '*.md' -print0 | sort -z)

echo "→ catat $n berkas di tabel aset"
psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -q <<SQL
BEGIN;
CREATE TEMP TABLE aset_baru (jalur TEXT, url TEXT, mime TEXT, ukuran BIGINT, sha256 TEXT) ON COMMIT DROP;
\copy aset_baru FROM '$TSV'
INSERT INTO aset (jalur, url, mime, ukuran, sha256)
  SELECT jalur, url, mime, ukuran, sha256 FROM aset_baru
  ON CONFLICT (jalur) DO UPDATE
    SET url = EXCLUDED.url, mime = EXCLUDED.mime, ukuran = EXCLUDED.ukuran, sha256 = EXCLUDED.sha256,
        diunggah = CASE WHEN aset.sha256 = EXCLUDED.sha256 THEN aset.diunggah ELSE now() END;
DELETE FROM aset WHERE jalur NOT IN (SELECT jalur FROM aset_baru);
COMMIT;
SQL
echo "✓ selesai — restart server agar peta aset dimuat ulang"
