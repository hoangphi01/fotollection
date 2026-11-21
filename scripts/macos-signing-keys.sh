#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# CI only (release.yml, installers.yml; ADR-0061): puts the Developer ID certificate into a
# keychain of this job and the notarization key into a file, then names the file in
# $GITHUB_ENV as APPLE_API_KEY_PATH. Reads APPLE_CERTIFICATE (the .p12, base64),
# APPLE_CERTIFICATE_PASSWORD and P8 (the App Store Connect API key's .p8 text).
set -euo pipefail

for v in APPLE_CERTIFICATE APPLE_CERTIFICATE_PASSWORD P8 RUNNER_TEMP GITHUB_ENV; do
  [ -n "${!v:-}" ] || { echo "macos-signing-keys: $v is not set" >&2; exit 1; }
done

keychain="$RUNNER_TEMP/signing.keychain-db"
pass=$(openssl rand -base64 24)
p12="$RUNNER_TEMP/cert.p12"
printf '%s' "$APPLE_CERTIFICATE" | base64 --decode > "$p12"

security create-keychain -p "$pass" "$keychain"
security set-keychain-settings -lut 21600 "$keychain"
security unlock-keychain -p "$pass" "$keychain"
security import "$p12" -k "$keychain" -P "$APPLE_CERTIFICATE_PASSWORD" -T /usr/bin/codesign
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$pass" "$keychain" >/dev/null
# shellcheck disable=SC2046 # the current list is meant to split
security list-keychains -d user -s "$keychain" $(security list-keychains -d user | tr -d '"')
rm -f "$p12"
security find-identity -v -p codesigning "$keychain"

key="$RUNNER_TEMP/notary.p8"
printf '%s\n' "$P8" > "$key"
echo "APPLE_API_KEY_PATH=$key" >> "$GITHUB_ENV"
