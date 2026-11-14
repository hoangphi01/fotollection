// SPDX-License-Identifier: GPL-3.0-only
//! The はがき's address side (FR-130): Japan Post's layout on the writing half of a Japanese
//! card, its photo on the other half as every postcard of the app (one side, PRD FR-130).
//! Upright, the text runs top to bottom (`vertical`): the stamp at the top left, 「郵便はがき」
//! centred at the top, the seven vermilion code boxes at the top right (or the code typed, in
//! full-width digits, where they would be), the recipient's address down the right, the name
//! down the middle, the largest, with 様, the sender on the left when its switch is on.
//! Positions are Japan Post's figures' numbers, estimates until measured on a real card.

use crate::addassist::{Address, Country};
use crate::image::RgbImage;
use crate::library::PostcardText;
use crate::postcard::JAPAN_BOXES;
use crate::settings::{BorderColor, PostcardSettings};
use crate::sheet::RectMm;
use crate::vertical::{self, VerticalFace};

/// The stamp box, at the card's top left (FAQ 597: within 7.0 × 3.5 cm).
pub const STAMP_MM: (f32, f32) = (20.0, 25.0);
/// From the card's edges to the stamp box and the text.
pub const EDGE_MM: f32 = 5.0;
/// The code boxes' top and right offsets (Japan Post's figure: 12 mm and 8 mm).
pub const BOXES_TOP_MM: f32 = 12.0;
pub const BOXES_RIGHT_MM: f32 = 8.0;
/// 「郵便はがき」's em and its top.
const HEADING_MM: f32 = 3.0;
const HEADING_TOP_MM: f32 = 2.0;
/// A typed code's em: full-width digits 2.4–6 mm tall (別記1).
const CODE_MM: f32 = 4.4;
/// The name's em against the address's.
const NAME_SCALE: f32 = 1.55;
/// The sender's em against the address's.
const SENDER_SCALE: f32 = 0.72;

/// What a はがき's writing half draws.
pub struct HagakiSpec<'a> {
    /// The writing half, in sheet millimetres.
    pub card: RectMm,
    pub settings: &'a PostcardSettings,
    pub text: Option<&'a PostcardText>,
    pub paper: BorderColor,
}

/// The stamp box of a はがき whose writing half is `card`.
pub fn stamp_box(card: RectMm) -> RectMm {
    RectMm {
        x: card.x + EDGE_MM,
        y: card.y + EDGE_MM,
        width: STAMP_MM.0,
        height: STAMP_MM.1,
    }
}

/// The code boxes' top-left.
fn boxes_origin(card: RectMm) -> (f32, f32) {
    (
        card.x + card.width - BOXES_RIGHT_MM - JAPAN_BOXES.width_mm(),
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

/// An address's lines in kanji: the places, then the typed line.
fn kanji_lines(a: &Address) -> String {
    let places = format!("{}{}{}", a.region, a.city, a.town);
    let line = a.line.trim();
    if line.is_empty() {
        full_width(&places)
    } else {
        format!("{}\n{}", full_width(&places), full_width(line))
    }
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

/// The recipient from the card's text: its structured address, or the free address's last
/// line as the name and the lines above it as the address.
fn recipient(text: Option<&PostcardText>) -> Option<Address> {
    let text = text?;
    if let Some(r) = &text.recipient {
        return Some(r.clone());
    }
    let lines: Vec<&str> = text
        .address
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    let (name, rest) = lines.split_last()?;
    let mut a = Address::new(Country::Jp);
    (*name).clone_into(&mut a.name);
    a.line = rest.join("\n");
    Some(a)
}

fn ink(settings: &PostcardSettings, paper: BorderColor) -> [u8; 3] {
    settings.ink.rgb(paper)
}

/// Draws a はがき's upright writing half onto `canvas`, `density` pixels per millimetre.
#[allow(clippy::too_many_lines)]
pub fn draw_upright(canvas: &mut RgbImage, density: f32, spec: &HagakiSpec<'_>) {
    let Some(face) = vertical::KLEE.as_ref() else {
        return;
    };
    let card = spec.card;
    let settings = spec.settings;
    let (outline, _) = crate::postcard::inks(spec.paper);
    let text_ink = ink(settings, spec.paper);
    let mm = |v: f32| v * density;
    let em = crate::settings::postcard_text_mm(settings.text_size);

    // The stamp box, 切手 down its middle.
    let stamp = stamp_box(card);
    crate::postcard::draw_dashed_box(canvas, density, stamp, outline);
    let word_em = 3.6;
    let word = vertical::set(
        face,
        "切手",
        mm(word_em),
        mm(stamp.x + stamp.width / 2.0 + word_em / 2.0),
        mm(stamp.y + stamp.height / 2.0 - word_em),
        mm(word_em * 2.5),
        mm(word_em),
        1,
        0,
    );
    vertical::draw(canvas, face, &word, mm(word_em), outline);

    // 「郵便はがき」, which makes a card printed at home a postcard (内国郵便約款 §22(6)).
    if settings.postal_use {
        let (probe, width) = vertical::across(face, "郵便はがき", mm(HEADING_MM), 0.0, 0.0, 0.5);
        drop(probe);
        let left = mm(card.x + card.width / 2.0) - width / 2.0;
        let (heading, _) = vertical::across(
            face,
            "郵便はがき",
            mm(HEADING_MM),
            left,
            mm(card.y + HEADING_TOP_MM),
            0.5,
        );
        vertical::draw(canvas, face, &heading, mm(HEADING_MM), text_ink);
    }

    let to = recipient(spec.text);
    // The code: typed, printed where the boxes would be, with no boxes and no 〒; else the
    // seven boxes, empty, for the hand.
    let (bx, by) = boxes_origin(card);
    match to.as_ref().and_then(|a| printed_code(&a.code)) {
        Some(code) => {
            let (_, width) = vertical::across(face, &code, mm(CODE_MM), 0.0, 0.0, 0.08);
            let right = mm(card.x + card.width - BOXES_RIGHT_MM);
            let (set, _) =
                vertical::across(face, &code, mm(CODE_MM), right - width, mm(by + 1.5), 0.08);
            vertical::draw(canvas, face, &set, mm(CODE_MM), text_ink);
        }
        None => crate::postcard::draw_code_boxes(canvas, density, bx, by, &JAPAN_BOXES),
    }

    // Below the boxes, with a character's blank: the address down the right, the name down
    // the middle.
    let top = by + JAPAN_BOXES.box_mm.1 + em;
    let bottom = card.y + card.height - EDGE_MM;
    let right = card.x + card.width - BOXES_RIGHT_MM;
    if let Some(to) = &to {
        if to.latin {
            draw_latin(canvas, face, density, spec, to, top, text_ink);
        } else {
            let address = kanji_lines(to);
            let set = vertical::set(
                face,
                &address,
                mm(em),
                mm(right),
                mm(top),
                mm(bottom - top),
                mm(em * 1.3),
                3,
                1,
            );
            vertical::draw(canvas, face, &set, mm(em), text_ink);
            let name = to.name.trim();
            if !name.is_empty() {
                let label = format!("{}　様", full_width(name));
                let chars = label.chars().count();
                #[allow(clippy::cast_precision_loss)]
                let name_em = (em * NAME_SCALE).min((bottom - top) / chars.max(1) as f32);
                let centre = card.x + card.width / 2.0 + 4.0;
                let set = vertical::set(
                    face,
                    &label,
                    mm(name_em),
                    mm(centre + name_em / 2.0),
                    mm(top),
                    mm(bottom - top),
                    mm(name_em),
                    1,
                    0,
                );
                vertical::draw(canvas, face, &set, mm(name_em), text_ink);
            }
        }
    }

    // The sender down the left, under the stamp box, when its switch is on.
    if settings.sender_on
        && let Some(from) = settings.sender.as_ref().filter(|a| !a.is_blank())
    {
        let small = em * SENDER_SCALE;
        let from_top = stamp.y + stamp.height + 3.0;
        let code_room = if printed_code(&from.code).is_some() {
            small + 1.0
        } else {
            0.0
        };
        let mut text = kanji_lines(from);
        if !from.name.trim().is_empty() {
            text.push('\n');
            text.push_str(&full_width(from.name.trim()));
        }
        let set = vertical::set(
            face,
            &text,
            mm(small),
            mm(stamp.x + stamp.width),
            mm(from_top),
            mm(bottom - from_top - code_room),
            mm(small * 1.3),
            4,
            1,
        );
        vertical::draw(canvas, face, &set, mm(small), text_ink);
        if let Some(code) = printed_code(&from.code) {
            let (set, _) =
                vertical::across(face, &code, mm(small), mm(stamp.x), mm(bottom - small), 0.0);
            vertical::draw(canvas, face, &set, mm(small), text_ink);
        }
    }
}

/// The stamp box of a landscape はがき whose writing half is `card`: its top right.
pub fn stamp_box_landscape(card: RectMm) -> RectMm {
    RectMm {
        x: card.x + card.width - EDGE_MM - STAMP_MM.0,
        y: card.y + EDGE_MM,
        width: STAMP_MM.0,
        height: STAMP_MM.1,
    }
}

/// The landscape はがき's code column: its top-left, under the stamp box, along the right edge.
fn boxes_down_origin(card: RectMm) -> (f32, f32) {
    let stamp = stamp_box_landscape(card);
    (
        card.x + card.width - EDGE_MM - JAPAN_BOXES.box_mm.1,
        stamp.y + stamp.height + 4.0,
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
        let capacity = (width / em).floor().max(2.0) as usize;
        for (i, line) in vertical::columns(paragraph, capacity).iter().enumerate() {
            let indent = if i == 0 { 0.0 } else { em };
            let line: String = line.iter().collect();
            let (set, _) = vertical::across(face, &line, em, left + indent, y, 0.0);
            vertical::draw(canvas, face, &set, em, ink);
            y += step;
        }
    }
    y
}

/// Draws a はがき's landscape writing half (横書き): the stamp at the top right, the seven
/// boxes down the right edge under it with 「郵便はがき」 beside them, the address across in
/// Arabic figures, the name in the middle with 様, the sender at the foot.
#[allow(clippy::too_many_lines)]
pub fn draw_landscape(canvas: &mut RgbImage, density: f32, spec: &HagakiSpec<'_>) {
    let Some(face) = vertical::KLEE.as_ref() else {
        return;
    };
    let card = spec.card;
    let settings = spec.settings;
    let (outline, _) = crate::postcard::inks(spec.paper);
    let text_ink = ink(settings, spec.paper);
    let mm = |v: f32| v * density;
    let em = crate::settings::postcard_text_mm(settings.text_size);

    let stamp = stamp_box_landscape(card);
    crate::postcard::draw_dashed_box(canvas, density, stamp, outline);
    let word_em = 3.6;
    let (_, word_w) = vertical::across(face, "切手", mm(word_em), 0.0, 0.0, 0.0);
    let (word, _) = vertical::across(
        face,
        "切手",
        mm(word_em),
        mm(stamp.x + stamp.width / 2.0) - word_w / 2.0,
        mm(stamp.y + stamp.height / 2.0 - word_em / 2.0),
        0.0,
    );
    vertical::draw(canvas, face, &word, mm(word_em), outline);

    let to = recipient(spec.text);
    let (bx, by) = boxes_down_origin(card);
    let typed = to.as_ref().and_then(|a| printed_code(&a.code));
    if typed.is_none() {
        crate::postcard::draw_code_boxes_down(canvas, density, bx, by, &JAPAN_BOXES);
    }
    // 「郵便はがき」 beside the boxes, down the left of their column.
    if settings.postal_use {
        let set = vertical::set(
            face,
            "郵便はがき",
            mm(HEADING_MM),
            mm(bx - 1.0),
            mm(by),
            mm(HEADING_MM * 6.0),
            mm(HEADING_MM),
            1,
            0,
        );
        vertical::draw(canvas, face, &set, mm(HEADING_MM), text_ink);
    }

    // The text area: from the left edge to the heading's column.
    let left = card.x + EDGE_MM;
    let right = bx - HEADING_MM - 3.0;
    let width = right - left;
    let bottom = card.y + card.height - EDGE_MM;
    if let Some(to) = &to {
        let mut y = card.y + EDGE_MM + 3.0;
        if let Some(code) = &typed {
            let (set, _) = vertical::across(face, code, mm(CODE_MM), mm(left), mm(y), 0.08);
            vertical::draw(canvas, face, &set, mm(CODE_MM), text_ink);
            y += CODE_MM * 1.6;
        }
        let address = if to.latin {
            latin_lines(to).join("\n")
        } else {
            let places = format!("{}{}{}", to.region, to.city, to.town);
            let line = to.line.trim();
            if line.is_empty() {
                places
            } else {
                format!("{places}\n{line}")
            }
        };
        let room = (stamp.x - 3.0 - left).max(width * 0.6);
        let after = draw_lines_across(
            canvas,
            face,
            &address,
            mm(em),
            mm(left),
            mm(y),
            mm(room.min(width)),
            mm(em * 1.4),
            text_ink,
        );
        let name = to.name.trim();
        if !name.is_empty() {
            let label = if to.latin {
                name.to_owned()
            } else {
                format!("{name}　様")
            };
            let name_em = em * NAME_SCALE;
            let (_, w) = vertical::across(face, &label, mm(name_em), 0.0, 0.0, 0.0);
            let centre_y = (after / density + name_em).max(card.y + card.height * 0.5);
            let x = (mm(left + width / 2.0) - w / 2.0).max(mm(left));
            let (set, _) = vertical::across(face, &label, mm(name_em), x, mm(centre_y), 0.0);
            vertical::draw(canvas, face, &set, mm(name_em), text_ink);
        }
    }

    // The sender at the foot, its typed code first.
    if settings.sender_on
        && let Some(from) = settings.sender.as_ref().filter(|a| !a.is_blank())
    {
        let small = em * SENDER_SCALE;
        let mut lines = Vec::new();
        if let Some(code) = printed_code(&from.code) {
            lines.push(code);
        }
        let places = format!(
            "{}{}{}{}",
            from.region,
            from.city,
            from.town,
            from.line.trim()
        );
        if !places.is_empty() {
            lines.push(places);
        }
        if !from.name.trim().is_empty() {
            lines.push(from.name.trim().to_owned());
        }
        #[allow(clippy::cast_precision_loss)]
        let top = bottom - small * 1.35 * lines.len() as f32;
        draw_lines_across(
            canvas,
            face,
            &lines.join("\n"),
            mm(small),
            mm(left),
            mm(top),
            mm(width),
            mm(small * 1.35),
            text_ink,
        );
    }
}

/// The recipient in Latin letters, across, right of the middle (romaji reads across).
fn draw_latin(
    canvas: &mut RgbImage,
    face: &VerticalFace,
    density: f32,
    spec: &HagakiSpec<'_>,
    to: &Address,
    top: f32,
    ink: [u8; 3],
) {
    let em = crate::settings::postcard_text_mm(spec.settings.text_size) * 0.8;
    let left = spec.card.x + spec.card.width * 0.38;
    let mut y = top;
    let name = to.name.trim();
    if !name.is_empty() {
        let (set, _) = vertical::across(
            face,
            name,
            em * NAME_SCALE * density,
            left * density,
            y * density,
            0.0,
        );
        vertical::draw(canvas, face, &set, em * NAME_SCALE * density, ink);
        y += em * NAME_SCALE * 1.4;
    }
    for line in latin_lines(to) {
        let (set, _) =
            vertical::across(face, &line, em * density, left * density, y * density, 0.0);
        vertical::draw(canvas, face, &set, em * density, ink);
        y += em * 1.35;
    }
}

/// The message on a bordered photo's half (board A): one line across the band under the
/// photo, in the はがき's face.
pub fn draw_message(
    canvas: &mut RgbImage,
    density: f32,
    band: RectMm,
    message: &str,
    settings: &PostcardSettings,
    paper: BorderColor,
) {
    let Some(face) = vertical::KLEE.as_ref() else {
        return;
    };
    let line = message
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("");
    if line.is_empty() {
        return;
    }
    let em = (band.height * 0.55).min(crate::settings::postcard_text_mm(settings.text_size));
    // As much of the line as fits the band's width.
    let room = (band.width - 2.0 * 2.0) * density;
    let mut shown = String::new();
    for c in line.chars() {
        shown.push(c);
        let (_, w) = vertical::across(face, &shown, em * density, 0.0, 0.0, 0.0);
        if w > room {
            shown.pop();
            break;
        }
    }
    let (set, _) = vertical::across(
        face,
        &shown,
        em * density,
        (band.x + 2.0) * density,
        (band.y + (band.height - em) / 2.0) * density,
        0.0,
    );
    vertical::draw(canvas, face, &set, em * density, ink(settings, paper));
}

#[cfg(test)]
mod tests {
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
            recipient: Some(to),
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
    fn fr_130_the_code_is_boxes_for_the_hand_or_printed_without_them() {
        let settings = PostcardSettings::default();
        let mut text = kamakura();
        if let Some(r) = text.recipient.as_mut() {
            r.code.clear();
        }
        let boxes = drawn(&settings, Some(&text));
        let (bx, by) = boxes_origin(HALF);
        let area = RectMm {
            x: bx - 1.0,
            y: by - 1.0,
            width: JAPAN_BOXES.width_mm() + 2.0,
            height: JAPAN_BOXES.box_mm.1 + 2.0,
        };
        assert!(
            inked(&boxes, area, vermilion) > 1000,
            "seven vermilion boxes"
        );
        let typed = drawn(&settings, Some(&kamakura()));
        assert_eq!(
            inked(&typed, HALF, vermilion),
            0,
            "a typed code has no boxes"
        );
        assert!(
            inked(&typed, area, dark) > 100,
            "the code is printed where they were"
        );
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
        let left = RectMm {
            x: 2.0,
            y: 32.0,
            width: 26.0,
            height: 38.0,
        };
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
                sender: Some(from),
                ..PostcardSettings::default()
            },
            Some(&kamakura()),
        );
        assert!(inked(&with, left, dark) > 500, "the sender down the left");
    }

    #[test]
    fn fr_130_a_free_address_reads_its_last_line_as_the_name() {
        let text = PostcardText {
            message: String::new(),
            address: "神奈川県鎌倉市\n鎌倉花子".into(),
            recipient: None,
        };
        let a = recipient(Some(&text)).unwrap_or_else(|| Address::new(Country::Jp));
        assert_eq!(
            (a.name.as_str(), a.line.as_str()),
            ("鎌倉花子", "神奈川県鎌倉市")
        );
        assert_eq!(
            printed_code("2480024").as_deref(),
            Some("２４８\u{2212}００２４")
        );
        assert_eq!(printed_code("24800"), None);
    }

    #[test]
    fn fr_130_latin_lines_come_from_the_data() {
        let Some(to) = kamakura().recipient else {
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
            },
        );
        (canvas, half)
    }

    #[test]
    fn fr_130_the_landscape_card_runs_its_boxes_down_the_right_edge() {
        let settings = PostcardSettings::default();
        let mut text = kamakura();
        if let Some(r) = text.recipient.as_mut() {
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
        // A typed code: no boxes.
        let (typed, _) = drawn_landscape(&settings, Some(&kamakura()));
        assert_eq!(inked(&typed, half, vermilion), 0);
    }
}
