# ADR-0008: License — GPL-3.0, keeping the option to relicense

- **Status:** Accepted
- **Date:** 2026-10-23
- **Related:** ADR-0005, [SECURITY.md §6](../SECURITY.md#6-supply-chain), [CONTRIBUTING.md](../../CONTRIBUTING.md#contributor-license-agreement)

## Context

Fotollection is developed in the open on a public GitHub repository. The owner wants:

- Copyleft protection, so that nobody can ship a closed-source fork of the open code.
- The **option to release future versions under a different license**, including a proprietary one.

Comparable open-source photo apps (GIMP, darktable, Krita) use the GPL.

## Options considered

1. **MIT / Apache-2.0:** maximum adoption, but anyone can make a closed fork.
2. **LGPL-3.0:** designed for libraries. For an application it adds almost nothing over the GPL and is unusual.
3. **MPL-2.0:** file-level copyleft, which is weaker protection.
4. **GPL-3.0:** strong copyleft, the norm for desktop creative apps, and compatible with our LGPL dependencies.

## Decision

License Fotollection under **GPL-3.0-only** (SPDX: `GPL-3.0-only`). The full text is in [LICENSE](../../LICENSE).

The **copyright holder (@hoangphi01) may relicense future versions**. This is possible only as long as the following rules hold:

1. **CLA for outside contributions.** Before an outside contribution is merged, its author agrees to a Contributor License Agreement. The CLA grants the project owner the right to relicense the contribution. Without it, contributed code stays GPL-only and blocks relicensing.
2. **Permissive dependencies only.** Rust and npm dependencies must be under MIT, Apache-2.0, BSD, ISC, Zlib, Unicode or a similar permissive license. `cargo deny` enforces an allowlist. **GPL/AGPL dependencies are banned**, because they would lock the project to the GPL.
3. **LGPL native libraries are isolated.** libheif and libde265 (LGPL) are fine under the GPL. A future proprietary edition would have to link them **dynamically** (or otherwise let users relink them) to comply with the LGPL.

## Consequences

- Every version released under the GPL **stays GPL forever**. A relicense only applies to new versions, and the community can keep forking the last GPL release.
- A CLA adds friction for contributors. It is set up (for example with CLA Assistant) before the first outside pull request is merged. Tracked in [STATUS.md](../STATUS.md).
- Source files get an SPDX header: `// SPDX-License-Identifier: GPL-3.0-only`.
- Third-party data attribution (GeoNames, CC BY 4.0) appears in the About dialog and README.
