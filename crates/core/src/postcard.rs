// SPDX-License-Identifier: GPL-3.0-only
//! The postcard half of a collage sheet (FR-073): a dashed stamp box at the sheet's top
//! right, and under it writing lines, the photo's own text typed on them in a
//! typewriter face (Xanh Mono, OFL) or the user's font (FR-042). Drawn on the sheet's paper
//! after the photo, in millimetres turned into the sheet's pixels.

use std::sync::LazyLock;

use ab_glyph::{Font, FontArc, PxScale, ScaleFont};

use crate::image::RgbImage;
use crate::settings::{BorderColor, PostcardInk};
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
/// The credit line under the text box: its em, and the gap above it.
const CREDIT_MM: f32 = 2.2;
const CREDIT_GAP_MM: f32 = 0.9;

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
    /// A small line under the text box on every card ("Photo © …").
    pub credit: Option<&'a str>,
    /// The face for the typed text; `None` uses the bundled one.
    pub font: Option<&'a FontArc>,
    pub paper: BorderColor,
    /// The typed text's ink.
    pub ink: PostcardInk,
}

/// Grey for the stamp box and the credit, and for the writing lines, on this paper.
fn inks(paper: BorderColor) -> ([u8; 3], [u8; 3]) {
    match paper {
        BorderColor::White => ([140, 140, 140], [185, 185, 185]),
        BorderColor::Black => ([150, 150, 150], [95, 95, 95]),
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
    let (outline, rule) = inks(spec.paper);
    let ink = spec.ink.rgb(spec.paper);
    let px = |mm: f32| (mm * density).round().max(0.0) as u32;
    let thick = |mm: f32| (mm * density).round().max(1.0) as u32;
    // The stamp box, dashed as on a postcard: dashes and gaps of `DASH_MM` along each side.
    // The text needs only its lines.
    let s = layout.stamp;
    let (x, y, w, h, t) = (
        px(s.x),
        px(s.y),
        px(s.width),
        px(s.height),
        thick(OUTLINE_MM),
    );
    let dash = thick(DASH_MM);
    let mut side = |x: u32, y: u32, len: u32, across: bool| {
        let mut at = 0;
        while at < len {
            let run = dash.min(len - at);
            if across {
                crate::sheet::fill_rect(canvas, x + at, y, run, t, outline);
            } else {
                crate::sheet::fill_rect(canvas, x, y + at, t, run, outline);
            }
            at += 2 * dash;
        }
    };
    side(x, y, w, true);
    side(x, (y + h).saturating_sub(t), w, true);
    side(x, y, h, false);
    side((x + w).saturating_sub(t), y, h, false);
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
    // The chosen face, with Inter for the letters it lacks (Xanh Mono has no Greek or
    // Cyrillic): an address copied from anywhere in Europe prints as typed.
    let faces = Faces {
        main: spec.font.unwrap_or(bundled),
        fallback: crate::render::regular(),
    };
    if let Some(credit) = spec.credit {
        draw_credit(canvas, density, faces, &layout, credit, outline);
    }
    // The typed text, wrapped to the box and set on the lines; what does not fit is cut.
    let Some(text) = spec.text.filter(|t| !t.trim().is_empty()) else {
        return;
    };
    let step = layout.text.height / f32::from(spec.lines.max(1));
    let text_px = (step * TEXT_EM).min(MAX_TEXT_MM) * density;
    let room = (layout.text.width - 2.0 * TEXT_PAD_MM) * density;
    for (row, y) in wrap(faces, text_px, text, room).iter().zip(&layout.lines) {
        let baseline = (y - step * BASELINE_GAP / 2.0) * density;
        faces.draw(
            canvas,
            row,
            (layout.text.x + TEXT_PAD_MM) * density,
            baseline,
            text_px,
            ink,
        );
    }
}

/// A face and the one that sets the letters it lacks; each run of text goes to the first
/// face with its glyphs.
#[derive(Clone, Copy)]
pub struct Faces<'a> {
    pub main: &'a FontArc,
    pub fallback: Option<&'a FontArc>,
}

impl<'a> Faces<'a> {
    fn pick(&self, c: char) -> &'a FontArc {
        if c.is_whitespace() || self.main.glyph_id(c).0 != 0 {
            return self.main;
        }
        self.fallback
            .filter(|f| f.glyph_id(c).0 != 0)
            .unwrap_or(self.main)
    }

    /// `text` cut where the face changes.
    fn runs<'s>(&self, text: &'s str) -> Vec<(&'a FontArc, &'s str)> {
        let mut runs: Vec<(&'a FontArc, &'s str)> = Vec::new();
        let mut start = 0;
        let mut current: Option<&'a FontArc> = None;
        for (i, c) in text.char_indices() {
            let face = self.pick(c);
            if let Some(prev) = current
                && !std::ptr::eq(prev, face)
            {
                runs.push((prev, &text[start..i]));
                start = i;
            }
            current = Some(face);
        }
        if let Some(face) = current {
            runs.push((face, &text[start..]));
        }
        runs
    }

    /// The width of `text` in pixels.
    pub fn width(&self, px: f32, text: &str) -> f32 {
        self.runs(text)
            .iter()
            .map(|(f, s)| text_width(f, px, s, 0.0))
            .sum()
    }

    /// Draws `text` from `left` with its baseline at `baseline`, each face on that baseline.
    fn draw(
        &self,
        canvas: &mut RgbImage,
        text: &str,
        left: f32,
        baseline: f32,
        px: f32,
        ink: [u8; 3],
    ) {
        let mut x = left;
        for (face, run) in self.runs(text) {
            let ascent = face.as_scaled(PxScale::from(px)).ascent();
            draw_text(
                canvas,
                &TextRun {
                    font: face,
                    text: run,
                    px,
                    tracking: 0.0,
                },
                x,
                baseline - ascent,
                ink,
                1.0,
            );
            x += text_width(face, px, run, 0.0);
        }
    }
}

/// The credit, small and light, in the margin under the text box; cut to the box's width.
fn draw_credit(
    canvas: &mut RgbImage,
    density: f32,
    faces: Faces<'_>,
    layout: &CardLayout,
    credit: &str,
    ink: [u8; 3],
) {
    let credit = credit.trim();
    if credit.is_empty() {
        return;
    }
    let px = CREDIT_MM * density;
    if let Some(row) = wrap(faces, px, credit, layout.text.width * density).first() {
        let top = (layout.text.y + layout.text.height + CREDIT_GAP_MM) * density;
        let ascent = faces.main.as_scaled(PxScale::from(px)).ascent();
        faces.draw(canvas, row, layout.text.x * density, top + ascent, px, ink);
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
pub fn wrap(faces: Faces<'_>, px: f32, text: &str, room: f32) -> Vec<String> {
    let fits = |s: &str| faces.width(px, s) <= room;
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

    /// FR-073: the credit line prints small in the margin under the text box, and nothing
    /// prints there without one.
    #[test]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn fr_073_the_credit_sits_under_the_text_box() {
        let density = 10.0; // px per mm
        let card = half(76.0, 0.0, 76.0, 102.0);
        let ink_under_box = |credit: Option<&str>| {
            let mut canvas = crate::render::filled(1520, 1020, [255, 255, 255]);
            draw(
                &mut canvas,
                density,
                &CardSpec {
                    card,
                    sheet_width: 152.0,
                    lines: 4,
                    text: None,
                    credit,
                    font: None,
                    paper: BorderColor::White,
                    ink: PostcardInk::Auto,
                },
            );
            let box_bottom = layout(152.0, card, 4).text;
            let y0 = ((box_bottom.y + box_bottom.height) * density) as u32 + 2;
            (y0..1020)
                .flat_map(|y| (760..1520).map(move |x| (x, y)))
                .filter(|&(x, y)| canvas.data[((y * 1520 + x) * 3) as usize] < 200)
                .count()
        };
        assert_eq!(ink_under_box(None), 0);
        assert_eq!(ink_under_box(Some("   ")), 0);
        assert!(ink_under_box(Some("Photo © Anna Schmidt")) > 50);
    }

    /// FR-073: the typed text takes the chosen ink; the lines stay grey.
    #[test]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn fr_073_the_text_takes_its_ink() {
        let mut canvas = crate::render::filled(1520, 1020, [255, 255, 255]);
        draw(
            &mut canvas,
            10.0,
            &CardSpec {
                card: half(76.0, 0.0, 76.0, 102.0),
                sheet_width: 152.0,
                lines: 4,
                text: Some("Liebe Grüße"),
                credit: None,
                font: None,
                paper: BorderColor::White,
                ink: PostcardInk::Blue,
            },
        );
        let blue = canvas
            .data
            .as_chunks::<3>()
            .0
            .iter()
            .filter(|[r, g, b]| *b > 120 && *r < 60 && *g < 90)
            .count();
        assert!(blue > 100, "{blue} blue pixels");
        assert_eq!(PostcardInk::Auto.rgb(BorderColor::Black), [235, 235, 235]);
    }

    /// FR-073: an address in any European script prints: the Latin letters of French,
    /// Czech, Hungarian or Polish are in Xanh Mono, and Greek or Cyrillic come from Inter.
    #[test]
    fn fr_073_european_addresses_find_their_letters() {
        let (Some(mono), Some(inter)) = (font(), crate::render::regular()) else {
            panic!("the bundled faces load");
        };
        for c in "àçéœÿ čřůž őű ąłż șț åøæ ñ ğış āėų"
            .chars()
            .filter(|c| !c.is_whitespace())
        {
            assert_ne!(mono.glyph_id(c).0, 0, "{c} in Xanh Mono");
        }
        let faces = Faces {
            main: mono,
            fallback: Some(inter),
        };
        let runs = faces.runs("Ул. Витоша 1, Athína Αθήνα");
        assert!(runs.len() >= 3, "{} runs", runs.len());
        for (face, run) in &runs {
            for c in run.chars().filter(|c| !c.is_whitespace()) {
                assert_ne!(face.glyph_id(c).0, 0, "{c} has no glyph in its run");
            }
        }
        assert!(faces.width(20.0, "Αθήνα") > 0.0);
    }

    /// Text wraps between words, keeps its own line breaks, and splits a word wider than a
    /// row; Vietnamese, Hungarian and German letters are in the face.
    #[test]
    fn fr_073_text_wraps_and_the_face_has_the_letters() {
        let Some(face) = font() else {
            panic!("the bundled face loads");
        };
        let only = Faces {
            main: face,
            fallback: None,
        };
        let room = text_width(face, 20.0, "Liebe Grüße", 0.0) + 1.0;
        assert_eq!(
            wrap(only, 20.0, "Liebe Grüße aus\nBudapest", room),
            ["Liebe Grüße", "aus", "Budapest"]
        );
        let long = wrap(only, 20.0, "Donaudampfschifffahrt", room / 2.0);
        assert!(long.len() > 1 && long.concat() == "Donaudampfschifffahrt");
        for c in "Thương nhớ Hà Nội őű čř ß €"
            .chars()
            .filter(|c| !c.is_whitespace())
        {
            assert_ne!(face.glyph_id(c).0, 0, "{c}");
        }
    }
}
