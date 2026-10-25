#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Prepares a release commit (docs/RELEASE.md §4, ADR-0021). Pushing it to `main` releases.
#
#   scripts/release.sh               # proposes the next version from the commits since the last tag
#   scripts/release.sh 0.2.0         # or name it (X.Y.Z or X.Y.Z-beta.N)
#   scripts/release.sh 0.2.0 --dry-run
#
# Sets the version in Cargo.toml, Cargo.lock, ui/package.json and package-lock.json, turns the
# CHANGELOG's `## [Unreleased]` into `## [X.Y.Z] - date`, and commits. It never pushes.
# Portable to macOS bash 3.2.
set -euo pipefail
cd "$(dirname "$0")/.."

version=""
dry_run=false
for arg in "$@"; do
  case "$arg" in
    --dry-run) dry_run=true ;;
    -*) echo "unknown option: $arg" >&2; exit 2 ;;
    *) version="$arg" ;;
  esac
done

current=$(sed -nE 's/^version = "([^"]+)"$/\1/p' Cargo.toml | head -n 1)
last_tag=$(git describe --tags --abbrev=0 --match 'v*' 2>/dev/null || true)

if [ -z "$version" ]; then
  # A `feat` since the last tag means MINOR, otherwise PATCH (pre-release suffixes dropped).
  base="${last_tag#v}"
  [ -n "$base" ] || base="$current"
  base="${base%%-*}"
  IFS=. read -r major minor patch <<EOF
$base
EOF
  range="HEAD"
  [ -n "$last_tag" ] && range="$last_tag..HEAD"
  feats=$(git log --format=%s "$range" | grep -cE '^feat(\(|!|:)' || true)
  if [ "$feats" -gt 0 ]; then
    version="$major.$((minor + 1)).0"
  else
    version="$major.$minor.$((patch + 1))"
  fi
  echo "Proposed version: $version (current $current, last tag ${last_tag:-none})"
  if [ "$dry_run" = false ]; then
    printf 'Release v%s? [y/N] ' "$version"
    read -r answer
    [ "$answer" = "y" ] || [ "$answer" = "Y" ] || { echo "cancelled"; exit 1; }
  fi
fi

echo "$version" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+(-(alpha|beta|rc)\.[0-9]+)?$' \
  || { echo "not a release version: $version" >&2; exit 2; }
git rev-parse -q --verify "refs/tags/v$version" >/dev/null && { echo "tag v$version already exists" >&2; exit 1; }
grep -q '^## \[Unreleased\]' CHANGELOG.md || { echo "CHANGELOG.md has no [Unreleased] section" >&2; exit 1; }
# The Unreleased section must say something.
awk '/^## \[Unreleased\]/{on=1; next} /^## \[/{on=0} on && /^- /{found=1} END{exit !found}' CHANGELOG.md \
  || { echo "CHANGELOG.md [Unreleased] is empty" >&2; exit 1; }
# The release page shows only the Highlights (RELEASE.md §4), so they must be there.
awk '/^## \[Unreleased\]/{on=1; next} /^## \[/{on=0} on && /^### Highlights/{found=1} END{exit !found}' CHANGELOG.md \
  || { echo "CHANGELOG.md [Unreleased] has no ### Highlights" >&2; exit 1; }

if [ "$dry_run" = false ]; then
  [ "$(git rev-parse --abbrev-ref HEAD)" = "main" ] || { echo "not on main" >&2; exit 1; }
  # Untracked files never reach the commit, so only tracked changes block a release.
  [ -z "$(git status --porcelain --untracked-files=no)" ] || { echo "working tree is not clean" >&2; exit 1; }
fi

today=$(date +%Y-%m-%d)
echo "v$current → v$version ($today)"
if [ "$dry_run" = true ]; then
  echo "dry run: would set the version in Cargo.toml, Cargo.lock, ui/package.json, package-lock.json"
  echo "dry run: would add '## [$version] - $today' to CHANGELOG.md and commit 'chore(release): v$version'"
  exit 0
fi

sed -i.bak -E "s/^version = \"$current\"$/version = \"$version\"/" Cargo.toml && rm Cargo.toml.bak
cargo update --workspace --quiet
npm version "$version" --workspace ui --no-git-tag-version --allow-same-version >/dev/null
awk -v v="$version" -v d="$today" '
  /^## \[Unreleased\]/ && !done { print; print ""; print "## [" v "] - " d; done=1; next }
  { print }
' CHANGELOG.md >CHANGELOG.md.tmp && mv CHANGELOG.md.tmp CHANGELOG.md

scripts/check-docs.sh >/dev/null
git add Cargo.toml Cargo.lock ui/package.json package-lock.json CHANGELOG.md
git commit -q -m "chore(release): v$version"
echo "Committed chore(release): v$version. Push to main to build and publish it."
