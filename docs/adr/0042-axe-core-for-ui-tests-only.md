# ADR-0042: axe-core for the UI tests only, an MPL-2.0 tool never shipped

- **Status:** Accepted (owner, 2026-11-07)
- **Date:** 2026-11-07
- **Related:** NFR-007, ADR-0008, [UI_TESTS.md](../UI_TESTS.md)

## Context

The owner asked for every UI and UX case to be tested, mostly by a machine, after finding small faults by eye one after another. WCAG 2.2 has many checks a person cannot run on every view in two languages and two themes: accessible names, roles, `aria-*` values, colour contrast. The standard tool for them is Deque's axe-core, licensed **MPL-2.0**. The project's rule (CLAUDE.md, ADR-0008) admits only permissive licences such as MIT, Apache-2.0 and BSD, so that the code stays relicensable.

## Decision

- **axe-core and `@axe-core/playwright` are allowed as development dependencies of the UI tests only** (the root `package.json`, used by `site/screenshots/checks.mjs`). They are never bundled into the app, the installers or the website, and no app code imports them.
- MPL-2.0 is a file-level copyleft: it would bind changes to axe-core's own files, which this project neither changes nor ships. The app's licence and its relicensing option are untouched.
- Every other dependency keeps the rule. A second exception needs its own ADR.

## Consequences

- `scripts/ui-check.sh` reports WCAG 2.2 A and AA violations (UI-A11Y-002, UI-THEME-001) in every view and state it opens.
- The README's licence notes name axe-core as a test-only tool.
- If the licence rule ever has to cover development tools too, the axe checks are replaced by our own (contrast, names), and UI_TESTS.md marks those rows **P** until then.
