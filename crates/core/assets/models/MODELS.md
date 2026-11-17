# Bundled models

The models used by best-shot suggestions ([FR-041](../../../../docs/PRD.md#fr-041--best-shot-suggestions), [ADR-0018](../../../../docs/adr/0018-face-models-and-runtime.md)). They are compiled into the app, are checked against the BLAKE3 sums below when loaded, and are **never downloaded at runtime**. `scripts/models/fetch-and-convert.sh` rebuilds them from upstream.

| File | What it does | Source | Licence |
|------|--------------|--------|---------|
| `yunet.onnx` | Face detector with 5 landmarks, input 1×3×640×640 | OpenCV Zoo, `models/face_detection_yunet/face_detection_yunet_2023mar.onnx`, used as published | MIT, © 2020 Shiqi Yu (`LICENSE-YuNet.txt`) |
| `facemesh.onnx` | 478 3D face landmarks from a 256×256 face crop, used to measure eye openness | MediaPipe Face Landmarker bundle (`face_landmarker.task`, float16), file `face_landmarks_detector.tflite` (FaceMesh V2), converted to ONNX | Apache-2.0, © Google (`LICENSE-MediaPipe.txt`; model card "MediaPipe FaceMesh", licensed under Apache License 2.0) |

## Checksums

| File | BLAKE3 (checked at load) | SHA-256 |
|------|--------------------------|---------|
| `yunet.onnx` | `3d5938c4cd5a02dc416f1cd1f7fc1f662a22adc370477112c871954587e63431` | `8f2383e4dd3cfbb4553ea8718107fc0423210dc964f9f4280604804ed2552fa4` |
| `facemesh.onnx` | `e01c5f5c50e4dc2020947037d2a08cada1df73a9dc7f14f0c9d464c4ba2c23fa` | `1ff6e0d1e52d4e2c29bb7d138020374b1db2c24f47e69e72fb8f9a10728d3738` |

Upstream downloads used on 2026-10-24:

| Download | SHA-256 |
|----------|---------|
| `face_detection_yunet_2023mar.onnx` | `8f2383e4dd3cfbb4553ea8718107fc0423210dc964f9f4280604804ed2552fa4` |
| `face_landmarker.task` (float16, latest) | `64184e229b263107bc2b804c6625db1341ff2bb731874b0bcc2fe6544e0bc9ff` |
| `face_landmarks_detector.tflite` inside it | `c7d54204ce0448474c7f3fa9af494787c0965cbdd6f20fc72867e43046bd43d5` |

## Conversion

The TFLite model stores fp16 weights behind `DEQUANTIZE` ops, which `tract-tflite` cannot load, so it is converted once with `tf2onnx 1.16.1` (TensorFlow 2.15.1, Python 3.9, `--opset 13`). The conversion is **functionally** reproducible but not byte-identical, because tf2onnx names nodes randomly. That is why the committed file is pinned by its checksum.

The script checks every conversion against the TFLite interpreter on a fixed random input. For the committed file, the largest difference over the 1,434 landmark values was **0.00045 px**, on coordinates up to 180 px.

## CLIP candidates (FR-141 S6, measured, not bundled yet)

`scripts/models/clip-export.sh <out> [--check <images>]` exports three image towers for the measurement, each with its labels' text embeddings computed once (`<name>.labels.json`); the app would run the image tower only. In a throwaway uv environment: torch 2.5.1 (BSD-3-Clause), transformers 4.46.3 (Apache-2.0), onnx 1.17.0 (Apache-2.0), onnxruntime 1.20.1 (MIT). Nothing here is in the app until the owner chooses (S6b); `crates/core/examples/clip_bench.rs` measures them.

| Name | Upstream (Hugging Face, commit) | Licence | fp32 ONNX | int8 (MatMul/Gemm) |
|------|--------------------------------|---------|-----------|--------------------|
| `clip-b32` | `openai/clip-vit-base-patch32` | MIT | 351.6 MB, `7429cef7…` | 95.8 MB, `d114d9e6…` |
| `tinyclip-39m` | `wkcn/TinyCLIP-ViT-39M-16-Text-19M-YFCC15M`, `07a4b0bc` | MIT (Microsoft) | 154.6 MB, `87d279c4…` | 40.7 MB, `e52c786a…` |
| `tinyclip-8m` | `wkcn/TinyCLIP-ViT-8M-16-Text-3M-YFCC15M`, `a2a8c6ea` | MIT (Microsoft) | 33.3 MB, `58600187…` | 9.4 MB, `3a96a480…` |

Each export is checked against PyTorch on random inputs (fp32 cosine 0.9999998; int8 0.989 for B/32, 0.998 for the TinyCLIPs). SHA-256 prefixes; the full sums are in the export's report.

## Use limits from the model cards

FaceMesh is meant for faces seen from the front: less than 80° away from the camera, less than 8° roll, and more than half visible. Fotollection only uses it to measure whether eyes are open. It never identifies people or stores a face representation, which the model card lists as out of scope.
