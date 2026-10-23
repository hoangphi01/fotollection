# ADR-0006: Docs-as-code with requirement traceability

- **Status:** Accepted
- **Date:** 2026-10-23
- **Related:** [CONTRIBUTING.md](../../CONTRIBUTING.md), [STYLE_RULE.md §6](../STYLE_RULE.md#6-documentation)

## Context

Documentation that lives apart from code drifts quickly. This project is developed with heavy AI assistance, which makes it even more important that the docs are trustworthy: they are the context every change starts from.

## Decision

- Docs live in the repository and change **in the same commit or PR** as the code they describe.
- Every requirement has a stable ID (`FR-NNN` / `NFR-NNN`) defined in [PRD.md](../PRD.md). The same ID appears in [ROADMAP.md](../ROADMAP.md), [STATUS.md](../STATUS.md), test names (`fr_NNN_*`), and commit messages.
- Each fact has exactly one home ([docs/README.md](../README.md#where-does-each-fact-live)).
- CI (`.github/workflows/docs.yml`) runs markdownlint, a link check, and `scripts/check-docs.sh` for ID consistency.
- The PR template's Definition of Done includes the docs updates.

## Consequences

- A small overhead on every change, in exchange for docs that can be trusted.
- `check-docs.sh` grows over time. Once code exists, it will also verify that every `Done` FR has a test.
