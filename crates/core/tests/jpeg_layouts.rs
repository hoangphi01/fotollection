// SPDX-License-Identifier: GPL-3.0-only
//! Every JPEG layout a camera, a phone or an editor writes must decode to the photo, not to
//! green stripes (FR-001, LESSONS "A decoder can be silently wrong"). The fixtures are one
//! picture saved in each layout by libjpeg-turbo's `cjpeg` or Pillow (tests/fixtures/jpeg/
//! README.md); each decode is compared with the picture itself, pixel by pixel, so a decoder
//! upgrade that breaks a layout, or a part of one, fails here.

use fotollection_core::codec::{CodecError, PureRustCodec, decode_image};
use fotollection_core::jpeg_layout;
use std::path::Path;

/// Errors allowed against the source, per channel out of 255. Quality 90 with halved colour
/// stays near 2–4 on average; a broken decode is far above 20. The 99th percentile catches a
/// wrong band or block that a mean hides: a good 4:2:0 decode stays at 45, where halved
/// colour meets the sharp colour edges of the picture. The single worst pixel is no test:
/// at those edges a correct 4:2:0 decode is off by up to 217.
const MAX_MEAN_ERROR: f64 = 8.0;
const MAX_P99_ERROR: u8 = 60;

/// How a fixture must go: decoded by zune-jpeg (a proven layout), by the reference decoder,
/// or refused with an error rather than a wrong picture.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Route {
    Zune,
    Reference,
    Refused,
}

fn fixture(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/jpeg")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

#[test]
fn fr_001_every_jpeg_layout_decodes_like_the_photo() -> Result<(), CodecError> {
    use Route::{Reference, Refused, Zune};
    let codec = PureRustCodec;
    let source = decode_image(&codec, &fixture("source.png"))?;
    let cases = [
        ("baseline-444.jpg", Zune),
        ("baseline-422.jpg", Zune),
        ("baseline-420.jpg", Zune),
        ("baseline-440.jpg", Zune),
        ("baseline-411.jpg", Zune),
        ("progressive-444.jpg", Zune),
        ("progressive-422.jpg", Zune),
        ("progressive-420.jpg", Zune),
        ("restart-420.jpg", Zune),
        ("restart-444.jpg", Zune),
        ("restart-422.jpg", Zune),
        ("optimized-420.jpg", Zune),
        // One component per scan: zune-jpeg 0.5.15 decodes these as green stripes.
        ("noninterleaved-420.jpg", Reference),
        ("noninterleaved-444.jpg", Reference),
        // Progressive, its DC scan one component at a time.
        ("progressive-noninterleaved-dc-420.jpg", Zune),
        ("grey.jpg", Zune),
        // Lossless (SOF3): 16-bit grey and 8-bit colour.
        ("lossless-16-grey.jpg", Reference),
        ("lossless-rgb.jpg", Reference),
        // Adobe CMYK, as Photoshop writes it.
        ("cmyk.jpg", Zune),
        // Adobe YCCK (APP14 transform 2), as Photoshop writes CMYK at most qualities.
        ("ycck.jpg", Zune),
        // 8-bit extended sequential (SOF1): a quantization value above 255. zune decodes this
        // one right, but SOF1 also allows four Huffman tables, which no fixture covers yet.
        ("extended-sof1-420.jpg", Reference),
        // Neither decoder reads arithmetic coding; a cut-off file must not paint a half photo.
        ("arithmetic-420.jpg", Refused),
        ("truncated-420.jpg", Refused),
    ];
    let mut failures = Vec::new();
    for (name, route) in cases {
        let bytes = fixture(name);
        let proven = jpeg_layout::read(&bytes).is_some_and(|l| l.zune_proven());
        if route != Refused && proven != (route == Zune) {
            failures.push(format!(
                "{name}: routed to {}, expected {route:?}",
                if proven {
                    "zune"
                } else {
                    "the reference decoder"
                }
            ));
        }
        let decoded = match (decode_image(&codec, &bytes), route) {
            (Err(_), Refused) => continue,
            (Ok(_), Refused) => {
                failures.push(format!("{name}: decoded, but must be refused"));
                continue;
            }
            (Err(e), _) => {
                failures.push(format!("{name}: {e}"));
                continue;
            }
            (Ok(d), _) => d.image,
        };
        assert_eq!(
            (decoded.width, decoded.height),
            (source.image.width, source.image.height),
            "{name}: size"
        );
        let grey = name.contains("grey");
        let stats = errors(&source.image.data, &decoded.data, grey);
        if stats.mean > MAX_MEAN_ERROR || stats.p99 > MAX_P99_ERROR {
            failures.push(format!("{name}: mean {:.1}, p99 {}", stats.mean, stats.p99));
        }
    }
    assert!(failures.is_empty(), "wrong decodes: {failures:#?}");
    Ok(())
}

struct Errors {
    mean: f64,
    p99: u8,
}

/// Per-channel absolute differences; a grey decode is compared with the source's luma.
#[allow(clippy::cast_precision_loss)]
fn errors(source: &[u8], decoded: &[u8], grey: bool) -> Errors {
    let mut diffs: Vec<u8> = Vec::with_capacity(source.len());
    for (s, d) in source
        .as_chunks::<3>()
        .0
        .iter()
        .zip(decoded.as_chunks::<3>().0)
    {
        if grey {
            let luma =
                (299 * u32::from(s[0]) + 587 * u32::from(s[1]) + 114 * u32::from(s[2])) / 1000;
            diffs.push(u8::try_from(luma.abs_diff(u32::from(d[0]))).unwrap_or(u8::MAX));
        } else {
            diffs.extend(s.iter().zip(d).map(|(a, b)| a.abs_diff(*b)));
        }
    }
    let mean = diffs.iter().map(|&d| f64::from(d)).sum::<f64>() / diffs.len() as f64;
    diffs.sort_unstable();
    let p99 = diffs[(diffs.len() * 99 / 100).min(diffs.len() - 1)];
    Errors { mean, p99 }
}
