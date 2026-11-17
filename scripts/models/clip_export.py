# SPDX-License-Identifier: GPL-3.0-only
"""FR-141 S6: CLIP candidates exported for the measurement (clip_bench), never at runtime.

For each model: the image tower with its projection to ONNX (fp32), a dynamic int8 copy,
both checked against PyTorch on random inputs; the text side computed here, once, for the
labels' prompts and written as `<name>.labels.json` (the app runs the image tower only).
`--check <dir>` also writes the embeddings onnxruntime gives for the images in it, with the
model's own preprocessing, for the Rust side to be compared against.

    python clip_export.py <out-dir> [--check <images-dir>]
"""

import hashlib
import json
import os
import sys

import numpy as np
import onnxruntime as ort
import torch
from onnxruntime.quantization import QuantType, quantize_dynamic
from PIL import Image
from transformers import CLIPModel, CLIPProcessor

MODELS = {
    "clip-b32": "openai/clip-vit-base-patch32",
    "tinyclip-39m": "wkcn/TinyCLIP-ViT-39M-16-Text-19M-YFCC15M",
    "tinyclip-8m": "wkcn/TinyCLIP-ViT-8M-16-Text-3M-YFCC15M",
}

# Each label is the mean of its prompts' embeddings (CLIP's prompt ensembling).
LABELS = {
    # Where an undecided file came from.
    "src-camera": ["a photo taken with a phone camera", "a casual snapshot", "a family photo"],
    "src-screenshot": ["a screenshot of a phone screen", "a screenshot of an app", "a screenshot of a chat"],
    "src-meme": ["a meme", "an internet meme with a caption", "a funny picture with text"],
    "src-web": ["a picture downloaded from the internet", "a stock photo", "a poster or an advertisement"],
    # What it shows.
    "people": ["a photo of a person", "a photo of people", "a selfie"],
    "dog": ["a photo of a dog"],
    "cat": ["a photo of a cat"],
    "animal": ["a photo of an animal", "a photo of a bird", "a photo of a fish"],
    "scenery": ["a photo of a landscape", "a photo of a city street", "a photo of the sea", "a photo of mountains"],
    "food": ["a photo of food", "a photo of a meal", "a photo of a drink"],
    "document": ["a photo of a document", "a photo of a receipt", "a photo of a paper with text"],
    "screen": ["a screenshot", "text on a screen", "a user interface"],
    "meme": ["a meme", "an internet meme with a caption"],
}


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


class Image_(torch.nn.Module):
    def __init__(self, model):
        super().__init__()
        self.model = model

    def forward(self, pixel_values):
        return self.model.get_image_features(pixel_values=pixel_values)


def cosine(a, b):
    a = a / np.linalg.norm(a, axis=-1, keepdims=True)
    b = b / np.linalg.norm(b, axis=-1, keepdims=True)
    return (a * b).sum(-1)


def export(name, repo, out, check_dir):
    model = CLIPModel.from_pretrained(repo).eval()
    proc = CLIPProcessor.from_pretrained(repo)
    fp32 = os.path.join(out, f"{name}.onnx")
    x = torch.randn(1, 3, 224, 224)
    torch.onnx.export(Image_(model), x, fp32, input_names=["pixels"], output_names=["embedding"],
                      dynamic_axes={"pixels": {0: "n"}, "embedding": {0: "n"}}, opset_version=17)
    int8 = os.path.join(out, f"{name}.int8.onnx")
    # MatMul and Gemm hold most of a ViT's weights; ConvInteger has no CPU kernel in onnxruntime.
    quantize_dynamic(fp32, int8, weight_type=QuantType.QInt8, op_types_to_quantize=["MatMul", "Gemm"])
    xs = torch.randn(4, 3, 224, 224)
    with torch.no_grad():
        want = Image_(model)(xs).numpy()
    report = {"repo": repo, "commit": getattr(model.config, "_commit_hash", None)}
    for path in (fp32, int8):
        got = ort.InferenceSession(path).run(None, {"pixels": xs.numpy()})[0]
        report[os.path.basename(path)] = {
            "bytes": os.path.getsize(path), "sha256": sha256(path),
            "min_cosine_vs_torch": float(cosine(want, got).min()),
        }
    # The text side, once.
    labels = {}
    with torch.no_grad():
        for key, prompts in LABELS.items():
            t = proc(text=prompts, return_tensors="pt", padding=True)
            e = model.get_text_features(**t)
            e = e / e.norm(dim=-1, keepdim=True)
            m = e.mean(0)
            labels[key] = (m / m.norm()).tolist()
    meta = {"name": name, "repo": repo, "mean": proc.image_processor.image_mean,
            "std": proc.image_processor.image_std, "size": 224,
            "logit_scale": float(model.logit_scale.exp()), "labels": labels}
    json.dump(meta, open(os.path.join(out, f"{name}.labels.json"), "w"))
    if check_dir:
        files = sorted(os.listdir(check_dir))[:20]
        sess = ort.InferenceSession(fp32)
        refs = {}
        for f in files:
            img = Image.open(os.path.join(check_dir, f)).convert("RGB")
            px = proc(images=img, return_tensors="np")["pixel_values"]
            refs[f] = sess.run(None, {"pixels": px})[0][0].tolist()
        json.dump(refs, open(os.path.join(out, f"{name}.check.json"), "w"))
    return report


def main():
    out = sys.argv[1]
    check_dir = sys.argv[sys.argv.index("--check") + 1] if "--check" in sys.argv else None
    os.makedirs(out, exist_ok=True)
    reports = {name: export(name, repo, out, check_dir) for name, repo in MODELS.items()}
    print(json.dumps(reports, indent=1))


if __name__ == "__main__":
    main()
