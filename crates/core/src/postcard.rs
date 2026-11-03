// SPDX-License-Identifier: GPL-3.0-only
//! The postcard half of a collage sheet (FR-073): a dashed stamp box at the sheet's top
//! right and writing lines over the whole half, the message on the top ones and the address
//! on the bottom ones, short beside the stamp. Each card's own text is typed on them in a
//! typewriter face (Xanh Mono, OFL) or the user's font (FR-042), bold, italic or underlined
//! per part. Drawn on the sheet's paper after the photo, in millimetres turned into pixels.

use std::sync::LazyLock;

use ab_glyph::{Font, FontArc, PxScale, ScaleFont};

use crate::image::RgbImage;
use crate::library::PostcardText;
use crate::settings::{BorderColor, PostcardSettings, TextStyle};
use crate::sheet::RectMm;
use crate::text::{TextRun, draw_text, draw_text_slanted, text_width};

static XANH_MONO: &[u8] = include_bytes!("../assets/fonts/XanhMono-Regular.ttf");
static XANH_MONO_ITALIC: &[u8] = include_bytes!("../assets/fonts/XanhMono-Italic.ttf");
static FONT: LazyLock<Option<FontArc>> =
    LazyLock::new(|| FontArc::try_from_vec(XANH_MONO.to_vec()).ok());
static ITALIC: LazyLock<Option<FontArc>> =
    LazyLock::new(|| FontArc::try_from_vec(XANH_MONO_ITALIC.to_vec()).ok());

/// The stamp box, width × height: about the size of a German or Hungarian stamp, so a real
/// one covers it without reaching the address.
pub const STAMP_MM: (f32, f32) = (26.0, 32.0);
/// Space between the sheet's edge and what is drawn, and between the stamp and the lines.
pub const EDGE_MM: f32 = 5.0;
const STAMP_GAP_MM: f32 = 4.0;
/// The address lines, under the message's: name, street, town and country.
pub const ADDRESS_LINES: u8 = 3;
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
/// Bold, underline and oblique, in em: the second stroke's offset, the underline's depth
/// and thickness, and the slant of a face with no italic.
const BOLD_EM: f32 = 0.045;
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
    /// The distance between two lines.
    pub step: f32,
    /// The credit's left edge and its top.
    pub credit: (f32, f32),
}

/// Lays out the postcard half `card` of a sheet `sheet_width` wide: the stamp box at the
/// sheet's top-right corner, then `message_lines` lines, one blank, and the address lines,
/// spread evenly from the half's top to the credit's room at its foot; a line beside the
/// stamp stops short of it.
pub fn layout(sheet_width: f32, card: RectMm, message_lines: u8) -> CardLayout {
    let stamp = RectMm {
        x: sheet_width - EDGE_MM - STAMP_MM.0,
        y: EDGE_MM,
        width: STAMP_MM.0,
        height: STAMP_MM.1,
    };
    let top = card.y + EDGE_MM;
    let bottom = card.y + card.height - EDGE_MM - CREDIT_ROOM_MM;
    let message_lines = message_lines.max(1);
    let rows = f32::from(message_lines + 1 + ADDRESS_LINES);
    let step = (bottom - top) / rows;
    let (x0, x1) = (card.x + EDGE_MM, card.x + card.width - EDGE_MM);
    let line = |row: u8| {
        let y = top + step * (f32::from(row) + 1.0) - step * 0.15;
        // The row's text reaches up a line spacing: beside the stamp it stops before it.
        let beside = y - step < stamp.y + stamp.height && stamp.x < x1;
        Line {
            y,
            x0,
            x1: if beside { stamp.x - STAMP_GAP_MM } else { x1 },
        }
    };
    CardLayout {
        stamp,
        message: (0..message_lines).map(line).collect(),
        address: (0..ADDRESS_LINES)
            .map(|k| line(message_lines + 1 + k))
            .collect(),
        step,
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
    let layout = layout(spec.sheet_width, spec.card, settings.lines);
    let (outline, rule) = inks(spec.paper);
    let px = |mm: f32| (mm * density).round().max(0.0) as u32;
    let thick = |mm: f32| (mm * density).round().max(1.0) as u32;
    draw_stamp_box(canvas, density, layout.stamp, outline);
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
    let Some(bundled) = font() else {
        return;
    };
    // The chosen face, with Inter for the letters it lacks (Xanh Mono has no Greek or
    // Cyrillic): an address copied from anywhere in Europe prints as typed.
    let faces = Faces {
        main: spec.font.unwrap_or(bundled),
        fallback: crate::render::regular(),
    };
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
    let Some(text) = spec.text else {
        return;
    };
    // Each part on its own lines, wrapped to their lengths; what does not fit is cut.
    let text_px = (layout.step * TEXT_EM)
        .min(crate::settings::postcard_text_mm(settings.text_size))
        * density;
    let ink = settings.ink.rgb(spec.paper);
    for (part, lines, style) in [
        (&text.message, &layout.message, settings.message_style),
        (&text.address, &layout.address, settings.address_style),
    ] {
        let room = |row: usize| {
            lines
                .get(row)
                .map_or(0.0, |l| (l.x1 - l.x0 - TEXT_PAD_MM) * density)
        };
        for (row, line) in wrap(faces, text_px, part, room).iter().zip(lines) {
            let baseline = (line.y - layout.step * BASELINE_GAP) * density;
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
            let strokes: &[f32] = if style.bold { &[0.0, BOLD_EM] } else { &[0.0] };
            for dx in strokes {
                let run_text = TextRun {
                    font: face,
                    text: run,
                    px,
                    tracking: 0.0,
                };
                let at = x + dx * px;
                if style.italic && own_italic.is_none() {
                    draw_text_slanted(canvas, &run_text, at, baseline - ascent, ink, SLANT);
                } else {
                    draw_text(canvas, &run_text, at, baseline - ascent, ink, 1.0);
                }
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

    /// FR-073: the stamp box sits 5 mm from the sheet's top-right corner; the message's lines
    /// come first, then a blank one and the 3 address lines, evenly spaced over the half; a
    /// line beside the stamp stops before it, and a portrait card uses the room left of it.
    #[test]
    fn fr_073_message_and_address_lines_fill_the_half() {
        let landscape = layout(152.0, half(76.0, 0.0, 76.0, 102.0), 4);
        assert_eq!(landscape.stamp, half(121.0, 5.0, 26.0, 32.0));
        assert_eq!((landscape.message.len(), landscape.address.len()), (4, 3));
        let ys: Vec<f32> = landscape
            .message
            .iter()
            .chain(&landscape.address)
            .map(|l| l.y)
            .collect();
        assert!(ys.windows(2).all(|w| w[1] > w[0]));
        // The blank line between the parts: two spacings from the last message line.
        let gap = landscape.address[0].y - landscape.message[3].y;
        assert!((gap - 2.0 * landscape.step).abs() < 1e-3, "{gap}");
        for line in landscape.message.iter().chain(&landscape.address) {
            let beside = line.y - landscape.step < landscape.stamp.y + landscape.stamp.height;
            assert_eq!(line.x1 < landscape.stamp.x, beside, "{line:?}");
            assert!(line.y < 102.0 - EDGE_MM - CREDIT_ROOM_MM + 1e-3);
        }
        let portrait = layout(102.0, half(0.0, 0.0, 102.0, 76.0), 3);
        assert_eq!(portrait.stamp, half(71.0, 5.0, 26.0, 32.0));
        assert!(portrait.message[0].x1 < 71.0 && portrait.address[2].x1 > 90.0);
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
        let l = layout(152.0, half(76.0, 0.0, 76.0, 102.0), settings.lines);
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
        for c in "àçéœÿ čřůž őű ąłż șț åøæ ñ ğış āėų ơư"
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
