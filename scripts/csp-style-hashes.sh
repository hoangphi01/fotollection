#!/usr/bin/env bash
# Prints the CSP hashes for the static <style> tags Radix injects (docs/SECURITY.md §2).
# Re-run after upgrading radix-ui and update style-src in src-tauri/tauri.conf.json.
set -euo pipefail
cd "$(dirname "$0")/.."
for pkg in react-select react-scroll-area; do
  css=$(grep -o '__html: `[^`]*`' "node_modules/@radix-ui/$pkg/dist/index.mjs" | head -1 | sed 's/^__html: `//; s/`$//')
  printf "%s 'sha256-%s'\n" "$pkg" "$(printf '%s' "$css" | openssl dgst -sha256 -binary | base64)"
done
