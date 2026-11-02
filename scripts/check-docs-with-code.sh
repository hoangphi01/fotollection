#!/usr/bin/env bash
# Docs move with code (ADR-0006, DOCS_GUIDE.md): every commit that changes the app's code also
# changes a doc, the CHANGELOG or a README, unless its message says `[no-docs]` with a reason.
# Portable to macOS bash 3.2.
#
#   scripts/check-docs-with-code.sh              # the working tree's changes, else the last commit
#   scripts/check-docs-with-code.sh A..B         # every commit in a range (CI)
set -euo pipefail
cd "$(dirname "$0")/.."

code='^(crates|src-tauri/src|ui/src)/'
docs='^(docs/|CHANGELOG\.md|README(\.vi)?\.md|site/)'
# Generated or mechanical files that never need a doc of their own.
ignore='^(ui/src/ipc/bindings\.ts|crates/core/tests/fixtures/)'

check() { # $1: a label, then the changed files on stdin
  files=$(cat)
  touched=$(printf '%s\n' "$files" | grep -E "$code" | grep -vE "$ignore" || true)
  [ -n "$touched" ] || return 0
  printf '%s\n' "$files" | grep -qE "$docs" && return 0
  echo "  ✗ $1 changes code but no doc, CHANGELOG or README (DOCS_GUIDE.md):"
  printf '%s\n' "$touched" | head -5 | sed 's/^/      /'
  return 1
}

status=0
if [ $# -gt 0 ]; then
  for c in $(git rev-list --no-merges "$1"); do
    git log -1 --format=%B "$c" | grep -q '\[no-docs\]' && continue
    git diff-tree --no-commit-id --name-only -r "$c" | check "commit $(git log -1 --format='%h %s' "$c" | cut -c1-70)" || status=1
  done
elif [ -n "$(git status --porcelain --untracked-files=no)" ]; then
  { git diff --name-only HEAD; git ls-files --others --exclude-standard; } | check "the working tree" || status=1
else
  c=$(git rev-parse HEAD)
  if ! git log -1 --format=%B "$c" | grep -q '\[no-docs\]' && [ "$(git rev-list --parents -n 1 "$c" | wc -w)" -le 2 ]; then
    git diff-tree --no-commit-id --name-only -r "$c" | check "the last commit" || status=1
  fi
fi
exit $status
