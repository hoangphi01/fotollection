# ADR-0014: Camera brand logos are user-supplied

- **Status:** Superseded in part by [ADR-0020](0020-bundled-brand-logos.md) (bundled public-domain logos); user PNGs remain
- **Date:** 2026-10-23
- **Related:** FR-040, ADR-0008, [SECURITY.md §3](../SECURITY.md#other-untrusted-inputs)

## Context

The info frame (FR-040) shows a brand mark on the left, as camera makers do in their own apps ("LUMIX", "Leica", etc.). Those logos are registered trademarks. Bundling them in a public GPL-licensed app would redistribute third-party marks without permission.

## Options considered

1. **Bundle brand logos:** looks best out of the box, but risks trademark claims and has to be maintained per brand.
2. **Typeset the brand name only:** safe, but less personal.
3. **Typeset by default, and let the user add their own PNG per camera:** safe for the project, and flexible for the user.

## Decision

Option 3.

- The default logo is the brand name (`make`, first word, upper-cased), typeset with the bundled Inter font. The user can edit the text.
- A user can choose a PNG for a camera. The file is validated (magic bytes, ≤ 2 MB, ≤ 4096 px, pure-Rust decoder), and then copied into `config/logos/<camera-id>.png`.

## Consequences

- No trademark assets are in the repository or in the installers.
- A logo is a per-user asset. It is not inside the project file. Moving to another machine means re-adding the logo, or copying the config folder.
