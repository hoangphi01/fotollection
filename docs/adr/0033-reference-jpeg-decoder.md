# ADR-0033: A reference JPEG decoder behind the fast one

- **Status:** Accepted. Amends [ADR-0012](0012-pure-rust-codecs-first.md)
- **Date:** 2026-11-01
- **Related:** FR-001, NFR-001, [RESULT.md §19](../RESULT.md#19-jpeg-layouts-and-the-reference-decoder-v081), [LESSONS.md](../LESSONS.md)

## Context

A v0.8.0 user saw half of a folder as green tiles with fine horizontal stripes and a ghost of the photo. The photos were JPEGs saved with **one scan per component**, a rare but valid baseline layout. `zune-jpeg` 0.5.15 decodes it with the colour planes wrong (the Cb scan reads into the Cr scan) and **reports no error**. The same decode feeds thumbnails, the preview, best-shot analysis and the export, so prints came out green too. The upstream fix is only in 0.5.16 release candidates, and another project still saw an RC misread files.

A wrong decode that looks like success cannot be caught after the fact. The question is how to keep zune's speed (45 ms for 16 MP) without trusting it on layouts nobody has checked.

## Options considered

1. **Upgrade to a 0.5.16 release candidate.** Fixes this file, but ships an RC, and the next unchecked layout fails the same way.
2. **Decode every JPEG through the OS** (ImageIO, WIC). Right everywhere, but slower, different per system, and a new platform path to test.
3. **Check every decode** (compare with a second decoder, or look for green). Doubles the cost, or guesses.
4. **Read the layout from the header first.** zune decodes only the layouts a corpus test proves; every other layout, and any file zune refuses, goes to `jpeg-decoder` (pure Rust, MIT or Apache-2.0, the decoder the `image` crate used for years).

## Decision

Option 4.

- `jpeg_layout::read` reads the markers up to the first scan: the frame type, the precision, each component's sampling, and the components in the first scan. It costs 0.2 µs a file and never panics on broken input.
- `Layout::zune_proven` lets zune decode 8-bit baseline, extended or progressive files, grey or colour, with 4:4:4, 4:2:2, 4:2:0, 4:4:0 or 4:1:1 colour, and every component in the first scan of a baseline file. CMYK stays on zune as before, until a fixture proves either decoder. **Everything else goes to the reference decoder**, as does any file zune refuses.
- **2026-11-02 (v0.9.1):** a fixture proves Adobe CMYK on zune (`cmyk.jpg`), and the reference decoder's CMYK fallback, which came out negative, matches it. Extended SOF1 leaves `zune_proven` (no fixture). A header `jpeg_layout::read` cannot parse goes to the reference decoder, not to zune. A file whose data never reaches its end-of-image marker is refused (`ERR-IMG-0006`). Grey or CMYK ICC profiles are dropped from the RGB pixels. The corpus test checks the 99th-percentile error and the route of each file. Later the same day, `ycck.jpg` proves Adobe YCCK on zune, and `extended-sof1-420.jpg` shows zune decoding one SOF1 file right; SOF1 still stays off, as it also allows four Huffman tables, which no fixture covers.
- The corpus is `crates/core/tests/fixtures/jpeg/`: one picture saved by `cjpeg` in each layout, compared with the picture itself (`tests/jpeg_layouts.rs`). A decoder upgrade that breaks a layout fails CI. A layout joins the allowlist only with a fixture that proves it.
- A decoder fix that changes the pixels of files already seen bumps `thumbs::CACHE_VERSION`, so thumbnails and best-shot analyses are made again.
- zune-jpeg stays on 0.5.15. The 0.5.16 stable release will be tried against the corpus.

## Consequences

- Rare layouts decode correctly, at about half zune's speed (88 ms for 16 MP); the usual ones keep zune's speed.
- One more pure-Rust, permissively licensed dependency.
- After the v0.8.1 update every thumbnail is made once more; the old files age out of the cache.
