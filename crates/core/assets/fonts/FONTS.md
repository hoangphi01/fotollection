# Bundled fonts

The faces compiled into the app (`include_bytes!`), all under the SIL Open Font License 1.1, each with its licence beside it. They are never downloaded at runtime.

| File | What it draws | Source | Licence |
|------|---------------|--------|---------|
| `DSEG14Classic-Italic.ttf` | The date stamp, a film camera's LCD ([FR-004](../../../../docs/PRD.md#fr-004--date-stamp)) | [keshikan/DSEG](https://github.com/keshikan/DSEG) | OFL 1.1, © keshikan (`DSEG-OFL.txt`) |
| `Inter.ttf` | The info frame ([FR-040](../../../../docs/PRD.md#fr-040--exif-info-frame)), and the letters a postcard face lacks (Greek, Cyrillic) | [rsms/inter](https://rsms.me/inter/) | OFL 1.1, © The Inter Project Authors (`Inter-OFL.txt`) |
| `XanhMono-Regular.ttf`, `XanhMono-Italic.ttf` | Postcard text, the typewriter face ([FR-073](../../../../docs/PRD.md#fr-073--postcard-sheets)) | [yellow-type-foundry/xanhmono](https://github.com/yellow-type-foundry/xanhmono) | OFL 1.1, © Yellow Type Foundry (`XanhMono-OFL.txt`) |
| `PatrickHand-Regular.ttf` | Postcard text, the handwriting face ([FR-076](../../../../docs/PRD.md#fr-076--a-handwriting-face-for-postcards)) | [google/fonts `ofl/patrickhand`](https://github.com/google/fonts/tree/main/ofl/patrickhand), downloaded 2026-11-04 | OFL 1.1, © 2010–2012 Patrick Wagesreiter (`PatrickHand-OFL.txt`); also every Latin letter on a Japanese or Korean card (FR-130, FR-132) |
| `KleeOne-SemiBold-subset.ttf` | The はがき's Japanese text, set across or top to bottom with its own vertical forms ([FR-130](../../../../docs/PRD.md#fr-130--the-japanese-postcard)) | [google/fonts `ofl/kleeone`](https://github.com/google/fonts/tree/main/ofl/kleeone) (Fontworks' Klee), downloaded 2026-11-15, the SemiBold (the Regular's strokes are 0.21 mm at the medium size, under the 0.25 mm below; the SemiBold's 0.27), cut by `scripts/cjk-font.sh print-ja` to JIS X 0208 (6,356 kanji, kana, punctuation; every character of Japan Post's postal data) and ASCII, its `vert` and `vrt2` kept: 4.7 MB of 8.9 MB | OFL 1.1, © 2020 The Klee Project Authors (`KleeOne-OFL.txt`) |
| `NanumPenScript-Regular-subset.ttf` | The 우편엽서's text in the pen hand, and the letters Gaegu lacks ([FR-132](../../../../docs/PRD.md#fr-132--the-korean-postcards-address-side-and-koreas-print-sizes)) | [google/fonts `ofl/nanumpenscript`](https://github.com/google/fonts/tree/main/ofl/nanumpenscript), downloaded 2026-11-15, cut by `scripts/cjk-font.sh print-ko` to all 11,172 Hangul syllables, the jamo, CJK and general punctuation and ASCII: 2.0 MB of 3.2 MB; strokes 0.33 mm at the medium size | OFL 1.1, © 2010 NHN Corporation (`NanumPenScript-OFL.txt`) |
| `Gaegu-Regular-subset.ttf` | The 우편엽서's text in the rounder hand (FR-132) | [google/fonts `ofl/gaegu`](https://github.com/google/fonts/tree/main/ofl/gaegu), downloaded 2026-11-15, cut the same way: its 2,350 KS X 1001 syllables, 1.6 MB of 3.2 MB; strokes 0.25 mm at the medium size | OFL 1.1, © 2018 The Gaegu Project Authors (`Gaegu-OFL.txt`) |

A postcard face must hold every letter of the European languages written in Latin script and of Vietnamese, and its strokes must be at least as thick as Xanh Mono's regular at the medium size (about 0.25 mm), which the owner's print test of 2026-11-04 found the thinnest still readable on a kiosk print (`postcard::tests::fr_076_*`).

## Checksums (SHA-256)

| File | SHA-256 |
|------|---------|
| `DSEG14Classic-Italic.ttf` | `bd65525be4282621f2b79f7e4344007f1a1f51f5311fee1d77ede77919825218` |
| `Inter.ttf` | `29160a80ff49ddcab2c97711247e08b1fab27a484a329ce8b813d820dc559031` |
| `XanhMono-Regular.ttf` | `33e64b43fdcb40bd0141853601c7aaffd5be15ebd3cf56340d2473ff8be8e69c` |
| `XanhMono-Italic.ttf` | `e74ed4105efc797c05ec0631182e0a76bb5ecbf1b6adedb9e0170c79964b2f6f` |
| `PatrickHand-Regular.ttf` | `0f173b3e6cb6d1af25babf7f0057c5ac4ee11f9992b0469bb817e967ef4ad0fc` |
| `KleeOne-SemiBold-subset.ttf` | `c93a6ac4dbbae1fd187ae3f2a867d3f3f649fb5ceb2ac756b32efb1352e4f9b1` (source `KleeOne-SemiBold.ttf`, SHA-1 `6a12ad3c1442e15fa4f422fae55e708cf8c00b26`) |
| `NanumPenScript-Regular-subset.ttf` | `b9cca2f5f9d411d3889cf4c9744c9804fde6dd27772bd1885f26f58e5d77471c` (source SHA-1 `e99544f1718d51d5c4baa1ad7a1e5d291f61f475`) |
| `Gaegu-Regular-subset.ttf` | `6ec585d30f74c8bc3192d37c4f8ac0f465cb7f6e57e6936ae4099bb09fc620a2` (source SHA-1 `9762f395c88146197b2a8f5cafc1daf6acc72f88`) |
