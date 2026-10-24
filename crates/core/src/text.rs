// SPDX-License-Identifier: GPL-3.0-only
//! Text measuring and drawing shared by the stamp (`stamp`) and the info frame (`render`),
//! plus ASCII folding for fonts and searches without accented letters.

use ab_glyph::{Font, FontArc, PxScale, ScaleFont, point};

use crate::image::RgbImage;

/// One line of text to measure or draw.
pub struct TextRun<'a> {
    pub font: &'a FontArc,
    pub text: &'a str,
    pub px: f32,
    /// Extra space between glyphs, in em.
    pub tracking: f32,
}

/// Advance width of `text` in pixels, including `tracking` (extra space per glyph, in em).
pub fn text_width(font: &FontArc, px: f32, text: &str, tracking: f32) -> f32 {
    let scaled = font.as_scaled(PxScale::from(px));
    let mut width = 0.0;
    let mut prev = None;
    for c in text.chars() {
        let id = scaled.glyph_id(c);
        if let Some(p) = prev {
            width += scaled.kern(p, id) + tracking * px;
        }
        width += scaled.h_advance(id);
        prev = Some(id);
    }
    width
}

/// Calls `paint(x, y, coverage)` for every pixel the glyphs of `run` cover, with the text's
/// top-left at (`left`, `top`).
#[allow(clippy::cast_possible_truncation)]
fn rasterize(run: &TextRun<'_>, left: f32, top: f32, mut paint: impl FnMut(i32, i32, f32)) {
    let scale = PxScale::from(run.px);
    let scaled = run.font.as_scaled(scale);
    let baseline = top + scaled.ascent();
    let mut caret = left;
    let mut prev = None;
    for c in run.text.chars() {
        let id = scaled.glyph_id(c);
        if let Some(p) = prev {
            caret += scaled.kern(p, id) + run.tracking * run.px;
        }
        let glyph = id.with_scale_and_position(scale, point(caret, baseline));
        caret += scaled.h_advance(id);
        prev = Some(id);
        let Some(outline) = run.font.outline_glyph(glyph) else {
            continue;
        };
        let b = outline.px_bounds();
        #[allow(clippy::cast_possible_wrap)]
        outline.draw(|gx, gy, coverage| {
            paint(
                b.min.x as i32 + gx as i32,
                b.min.y as i32 + gy as i32,
                coverage,
            );
        });
    }
}

/// Blends `color` into `image` at (`x`, `y`) with opacity `alpha` (0–1); outside is ignored.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]
pub fn blend(image: &mut RgbImage, x: i32, y: i32, color: [u8; 3], alpha: f32) {
    if x < 0 || y < 0 || x >= image.width as i32 || y >= image.height as i32 || alpha <= 0.0 {
        return;
    }
    let alpha = alpha.min(1.0);
    let i = (y as usize * image.width as usize + x as usize) * 3;
    for (channel, target) in image.data[i..i + 3].iter_mut().zip(color) {
        let blended = f32::from(*channel) * (1.0 - alpha) + f32::from(target) * alpha;
        *channel = blended.round() as u8;
    }
}

/// Draws `run` with its top-left at (`left`, `top`).
pub fn draw_text(
    image: &mut RgbImage,
    run: &TextRun<'_>,
    left: f32,
    top: f32,
    color: [u8; 3],
    opacity: f32,
) {
    rasterize(run, left, top, |x, y, coverage| {
        blend(image, x, y, color, coverage * opacity);
    });
}

/// The glyph coverage of `run` in a `width` × `height` mask, with the text's top-left at
/// (`left`, `top`) inside the mask. Used for effects that need the shape (glow, shadow).
#[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
pub fn coverage_mask(
    run: &TextRun<'_>,
    width: usize,
    height: usize,
    left: f32,
    top: f32,
) -> Vec<f32> {
    let mut mask = vec![0f32; width * height];
    rasterize(run, left, top, |x, y, coverage| {
        if x >= 0 && y >= 0 && (x as usize) < width && (y as usize) < height {
            let m = &mut mask[y as usize * width + x as usize];
            *m = (*m + coverage).min(1.0);
        }
    });
    mask
}

const FROM: &str = "àáạảãâầấậẩẫăằắặẳẵäåāąèéẹẻẽêềếệểễëēęěìíịỉĩîïīòóọỏõôồốộổỗơờớợởỡöøōőùúụủũưừứựửữüūůűỳýỵỷỹÿçćčđďğłńñňřśšşţťžźżß";
const TO: &str = "aaaaaaaaaaaaaaaaaaaaaeeeeeeeeeeeeeeeiiiiiiiiooooooooooooooooooooouuuuuuuuuuuuuuuyyyyyycccddglnnnrsssttzzzs";

/// The lower-case ASCII letter for a lower-case accented one (`ộ` → `o`, `đ` → `d`).
pub fn fold_char(c: char) -> Option<char> {
    FROM.chars()
        .position(|f| f == c)
        .and_then(|i| TO.chars().nth(i))
}

/// Upper-case ASCII without accents, keeping spaces and ASCII punctuation: "Hà Nội 2025"
/// → "HA NOI 2025". Other characters are dropped.
pub fn to_ascii_upper(text: &str) -> String {
    text.chars()
        .flat_map(char::to_lowercase)
        .filter_map(|c| {
            if c.is_ascii() && !c.is_ascii_control() {
                Some(c)
            } else {
                fold_char(c)
            }
        })
        .flat_map(char::to_uppercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fr_036_text_is_folded_to_plain_upper_case() {
        assert_eq!(to_ascii_upper("Hà Nội · Tết 2025!"), "HA NOI  TET 2025!");
        assert_eq!(to_ascii_upper("Budaörs"), "BUDAORS");
        assert_eq!(to_ascii_upper("Đà Lạt\n"), "DA LAT");
    }
}
