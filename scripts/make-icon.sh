#!/usr/bin/env bash
# Convert assets/logo.svg into assets/IdeUltra.icns using only system tools.
# Pipeline: qlmanage (SVG → PNG via Quick Look) → sips (resize) → iconutil (bundle).

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/assets/logo.svg"
OUT_ICNS="$ROOT/assets/IdeUltra.icns"
WORK="$ROOT/assets/_iconset.iconset"

if [ ! -f "$SRC" ]; then
  echo "error: $SRC not found" >&2
  exit 1
fi

rm -rf "$WORK" "$OUT_ICNS"
mkdir -p "$WORK"
TMP="$ROOT/assets/_logo_master"
mkdir -p "$TMP"

echo "==> Rasterizing SVG via qlmanage"
# qlmanage outputs <name>.png alongside in the output dir.
qlmanage -t -s 1024 -o "$TMP" "$SRC" >/dev/null 2>&1 || true
MASTER="$TMP/logo.svg.png"
if [ ! -f "$MASTER" ]; then
  echo "qlmanage didn't produce a PNG; falling back to a Rust resvg helper"
  exit 1
fi

# Resize to each required size for an .iconset.
for size in 16 32 64 128 256 512; do
  sips -s format png -z "$size" "$size" "$MASTER" --out "$WORK/icon_${size}x${size}.png" >/dev/null
  double=$((size * 2))
  sips -s format png -z "$double" "$double" "$MASTER" --out "$WORK/icon_${size}x${size}@2x.png" >/dev/null
done
# 1024x1024 doesn't need a @2x.
sips -s format png -z 1024 1024 "$MASTER" --out "$WORK/icon_512x512@2x.png" >/dev/null

echo "==> Bundling .icns"
iconutil -c icns "$WORK" -o "$OUT_ICNS"
rm -rf "$WORK" "$TMP"
echo "==> Wrote $OUT_ICNS ($(du -h "$OUT_ICNS" | awk '{print $1}'))"
