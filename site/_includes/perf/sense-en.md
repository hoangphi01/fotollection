### A load test, not the usual day

Most people sort one trip at a time, 2,000 to 3,000 photos. To see whether Fotollection stays steady far beyond that, RollSense was given 10,044 photos at once on an ordinary Windows laptop from 2020 (Lenovo ThinkPad X1, Intel i7-10610U, 15 W, 16 GB): camera JPEGs, phone photos, screenshots and Canon and Sony RAW files. It read 9,639 of them in 70 minutes on every core, the processor busy the whole time and the memory between 400 and 850 MB, never growing.

### Where the time goes

Each photo is decoded once, made small and read by the subject model. The model alone takes 128 ms a photo on this laptop and the whole step 431 ms, so about 70 % of the time is reading and decoding the photo itself, mostly full-size JPEG and RAW files. The graphics card stayed idle, at most 1.5 %: all of this runs in Rust on the processor.

### More threads, and a laptop's limit

Two threads take 614 ms a photo, four 498 ms and eight 438 ms. A thin laptop shares 15 watts among its cores, so with all eight busy each one runs slower (1.8 GHz on average, against 3.4 GHz at rest), and four of the eight threads share a core with the other four.

| | This laptop (measured) |
|---|---|
| A trip of 2,500 photos | 18 min |
| A phone's roll of 15,000 photos | 108 min |
| A photo read again from the cache | 1.3 ms |

Measured on 2026-11-20 with `sort_pass_bench`, telemetry every second.
