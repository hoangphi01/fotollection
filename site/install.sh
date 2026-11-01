#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-only
# Fotollection for Linux (ADR-0034, ADR-0035): https://fotollection.org/install.sh
#
#   curl -fsSL https://fotollection.org/install.sh | sh                  install or update
#   curl -fsSL https://fotollection.org/install.sh | sh -s -- --appimage  the AppImage, even on Debian/Ubuntu
#   curl -fsSL https://fotollection.org/install.sh | sh -s -- --uninstall
#
# Ubuntu, Debian, Mint and their family get the signed APT repository at fotollection.org/apt
# (needs sudo once; updates then come with `sudo apt upgrade`). Every other 64-bit Linux with
# glibc 2.35 or later gets the AppImage from GitHub, checked against SHA256SUMS.txt, in
# ~/.local without sudo. Read this script before running it: it is short on purpose.
set -eu

REPO="https://github.com/hoangphi01/fotollection"
SITE="https://fotollection.org"
KEYRING="/usr/share/keyrings/fotollection.gpg"
LIST="/etc/apt/sources.list.d/fotollection.list"
DATA="${XDG_DATA_HOME:-$HOME/.local/share}"
APPIMAGE_DIR="$DATA/fotollection"
BIN="$HOME/.local/bin"
DESKTOP="$DATA/applications/fotollection.desktop"
ICON="$DATA/icons/hicolor/256x256/apps/fotollection.png"

say() { printf '%s\n' "$*"; }
fail() { printf 'fotollection: %s\n' "$*" >&2; exit 1; }
have() { command -v "$1" >/dev/null 2>&1; }

sudo_cmd() {
  if [ "$(id -u)" -eq 0 ]; then "$@"; elif have sudo; then sudo "$@"; else fail "this step needs root: install sudo or run as root"; fi
}

mode=auto
for arg in "$@"; do
  case "$arg" in
    --appimage) mode=appimage ;;
    --uninstall) mode=uninstall ;;
    -h|--help) sed -n '3,12p' "$0" 2>/dev/null || true; exit 0 ;;
    *) fail "unknown option: $arg" ;;
  esac
done

[ "$(uname -s)" = Linux ] || fail "this installer is for Linux; get the macOS and Windows versions at $SITE"

uninstall() {
  if have dpkg && dpkg -s fotollection >/dev/null 2>&1; then
    sudo_cmd apt-get remove -y fotollection
  fi
  [ -f "$LIST" ] && sudo_cmd rm -f "$LIST"
  [ -f "$KEYRING" ] && sudo_cmd rm -f "$KEYRING"
  rm -f "$BIN/fotollection" "$DESKTOP" "$ICON"
  rm -rf "$APPIMAGE_DIR"
  say "Fotollection is removed. Your settings and projects stay in ~/.config and ~/.local/share."
}

if [ "$mode" = uninstall ]; then uninstall; exit 0; fi

have curl || fail "curl is needed"

case "$(uname -m)" in
  x86_64|amd64) deb_arch=amd64; appimage_arch=amd64 ;;
  aarch64|arm64) deb_arch=arm64; appimage_arch=aarch64 ;;
  *) fail "Fotollection runs on 64-bit x86 and ARM only (this is $(uname -m))" ;;
esac

# glibc 2.35 or later (Ubuntu 22.04, Debian 12, Fedora 36…); musl systems such as Alpine cannot run it.
glibc=$(ldd --version 2>/dev/null | head -n 1 | grep -o '[0-9]\+\.[0-9]\+$' || true)
[ -n "$glibc" ] || fail "glibc not found (musl systems such as Alpine are not supported)"
major=${glibc%%.*}
minor=${glibc#*.}
if [ "$major" -lt 2 ] || { [ "$major" -eq 2 ] && [ "$minor" -lt 35 ]; }; then
  fail "glibc $glibc is too old: Fotollection needs 2.35 or later (Ubuntu 22.04, Debian 12 or newer)"
fi

debian_family() {
  [ -r /etc/os-release ] || return 1
  # shellcheck disable=SC1091
  . /etc/os-release
  case " ${ID:-} ${ID_LIKE:-} " in *" debian "*|*" ubuntu "*) have apt-get ;; *) return 1 ;; esac
}

install_apt() {
  say "Setting up the Fotollection APT repository (signed; sudo asks for your password)…"
  tmp=$(mktemp)
  curl -fsSL "$SITE/apt/fotollection.gpg" -o "$tmp"
  sudo_cmd install -m 644 "$tmp" "$KEYRING"
  rm -f "$tmp"
  echo "deb [arch=$deb_arch signed-by=$KEYRING] $SITE/apt stable main" | sudo_cmd tee "$LIST" >/dev/null
  sudo_cmd apt-get update -o Dir::Etc::sourcelist="$LIST" -o Dir::Etc::sourceparts=- -o APT::Get::List-Cleanup=0
  sudo_cmd apt-get install -y fotollection
  say "Done: Fotollection is in your applications menu. Updates come with: sudo apt upgrade"
}

install_appimage() {
  # The latest release's tag, from GitHub's redirect: no API call, no JSON to parse.
  latest=$(curl -fsSLI -o /dev/null -w '%{url_effective}' "$REPO/releases/latest")
  version=${latest##*/v}
  case "$version" in [0-9]*.[0-9]*.[0-9]*) ;; *) fail "could not find the latest version" ;; esac
  file="Fotollection_${version}_${appimage_arch}.AppImage"
  tmp=$(mktemp -d)
  trap 'rm -rf "$tmp"' EXIT
  say "Downloading Fotollection $version ($appimage_arch)…"
  curl -fL --progress-bar "$REPO/releases/download/v$version/$file" -o "$tmp/$file"
  curl -fsSL "$REPO/releases/download/v$version/SHA256SUMS.txt" -o "$tmp/SHA256SUMS.txt"
  if have sha256sum; then
    (cd "$tmp" && grep " $file\$" SHA256SUMS.txt | sha256sum -c -) || fail "the download does not match SHA256SUMS.txt"
  else
    say "sha256sum not found: the checksum was not verified"
  fi
  mkdir -p "$APPIMAGE_DIR" "$BIN" "$(dirname "$DESKTOP")" "$(dirname "$ICON")"
  install -m 755 "$tmp/$file" "$APPIMAGE_DIR/Fotollection.AppImage"
  ln -sf "$APPIMAGE_DIR/Fotollection.AppImage" "$BIN/fotollection"
  curl -fsSL "$SITE/assets/app-icon-256.png" -o "$ICON" || true
  cat >"$DESKTOP" <<EOF
[Desktop Entry]
Type=Application
Name=Fotollection
Comment=Fast, private, offline photo-print preparation
Exec=$APPIMAGE_DIR/Fotollection.AppImage %F
Icon=fotollection
Categories=Graphics;Photography;
MimeType=image/jpeg;image/png;image/x-adobe-dng;image/x-canon-cr2;image/x-canon-cr3;image/x-nikon-nef;image/x-sony-arw;image/x-fuji-raf;image/x-olympus-orf;image/x-panasonic-rw2;application/vnd.fotollection.project+zip;
Terminal=false
EOF
  have update-desktop-database && update-desktop-database "$(dirname "$DESKTOP")" >/dev/null 2>&1 || true
  # AppImages need FUSE 2, which Ubuntu 24.04 and other new systems no longer install.
  if have ldconfig && ! ldconfig -p 2>/dev/null | grep -q 'libfuse\.so\.2'; then
    say "Note: FUSE 2 is missing. Install it (Ubuntu/Debian: sudo apt install libfuse2t64; Fedora: sudo dnf install fuse-libs),"
    say "      or start the app with: fotollection --appimage-extract-and-run"
  fi
  say "Done: Fotollection $version is in your applications menu, and 'fotollection' in $BIN."
  case ":$PATH:" in *":$BIN:"*) ;; *) say "Add $BIN to your PATH to start it from a terminal." ;; esac
}

if [ "$mode" = auto ] && debian_family; then install_apt; else install_appimage; fi
