// SPDX-License-Identifier: GPL-3.0-only
//! The はがき's address side (FR-130): Japan Post's layout on the writing half of a Japanese
//! card, its photo on the other half as every postcard of the app (one side, PRD FR-130).
//! Upright, the text runs top to bottom (`vertical`): the stamp at the top left, 「郵便はがき」
//! centred at the top, the seven vermilion code boxes at the top right (or the code typed, in
//! full-width digits, where they would be), the recipient's address down the right, the name
//! down the middle, the largest, with 様, the sender on the left when its switch is on.
//! Positions are the chosen style board's (boards A, B and Landscape), within Japan Post's figures.

use crate::addassist::{Address, Country};
use crate::image::RgbImage;
use crate::library::PostcardText;
use crate::postcard::JAPAN_BOXES;
use crate::settings::{BorderColor, PostcardSettings};
use crate::sheet::RectMm;
use crate::vertical::{self, VerticalFace};

// Every position below is the chosen style board's (PRD FR-130): artboards A and B (upright)
// and Landscape, 4 px a millimetre, measured in Chromium (LESSONS 2026-11-15: a layout that has
// a board takes the board's numbers), except where Japan Post's 内国郵便約款 and 別記1 ask
// otherwise, which win (the Japan Post review, 2026-11-15). Ems scale with the text size; the
// board's is 17 px.

/// The stamp box, 6 mm from the top and left edges (board A: 24 px), 20 × 25 mm (FAQ 597:
/// within 7.0 × 3.5 cm).
pub const STAMP_MM: (f32, f32) = (20.0, 25.0);
const STAMP_EDGE_MM: f32 = 6.0;
/// 切手's em in its box (15 px).
const STAMP_WORD_MM: f32 = 3.75;
/// The code boxes' top and right offsets (別記1: 12 and 8 mm).
pub const BOXES_TOP_MM: f32 = 12.0;
pub const BOXES_RIGHT_MM: f32 = 8.0;
/// 「郵便はがき」: em 3.25 mm (13 px), 1.5 mm between characters (6 px), centred at the top.
const HEADING_MM: f32 = 3.25;
const HEADING_TRACK: f32 = 1.5 / 3.25;
const HEADING_TOP_MM: f32 = 4.2;
/// A typed code's digits in the boxes: their em 62 % of a box's long side, the digits'
/// middle 0.52 em under the em's top (Klee One).
const DIGIT_SCALE: f32 = 0.62;
const CODE_MIDDLE: f32 = 0.52;
/// Japan Post's postmark zone, the top left 35 mm wide and 70 mm tall upright (70 × 35 mm at
/// the top right on a landscape card): nothing but the stamp box in it (別記1 備考3).
pub const POSTMARK_MM: (f32, f32) = (35.0, 70.0);
/// The address: its first column's top 32 mm, 11 mm under the boxes' frames (別記1 (1)(ｵ) asks
/// 10; the Japan Post review, 2026-11-15), its em 16/17 of the text's (board B: 16 px), its ink
/// 15 mm or more from the right edge, inside 別記1's address area (例1: 10, 15 and 10 mm),
/// columns 1.5 em apart (line-height 1.5), each after the first one character lower.
const ADDRESS_TOP_MM: f32 = 32.0;
const ADDRESS_SCALE: f32 = 16.0 / 17.0;
const ADDRESS_RIGHT_MM: f32 = 14.5;
const COLUMN_STEP: f32 = 1.5;
/// The name: its column centred 51.4 mm from the left, starting 1 mm above the address, at
/// most 30/17 of the address's em (A: 30 px), smaller to end at the half's foot, 0.1 em
/// between characters (3 px).
const NAME_CENTRE_MM: f32 = 51.4;
const NAME_SCALE: f32 = 30.0 / 17.0;
const NAME_TRACK: f32 = 0.1;
/// The sender: right of the postmark zone, top 34 mm (board A: 136 px), 11/17 of the
/// address's em; its name a column of its own on the left, 13/17, 60/11 of the sender's em
/// lower; in Latin letters lines 22 mm wide.
const SENDER_LATIN_WIDTH_MM: f32 = 22.0;
const SENDER_TOP_MM: f32 = 34.0;
const SENDER_SCALE: f32 = 11.0 / 17.0;
const SENDER_NAME_SCALE: f32 = 13.0 / 17.0;
const SENDER_NAME_DROP: f32 = 60.0 / 11.0;
/// The message under a bordered photo: 14/17 em, 2.6 mm under it, 1 mm in (board A).
const MESSAGE_SCALE: f32 = 14.0 / 17.0;
const MESSAGE_GAP_MM: f32 = 2.6;
const MESSAGE_IN_MM: f32 = 1.0;

/// Landscape: the stamp box 6 mm from the top and right, 20 × 24 mm, 切手 across.
const L_STAMP_MM: (f32, f32) = (20.0, 24.0);
/// The code column: its right 7.5 mm from the edge (board: 30 px), its top 37 mm, 2 mm under
/// the postmark zone for a kiosk's shift (the Japan Post review, 2026-11-15).
const L_BOXES_RIGHT_MM: f32 = 7.5;
const L_BOXES_TOP_MM: f32 = 37.0;
/// The text keeps 10 mm from the code column (別記1 例2) and above 85 mm (its 15 mm foot).
const L_BOXES_CLEAR_MM: f32 = 10.0;
const L_TEXT_BOTTOM_MM: f32 = 85.0;
/// 「郵便はがき」 down the edge right of the boxes: em 2.75 mm (11 px), 1 mm between (4 px),
/// its column centred 4 mm from the edge and on the card's height (§22(6): 右側部 中央).
const L_HEADING_MM: f32 = 2.75;
const L_HEADING_PITCH_MM: f32 = 3.75;
const L_HEADING_CENTRE_MM: f32 = 4.0;
/// 50 mm less half the column (4 × 3.75 + 2.75 mm).
const L_HEADING_TOP_MM: f32 = 41.125;
/// The text from 4 mm into the half (312 px). The address 15/17 em, lines 1.6 em apart, its
/// first em at 36.1 mm (35 mm and the line's half-leading), each line after the first one em
/// in; the name 28/17 em at 51.6 mm; the sender's lines from 73.3 mm, 50 mm wide (200 px),
/// its building two ems in, its name 13/17 em on the right.
const L_TEXT_LEFT_MM: f32 = 4.0;
const L_ADDRESS_SCALE: f32 = 15.0 / 17.0;
const L_ADDRESS_TOP_MM: f32 = 36.1;
const L_LINE: f32 = 1.6;
const L_NAME_SCALE: f32 = 28.0 / 17.0;
const L_NAME_TOP_MM: f32 = 51.6;
const L_SENDER_TOP_MM: f32 = 73.3;
const L_SENDER_WIDTH_MM: f32 = 50.0;

/// What a はがき's writing half draws.
pub struct HagakiSpec<'a> {
    /// The writing half, in sheet millimetres.
    pub card: RectMm,
    pub settings: &'a PostcardSettings,
    pub text: Option<&'a PostcardText>,
    pub paper: BorderColor,
    /// The message for the card's 通信欄, when it does not fit the line under the photo.
    pub message: Option<&'a str>,
}

/// The stamp box of a はがき whose writing half is `card`.
pub fn stamp_box(card: RectMm) -> RectMm {
    RectMm {
        x: card.x + STAMP_EDGE_MM,
        y: card.y + STAMP_EDGE_MM,
        width: STAMP_MM.0,
        height: STAMP_MM.1,
    }
}

/// The code boxes' top-left.
fn boxes_origin(card: RectMm) -> (f32, f32) {
    // 12 and 8 mm to the boxes' insides (別記1).
    (
        card.x + card.width - BOXES_RIGHT_MM - JAPAN_BOXES.span_mm(),
        card.y + BOXES_TOP_MM,
    )
}

/// Half-width digits and letters as their full-width forms, which stand upright in a column
/// and read as one character each, and the hyphen as ー, which turns in a column. Other marks
/// stay as typed: JIS X 0208, the face's cut, lacks some full-width marks (－ U+FF0D).
fn full_width(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '-' => 'ー',
            '0'..='9' | 'A'..='Z' | 'a'..='z' => char::from_u32(c as u32 + 0xFEE0).unwrap_or(c),
            ' ' => '　',
            _ => c,
        })
        .collect()
}

/// `2480024` as Japan Post asks a printed code: `２４８−００２４` (full-width digits, the
/// JIS minus as the hyphen).
fn printed_code(digits: &str) -> Option<String> {
    (digits.len() == 7 && digits.chars().all(|c| c.is_ascii_digit())).then(|| {
        format!(
            "{}\u{2212}{}",
            full_width(&digits[..3]),
            full_width(&digits[3..])
        )
    })
}

/// Arabic figures as kanji numerals, as an address is written top to bottom (board A: 5-27 →
/// 五ー二十七, 3号室 → 三号室): up to 99 with 十, longer numbers figure by figure (一〇一).
fn kanji_numerals(text: &str) -> String {
    fn flush(run: &mut Vec<usize>, out: &mut String) {
        const DIGITS: [char; 10] = ['〇', '一', '二', '三', '四', '五', '六', '七', '八', '九'];
        let d = |i: usize| DIGITS.get(i).copied().unwrap_or('〇');
        match run.as_slice() {
            [] => {}
            [one] => out.push(d(*one)),
            [tens, units] if *tens > 0 => {
                if *tens > 1 {
                    out.push(d(*tens));
                }
                out.push('十');
                if *units > 0 {
                    out.push(d(*units));
                }
            }
            all => out.extend(all.iter().map(|&i| d(i))),
        }
        run.clear();
    }
    let mut out = String::new();
    let mut run = Vec::new();
    for c in text.chars() {
        if let Some(v) = c.to_digit(10).filter(|_| c.is_ascii_digit()) {
            run.push(v as usize);
        } else {
            flush(&mut run, &mut out);
            out.push(c);
        }
    }
    flush(&mut run, &mut out);
    out
}

/// The typed line's number and what follows it (`5-27 ロイヤルアパート3号室`): the number goes
/// with the town, the rest (a building) on a line of its own (boards A and Landscape).
fn split_line(line: &str) -> (&str, &str) {
    let line = line.trim();
    line.split_once(char::is_whitespace)
        .map_or((line, ""), |(number, rest)| (number, rest.trim()))
}

/// An address top to bottom, as board A: the region and city, the town and number, the
/// building, a column each, in kanji numerals.
fn column_lines(a: &Address) -> String {
    let (number, rest) = split_line(&a.line);
    let columns = [
        format!("{}{}", a.region, a.city),
        format!("{}{number}", a.town),
        rest.to_owned(),
    ];
    let text: Vec<&str> = columns
        .iter()
        .map(|c| c.trim())
        .filter(|c| !c.is_empty())
        .collect();
    full_width(&kanji_numerals(&text.join("\n")))
}

/// A line across with a space between Japanese and Arabic figures (the landscape board:
/// ロイヤルアパート 3 号室).
fn spaced(text: &str) -> String {
    let cjk = |c: char| u32::from(c) >= 0x2E80;
    let mut out = String::new();
    let mut last: Option<char> = None;
    for c in text.chars() {
        if let Some(l) = last
            && ((cjk(l) && c.is_ascii_digit()) || (l.is_ascii_digit() && cjk(c)))
        {
            out.push(' ');
        }
        out.push(c);
        last = Some(c);
    }
    out
}

/// One column centred on `centre`, `em` a character drawn `pitch` from the next (CSS's
/// letter-spacing): `vertical::set` steps by its em, so it is given the pitch.
fn tracked_column(
    face: &VerticalFace,
    text: &str,
    (em, pitch): (f32, f32),
    centre: f32,
    top: f32,
    height: f32,
) -> vertical::Set {
    vertical::set(
        face,
        text,
        pitch,
        centre + pitch - em / 2.0,
        top,
        height,
        pitch,
        1,
        0,
    )
}

/// An address in Latin letters (`1-2-3, Inamuragasaki, Kamakura-shi, Kanagawa-ken`): its places
/// from `AddAssist`'s data when the code is listed, else as typed.
fn latin_lines(a: &Address) -> Vec<String> {
    let found = crate::addassist::lookup(Country::Jp, &a.code)
        .ok()
        .and_then(|f| {
            f.into_iter()
                .find(|f| f.town == a.town || a.town.is_empty())
        });
    let places = found.map_or_else(
        || format!("{}{}{}", a.region, a.city, a.town),
        |f| {
            [f.town_latin, f.city_latin, f.region_latin]
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(", ")
        },
    );
    let mut lines: Vec<String> = a
        .line
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect();
    lines.push(places);
    lines
}

/// The recipient of this card: its own record, never the European card's free address
/// (owner, 2026-11-15: each card its own record).
fn recipient(text: Option<&PostcardText>) -> Option<Address> {
    text?.recipient(Country::Jp).cloned()
}

fn ink(settings: &PostcardSettings, paper: BorderColor) -> [u8; 3] {
    settings.ink.rgb(paper)
}

/// Draws a はがき's upright writing half onto `canvas`, `density` pixels per millimetre, as
/// boards A and B within Japan Post's rules; whether a column was left out (the warning).
#[allow(clippy::too_many_lines)]
pub fn draw_upright(
    canvas: &mut RgbImage,
    density: f32,
    spec: &HagakiSpec<'_>,
) -> crate::postcard::Fit {
    let Some(face) = vertical::KLEE.as_ref() else {
        return crate::postcard::Fit::default();
    };
    let mut cut = false;
    let card = spec.card;
    let settings = spec.settings;
    let (outline, _) = crate::postcard::inks(spec.paper);
    let text_ink = ink(settings, spec.paper);
    let mm = |v: f32| v * density;
    let em = crate::settings::postcard_text_mm(settings.text_size);

    // The stamp box, 切手 down its middle.
    let stamp = stamp_box(card);
    crate::postcard::draw_dashed_box(canvas, density, stamp, outline);
    let word = vertical::set(
        face,
        "切手",
        mm(STAMP_WORD_MM),
        mm(stamp.x + stamp.width / 2.0 + STAMP_WORD_MM / 2.0),
        mm(stamp.y + stamp.height / 2.0 - STAMP_WORD_MM),
        mm(STAMP_WORD_MM * 2.5),
        mm(STAMP_WORD_MM),
        1,
        0,
    );
    vertical::draw(canvas, face, &word, mm(STAMP_WORD_MM), outline);

    // 「郵便はがき」, which makes a card printed at home a postcard (内国郵便約款 §22(6)).
    if settings.postal_use {
        let (_, width) =
            vertical::across(face, "郵便はがき", mm(HEADING_MM), 0.0, 0.0, HEADING_TRACK);
        let left = mm(card.x + card.width / 2.0) - width / 2.0;
        let (heading, _) = vertical::across(
            face,
            "郵便はがき",
            mm(HEADING_MM),
            left,
            mm(card.y + HEADING_TOP_MM),
            HEADING_TRACK,
        );
        vertical::draw(canvas, face, &heading, mm(HEADING_MM), text_ink);
    }

    let to = recipient(spec.text);
    // The seven boxes always (別記1's figure); a typed code's digits in them, black, one a box.
    let (bx, by) = boxes_origin(card);
    crate::postcard::draw_code_boxes(canvas, density, bx, by, &JAPAN_BOXES);
    if let Some(a) = to.as_ref().filter(|a| printed_code(&a.code).is_some()) {
        draw_code_digits(
            canvas,
            face,
            density,
            &JAPAN_BOXES.boxes(bx, by),
            &a.code,
            text_ink,
            false,
        );
    }

    // The address 10 mm or more under the boxes (別記1 (1)(ｵ)), down the right; the name down
    // the middle; both to the half's foot.
    let top = card.y + ADDRESS_TOP_MM;
    let bottom = card.y + card.height;
    let right = card.x + card.width - ADDRESS_RIGHT_MM;
    let em_address = em * ADDRESS_SCALE;

    // The sender right of the postmark's 35 × 70 mm (別記1 備考3; owner, 2026-11-15), when its
    // switch is on: its name a column on the left, a little lower, its address in columns right
    // of it, its code across above them. What it takes moves the recipient's name right.
    let mut sender_right = card.x + POSTMARK_MM.0;
    let small = em * SENDER_SCALE;
    // 3 mm clear of the zone, for a kiosk's shift (the Japan Post review).
    let left = card.x + POSTMARK_MM.0 + 3.0;
    if let Some(from) = sender(settings, Country::Jp) {
        let from_top = card.y + SENDER_TOP_MM;
        if from.latin {
            let mut lines = latin_lines(&from);
            if !from.name.trim().is_empty() {
                lines.push(from.name.trim().to_owned());
            }
            draw_lines_across(
                canvas,
                face,
                &lines.join("\n"),
                mm(small),
                mm(left),
                mm(from_top),
                mm(SENDER_LATIN_WIDTH_MM),
                mm(small * 1.35),
                text_ink,
            );
            sender_right = left + SENDER_LATIN_WIDTH_MM;
        } else {
            let step = small * COLUMN_STEP;
            let name = from.name.trim();
            let name_em = em * SENDER_NAME_SCALE;
            let name_step = if name.is_empty() {
                0.0
            } else {
                name_em * COLUMN_STEP
            };
            let text = column_lines(&from);
            let height = bottom - from_top;
            let set_from = |right: f32| {
                vertical::set(
                    face,
                    &text,
                    mm(small),
                    mm(right),
                    mm(from_top),
                    mm(height),
                    mm(step),
                    3,
                    1,
                )
            };
            #[allow(clippy::cast_precision_loss)]
            let block_right = left + name_step + set_from(0.0).columns as f32 * step;
            let block = set_from(block_right);
            cut |= block.cut;
            vertical::draw(canvas, face, &block, mm(small), text_ink);
            if !name.is_empty() {
                let drop = small * SENDER_NAME_DROP;
                let set = vertical::set(
                    face,
                    &full_width(name),
                    mm(name_em),
                    mm(left + name_step),
                    mm(from_top + drop),
                    mm(bottom - from_top - drop),
                    mm(name_step),
                    1,
                    0,
                );
                cut |= set.cut;
                vertical::draw(canvas, face, &set, mm(name_em), text_ink);
            }
            sender_right = block_right;
        }
        if printed_code(&from.code).is_some() {
            let code = format!("{}-{}", &from.code[..3], &from.code[3..]);
            let (set, _) = vertical::across(
                face,
                &code,
                mm(small),
                mm(left),
                mm(from_top - small * 1.5),
                0.0,
            );
            vertical::draw(canvas, face, &set, mm(small), text_ink);
        }
    }

    // What the recipient takes across the half, for the 通信欄 beside it.
    let mut taken: Vec<(f32, f32)> = Vec::new();
    if let Some(to) = &to {
        if to.latin {
            let left = (sender_right + 2.0).max(card.x + card.width * 0.38);
            cut |= draw_latin(canvas, face, density, spec, to, left, top, text_ink);
            taken.push((left, right));
        } else {
            let set = vertical::set(
                face,
                &column_lines(to),
                mm(em_address),
                mm(right),
                mm(top),
                mm(bottom - top),
                mm(em_address * COLUMN_STEP),
                3,
                1,
            );
            cut |= set.cut;
            #[allow(clippy::cast_precision_loss)]
            let address_left = right - set.columns as f32 * em_address * COLUMN_STEP;
            taken.push((address_left, right));
            vertical::draw(canvas, face, &set, mm(em_address), text_ink);
            let name = to.name.trim();
            if !name.is_empty() {
                let label = format!("{}　様", full_width(name));
                let name_top = top - 1.0;
                #[allow(clippy::cast_precision_loss)]
                let chars = label.chars().count().max(1) as f32;
                let name_em =
                    (em * NAME_SCALE).min((bottom - name_top) / chars / (1.0 + NAME_TRACK));
                // The board's middle, or right of the sender when it reaches that far.
                let centre = (card.x + NAME_CENTRE_MM).max(sender_right + 2.0 + name_em / 2.0);
                let set = tracked_column(
                    face,
                    &label,
                    (mm(name_em), mm(name_em * (1.0 + NAME_TRACK))),
                    mm(centre),
                    mm(name_top),
                    mm(bottom - name_top),
                );
                cut |= set.cut;
                taken.push((centre - name_em / 2.0, centre + name_em / 2.0));
                vertical::draw(canvas, face, &set, mm(name_em), text_ink);
            }
        }
    }

    // The 通信欄 in the widest free stretch between the sender (or the postmark zone) and the
    // recipient, 5 mm clear of each, its columns top to bottom (§23(4) ただし書).
    taken.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut from = sender_right.max(card.x + POSTMARK_MM.0) + BOX_CLEAR_MM;
    let mut widest = (0.0, 0.0);
    for &(l, r) in taken.iter().chain(std::iter::once(&(right, right))) {
        if l - BOX_CLEAR_MM - from > widest.1 - widest.0 {
            widest = (from, l - BOX_CLEAR_MM);
        }
        from = from.max(r + BOX_CLEAR_MM);
    }
    let rect = RectMm {
        x: widest.0,
        y: card.y + ADDRESS_TOP_MM - 1.0,
        width: (widest.1 - widest.0).max(0.0),
        height: card.y + card.height - 2.0 - (card.y + ADDRESS_TOP_MM - 1.0),
    };
    // Top to bottom for Japanese; a message with Latin letters (Vietnamese, English) reads
    // across, as a column of single letters would not.
    let latin = spec.message.is_some_and(|m| {
        m.chars()
            .any(|c| c.is_ascii_alphabetic() || ('\u{00C0}'..='\u{1EFF}').contains(&c))
    });
    let message_cut = message_box(canvas, face, density, rect, spec, !latin);
    crate::postcard::Fit {
        message_cut,
        address_cut: cut,
    }
}

/// The 通信欄: 5 mm clear of the address, the name and the sender (Japan Post's "clearly
/// apart", §23(4)); its text 14/17 of the address's em, a row 1.6 of that.
const BOX_CLEAR_MM: f32 = 5.0;
const BOX_STEP: f32 = 1.6;

/// Draws the 通信欄 in `rect` and the message in it, across or top to bottom; whether some of
/// the message was left out.
fn message_box(
    canvas: &mut RgbImage,
    face: &VerticalFace,
    density: f32,
    rect: RectMm,
    spec: &HagakiSpec<'_>,
    vertical: bool,
) -> bool {
    let em = crate::settings::postcard_text_mm(spec.settings.text_size) * MESSAGE_SCALE;
    let step = em * BOX_STEP;
    let rows = crate::postcard::draw_message_box(canvas, density, rect, step, vertical, spec.paper);
    let Some(message) = spec.message.map(str::trim).filter(|m| !m.is_empty()) else {
        return false;
    };
    let Some(first) = rows.first() else {
        return true;
    };
    // Rows set closer for the room take a smaller text.
    let row = if vertical { first.width } else { first.height };
    let em = em.min(row / BOX_STEP);
    let step = row;
    let mm = |v: f32| v * density;
    let ink = ink(spec.settings, spec.paper);
    if vertical {
        let set = vertical::set(
            face,
            message,
            mm(em),
            mm(first.x + first.width),
            mm(first.y + 0.5),
            mm(first.height - 0.5),
            mm(step),
            rows.len(),
            0,
        );
        vertical::draw(canvas, face, &set, mm(em), ink);
        return set.cut;
    }
    let lines: Vec<String> = message
        .lines()
        .flat_map(|p| vertical::wrap_across(face, p, mm(em), mm(first.width - 1.0), 0.0))
        .collect();
    for (row, line) in rows.iter().zip(&lines) {
        let (set, _) = vertical::across(
            face,
            line,
            mm(em),
            mm(row.x + 0.5),
            mm(row.y + row.height - em * 1.2),
            0.0,
        );
        vertical::draw(canvas, face, &set, mm(em), ink);
    }
    lines.len() > rows.len()
}

/// Whether `message` fits the one line under the photo (`band`, from the photo's foot): one
/// line as typed, no wider than the band.
pub fn message_fits_band(band: RectMm, message: &str, settings: &PostcardSettings) -> bool {
    let Some(face) = vertical::KLEE.as_ref() else {
        return false;
    };
    let mut lines = message.lines().map(str::trim).filter(|l| !l.is_empty());
    let (Some(line), None) = (lines.next(), lines.next()) else {
        return message.trim().is_empty();
    };
    // Measured at 10 px a millimetre: one would round the face too coarsely.
    let px = |mm: f32| mm * 10.0;
    let em = crate::settings::postcard_text_mm(settings.text_size) * MESSAGE_SCALE;
    let (_, w) = vertical::across(face, line, px(em), 0.0, 0.0, 0.0);
    w <= px(band.width - 2.0 * MESSAGE_IN_MM)
}

/// A typed code's digits, black, one centred in each box (別記1: the code written in the
/// boxes), clear of the frames; `turned` on a landscape card, each digit a quarter turn with its
/// top toward the right edge, as the boxes run.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn draw_code_digits(
    canvas: &mut RgbImage,
    face: &VerticalFace,
    density: f32,
    boxes: &[(RectMm, f32)],
    digits: &str,
    ink: [u8; 3],
    turned: bool,
) {
    for ((r, _), d) in boxes.iter().zip(digits.chars()) {
        let short = r.width.min(r.height);
        let long = r.width.max(r.height);
        let em = (long * DIGIT_SCALE).min(short * 1.1) * density;
        let text = d.to_string();
        let (_, w) = vertical::across(face, &text, em, 0.0, 0.0, 0.0);
        // The digit on its own small canvas, its ink centred, then set (or turned) in the box.
        let side = em.ceil() as u32 + 2;
        let Some(mut cell) = RgbImage::from_raw(side, side, vec![255; (side * side * 3) as usize])
        else {
            continue;
        };
        let (set, _) = vertical::across(
            face,
            &text,
            em,
            (side as f32 - w) / 2.0,
            side as f32 / 2.0 - em * CODE_MIDDLE,
            0.0,
        );
        vertical::draw(&mut cell, face, &set, em, ink);
        let (cx, cy) = (
            (r.x + r.width / 2.0) * density,
            (r.y + r.height / 2.0) * density,
        );
        let half = side as f32 / 2.0;
        for y in 0..side {
            for x in 0..side {
                let i = ((y * side + x) * 3) as usize;
                let Some(px) = cell.data.get(i..i + 3) else {
                    continue;
                };
                if px.iter().all(|&c| c == 255) {
                    continue;
                }
                // A quarter turn clockwise: the digit's top toward the right.
                let (dx, dy) = if turned {
                    (half - y as f32, x as f32 - half)
                } else {
                    (x as f32 - half, y as f32 - half)
                };
                let (tx, ty) = ((cx + dx) as i64, (cy + dy) as i64);
                if tx < 0
                    || ty < 0
                    || tx >= i64::from(canvas.width)
                    || ty >= i64::from(canvas.height)
                {
                    continue;
                }
                let j = ((ty as u32 * canvas.width + tx as u32) * 3) as usize;
                if let Some(dst) = canvas.data.get_mut(j..j + 3) {
                    for (d, s) in dst.iter_mut().zip(px) {
                        *d = (*d).min(*s);
                    }
                }
            }
        }
    }
}

/// This card's sender: its own country's record, when its switch is on (each card its own
/// record, LESSONS 2026-11-15).
fn sender(settings: &PostcardSettings, country: Country) -> Option<Address> {
    if !settings.sender_on {
        return None;
    }
    settings.sender(country).filter(|a| !a.is_blank()).cloned()
}

/// The stamp box of a landscape はがき whose writing half is `card`: its top right.
pub fn stamp_box_landscape(card: RectMm) -> RectMm {
    RectMm {
        x: card.x + card.width - STAMP_EDGE_MM - L_STAMP_MM.0,
        y: card.y + STAMP_EDGE_MM,
        width: L_STAMP_MM.0,
        height: L_STAMP_MM.1,
    }
}

/// The landscape はがき's code column: its top-left, along the right edge.
fn boxes_down_origin(card: RectMm) -> (f32, f32) {
    (
        // The insides' column; its frames end 7.5 mm from the edge.
        card.x + card.width - L_BOXES_RIGHT_MM - JAPAN_BOXES.box_mm.1 - 0.5,
        card.y + L_BOXES_TOP_MM,
    )
}

/// Draws lines across from `left`, `top`, each broken by 禁則 to `width`; every line after a
/// paragraph's first indented one character. Returns the next line's top.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::too_many_arguments
)]
fn draw_lines_across(
    canvas: &mut RgbImage,
    face: &VerticalFace,
    text: &str,
    em: f32,
    left: f32,
    top: f32,
    width: f32,
    step: f32,
    ink: [u8; 3],
) -> f32 {
    let mut y = top;
    for paragraph in text.split('\n') {
        // Each line measured: Latin breaks between words, Japanese between characters. A line
        // that would leave a character or two alone below (千葉県松戸市松 / 戸) is set a little
        // smaller to keep it whole, down to 80 %, else broken into even lines.
        let total: f32 = paragraph.chars().map(|c| face.advance(c, em)).sum();
        let mut lines = vertical::wrap_across(face, paragraph, em, width, em);
        let mut size = em;
        if lines.len() > 1 {
            let shrunk = em * width / total.max(1.0);
            if shrunk >= em * 0.8 {
                size = shrunk;
                lines = vec![paragraph.to_owned()];
            } else {
                #[allow(clippy::cast_precision_loss)]
                let even = total / lines.len() as f32 + em;
                lines = vertical::wrap_across(face, paragraph, em, even.min(width), em);
            }
        }
        for (i, line) in lines.iter().enumerate() {
            let indent = if i == 0 { 0.0 } else { em };
            let (set, _) = vertical::across(face, line, size, left + indent, y, 0.0);
            vertical::draw(canvas, face, &set, size, ink);
            y += step;
        }
    }
    y
}

/// Draws a はがき's landscape writing half (横書き), as the landscape board: the stamp at the
/// top right, the seven boxes down the right edge with 「郵便はがき」 right of them, the
/// address across in two lines, the name under it with 様, the sender lower down.
#[allow(clippy::too_many_lines)]
pub fn draw_landscape(
    canvas: &mut RgbImage,
    density: f32,
    spec: &HagakiSpec<'_>,
) -> crate::postcard::Fit {
    let Some(face) = vertical::KLEE.as_ref() else {
        return crate::postcard::Fit::default();
    };
    let card = spec.card;
    let settings = spec.settings;
    let (outline, _) = crate::postcard::inks(spec.paper);
    let text_ink = ink(settings, spec.paper);
    let mm = |v: f32| v * density;
    let em = crate::settings::postcard_text_mm(settings.text_size);

    let stamp = stamp_box_landscape(card);
    crate::postcard::draw_dashed_box(canvas, density, stamp, outline);
    let (_, word_w) = vertical::across(face, "切手", mm(STAMP_WORD_MM), 0.0, 0.0, 0.0);
    let (word, _) = vertical::across(
        face,
        "切手",
        mm(STAMP_WORD_MM),
        mm(stamp.x + stamp.width / 2.0) - word_w / 2.0,
        mm(stamp.y + stamp.height / 2.0 - STAMP_WORD_MM / 2.0),
        0.0,
    );
    vertical::draw(canvas, face, &word, mm(STAMP_WORD_MM), outline);

    let to = recipient(spec.text);
    let (bx, by) = boxes_down_origin(card);
    // The seven boxes turned down the right edge (別記1 例2), a typed code's digits in them.
    crate::postcard::draw_code_boxes_down(canvas, density, bx, by, &JAPAN_BOXES);
    if let Some(a) = to.as_ref().filter(|a| printed_code(&a.code).is_some()) {
        let boxes = JAPAN_BOXES.boxes_down(bx, by);
        draw_code_digits(canvas, face, density, &boxes, &a.code, text_ink, true);
    }
    if settings.postal_use {
        let set = tracked_column(
            face,
            "郵便はがき",
            (mm(L_HEADING_MM), mm(L_HEADING_PITCH_MM)),
            mm(card.x + card.width - L_HEADING_CENTRE_MM),
            mm(card.y + L_HEADING_TOP_MM),
            mm(L_HEADING_PITCH_MM * 6.0),
        );
        vertical::draw(canvas, face, &set, mm(L_HEADING_MM), text_ink);
    }

    // The text: from the half's left to the boxes.
    let left = card.x + L_TEXT_LEFT_MM;
    // From the column's frames, half a millimetre outside its insides.
    let width = bx - 0.5 - L_BOXES_CLEAR_MM - left;
    let bottom = card.y + L_TEXT_BOTTOM_MM;
    let a_em = em * L_ADDRESS_SCALE;
    // Where the recipient ends, for the 通信欄 under it: at least the name's room.
    let mut text_bottom = card.y + L_NAME_TOP_MM + em * L_NAME_SCALE * 1.2;
    if let Some(to) = &to {
        let draw_at = |canvas: &mut RgbImage, text: &str, indent: f32, top: f32| {
            draw_lines_across(
                canvas,
                face,
                text,
                mm(a_em),
                mm(left + indent),
                mm(top),
                mm(width - indent),
                mm(a_em * L_LINE),
                text_ink,
            ) / density
        };
        let mut y = card.y + L_ADDRESS_TOP_MM;
        if to.latin {
            y = draw_at(canvas, &latin_lines(to).join("\n"), 0.0, y);
        } else {
            // The region and city, then one em in the town and number, the building.
            let (number, rest) = split_line(&to.line);
            let first = format!("{}{}", to.region, to.city);
            let town: Vec<&str> = [to.town.trim(), number]
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect();
            let after: Vec<String> = [town.join(" "), spaced(rest)]
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect();
            let indent = if first.trim().is_empty() {
                0.0
            } else {
                y = draw_at(canvas, first.trim(), 0.0, y);
                a_em
            };
            if !after.is_empty() {
                y = draw_at(canvas, &after.join("\n"), indent, y);
            }
        }
        let name = to.name.trim();
        if !name.is_empty() {
            let label = if to.latin {
                name.to_owned()
            } else {
                format!("{name}　様")
            };
            let mut name_em = em * L_NAME_SCALE;
            let (_, w) = vertical::across(face, &label, mm(name_em), 0.0, 0.0, NAME_TRACK);
            if w > mm(width) {
                name_em *= mm(width) / w;
            }
            let name_top = (card.y + L_NAME_TOP_MM).max(y + a_em * 0.3);
            let (set, _) = vertical::across(
                face,
                &label,
                mm(name_em),
                mm(left),
                mm(name_top),
                NAME_TRACK,
            );
            vertical::draw(canvas, face, &set, mm(name_em), text_ink);
            text_bottom = text_bottom.max(name_top + name_em * 1.2);
        }
        text_bottom = text_bottom.max(y);
    }

    // The sender's lines, each with its em, its indent in ems and whether it sits on the
    // right: the code in Arabic figures, the address, the building two ems in, the name.
    let s_em = em * SENDER_SCALE;
    let mut lines: Vec<(String, f32, f32, bool)> = Vec::new();
    if let Some(from) = sender(settings, Country::Jp) {
        if printed_code(&from.code).is_some() {
            lines.push((
                format!("{}-{}", &from.code[..3], &from.code[3..]),
                s_em,
                0.0,
                false,
            ));
        }
        if from.latin {
            lines.extend(
                latin_lines(&from)
                    .into_iter()
                    .map(|l| (l, s_em, 0.0, false)),
            );
        } else {
            let (number, rest) = split_line(&from.line);
            let places = format!("{}{}{}", from.region, from.city, from.town);
            let first: Vec<&str> = [places.trim(), number]
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect();
            if !first.is_empty() {
                lines.push((first.join(" "), s_em, 0.0, false));
            }
            if !rest.is_empty() {
                lines.push((spaced(rest), s_em, 2.0, false));
            }
        }
        if !from.name.trim().is_empty() {
            let name = full_width(from.name.trim());
            lines.push((name, em * SENDER_NAME_SCALE, 0.0, true));
        }
    }
    // Each line's em sits in its line-height's middle (the board's line-height 1.6).
    let lead = (L_LINE - 1.0) / 2.0;
    let height: f32 = lines.iter().map(|l| l.1 * L_LINE).sum();
    let mut line_top = (card.y + L_SENDER_TOP_MM - lead * s_em).min(bottom - height);
    // The 通信欄 between the recipient and the sender, across; with no sender to the card's
    // edge less 5 mm, since the 85 mm line bounds the address's area, not a message apart.
    let box_bottom = if lines.is_empty() {
        card.y + card.height - BOX_CLEAR_MM
    } else {
        line_top - BOX_CLEAR_MM
    };
    let rect = RectMm {
        x: left,
        y: text_bottom + BOX_CLEAR_MM,
        width,
        height: (box_bottom - text_bottom - BOX_CLEAR_MM).max(0.0),
    };
    let message_cut = message_box(canvas, face, density, rect, spec, false);

    let sender_width = L_SENDER_WIDTH_MM.min(width);
    for (text, size, indent, right) in &lines {
        let top = line_top + lead * size;
        if *right {
            let (_, w) = vertical::across(face, text, mm(*size), 0.0, 0.0, 0.0);
            let x = (mm(left + sender_width) - w).max(mm(left));
            let (set, _) = vertical::across(face, text, mm(*size), x, mm(top), 0.0);
            vertical::draw(canvas, face, &set, mm(*size), text_ink);
            line_top += size * L_LINE;
        } else {
            let next = draw_lines_across(
                canvas,
                face,
                text,
                mm(*size),
                mm(left + indent * size),
                mm(top),
                mm(sender_width - indent * size),
                mm(size * L_LINE),
                text_ink,
            );
            line_top = next / density - lead * size;
        }
    }
    // The address and sender wrap and push down; nothing of them is left out.
    crate::postcard::Fit {
        message_cut,
        address_cut: false,
    }
}

/// The recipient in Latin letters, across from `left` (romaji reads across): a long name set
/// smaller to fit the width, the lines kept above the half's foot.
#[allow(clippy::too_many_arguments)]
fn draw_latin(
    canvas: &mut RgbImage,
    face: &VerticalFace,
    density: f32,
    spec: &HagakiSpec<'_>,
    to: &Address,
    left: f32,
    top: f32,
    ink: [u8; 3],
) -> bool {
    let em = crate::settings::postcard_text_mm(spec.settings.text_size) * 0.8;
    let right = spec.card.x + spec.card.width - BOXES_RIGHT_MM;
    let bottom = spec.card.y + spec.card.height - 1.0;
    let width = (right - left) * density;
    let mut y = top;
    let name = to.name.trim();
    if !name.is_empty() {
        let mut size = em * NAME_SCALE * density;
        let (_, w) = vertical::across(face, name, size, 0.0, 0.0, 0.0);
        if w > width {
            size *= width / w;
        }
        let (set, _) = vertical::across(face, name, size, left * density, y * density, 0.0);
        vertical::draw(canvas, face, &set, size, ink);
        y += size / density * 1.4;
    }
    for line in latin_lines(to) {
        for row in vertical::wrap_across(face, &line, em * density, width, 0.0) {
            if y + em > bottom {
                return true;
            }
            let (set, _) =
                vertical::across(face, &row, em * density, left * density, y * density, 0.0);
            vertical::draw(canvas, face, &set, em * density, ink);
            y += em * 1.35;
        }
    }
    false
}

/// The message under a bordered photo (board A): one line from 1 mm in, its em 14/17 of the
/// text's and its top 2.6 mm under the photo (the board's 552 px against the photo's 544),
/// cut where the band ends (and says so); `band` starts at the photo's foot.
pub fn draw_message(
    canvas: &mut RgbImage,
    density: f32,
    band: RectMm,
    message: &str,
    settings: &PostcardSettings,
    paper: BorderColor,
) -> bool {
    let Some(face) = vertical::KLEE.as_ref() else {
        return false;
    };
    let line = message
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("");
    if line.is_empty() {
        return false;
    }
    // One line is printed: more lines, or more than the width, are left out.
    let mut cut = message.lines().filter(|l| !l.trim().is_empty()).count() > 1;
    let em = crate::settings::postcard_text_mm(settings.text_size) * MESSAGE_SCALE;
    let top = (band.y + MESSAGE_GAP_MM).min(band.y + band.height - em);
    // As much of the line as fits the photo's width.
    let room = (band.width - 2.0 * MESSAGE_IN_MM) * density;
    let mut shown = String::new();
    for c in line.chars() {
        shown.push(c);
        let (_, w) = vertical::across(face, &shown, em * density, 0.0, 0.0, 0.0);
        if w > room {
            shown.pop();
            cut = true;
            break;
        }
    }
    let (set, _) = vertical::across(
        face,
        &shown,
        em * density,
        (band.x + MESSAGE_IN_MM) * density,
        top * density,
        0.0,
    );
    vertical::draw(canvas, face, &set, em * density, ink(settings, paper));
    cut
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    const DENSITY: f32 = 300.0 / 25.4;
    const HALF: RectMm = RectMm {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 74.0,
    };

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn blank() -> RgbImage {
        let (w, h) = ((100.0 * DENSITY) as u32, (74.0 * DENSITY) as u32);
        RgbImage::from_raw(w, h, vec![255; (w * h * 3) as usize])
            .unwrap_or_else(|| RgbImage::new(1, 1))
    }

    fn kamakura() -> PostcardText {
        let mut to = Address::new(Country::Jp);
        to.code = "2480024".into();
        to.region = "神奈川県".into();
        to.city = "鎌倉市".into();
        to.town = "稲村ガ崎".into();
        to.line = "1-2-3".into();
        to.name = "鎌倉花子".into();
        PostcardText {
            message: String::new(),
            address: String::new(),
            recipients: vec![to],
        }
    }

    fn drawn(settings: &PostcardSettings, text: Option<&PostcardText>) -> RgbImage {
        let mut canvas = blank();
        draw_upright(
            &mut canvas,
            DENSITY,
            &HagakiSpec {
                card: HALF,
                settings,
                text,
                paper: BorderColor::White,
                // A message the test types goes to the 通信欄, as on a card whose line it overflows.
                message: text.map(|t| t.message.as_str()).filter(|m| !m.is_empty()),
            },
        );
        canvas
    }

    /// Pixels inside `r` (millimetres) that are not white.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn inked(canvas: &RgbImage, r: RectMm, pred: impl Fn([u8; 3]) -> bool) -> usize {
        let (x0, y0) = ((r.x * DENSITY) as u32, (r.y * DENSITY) as u32);
        let (x1, y1) = (
            ((r.x + r.width) * DENSITY) as u32,
            ((r.y + r.height) * DENSITY) as u32,
        );
        let mut n = 0;
        for y in y0..y1.min(canvas.height) {
            for x in x0..x1.min(canvas.width) {
                let i = ((y * canvas.width + x) * 3) as usize;
                let p = [canvas.data[i], canvas.data[i + 1], canvas.data[i + 2]];
                if pred(p) {
                    n += 1;
                }
            }
        }
        n
    }

    fn vermilion(p: [u8; 3]) -> bool {
        p == JAPAN_BOXES.color
    }

    fn dark(p: [u8; 3]) -> bool {
        p.iter().all(|c| *c < 120)
    }

    #[test]
    fn fr_130_the_address_keeps_out_of_the_stamp_box() {
        let settings = PostcardSettings::default();
        let full = drawn(&settings, Some(&kamakura()));
        let stamp_only = drawn(&settings, None);
        let s = stamp_box(HALF);
        // Inside the stamp box, the card with an address looks exactly like the empty one.
        let differs = inked(&full, s, |_| true) - inked(&stamp_only, s, |_| true);
        assert_eq!(differs, 0);
        assert_eq!(inked(&full, s, dark), inked(&stamp_only, s, dark));
    }

    #[test]
    fn fr_130_a_typed_code_is_written_in_the_boxes() {
        // 別記1: the code is written in the seven vermilion boxes, one digit a box, clear of
        // the frames (the owner, 2026-11-15: typed digits in the vermilion boxes).
        let settings = PostcardSettings::default();
        let (bx, by) = boxes_origin(HALF);
        let boxes = JAPAN_BOXES.boxes(bx, by);
        let area = zone(
            bx - 1.0,
            by - 1.0,
            JAPAN_BOXES.width_mm() + 2.0,
            JAPAN_BOXES.box_mm.1 + 2.0,
        );
        let typed = drawn(&settings, Some(&kamakura()));
        let mut empty = kamakura();
        if let Some(r) = empty.recipients.first_mut() {
            r.code.clear();
        }
        for card in [&typed, &drawn(&settings, Some(&empty))] {
            assert!(
                inked(card, area, vermilion) > 1000,
                "the seven boxes, typed or not"
            );
        }
        let mut inside = 0;
        for (r, _) in &boxes {
            let inner = zone(r.x + 0.6, r.y + 0.6, r.width - 1.2, r.height - 1.2);
            let digit = inked(&typed, inner, dark);
            assert!(digit > 50, "a digit in each box");
            inside += digit;
        }
        assert_eq!(
            inked(&typed, area, dark),
            inside,
            "no digit touches a frame"
        );
        assert_eq!(inked(&drawn(&settings, Some(&empty)), area, dark), 0);
    }

    #[test]
    fn fr_130_the_heading_follows_postal_use() {
        let band = RectMm {
            x: 30.0,
            y: 0.0,
            width: 40.0,
            height: 6.0,
        };
        let on = drawn(&PostcardSettings::default(), None);
        assert!(inked(&on, band, dark) > 100);
        let off = drawn(
            &PostcardSettings {
                postal_use: false,
                ..PostcardSettings::default()
            },
            None,
        );
        assert_eq!(inked(&off, band, dark), 0);
    }

    #[test]
    fn fr_130_the_name_runs_down_the_middle_and_the_sender_only_when_on() {
        let settings = PostcardSettings::default();
        let card = drawn(&settings, Some(&kamakura()));
        let middle = RectMm {
            x: 46.0,
            y: 25.0,
            width: 16.0,
            height: 44.0,
        };
        assert!(
            inked(&card, middle, dark) > 2000,
            "the name, large, in the middle"
        );
        // Right of the postmark zone, left of the name.
        let left = zone(POSTMARK_MM.0, 28.0, 12.0, 44.0);
        assert_eq!(
            inked(&card, left, dark),
            0,
            "no sender while its switch is off"
        );
        let mut from = Address::new(Country::Jp);
        from.region = "東京都".into();
        from.city = "足立区".into();
        from.name = "東京子".into();
        from.code = "1200024".into();
        let with = drawn(
            &PostcardSettings {
                sender_on: true,
                senders: vec![from],
                ..PostcardSettings::default()
            },
            Some(&kamakura()),
        );
        assert!(
            inked(&with, left, dark) > 500,
            "the sender right of the postmark zone"
        );
    }

    #[test]
    fn fr_130_a_european_address_never_becomes_the_recipient() {
        let mut text = PostcardText {
            message: String::new(),
            address: "HOANG PHI PHAM\nKodály Zoltán u. 9\n4028 DEBRECEN".into(),
            recipients: Vec::new(),
        };
        assert_eq!(recipient(Some(&text)), None);
        // A Korean recipient is not a Japanese card's either, and setting one keeps the other.
        let mut kr = Address::new(Country::Kr);
        kr.name = "홍길동".into();
        text.set_recipient(Country::Kr, Some(kr));
        assert_eq!(recipient(Some(&text)), None);
        let mut jp = Address::new(Country::Jp);
        jp.name = "鎌倉花子".into();
        text.set_recipient(Country::Jp, Some(jp));
        assert_eq!(text.recipients.len(), 2);
        assert_eq!(
            recipient(Some(&text)).map(|a| a.name),
            Some("鎌倉花子".to_owned())
        );
        assert_eq!(
            printed_code("2480024").as_deref(),
            Some("２４８\u{2212}００２４")
        );
        assert_eq!(printed_code("24800"), None);
    }

    #[test]
    fn fr_130_latin_lines_come_from_the_data() {
        let Some(to) = kamakura().recipients.into_iter().next() else {
            return;
        };
        let lines = latin_lines(&to);
        assert_eq!(
            lines,
            ["1-2-3", "Inamuragasaki, Kamakura-shi, Kanagawa-ken"]
        );
    }

    #[test]
    fn fr_130_every_character_printed_is_in_the_face() {
        use ab_glyph::Font;
        // A typed line with marks, a code, 様 and 切手: no character falls to the empty glyph.
        let Some(face) = vertical::KLEE.as_ref() else {
            return;
        };
        let printed = [
            full_width("1-2-3 Room 4B, #5 (rear)"),
            printed_code("2480024").unwrap_or_default(),
            "郵便はがき切手様".to_owned(),
        ];
        for text in &printed {
            for c in text.chars().filter(|c| !c.is_whitespace()) {
                assert_ne!(face.font.glyph_id(c).0, 0, "{c:?} is not in the face");
            }
        }
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn drawn_landscape(
        settings: &PostcardSettings,
        text: Option<&PostcardText>,
    ) -> (RgbImage, RectMm) {
        let half = RectMm {
            x: 0.0,
            y: 0.0,
            width: 74.0,
            height: 100.0,
        };
        let (w, h) = ((74.0 * DENSITY) as u32, (100.0 * DENSITY) as u32);
        let mut canvas = RgbImage::from_raw(w, h, vec![255; (w * h * 3) as usize])
            .unwrap_or_else(|| RgbImage::new(1, 1));
        draw_landscape(
            &mut canvas,
            DENSITY,
            &HagakiSpec {
                card: half,
                settings,
                text,
                paper: BorderColor::White,
                // A message the test types goes to the 通信欄, as on a card whose line it overflows.
                message: text.map(|t| t.message.as_str()).filter(|m| !m.is_empty()),
            },
        );
        (canvas, half)
    }

    #[test]
    fn fr_130_the_landscape_card_runs_its_boxes_down_the_right_edge() {
        let settings = PostcardSettings::default();
        let mut text = kamakura();
        if let Some(r) = text.recipients.first_mut() {
            r.code.clear();
        }
        let (canvas, half) = drawn_landscape(&settings, Some(&text));
        let (bx, by) = boxes_down_origin(half);
        // A column taller than wide, vermilion, under the stamp box.
        let column = RectMm {
            x: bx - 0.5,
            y: by - 0.5,
            width: JAPAN_BOXES.box_mm.1 + 1.0,
            height: JAPAN_BOXES.width_mm() + 1.0,
        };
        assert!(column.height > column.width * 4.0);
        let in_column = inked(&canvas, column, vermilion);
        assert!(in_column > 1000);
        assert_eq!(
            inked(&canvas, half, vermilion),
            in_column,
            "no box elsewhere"
        );
        // The stamp box stays as it is drawn alone.
        let (empty, _) = drawn_landscape(&settings, None);
        let s = stamp_box_landscape(half);
        assert_eq!(inked(&canvas, s, dark), inked(&empty, s, dark));
        // A typed code: its digits in the turned boxes.
        let (typed, _) = drawn_landscape(&settings, Some(&kamakura()));
        assert_eq!(inked(&typed, half, vermilion), in_column);
        for (r, _) in JAPAN_BOXES.boxes_down(bx, by) {
            let inner = zone(r.x + 0.6, r.y + 0.6, r.width - 1.2, r.height - 1.2);
            assert!(
                inked(&typed, inner, dark) > 50,
                "a digit in each turned box"
            );
        }
    }

    #[test]
    fn fr_130_the_senders_latin_switch_turns_its_places() {
        let mut from = Address::new(Country::Jp);
        from.code = "2710092".into();
        from.region = "千葉県".into();
        from.city = "松戸市".into();
        from.town = "松戸".into();
        from.latin = true;
        assert_eq!(latin_lines(&from), ["Matsudo, Matsudo-shi, Chiba-ken"]);
        let settings = PostcardSettings {
            sender_on: true,
            senders: vec![from.clone()],
            ..PostcardSettings::default()
        };
        let mut kanji = from;
        kanji.latin = false;
        let latin_card = drawn_landscape(&settings, None).0;
        let kanji_card = drawn_landscape(
            &PostcardSettings {
                sender_on: true,
                senders: vec![kanji],
                ..PostcardSettings::default()
            },
            None,
        )
        .0;
        assert_ne!(
            latin_card.data, kanji_card.data,
            "the switch changes the print"
        );
    }

    #[test]
    fn fr_130_nothing_but_the_stamp_in_the_postmark_zone() {
        // 別記1 備考3: the top left 35 × 70 mm upright, the top right 70 × 35 mm landscape.
        let mut latin = board_sender("1200024");
        if let Some(s) = latin.senders.first_mut() {
            s.latin = true;
        }
        let empty = PostcardSettings::default();
        let upright_zone = zone(0.0, 0.0, POSTMARK_MM.0, POSTMARK_MM.1);
        let landscape_zone = zone(74.0 - POSTMARK_MM.1, 0.0, POSTMARK_MM.1, POSTMARK_MM.0);
        for settings in [board_sender("1200024"), latin, PostcardSettings::default()] {
            for code in ["2480024", ""] {
                // A long message in the 通信欄 keeps out of the zone too.
                let mut text = board_text(code);
                text.message = "鎌倉の紅葉を送ります。お元気で。また一緒に歩きましょう。".repeat(3);
                let card = drawn(&settings, Some(&text));
                assert_eq!(
                    inked(&card, upright_zone, |_| true),
                    inked(&drawn(&empty, None), upright_zone, |_| true),
                    "upright"
                );
                let (card, _) = drawn_landscape(&settings, Some(&text));
                let (alone, _) = drawn_landscape(&empty, None);
                // The boxes start where the zone ends (35 mm); their top frame is theirs.
                let above = zone(landscape_zone.x, 0.0, landscape_zone.width, 34.5);
                assert_eq!(
                    inked(&card, above, |_| true),
                    inked(&alone, above, |_| true),
                    "landscape"
                );
            }
        }
    }

    #[test]
    fn fr_130_a_korean_sender_never_prints_on_a_hagaki() {
        // Each country its own sender (the QA review, 2026-11-15).
        let mut kr = Address::new(Country::Kr);
        kr.region = "부산광역시".into();
        kr.name = "김민지".into();
        let korean_only = PostcardSettings {
            sender_on: true,
            senders: vec![kr.clone()],
            ..PostcardSettings::default()
        };
        let text = board_text("2480024");
        let none = drawn(&PostcardSettings::default(), Some(&text));
        assert_eq!(drawn(&korean_only, Some(&text)).data, none.data);
        // A Japanese sender typed beside it is the one the はがき prints; the Korean one stays.
        let mut both = board_sender("");
        both.senders.push(kr);
        assert_ne!(drawn(&both, Some(&text)).data, none.data);
    }

    #[test]
    fn fr_130_the_text_keeps_to_the_address_area() {
        // 別記1 例1 and 例2 (the Japan Post review, 2026-11-15): upright, the recipient's ink
        // 15 mm or more from the right edge; landscape, the text 10 mm from the boxes and above
        // 85 mm, the boxes 2 mm under the postmark zone.
        let card = drawn(&board_sender("1200024"), Some(&board_text("2480024")));
        let right = runs(&card, zone(60.0, 21.0, 40.0, 53.0), true, dark)
            .iter()
            .map(|b| b[2])
            .fold(0.0, f32::max);
        assert!(right <= 85.0, "{right}");
        let (card, half) = drawn_landscape(&board_sender("1200024"), Some(&board_text("")));
        let (bx, by) = boxes_down_origin(half);
        assert!(by >= POSTMARK_MM.0 + 2.0 - 0.01);
        let text = runs(&card, zone(0.0, 30.0, bx - 1.0, 70.0), false, dark);
        let right = text.iter().map(|b| b[2]).fold(0.0, f32::max);
        let foot = text.iter().map(|b| b[3]).fold(0.0, f32::max);
        assert!(right <= bx - 0.5 - 10.0 + 0.3, "{right}");
        assert!(foot <= 85.0, "{foot}");
    }

    #[test]
    fn fr_130_the_address_keeps_10_mm_under_the_boxes() {
        // 別記1 (1)(ｵ): 10 mm or more between the boxes and the address.
        for code in ["2480024", ""] {
            let card = drawn(&PostcardSettings::default(), Some(&board_text(code)));
            let address = runs(&card, zone(70.0, 21.0, 29.0, 53.0), true, dark);
            let top = address.iter().map(|b| b[1]).fold(f32::MAX, f32::min);
            // From the frames' outside edge (the Japan Post review: 11 mm is the safe margin).
            let boxes_foot = BOXES_TOP_MM + JAPAN_BOXES.box_mm.1 + 0.5;
            assert!(top >= boxes_foot + 10.0, "{top}");
        }
    }

    #[test]
    fn fr_130_the_landscape_heading_is_centred_on_the_right() {
        // §22(6): on a landscape card, 「郵便はがき」 centred on the right side.
        let (card, _) = drawn_landscape(&PostcardSettings::default(), None);
        let heading = runs(&card, zone(67.5, 0.0, 6.5, 100.0), true, dark);
        let [b] = heading.as_slice() else {
            panic!("one column: {heading:?}");
        };
        let centre = f32::midpoint(b[1], b[3]);
        assert!((centre - 50.0).abs() <= 1.0, "{centre}");
    }

    #[test]
    fn fr_130_an_address_line_never_leaves_a_character_alone() {
        // 千葉県松戸市松戸 a little too wide for its line: drawn on one smaller line, not with
        // 戸 alone below (the QA review: the test now draws it).
        let Some(face) = vertical::KLEE.as_ref() else {
            return;
        };
        let em = 4.0 * DENSITY;
        let width = em * 7.6;
        assert_eq!(
            vertical::wrap_across(face, "千葉県松戸市松戸", em, width, em).len(),
            2
        );
        let mut canvas = blank();
        draw_lines_across(
            &mut canvas,
            face,
            "千葉県松戸市松戸",
            em,
            0.0,
            0.0,
            width,
            em * 1.4,
            [0, 0, 0],
        );
        let rows = runs(&canvas, HALF, false, dark);
        assert_eq!(rows.len(), 1, "{rows:?}");
    }

    /// The ink inside `zone` (millimetres), cut into its columns (`down`: vertical text, split
    /// where a strip of `zone`'s width has no ink) or its lines: each one's box, right to left
    /// or top to bottom, in millimetres.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    pub(crate) fn runs(
        canvas: &RgbImage,
        zone: RectMm,
        down: bool,
        pred: impl Fn([u8; 3]) -> bool,
    ) -> Vec<[f32; 4]> {
        let px = |v: f32| (v * DENSITY) as u32;
        let (x0, y0) = (px(zone.x), px(zone.y));
        let (x1, y1) = (
            px(zone.x + zone.width).min(canvas.width),
            px(zone.y + zone.height).min(canvas.height),
        );
        let ink = |x: u32, y: u32| {
            let i = ((y * canvas.width + x) * 3) as usize;
            pred([canvas.data[i], canvas.data[i + 1], canvas.data[i + 2]])
        };
        let (outer, inner) = if down {
            (x0..x1, y0..y1)
        } else {
            (y0..y1, x0..x1)
        };
        let gap = px(0.4);
        let mut out: Vec<[u32; 4]> = Vec::new();
        let mut last: Option<u32> = None;
        for o in outer {
            let hits: Vec<u32> = inner
                .clone()
                .filter(|&i| if down { ink(o, i) } else { ink(i, o) })
                .collect();
            let (Some(&lo), Some(&hi)) = (hits.first(), hits.last()) else {
                continue;
            };
            match (last, out.last_mut()) {
                (Some(l), Some(b)) if o - l <= gap => {
                    b[1] = o;
                    b[2] = b[2].min(lo);
                    b[3] = b[3].max(hi);
                }
                _ => out.push([o, o, lo, hi]),
            }
            last = Some(o);
        }
        let mm = |v: u32| v as f32 / DENSITY;
        let mut boxes: Vec<[f32; 4]> = out
            .iter()
            .map(|&[a, b, lo, hi]| {
                if down {
                    [mm(a), mm(lo), mm(b + 1), mm(hi + 1)]
                } else {
                    [mm(lo), mm(a), mm(hi + 1), mm(b + 1)]
                }
            })
            .collect();
        if down {
            boxes.reverse();
        }
        boxes
    }

    /// Each part's ink against the board's, measured in Chromium from the chosen artboards
    /// (4 px a millimetre, `Klee One`; scratch `board-measure`), within `TOLERANCE_MM`.
    pub(crate) const TOLERANCE_MM: f32 = 1.5;

    pub(crate) fn assert_board(part: &str, got: &[[f32; 4]], board: &[[f32; 4]], dx: f32) {
        assert_eq!(got.len(), board.len(), "{part}: {got:?} against {board:?}");
        for (g, b) in got.iter().zip(board) {
            let off = [b[0] - dx, b[1], b[2] - dx, b[3]];
            let worst = g
                .iter()
                .zip(off)
                .map(|(g, b)| (g - b).abs())
                .fold(0.0, f32::max);
            eprintln!("{part}: {worst:.2} mm {g:?} {off:?}");
            assert!(
                worst <= TOLERANCE_MM,
                "{part}: {g:?} against the board's {off:?}, {worst:.2} mm off"
            );
        }
    }

    /// The boards' sample: 静岡県熱海市和田町 5-27, 東京太郎 (○○町 on the board, a real town's
    /// characters here, as in the measure).
    fn board_text(code: &str) -> PostcardText {
        let mut to = Address::new(Country::Jp);
        to.code = code.into();
        to.region = "静岡県".into();
        to.city = "熱海市".into();
        to.town = "和田町".into();
        to.line = "5-27".into();
        to.name = "東京太郎".into();
        PostcardText {
            message: String::new(),
            address: String::new(),
            recipients: vec![to],
        }
    }

    fn board_sender(code: &str) -> PostcardSettings {
        let mut from = Address::new(Country::Jp);
        from.code = code.into();
        from.region = "東京都".into();
        from.city = "足立区".into();
        from.town = "千住関町".into();
        from.line = "5-27 ロイヤルアパート3号室".into();
        from.name = "東 京子".into();
        PostcardSettings {
            sender_on: true,
            senders: vec![from],
            ..PostcardSettings::default()
        }
    }

    /// A board's box moved by a rule of Japan Post's (`dx`, `dy` in millimetres).
    pub(crate) fn moved(b: [f32; 4], dx: f32, dy: f32) -> [f32; 4] {
        [b[0] + dx, b[1] + dy, b[2] + dx, b[3] + dy]
    }

    pub(crate) const fn zone(x: f32, y: f32, width: f32, height: f32) -> RectMm {
        RectMm {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn fr_130_the_upright_card_matches_board_a() {
        // Board A, where Japan Post agrees: its heading; the address and name as board B's,
        // moved by 別記1: the address 7 mm left into its 15 mm margin, both 1.5 mm lower for
        // 11 mm under the boxes (the Japan Post review, 2026-11-15).
        let card = drawn(&PostcardSettings::default(), Some(&board_text("2480024")));
        let address = [
            moved([87.63, 30.63, 91.44, 54.44], -7.0, 1.5),
            moved([81.63, 34.75, 85.44, 66.19], -7.0, 1.5),
        ];
        let name = [moved([48.0, 29.81, 54.25, 72.13], 0.0, 1.5)];
        let heading = [[38.25, 4.63, 59.63, 7.63]];
        let parts: [(&str, RectMm, bool, &[[f32; 4]]); 3] = [
            ("heading", zone(30.0, 0.0, 40.0, 9.0), false, &heading),
            ("address", zone(66.0, 22.0, 30.0, 52.0), true, &address),
            ("name", zone(40.0, 22.0, 22.0, 52.0), true, &name),
        ];
        for (part, z, down, board) in parts {
            assert_board(part, &runs(&card, z, down, dark), board, 0.0);
        }
    }

    #[test]
    fn fr_130_the_upright_card_matches_board_b() {
        // Board B: the code boxes for the hand; the address and name start lower.
        let card = drawn(&PostcardSettings::default(), Some(&board_text("")));
        let boxes = runs(&card, HALF, false, vermilion);
        let whole = [
            boxes.iter().map(|b| b[0]).fold(f32::MAX, f32::min),
            boxes.iter().map(|b| b[1]).fold(f32::MAX, f32::min),
            boxes.iter().map(|b| b[2]).fold(0.0, f32::max),
            boxes.iter().map(|b| b[3]).fold(0.0, f32::max),
        ];
        // 別記1 measures the boxes inside their frames: the frames reach 0.5 mm further.
        assert_board("boxes", &[whole], &[[44.25, 11.5, 92.3, 20.5]], 0.0);
        let address = runs(&card, zone(66.0, 22.0, 30.0, 52.0), true, dark);
        // Moved by 別記1 as in board A's test.
        let board = [
            moved([87.63, 30.63, 91.44, 54.44], -7.0, 1.5),
            moved([81.63, 34.75, 85.44, 66.19], -7.0, 1.5),
        ];
        assert_board("address", &address, &board, 0.0);
        let name = runs(&card, zone(40.0, 22.0, 22.0, 52.0), true, dark);
        assert_board(
            "name",
            &name,
            &[moved([48.0, 29.81, 54.25, 72.13], 0.0, 1.5)],
            0.0,
        );
    }

    #[test]
    fn fr_130_the_landscape_card_matches_its_board() {
        // The landscape board: the boxes, the sender with its code; the half starts at 74 mm.
        let (card, _) = drawn_landscape(&board_sender("1200024"), Some(&board_text("")));
        let dx = 74.0;
        let boxes = runs(&card, zone(0.0, 0.0, 74.0, 100.0), true, vermilion);
        let whole = [
            boxes.iter().map(|b| b[0]).fold(f32::MAX, f32::min),
            boxes.iter().map(|b| b[1]).fold(f32::MAX, f32::min),
            boxes.iter().map(|b| b[2]).fold(0.0, f32::max),
            boxes.iter().map(|b| b[3]).fold(0.0, f32::max),
        ];
        // 2 mm lower and measured inside the frames (the Japan Post review, 2026-11-15).
        assert_board("boxes", &[whole], &[[131.5, 36.5, 140.5, 85.0]], dx);
        let parts: [(&str, RectMm, bool, &[[f32; 4]]); 4] = [
            (
                "heading",
                zone(67.5, 32.0, 6.5, 28.0),
                true,
                &[[142.69, 41.31, 145.38, 58.68]],
            ),
            (
                "address",
                zone(2.0, 33.0, 55.0, 16.0),
                false,
                &[[78.13, 36.38, 100.38, 39.94], [81.88, 42.44, 102.13, 45.88]],
            ),
            (
                "name",
                zone(2.0, 49.0, 55.0, 13.0),
                false,
                // 様 ends 10 mm from the boxes (別記1 例2), the name a little smaller.
                &[[78.13, 52.19, 122.0, 58.5]],
            ),
            (
                "sender",
                zone(2.0, 62.0, 55.0, 33.0),
                false,
                // Kept above 85 mm, 別記1's 15 mm foot (the Japan Post review): 5.7 mm higher.
                &[
                    moved([78.31, 73.75, 90.44, 75.81], 0.0, -5.7),
                    moved([78.06, 77.94, 112.19, 80.56], 0.0, -5.7),
                    moved([84.0, 82.19, 114.25, 84.75], 0.0, -5.7),
                    // Its right end 10 mm from the boxes, as the address's (別記1 例2).
                    moved([115.06, 86.75, 127.94, 89.81], -6.0, -5.7),
                ],
            ),
        ];
        for (part, z, down, board) in parts {
            assert_board(part, &runs(&card, z, down, dark), board, dx);
        }
    }

    #[test]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn fr_130_the_message_under_the_photo_matches_board_a() {
        // Board A's photo is 90 mm wide and ends at 136 mm; the band starts there.
        let (w, h) = ((100.0 * DENSITY) as u32, (148.0 * DENSITY) as u32);
        let mut card = RgbImage::from_raw(w, h, vec![255; (w * h * 3) as usize])
            .unwrap_or_else(|| RgbImage::new(1, 1));
        let band = zone(5.0, 136.0, 90.0, 12.0);
        let settings = PostcardSettings::default();
        draw_message(
            &mut card,
            DENSITY,
            band,
            "鎌倉の紅葉を送ります。お元気で。",
            &settings,
            BorderColor::White,
        );
        let line = runs(&card, band, false, dark);
        assert_board("message", &line, &[[6.06, 139.06, 58.88, 142.38]], 0.0);
    }
}
