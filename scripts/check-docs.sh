#!/usr/bin/env bash
# Consistency checks for the docs-as-code framework (see docs/adr/0006-docs-as-code.md).
# Portable to macOS bash 3.2. Exits non-zero on any failure.
set -euo pipefail

cd "$(dirname "$0")/.."

PRD=docs/PRD.md
ROADMAP=docs/ROADMAP.md
STATUS=docs/STATUS.md
ADR_INDEX=docs/adr/README.md
DOCS_INDEX=docs/README.md

failures=0
fail() {
  echo "  ✗ $1"
  failures=$((failures + 1))
}

# Requirement IDs are defined only by `### FR-NNN — Title` headings in the PRD.
defined_ids() {
  grep -oE '^### (FR|NFR)-[0-9]{3}' "$PRD" | sed 's/^### //'
}

echo "• Requirement IDs are unique in PRD"
dupes=$(defined_ids | sort | uniq -d)
for id in $dupes; do fail "$id is defined more than once in $PRD"; done

echo "• Every FR is scheduled in ROADMAP and tracked in STATUS"
for id in $(defined_ids | grep '^FR-'); do
  count=$(grep -cE "^\| $id \|" "$ROADMAP" || true)
  [ "$count" -eq 1 ] || fail "$id appears $count times in $ROADMAP table (expected exactly 1)"
  grep -qE "^\| $id \|" "$STATUS" || fail "$id has no row in $STATUS"
done

echo "• Every NFR is tracked in STATUS"
for id in $(defined_ids | grep '^NFR-'); do
  grep -qE "^\| $id \|" "$STATUS" || fail "$id has no row in $STATUS"
done

echo "• No orphan requirement IDs referenced anywhere"
referenced=$(grep -rhoE '\b(FR|NFR)-[0-9]{3}\b' --include='*.md' . | sort -u)
for id in $referenced; do
  defined_ids | grep -qx "$id" || fail "$id is referenced but not defined in $PRD"
done

echo "• Every ADR file is listed in the ADR index"
for f in docs/adr/[0-9][0-9][0-9][0-9]-*.md; do
  name=$(basename "$f")
  [ "$name" = "0000-template.md" ] && continue
  grep -q "($name)" "$ADR_INDEX" || fail "$name is not listed in $ADR_INDEX"
done

echo "• Every top-level doc is linked from the docs index"
for f in docs/*.md; do
  name=$(basename "$f")
  [ "$name" = "README.md" ] && continue
  grep -q "($name)" "$DOCS_INDEX" || fail "$name is not linked from $DOCS_INDEX"
done

echo "• Every doc has a status/last-reviewed header"
for f in docs/*.md; do
  [ "$(basename "$f")" = "README.md" ] && continue
  head -n 5 "$f" | grep -qE 'Last (reviewed|updated):' || fail "$f lacks a 'Last reviewed:' header"
done

echo "• README.vi.md mirrors README.md (same sections, same charts)"
if [ -f README.vi.md ]; then
  en_sections=$(grep -c '^## ' README.md)
  vi_sections=$(grep -c '^## ' README.vi.md)
  [ "$en_sections" -eq "$vi_sections" ] || fail "README.md has $en_sections sections, README.vi.md has $vi_sections"
  for img in $(grep -oE 'docs/images/bench[a-z-]*\.svg' README.md | sort -u); do
    vi_img="${img%.svg}-vi.svg"
    grep -q "$vi_img" README.vi.md || fail "README.vi.md does not show $vi_img (README.md shows $img)"
  done
  grep -q 'README.vi.md' README.md || fail "README.md has no link to README.vi.md"
else
  fail "README.vi.md is missing"
fi

echo "• Chart budgets match OPTIMIZATION.md §3"
for id in NFR-001 NFR-002 NFR-003 NFR-004; do
  text=$(grep -oE "\"$id\": \{[^}]*\"text\": \"[^\"]*\"" bench/results/budgets.json | sed -E 's/.*"text": "([^"]*)"/\1/')
  [ -n "$text" ] || { fail "bench/results/budgets.json has no text for $id"; continue; }
  grep -E "^\| \[$id\]" docs/OPTIMIZATION.md | grep -qF "$text" || fail "$id budget '$text' in bench/results/budgets.json does not match docs/OPTIMIZATION.md §3"
done

if [ "$failures" -gt 0 ]; then
  echo "check-docs: $failures problem(s) found"
  exit 1
fi
echo "check-docs: all checks passed"
