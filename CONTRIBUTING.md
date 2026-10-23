# Contributing to Fotollection

Thanks for helping! Please read [docs/README.md](docs/README.md) and [docs/STYLE_RULE.md](docs/STYLE_RULE.md) first.

## Workflow

1. **Find or create the requirement.** Every feature and bug fix traces to an `FR-`/`NFR-` ID in [docs/PRD.md](docs/PRD.md). For new scope, propose a PRD change first.
2. **Branch:** `feat/<name>`, `fix/<name>`, `docs/<name>`, …
3. **Commit:** use [Conventional Commits](https://www.conventionalcommits.org/) with the ID, for example `fix(crop): keep portrait ratio on rotated input [FR-006]`.
4. **Open a PR** using the template. Keep PRs focused on one concern.

## Definition of Done

A change is done when **all** of the following hold:

- [ ] Code follows [STYLE_RULE.md](docs/STYLE_RULE.md). Formatters and linters pass.
- [ ] Tests are added or updated. Tests tied to a requirement are named `fr_NNN_*` ([TESTING.md](docs/TESTING.md)).
- [ ] **Docs are updated in the same PR:**
  - Behaviour or scope → [PRD.md](docs/PRD.md) / [ROADMAP.md](docs/ROADMAP.md)
  - Architecture, stack or IPC → [ARCHITECTURE.md](docs/ARCHITECTURE.md)
  - Performance-relevant change → a benchmark row in [OPTIMIZATION.md §5](docs/OPTIMIZATION.md#5-benchmark-log)
  - Security-relevant change → [SECURITY.md](docs/SECURITY.md)
  - Significant decision → a new ADR ([template](docs/adr/0000-template.md))
  - Progress → [STATUS.md](docs/STATUS.md)
- [ ] A [CHANGELOG.md](CHANGELOG.md) entry is added under `Unreleased` if the change is user-visible.
- [ ] `scripts/check-docs.sh` passes and CI is green.

## Running the docs checks locally

```sh
scripts/check-docs.sh
npx markdownlint-cli2 "**/*.md"
```

## Reporting security issues

Please use a private GitHub security advisory rather than a public issue. See [docs/SECURITY.md](docs/SECURITY.md).
