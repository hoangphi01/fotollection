# DNG

Specification: Adobe's *Digital Negative Specification* 1.7 (public), on top of TIFF 6.0 and TIFF/EP. Lossless JPEG from ITU T.81 (process 14). Code: `crates/rawcore/src/{tiff,dng,ljpeg,cfa,color}.rs`.

## What is read

- **The raw image:** the IFD (IFD0 or a SubIFD) with `NewSubfileType` 0 and `PhotometricInterpretation` 32803 (CFA). A linear DNG (34892) is refused, so the next decoder takes it.
- **Filter:** `CFARepeatPatternDim` must be 2×2 and `CFAPattern` a Bayer one (one red, one blue, the greens on a diagonal).
- **Compression:** 1 (strips; 8 or 16 bits as stored, other depths packed most significant bit first, each row starting on a byte) and 7 (lossless JPEG, strips or tiles, decoded in parallel). Others (lossy 34892, deflate floating point) are refused.
- **Levels:** `LinearizationTable`, `BlackLevel` with its repeat dimension (1×1, 1×2, 2×1 or 2×2, values row by row), `WhiteLevel`. `BlackLevelDeltaH`/`DeltaV` are not read yet.
- **Origin:** `CFAPattern` and `BlackLevel` both start at the top-left corner of the `ActiveArea`, not of the stored image (`Mosaic::phase`); an area at an odd row or column shifts both (tested on a synthetic DNG in `dng.rs`).
- **Geometry:** `ActiveArea`, then `DefaultCropOrigin`/`DefaultCropSize`, which count from the active area. The develop has exactly the crop's size, and `dimensions` reads the same tags without decoding.
- **Colour:** `ColorMatrix1/2`, `ForwardMatrix1/2`, `CameraCalibration1/2`, `CalibrationIlluminant1/2`, `AnalogBalance`, `AsShotNeutral`, `BaselineExposure`.
- **Embedded JPEG:** the largest IFD with `NewSubfileType` 1 and JPEG compression, used for the preview and to learn the camera's look.

## Colour, as the specification gives it

1. The shot's white is `AsShotNeutral`. Its temperature is found by iteration: guess a temperature, blend the two colour matrices by inverse temperature, take the white to XYZ, take its CCT (McCamy), repeat.
2. With forward matrices: camera RGB, balanced so the neutral is (1, 1, 1), goes to XYZ D50 through the blended `ForwardMatrix`. Without them: the inverse of the blended `ColorMatrix`, then Bradford from the white to D50.
3. XYZ D50 to linear sRGB (Bradford-adapted to D65), scaled by `2^BaselineExposure`.

White balance is applied before the demosaic (Malvar–He–Cutler), and sites are clipped at the white level so blown highlights stay neutral.

## DNG kinds met in the wild (researched 2026-11-03)

Prompted by Martin Evening's *DNG Pros, Cons and Myths* (Adobe, 2015) and by phone DNGs. Our reader takes one of them today, the Bayer DNG from the Adobe DNG Converter or a camera; every other kind is refused and goes to the next decoder (`ImageIO` on macOS, rawler elsewhere), whose result we do not control. FR-071 shows which kind a file is; FR-072 reads them, in the order of the last column.

| Kind | Made by | How to tell | Today | Plan (FR-072) |
|---|---|---|---|---|
| Bayer, lossless JPEG or uncompressed | DNG Converter, Leica, Pentax, Ricoh, our tests | Photometric 32803, `CFARepeatPatternDim` 2×2, Compression 1 or 7 | **Ours** | Stays ours; RCD replaces Malvar |
| Phone DNG with a gain map | Android (Pixel, Samsung…), camera apps | Bayer, and `OpcodeList2` holds `GainMap` opcodes, one per CFA channel, applied before the demosaic | Ours, **gain map ignored**: corners darker and tinted. The shading is learned from the embedded JPEG (ADR-0030), but these files rarely carry a large one | **1st.** `GainMap` from `OpcodeList2`, with caps on the opcode count and the map size (an untrusted list, SECURITY §3); needs a Pixel sample first |
| Lossy DNG | Lightroom "lossy compression", smart previews | Photometric 34892 (LinearRaw), Compression 34892, 8-bit, 3 samples | Refused | **2nd.** Already demosaiced: decode the JPEG tiles with our JPEG path, then the colour tags |
| JPEG XL DNG, Apple ProRAW | DNG Converter 1.7 ("JPEG XL"), iPhone | LinearRaw; Compression 52546 (JPEG XL) or lossless JPEG on 3 samples | Refused | **3rd.** LinearRaw with lossless JPEG first; JPEG XL needs a permissive decoder (licence check) |
| Raw image digest | DNG Converter, most writers | `RawImageDigest` / `NewRawImageDigest` (MD5) | Not checked | **4th.** A mismatch is a damaged file with its own `ERR-IMG` code, instead of a garbled develop. Read the tags' exact definition in the specification before writing it |
| HDR or panorama merge | Lightroom / Camera Raw Photo Merge | SampleFormat 3 (floating point, 16, 24 or 32 bits), Compression 8 (deflate) with Predictor 34894 or 34895; often LinearRaw | Refused | **5th.** Float samples, more than 30 stops: needs tone mapping before the curve |
| X-Trans | DNG Converter on Fujifilm RAF | `CFARepeatPatternDim` 6×6 | Refused (rawler's bilinear) | **6th,** with Markesteijn |
| Original raw embedded | DNG Converter option | `OriginalRawFileData` | Ignored, correctly | Nothing; the file is about twice as large, and the develop reads it whole into memory |
| Edited in Lightroom, settings saved into the file | Lightroom, Camera Raw | XMP `crs:` settings inside the DNG; the embedded preview is the **edited** image | The thumbnail shows the edit, our develop does not; the look match learns part of the edit; `crs:Crop` is not applied | Decide with a real sample: skip the look match when `crs:` settings exist, apply `crs:Crop` with the XMP work of FR-031 |
| A JPEG saved as DNG | Lightroom "Convert to DNG" on a JPEG | Not a raw image | Refused (not a CFA) | Unverified how Adobe stores it; check with a sample before planning |

**Not verified yet:** the exact definitions of the digest tags, and how a JPEG converted to DNG is stored. Both need the specification text or a sample, never a guess.

**Samples wanted:** a Pixel or Samsung DNG, a lossy DNG, an HDR merge DNG, a ProRAW file, a DNG edited in Lightroom.

**Sources:** Martin Evening, *DNG Pros, Cons and Myths*, Adobe blog, 2015-12-07; Adobe, [DNG 1.4 specification notes](https://webarchive.library.unt.edu/web/20130114140633oe_/http://blogs.adobe.com/lightroomjournal/2012/10/dng-1-4-specification-notes.html) (floating point, lossy 34892); [TIFF Predictor](https://awaresystems.be/imaging/tiff/tifftags/predictor.html); pixls.us, [Smartphone DNGs and lens shading](https://discuss.pixls.us/t/smartphone-dngs-and-lens-shading/24858).

## Not done yet

- Opcode lists (`OpcodeList1/2/3`): gain maps, lens distortion (WarpRectilinear), vignette correction. The lens shading is learned from the embedded JPEG instead (ADR-0030).
- Camera profiles in the file (`ProfileHueSatMap`, `ProfileLookTable`, `ProfileToneCurve`).
- Uncompressed tiles are not read yet. Packed depths are written from the specification but not seen in a sample.
