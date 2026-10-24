# ADR-0019: Group similar shots by a layout and colour signature, scored on labelled photos

- **Status:** Accepted
- **Date:** 2026-10-24
- **Related:** FR-041, NFR-014, ADR-0018

## Context

Best-shot suggestions ([FR-041](../PRD.md#fr-041--best-shot-suggestions)) first group the shots of one moment and then rank each group. The first version confirmed groups with a 64-bit dHash and split them whenever the focal length changed by more than 20 %.

To check it, 95 real photos from the owner's library (people, food, streets, night shots, sunsets) were labelled by hand. The labels give 22 groups of one moment, and for 6 of them the photo to keep. They are stored in `bench/bestshot/truth-103_09.json` as file names only, and `crates/core/examples/eval_bestshot` scores the app against them.

The first version found **3 of the 22 groups**. Two causes stood out:

- **dHash is unstable in flat areas.** A dHash records whether each cell is brighter than its neighbour, so in sky or plain walls its bits follow noise. Two near-identical shots of a hotel were 38 bits apart, while unrelated scenes were 18–37 bits apart. No threshold could separate them.
- **Re-shooting closer was treated as a new scene.** Zooming from 5 to 9.9 mm on the same church split the group.

## Decision

1. **Signature per photo**, stored in the analysis cache (`ANALYSIS_VERSION` 2). It is computed from a 256 px copy so it stays within [NFR-014](../PRD.md#nfr-014--best-shot-analysis-speed), and has two parts:
   - a 32 × 24 box-averaged luma image;
   - a 37-bin hue × saturation histogram (12 hues × 3 saturations, plus greys).
2. **Layout similarity** is the best Pearson correlation between the two luma images over shifts of ±3 cells and zooms of ×0.85–1.18, with the centres aligned. A landscape and a portrait shot of one view therefore also compare. When EXIF has both focal lengths, their ratio sets the zoom, and the closer shot is compared with the matching centre of the wider one. Correlation ignores exposure differences, and flat areas carry no weight.
3. **Colour similarity** is histogram intersection. It separates scenes with a similar layout, for example the church and the street next to it.
4. **A shot joins the group** when, against the previous shot, layout ≥ 0.6 and colour ≥ 0.72, and, against the group's first shot, layout ≥ 0.4 (so a slow pan does not chain on). The time-gap, camera, lens and GPS rules stay. A focal-length change alone no longer splits a group.
5. **Ranking fixes** found with the same labels:
   - Every shot of a group is measured on the same sharpness: eye-area sharpness only when every shot has a face, whole-image sharpness otherwise.
   - A face turned away or hidden (fewer faces than in the group's best shot) costs 0.5 in the eyes term.

The thresholds were chosen from the measured distributions with a margin. They are not tuned to every pair: wider zooms (×0.7–1.43) were tried and rejected, because they added a wrong group and found no new correct one.

## Results (same 95 photos)

| Step | Groups found (of 22) | Pair recall | Pair precision | Acceptable picks |
|------|-----:|-----:|-----:|-----:|
| dHash, focal split (before) | 3 | 9 % | 100 % | 2/2 |
| + layout and colour signature | 13 | 67 % | 100 % | 3/4 |
| + zoom from focal length | 15 | 73 % | 100 % | 5/6 |
| + fair sharpness, hidden faces | 15 | 73 % | 100 % | 6/6 |

The 2 extra groups the app makes are pairs labelled as ambiguous. Analysis time is unchanged: 17.4 ms per photo, 27.6 ms with faces.

## Consequences

- Cached analyses from version 1 are recomputed automatically. Each entry grows by about 1 KB (the signature).
- 7 labelled groups are still missed: the same moment with a very different pose or framing, such as a pizza shot again from the other side or a selfie taken upside down. Telling those apart needs an image-understanding model (an embedding network), which must first pass the licence and speed checks of [ADR-0018](0018-face-models-and-runtime.md).
- The labelled set is small, comes from one camera (a Canon compact) and was labelled by one person. The scores are evidence for the direction, not a guarantee. Any change to grouping or ranking must re-run `eval_bestshot` and must not lower these numbers. More labelled folders, ideally from other cameras and phones, should be added over time.
