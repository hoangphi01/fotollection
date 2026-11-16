# ADR-0052: Japanese text top to bottom by the face's own vertical forms, no shaper

- **Status:** Accepted. Amended by FR-134 (2026-11-16): GSUB read through read-fonts
- **Date:** 2026-11-15
- **Related:** FR-130, ADR-0044

## Context

The はがき's upright address side is written top to bottom (縦書き, PRD FR-130): each character one em down its column, the columns right to left, and the marks that read differently in a column (、。「」ー) turned or moved. The core draws text with `ab_glyph`, one character after another along a line, with no shaping and no vertical metrics. Klee One, the bundled print face, carries its vertical forms as GSUB `vert` and `vrt2` single substitutions; `ttf-parser` 0.25 (already under `ab_glyph`) reads GSUB.

## Options considered

1. **A shaper (rustybuzz, swash).** Handles every OpenType feature and script; but a new dependency tree for one feature, and Japanese vertical text needs only single substitutions and a fixed one-em advance.
2. **Rotate or replace the marks by hand** (a table of codepoints, U+FE10–FE19 presentation forms). No font reading; but the forms are the face's design, and a hand table drifts from the face and misses marks it never listed.
3. **Read the face's `vert`/`vrt2` substitutions once with `ttf-parser`, place characters on an em grid, break lines by 禁則.** Uses the face's own forms, no new crate (ttf-parser is in the tree already), a few dozen lines.

## Decision

Option 3: `crates/core/src/vertical.rs`. The face's horizontal → vertical glyph map is read once (`LazyLock`); a character takes its vertical form when it has one; each cell is one em tall, its em box's top at the face's typographic ascender (`OS/2`), the scale made em-true (ab_glyph's `PxScale` is the line's height, LESSONS 2026-11-15); 禁則 pulls the character before a mark that may not start a column (、。」ー, small kana) along with it, hangs 、 and 。 below a full column, and moves an opening bracket off a column's foot. Digits and Latin letters stand upright, one to a cell; two digits across one cell (縦中横) is later.

## Consequences

- The はがき prints with the face's own vertical punctuation and long vowel mark, tested on Klee One itself (`fr_130_brackets_and_the_long_mark_turn_in_a_column`).
- No shaper: features that need contextual or positioning lookups (proportional vertical kana, `vpal`) are not applied; a Japanese address does not need them.
- A user font without `vert` falls back to its horizontal forms; the bundled face always has them.
