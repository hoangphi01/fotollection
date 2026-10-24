# ADR-0020: Bundle public-domain camera-brand logos for the info frame

- **Status:** Accepted
- **Date:** 2026-10-24
- **Supersedes:** [ADR-0014](0014-user-supplied-brand-logos.md) in part (user PNGs stay)
- **Related:** FR-040, ADR-0008, ADR-0016

## Context

The "Shot on …" info frame ([FR-040](../PRD.md#fr-040--exif-info-frame)) looks right with the camera maker's logo, as the makers' own apps show it. [ADR-0014](0014-user-supplied-brand-logos.md) left logos to the user, to keep trademarks out of the public repository. The owner asked for the logos to come with the app, after weighing these points:

- **Copyright and trademark are separate.** A logo can be free of copyright and still be a trademark.
- **Being non-commercial helps, but it is not a licence.** Trademark law is about confusion and implied endorsement, and a public repository is easy to send a takedown request to.
- **Wikipedia is not a licence either.** English Wikipedia hosts many logos as *non-free* (fair use inside its articles only). Only files on Wikimedia Commons marked public domain can be copied.

## Decision

Bundle the logos of 11 brands, under these conditions:

1. **Source:** only files on **Wikimedia Commons** whose licence is *Public domain* (`PD-textlogo` / `PD-shape`). On 2026-10-24 all of them were verified through the Commons API, with the restriction *trademarked*: Apple, Canon, Fujifilm, Hasselblad, Leica, Lumix (Panasonic), Nikon, Phase One, Ricoh, Samsung and Sony.
2. **Provenance:**
   - The SVGs are pinned by SHA-256 in `scripts/brands/fetch-and-render.sh`, checked for scripts and external references, and kept in `crates/core/assets/brands/src/`.
   - The PNGs the app uses are rendered from them, on a transparent background.
   - [`BRANDS.md`](../../crates/core/assets/brands/BRANDS.md) lists each file, its Commons page and the owner of the trademark.
3. **Notice:** BRANDS.md and both READMEs state the following:
   - the marks belong to their owners;
   - Fotollection is not affiliated with, sponsored or endorsed by any of them;
   - the logos only show which camera took the photo, on the user's own prints;
   - a logo is removed on request.
4. **Use:**
   - A camera whose EXIF make matches a bundled brand defaults to `logo: "brand"`.
   - The user can still choose a typeset name, their own PNG ([ADR-0014](0014-user-supplied-brand-logos.md)), or nothing.
   - **Settings v4** migrates cameras that were still on the default typeset name (no text typed) to the brand logo.
5. **Colours are applied at run time; the files are never edited.** Three styles:
   - **Classic** (the default) is like a classic "Shot on" frame:
     - the Canon, Samsung and Hasselblad wordmarks are in the text colour;
     - the black parts of Apple, Fujifilm, Lumix and Sony are in the text colour, and their colour accents stay (Fujifilm's red "I");
     - Ricoh, Phase One and the badges keep their colours.
   - **Brand colours.**
   - **One colour.**

   On a black frame, any part darker than luma 64 turns white. The Leica and Nikon badges are never recoloured.
6. **Size:**
   - Wordmarks (wider than 2.5 : 1) are drawn at about the cap height of the frame text.
   - Square marks are drawn larger, so both read at the same weight.

## Consequences

- The installer grows by about 520 KB of PNGs; the SVG sources add 55 KB to the repository.
- A takedown request means deleting one brand's PNG, its SVG and its table row. Cameras of that brand then fall back to the typeset name, with no settings change.
- If the project is relicensed for commercial use ([ADR-0008](0008-license-gpl3-with-relicensing-option.md)), this decision is reviewed first.
- More brands (Olympus/OM System, Pentax, Sigma, Xiaomi, Google…) are added only under the same rule: public domain on Commons, verified, with a row in BRANDS.md.
