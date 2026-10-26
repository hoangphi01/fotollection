# ADR-0009: Ship unsigned builds (no paid code-signing certificates)

- **Status:** Superseded in part by [ADR-0021](0021-versioning-and-automated-releases.md) (no updater, so no signed updater manifest); unsigned installers stay
- **Date:** 2026-10-23
- **Related:** [RELEASE.md §3](../RELEASE.md#3-code-signing), FR-021

## Context

Fotollection is a free, open-source project with no budget. Code signing costs 99 USD/year for an Apple Developer ID (needed for notarization) and a recurring fee for Windows Authenticode, or about 10 USD/month for Azure Trusted Signing.

## Options considered

1. **Buy certificates now:** the best first-run experience, but an ongoing cost with no revenue.
2. **Ship unsigned and document the workaround:** free, but users see OS warnings on first launch.
3. **Ship unsigned now, adopt free OSS signing later:** [SignPath Foundation](https://signpath.org/) offers free Windows code signing to qualifying open-source projects. No free equivalent exists for macOS notarization.

## Decision

Option 3. Ship **unsigned** macOS and Windows builds for now:

- macOS builds get an ad-hoc signature, which Apple Silicon requires to run at all.
- **Always sign the updater manifest** with the free Tauri updater key. That key ensures integrity, so it is not optional.
- Publish **SHA-256 checksums** for every artifact on the release page.
- Apply to SignPath Foundation for Windows signing once the project has a public release history.

## Consequences

- **macOS:** on first launch, Gatekeeper blocks the app. The user opens **System Settings → Privacy & Security → Open Anyway**. Since macOS 15 (Sequoia), right-click → Open no longer bypasses Gatekeeper. The install steps are shown on the release page and in the README.
- **Windows:** SmartScreen shows "Windows protected your PC". The user clicks **More info → Run anyway**.
- 2026-10-26: applied to SignPath Foundation for free Windows signing and was declined the same day for too little public adoption yet; reapply later ([RELEASE.md §3](../RELEASE.md#3-code-signing)). Its terms forbid commercial dual-licensing, so using it and relicensing under [ADR-0008](0008-license-gpl3-with-relicensing-option.md) exclude each other.
- Some users will not install an unsigned app. We accept this for the `0.x` releases and revisit it when the project gains users or revenue, by writing a new ADR that supersedes this one.
