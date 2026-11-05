### Why prints from a web editor look flat

The test exports 20 camera photos and 5 Display P3 versions (the wider colour of recent phones and Macs) through Fotollection's real export, then reads both files with an independent decoder, converts them to CIELAB through their colour profiles and compares them pixel by pixel with the original. A typical web-editor export goes through the same comparison: profile dropped, resized to 2,000 px and enlarged back for the print, saved at quality 80 with halved colour (4:2:0).

| Median | Fotollection, sRGB | Web editor | Fotollection, Display P3 | Web editor |
|---|---|---|---|---|
| ΔE2000 (colour error, under 1 is invisible) | 0.19 | 1.39 | 0.31 | 2.15 |
| PSNR | 52.6 dB | 37.9 dB | 50.8 dB | 37.0 dB |
| Saturation kept | 100 % | 99.9 % | 99.9 % | 86.8 % |
| Fine detail kept | 99.8 % | 13 % | 100 % | 18 % |

Two causes explain the flat look. Shrinking to 2,000 px and enlarging again for the lab throws away 87 % of the fine detail that makes a print look crisp. And dropping the colour profile reads a Display P3 photo as if it were sRGB, which takes 13 % of its saturation. Fotollection keeps the profile (25 of 25 files), exports at the photo's full resolution with full colour (4:4:4) at quality 95, and its larger errors sit only on sharp edges, where any JPEG works hardest. Still to come: real iPhone HEIC files, and prints measured with a spectrophotometer.
