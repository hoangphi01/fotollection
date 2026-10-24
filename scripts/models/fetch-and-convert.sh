#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Rebuilds the bundled face models (crates/core/assets/models, ADR-0018) from upstream.
# Downloads are checked against pinned SHA-256 sums; the FaceMesh conversion is checked
# against the TFLite interpreter. Needs Python 3.9–3.11 and ~2 GB of temporary disk space.
# The app itself never downloads anything.
#
#   scripts/models/fetch-and-convert.sh [out-dir=crates/core/assets/models]
set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
out="${1:-$root/crates/core/assets/models}"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

YUNET_URL="https://github.com/opencv/opencv_zoo/raw/main/models/face_detection_yunet/face_detection_yunet_2023mar.onnx"
YUNET_SHA256="8f2383e4dd3cfbb4553ea8718107fc0423210dc964f9f4280604804ed2552fa4"
TASK_URL="https://storage.googleapis.com/mediapipe-models/face_landmarker/face_landmarker/float16/latest/face_landmarker.task"
TFLITE_SHA256="c7d54204ce0448474c7f3fa9af494787c0965cbdd6f20fc72867e43046bd43d5"

check() { # check <file> <sha256>
  echo "$2  $1" | shasum -a 256 -c - >/dev/null || { echo "checksum mismatch: $1" >&2; exit 1; }
}

echo "downloading" >&2
curl -sSL -o "$work/yunet.onnx" "$YUNET_URL"
check "$work/yunet.onnx" "$YUNET_SHA256"
curl -sSL -o "$work/face_landmarker.task" "$TASK_URL"
unzip -q -o "$work/face_landmarker.task" face_landmarks_detector.tflite -d "$work"
# "latest" can move: the model inside must still be the one we checked.
check "$work/face_landmarks_detector.tflite" "$TFLITE_SHA256"

echo "converting FaceMesh to ONNX" >&2
python3 -m venv "$work/venv"
"$work/venv/bin/pip" install -q "tensorflow==2.15.1" "tf2onnx==1.16.1"
"$work/venv/bin/python" -m tf2onnx.convert --tflite "$work/face_landmarks_detector.tflite" \
  --output "$work/facemesh.onnx" --opset 13 >/dev/null 2>&1

echo "checking the conversion against the TFLite interpreter" >&2
"$work/venv/bin/pip" install -q "onnxruntime==1.19.2"
"$work/venv/bin/python" -I - "$work" <<'PY'
import sys, numpy as np, tensorflow as tf, onnxruntime as ort
d = sys.argv[1]
np.random.seed(7)
x = np.random.rand(1, 256, 256, 3).astype(np.float32)
it = tf.lite.Interpreter(model_path=f"{d}/face_landmarks_detector.tflite")
it.allocate_tensors()
it.set_tensor(it.get_input_details()[0]["index"], x)
it.invoke()
ref = max((it.get_tensor(o["index"]) for o in it.get_output_details()), key=lambda v: v.size)
sess = ort.InferenceSession(f"{d}/facemesh.onnx")
got = max(sess.run(None, {sess.get_inputs()[0].name: x}), key=lambda v: v.size)
diff = float(np.abs(ref.reshape(-1) - got.reshape(-1)).max())
print(f"max difference {diff:.6f}")
sys.exit(0 if diff < 0.01 else 1)
PY

cp "$work/yunet.onnx" "$work/facemesh.onnx" "$out/"
echo "written to $out. Update the BLAKE3 sums in MODELS.md and crates/core/src/culling/models.rs." >&2
