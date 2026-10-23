#!/usr/bin/env bash
# Set up a fresh macOS or Linux machine to build Fotollection. See docs/DEVELOPMENT.md §1.
#
# Usage: scripts/bootstrap.sh [--install] [--tools]
#   --install  install missing prerequisites (rustup; Linux system packages via apt)
#   --tools    also install optional dev tools (cargo-sweep, cargo-deny)
# Versions come from the repo: rust-toolchain.toml, .node-version, Cargo.lock, package-lock.json.
set -euo pipefail

cd "$(dirname "$0")/.."

install=0
tools=0
for arg in "$@"; do
  case "$arg" in
    --install) install=1 ;;
    --tools) tools=1 ;;
    *) echo "unknown option: $arg" >&2; exit 2 ;;
  esac
done

missing=0
ok() { echo "  ✓ $1"; }
need() { echo "  ✗ $1"; missing=1; }

echo "• System prerequisites"
command -v git >/dev/null && ok "git" || need "git"

case "$(uname -s)" in
  Darwin)
    if xcode-select -p >/dev/null 2>&1; then ok "Xcode Command Line Tools"; else
      need "Xcode Command Line Tools — run: xcode-select --install"
    fi
    ;;
  Linux)
    # Tauri 2 Linux build dependencies (Debian/Ubuntu names).
    pkgs="build-essential curl wget file libwebkit2gtk-4.1-dev libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev"
    if command -v dpkg >/dev/null; then
      absent=""
      for p in $pkgs; do dpkg -s "$p" >/dev/null 2>&1 || absent="$absent $p"; done
      if [ -z "$absent" ]; then ok "Tauri system packages"
      elif [ "$install" -eq 1 ]; then sudo apt-get update && sudo apt-get install -y $absent && ok "Tauri system packages"
      else need "packages:$absent — rerun with --install or apt-get install them"
      fi
    else
      echo "  ? non-Debian distro: install the Tauri prerequisites from https://v2.tauri.app/start/prerequisites/"
    fi
    ;;
esac

echo "• Rust"
if ! command -v rustup >/dev/null; then
  if [ "$install" -eq 1 ]; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
    # shellcheck disable=SC1091
    . "$HOME/.cargo/env"
  else
    need "rustup — rerun with --install or see https://rustup.rs"
  fi
fi
if command -v rustup >/dev/null; then
  rustup show active-toolchain >/dev/null 2>&1 || rustup toolchain install   # reads rust-toolchain.toml
  ok "$(rustc --version)"
fi

echo "• Node.js"
if command -v node >/dev/null; then
  major=$(node -p 'process.versions.node.split(".")[0]')
  if [ "$major" -ge 22 ]; then ok "node $(node --version) (recommended: $(cat .node-version) LTS)"; else
    need "node >= 22 (found $(node --version)); recommended $(cat .node-version) via fnm/nvm"
  fi
else
  need "node >= 22 — install via https://nodejs.org or fnm/nvm (version in .node-version)"
fi

if [ "$missing" -eq 1 ]; then
  echo "bootstrap: prerequisites missing (see ✗ above)"
  exit 1
fi

echo "• Project dependencies (from lockfiles)"
npm ci
cargo fetch --locked

if [ "$tools" -eq 1 ]; then
  echo "• Optional dev tools"
  cargo install --locked cargo-sweep cargo-deny
fi

echo "bootstrap: done — run 'npm run dev' to start the app"
