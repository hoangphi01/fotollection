# ADR-0021: One SemVer version, released automatically when it changes on `main`; no in-app updater

- **Status:** Accepted
- **Date:** 2026-10-24
- **Supersedes:** [ADR-0009](0009-unsigned-distribution.md) in part (the signed updater manifest); unsigned installers stay
- **Related:** FR-021, NFR-006, [RELEASE.md](../RELEASE.md), ADR-0006

## Context

The app is ready for a first public preview, but nothing was in place to ship it: no tags, no release workflow, and a version (`0.0.1`) repeated in `Cargo.toml` and `ui/package.json` with nothing keeping them in step. The draft release process assumed an in-app updater (FR-021) with a signing key and a `latest.json` manifest.

Fotollection is offline by design ([NFR-006](../PRD.md#nfr-006--offline-and-zero-telemetry)). An updater is the one feature that would make the app call the network on its own, and it needs a long-lived private key that must never be lost. The owner prefers that new versions are simply published, for people to download when they want.

Releases should happen without hand work in the GitHub UI, but never by accident on an ordinary push.

## Options considered

1. **Tag-triggered workflow.** Push `vX.Y.Z` and the workflow builds. Explicit, but the tag and the version in the files can disagree, and a tag pushed from a stale checkout ships the wrong code.
2. **release-please / semantic-release.** Versions and the changelog are generated from Conventional Commits, through a release pull request. Fully automatic, but it rewrites the hand-written CHANGELOG, which is user-facing prose here, and adds a PR step to a repository where the owner pushes to `main`.
3. **Release when the version changes on `main`.** `scripts/release.sh` bumps the version and turns the CHANGELOG's `Unreleased` section into the version's section in one commit. When that commit reaches `main`, a workflow sees that `v<version>` has no tag yet, runs the full CI, builds the installers, tags the commit and publishes the release. Every other push sees an existing tag and does nothing.

## Decision

**Option 3, with one SemVer version and no in-app updater.**

- **One version.** The workspace `Cargo.toml` is the source. `ui/package.json` and `Cargo.lock` follow it, and `tauri.conf.json` has no version of its own. `scripts/check-docs.sh` fails when they disagree, or when the version has no CHANGELOG section once it is released.
- **What the numbers mean.**
  - **Until the MVP is complete (0.x),** MINOR is a release with new features (each work stage is one 0.y.0), and PATCH is fixes only.
  - **From 1.0.0,** MAJOR is a change that breaks users: a file that can no longer be migrated, a dropped operating system, or a workflow that changes meaning. MINOR is new features and PATCH is fixes.
  - **Pre-releases** are `X.Y.Z-beta.N` and are marked as pre-releases on GitHub.
- **Commits suggest, a person decides.** With no argument, `release.sh` proposes the next version from the Conventional Commits since the last tag: any `feat` gives MINOR, otherwise PATCH. The owner confirms or overrides it.
- **File formats version independently.** The project, settings and bundle schemas keep their own `schema_version` ([DATABASE.md](../DATABASE.md)). Release notes name any schema that changed.
- **Published at once.** The release is published as soon as the build succeeds, with the CHANGELOG section, the first-launch steps and `SHA256SUMS.txt`. The workflow then rebuilds the website so its download buttons show the new version.
- **No updater.** FR-021 becomes manual updates: new versions are announced on GitHub Releases and the website, and the app shows its version (at the foot of the settings panel) but never checks for updates.

## Consequences

- A release is a single commit (`chore(release): vX.Y.Z`) and a push. A failed build publishes nothing, and pushing the same commit again (or re-running the workflow) retries it.
- No secrets are needed: there is no updater key and no paid signing ([ADR-0009](0009-unsigned-distribution.md)). Integrity rests on GitHub's HTTPS and the published SHA-256 checksums.
- People on old versions are not told about new ones. That is the price of staying fully offline. The website and the README say where to look.
- `tauri-plugin-updater` is not added; [RELEASE.md](../RELEASE.md) §3–§5 and [SECURITY.md](../SECURITY.md) drop the updater manifest.
- The ROADMAP's Phase 1 exit becomes v1.0.0. v0.1.0 is the first public preview.
