# ADR-0061: Sign and notarize the macOS app with a Developer ID

- **Status:** Accepted
- **Date:** 2026-11-21
- **Related:** FR-021, [ADR-0009](0009-unsigned-distribution.md) (superseded for macOS), [ADR-0060](0060-imageio-raw-develop-in-a-child-process.md), [RELEASE.md §3](../RELEASE.md#3-code-signing)

## Context

Since v0.1.0 the macOS `.dmg` carries only an ad-hoc signature (ADR-0009): Gatekeeper blocks the first launch, and the user has to find **System Settings → Privacy & Security → Open Anyway**, since right-click → Open no longer works on macOS 15. On 2026-11-21 the owner bought the Apple Developer Program membership (99 USD a year), the only way to notarize for macOS.

## Options considered

1. **Keep the ad-hoc signature:** free, but every new user meets the block, and some stop there.
2. **Sign with Developer ID and notarize, in the release workflow:** Gatekeeper opens the app at once; the secrets live only in GitHub Actions.
3. **Sign on the owner's Mac and upload by hand:** no secrets in CI, but the release stops being one pushed commit (ADR-0021).

## Decision

Option 2. The certificate "Developer ID Application: Hoang Phi Pham (6B3C29XQ9X)" signs the app with the hardened runtime, and an App Store Connect API key with the Developer role notarizes it. `scripts/macos-signing-keys.sh` puts the certificate into a keychain of the job; Tauri signs, notarizes and staples the `.app`; `scripts/notarize-dmg.sh` signs, notarizes and staples the `.dmg` around it and fails the release unless Gatekeeper accepts both. The RAW helper of ADR-0060 is the app's own executable started again, so it carries the same signature.

The six secrets (`APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_API_ISSUER`, `APPLE_API_KEY`, `APPLE_API_KEY_P8`) are GitHub Actions secrets set by the owner; the private key, the `.p12` and the `.p8` stay on the owner's machine, never in the repository. A local `npm run build` without them still signs ad hoc (`signingIdentity: "-"` in `tauri.conf.json`, which the CI's `APPLE_SIGNING_IDENTITY` replaces).

## Consequences

- From v0.14.5 a downloaded `.dmg` opens with no Gatekeeper step; releases before it stay ad-hoc signed, since re-signing a published file would change its checksum and the Homebrew cask's.
- The first launch of the first signed version may ask again for folder access, since the app's signature changed; settings and projects stay (same bundle identifier).
- Notarization adds a few minutes to the macOS build; the `installers` workflow builds a signed, notarized Apple Silicon `.dmg` whenever the signing scripts or the bundle settings change, so a signing fault shows before a release.
- The certificate expires on 2027-12-21 and the membership renews yearly: the release fails at the signing step when either lapses.
- Windows stays unsigned (ADR-0009).
