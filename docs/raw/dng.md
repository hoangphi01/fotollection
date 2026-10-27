# DNG

Specification: Adobe's *Digital Negative Specification* 1.7 (public), on top of TIFF 6.0 and TIFF/EP. Lossless JPEG from ITU T.81 (process 14). Code: `crates/rawcore/src/{tiff,dng,ljpeg,cfa,color}.rs`.

## What is read

- **The raw image:** the IFD (IFD0 or a SubIFD) with `NewSubfileType` 0 and `PhotometricInterpretation` 32803 (CFA). A linear DNG (34892) is refused, so the next decoder takes it.
- **Filter:** `CFARepeatPatternDim` must be 2×2 and `CFAPattern` a Bayer one (one red, one blue, the greens on a diagonal).
- **Compression:** 1 (strips; 8 or 16 bits as stored, other depths packed most significant bit first, each row starting on a byte) and 7 (lossless JPEG, strips or tiles, decoded in parallel). Others (lossy 34892, deflate floating point) are refused.
- **Levels:** `LinearizationTable`, `BlackLevel` with its repeat dimension (up to 2×2), `WhiteLevel`.
- **Geometry:** `ActiveArea`, then `DefaultCropOrigin`/`DefaultCropSize`, which count from the active area. The develop has exactly the crop's size, and `dimensions` reads the same tags without decoding.
- **Colour:** `ColorMatrix1/2`, `ForwardMatrix1/2`, `CameraCalibration1/2`, `CalibrationIlluminant1/2`, `AnalogBalance`, `AsShotNeutral`, `BaselineExposure`.
- **Embedded JPEG:** the largest IFD with `NewSubfileType` 1 and JPEG compression, used for the preview and to learn the camera's look.

## Colour, as the specification gives it

1. The shot's white is `AsShotNeutral`. Its temperature is found by iteration: guess a temperature, blend the two colour matrices by inverse temperature, take the white to XYZ, take its CCT (McCamy), repeat.
2. With forward matrices: camera RGB, balanced so the neutral is (1, 1, 1), goes to XYZ D50 through the blended `ForwardMatrix`. Without them: the inverse of the blended `ColorMatrix`, then Bradford from the white to D50.
3. XYZ D50 to linear sRGB (Bradford-adapted to D65), scaled by `2^BaselineExposure`.

White balance is applied before the demosaic (Malvar–He–Cutler), and sites are clipped at the white level so blown highlights stay neutral.

## Not done yet

- Opcode lists (`OpcodeList1/2/3`): gain maps, lens distortion (WarpRectilinear), vignette correction. The lens shading is learned from the embedded JPEG instead (ADR-0030).
- Camera profiles in the file (`ProfileHueSatMap`, `ProfileLookTable`, `ProfileToneCurve`).
- Uncompressed tiles are not read yet. Packed depths are written from the specification but not seen in a sample.
