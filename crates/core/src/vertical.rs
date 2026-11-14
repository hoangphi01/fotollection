// SPDX-License-Identifier: GPL-3.0-only
//! Japanese text set top to bottom (縦書き), for the はがき (FR-130, ADR-0052). Each character
//! takes one em down its column, the columns run right to left, and the punctuation and long
//! vowel mark turn by the face's own vertical forms: its GSUB `vert` and `vrt2` single
//! substitutions, read once with `ttf-parser`, no shaper. Lines break by 禁則: a mark that may
//! not start a column (、。」ー, a small kana) pulls the character before it along, or hangs
//! below the column if it is 、 or 。; an opening bracket never ends one.

use std::collections::HashMap;
use std::sync::LazyLock;

use ab_glyph::{Font, FontArc, GlyphId, PxScale, ScaleFont, point};

use crate::image::RgbImage;

const KLEE_ONE: &[u8] = include_bytes!("../assets/fonts/KleeOne-SemiBold-subset.ttf");

/// A face and its vertical forms.
pub struct VerticalFace {
    pub font: FontArc,
    /// Horizontal glyph → its vertical form.
    forms: HashMap<u16, u16>,
    /// The em box's top above the baseline, in em: the typographic ascender (0.88 in Klee
    /// One), not the line's ascent (1.16), so a character fills its cell.
    em_ascent: f32,
}

/// Klee One, the はがき's print face (FONTS.md), with its vertical forms.
pub static KLEE: LazyLock<Option<VerticalFace>> = LazyLock::new(|| VerticalFace::new(KLEE_ONE));

impl VerticalFace {
    /// A face from its bytes, `None` if they are not a font.
    pub fn new(bytes: &'static [u8]) -> Option<Self> {
        let em_ascent = ttf_parser::Face::parse(bytes, 0)
            .ok()
            .and_then(|f| {
                let per_em = f32::from(f.units_per_em());
                f.typographic_ascender()
                    .map(|a| f32::from(a) / per_em)
                    .filter(|a| (0.5..=1.0).contains(a))
            })
            .unwrap_or(0.88);
        Some(Self {
            font: FontArc::try_from_slice(bytes).ok()?,
            forms: vertical_forms(bytes),
            em_ascent,
        })
    }

    /// The scale that makes one em `em` pixels: `ab_glyph`'s `PxScale` is the line's height
    /// (ascent to descent), which a Japanese face makes taller than its em.
    pub fn scale(&self, em: f32) -> PxScale {
        let per_em = self.font.units_per_em().unwrap_or(1000.0);
        PxScale::from(em * self.font.height_unscaled() / per_em)
    }

    /// The glyph a character takes in a column: its vertical form when the face has one.
    pub fn glyph(&self, c: char) -> GlyphId {
        let id = self.font.glyph_id(c);
        self.forms.get(&id.0).map_or(id, |v| GlyphId(*v))
    }
}

/// A face's vertical forms from its GSUB `vert` and `vrt2` features (single substitutions
/// only, the kind a Japanese face uses for them); empty when it has none.
fn vertical_forms(bytes: &[u8]) -> HashMap<u16, u16> {
    use ttf_parser::Tag;
    use ttf_parser::gsub::{SingleSubstitution, SubstitutionSubtable};
    let mut forms = HashMap::new();
    let Ok(face) = ttf_parser::Face::parse(bytes, 0) else {
        return forms;
    };
    let Some(gsub) = face.tables().gsub else {
        return forms;
    };
    let glyphs = face.number_of_glyphs();
    for tag in [b"vert", b"vrt2"] {
        let Some(feature) = gsub.features.find(Tag::from_bytes(tag)) else {
            continue;
        };
        for index in feature.lookup_indices {
            let Some(lookup) = gsub.lookups.get(index) else {
                continue;
            };
            for i in 0..lookup.subtables.len() {
                let Some(SubstitutionSubtable::Single(single)) =
                    lookup.subtables.get::<SubstitutionSubtable>(i)
                else {
                    continue;
                };
                for g in 0..glyphs {
                    let glyph = ttf_parser::GlyphId(g);
                    let Some(at) = single.coverage().get(glyph) else {
                        continue;
                    };
                    let to = match single {
                        SingleSubstitution::Format1 { delta, .. } => {
                            Some(g.wrapping_add_signed(delta))
                        }
                        SingleSubstitution::Format2 { substitutes, .. } => {
                            substitutes.get(at).map(|s| s.0)
                        }
                    };
                    if let Some(to) = to {
                        forms.entry(g).or_insert(to);
                    }
                }
            }
        }
    }
    forms
}

/// Marks that may not start a column (行頭禁則).
const NO_START: &str =
    "、。，．」』）】〕〉》ー・ぁぃぅぇぉっゃゅょゎァィゥェォッャュョヮヵヶ々！？!?";
/// Marks that may hang one cell below a full column instead (ぶら下げ).
const HANGING: &str = "、。，．";
/// Marks that may not end a column (行末禁則).
const NO_END: &str = "「『（【〔〈《";

/// The characters of each column, right to left: `text` broken at its line breaks and every
/// `capacity` characters, by 禁則. A hanging mark makes a column one longer.
pub fn columns(text: &str, capacity: usize) -> Vec<Vec<char>> {
    let capacity = capacity.max(2);
    let mut out: Vec<Vec<char>> = Vec::new();
    for paragraph in text.split('\n') {
        let mut column: Vec<char> = Vec::new();
        for c in paragraph.chars() {
            if column.len() >= capacity {
                if HANGING.contains(c) && column.len() == capacity {
                    column.push(c);
                    continue;
                }
                let mut next = Vec::new();
                if NO_START.contains(c) {
                    // Pull the last character along, so the mark does not start the column.
                    if let Some(last) = column.pop() {
                        next.push(last);
                    }
                }
                out.push(std::mem::take(&mut column));
                column = next;
            } else if column.len() + 1 == capacity && NO_END.contains(c) {
                // An opening bracket at the column's foot goes to the next one.
                out.push(std::mem::take(&mut column));
            }
            column.push(c);
        }
        out.push(column);
    }
    out
}

/// Text set in columns, ready to draw.
pub struct Set {
    /// Each glyph and the top-left of its em cell, in pixels.
    pub glyphs: Vec<(GlyphId, f32, f32)>,
    /// How many columns it took.
    pub columns: usize,
    /// Some of the text did not fit and is left out.
    pub cut: bool,
}

/// Lays `text` out top to bottom from (`right`, `top`): `em` pixels a character, columns
/// `step` pixels apart, at most `height` pixels tall and `max_columns` wide; every column after
/// a paragraph's first starts `indent` characters lower (a Japanese address's second line).
#[allow(clippy::cast_precision_loss, clippy::too_many_arguments)]
pub fn set(
    face: &VerticalFace,
    text: &str,
    em: f32,
    right: f32,
    top: f32,
    height: f32,
    step: f32,
    max_columns: usize,
    indent: usize,
) -> Set {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let capacity = (height / em).floor().max(2.0) as usize;
    let scaled = face.font.as_scaled(face.scale(em));
    let mut glyphs = Vec::new();
    let mut n = 0;
    let mut cut = false;
    for paragraph in text.split('\n') {
        let mut first = true;
        // A continuation column is shorter by its indent.
        for column in columns(paragraph, capacity.saturating_sub(indent).max(2)) {
            if n >= max_columns {
                cut |= !column.is_empty();
                continue;
            }
            let offset = if first { 0 } else { indent };
            let x = right - (n as f32 + 1.0) * step + (step - em) / 2.0;
            for (row, c) in column.iter().enumerate() {
                let id = face.glyph(*c);
                // A narrow glyph (a digit, a Latin letter) is centred in its cell.
                let advance = scaled.h_advance(face.font.glyph_id(*c));
                let left = x + (em - advance.min(em)) / 2.0;
                glyphs.push((id, left, top + (row + offset) as f32 * em));
            }
            n += 1;
            first = false;
        }
    }
    Set {
        glyphs,
        columns: n,
        cut,
    }
}

/// Draws a set in `color`.
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub fn draw(canvas: &mut RgbImage, face: &VerticalFace, set: &Set, em: f32, color: [u8; 3]) {
    let scale = face.scale(em);
    let baseline = em * face.em_ascent;
    for &(id, left, top) in &set.glyphs {
        let glyph = id.with_scale_and_position(scale, point(left, top + baseline));
        let Some(outline) = face.font.outline_glyph(glyph) else {
            continue;
        };
        let b = outline.px_bounds();
        outline.draw(|gx, gy, coverage| {
            crate::text::blend(
                canvas,
                b.min.x as i32 + gx as i32,
                b.min.y as i32 + gy as i32,
                color,
                coverage,
            );
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn klee() -> &'static VerticalFace {
        match KLEE.as_ref() {
            Some(face) => face,
            None => panic!("Klee One is bundled"),
        }
    }

    #[test]
    fn fr_130_brackets_and_the_long_mark_turn_in_a_column() {
        let face = klee();
        for c in ['「', '」', 'ー', '、', '。'] {
            let horizontal = face.font.glyph_id(c);
            assert_ne!(horizontal.0, 0, "{c} is in the cut");
            assert_ne!(face.glyph(c), horizontal, "{c} has a vertical form");
        }
        // A kanji keeps its glyph.
        assert_eq!(face.glyph('鎌'), face.font.glyph_id('鎌'));
    }

    #[test]
    fn fr_130_lines_break_by_kinsoku() {
        let len = |cols: &[Vec<char>]| cols.iter().map(Vec::len).collect::<Vec<_>>();
        // 。 hangs below a full column.
        assert_eq!(len(&columns("あいうえお。", 5)), [6]);
        assert_eq!(len(&columns("あいうえおか", 5)), [5, 1]);
        // ー may not start a column: お comes along.
        assert_eq!(len(&columns("あいうえおー", 5)), [4, 2]);
        // 「 may not end one.
        let cols = columns("あいう「えお", 4);
        assert_eq!(cols[0], ['あ', 'い', 'う']);
        assert_eq!(cols[1][0], '「');
        // A line break starts a column.
        assert_eq!(len(&columns("鎌倉\n稲村ガ崎", 10)), [2, 4]);
    }

    #[test]
    fn fr_130_a_set_reports_what_does_not_fit() {
        let face = klee();
        let s = set(
            face,
            "神奈川県鎌倉市稲村ガ崎一丁目",
            10.0,
            100.0,
            0.0,
            50.0,
            12.0,
            2,
            1,
        );
        assert_eq!(s.columns, 2);
        assert!(s.cut);
        let s = set(face, "鎌倉市", 10.0, 100.0, 0.0, 50.0, 12.0, 2, 1);
        assert!(!s.cut);
        // The second column of a paragraph starts one character lower.
        let s = set(
            face,
            "神奈川県鎌倉市稲村",
            10.0,
            100.0,
            0.0,
            50.0,
            12.0,
            2,
            1,
        );
        let second = s
            .glyphs
            .iter()
            .filter(|g| g.1 < 80.0)
            .map(|g| g.2)
            .fold(f32::MAX, f32::min);
        assert!((second - 10.0).abs() < 0.01, "starts at {second}");
    }

    #[test]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    fn fr_130_strokes_are_thick_enough_to_print() {
        // 一 at the medium size, 4.2 mm, at 300 dpi: its stroke at least 0.25 mm.
        let face = klee();
        let px_per_mm = 300.0 / 25.4;
        let em = 4.2 * px_per_mm;
        let size = (em * 2.0) as u32;
        let mut canvas = RgbImage::from_raw(size, size, vec![255; (size * size * 3) as usize])
            .unwrap_or_else(|| RgbImage::new(1, 1));
        let s = set(face, "一", em, size as f32, 0.0, em * 1.5, em, 1, 0);
        draw(&mut canvas, face, &s, em, [0, 0, 0]);
        // The column's cell spans the right em: look down its middle.
        let column = (size as f32 - em / 2.0) as usize;
        // The ink down the column, its coverage summed (as FONTS.md measures a stroke).
        let ink: f32 = (0..size as usize)
            .map(|y| f32::from(255 - canvas.data[(y * size as usize + column) * 3]) / 255.0)
            .sum();
        assert!(ink / px_per_mm >= 0.25, "stroke {:.2} mm", ink / px_per_mm);
    }

    #[test]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    fn fr_130_a_character_fills_its_cell_and_stays_in_it() {
        // 鎌 drawn in a 100 px cell at the top of a 300 px canvas: its ink within the cell,
        // and most of the cell's height (the em box, not the line's taller ascent).
        let face = klee();
        let mut canvas = RgbImage::from_raw(100, 300, vec![255; 100 * 300 * 3])
            .unwrap_or_else(|| RgbImage::new(1, 1));
        let s = set(face, "鎌", 100.0, 100.0, 0.0, 300.0, 100.0, 1, 0);
        draw(&mut canvas, face, &s, 100.0, [0, 0, 0]);
        let inked: Vec<usize> = (0..300)
            .filter(|y| (0..100).any(|x| canvas.data[(y * 100 + x) * 3] < 128))
            .collect();
        let (first, last) = (inked[0], inked[inked.len() - 1]);
        assert!(last < 100, "ink reaches row {last}, below the cell");
        assert!(last - first > 80, "ink {first}..{last}");
    }
}
