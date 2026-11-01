#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Builds the signed APT repository served at https://fotollection.org/apt (ADR-0035) from the
# latest GitHub release's .deb files, into the folder given (the website's _site/apt).
# Run by pages.yml on every website build; needs gh, apt-ftparchive and gpg, and the secret
# APT_GPG_PRIVATE_KEY (armored). Without the key, or before a release has a .deb, it builds
# nothing and the website goes out without the repository.
#
#   scripts/apt-repo.sh _site/apt
set -euo pipefail

out=${1:?usage: apt-repo.sh <output-dir>}
# Absolute now: the script works in a temporary folder below, and a relative path would
# then point inside it (LESSONS 2026-11-01).
mkdir -p "$out"
out=$(cd "$out" && pwd)
if [ -z "${APT_GPG_PRIVATE_KEY:-}" ]; then
  echo "apt-repo: no APT_GPG_PRIVATE_KEY, skipped"
  exit 0
fi

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/repo/pool/main/f/fotollection"
gh release download --repo "${GITHUB_REPOSITORY:-hoangphi01/fotollection}" \
  --pattern '*.deb' --dir "$work/repo/pool/main/f/fotollection" || true
if ! ls "$work"/repo/pool/main/f/fotollection/*.deb >/dev/null 2>&1; then
  echo "apt-repo: the latest release has no .deb yet, skipped"
  exit 0
fi

cd "$work/repo"
for arch in amd64 arm64; do
  dir="dists/stable/main/binary-$arch"
  mkdir -p "$dir"
  apt-ftparchive --arch "$arch" packages pool > "$dir/Packages"
  gzip -9kn "$dir/Packages"
done
apt-ftparchive \
  -o APT::FTPArchive::Release::Origin=Fotollection \
  -o APT::FTPArchive::Release::Label=Fotollection \
  -o APT::FTPArchive::Release::Suite=stable \
  -o APT::FTPArchive::Release::Codename=stable \
  -o APT::FTPArchive::Release::Architectures="amd64 arm64" \
  -o APT::FTPArchive::Release::Components=main \
  -o APT::FTPArchive::Release::Description="Fotollection, fast private offline photo-print preparation" \
  release dists/stable > "$work/Release"
mv "$work/Release" dists/stable/Release

export GNUPGHOME="$work/gnupg"
mkdir -m 700 "$GNUPGHOME"
printf '%s\n' "$APT_GPG_PRIVATE_KEY" | gpg --batch --quiet --import
gpg --batch --yes --clearsign -o dists/stable/InRelease dists/stable/Release
gpg --batch --yes --armor --detach-sign -o dists/stable/Release.gpg dists/stable/Release
# The key apt checks with (`signed-by`), in the binary form /usr/share/keyrings expects.
gpg --batch --export > fotollection.gpg

cp -R . "$out/"
echo "apt-repo: $(find pool -name '*.deb' | wc -l | tr -d ' ') packages signed into $out"
