#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Signs, notarizes and staples a macOS .dmg, then checks that Gatekeeper accepts it and the app
# inside it (docs/RELEASE.md §3, ADR-0061). Tauri signs and notarizes the .app; the .dmg around
# it is notarized here so a downloaded disk image opens without a warning.
#
#   scripts/notarize-dmg.sh path/to/Fotollection.dmg
#
# Reads APPLE_SIGNING_IDENTITY, APPLE_API_KEY (the key's ID), APPLE_API_ISSUER and
# APPLE_API_KEY_PATH (the .p8 file), the names Tauri reads.
set -euo pipefail

dmg="${1:?usage: scripts/notarize-dmg.sh path/to/app.dmg}"
for v in APPLE_SIGNING_IDENTITY APPLE_API_KEY APPLE_API_ISSUER APPLE_API_KEY_PATH; do
  [ -n "${!v:-}" ] || { echo "notarize-dmg: $v is not set" >&2; exit 1; }
done

codesign --force --timestamp --sign "$APPLE_SIGNING_IDENTITY" "$dmg"
xcrun notarytool submit "$dmg" --key "$APPLE_API_KEY_PATH" --key-id "$APPLE_API_KEY" \
  --issuer "$APPLE_API_ISSUER" --wait --timeout 30m
xcrun stapler staple "$dmg"

# Gatekeeper's own verdicts: the disk image, then the app inside it.
spctl --assess --type open --context context:primary-signature --verbose=2 "$dmg"
mnt=$(mktemp -d)
hdiutil attach -nobrowse -readonly -mountpoint "$mnt" "$dmg" >/dev/null
trap 'hdiutil detach "$mnt" -quiet || true' EXIT
app=$(find "$mnt" -maxdepth 1 -name '*.app' -print -quit)
[ -n "$app" ] || { echo "notarize-dmg: no .app inside $dmg" >&2; exit 1; }
codesign --verify --deep --strict --verbose=2 "$app"
xcrun stapler validate "$app"
spctl --assess --type execute --verbose=2 "$app"
