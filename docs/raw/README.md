# Camera RAW formats

Fotollection reads RAW through three decoders tried in order ([ADR-0029](../adr/0029-raw-through-rawler-then-our-own-decoder.md), [ADR-0030](../adr/0030-our-own-raw-decoder.md)): our own (`crates/rawcore`) for the formats it has taken over, then ImageIO on macOS or rawler on Windows, each with the other as fallback.

A format moves to our own decoder only after it passes the quality gate of ADR-0030 on the CC0 samples (`scripts/fetch-raw-samples.py`). The numbers are in [RESULT.md §16](../RESULT.md#16-our-own-raw-decoder-dng-v080-measured-first).

| Format | First decoder | Notes | Status |
|--------|---------------|-------|--------|
| DNG (Bayer, uncompressed or lossless JPEG) | **Our own** | [dng.md](dng.md) | Passed the gate on the Ricoh GR III sample |
| ARW (Sony) | ImageIO / rawler | not written yet | Next |
| RW2 (Panasonic) | ImageIO / rawler | not written yet | Next |
| NEF, CR2, ORF, CR3, RAF | ImageIO / rawler | | Later |

Rules for these notes: written in our own words from public specifications and from looking at files with our own tools. No code from rawler, rawloader, dcraw, LibRaw or RawTherapee is read, copied or paraphrased.
