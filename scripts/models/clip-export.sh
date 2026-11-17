#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# FR-141 S6: exports the CLIP candidates (OpenAI CLIP ViT-B/32 and two TinyCLIP, all MIT)
# for the measurement, with their labels' text embeddings (clip_export.py). Downloads from
# Hugging Face; needs uv and about 3 GB of temporary disk space. The app never downloads.
#
#   scripts/models/clip-export.sh <out-dir> [--check <images-dir>]
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
out="${1:?usage: clip-export.sh <out-dir> [--check <images-dir>]}"
shift
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

uv venv -q --python 3.12 "$work/venv"
uv pip install -q --python "$work/venv/bin/python" \
  --index-url https://download.pytorch.org/whl/cpu "torch==2.5.1"
uv pip install -q --python "$work/venv/bin/python" \
  "transformers==4.46.3" "onnx==1.17.0" "onnxruntime==1.20.1" "pillow==11.0.0" "numpy==2.1.3"
"$work/venv/bin/python" "$here/clip_export.py" "$out" "$@"
