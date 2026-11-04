# Bundled fonts

The faces compiled into the app (`include_bytes!`), all under the SIL Open Font License 1.1, each with its licence beside it. They are never downloaded at runtime.

| File | What it draws | Source | Licence |
|------|---------------|--------|---------|
| `DSEG14Classic-Italic.ttf` | The date stamp, a film camera's LCD ([FR-004](../../../../docs/PRD.md#fr-004--date-stamp)) | [keshikan/DSEG](https://github.com/keshikan/DSEG) | OFL 1.1, © keshikan (`DSEG-OFL.txt`) |
| `Inter.ttf` | The info frame ([FR-040](../../../../docs/PRD.md#fr-040--exif-info-frame)), and the letters a postcard face lacks (Greek, Cyrillic) | [rsms/inter](https://rsms.me/inter/) | OFL 1.1, © The Inter Project Authors (`Inter-OFL.txt`) |
| `XanhMono-Regular.ttf`, `XanhMono-Italic.ttf` | Postcard text, the typewriter face ([FR-073](../../../../docs/PRD.md#fr-073--postcard-sheets)) | [yellow-type-foundry/xanhmono](https://github.com/yellow-type-foundry/xanhmono) | OFL 1.1, © Yellow Type Foundry (`XanhMono-OFL.txt`) |
| `PatrickHand-Regular.ttf` | Postcard text, the handwriting face ([FR-076](../../../../docs/PRD.md#fr-076--a-handwriting-face-for-postcards)) | [google/fonts `ofl/patrickhand`](https://github.com/google/fonts/tree/main/ofl/patrickhand), downloaded 2026-11-04 | OFL 1.1, © 2010–2012 Patrick Wagesreiter (`PatrickHand-OFL.txt`) |

A postcard face must hold every letter of the European languages written in Latin script and of Vietnamese, and its strokes must be at least as thick as Xanh Mono's regular at the medium size (about 0.25 mm), which the owner's print test of 2026-11-04 found the thinnest still readable on a kiosk print (`postcard::tests::fr_076_*`).

## Checksums (SHA-256)

| File | SHA-256 |
|------|---------|
| `DSEG14Classic-Italic.ttf` | `bd65525be4282621f2b79f7e4344007f1a1f51f5311fee1d77ede77919825218` |
| `Inter.ttf` | `29160a80ff49ddcab2c97711247e08b1fab27a484a329ce8b813d820dc559031` |
| `XanhMono-Regular.ttf` | `33e64b43fdcb40bd0141853601c7aaffd5be15ebd3cf56340d2473ff8be8e69c` |
| `XanhMono-Italic.ttf` | `e74ed4105efc797c05ec0631182e0a76bb5ecbf1b6adedb9e0170c79964b2f6f` |
| `PatrickHand-Regular.ttf` | `0f173b3e6cb6d1af25babf7f0057c5ac4ee11f9992b0469bb817e967ef4ad0fc` |
