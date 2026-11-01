// SPDX-License-Identifier: GPL-3.0-only
//! Every JPEG layout a camera, a phone or an editor writes must decode to the photo, not to
//! green stripes (FR-001, LESSONS "A decoder can be silently wrong"). The fixtures are one
//! picture saved in each layout by libjpeg-turbo's `cjpeg` (tests/fixtures/jpeg/README.md);
//! each decode is compared with the picture itself, so a decoder upgrade that breaks a
//! layout fails here.

use fotollection_core::codec::{CodecError, PureRustCodec, decode_image};
use std::path::Path;

/// Mean absolute difference per channel allowed against the source, out of 255. Quality 90
/// with halved colour stays near 2 to 4; a broken decode is far above 20.
const MAX_MEAN_ERROR: f64 = 8.0;

fn fixture(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/jpeg")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

#[test]
fn fr_001_every_jpeg_layout_decodes_like_the_photo() -> Result<(), CodecError> {
    let codec = PureRustCodec;
    let source = decode_image(&codec, &fixture("source.png"))?;
    let names = [
        "baseline-444.jpg",
        "baseline-422.jpg",
        "baseline-420.jpg",
        "baseline-440.jpg",
        "baseline-411.jpg",
        "progressive-444.jpg",
        "progressive-422.jpg",
        "progressive-420.jpg",
        "restart-420.jpg",
        "optimized-420.jpg",
        // One component per scan: zune-jpeg 0.5.15 decodes these as green stripes.
        "noninterleaved-420.jpg",
        "noninterleaved-444.jpg",
        "grey.jpg",
    ];
    let mut failures = Vec::new();
    for name in names {
        let decoded = match decode_image(&codec, &fixture(name)) {
            Ok(d) => d.image,
            Err(e) => {
                failures.push(format!("{name}: {e}"));
                continue;
            }
        };
        assert_eq!(
            (decoded.width, decoded.height),
            (source.image.width, source.image.height),
            "{name}: size"
        );
        let grey = name.starts_with("grey");
        let error = mean_error(&source.image.data, &decoded.data, grey);
        if error > MAX_MEAN_ERROR {
            failures.push(format!("{name}: mean error {error:.1}"));
        }
    }
    assert!(failures.is_empty(), "wrong decodes: {failures:#?}");
    Ok(())
}

/// Mean absolute difference per channel; a grey decode is compared with the source's luma.
#[allow(clippy::cast_precision_loss)]
fn mean_error(source: &[u8], decoded: &[u8], grey: bool) -> f64 {
    let mut sum = 0u64;
    for (s, d) in source
        .as_chunks::<3>()
        .0
        .iter()
        .zip(decoded.as_chunks::<3>().0)
    {
        if grey {
            let luma =
                (299 * u32::from(s[0]) + 587 * u32::from(s[1]) + 114 * u32::from(s[2])) / 1000;
            sum += u64::from(luma.abs_diff(u32::from(d[0])));
        } else {
            sum += s
                .iter()
                .zip(d)
                .map(|(a, b)| u64::from(a.abs_diff(*b)))
                .sum::<u64>()
                / 3;
        }
    }
    sum as f64 / (source.len() / 3) as f64
}
