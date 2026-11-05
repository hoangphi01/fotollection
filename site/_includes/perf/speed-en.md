### What happens when you open a folder

A folder of photos is not read whole. For each file, Fotollection reads its EXIF data (date, camera, place) and the small thumbnail every camera stores inside the file, and nothing else: 1,000 photos take 0.12 s, against a budget of 1 s set before the code was written. Sharper 400 px thumbnails are then made in the background, on two threads so the app stays quick, and kept in a cache on disk, so a folder opened again is instant. For 5,035 photos, the import took 383 ms and the embedded thumbnails 236 ms.

### Where a preview's 17 ms go

Changing a setting re-renders the preview from a 2,048 px copy of the photo kept in memory, never from the full file. Of the 16.7 ms it takes, cropping and resizing with a Lanczos filter take 9.7 ms, the date stamp under 0.1 ms, and encoding the preview as a JPEG for the screen 7.0 ms. That encoder was tuned by measurement: with the export's settings (full colour resolution and optimised tables) it took 16.2 ms; the screen does not need them, so the preview uses halved colour and plain tables, while exports keep full colour (4:4:4). A cheaper resize filter was tried and turned out slower on this processor (12.5 ms), so Lanczos stayed.

### Native code against a browser

The same job (decode, crop to 2:3, date stamp, JPEG at quality 95) was run on the same 95 photos in Fotollection and in Chromium with one worker per core. Export speed came out the same, 18.0 against 17.9 photos a second, which proved our first guess wrong: a browser is not slow. The reasons for native code are elsewhere: Fotollection writes 4:4:4 files at full colour resolution where the browser's encoder halves colour, it keeps each photo's colour profile, it writes files straight to disk, and it used a quarter less memory (1.35 GB against 1.78 GB). Since then the print encoder was made faster still: 23.5 photos a second at 16 MP.
