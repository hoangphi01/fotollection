// SPDX-License-Identifier: GPL-3.0-only
//! The postcard half of a collage sheet (FR-073): a dashed stamp box at the sheet's top
//! right with room for more stamps beside or under it, and the writing lines together at
//! the foot, the message's then the address's. Each card's own text is typed on them in a
//! typewriter face (Xanh Mono, OFL) or the user's font (FR-042), bold, italic or underlined
//! per part. Drawn on the sheet's paper after the photo, in millimetres turned into pixels.

use std::sync::LazyLock;

use ab_glyph::{Font, FontArc, PxScale, ScaleFont};

use crate::image::RgbImage;
use crate::library::PostcardText;
use crate::settings::{BorderColor, PostcardSettings, TextStyle};
use crate::sheet::RectMm;
use crate::text::{TextRun, draw_text, draw_text_bold, draw_text_slanted, text_width};

static XANH_MONO: &[u8] = include_bytes!("../assets/fonts/XanhMono-Regular.ttf");
static XANH_MONO_ITALIC: &[u8] = include_bytes!("../assets/fonts/XanhMono-Italic.ttf");
static FONT: LazyLock<Option<FontArc>> =
    LazyLock::new(|| FontArc::try_from_vec(XANH_MONO.to_vec()).ok());
static ITALIC: LazyLock<Option<FontArc>> =
    LazyLock::new(|| FontArc::try_from_vec(XANH_MONO_ITALIC.to_vec()).ok());

/// The stamp box, width × height: smaller than most stamps, so the stamp stuck on it hides
/// its dashed line (owner, 2026-11-04; 26 × 32 mm before), and far from the address.
pub const STAMP_MM: (f32, f32) = (20.0, 25.0);
/// Space between the sheet's edge and what is drawn, and between the stamp and the lines.
pub const EDGE_MM: f32 = 5.0;
const STAMP_GAP_MM: f32 = 4.0;
/// The address lines, under the message's: name, street, town; a long address gets four
/// in the room of those three, closer together and in smaller type (owner, 2026-11-04).
pub const ADDRESS_LINES: u8 = 3;
pub const MAX_ADDRESS_LINES: u8 = 4;
/// Lines no further apart than a ruled notebook's, and the gap between the message's and
/// the address's, in lines.
const MAX_STEP_MM: f32 = 8.0;
const PART_GAP: f32 = 0.5;
/// The room between a portrait card's two columns, the rule in its middle.
const COLUMN_GAP_MM: f32 = 6.0;
const OUTLINE_MM: f32 = 0.3;
/// The stamp box's dashes and the gaps between them.
const DASH_MM: f32 = 1.2;
const LINE_MM: f32 = 0.25;
/// The word in the stamp box, its height in millimetres.
const STAMP_WORD: &str = "STAMP";
const STAMP_WORD_MM: f32 = 3.2;
/// Typed text: at most this share of the line spacing, whatever size is chosen, so a row
/// never reaches the line above; and how far above its line a row sits, in line spacings.
const TEXT_EM: f32 = 0.75;
const BASELINE_GAP: f32 = 0.09;
/// Room left of the typed text on its line.
const TEXT_PAD_MM: f32 = 1.0;
/// The credit line at the foot of the half: its em, and the room kept for it.
const CREDIT_MM: f32 = 2.2;
const CREDIT_ROOM_MM: f32 = 3.5;
/// Bold, underline and oblique, in em: how much bold thickens the strokes, the underline's depth
/// and thickness, and the slant of a face with no italic.
const BOLD_EM: f32 = 0.035;
const UNDERLINE_EM: f32 = 0.12;
const UNDERLINE_THICK_EM: f32 = 0.055;
const SLANT: f32 = 0.2;

/// The bundled typewriter face.
pub fn font() -> Option<&'static FontArc> {
    FONT.as_ref()
}

/// One writing line, in millimetres on the sheet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    pub y: f32,
    pub x0: f32,
    pub x1: f32,
}

/// Where the stamp box and the lines go.
#[derive(Debug, Clone, PartialEq)]
pub struct CardLayout {
    pub stamp: RectMm,
    pub message: Vec<Line>,
    pub address: Vec<Line>,
    /// The distance between two lines, and between two of the address's (closer when it has
    /// four).
    pub step: f32,
    pub address_step: f32,
    /// A portrait card's rule between its two columns: x, top and bottom.
    pub divider: Option<(f32, f32, f32)>,
    /// The credit's left edge and its top.
    pub credit: (f32, f32),
}

/// Lays out the postcard half `card` of a sheet `sheet_width` wide. The stamp box sits at
/// the sheet's top-right corner and the band across the top stays free, for more stamps or a
/// long label printed at the post office. The lines fill the rest from the foot up, no
/// further apart than a ruled notebook's: on a landscape card (its half stands) the
/// `message_lines`, half a line's gap, then the `address_lines`; on a portrait card (its
/// half lies, too low to stack them) the message in the left column and the address in the
/// right, a rule between them.
pub fn layout(sheet_width: f32, card: RectMm, message_lines: u8, address_lines: u8) -> CardLayout {
    let stamp = RectMm {
        x: sheet_width - EDGE_MM - STAMP_MM.0,
        y: EDGE_MM,
        width: STAMP_MM.0,
        height: STAMP_MM.1,
    };
    let standing = card.height >= card.width;
    let top = stamp.y + stamp.height + STAMP_GAP_MM;
    let (x0, x1) = (card.x + EDGE_MM, card.x + card.width - EDGE_MM);
    let bottom = card.y + card.height - EDGE_MM - CREDIT_ROOM_MM;
    let message_lines = message_lines.max(1);
    let address_lines = address_lines.clamp(1, MAX_ADDRESS_LINES);
    // The address keeps the room of its three lines whatever their number.
    let address_step = |step: f32| step * f32::from(ADDRESS_LINES) / f32::from(address_lines);
    let line = |y: f32, x0: f32, x1: f32| Line { y, x0, x1 };
    if standing {
        let address_at = f32::from(message_lines) + PART_GAP;
        let last = address_at + f32::from(ADDRESS_LINES - 1);
        let step = ((bottom - top) / (last + 1.0)).min(MAX_STEP_MM);
        // Row `at` (in lines from the first), the last one just above the credit's room.
        let y = |at: f32| bottom - step * (last - at) - step * 0.15;
        let a_step = address_step(step);
        return CardLayout {
            stamp,
            message: (0..message_lines)
                .map(|k| line(y(f32::from(k)), x0, x1))
                .collect(),
            // From the address's last line up, as many as it has.
            address: (0..address_lines)
                .map(|k| line(y(last) - a_step * f32::from(address_lines - 1 - k), x0, x1))
                .collect(),
            step,
            address_step: a_step,
            divider: None,
            credit: (x0, bottom + 0.6),
        };
    }
    let mid = f32::midpoint(x0, x1);
    let rows = message_lines.max(ADDRESS_LINES);
    let step = ((bottom - top) / f32::from(rows)).min(MAX_STEP_MM);
    let a_step = address_step(step);
    // Both columns end on the same last line.
    let column = |count: u8, step: f32, x0: f32, x1: f32| {
        (0..count)
            .map(|k| {
                line(
                    bottom - step * f32::from(count - 1 - k) - step * 0.15,
                    x0,
                    x1,
                )
            })
            .collect()
    };
    CardLayout {
        stamp,
        message: column(message_lines, step, x0, mid - COLUMN_GAP_MM / 2.0),
        address: column(address_lines, a_step, mid + COLUMN_GAP_MM / 2.0, x1),
        step,
        address_step: a_step,
        divider: Some((mid, bottom - step * f32::from(rows), bottom)),
        credit: (x0, bottom + 0.6),
    }
}

/// What a postcard half draws.
pub struct CardSpec<'a> {
    pub card: RectMm,
    pub sheet_width: f32,
    pub settings: &'a PostcardSettings,
    pub text: Option<&'a PostcardText>,
    /// The face for the typed text; `None` uses the bundled one.
    pub font: Option<&'a FontArc>,
    pub paper: BorderColor,
}

/// Which parts of a card's text do not fit their lines and are cut on the print.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Fit {
    pub message_cut: bool,
    pub address_cut: bool,
}

/// A card laid out for its text: the lines (with a fourth address line when the address
/// needs it), the text's size in pixels and each part's rows, at `density` pixels per mm.
struct Typeset {
    layout: CardLayout,
    message_px: f32,
    address_px: f32,
    message: Vec<String>,
    address: Vec<String>,
}

fn typeset(spec: &CardSpec<'_>, faces: Faces<'_>, density: f32) -> Typeset {
    let settings = spec.settings;
    let empty = PostcardText::default();
    let text = spec.text.unwrap_or(&empty);
    let set = |address_lines: u8| {
        let layout = layout(spec.sheet_width, spec.card, settings.lines, address_lines);
        let size = |step: f32| {
            (step * TEXT_EM).min(crate::settings::postcard_text_mm(settings.text_size)) * density
        };
        let (message_px, address_px) = (size(layout.step), size(layout.address_step));
        let rows = |part: &str, lines: &[Line], text_px: f32| {
            // Rows past the last line are cut; they are measured as long as it, so what is
            // left over counts as rows, not as one letter a row.
            let room = |row: usize| {
                lines
                    .get(row)
                    .or(lines.last())
                    .map_or(0.0, |l| (l.x1 - l.x0 - TEXT_PAD_MM) * density)
            };
            if part.trim().is_empty() {
                return Vec::new();
            }
            // A break or blanks typed after the last word print nothing: no row for them.
            let mut rows = wrap(faces, text_px, part, room);
            while rows.last().is_some_and(|r| r.trim().is_empty()) {
                rows.pop();
            }
            rows
        };
        let message = rows(&text.message, &layout.message, message_px);
        let address = rows(&text.address, &layout.address, address_px);
        Typeset {
            layout,
            message_px,
            address_px,
            message,
            address,
        }
    };
    let three = set(ADDRESS_LINES);
    if three.address.len() > usize::from(ADDRESS_LINES) {
        set(MAX_ADDRESS_LINES)
    } else {
        three
    }
}

/// Whether the card's text fits its lines (FR-073), as the print sets it: the warning on a
/// card whose message or address would be cut.
pub fn fit(spec: &CardSpec<'_>) -> Fit {
    let Some(bundled) = font() else {
        return Fit::default();
    };
    let faces = Faces {
        main: spec.font.unwrap_or(bundled),
        fallback: crate::render::regular(),
    };
    // Widths scale with the size, so any density gives the print's rows.
    let t = typeset(spec, faces, 12.0);
    Fit {
        message_cut: t.message.len() > t.layout.message.len(),
        address_cut: t.address.len() > t.layout.address.len(),
    }
}

/// Grey for the stamp box and the credit, and for the writing lines, on this paper.
fn inks(paper: BorderColor) -> ([u8; 3], [u8; 3]) {
    match paper {
        BorderColor::White => ([140, 140, 140], [185, 185, 185]),
        BorderColor::Black => ([150, 150, 150], [95, 95, 95]),
    }
}

/// Draws the postcard half onto `canvas`, `density` pixels per millimetre.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn draw(canvas: &mut RgbImage, density: f32, spec: &CardSpec<'_>) {
    let settings = spec.settings;
    let Some(bundled) = font() else {
        return;
    };
    // The chosen face, with Inter for the letters it lacks (Xanh Mono has no Greek or
    // Cyrillic): an address copied from anywhere in Europe prints as typed.
    let faces = Faces {
        main: spec.font.unwrap_or(bundled),
        fallback: crate::render::regular(),
    };
    let set = typeset(spec, faces, density);
    let layout = &set.layout;
    let (outline, rule) = inks(spec.paper);
    let px = |mm: f32| (mm * density).round().max(0.0) as u32;
    let thick = |mm: f32| (mm * density).round().max(1.0) as u32;
    draw_stamp_box(canvas, density, layout.stamp, outline);
    if let Some((x, top, bottom)) = layout.divider {
        crate::sheet::fill_rect(
            canvas,
            px(x),
            px(top),
            thick(LINE_MM),
            px(bottom - top),
            rule,
        );
    }
    for line in layout.message.iter().chain(&layout.address) {
        crate::sheet::fill_rect(
            canvas,
            px(line.x0),
            px(line.y),
            px(line.x1 - line.x0),
            thick(LINE_MM),
            rule,
        );
    }
    let credit = settings.credit.trim();
    if !credit.is_empty() {
        let cpx = CREDIT_MM * density;
        let room = |_| (spec.card.width - 2.0 * EDGE_MM) * density;
        if let Some(row) = wrap(faces, cpx, credit, room).first() {
            let (x, top) = layout.credit;
            let ascent = faces.main.as_scaled(PxScale::from(cpx)).ascent();
            faces.draw(
                canvas,
                row,
                x * density,
                top * density + ascent,
                cpx,
                outline,
                TextStyle::default(),
            );
        }
    }
    // Each part on its own lines, wrapped to their lengths; what does not fit is cut.
    let ink = settings.ink.rgb(spec.paper);
    for (rows, lines, style, text_px, step) in [
        (
            &set.message,
            &layout.message,
            settings.message_style,
            set.message_px,
            layout.step,
        ),
        (
            &set.address,
            &layout.address,
            settings.address_style,
            set.address_px,
            layout.address_step,
        ),
    ] {
        for (row, line) in rows.iter().zip(lines) {
            let baseline = (line.y - step * BASELINE_GAP) * density;
            faces.draw(
                canvas,
                row,
                (line.x0 + TEXT_PAD_MM) * density,
                baseline,
                text_px,
                ink,
                style,
            );
        }
    }
}

/// The stamp box, dashed as on a postcard, with STAMP in its middle.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn draw_stamp_box(canvas: &mut RgbImage, density: f32, s: RectMm, outline: [u8; 3]) {
    let px = |mm: f32| (mm * density).round().max(0.0) as u32;
    let thick = |mm: f32| (mm * density).round().max(1.0) as u32;
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
    let Some(face) = font() else {
        return;
    };
    let word_px = STAMP_WORD_MM * density / cap_height(face);
    let word_w = text_width(face, word_px, STAMP_WORD, 0.08);
    let scaled = face.as_scaled(PxScale::from(word_px));
    draw_text(
        canvas,
        &TextRun {
            font: face,
            text: STAMP_WORD,
            px: word_px,
            tracking: 0.08,
        },
        (s.x + s.width / 2.0) * density - word_w / 2.0,
        (s.y + s.height / 2.0) * density - f32::midpoint(scaled.ascent(), scaled.descent()),
        outline,
        1.0,
    );
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

    /// Draws `text` from `left` with its baseline at `baseline`, each face on that baseline,
    /// bold, italic or underlined as `style` says. Italic is Xanh Mono's own when the text
    /// is in it, an oblique otherwise; bold is a second stroke beside the first.
    #[allow(
        clippy::too_many_arguments,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    fn draw(
        &self,
        canvas: &mut RgbImage,
        text: &str,
        left: f32,
        baseline: f32,
        px: f32,
        ink: [u8; 3],
        style: TextStyle,
    ) {
        let bundled = font();
        let mut x = left;
        for (face, run) in self.runs(text) {
            let own_italic = (style.italic && bundled.is_some_and(|b| std::ptr::eq(b, face)))
                .then(|| ITALIC.as_ref())
                .flatten();
            let face = own_italic.unwrap_or(face);
            let ascent = face.as_scaled(PxScale::from(px)).ascent();
            let run_text = TextRun {
                font: face,
                text: run,
                px,
                tracking: 0.0,
            };
            let slant = if style.italic && own_italic.is_none() {
                SLANT
            } else {
                0.0
            };
            if style.bold {
                draw_text_bold(
                    canvas,
                    &run_text,
                    x,
                    baseline - ascent,
                    ink,
                    BOLD_EM * px,
                    slant,
                );
            } else if slant > 0.0 {
                draw_text_slanted(canvas, &run_text, x, baseline - ascent, ink, slant);
            } else {
                draw_text(canvas, &run_text, x, baseline - ascent, ink, 1.0);
            }
            x += text_width(face, px, run, 0.0);
        }
        if style.underline && x > left {
            crate::sheet::fill_rect(
                canvas,
                left.max(0.0) as u32,
                (baseline + UNDERLINE_EM * px).max(0.0) as u32,
                (x - left).round() as u32,
                (UNDERLINE_THICK_EM * px).round().max(1.0) as u32,
                ink,
            );
        }
    }
}

/// The height of a capital (its "H") per pixel of em, so the stamp word is sized by its
/// visible letters.
fn cap_height(font: &FontArc) -> f32 {
    let scaled = font.as_scaled(PxScale::from(100.0));
    let bounds = font.glyph_bounds(&scaled.scaled_glyph('H'));
    (bounds.height() / 100.0).max(0.3)
}

/// `text` broken into rows, row `n` no wider than `room(n)` pixels: at its own line breaks,
/// then between words, and inside a word too long for a row.
pub fn wrap(faces: Faces<'_>, px: f32, text: &str, room: impl Fn(usize) -> f32) -> Vec<String> {
    let mut rows: Vec<String> = Vec::new();
    let fits = |s: &str, row: usize| faces.width(px, s) <= room(row);
    for paragraph in text.split('\n') {
        let mut row = String::new();
        for word in paragraph.split_whitespace() {
            let candidate = if row.is_empty() {
                word.to_owned()
            } else {
                format!("{row} {word}")
            };
            if fits(&candidate, rows.len()) {
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
                if !fits(&next, rows.len()) && !row.is_empty() {
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
    use crate::settings::PostcardInk;

    fn half(x: f32, y: f32, width: f32, height: f32) -> RectMm {
        RectMm {
            x,
            y,
            width,
            height,
        }
    }

    /// A white landscape 10 × 15 sheet's card half drawn at 10 px a millimetre.
    fn drawn(settings: &PostcardSettings, text: Option<&PostcardText>) -> RgbImage {
        let mut canvas = crate::render::filled(1520, 1020, [255, 255, 255]);
        draw(
            &mut canvas,
            10.0,
            &CardSpec {
                card: half(76.0, 0.0, 76.0, 102.0),
                sheet_width: 152.0,
                settings,
                text,
                font: None,
                paper: BorderColor::White,
            },
        );
        canvas
    }

    fn text(message: &str, address: &str) -> PostcardText {
        PostcardText {
            message: message.into(),
            address: address.into(),
        }
    }

    /// FR-073: the stamp box sits 5 mm from the sheet's top-right corner and the band across
    /// the top stays free for more stamps or a long label; the lines sit at the foot, no more
    /// than 8 mm apart: stacked on a landscape card, the address half a line under the
    /// message; in two columns with a rule between them on a portrait card.
    #[test]
    fn fr_073_lines_sit_together_at_the_foot() {
        let landscape = layout(152.0, half(76.0, 0.0, 76.0, 102.0), 4, ADDRESS_LINES);
        let portrait = layout(102.0, half(0.0, 0.0, 102.0, 76.0), 4, ADDRESS_LINES);
        assert_eq!(landscape.stamp, half(127.0, 5.0, 20.0, 25.0));
        assert_eq!(portrait.stamp, half(77.0, 5.0, 20.0, 25.0));
        for (l, bottom) in [(&landscape, 102.0), (&portrait, 76.0)] {
            assert_eq!((l.message.len(), l.address.len()), (4, 3));
            assert!(l.step <= MAX_STEP_MM + 1e-3);
            let lines: Vec<&Line> = l.message.iter().chain(&l.address).collect();
            // Nothing in the band of the stamps, nothing in the credit's room.
            assert!(
                lines
                    .iter()
                    .all(|line| line.y - l.step > l.stamp.y + l.stamp.height)
            );
            assert!(
                lines
                    .iter()
                    .all(|line| line.y < bottom - EDGE_MM - CREDIT_ROOM_MM)
            );
        }
        let gap = landscape.address[0].y - landscape.message[3].y;
        assert!(
            (gap - (1.0 + PART_GAP) * landscape.step).abs() < 1e-3,
            "{gap}"
        );
        assert!(landscape.divider.is_none() && landscape.address[0].x1 > 140.0);
        // Portrait: the message on the left, the address on the right, ending together.
        let Some((rule, _, _)) = portrait.divider else {
            panic!("a portrait card has a rule between its columns");
        };
        assert!(portrait.message.iter().all(|line| line.x1 < rule));
        assert!(portrait.address.iter().all(|line| line.x0 > rule));
        assert!((portrait.message[3].y - portrait.address[2].y).abs() < 1e-3);
    }

    /// FR-073: the address has 3 lines, and a fourth when it needs one; text that still does
    /// not fit is reported for the warning on the card, part by part.
    #[test]
    fn fr_073_a_long_address_gets_a_fourth_line_and_a_cut_is_reported() {
        let settings = PostcardSettings {
            enabled: true,
            ..PostcardSettings::default()
        };
        let card = |text: &PostcardText| -> (usize, Fit) {
            let spec = CardSpec {
                card: half(76.0, 0.0, 76.0, 102.0),
                sheet_width: 152.0,
                settings: &settings,
                text: Some(text),
                font: None,
                paper: BorderColor::White,
            };
            let Some(bundled) = font() else {
                panic!("the bundled face loads");
            };
            let faces = Faces {
                main: bundled,
                fallback: crate::render::regular(),
            };
            (typeset(&spec, faces, 12.0).layout.address.len(), fit(&spec))
        };
        // Four address lines take the room of three; the message's lines do not move.
        for card in [half(76.0, 0.0, 76.0, 102.0), half(0.0, 0.0, 102.0, 76.0)] {
            let width = if card.x > 0.0 { 152.0 } else { 102.0 };
            let (three, four) = (layout(width, card, 4, 3), layout(width, card, 4, 4));
            assert_eq!(three.message, four.message);
            assert!((three.step - four.step).abs() < 1e-6);
            let room =
                |l: &CardLayout| l.address[l.address.len() - 1].y - l.address[0].y + l.address_step;
            assert!((room(&three) - room(&four)).abs() < 1e-3);
            assert!((four.address_step - three.step * 0.75).abs() < 1e-3);
        }
        let short = text(
            "Ahoj Praha!",
            "Národní muzeum\nVáclavské náměstí 68\n110 00 Praha 1",
        );
        assert_eq!(card(&short), (3, Fit::default()));
        let paris = text(
            "",
            "Cao-Minh TRUONG / Chambre 248, Collège Franco-Britannique, 9B, Boulevard Jourdan, 75014, Paris, France",
        );
        assert_eq!(
            card(&paris),
            (4, Fit::default()),
            "the owner's address fits on four"
        );
        // As typed in the app: one line break, doubled spaces, a trailing break. With three
        // breaks it does need a fifth line, and the warning is right.
        let typed = "Cao-Minh TRUONG  /\nChambre 248, Collège Franco-Britannique, 9B, Boulevard Jourdan, 75014, Paris, France";
        for t in [typed.to_owned(), format!("{typed}\n")] {
            assert_eq!(card(&text("", &t)), (4, Fit::default()), "{t:?}");
        }
        let broken = "Cao-Minh TRUONG /\nChambre 248, Collège Franco-Britannique,\n9B, Boulevard Jourdan, 75014, Paris, France";
        assert!(card(&text("", broken)).1.address_cut);
        let long = text(
            &"Liebe Grüße aus Budapest! ".repeat(20),
            &"Hauptstraße 1, ".repeat(12),
        );
        assert_eq!(
            card(&long).1,
            Fit {
                message_cut: true,
                address_cut: true
            }
        );
    }

    /// The message and the address each go on their own lines: an address never spills onto
    /// the message's lines, and a long message is cut at its last line.
    #[test]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn fr_073_each_part_keeps_to_its_lines() {
        let settings = PostcardSettings {
            enabled: true,
            ..PostcardSettings::default()
        };
        let l = layout(
            152.0,
            half(76.0, 0.0, 76.0, 102.0),
            settings.lines,
            ADDRESS_LINES,
        );
        let band = |canvas: &RgbImage, line: &Line| {
            let (y0, y1) = (
                ((line.y - l.step * 0.8) * 10.0) as u32,
                (line.y * 10.0) as u32 - 3,
            );
            (y0..y1)
                .flat_map(|y| {
                    (((line.x0 + 2.0) * 10.0) as u32..(line.x1 * 10.0) as u32).map(move |x| (x, y))
                })
                .filter(|&(x, y)| canvas.data[((y * 1520 + x) * 3) as usize] < 120)
                .count()
        };
        let only_address = drawn(
            &settings,
            Some(&text("", "Anna Schmidt\nHauptstraße 1\n10115 Berlin")),
        );
        assert!(l.message.iter().all(|line| band(&only_address, line) == 0));
        assert!(l.address.iter().all(|line| band(&only_address, line) > 0));
        let long = "Liebe Grüße aus Budapest! ".repeat(20);
        let cut = drawn(&settings, Some(&text(&long, "")));
        assert!(l.message.iter().all(|line| band(&cut, line) > 0));
        assert!(l.address.iter().all(|line| band(&cut, line) == 0));
    }

    /// FR-073: bold darkens more, italic leans, underline draws a line under the row; the
    /// text takes its ink and the lines stay grey.
    #[test]
    fn fr_073_styles_and_ink_show_on_the_print() {
        let count = |canvas: &RgbImage, test: &dyn Fn(&[u8; 3]) -> bool| {
            canvas
                .data
                .as_chunks::<3>()
                .0
                .iter()
                .filter(|p| test(p))
                .count()
        };
        let dark = |p: &[u8; 3]| p[0] < 100;
        let plain = PostcardSettings {
            enabled: true,
            ..PostcardSettings::default()
        };
        let message = text("Liebe Grüße", "");
        let base = count(&drawn(&plain, Some(&message)), &dark);
        for style in [
            TextStyle {
                bold: true,
                ..TextStyle::default()
            },
            TextStyle {
                underline: true,
                ..TextStyle::default()
            },
        ] {
            let styled = PostcardSettings {
                message_style: style,
                ..plain.clone()
            };
            assert!(
                count(&drawn(&styled, Some(&message)), &dark) > base * 11 / 10,
                "{style:?}"
            );
        }
        let italic = PostcardSettings {
            message_style: TextStyle {
                italic: true,
                ..TextStyle::default()
            },
            ..plain.clone()
        };
        assert_ne!(
            drawn(&italic, Some(&message)).data,
            drawn(&plain, Some(&message)).data
        );
        let blue = PostcardSettings {
            ink: PostcardInk::Blue,
            ..plain.clone()
        };
        let blue_px = count(&drawn(&blue, Some(&message)), &|[r, g, b]| {
            *b > 120 && *r < 60 && *g < 90
        });
        assert!(blue_px > 100, "{blue_px} blue pixels");
        assert_eq!(PostcardInk::Auto.rgb(BorderColor::Black), [235, 235, 235]);
    }

    /// FR-073: the credit prints small at the foot of the half, and nothing prints there
    /// without one.
    #[test]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn fr_073_the_credit_sits_at_the_foot() {
        let at_foot = |credit: &str| {
            let settings = PostcardSettings {
                enabled: true,
                credit: credit.into(),
                ..PostcardSettings::default()
            };
            let canvas = drawn(&settings, None);
            let y0 = ((102.0 - EDGE_MM - CREDIT_ROOM_MM + 0.3) * 10.0) as u32;
            (y0..1020)
                .flat_map(|y| (760..1520).map(move |x| (x, y)))
                .filter(|&(x, y)| canvas.data[((y * 1520 + x) * 3) as usize] < 200)
                .count()
        };
        assert_eq!(at_foot(""), 0);
        assert_eq!(at_foot("   "), 0);
        assert!(at_foot("Photo © Anna Schmidt") > 50);
    }

    /// FR-073: an address in any European script prints: the Latin letters of French,
    /// Czech, Hungarian or Polish are in Xanh Mono, its italic too, and Greek or Cyrillic
    /// come from Inter.
    #[test]
    fn fr_073_european_addresses_find_their_letters() {
        let (Some(mono), Some(italic), Some(inter)) =
            (font(), ITALIC.as_ref(), crate::render::regular())
        else {
            panic!("the bundled faces load");
        };
        for c in "àçéœÿ áěí čřůž őű ąłż șț åøæ ñ ğış āėų ơư"
            .chars()
            .filter(|c| !c.is_whitespace())
        {
            assert_ne!(mono.glyph_id(c).0, 0, "{c} in Xanh Mono");
            assert_ne!(italic.glyph_id(c).0, 0, "{c} in Xanh Mono Italic");
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
    /// row; each row has its own room.
    #[test]
    fn fr_073_text_wraps_to_each_lines_length() {
        let Some(face) = font() else {
            panic!("the bundled face loads");
        };
        let only = Faces {
            main: face,
            fallback: None,
        };
        let room = text_width(face, 20.0, "Liebe Grüße", 0.0) + 1.0;
        assert_eq!(
            wrap(only, 20.0, "Liebe Grüße aus\nBudapest", |_| room),
            ["Liebe Grüße", "aus", "Budapest"]
        );
        let long = wrap(only, 20.0, "Donaudampfschifffahrt", |_| room / 2.0);
        assert!(long.len() > 1 && long.concat() == "Donaudampfschifffahrt");
        // A short first line, then long ones.
        let rows = wrap(only, 20.0, "Liebe Grüße aus Budapest", |row| {
            if row == 0 { room / 2.0 } else { room * 3.0 }
        });
        assert_eq!(rows, ["Liebe", "Grüße aus Budapest"]);
    }
}
