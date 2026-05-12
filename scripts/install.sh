#!/usr/bin/env bash
# IdeUltra one-line installer for macOS.
#
#   curl -fsSL https://raw.githubusercontent.com/MPJHorner/IdeUltra/main/scripts/install.sh | bash
#
# Downloads the latest release DMG from GitHub, mounts it via hdiutil,
# copies IdeUltra.app to /Applications, clears the macOS quarantine
# flag, and ejects the DMG. macOS-only — the .dmg is a universal
# (arm64 + x86_64) binary, so any Mac released since 2020 works.

set -euo pipefail

REPO="MPJHorner/IdeUltra"
APP_NAME="IdeUltra"
INSTALL_DIR="/Applications"

# Pretty output, even when piped.
if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
  C_RESET='\033[0m'; C_DIM='\033[2m'; C_BLUE='\033[1;34m'
  C_GREEN='\033[1;32m'; C_RED='\033[1;31m'; C_BOLD='\033[1m'
else
  C_RESET=''; C_DIM=''; C_BLUE=''; C_GREEN=''; C_RED=''; C_BOLD=''
fi
info()  { printf '%b==>%b %s\n' "$C_BLUE"  "$C_RESET" "$*"; }
ok()    { printf '%b ✓%b %s\n' "$C_GREEN" "$C_RESET" "$*"; }
fail()  { printf '%b ✗%b %s\n' "$C_RED"   "$C_RESET" "$*" >&2; exit 1; }

# Sanity: macOS only.
if [ "$(uname -s)" != "Darwin" ]; then
  fail "IdeUltra is macOS-only right now. Build from source for Linux: https://github.com/${REPO}"
fi

# Sanity: hdiutil exists (it does on every Mac).
command -v hdiutil >/dev/null || fail "hdiutil not found — odd, this should be on every Mac."

info "Fetching latest release metadata from GitHub"
LATEST_JSON=$(curl -fsSL \
  -H 'Accept: application/vnd.github+json' \
  -H 'User-Agent: ideultra-installer' \
  "https://api.github.com/repos/${REPO}/releases/latest") || fail "Could not reach GitHub. Check your connection."

# Pull tag_name like "v0.23.1" → "0.23.1".
VERSION=$(printf '%s' "$LATEST_JSON" | sed -nE 's/.*"tag_name": *"v?([^"]+)".*/\1/p' | head -n1)
[ -n "$VERSION" ] || fail "Could not parse latest version from GitHub response."

DMG_URL=$(printf '%s' "$LATEST_JSON" \
  | grep -E '"browser_download_url"' \
  | grep -E '\.dmg"' \
  | head -n1 \
  | sed -E 's/.*"browser_download_url": *"([^"]+)".*/\1/')
[ -n "$DMG_URL" ] || fail "Latest release v${VERSION} has no .dmg asset. Try later or download manually."

ok "Latest release: v${VERSION}"
printf '%b    %s%b\n' "$C_DIM" "$DMG_URL" "$C_RESET"

WORK=$(mktemp -d -t ideultra-install)
trap 'rm -rf "$WORK"' EXIT
DMG_PATH="$WORK/IdeUltra.dmg"

info "Downloading the .dmg"
curl -fL --progress-bar "$DMG_URL" -o "$DMG_PATH" || fail "Download failed."
ok "Downloaded $(du -h "$DMG_PATH" | awk '{print $1}')"

# If a previous install is open, ask the user to quit it.
if pgrep -x "$APP_NAME" >/dev/null 2>&1; then
  info "$APP_NAME is currently running — please quit it first."
  fail "Re-run this installer after quitting."
fi

info "Mounting"
MOUNT_OUT=$(hdiutil attach -nobrowse -noautoopen -quiet "$DMG_PATH")
MOUNT_POINT=$(printf '%s' "$MOUNT_OUT" | awk -F'\t' 'NF>=3 { last=$NF } END { print last }')
[ -d "$MOUNT_POINT" ] || fail "Could not determine mount point."
trap 'hdiutil detach -quiet "$MOUNT_POINT" >/dev/null 2>&1 || true; rm -rf "$WORK"' EXIT

SRC_APP="$MOUNT_POINT/${APP_NAME}.app"
[ -d "$SRC_APP" ] || fail "The DMG does not contain ${APP_NAME}.app at the expected path."

info "Copying ${APP_NAME}.app to ${INSTALL_DIR}"
# Replace any existing install. /Applications usually doesn't need sudo
# for the current user; if it does, ask cleanly.
TARGET="${INSTALL_DIR}/${APP_NAME}.app"
if [ -w "$INSTALL_DIR" ]; then
  rm -rf "$TARGET"
  cp -R "$SRC_APP" "$TARGET"
else
  info "Need sudo to write to ${INSTALL_DIR}"
  sudo rm -rf "$TARGET"
  sudo cp -R "$SRC_APP" "$TARGET"
fi

info "Clearing Gatekeeper quarantine flag"
xattr -d com.apple.quarantine "$TARGET" 2>/dev/null || true

info "Ejecting the disk image"
hdiutil detach -quiet "$MOUNT_POINT" >/dev/null

ok "Installed IdeUltra v${VERSION} to ${TARGET}"
printf '\n'
printf '  %bLaunch it:%b\n' "$C_BOLD" "$C_RESET"
printf '    open -a IdeUltra\n\n'
printf '  %bOr from Spotlight:%b ⌘Space → IdeUltra\n\n' "$C_BOLD" "$C_RESET"
printf '  %bDocs:%b https://mpjhorner.github.io/IdeUltra/\n' "$C_BOLD" "$C_RESET"
printf '  %bIssues:%b https://github.com/%s/issues\n' "$C_BOLD" "$C_RESET" "$REPO"
