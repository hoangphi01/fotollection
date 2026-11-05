## 1. The sensor: counting light {#wiki-sensor}

A camera's sensor is a grid of millions of tiny wells called photosites. During the exposure each one collects the light that falls on it, turns it into an electric charge, and at the end a converter turns that charge into a number. Twice the light gives twice the number: the sensor is **linear**, unlike our eyes.

There are two ways to read the grid out:

- **CCD** (charge-coupled device) moves the charge of every photosite, row by row, to one amplifier at the edge of the chip. It was the sensor of the first digital cameras: clean, but slow and power-hungry.
- **CMOS** gives every photosite its own amplifier and reads many of them at once. It is faster, uses less power and is cheaper to make, so nearly every camera and phone today has one. Most read the rows one after another, which is why a fast-moving subject can lean in a photo taken with the electronic shutter (the *rolling shutter*).

The numbers have more steps than a JPEG: a camera stores 12 or 14 bits per photosite (4,096 or 16,384 levels), where a JPEG keeps 8 (256). And zero light is not zero: every sensor adds a small **black level**. In the Nikon D750 sample, for example, black is 600 and the brightest value the sensor can hold, its **white level**, is 15,520. Everything a raw developer does starts by putting those two numbers back to 0 and 1.

## 2. Colour: one filter on every photosite {#wiki-cfa}

A photosite only measures how much light arrives, not its colour. To see colour, the sensor is covered by a **colour filter array**: a tiny red, green or blue filter over each photosite, so each one counts only one part of the spectrum.

- **Bayer**, patented by Bryce Bayer at Kodak in 1976, repeats a 2 × 2 square: one red, two green, one blue. Half the photosites are green because our eyes take most of their sense of detail from the green part of the light. Almost every camera uses it.
- **X-Trans**, Fujifilm's pattern since 2012, repeats a 6 × 6 square with red, green and blue in every row and column. Its irregularity makes moiré less likely, at the cost of a harder reconstruction.
- **Other designs:** monochrome sensors have no filter at all (Leica's Monochrom cameras); Sigma's Foveon stacks three layers in depth; many phones group four photosites of one colour (*Quad Bayer*).

So a 24 MP Bayer sensor records 24 million numbers, each of a single colour. Two of the three colours of every pixel are missing, and have to be worked out.

## 3. What a RAW file holds {#wiki-raw}

A JPEG is a finished picture. A RAW file is the sensor's numbers, kept before any of those decisions are made, with what is needed to make them:

- the samples themselves, one per photosite, and which filter colour each one sat under;
- the black and white levels;
- the **active area** (the photosites that saw light; some at the edges are masked to measure black) and the camera's **default crop**;
- the **white balance** the camera chose at the shot;
- **colour matrices**, which say how this sensor's red, green and blue relate to the colours we see;
- a **preview**, a JPEG the camera made itself, and the EXIF data.

Each maker has its own format, mostly built on TIFF and partly undocumented: CR2 and CR3 (Canon), NEF (Nikon), ARW (Sony), RW2 (Panasonic), ORF (Olympus), RAF (Fujifilm). **DNG** is Adobe's openly published format (version 1.7), used by Ricoh, Leica, Pentax and many phones.

Fotollection reads DNG with its own reader, written from the specification. The other formats are unpacked by rawler, an LGPL library built into the app. Thumbnails never need the sensor data: they come from the preview inside the file, in 9 to 32 ms.

## 4. Demosaicing: working out the missing colours {#wiki-demosaic}

Rebuilding the two missing colours of every pixel from its neighbours is called **demosaicing**. It is the step that most decides how sharp and how clean a RAW photo looks.

- **Bilinear** averages the nearest photosites of each colour. It is simple and fast, but soft, and it puts coloured fringes along every sharp edge.
- **Malvar–He–Cutler** (2004) starts from that average and corrects it with the detail of the photosite's own colour. It is sharper and still fast. Fotollection uses it today.
- **PPG** (patterned pixel grouping) first decides along which direction the image changes least, and interpolates along it. rawler uses it.
- **RCD** (ratio corrected demosaicing, by Luis Sanz Rodríguez) works on ratios between colours and is known for few artefacts on fine detail; darktable and RawTherapee offer it.
- **Markesteijn**, after Frank Markesteijn, is the method made for X-Trans, in one or three passes.

When a method guesses wrong, it shows in typical ways:

- **zipper:** a row of alternating light and dark pixels along an edge;
- **maze:** a fine worm-like pattern in flat areas, when the two greens of a Bayer square do not read quite the same;
- **false colour:** coloured dots or fringes on detail that is really grey, such as fabric, hair or a distant fence;
- **moiré:** coloured waves on very fine repeating patterns.

**How we judge one.** Our eyes are easily fooled, so every method is scored on pictures whose truth is known. A computer draws a zone plate (rings that get finer and finer), a Siemens star, saturated colour bars and smooth ramps, turns them into a sensor's mosaic, demosaics it, and compares the result with the original:

| Chart (768 px) | Bilinear | Malvar (ours today) |
|---|---|---|
| Zone plate: PSNR, higher is better | 13.0 dB | 17.3 dB |
| Zone plate: false colour, lower is better | 65.9 | 43.0 |
| Siemens star: PSNR | 21.3 dB | 25.2 dB |
| Saturated colour bars: error along the edges, lower is better | 11.6 | 21.6 |

Malvar is clearly better on fine grey detail, but worse on edges between strong colours, because its correction assumes the colours change together. That is the gap RCD and a false-colour step have to close, measured on the same charts (coming in v0.11.0).

## 5. From the camera's colours to yours {#wiki-colour}

The camera's red, green and blue are not the red, green and blue of your screen: every sensor has filters of its own dyes. The colour matrices in the RAW file turn the camera's colours into **CIE XYZ**, the standard model of human colour vision (1931), and from there into **sRGB**, the colour space of most screens and of photo labs.

- **White balance** comes first: a gain on red and on blue so that something white in the scene comes out neutral. The camera records the one it chose.
- A DNG usually gives **two matrices**, one measured under warm tungsten light and one under daylight. The right one for your shot lies in between: the developer works out the shot's light from its white balance and blends the two. Fotollection does the same for the other formats, from the matrices rawler knows for each camera.
- All of this happens in **linear light**, on the sensor's own scale, before any curve: mixing and resizing colours there gives the right result.

**The camera's look.** A camera's JPEG is not neutral: each maker adds its own contrast, colour and skin tones. Today Fotollection fits its develop to the preview inside the RAW file, so a RAW photo looks like what the camera showed on its screen. Coming in v0.11.0: a neutral rendering by default, with **Like the camera's JPEG** as a built-in look, one click away.

## 6. Tone: from numbers to a picture {#wiki-tone}

Linear numbers look dark and flat on a screen, because our eyes see brightness in proportion, not in amounts. A **tone curve** turns the sensor's scale into a picture: it lifts the shadows, keeps the highlights from clipping at once, and gives the image its contrast. The file is then stored with the sRGB curve (its "gamma"), which spends more of the 256 levels of a JPEG on the shadows, where our eyes are most sensitive.

Two things the camera fixes in its JPEG are not in the RAW data:

- **Lens shading:** a lens is darker in the corners. Fotollection measures it from the camera's own preview: fitting a radial gain took our DNG develop from ΔE 3.8 to 2.67 against the camera's JPEG, with every region of the photo within ±0.5 L\*.
- **Distortion:** the slight barrel or pincushion of a lens, which Micro Four Thirds cameras (Panasonic, Olympus) rely on the camera to correct. On a Mac those files are developed by macOS, which corrects it; correcting it in our own develop is planned (FR-085).

## 7. How Fotollection develops a RAW {#wiki-fotollection}

Step by step, for a full develop (the print preview, zoom and export):

1. **Recognise** the format from the file's first bytes, not its name.
2. **Read** the sensor's numbers: DNG with our own reader, the others unpacked by rawler. A file too large or damaged is refused before any pixel is decoded.
3. **Levels:** black to 0 and white to 1, per position of the colour filter.
4. **Demosaic** (Malvar today; RCD and Markesteijn coming in v0.11.0).
5. **Colour:** white balance and the camera's matrices, in linear light.
6. **Tone and look**, fitted to the camera's preview, with its lens shading.
7. From here a RAW photo is like any other: crop, date stamp, frame, print size and colour profile, exported at full resolution.

A few rules hold throughout. One develop runs at a time, on every processor core, and never on the window's own thread, so the app keeps answering. Exports always run on the processor, so a print comes out the same on every computer; the graphics card only speeds up what you see while editing. And the code is ours, written from the published specifications and papers, not copied from other projects, then checked: on the synthetic charts above, on nine RAW files from different cameras (CC0, from raw.pixls.us), and by cutting and corrupting files on purpose so a bad file is an error, never a crash.

**Coming in v0.11.0:** our own develop takes every camera with a Bayer sensor, and rawler only unpacks the files. Measured on the nine sample files (full develop, median of 3):

| Camera | Ours | rawler | macOS |
|---|---|---|---|
| Sony A7 III | 211 ms | 241 ms | 488 ms |
| Canon R10 | 293 ms | 360 ms | 1,647 ms |
| Nikon D750 | 340 ms | 402 ms | 412 ms |
| Canon 5D Mark IV | 462 ms | 565 ms | 418 ms |
| Ricoh GR III | 333 ms | 650 ms | — |

Fujifilm X-Trans follows with Markesteijn, and the develop on the graphics card for the preview after that, with the processor's result as its reference.
