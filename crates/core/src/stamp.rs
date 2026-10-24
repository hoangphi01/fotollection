// SPDX-License-Identifier: GPL-3.0-only
//! The text printed inside the photo (FR-004, FR-005, FR-033, FR-036): the date, with the
//! place on the same line, and an optional caption line below it. Always set in DSEG14
//! Classic Italic (OFL), a 14-segment LCD face like a 1990s film camera's date imprint, with
//! the soft glow the imprint's LEDs left on film. The info frame has its own fonts.

use std::sync::LazyLock;

use ab_glyph::{Font, FontArc, PxScale, ScaleFont};

use crate::image::RgbImage;
use crate::plan::StampSpec;
use crate::settings::Corner;
use crate::text::{TextRun, blend, coverage_mask, text_width, to_ascii_upper};

static DSEG14_ITALIC: &[u8] = include_bytes!("../assets/fonts/DSEG14Classic-Italic.ttf");
static FONT: LazyLock<Option<FontArc>> =
    LazyLock::new(|| FontArc::try_from_vec(DSEG14_ITALIC.to_vec()).ok());

/// The caption is set a little smaller than the date line.
const CAPTION_SCALE: f32 = 0.8;
/// Space between the date line and the caption, in em of the date line.
const LINE_GAP_EM: f32 = 0.35;
/// Glow radius and strength, and the faint shadow that keeps the imprint readable on
/// bright skies.
const GLOW_RADIUS_EM: f32 = 0.09;
const GLOW_OPACITY: f32 = 0.6;
const SHADOW_OFFSET_EM: f32 = 0.04;
const SHADOW_OPACITY: f32 = 0.3;

/// The stamp font.
pub fn font() -> Option<&'static FontArc> {
    FONT.as_ref()
}

/// Text as the LCD face can show it: upper-case ASCII, with anything it has no glyph for
/// shown as a space.
pub fn displayable(font: &FontArc, text: &str) -> String {
    to_ascii_upper(text)
        .chars()
        .map(|c| if font.glyph_id(c).0 == 0 { ' ' } else { c })
        .collect::<String>()
        .trim()
        .to_owned()
}

/// One laid-out line, in output pixels (top-left corner).
#[derive(Debug, Clone, PartialEq)]
pub struct StampLine {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub px: f32,
    pub width: f32,
    pub height: f32,
}

fn line_height(font: &FontArc, px: f32) -> f32 {
    let scaled = font.as_scaled(PxScale::from(px));
    scaled.ascent() - scaled.descent()
}

/// Where the date line and the caption go on a `width` × `height` photo.
pub fn layout(width: u32, height: u32, spec: &StampSpec) -> Vec<StampLine> {
    let Some(font) = font() else {
        return Vec::new();
    };
    #[allow(clippy::cast_precision_loss)]
    let (w, h, m) = (width as f32, height as f32, spec.margin_px);
    let available = (w - 2.0 * m).max(1.0);

    let mut lines = Vec::new();
    let mut add = |text: String, px: f32| {
        if text.is_empty() {
            return;
        }
        // A caption longer than the photo is set smaller until it fits.
        let natural = text_width(font, px, &text, 0.0);
        let px = if natural > available {
            px * available / natural
        } else {
            px
        };
        let width = text_width(font, px, &text, 0.0);
        lines.push(StampLine {
            height: line_height(font, px),
            text,
            x: 0.0,
            y: 0.0,
            px,
            width,
        });
    };
    add(displayable(font, &spec.text), spec.font_px);
    if let Some(caption) = &spec.caption {
        add(displayable(font, caption), spec.font_px * CAPTION_SCALE);
    }

    let gap = LINE_GAP_EM * spec.font_px;
    // At the bottom, the caption's row is always kept free under the date, as on a film
    // camera's imprint: the date sits a little higher and never moves when a caption is
    // added or removed.
    let caption_row = line_height(font, spec.font_px * CAPTION_SCALE) + gap;
    let mut y = match spec.corner {
        Corner::BottomRight | Corner::BottomLeft => {
            h - m - caption_row - lines.first().map_or(0.0, |l| l.height)
        }
        Corner::TopRight | Corner::TopLeft => m,
    };
    let bottom = matches!(spec.corner, Corner::BottomRight | Corner::BottomLeft);
    for (i, line) in lines.iter_mut().enumerate() {
        line.x = match spec.corner {
            Corner::BottomRight | Corner::TopRight => w - m - line.width,
            Corner::BottomLeft | Corner::TopLeft => m,
        };
        // At the bottom, a caption set smaller to fit still sits on the safe-area edge.
        line.y = if bottom && i > 0 {
            h - m - line.height
        } else {
            y
        };
        y += line.height + gap;
    }
    lines
}

/// Three box blurs in a row: close to a Gaussian, in linear time.
fn blur(mask: &[f32], width: usize, height: usize, radius: usize) -> Vec<f32> {
    if radius == 0 {
        return mask.to_vec();
    }
    let mut a = mask.to_vec();
    let mut b = vec![0f32; a.len()];
    #[allow(clippy::cast_precision_loss)]
    let norm = 1.0 / (2 * radius + 1) as f32;
    for _ in 0..3 {
        // Horizontal, then vertical, with a running sum.
        for y in 0..height {
            let row = &a[y * width..(y + 1) * width];
            let mut sum: f32 = row[..radius.min(width)].iter().sum();
            for x in 0..width {
                if x + radius < width {
                    sum += row[x + radius];
                }
                b[y * width + x] = sum * norm;
                if x >= radius {
                    sum -= row[x - radius];
                }
            }
        }
        for x in 0..width {
            let mut sum: f32 = (0..radius.min(height)).map(|y| b[y * width + x]).sum();
            for y in 0..height {
                if y + radius < height {
                    sum += b[(y + radius) * width + x];
                }
                a[y * width + x] = sum * norm;
                if y >= radius {
                    sum -= b[(y - radius) * width + x];
                }
            }
        }
    }
    a
}

/// Draws the stamp: a faint shadow, the glow, then the sharp segments.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap
)]
pub fn draw(image: &mut RgbImage, spec: &StampSpec) {
    let Some(font) = font() else {
        return;
    };
    for line in layout(image.width, image.height, spec) {
        let pad = (line.px * 0.4).ceil();
        let (mw, mh) = (
            (line.width + 2.0 * pad).ceil() as usize,
            (line.height + 2.0 * pad).ceil() as usize,
        );
        let run = TextRun {
            font,
            text: &line.text,
            px: line.px,
            tracking: 0.0,
        };
        let mask = coverage_mask(&run, mw, mh, pad, pad);
        let glow = blur(
            &mask,
            mw,
            mh,
            (GLOW_RADIUS_EM * line.px).round().max(1.0) as usize,
        );
        let (ox, oy) = ((line.x - pad).round() as i32, (line.y - pad).round() as i32);
        let shadow = (SHADOW_OFFSET_EM * line.px).round().max(1.0) as i32;
        for y in 0..mh {
            for x in 0..mw {
                let i = y * mw + x;
                let (px, py) = (ox + x as i32, oy + y as i32);
                blend(
                    image,
                    px + shadow,
                    py + shadow,
                    [0, 0, 0],
                    glow[i] * SHADOW_OPACITY,
                );
            }
        }
        for y in 0..mh {
            for x in 0..mw {
                let i = y * mw + x;
                let (px, py) = (ox + x as i32, oy + y as i32);
                blend(image, px, py, spec.color, glow[i] * GLOW_OPACITY);
                blend(image, px, py, spec.color, mask[i]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(text: &str, caption: Option<&str>, corner: Corner) -> StampSpec {
        StampSpec {
            text: text.to_owned(),
            caption: caption.map(str::to_owned),
            color: [255, 140, 40],
            corner,
            font_px: 40.0,
            margin_px: 30.0,
        }
    }

    #[test]
    fn fr_036_caption_sits_under_the_date_and_fits_the_width() {
        let s = spec(
            "'25 09 21  Budaörs",
            Some("a very long caption that runs across the whole photo and beyond it"),
            Corner::BottomRight,
        );
        let lines = layout(1200, 800, &s);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].text, "'25 09 21  BUDAORS");
        assert!(lines[1].y > lines[0].y + lines[0].height, "{lines:?}");
        assert!(lines[1].width <= 1200.0 - 60.0 + 0.5, "{lines:?}");
        assert!((lines[1].y + lines[1].height - (800.0 - 30.0)).abs() < 1.0);
        // Without a caption the date stays where it was: its row below is kept free.
        let alone = layout(
            1200,
            800,
            &spec("'25 09 21  Budaörs", None, Corner::BottomRight),
        );
        assert!(
            (alone[0].y - lines[0].y).abs() < 0.01,
            "{alone:?} {lines:?}"
        );
        // Right corners align right.
        assert!((lines[0].x + lines[0].width - 1170.0).abs() < 0.5);
    }

    #[test]
    fn fr_033_top_corners_stack_downwards_and_draw_with_a_glow() {
        let s = spec("12.10.2026", Some("hello"), Corner::TopLeft);
        let lines = layout(800, 600, &s);
        assert!((lines[0].y - 30.0).abs() < 0.01 && lines[1].y > lines[0].y);
        let mut image = RgbImage::new(800, 600);
        draw(&mut image, &s);
        // Lit segments, and a glow beyond them that is dimmer than the segments.
        let lit = image
            .data
            .as_chunks::<3>()
            .0
            .iter()
            .filter(|p| p[0] > 200)
            .count();
        let glow = image
            .data
            .as_chunks::<3>()
            .0
            .iter()
            .filter(|p| p[0] > 10 && p[0] < 120)
            .count();
        assert!(lit > 100 && glow > lit / 4, "lit {lit}, glow {glow}");
    }

    #[test]
    fn missing_glyphs_become_spaces() {
        let Some(f) = font() else { return };
        assert_eq!(displayable(f, "Tết · 2025"), "TET  2025");
    }
}
