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

    /// The glyph for `c` and whether it comes from the Latin face: every Latin letter (a
    /// Vietnamese name, a message in English) is set in the handwriting face, Patrick Hand, so a
    /// word never mixes two hands (owner, 2026-11-15); what the face lacks otherwise too.
    fn pick(&self, c: char) -> (GlyphId, bool) {
        if c.is_whitespace() {
            return (self.glyph(c), false);
        }
        let own = self.font.glyph_id(c).0 != 0;
        match latin() {
            Some(l) if (is_latin(c) || !own) && l.glyph_id(c).0 != 0 => (l.glyph_id(c), true),
            _ => (self.glyph(c), false),
        }
    }

    /// `c`'s advance at an em of `em` pixels, in whichever face sets it.
    pub fn advance(&self, c: char, em: f32) -> f32 {
        let (id, from_latin) = self.pick(c);
        match (from_latin, latin()) {
            (true, Some(l)) => l.as_scaled(em_scale(l, em)).h_advance(id),
            _ => self
                .font
                .as_scaled(self.scale(em))
                .h_advance(self.font.glyph_id(c)),
        }
    }

    /// Whether this face or the Latin one can set `c`.
    pub fn has(&self, c: char) -> bool {
        self.font.glyph_id(c).0 != 0 || latin().is_some_and(|l| l.glyph_id(c).0 != 0)
    }
}

/// Patrick Hand, the postcard's handwriting face (FR-076: every European letter and
/// Vietnamese), for the Latin letters on a Japanese or Korean card (owner, 2026-11-15).
fn latin() -> Option<&'static FontArc> {
    crate::postcard::handwriting()
}

/// A Latin letter, digit or mark as typed (ASCII and the Latin blocks, Vietnamese included);
/// never a full-width form, which belongs to the Japanese text.
fn is_latin(c: char) -> bool {
    c.is_ascii_graphic()
        || matches!(c, '\u{00C0}'..='\u{024F}' | '\u{0300}'..='\u{036F}' | '\u{1E00}'..='\u{1EFF}')
}

/// The scale that makes one em of `font` `em` pixels (`PxScale` is the line's height).
fn em_scale(font: &FontArc, em: f32) -> PxScale {
    let per_em = font.units_per_em().unwrap_or(1000.0);
    PxScale::from(em * font.height_unscaled() / per_em)
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
    /// Each glyph, the top-left of its em cell in pixels, and whether it is the Latin face's.
    pub glyphs: Vec<(GlyphId, f32, f32, bool)>,
    /// How many columns it took.
    pub columns: usize,
    /// Some of the text did not fit and is left out.
    pub cut: bool,
}

/// Lays `text` out top to bottom from (`right`, `top`): `em` pixels a character, columns
/// `step` pixels apart, at most `height` pixels tall and `max_columns` wide; every column after
/// the first starts `indent` characters lower (a Japanese address's second line).
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
    let mut glyphs = Vec::new();
    let mut n = 0;
    let mut cut = false;
    let mut first = true;
    for (p, paragraph) in text.split('\n').enumerate() {
        // A column after the first is shorter by its indent; the first paragraph's first
        // column has the full height.
        let cols = if p == 0 {
            let full = columns(paragraph, capacity);
            match full.split_first() {
                Some((head, rest)) if !rest.is_empty() => {
                    let tail: String = rest.iter().flatten().collect();
                    std::iter::once(head.clone())
                        .chain(columns(&tail, capacity.saturating_sub(indent).max(2)))
                        .collect()
                }
                _ => full,
            }
        } else {
            columns(paragraph, capacity.saturating_sub(indent).max(2))
        };
        for column in cols {
            if n >= max_columns {
                cut |= !column.is_empty();
                continue;
            }
            let offset = if first { 0 } else { indent };
            let x = right - (n as f32 + 1.0) * step + (step - em) / 2.0;
            for (row, c) in column.iter().enumerate() {
                let (id, from_latin) = face.pick(*c);
                // A narrow glyph (a digit, a Latin letter) is centred in its cell.
                let advance = face.advance(*c, em);
                let left = x + (em - advance.min(em)) / 2.0;
                glyphs.push((id, left, top + (row + offset) as f32 * em, from_latin));
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

/// Lays `text` out across one line from (`left`, `top`), each glyph at its own advance plus
/// `tracking` em (for 「郵便はがき」 and a code typed in full-width digits); its width in pixels.
pub fn across(
    face: &VerticalFace,
    text: &str,
    em: f32,
    left: f32,
    top: f32,
    tracking: f32,
) -> (Set, f32) {
    let mut x = left;
    let mut glyphs = Vec::new();
    for c in text.chars() {
        let (_, from_latin) = face.pick(c);
        // Across, a character keeps its horizontal form; only the face may change.
        let id = if from_latin {
            face.pick(c).0
        } else {
            face.font.glyph_id(c)
        };
        glyphs.push((id, x, top, from_latin));
        x += face.advance(c, em) + tracking * em;
    }
    let width = (x - left - tracking * em).max(0.0);
    (
        Set {
            glyphs,
            columns: 1,
            cut: false,
        },
        width,
    )
}

/// `text` broken into lines no wider than `width` pixels at `em`, the lines after the first
/// `indent` pixels narrower: between words for Latin letters (never inside one, unless a word
/// alone is wider), between characters for Japanese, by 禁則 (a closing mark stays with the
/// character before it, an opening one with the one after).
pub fn wrap_across(
    face: &VerticalFace,
    text: &str,
    em: f32,
    width: f32,
    indent: f32,
) -> Vec<String> {
    let cjk = |c: char| u32::from(c) >= 0x2E80;
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let chars: Vec<char> = paragraph.chars().collect();
        let mut start = 0;
        while start < chars.len() {
            let room = if start == 0 { width } else { width - indent };
            let mut x = 0.0;
            let mut end = start;
            // The last place a line may end, past index `brk`.
            let mut brk = None;
            while end < chars.len() {
                let c = chars[end];
                let w = face.advance(c, em);
                if x + w > room && end > start {
                    break;
                }
                x += w;
                end += 1;
                let next = chars.get(end).copied();
                let may_break = c == ' '
                    || next.is_some_and(|n| {
                        (cjk(c) || cjk(n))
                            && !NO_START.contains(n)
                            && !NO_END.contains(c)
                            && n != ' '
                    });
                if may_break {
                    brk = Some(end);
                }
            }
            if end < chars.len()
                && let Some(b) = brk.filter(|b| *b > start)
            {
                end = b;
            }
            let line: String = chars[start..end]
                .iter()
                .collect::<String>()
                .trim_end()
                .to_owned();
            lines.push(line);
            start = end;
            while chars.get(start) == Some(&' ') {
                start += 1;
            }
        }
        if chars.is_empty() {
            lines.push(String::new());
        }
    }
    lines
}

/// Draws a set in `color`.
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub fn draw(canvas: &mut RgbImage, face: &VerticalFace, set: &Set, em: f32, color: [u8; 3]) {
    let scale = face.scale(em);
    let baseline = em * face.em_ascent;
    for &(id, left, top, from_latin) in &set.glyphs {
        let (font, scale) = match (from_latin, latin()) {
            (true, Some(l)) => (l, em_scale(l, em)),
            _ => (&face.font, scale),
        };
        let glyph = id.with_scale_and_position(scale, point(left, top + baseline));
        let Some(outline) = font.outline_glyph(glyph) else {
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

    #[test]
    fn fr_130_a_vietnamese_name_prints_whole() {
        // Every Latin letter comes from the handwriting face, so a word keeps one hand.
        let face = klee();
        for c in "Trương Minh Hồng, Đà Nẵng"
            .chars()
            .filter(|c| !c.is_whitespace())
        {
            assert!(face.has(c), "{c:?}");
            let (id, _) = face.pick(c);
            assert_ne!(id.0, 0, "{c:?} has a glyph");
        }
        assert!(face.pick('ư').1, "ư is Patrick Hand's");
        assert!(face.pick('c').1, "and so is c, beside it");
        assert!(!face.pick('鎌').1, "a kanji stays Klee One's");
    }

    #[test]
    fn fr_130_latin_lines_break_between_words() {
        let face = klee();
        let em = 10.0;
        let text = "Matsudo, Matsudo-shi, Chiba-ken";
        let width = face.advance('M', em) * 12.0;
        let lines = wrap_across(face, text, em, width, 0.0);
        assert!(lines.len() >= 2);
        // Never inside a word.
        for line in &lines {
            for word in line.split(' ') {
                assert!(text.split(' ').any(|w| w == word), "{word:?} was cut");
            }
        }
        // Japanese breaks between characters, a closing mark kept with the one before.
        let jp = wrap_across(face, "神奈川県鎌倉市稲村ガ崎。", em, em * 11.0, 0.0);
        assert!(jp.iter().all(|l| !l.starts_with('。')));
    }

    #[test]
    fn fr_130_a_marked_latin_word_is_set_in_one_hand() {
        // Klee One has plain ASCII but not ễ: per character, the word mixed two faces.
        let text = "Nguyễn Hoa";
        let (set, _) = across(klee(), text, 10.0, 0.0, 0.0, 0.0);
        let letters: Vec<bool> = text
            .chars()
            .zip(&set.glyphs)
            .filter(|(c, _)| !c.is_whitespace())
            .map(|(_, g)| g.3)
            .collect();
        assert!(letters.iter().all(|&latin| latin), "{letters:?}");
    }
}
