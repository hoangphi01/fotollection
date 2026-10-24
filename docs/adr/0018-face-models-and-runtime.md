# ADR-0018: Face and eye models for best-shot suggestions, run with tract

- **Status:** Accepted
- **Date:** 2026-10-24
- **Related:** FR-041, NFR-008, NFR-014, ADR-0008, ADR-0016

## Context

Best-shot suggestions ([FR-041](../PRD.md#fr-041--best-shot-suggestions)) must notice closed eyes and judge sharpness where the faces are. That needs a face detector and enough facial landmarks to measure eye openness. The project's constraints narrow the choice:

- **Licence:** permissive weights only. The project may be relicensed later ([ADR-0008](0008-license-gpl3-with-relicensing-option.md)), so non-commercial or unclear licences are out.
- **Offline:** models ship inside the app, and nothing is downloaded.
- **Speed:** analysis must stay within [NFR-014](../PRD.md#nfr-014--best-shot-analysis-speed), and the installer must stay small ([NFR-008](../PRD.md#nfr-008--install-size)).
- **Build:** no native runtime that complicates the macOS and Windows builds.

## Options considered

**Models:**

1. **YuNet** (OpenCV Zoo) for faces. The model folder states MIT for all its files, weights included. It is 0.23 MB.
2. **MediaPipe FaceMesh V2** for 478 landmarks. Its model card states Apache-2.0. It is 2.5 MB as TFLite and 4.9 MB as ONNX. Eye openness then comes from the eye aspect ratio (EAR) of the eyelid landmarks, so the separate Blendshape model (also Apache-2.0) is not needed.
3. **Rejected:**
   - InsightFace/SCRFD and the pyiqa models (TOPIQ, LIQE, CLIP-IQA): non-commercial licences.
   - OpenVINO `open-closed-eye-0001`: no licence stated for the weights themselves.

**Runtime:**

1. **`tract-onnx`** (MIT/Apache, pure Rust): no native library and one code path on both platforms.
2. **`ort`** (ONNX Runtime): faster, but it bundles a large native library per platform.
3. **`tract-tflite`:** it cannot load FaceMesh, because the fp16 weights sit behind `DEQUANTIZE` ops it does not support, and its op registry is not extensible from outside.

## Decision

Use **YuNet + FaceMesh V2** through **`tract-onnx`**.

- **YuNet** ships as published, ONNX opset 11 with a 640×640 input.
- **FaceMesh** is converted once from TFLite to ONNX with `tf2onnx 1.16.1` / TensorFlow 2.15.1, using `scripts/models/fetch-and-convert.sh`.
- Both models are committed under `crates/core/assets/models/`, with their licence files and [`MODELS.md`](../../crates/core/assets/models/MODELS.md). They are compiled into the binary and checked against pinned BLAKE3 sums when loaded.

The spike on REF-MAC (M3), single thread:

| Model | Load and optimize | One run | Accuracy |
|-------|------------------:|--------:|----------|
| YuNet, 640×640 | 72 ms | ≈ 19 ms | — |
| FaceMesh, one 256×256 face | 113 ms | ≈ 9.6 ms | Largest difference from the TFLite interpreter: 0.00045 px, on coordinates up to 180 px |

Integration (B1) changed how YuNet runs: tract is told to ignore the shapes stored in the file, so YuNet takes the photo's own shape at a 512 px long edge (padded to multiples of 32; portrait photos padded to a square, which tract runs faster than a tall input). That cut it from ≈ 19 to ≈ 8 ms and kept analysis within NFR-014 ([OPTIMIZATION.md §5](../OPTIMIZATION.md#5-benchmark-log)).

## Consequences

- The installer grows by about 5 MB of model data, plus tract.
- The conversion is reproducible in function, but not byte for byte: tf2onnx randomizes node names. The committed file, pinned by checksum, is the reference. The script verifies any new conversion against the TFLite interpreter.
- FaceMesh is meant for faces seen from the front. Faces smaller than 64 px, turned more than about 80°, or less than half visible skip the eye check instead of guessing.
- Per the model card, the landmarks are used only to measure eye openness. Nothing identifies a person or stores a face representation.
- Re-check the licences whenever a model is replaced ([SECURITY.md §6](../SECURITY.md#6-supply-chain)).
