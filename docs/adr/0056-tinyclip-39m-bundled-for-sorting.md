# ADR-0056: TinyCLIP-39M bundled for sorting

- **Status:** Accepted
- **Date:** 2026-11-18
- **Related:** FR-141, FR-015, NFR-008, NFR-022, ADR-0018, ADR-0054

## Context

Sorting a phone's roll (FR-141, ADR-0054) needs the pixels for what the metadata cannot say: what a photo shows (people, cats, dogs, scenery, food, documents, screens, memes). ADR-0054 named CLIP ViT-B/32. The owner asked to measure first and to balance accuracy against time: "10 % more right for 4 hours more is not worth it" (2026-11-18). The owner set the first pass's budget at 20–30 minutes for about 15,000 photos on REF-MAC, in the background and cached (NFR-022), and raised the install budget to 300 MB (NFR-008).

Measured in tract on the owner's 200 subject labels (OPTIMIZATION.md §5, 2026-11-18):

| Model (MIT) | Right of 200 | ms a photo, M3, 8 threads | Bytes, int8 weights |
|---|---|---|---|
| OpenAI CLIP ViT-B/32 | 162 | — (244 on the i5-8500T, 1 thread) | 95.8 MB |
| TinyCLIP-8M | 159 | 4.8 | 9.4 MB |
| TinyCLIP-39M | 172 | 17.6 | 40.8 MB |
| Cascade: 8M, then 39M below p 0.6 | 171 | about 11 | 50 MB |

## Options considered

1. **CLIP ViT-B/32**, as first named: less right than TinyCLIP-39M on these labels, twice its bytes.
2. **TinyCLIP-8M**: the fastest and smallest, 13 points less right.
3. **A cascade**: 1.5 minutes saved on 15,000 photos for one point and a second model.
4. **TinyCLIP-39M on every photo.**

## Decision

Option 4, by a written rule:

- a configuration must finish within NFR-022's budget;
- the most right one wins;
- a faster one wins only when it is at most 2 points less right **and** at least a quarter of the budget faster.

The owner confirmed it on 2026-11-18.

- **Weights:**
  - each MatMul and Gemm weight is stored as int8 per output channel behind a `DequantizeLinear`;
  - tract folds them back to f32 once at load, so the model runs at f32's speed from 40.8 MB;
  - cosine to PyTorch 0.9999;
  - dynamic int8 runs slower in tract.
- **Where it lives:**
  - committed in the repository (`crates/core/assets/models/`, BLAKE3-pinned, MODELS.md);
  - compiled into the app by the shell;
  - checked at load;
  - **never downloaded**.
- **The core** takes the bytes (`sorting::clip::Embedder::load`), so its tests and examples do not each carry 41 MB.
- **The labels' text side** is computed once at export and compiled in (`tinyclip-39m.labels.json`).
- **The pass** (`sorting/pass.rs`):
  - runs from the cached T1, else the file decoded for the screen;
  - caches the difference hash and the 512-float embedding per photo (DATABASE.md §8.2), never the labels, so a new label costs a dot product, not a pass;
  - on the M3, cold, the pass took 55 ms a photo on 8 threads (13.7 min for 15,000) and 141 ms on the background pool's 2 threads (35 min), on 24 MP JPEGs where decoding dominates. It therefore runs on more threads than the background pool while the app is idle (S7).
- **Use:** the model classifies what a photo shows. It never identifies a person, and no face or embedding leaves the computer.

## Consequences

- Installers grow by about 41 MB, within NFR-008's 300 MB; measured on CI's installers when the shell bundles it (S7).
- A new error code, `ERR-RUS-0006` (`sorting.clip_failed`): a photo whose pass fails keeps its metadata classes.
- Decoding, not the model, sets the cold pass's time on large JPEGs. A scaled JPEG decode is the next optimisation, measured first.
- A better model later replaces the file and its checksum, and bumps the cache's version.
