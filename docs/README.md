# Fotollection Documentation

> Start here. Every document has one job. If you are looking for a fact, the table below tells you where it lives.

## Reading order

| If you are… | Read |
|-------------|------|
| New to the project | [PRD](PRD.md) → [ARCHITECTURE](ARCHITECTURE.md) → [ROADMAP](ROADMAP.md) → [STATUS](STATUS.md) |
| About to write code | [DEVELOPMENT](DEVELOPMENT.md) → [STYLE_RULE](STYLE_RULE.md) → [TESTING](TESTING.md) → [CONTRIBUTING](../CONTRIBUTING.md) |
| Working on performance | [RESULT](RESULT.md) → [OPTIMIZATION](OPTIMIZATION.md) |
| Touching file input, IPC or the webview | [SECURITY](SECURITY.md) |
| Changing anything that is saved to disk | [DATABASE](DATABASE.md) → [EXTENSION](EXTENSION.md) |
| Changing anything (what else to update) | [DOCS_GUIDE](DOCS_GUIDE.md) |
| Shipping a build | [RELEASE](RELEASE.md) |
| Writing the manual or the website's help | [USER_GUIDE](USER_GUIDE.md) |
| Confused by a term | [GLOSSARY](GLOSSARY.md) |

## Where does each fact live?

| Fact | Single source of truth |
|------|------------------------|
| What the product does and why; requirement IDs | [PRD.md](PRD.md) |
| Which phase a requirement belongs to | [ROADMAP.md](ROADMAP.md) |
| Whether a requirement or task is done | [STATUS.md](STATUS.md) |
| System structure, tech stack, IPC contract | [ARCHITECTURE.md](ARCHITECTURE.md) |
| Persisted data: storage locations, schemas, migrations, cache | [DATABASE.md](DATABASE.md) |
| File types: extensions, MIME/UTI, recognition, versions, limits, compatibility rules | [EXTENSION.md](EXTENSION.md) |
| Why a significant decision was made | [adr/](adr/README.md) |
| Performance methodology, design techniques, benchmark history | [OPTIMIZATION.md](OPTIMIZATION.md) |
| Latest measured speed and memory, in one readable page | [RESULT.md](RESULT.md) |
| Code, UI, git and docs conventions | [STYLE_RULE.md](STYLE_RULE.md) |
| Toolchain setup, build commands, build disk cleanup | [DEVELOPMENT.md](DEVELOPMENT.md) |
| Threat model, defenses, error codes | [SECURITY.md](SECURITY.md) |
| Test strategy, corpus, CI matrix | [TESTING.md](TESTING.md) |
| Versioning, build, signing, distribution | [RELEASE.md](RELEASE.md) |
| The public website and how its screenshots are made | [site/README.md](../site/README.md) |
| How to use the app, step by step, with every shortcut | [USER_GUIDE.md](USER_GUIDE.md) |
| User-visible changes per version | [CHANGELOG.md](../CHANGELOG.md) |
| What to update when something changes (the checklist) | [DOCS_GUIDE.md](DOCS_GUIDE.md) |
| Rules for AI assistants | [CLAUDE.md](../CLAUDE.md) |

## Keeping docs in sync with code

See [ADR-0006](adr/0006-docs-as-code.md). In short: docs change in the same commit as the code they describe, requirement IDs connect every artifact, and `scripts/check-docs.sh` plus the docs CI workflow catch drift. **[DOCS_GUIDE.md](DOCS_GUIDE.md) lists, for each kind of change, every file to update**, including the website.
