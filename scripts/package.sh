#!/usr/bin/env bash
# Build a macOS universal (arm64 + x86_64) .app and .dmg for IdeUltra.
#
# Usage:
#   ./scripts/package.sh                  # uses version from Cargo.toml, universal
#   ./scripts/package.sh 0.7.0            # explicit version, universal
#   IDEULTRA_ARCH=arm64 ./scripts/package.sh   # arm64-only (skip x86_64 target)
#
# Output:
#   dist/IdeUltra.app
#   dist/IdeUltra-<VERSION>.dmg

set -euo pipefail

VERSION="${1:-$(grep '^version' Cargo.toml | head -1 | awk -F'"' '{print $2}')}"
ARCH_MODE="${IDEULTRA_ARCH:-universal}"

DIST="dist"
APP="$DIST/IdeUltra.app"
BINARY_NAME="ideultra"
BUNDLE_ID="com.mpjhorner.ideultra"

echo "==> Cleaning $DIST"
rm -rf "$DIST"
mkdir -p "$DIST"

case "$ARCH_MODE" in
  universal)
    echo "==> Building arm64 + x86_64 (universal)"
    cargo build --release --target aarch64-apple-darwin
    cargo build --release --target x86_64-apple-darwin
    echo "==> lipo → universal binary"
    BIN_PATH="$DIST/$BINARY_NAME"
    lipo -create -output "$BIN_PATH" \
      "target/aarch64-apple-darwin/release/$BINARY_NAME" \
      "target/x86_64-apple-darwin/release/$BINARY_NAME"
    ;;
  arm64)
    echo "==> Building arm64 only"
    cargo build --release --target aarch64-apple-darwin
    BIN_PATH="target/aarch64-apple-darwin/release/$BINARY_NAME"
    ;;
  *)
    echo "error: IDEULTRA_ARCH must be 'universal' or 'arm64', got '$ARCH_MODE'" >&2
    exit 1
    ;;
esac

echo "==> Assembling $APP"
mkdir -p "$APP/Contents/MacOS"
mkdir -p "$APP/Contents/Resources"
cp "$BIN_PATH" "$APP/Contents/MacOS/$BINARY_NAME"

cat > "$APP/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key><string>en</string>
  <key>CFBundleExecutable</key><string>$BINARY_NAME</string>
  <key>CFBundleIdentifier</key><string>$BUNDLE_ID</string>
  <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
  <key>CFBundleName</key><string>IdeUltra</string>
  <key>CFBundleDisplayName</key><string>IdeUltra</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$VERSION</string>
  <key>CFBundleVersion</key><string>$VERSION</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSSupportsAutomaticGraphicsSwitching</key><true/>
  <key>CFBundleIconFile</key><string>IdeUltra.icns</string>
  <key>LSApplicationCategoryType</key><string>public.app-category.developer-tools</string>
  <key>NSHumanReadableCopyright</key><string>MIT — github.com/MPJHorner/IdeUltra</string>
</dict>
</plist>
EOF

if [ -f "assets/IdeUltra.icns" ]; then
  cp "assets/IdeUltra.icns" "$APP/Contents/Resources/IdeUltra.icns"
  echo "==> Bundled icon"
else
  echo "==> No icon at assets/IdeUltra.icns (skipped)"
fi

# Verify the binary is actually universal when we asked for it.
if [ "$ARCH_MODE" = "universal" ]; then
  ARCHES=$(lipo -archs "$APP/Contents/MacOS/$BINARY_NAME")
  if [ "$ARCHES" != "x86_64 arm64" ] && [ "$ARCHES" != "arm64 x86_64" ]; then
    echo "warning: expected universal binary, lipo reports: $ARCHES"
  fi
  echo "    arches: $ARCHES"
fi

echo "==> Building DMG"
DMG="$DIST/IdeUltra-$VERSION.dmg"
hdiutil create \
  -volname "IdeUltra $VERSION" \
  -srcfolder "$APP" \
  -ov \
  -format UDZO \
  "$DMG" >/dev/null

du -h "$APP/Contents/MacOS/$BINARY_NAME" | awk '{print "    binary: " $1}'
du -h "$DMG" | awk '{print "    dmg:    " $1}'

echo "==> Done"
echo "    Open: open $APP"
echo "    DMG:  open $DMG"
