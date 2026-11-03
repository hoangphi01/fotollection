// SPDX-License-Identifier: GPL-3.0-only
//! The postcard half of a collage sheet (FR-073): a stamp box at the sheet's top right, and
//! under it a text box with writing lines, the photo's own text typed on them in a
//! typewriter face (Xanh Mono, OFL) or the user's font (FR-042). Drawn on the sheet's paper
//! after the photo, in millimetres turned into the sheet's pixels.

use std::sync::LazyLock;

use ab_glyph::{Font, FontArc, PxScale, ScaleFont};

use crate::image::RgbImage;
use crate::settings::BorderColor;
use crate::sheet::RectMm;
use crate::text::{TextRun, draw_text, text_width};

static XANH_MONO: &[u8] = include_bytes!("../assets/fonts/XanhMono-Regular.ttf");
static FONT: LazyLock<Option<FontArc>> =
    LazyLock::new(|| FontArc::try_from_vec(XANH_MONO.to_vec()).ok());

/// The stamp box, width × height: about the size of a German or Hungarian stamp, so a real
/// one covers it without reaching the address.
pub const STAMP_MM: (f32, f32) = (26.0, 32.0);
/// Space between the sheet's edge and the boxes, and between the stamp and the text box.
pub const EDGE_MM: f32 = 5.0;
const OUTLINE_MM: f32 = 0.3;
/// The stamp box's dashes and the gaps between them.
const DASH_MM: f32 = 1.2;
const LINE_MM: f32 = 0.25;
/// The word in the stamp box, its height in millimetres.
const STAMP_WORD: &str = "STAMP";
const STAMP_WORD_MM: f32 = 3.2;
/// Typed text: its em as a share of the line spacing, at most `MAX_TEXT_MM` (about 12 pt,
/// typewriter size, however far apart the lines are), and the gap above each line.
const TEXT_EM: f32 = 0.5;
const MAX_TEXT_MM: f32 = 4.2;
const BASELINE_GAP: f32 = 0.18;
/// Room left and right of the typed text inside the box.
const TEXT_PAD_MM: f32 = 2.0;

/// The bundled typewriter face.
pub fn font() -> Option<&'static FontArc> {
    FONT.as_ref()
}

/// Where the boxes and lines go, in millimetres on the sheet.
#[derive(Debug, Clone, PartialEq)]
pub struct CardLayout {
    pub stamp: RectMm,
    pub text: RectMm,
    /// The y of each writing line, top to bottom.
    pub lines: Vec<f32>,
}

/// Lays out the postcard half `card` of a `sheet_width` × `sheet_height` sheet: the stamp
/// box at the sheet's top-right corner, the text box filling the half below it, `lines`
/// writing lines spread evenly in the box with room above the first for its text.
pub fn layout(sheet_width: f32, card: RectMm, lines: u8) -> CardLayout {
    let stamp = RectMm {
        x: sheet_width - EDGE_MM - STAMP_MM.0,
        y: EDGE_MM,
        width: STAMP_MM.0,
        height: STAMP_MM.1,
    };
    let top = (stamp.y + stamp.height + EDGE_MM).max(card.y + EDGE_MM);
    let text = RectMm {
        x: card.x + EDGE_MM,
        y: top,
        width: (card.width - 2.0 * EDGE_MM).max(1.0),
        height: (card.y + card.height - EDGE_MM - top).max(1.0),
    };
    let n = f32::from(lines.max(1));
    let step = text.height / n;
    let lines = (1..=lines.max(1))
        .map(|k| text.y + step * f32::from(k) - step * 0.2)
        .collect();
    CardLayout { stamp, text, lines }
}

/// What a postcard half draws.
pub struct CardSpec<'a> {
    pub card: RectMm,
    pub sheet_width: f32,
    pub lines: u8,
    pub text: Option<&'a str>,
    /// The face for the typed text; `None` uses the bundled one.
    pub font: Option<&'a FontArc>,
    pub paper: BorderColor,
}

/// Ink for the boxes, the lines and the typed text on this paper.
fn inks(paper: BorderColor) -> ([u8; 3], [u8; 3], [u8; 3]) {
    match paper {
        BorderColor::White => ([140, 140, 140], [185, 185, 185], [35, 35, 35]),
        BorderColor::Black => ([150, 150, 150], [95, 95, 95], [235, 235, 235]),
    }
}

/// Draws the postcard half onto `canvas`, `density` pixels per millimetre.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
pub fn draw(canvas: &mut RgbImage, density: f32, spec: &CardSpec<'_>) {
    let layout = layout(spec.sheet_width, spec.card, spec.lines);
    let (outline, rule, ink) = inks(spec.paper);
    let px = |mm: f32| (mm * density).round().max(0.0) as u32;
    let thick = |mm: f32| (mm * density).round().max(1.0) as u32;
    // A box's four sides; `dash` in pixels breaks them into dashes and gaps of that length.
    let frame = |canvas: &mut RgbImage, r: RectMm, dash: Option<u32>| {
        let (x, y, w, h, t) = (
            px(r.x),
            px(r.y),
            px(r.width),
            px(r.height),
            thick(OUTLINE_MM),
        );
        let mut side = |x: u32, y: u32, len: u32, across: bool| {
            let step = dash.unwrap_or(len).max(1);
            let mut at = 0;
            while at < len {
                let run = step.min(len - at);
                if across {
                    crate::sheet::fill_rect(canvas, x + at, y, run, t, outline);
                } else {
                    crate::sheet::fill_rect(canvas, x, y + at, t, run, outline);
                }
                at += 2 * step;
            }
        };
        side(x, y, w, true);
        side(x, (y + h).saturating_sub(t), w, true);
        side(x, y, h, false);
        side((x + w).saturating_sub(t), y, h, false);
    };
    // The stamp box is dashed, as on a postcard; the text box is a plain line.
    frame(canvas, layout.stamp, Some(thick(DASH_MM)));
    frame(canvas, layout.text, None);
    let rule_inset = TEXT_PAD_MM / 2.0;
    for y in &layout.lines {
        crate::sheet::fill_rect(
            canvas,
            px(layout.text.x + rule_inset),
            px(*y),
            px(layout.text.width - 2.0 * rule_inset),
            thick(LINE_MM),
            rule,
        );
    }
    let Some(bundled) = font() else {
        return;
    };
    // The word in the stamp box, centred.
    let word_px = STAMP_WORD_MM * density / cap_height(bundled);
    let word_w = text_width(bundled, word_px, STAMP_WORD, 0.08);
    let scaled = bundled.as_scaled(PxScale::from(word_px));
    let s = layout.stamp;
    draw_text(
        canvas,
        &TextRun {
            font: bundled,
            text: STAMP_WORD,
            px: word_px,
            tracking: 0.08,
        },
        (s.x + s.width / 2.0) * density - word_w / 2.0,
        (s.y + s.height / 2.0) * density - f32::midpoint(scaled.ascent(), scaled.descent()),
        outline,
        1.0,
    );
    // The typed text, wrapped to the box and set on the lines; what does not fit is cut.
    let Some(text) = spec.text.filter(|t| !t.trim().is_empty()) else {
        return;
    };
    let face = spec.font.unwrap_or(bundled);
    let step = layout.text.height / f32::from(spec.lines.max(1));
    let text_px = (step * TEXT_EM).min(MAX_TEXT_MM) * density;
    let room = (layout.text.width - 2.0 * TEXT_PAD_MM) * density;
    let ascent = face.as_scaled(PxScale::from(text_px)).ascent();
    for (row, y) in wrap(face, text_px, text, room).iter().zip(&layout.lines) {
        let baseline = (y - step * BASELINE_GAP / 2.0) * density;
        draw_text(
            canvas,
            &TextRun {
                font: face,
                text: row,
                px: text_px,
                tracking: 0.0,
            },
            (layout.text.x + TEXT_PAD_MM) * density,
            baseline - ascent,
            ink,
            1.0,
        );
    }
}

/// The height of a capital (its "H") per pixel of em, so the stamp word is sized by its
/// visible letters.
fn cap_height(font: &FontArc) -> f32 {
    let scaled = font.as_scaled(PxScale::from(100.0));
    let bounds = font.glyph_bounds(&scaled.scaled_glyph('H'));
    (bounds.height() / 100.0).max(0.3)
}

/// `text` broken into rows no wider than `room` pixels: at its own line breaks, then between
/// words, and inside a word too long for a row.
pub fn wrap(font: &FontArc, px: f32, text: &str, room: f32) -> Vec<String> {
    let fits = |s: &str| text_width(font, px, s, 0.0) <= room;
    let mut rows = Vec::new();
    for paragraph in text.split('\n') {
        let mut row = String::new();
        for word in paragraph.split_whitespace() {
            let candidate = if row.is_empty() {
                word.to_owned()
            } else {
                format!("{row} {word}")
            };
            if fits(&candidate) {
                row = candidate;
                continue;
            }
            if !row.is_empty() {
                rows.push(std::mem::take(&mut row));
            }
            // A word longer than a row is split where it reaches the edge.
            for c in word.chars() {
                let mut next = row.clone();
                next.push(c);
                if !fits(&next) && !row.is_empty() {
                    rows.push(std::mem::take(&mut row));
                    next = c.to_string();
                }
                row = next;
            }
        }
        rows.push(row);
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    fn half(x: f32, y: f32, width: f32, height: f32) -> RectMm {
        RectMm {
            x,
            y,
            width,
            height,
        }
    }

    /// FR-073: on a landscape 10 × 15 sheet the stamp box sits 5 mm from the top-right
    /// corner, the text box fills the right half below it, and the lines lie inside the box,
    /// evenly spaced; on a portrait sheet the box spans the top half under the stamp.
    #[test]
    fn fr_073_boxes_and_lines_sit_in_the_postcard_half() {
        let landscape = layout(152.0, half(76.0, 0.0, 76.0, 102.0), 4);
        assert_eq!(landscape.stamp, half(121.0, 5.0, 26.0, 32.0));
        assert_eq!(landscape.text, half(81.0, 42.0, 66.0, 55.0));
        assert_eq!(landscape.lines.len(), 4);
        let gaps: Vec<f32> = landscape.lines.windows(2).map(|w| w[1] - w[0]).collect();
        assert!(gaps.iter().all(|g| (g - gaps[0]).abs() < 1e-3));
        assert!(
            landscape
                .lines
                .iter()
                .all(|y| *y > landscape.text.y && *y < landscape.text.y + landscape.text.height)
        );

        let portrait = layout(102.0, half(0.0, 0.0, 102.0, 76.0), 3);
        assert_eq!(portrait.stamp, half(71.0, 5.0, 26.0, 32.0));
        assert_eq!(portrait.text, half(5.0, 42.0, 92.0, 29.0));
        assert_eq!(portrait.lines.len(), 3);
    }

    /// Text wraps between words, keeps its own line breaks, and splits a word wider than a
    /// row; Vietnamese, Hungarian and German letters are in the face.
    #[test]
    fn fr_073_text_wraps_and_the_face_has_the_letters() {
        let Some(face) = font() else {
            panic!("the bundled face loads");
        };
        let room = text_width(face, 20.0, "Liebe Grüße", 0.0) + 1.0;
        assert_eq!(
            wrap(face, 20.0, "Liebe Grüße aus\nBudapest", room),
            ["Liebe Grüße", "aus", "Budapest"]
        );
        let long = wrap(face, 20.0, "Donaudampfschifffahrt", room / 2.0);
        assert!(long.len() > 1 && long.concat() == "Donaudampfschifffahrt");
        for c in "Thương nhớ Hà Nội őű čř ß €"
            .chars()
            .filter(|c| !c.is_whitespace())
        {
            assert_ne!(face.glyph_id(c).0, 0, "{c}");
        }
    }
}
