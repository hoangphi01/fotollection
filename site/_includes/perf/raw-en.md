### What a sensor really records

A camera's sensor is a grid of millions of tiny wells called photosites. While the shutter is open, each one collects the light that reaches it and turns it into an electric charge; when it closes, a converter turns each charge into a number. The relation is **linear**: twice the light gives twice the number. Our eyes are not linear at all, they judge brightness in proportion, which is why a raw file looks dark and flat until it is developed.

Two kinds of sensor read that grid out:

- **CCD** (charge-coupled device) shifts the charge of every photosite, row by row, to one amplifier at the edge of the chip. It was the sensor of the first digital cameras and of many early medium-format backs: clean images, but slow to read and hungry for power.
- **CMOS** gives every photosite its own amplifier and reads many of them at once. It is faster, uses less power and is cheaper to make, so nearly every camera and phone today has one. Most CMOS sensors read their rows one after another, which is why a fast-moving subject can lean in a photo taken with the electronic shutter (the *rolling shutter*), and why stacked sensors, which read much faster, lean less.

The numbers have far more steps than a JPEG. A camera stores 12 or 14 bits per photosite, 4,096 or 16,384 levels, where a JPEG keeps 8 bits, 256 levels. Those extra levels are what let you lift a shadow or pull back a bright sky without banding. And no light is not zero: every sensor adds a small offset, the **black level**, so that noise around black is not cut off. In the Nikon D750 sample, black is 600 and the brightest value the sensor can hold, the **white level**, is 15,520. The very first thing any raw developer does is bring those two numbers to 0 and 1, for each position of the colour filter, because some cameras give each one its own black level.

### Colour comes from a filter over each photosite

A photosite only measures how much light arrives, never its colour. To see colour, the sensor is covered by a **colour filter array**: a tiny red, green or blue filter over each photosite, so each one counts only part of the spectrum.

- **Bayer**, patented by Bryce Bayer at Kodak in 1976, repeats a 2 × 2 square: one red, two green, one blue. Half the photosites are green because our eyes take most of their sense of detail from the middle of the spectrum, so green carries most of the sharpness. Almost every camera uses this pattern, in one of four orientations (RGGB, BGGR, GRBG, GBRG) that the file has to state.
- **X-Trans**, Fujifilm's pattern since 2012, repeats a 6 × 6 square with red, green and blue in every row and every column. Its irregular layout makes colour moiré less likely on fine repeating detail, so Fujifilm can leave out the blurring filter many sensors carry, at the cost of a harder reconstruction.
- **Other designs:** monochrome sensors have no filter at all, so every photosite sees full detail (Leica's Monochrom cameras); Sigma's Foveon stacks three layers that absorb colour at different depths of the silicon; many phones group four photosites of one colour (*Quad Bayer*) and read them as one in dim light.

So a 24 MP Bayer sensor records 24 million numbers, each for a single colour: two of the three colours of every pixel are missing. Turning that mosaic into a picture is the job of demosaicing, below.

### What a RAW file holds

A JPEG is a finished picture: the camera has already made every decision. A RAW file keeps the sensor's numbers from before those decisions, with everything needed to make them later:

- the samples themselves, one per photosite, and which filter colour each one sat under;
- the black and white levels;
- the **active area** (the photosites that saw light; a few rows and columns at the edges are masked on purpose, to measure black) and the camera's **default crop**;
- the **white balance** the camera chose when the photo was taken;
- **colour matrices**, which say how this sensor's own red, green and blue relate to the colours people see;
- a **preview**, a JPEG the camera made itself, with its own colour and tone, plus the EXIF data.

Each maker has its own format, mostly built on TIFF and partly undocumented: CR2 and CR3 (Canon), NEF (Nikon), ARW (Sony), RW2 (Panasonic), ORF (Olympus), RAF (Fujifilm). **DNG** is the format Adobe publishes openly (version 1.7), used by Ricoh, Leica, Pentax and many phones, and the one whose rules this page follows.

Fotollection reads DNG with its own reader, written from the specification, and the other formats are unpacked by rawler, an LGPL library built into Fotollection. The thumbnails in the grid never touch the sensor data at all: they come from the preview inside the file, in 9 to 32 ms, which is why a folder of RAW files opens as fast as a folder of JPEGs.

### Demosaicing: working out the two missing colours

Rebuilding the two missing colours of every pixel from its neighbours is called **demosaicing**, and it decides more than any other step how sharp and how clean a RAW photo looks. Every method faces the same dilemma: averaging neighbours is safe in flat areas, but across an edge it mixes two different surfaces, so the art is in knowing which neighbours belong together.

- **Bilinear** simply averages the nearest photosites of each colour. It is the floor every method is measured against: soft, and with coloured fringes along every sharp edge, because red, green and blue are averaged over slightly different places.
- **Malvar–He–Cutler** (2004) starts from that average and corrects it with the detail of the colour the photosite did see, through fixed 5 × 5 filters. It is sharper and still very fast. Fotollection developed with it until v0.10.0.
- **PPG** (patterned pixel grouping) first decides along which direction the image changes least, and interpolates along that direction only. rawler uses it, and so did Fotollection's Windows and Linux builds for most formats until v0.10.0.
- **RCD** (ratio corrected demosaicing, by Luis Sanz Rodríguez) works on ratios between the colours rather than differences, which holds better in bright and dark areas alike, and is known for few artefacts on fine detail; darktable and RawTherapee offer it.
- **Markesteijn**, after Frank Markesteijn, is the method made for X-Trans, in one pass or, slower and cleaner, three.
- **Fotollection's demosaic since v0.10.0** takes RCD's way of deciding direction, then estimates green from each side corrected by the site's own colour (as Hamilton and Adams did), weighted against how much each side changes. We wrote RCD's own ratio step first, and it scored worse on our charts, so the variant that won is the one that ships, under its own name.

When a method guesses wrong, it shows in four typical ways:

- **zipper:** a row of alternating light and dark pixels along a sharp edge, where the method picked neighbours from both sides;
- **maze:** a fine worm-like pattern in flat areas, when the two greens of a Bayer square do not read quite the same (they sit on rows with different neighbours, and on many sensors differ by a percent or two);
- **false colour:** coloured dots or fringes on detail that is really grey, such as fabric, hair, a brick wall or a distant fence;
- **moiré:** coloured waves over very fine repeating patterns, finer than the colour filter can sample.

**How we judge one.** Our eyes are easily fooled by a sharper-looking result, so every method is scored on pictures whose truth is known exactly. A computer draws five charts at 768 px: a zone plate (rings that get finer towards the edge, up to the finest detail the green photosites can hold), a Siemens star with 72 spokes, bars of saturated colour on grey in three directions, smooth colour ramps, and a flat grey with the two greens 2 % apart. Each is turned into what a Bayer sensor would record, demosaiced, and compared with the original. PSNR measures the overall error in decibels (higher is better; every 6 dB halves it); false colour is the average colour that appears on the grey charts; the edge error is the average error within 2 px of an edge.

| Chart (768 px) | Bilinear | Malvar (ours until v0.10.0) | Ours since v0.10.0 |
|---|---|---|---|
| Zone plate: PSNR, higher is better | 13.0 dB | 17.3 dB | 16.5 dB |
| Zone plate: false colour, lower is better | 65.9 | 43.0 | 18.7 |
| Siemens star: PSNR | 21.3 dB | 25.2 dB | 37.9 dB |
| Siemens star: false colour | 9.4 | 8.4 | 0.16 |
| Saturated colour bars: error along the edges, lower is better | 11.6 | 21.6 | 10.5 |
| Smooth ramps: PSNR | 78.9 dB | 77.5 dB | 78.4 dB |
| Flat grey with a green gap: maze, lower is better | 0.38 | 0.38 | 0 |
| Texture with stuck pixels: left per 10,000, lower is better | 35.8 | 66.5 | 0 |

Malvar is about 4 dB better than bilinear on fine grey detail, as its paper promises, but worse along edges between strong colours, because its correction borrows the detail of one colour for the others and assumes they change together, which a red sign on a grey wall does not. Neither removes the maze, which is not a demosaicing problem but a difference between the two greens, to be evened out before. Those are the gaps our chain closes since v0.10.0: stuck pixels and the green imbalance are fixed while each site still holds only what it measured, the directional demosaic rebuilds the colours, and a small median on the colour differences removes the false colour left. On the charts it wins everywhere but the zone plate's overall PSNR, 0.8 dB lower than Malvar's near the finest detail, where every method aliases; even there its false colour halves. On the nine real files it brings every Bayer camera's develop as close or closer to the camera's own JPEG, and costs about 180 ms on 24 MP, against Malvar's 40: a few tenths of a second more per photo, for prints with cleaner detail.

### From the camera's colours to yours

The camera's red, green and blue are not those of your screen: every sensor's filters use dyes of their own, and they overlap differently from the cones of our eyes. The colour matrices in the file turn the camera's colours into **CIE XYZ**, the standard model of human colour vision defined in 1931, and from there into **sRGB**, the colour space of most screens and of photo labs.

- **White balance** comes first: a gain on red and on blue, so that something white in the scene comes out neutral. The camera records the gains it chose, and Fotollection applies them before demosaicing, so the colour differences the methods above rely on stay small.
- A DNG usually carries **two matrices**, one measured under warm tungsten light and one under daylight. The right one for your photo lies in between: the developer works out the light of the shot from its white balance, as a colour temperature, and blends the two. Fotollection does the same for the other formats, from the matrices rawler knows for each camera model.
- All of this happens in **linear light**, on the sensor's own scale, before any curve. Mixing, scaling and resizing colours in linear light is what gives physically right results; doing it after the curve darkens edges and shifts colours.

**The camera's look.** A camera's own JPEG is not neutral: each maker adds its contrast, its colour and its skin tones, and that look is part of why people choose a brand. Since v0.10.0 a RAW photo opens with a **neutral** rendering, faithful to the sensor's matrix, with the lens shading and the exposure the camera chose; **Like the camera's JPEG** is one click away in a RAW photo's *RAW colour* card, for a photo or for every RAW at once. (Until v0.10.0 every RAW was fitted to the camera's JPEG; a project saved then keeps that look.) That choice fits the camera's JPEG both ways, one curve and a matrix or one curve per channel, and keeps whichever comes closer for that file: on the Nikon D750 it halves the difference, 2.26 to 1.16 in ΔE2000. The grid's thumbnails keep the camera's own preview, as in Lightroom.

### Tone: from linear numbers to a picture

Linear numbers look dark and dull on a screen, because a doubling of light, which the sensor records as twice the number, looks to us like one step of brightness. A **tone curve** turns the sensor's scale into a picture: it places middle grey, lifts the shadows, rolls the highlights off gently instead of clipping them at once, and gives the image its contrast. The picture is then stored with the sRGB curve (its "gamma"), which spends more of a JPEG's 256 levels on the shadows, where our eyes notice steps the most.

Two corrections a camera makes in its JPEG are not in the raw data, and both matter for prints:

- **Lens shading:** every lens is darker towards the corners. Fotollection measures how much the camera brightened them from its own preview, as a radial gain. When our DNG develop was first compared with the camera's JPEG, the centre was about 4 L\* too bright and the corners 1.5 too dark; the radial gain took the difference from ΔE 3.8 to 2.67, with every region of the photo within ±0.5 L\*.
- **Distortion:** the slight barrel or pincushion shape of a lens. Micro Four Thirds cameras (Panasonic, Olympus) are designed for the camera to correct it, and their files record how. On a Mac those files are developed by macOS, which applies the correction; correcting it in our own develop is planned (FR-085).

### How Fotollection develops a RAW, step by step

For a full develop, the one behind the print preview, zoom and export:

1. **Recognise** the format from the file's first bytes, never its name, so a renamed file or a RAW that looks like a plain TIFF is still read right.
2. **Read** the sensor's numbers: DNG with our own reader, the other formats unpacked by rawler. A file too large or damaged is refused before any pixel is decoded, with an error code.
3. **Levels:** black to 0 and white to 1, for each position of the colour filter, counted from the corner of the active area.
4. **White balance**, then **demosaic**: stuck pixels and the green balance first, the directional demosaic, the false-colour median (Markesteijn for X-Trans comes next).
5. **Colour:** the camera's matrices, blended for the light of the shot, in linear light.
6. **Tone and look**, fitted to the camera's preview, with its lens shading.
7. From here a RAW photo is like any other: crop, date stamp, frame, print size and colour profile, exported at full resolution in 4:4:4 at quality 95.

A few rules hold throughout. Only one develop runs at a time, spread over every processor core, and never on the window's own thread, so Fotollection keeps answering while a 30 MP photo develops. Exports always run on the processor, so a print comes out the same on every computer; the graphics card only speeds up what you see while editing. And the code is our own, written from the published specifications and papers, not copied from other projects, then checked: on the synthetic charts above, on nine RAW files from nine cameras (CC0, from raw.pixls.us), and by cutting and corrupting files on purpose so a bad file is an error, never a crash.

**Since v0.10.0: our own develop for every Bayer camera.** rawler only unpacks the files, and the rest is Fotollection's. Full develop of the nine sample files, median of 3, on the same laptop: our develop with its cleaner demosaic, against rawler's simpler one and macOS's:

| Camera (format) | Fotollection's own | rawler | macOS |
|---|---|---|---|
| Panasonic GX9 (RW2, 20 MP) | 280 ms | 216 ms | 436 ms |
| Panasonic S5 (RW2, 24 MP) | 442 ms | 301 ms | 1,044 ms |
| Olympus E-M10 III (ORF, 16 MP) | 308 ms | 364 ms | 1,164 ms |
| Sony A7 III (ARW, 24 MP) | 372 ms | 241 ms | 488 ms |
| Canon R10 (CR3, 24 MP) | 494 ms | 360 ms | 1,647 ms |
| Ricoh GR III (DNG, 24 MP) | 523 ms | 650 ms | — |
| Nikon D750 (NEF, 24 MP) | 502 ms | 402 ms | 412 ms |
| Canon 5D Mark IV (CR2, 31 MP) | 691 ms | 565 ms | 418 ms |
| Fujifilm X-T3 (RAF, 26 MP, X-Trans) | next | 524 ms | 1,339 ms |

Ours takes a tenth to a quarter of a second more than rawler's for its cleaner demosaic and its fitted colour, and stays quicker than macOS on most cameras; it happens once per photo, and a slider then re-finishes a kept copy in 4 to 11 ms. On a Mac, the Panasonic and Olympus files stay with macOS for its lens correction until our own develop corrects distortion. The S5's grey patches come out more neutral than through rawler (a colour cast of 1.42 against 2.43, where 0 is perfectly grey). Fujifilm X-Trans follows with Markesteijn, and after it the develop on the graphics card for the preview, with the processor's result as its reference.
