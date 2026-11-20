# ADR-0059: Subjects routed by the photo's source

- **Status:** Accepted
- **Date:** 2026-11-20
- **Related:** FR-141, ADR-0054 (metadata first, then pixels), ADR-0056 (TinyCLIP-39M bundled)

## Context

RollSense names each photo's subject with TinyCLIP-39M zero-shot: a softmax over every subject's prompts, a subject named when its summed share reaches a bar. Every photo was asked the same question. On the owner's trip library, photos of people taken with a compact that prints the date in orange went under *Other*, because the *screen* prompts ("a screenshot", "text on a screen") took 0.2–0.43 of their probability; and screenshots were weighed against scenery. ADR-0054 already reads the source from the metadata before the pixels (99.6 % right on 237 labelled files), but the subjects did not use it. The owner (2026-11-20): "not every photo should go through the softmax; split the metadata first to see the source, then decide how to apply the model".

## Options considered

All measured on the owner's labels by recomputing stored probabilities (RESULT §42), against the version shipped (`b5637207`: 11 subjects, one softmax, 0.45):

| Option | Library *Other* (759) | Trip set right | Phone set right | Screenshots found |
|--------|----------------------:|---------------:|----------------:|------------------:|
| A. One softmax (shipped) | 30 | 90.2 % | 92.2 % | 101/112 |
| Screen and meme left out for cameras, one bar | 9 | 85.6 % | 90.7 % | 100/112 |
| B. Routes, lens 0.50 | 14 | 88.6 % | 89.6 % | 102/112 |
| C. Routes, lens 0.55 | 24 | 90.4 % | 90.6 % | 102/112 |
| **C + the screen route's guard** | **24** | **90.4 %** | **92.2 %** | **101/112** |

Three independent reviews (statistics, computer vision, product) each chose C, then defended it against the others: the gaps between A, B and C are inside the noise at n = 200, but the 8 photos B names beyond C are right only 4 times, so its fewer *Other* cost a wrong name in two; a wrong name costs more than *Other* because a subject filter can lead to a delete.

## Decision

1. **Three routes by the source**, read from the metadata before the pixels (`sorting/routes.rs`, `rules.json` `subject_routes`):
   - **lens** (the phone's camera, a panorama, another camera, another phone, a film scan): people, dog, cat, animal, scenery, building, vehicle, food, document; bar 0.55. A photo through a lens is never a screenshot or a meme.
   - **screen** (a certain screenshot): screen, document, meme; bar 0.35, with a **guard**: a screenshot goes this way only when the softmax over every subject also names one of them, else it goes the open way, so a file misread as a screenshot is never named *screen* with confidence.
   - **open** (a guessed screenshot, an app's, an edited, an unknown file): every subject; bar 0.45, as before.
2. A route's softmax is the summed shares renormalised over its subjects (the ratios hold, so it is the softmax over its prompts); each route has its own bar, since fewer subjects inflate the top share.
3. The rules that held keep: buildings and scenery together name the larger; people are added at 0.35.
4. Before release, 100 photos drawn at random from those C names on the library are labelled blind; C ships when the Wilson lower bound of their precision is at least 85 % (RESULT §42).
5. Later rounds are chosen by a rule written before measuring: the photos a candidate names beyond the current one are at least 75 % right, on at least 30 such photos, with the Wilson lower bound reported (the 2-point rule was under one standard error).

## Consequences

- No model, size or time change: the same pass, a different sum at its end; the cached embeddings stay valid and the next pass names every photo again.
- A phone's photo of a monitor through the lens route cannot be named *screen* (the structural error of this design); v0.14.5 adds lens-only prompts for a photo of a monitor, a laptop or a TV, the date-imprint crop and a second look at rotated photos left in *Other*, each measured on the photos it changes.
- A route's subjects and bar are data in `rules.json`, checked at load against the labels; a change is a measurement, not code.
