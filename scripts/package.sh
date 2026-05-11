#!/usr/bin/env bash
# Build a macOS .app bundle and .dmg for IdeUltra. arm64-only for v0.1.0;
# universal (lipo) is planned for v0.1.1.
#
# Usage: ./scripts/package.sh [VERSION]
# Example: ./scripts/package.sh 0.1.0
#
# Output:
#   dist/IdeUltra.app
#   dist/IdeUltra-<VERSION>.dmg

set -euo pipefail

VERSION="${1:-$(grep '^version' Cargo.toml | head -1 | awk -F'"' '{print $2}')}"
DIST="dist"
APP="$DIST/IdeUltra.app"
BINARY_NAME="ideultra"
BUNDLE_ID="com.mpjhorner.ideultra"

echo "==> Cleaning $DIST"
rm -rf "$DIST"
mkdir -p "$DIST"

echo "==> Building release binary (arm64)"
cargo build --release

echo "==> Assembling $APP"
mkdir -p "$APP/Contents/MacOS"
mkdir -p "$APP/Contents/Resources"
cp "target/release/$BINARY_NAME" "$APP/Contents/MacOS/$BINARY_NAME"

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

# Optional icon: drop assets/IdeUltra.icns next to the script and it'll be
# bundled. Skipped for v0.1.0 — see issue tracker.
if [ -f "assets/IdeUltra.icns" ]; then
  cp "assets/IdeUltra.icns" "$APP/Contents/Resources/IdeUltra.icns"
  echo "==> Bundled icon"
else
  echo "==> No icon at assets/IdeUltra.icns (v0.1.0 ships without one)"
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
