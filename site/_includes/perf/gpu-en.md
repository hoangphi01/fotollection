### Ten sliders for the price of one

Every colour slider (brightness, contrast, warmth, saturation, fade, toned shadows and highlights, black and white) is folded into one table of 33 × 33 × 33 colours, 35,937 points that say where each colour goes. Applying it to a photo means finding each pixel's cell in the table and interpolating between four of its corners (tetrahedral interpolation), so ten sliders cost what one does. Building the table with every slider set takes about 2.6 ms; applying it to a 1,600 px preview, under 2.2 ms; to a 24 MP export, about 24 ms.

### Why a drag still felt slow, and what the graphics card changed

Measured step by step, one move of a slider cost about 26 ms, and the colour itself was only 2 ms of it: the rest was encoding the preview as a JPEG (7 to 11 ms) and the window decoding it again (9 to 15 ms). The graphics card removes both. When a drag starts, the plain preview and a mask of where the photo is (not the date stamp, the frame or the border) go to the graphics card once; at every step only the new table follows, and a small WebGL2 program interpolates it, exactly as the core does: 1 ms a step in macOS's WebKit, 2 ms in Chromium. When you let go, the core's exact render replaces it. The two were compared on a strong look with every slider moved: they differ by 0.13 in ΔE2000 on average and 0.77 at the 99th percentile, far below the 1 an eye can see side by side.

No switch is needed. At the first drag, the app checks for itself: if there is no WebGL2, or the renderer is a software one, or a test step takes 8 ms or more, it keeps the processor's path for the session. A software renderer took 16.5 ms a step in the test, so it is right to stay on the processor there.

### Exports on every core

A photo's decode and encode run on one core each, so an export is fast when several photos are worked on at once. How many depends on memory: on 24 to 26 MP photos, 2 at once gave 4.9 photos a second, 4 gave 9.4, 6 gave 11.5 and 8 gave 12.5, with memory rising from 0.49 to 1.21 GB. The app picks the number from a memory budget of 1.6 GB and the size of the largest photo, and stops at 6, because on a real trip's photos 8 at once was no faster and used 0.2 GB more. Exports never use the graphics card: a print must come out the same on every computer, and the tests that compare exports byte for byte depend on it.
