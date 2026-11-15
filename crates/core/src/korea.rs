// SPDX-License-Identifier: GPL-3.0-only
//! The 우편엽서's address side (FR-132): Korea Post's rules on the writing half of a Korean
//! card (board K1, owner 2026-11-15), its photo on the other half. 「우편엽서」 centred at the
//! top, the sender at the top left (보내는 사람, its own five boxes) when its switch is on, the
//! 우표 box at the top right with `fotollection.org/ko` under it, the recipient at the bottom
//! right (받는 사람, the name the largest, 귀하 when chosen, the address in the road-name
//! order, the five boxes right-aligned with 4 mm clear round them), the machine zone's 17 mm
//! at the card's foot kept blank. An upright card sets the same on its top half, the zone down
//! its right edge (Korea Post's foot when the card is read turned).

use std::sync::LazyLock;

use ab_glyph::Font;

use crate::addassist::{Address, Country};
use crate::image::RgbImage;
use crate::library::PostcardText;
use crate::postcard::{CodeBoxes, KOREA_BOXES, KOREA_SENDER_BOXES};
use crate::settings::{BorderColor, PostcardFace, PostcardSettings};
use crate::sheet::RectMm;
use crate::vertical::{self, VerticalFace};

const NANUM_PEN: &[u8] = include_bytes!("../assets/fonts/NanumPenScript-Regular-subset.ttf");
const GAEGU: &[u8] = include_bytes!("../assets/fonts/Gaegu-Regular-subset.ttf");

/// Nanum Pen Script, every Hangul syllable (FONTS.md).
pub static NANUM: LazyLock<Option<VerticalFace>> = LazyLock::new(|| VerticalFace::new(NANUM_PEN));
/// Gaegu, KS X 1001's syllables; what it lacks is set in Nanum Pen.
pub static GAEGU_FACE: LazyLock<Option<VerticalFace>> = LazyLock::new(|| VerticalFace::new(GAEGU));

/// Korea Post's machine zone: the last 17 mm at the card's foot (jodal.koreapost.go.kr 216).
pub const MACHINE_ZONE_MM: f32 = 17.0;
/// The strip beside the machine zone for its line and words (owner, 2026-11-15: "a line as the
/// European card's, and the Korean words repeated"): the zone itself stays blank (Korea Post),
/// so the line is 23 mm from the foot and the words, 3 mm (2.2 read too small; owner,
/// 2026-11-15), between it and the zone.
pub const TAPE_MM: f32 = 6.0;
/// What the strip repeats, and its em in Nanum Pen.
const TAPE_WORDS: [&str; 1] = ["우편물 처리 영역"];
const TAPE_EM_MM: f32 = 3.0;

/// The machine zone's line and its words in the strip beside `zone`: above a zone at the foot,
/// left of a zone down an upright card's right edge.
pub fn draw_tape(canvas: &mut RgbImage, density: f32, zone: RectMm, paper: BorderColor) {
    let Some(face) = NANUM.as_ref() else {
        return;
    };
    let strip = if zone.width >= zone.height {
        RectMm {
            y: zone.y - TAPE_MM,
            height: TAPE_MM,
            ..zone
        }
    } else {
        RectMm {
            x: zone.x - TAPE_MM,
            width: TAPE_MM,
            ..zone
        }
    };
    let tape = crate::postcard::Tape {
        words: &TAPE_WORDS,
        font: &face.font,
        em_mm: TAPE_EM_MM,
    };
    crate::postcard::draw_postal_tape(canvas, density, strip, paper, &tape);
}

/// The stamp box and the card's margins.
pub const STAMP_MM: (f32, f32) = (20.0, 25.0);
const EDGE_MM: f32 = 5.0;
const GREY: [u8; 3] = [110, 110, 110];
/// The dotted rule under a name and an address line (board K1: 1 px dotted #9a9a9a).
const RULE: [u8; 3] = [154, 154, 154];

// Every position below is board K1's (PRD FR-132): 4 px a millimetre on a 148 × 100 card,
// measured in Chromium in Nanum Pen (LESSONS 2026-11-15: a layout that has a board takes the
// board's numbers). Ems scale with the text size; the board's is 17 px.

/// 「우 편 엽 서」, centred between 2.5 mm into the half and the stamp box's left: em 4.2 mm,
/// 0.8 mm between characters, its letters about 3 mm tall as 「郵便はがき」's. The board's
/// 2.25 mm was drawn in a sans; Nanum Pen's letters fill less of their em and read too small
/// on the card (owner, 2026-11-15).
const HEADING_MM: f32 = 4.2;
const HEADING_TRACK_MM: f32 = 0.8;
const HEADING_TOP_MM: f32 = 1.0;
/// 보내는 사람, 받는 사람.
const LABEL_MM: f32 = 2.2;
/// The sender: from 4 mm into the half, top 7.5 mm, 42 mm wide (312, 30, 168 px); its name
/// 20/17 em, its address 15/17, its code's digits 14/17; 0.75 mm between rows (3 px).
const SENDER_LEFT_MM: f32 = 4.0;
const SENDER_TOP_MM: f32 = 7.5;
const SENDER_WIDTH_MM: f32 = 42.0;
const SENDER_NAME: f32 = 20.0 / 17.0;
const SENDER_ADDRESS: f32 = 15.0 / 17.0;
const SENDER_GAP_MM: f32 = 0.75;
/// The recipient: from 8.5 mm into the half to 5 mm from its right (330 and 20 px); its name
/// 30/17 em on the right, its address 17/17, 1 mm between rows (4 px), the boxes 4 mm under the
/// address and 6 mm above the zone's words (FR-132 asks 4; 6 for a kiosk's cut), 22 mm from the
/// right edge.
const RECIPIENT_LEFT_MM: f32 = 8.5;
const RECIPIENT_NAME: f32 = 30.0 / 17.0;
const RECIPIENT_GAP_MM: f32 = 1.0;
const BOXES_GAP_MM: f32 = 6.0;
const BOXES_CLEAR_MM: f32 = 6.0;
/// 20 mm blank right of the code (Korea Post: 우편번호 오른쪽 끝에서 20 mm), 22 here for a
/// kiosk's cut (the Korea Post review, 2026-11-15).
const BOXES_RIGHT_MM: f32 = 22.0;
/// Where a line's em starts in its line box, against its em (Nanum Pen; measured).
const EM_IN_BOX: f32 = 0.05;
/// The message under a bordered photo: 19/17 em, 2.5 mm under it (322 px against 312).
const MESSAGE_SCALE: f32 = 19.0 / 17.0;
const MESSAGE_GAP_MM: f32 = 2.5;

/// What a 우편엽서's writing half draws.
pub struct KoreaSpec<'a> {
    pub card: RectMm,
    pub settings: &'a PostcardSettings,
    pub text: Option<&'a PostcardText>,
    pub paper: BorderColor,
    /// The machine zone, when it crosses the writing half (a landscape card's foot, an upright
    /// card's right edge).
    pub zone: Option<RectMm>,
    /// The message for the card's 통신란, when it does not fit the line under the photo.
    pub message: Option<&'a str>,
}

/// The two faces in use: the chosen one, and Nanum Pen for what it lacks.
fn faces(settings: &PostcardSettings) -> Option<(&'static VerticalFace, &'static VerticalFace)> {
    let nanum = NANUM.as_ref()?;
    let main = match settings.face {
        PostcardFace::Gaegu => GAEGU_FACE.as_ref().unwrap_or(nanum),
        _ => nanum,
    };
    Some((main, nanum))
}

/// The stamp box: the writing half's top right.
pub fn stamp_box(card: RectMm) -> RectMm {
    let height = STAMP_MM.1.min(card.height * 0.45);
    RectMm {
        x: card.x + card.width - EDGE_MM - STAMP_MM.0,
        y: card.y + EDGE_MM,
        width: STAMP_MM.0,
        height,
    }
}

/// One line across from (`left`, `top`) in pixels, each character in the first face that has
/// it; returns its width.
#[allow(clippy::too_many_arguments)]
fn line(
    canvas: Option<&mut RgbImage>,
    faces: (&VerticalFace, &VerticalFace),
    text: &str,
    em: f32,
    left: f32,
    top: f32,
    color: [u8; 3],
) -> f32 {
    let mut x = left;
    let mut glyphs: Vec<(&VerticalFace, char, f32)> = Vec::new();
    for c in text.chars() {
        // The chosen face, else Nanum Pen; Latin letters in the handwriting face (`across`).
        let face = if faces.0.font.glyph_id(c).0 != 0 || faces.1.font.glyph_id(c).0 == 0 {
            faces.0
        } else {
            faces.1
        };
        glyphs.push((face, c, x));
        x += face.advance(c, em);
    }
    if let Some(canvas) = canvas {
        for (face, c, gx) in glyphs {
            let (set, _) = vertical::across(face, &c.to_string(), em, gx, top, 0.0);
            vertical::draw(canvas, face, &set, em, color);
        }
    }
    x - left
}

/// `text` broken between words to `width` pixels.
fn wrap(faces: (&VerticalFace, &VerticalFace), text: &str, em: f32, width: f32) -> Vec<String> {
    let mut rows = Vec::new();
    for paragraph in text.lines() {
        let mut row = String::new();
        for word in paragraph.split_whitespace() {
            let candidate = if row.is_empty() {
                word.to_owned()
            } else {
                format!("{row} {word}")
            };
            if !row.is_empty() && line(None, faces, &candidate, em, 0.0, 0.0, [0; 3]) > width {
                rows.push(std::mem::take(&mut row));
                word.clone_into(&mut row);
            } else {
                row = candidate;
            }
        }
        if !row.is_empty() {
            rows.push(row);
        }
    }
    rows
}

/// An address in the road-name order: 시·도 시·군·구 읍·면 도로명 건물번호, then on a line of its
/// own what follows the number (상세주소); in Latin letters the international order, the code last.
fn address_text(a: &Address) -> String {
    if a.latin {
        let found = crate::addassist::lookup(Country::Kr, &a.code)
            .ok()
            .and_then(|f| f.into_iter().find(|f| f.road == a.road));
        let (road, city, region) = found.map_or_else(
            || {
                (
                    crate::addassist::romaji::korean_place(&a.road),
                    crate::addassist::romaji::korean_place(&a.city),
                    crate::addassist::romaji::korean_place(&a.region),
                )
            },
            |f| (f.road_latin, f.city_latin, f.region_latin),
        );
        let number = a.line.trim();
        let mut first = if number.is_empty() {
            road
        } else {
            format!("{number} {road}")
        };
        if !city.is_empty() {
            first.push_str(", ");
            first.push_str(&city);
        }
        return format!("{first}\n{region} {}", a.code).trim().to_owned();
    }
    // The building number with the road, what follows it (a building, a floor) on a line of
    // its own (board K1: 세종대로 175 / 세종문화회관 3층).
    let typed = a.line.trim();
    let (number, rest) = typed
        .split_once(|c: char| c == ',' || c.is_whitespace())
        .map_or((typed, ""), |(n, r)| (n, r.trim()));
    let first = [
        a.region.as_str(),
        a.city.as_str(),
        a.town.as_str(),
        a.road.as_str(),
        number,
    ]
    .into_iter()
    .filter(|s| !s.is_empty())
    .collect::<Vec<_>>()
    .join(" ");
    if rest.is_empty() {
        first
    } else {
        format!("{first}\n{rest}")
    }
}

/// The recipient of this card: its own record, never the European card's free address
/// (owner, 2026-11-15: each card its own record).
fn recipient(text: Option<&PostcardText>) -> Option<Address> {
    text?.recipient(Country::Kr).cloned()
}

/// Draws the boxes from `left`, `top` and, when the code is typed, its digits one in each.
#[allow(clippy::too_many_arguments)]
fn draw_boxes(
    canvas: &mut RgbImage,
    density: f32,
    faces: (&VerticalFace, &VerticalFace),
    left: f32,
    top: f32,
    boxes: &CodeBoxes,
    code: &str,
    ink: [u8; 3],
) {
    crate::postcard::draw_code_boxes(canvas, density, left, top, boxes);
    if code.len() != 5 || !code.chars().all(|c| c.is_ascii_digit()) {
        return;
    }
    let em = boxes.box_mm.1 * 0.8 * density;
    for ((r, _), digit) in boxes.boxes(left, top).iter().zip(code.chars()) {
        let d = digit.to_string();
        let w = line(None, faces, &d, em, 0.0, 0.0, ink);
        line(
            Some(canvas),
            faces,
            &d,
            em,
            (r.x + r.width / 2.0) * density - w / 2.0,
            (r.y + r.height * 0.1) * density,
            ink,
        );
    }
}

/// One line with `track` pixels after each character (「우 편 엽 서」); its width.
#[allow(clippy::too_many_arguments)]
fn tracked(
    mut canvas: Option<&mut RgbImage>,
    faces: (&VerticalFace, &VerticalFace),
    text: &str,
    em: f32,
    left: f32,
    top: f32,
    track: f32,
    color: [u8; 3],
) -> f32 {
    let mut x = left;
    for c in text.chars() {
        x += line(
            canvas.as_deref_mut(),
            faces,
            &c.to_string(),
            em,
            x,
            top,
            color,
        ) + track;
    }
    x - left
}

/// The dotted rule under a name or an address line, from `x0` to `x1` at `y` (millimetres).
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn dotted(canvas: &mut RgbImage, density: f32, x0: f32, x1: f32, y: f32) {
    let px = |v: f32| (v * density).round().max(0.0) as u32;
    let dot = px(0.25).max(1);
    let mut x = x0;
    while x < x1 {
        crate::sheet::fill_rect(canvas, px(x), px(y), dot, dot, RULE);
        x += 0.5;
    }
}

/// The part of `card` the writing may take: short of a machine zone down its right edge (an
/// upright card, the zone being its foot when read turned).
pub fn writing_area(card: RectMm, zone: Option<RectMm>) -> RectMm {
    match zone {
        Some(z) if z.height > z.width && z.x > card.x && z.x < card.x + card.width => RectMm {
            width: z.x - TAPE_MM - card.x,
            ..card
        },
        _ => card,
    }
}

/// Each line of `text` as rows at most `width` pixels wide, each with its em: a line a little
/// too wide set smaller on one row (down to 80 %) rather than a word left alone below.
fn fit_rows(
    faces: (&VerticalFace, &VerticalFace),
    text: &str,
    em: f32,
    width: f32,
) -> Vec<(String, f32)> {
    let mut rows = Vec::new();
    for paragraph in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let w = line(None, faces, paragraph, em, 0.0, 0.0, [0; 3]);
        if w <= width || w * 0.8 <= width {
            rows.push((paragraph.to_owned(), em * (width / w).min(1.0)));
        } else {
            rows.extend(
                wrap(faces, paragraph, em, width)
                    .into_iter()
                    .map(|r| (r, em)),
            );
        }
    }
    rows
}

/// A label set no wider than `width` pixels: its em, smaller for a long name.
fn fit_em(faces: (&VerticalFace, &VerticalFace), text: &str, em: f32, width: f32) -> f32 {
    let w = line(None, faces, text, em, 0.0, 0.0, [0; 3]);
    if w > width { em * width / w } else { em }
}

/// Draws a 우편엽서's writing half onto `canvas`, `density` pixels per millimetre, as board K1
/// within Korea Post's rules; whether anything typed was left out (a row).
#[allow(
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn draw(canvas: &mut RgbImage, density: f32, spec: &KoreaSpec<'_>) -> crate::postcard::Fit {
    let Some(faces) = faces(spec.settings) else {
        return crate::postcard::Fit::default();
    };
    let card = writing_area(spec.card, spec.zone);
    let settings = spec.settings;
    let (outline, _) = crate::postcard::inks(spec.paper);
    let ink = settings.ink.rgb(spec.paper);
    let mm = |v: f32| v * density;
    let em = crate::settings::postcard_text_mm(settings.text_size);
    let stamp = stamp_box(card);
    let mut cut = false;
    // The writing ends above a machine zone at the foot when it crosses this half.
    let limit = spec
        .zone
        .filter(|z| z.width >= z.height && z.y < card.y + card.height)
        .map_or(card.y + card.height, |z| z.y - TAPE_MM);
    // A line's em in its line box of `lh` ems from `top`.
    let em_top = |top: f32, size: f32, lh: f32| top + size * ((lh - 1.0) / 2.0 + EM_IN_BOX);

    crate::postcard::draw_dashed_box(canvas, density, stamp, outline);
    let word_w = line(None, faces, "우표", mm(3.4), 0.0, 0.0, outline);
    line(
        Some(canvas),
        faces,
        "우표",
        mm(3.4),
        mm(stamp.x + stamp.width / 2.0) - word_w / 2.0,
        mm(stamp.y + stamp.height / 2.0 - 1.7),
        outline,
    );

    if settings.postal_use {
        let heading = "우 편 엽 서";
        let track = mm(HEADING_TRACK_MM);
        let w = tracked(None, faces, heading, mm(HEADING_MM), 0.0, 0.0, track, ink);
        let centre = card.x + (card.width - STAMP_MM.0) / 2.0;
        tracked(
            Some(canvas),
            faces,
            heading,
            mm(HEADING_MM),
            mm(centre) - w / 2.0,
            mm(card.y + HEADING_TOP_MM),
            track,
            ink,
        );
    }

    // What the recipient keeps clear of: the stamp box and the sender.
    let mut clear_top = stamp.y + stamp.height + 4.0;
    let mut sender_drawn = false;
    let mut sender_end = card.y;
    // The sender at the top left, left of the stamp box: the label, the name, the address,
    // each over a dotted rule, then its five boxes.
    if let Some(from) = settings
        .sender(Country::Kr)
        .filter(|a| settings.sender_on && !a.is_blank())
    {
        let left = card.x + SENDER_LEFT_MM;
        let right = left + SENDER_WIDTH_MM.min(stamp.x - 2.0 - left);
        let width = mm(right - left);
        let mut y = card.y + SENDER_TOP_MM;
        line(
            Some(canvas),
            faces,
            "보내는 사람",
            mm(LABEL_MM),
            mm(left),
            mm(y),
            GREY,
        );
        y += LABEL_MM * 1.25 + SENDER_GAP_MM;
        let name = from.name.trim();
        if !name.is_empty() {
            let size = em * SENDER_NAME;
            let fitted = fit_em(faces, name, mm(size), width);
            line(
                Some(canvas),
                faces,
                name,
                fitted,
                mm(left),
                mm(em_top(y, size, 1.1)),
                ink,
            );
            y += size * 1.1;
            dotted(canvas, density, left, right, y);
            y += 0.25 + SENDER_GAP_MM;
        }
        let size = em * SENDER_ADDRESS;
        let rows = fit_rows(faces, &address_text(from), mm(size), width);
        cut |= rows.len() > 2;
        for (row, row_em) in rows.iter().take(2) {
            line(
                Some(canvas),
                faces,
                row,
                *row_em,
                mm(left),
                mm(em_top(y, size, 1.2)),
                ink,
            );
            y += size * 1.2;
            dotted(canvas, density, left, right, y);
            y += 0.25 + SENDER_GAP_MM;
        }
        if !from.latin {
            y += 1.0;
            draw_boxes(
                canvas,
                density,
                faces,
                left,
                y,
                &KOREA_SENDER_BOXES,
                &from.code,
                ink,
            );
            y += KOREA_SENDER_BOXES.box_mm.1;
        }
        clear_top = clear_top.max(y + 3.0);
        sender_drawn = true;
        sender_end = y;
    }

    // The recipient at the bottom right, its boxes 4 mm clear above the machine zone and 20 mm
    // or more from the card's right edge (Korea Post): the label, the name on the right, the
    // address, each over a dotted rule, then the boxes; drawn empty for the hand when no one is
    // typed (FR-132).
    let to = recipient(spec.text).unwrap_or_else(|| Address::new(Country::Kr));
    let left = card.x + RECIPIENT_LEFT_MM;
    let right = card.x + card.width - EDGE_MM;
    let width = mm(right - left);
    let mut rows = fit_rows(faces, &address_text(&to), mm(em), width);
    if rows.is_empty() {
        rows.push((String::new(), mm(em)));
    }
    let name_em = em * RECIPIENT_NAME;
    let row_h = em * 1.2 + 0.25 + RECIPIENT_GAP_MM;
    let boxes_h = if to.latin {
        0.0
    } else {
        BOXES_GAP_MM + KOREA_BOXES.height_mm()
    };
    let head = LABEL_MM * 1.25 + RECIPIENT_GAP_MM + name_em * 1.05 + 0.25 + RECIPIENT_GAP_MM;
    // Rows that do not fit above the boxes are left out, never pushed into the zone.
    // The label, on the left, keeps under the sender; the name, on the right, under the stamp
    // box: the block starts where both hold.
    let name_under_stamp = stamp.y + stamp.height + 3.0 - (LABEL_MM * 1.25 + RECIPIENT_GAP_MM);
    let top_bound = name_under_stamp.max(sender_end + 2.0);
    let room = limit - BOXES_CLEAR_MM - top_bound;
    let fit = ((room - head - boxes_h) / row_h).floor().max(1.0) as usize;
    if rows.len() > fit {
        rows.truncate(fit);
        cut = true;
    }
    let block = head + row_h * rows.len() as f32 + boxes_h;
    let mut y = limit - BOXES_CLEAR_MM - block;
    // The 통신란 between the sender (or the stamp box) and the recipient, across, 5 mm clear of
    // both (owner, 2026-11-15).
    // With no sender, its place beside the stamp box is the box's.
    let rect = if sender_drawn {
        RectMm {
            x: card.x + SENDER_LEFT_MM,
            y: clear_top + 1.0,
            width: card.x + card.width - EDGE_MM - (card.x + SENDER_LEFT_MM),
            height: (y - BOX_CLEAR_MM - (clear_top + 1.0)).max(0.0),
        }
    } else {
        RectMm {
            x: card.x + SENDER_LEFT_MM,
            y: card.y + SENDER_TOP_MM,
            width: (stamp.x - BOX_CLEAR_MM - (card.x + SENDER_LEFT_MM)).max(0.0),
            height: (y - BOX_CLEAR_MM - (card.y + SENDER_TOP_MM)).max(0.0),
        }
    };
    let message_cut = message_box(canvas, faces, density, rect, spec);
    line(
        Some(canvas),
        faces,
        "받는 사람",
        mm(LABEL_MM),
        mm(left),
        mm(y),
        GREY,
    );
    y += LABEL_MM * 1.25 + RECIPIENT_GAP_MM;
    let name = to.name.trim();
    if !name.is_empty() {
        let label = if to.honorific && !to.latin {
            format!("{name} 귀하")
        } else {
            name.to_owned()
        };
        let fitted = fit_em(faces, &label, mm(name_em), width);
        let w = line(None, faces, &label, fitted, 0.0, 0.0, ink);
        let top = mm(em_top(y, name_em, 1.05));
        line(Some(canvas), faces, &label, fitted, mm(right) - w, top, ink);
    }
    y += name_em * 1.05;
    dotted(canvas, density, left, right, y);
    y += 0.25 + RECIPIENT_GAP_MM;
    for (row, row_em) in &rows {
        line(
            Some(canvas),
            faces,
            row,
            *row_em,
            mm(left),
            mm(em_top(y, em, 1.2)),
            ink,
        );
        y += em * 1.2;
        dotted(canvas, density, left, right, y);
        y += 0.25 + RECIPIENT_GAP_MM;
    }
    if !to.latin {
        let boxes_right = right.min(spec.card.x + spec.card.width - BOXES_RIGHT_MM);
        let bx = KOREA_BOXES.left_for_right(boxes_right);
        let by = y + BOXES_GAP_MM - RECIPIENT_GAP_MM;
        draw_boxes(canvas, density, faces, bx, by, &KOREA_BOXES, &to.code, ink);
    }
    crate::postcard::Fit {
        message_cut,
        address_cut: cut,
    }
}

/// The 통신란: 5 mm clear of the sender and the recipient; its text the message's size.
const BOX_CLEAR_MM: f32 = 5.0;

/// Draws the 통신란 in `rect` and the message in it; whether some was left out.
fn message_box(
    canvas: &mut RgbImage,
    faces: (&VerticalFace, &VerticalFace),
    density: f32,
    rect: RectMm,
    spec: &KoreaSpec<'_>,
) -> bool {
    let em = crate::settings::postcard_text_mm(spec.settings.text_size) * MESSAGE_SCALE * 0.85;
    let step = em * 1.5;
    let rows = crate::postcard::draw_message_box(canvas, density, rect, step, false, spec.paper);
    let Some(message) = spec.message.map(str::trim).filter(|m| !m.is_empty()) else {
        return false;
    };
    let Some(first) = rows.first() else {
        return true;
    };
    // Rows set closer for the room take a smaller text.
    let em = em.min(first.height / 1.5);
    let mm = |v: f32| v * density;
    let lines: Vec<String> = message
        .lines()
        .flat_map(|p| wrap(faces, p, mm(em), mm(first.width - 1.0)))
        .collect();
    let ink = spec.settings.ink.rgb(spec.paper);
    for (row, text) in rows.iter().zip(&lines) {
        line(
            Some(canvas),
            faces,
            text,
            mm(em),
            mm(row.x + 0.5),
            mm(row.y + row.height - em * 1.1),
            ink,
        );
    }
    lines.len() > rows.len()
}

/// Whether `message` fits the one line under the photo (`band`): one line as typed, no wider.
pub fn message_fits_band(band: RectMm, message: &str, settings: &PostcardSettings) -> bool {
    let Some(faces) = faces(settings) else {
        return false;
    };
    let mut lines = message.lines().map(str::trim).filter(|l| !l.is_empty());
    let (Some(first), None) = (lines.next(), lines.next()) else {
        return message.trim().is_empty();
    };
    // Measured at 10 px a millimetre.
    let em = crate::settings::postcard_text_mm(settings.text_size) * MESSAGE_SCALE * 10.0;
    line(None, faces, first, em, 0.0, 0.0, [0; 3]) <= band.width * 10.0
}

/// The message under a bordered photo (board K1): one line from the photo's left, 19/17 em,
/// 2.5 mm under it, as much as fits (and whether some was left out); `band` starts at the
/// photo's foot.
pub fn draw_message(
    canvas: &mut RgbImage,
    density: f32,
    band: RectMm,
    message: &str,
    settings: &PostcardSettings,
    paper: BorderColor,
) -> bool {
    let Some(faces) = faces(settings) else {
        return false;
    };
    let first = message
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("");
    if first.is_empty() {
        return false;
    }
    // One line is printed: more lines, or more than the width, are left out.
    let mut cut = message.lines().filter(|l| !l.trim().is_empty()).count() > 1;
    let em_mm = crate::settings::postcard_text_mm(settings.text_size) * MESSAGE_SCALE;
    let top = (band.y + MESSAGE_GAP_MM).min(band.y + band.height - em_mm);
    let em = em_mm * density;
    let room = band.width * density;
    let mut shown = String::new();
    for c in first.chars() {
        shown.push(c);
        if line(None, faces, &shown, em, 0.0, 0.0, [0; 3]) > room {
            shown.pop();
            cut = true;
            break;
        }
    }
    line(
        Some(canvas),
        faces,
        &shown,
        em,
        band.x * density,
        top * density,
        settings.ink.rgb(paper),
    );
    cut
}

#[cfg(test)]
mod tests {
    use super::*;

    const DENSITY: f32 = 300.0 / 25.4;

    fn seoul() -> PostcardText {
        let mut to = Address::new(Country::Kr);
        to.code = "03172".into();
        to.region = "서울특별시".into();
        to.city = "종로구".into();
        to.road = "세종대로".into();
        to.line = "175, 3층".into();
        to.name = "홍길동".into();
        PostcardText {
            message: String::new(),
            address: String::new(),
            recipients: vec![to],
        }
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn drawn(
        card: RectMm,
        zone: Option<RectMm>,
        settings: &PostcardSettings,
        text: Option<&PostcardText>,
    ) -> RgbImage {
        let (w, h) = (
            ((card.x + card.width) * DENSITY) as u32,
            ((card.y + card.height).max(102.0) * DENSITY) as u32,
        );
        let mut canvas = RgbImage::from_raw(w, h, vec![255; (w * h * 3) as usize])
            .unwrap_or_else(|| RgbImage::new(1, 1));
        draw(
            &mut canvas,
            DENSITY,
            &KoreaSpec {
                card,
                settings,
                text,
                paper: BorderColor::White,
                zone,
                message: None,
            },
        );
        canvas
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn inked(canvas: &RgbImage, r: RectMm) -> usize {
        let (x0, y0) = ((r.x * DENSITY) as u32, (r.y * DENSITY) as u32);
        let (x1, y1) = (
            ((r.x + r.width) * DENSITY) as u32,
            ((r.y + r.height) * DENSITY) as u32,
        );
        let mut n = 0;
        for y in y0..y1.min(canvas.height) {
            for x in x0..x1.min(canvas.width) {
                let i = ((y * canvas.width + x) * 3) as usize;
                if canvas.data[i..i + 3].iter().all(|c| *c < 200) {
                    n += 1;
                }
            }
        }
        n
    }

    const LANDSCAPE: RectMm = RectMm {
        x: 76.0,
        y: 0.0,
        width: 76.0,
        height: 102.0,
    };
    const ZONE: RectMm = RectMm {
        x: 0.0,
        y: 85.0,
        width: 152.0,
        height: 17.0,
    };

    #[test]
    fn fr_132_the_machine_zone_stays_free_and_the_boxes_are_five() {
        let settings = PostcardSettings {
            face: PostcardFace::NanumPen,
            ..PostcardSettings::default()
        };
        let canvas = drawn(LANDSCAPE, Some(ZONE), &settings, Some(&seoul()));
        let zone_in_half = RectMm {
            x: LANDSCAPE.x,
            y: ZONE.y,
            width: LANDSCAPE.width,
            height: ZONE.height,
        };
        assert_eq!(
            inked(&canvas, zone_in_half),
            0,
            "nothing in the 17 mm machine zone"
        );
        assert_eq!(KOREA_BOXES.boxes(0.0, 0.0).len(), 5);
        // The recipient's block is drawn, low and right.
        let lower = RectMm {
            x: LANDSCAPE.x + 10.0,
            y: 40.0,
            width: 60.0,
            height: 44.0,
        };
        assert!(inked(&canvas, lower) > 2000);
    }

    #[test]
    fn fr_132_the_sender_prints_only_when_its_switch_is_on() {
        // Under 「우편엽서」, left of the 우표 box.
        let corner = RectMm {
            x: LANDSCAPE.x + 4.0,
            y: 6.0,
            width: 44.0,
            height: 27.0,
        };
        let off = drawn(
            LANDSCAPE,
            Some(ZONE),
            &PostcardSettings::default(),
            Some(&seoul()),
        );
        // No sender's text there (its place holds the empty 통신란's grey lines).
        let text_ink = |c: &RgbImage| {
            crate::hagaki::tests::runs(c, corner, false, |p| p.iter().all(|v| *v < 120)).len()
        };
        assert_eq!(text_ink(&off), 0);
        let mut from = Address::new(Country::Kr);
        from.region = "부산광역시".into();
        from.city = "해운대구".into();
        from.road = "해운대해변로".into();
        from.line = "264".into();
        from.name = "김민지".into();
        from.code = "48094".into();
        let on = drawn(
            LANDSCAPE,
            Some(ZONE),
            &PostcardSettings {
                sender_on: true,
                senders: vec![from],
                ..PostcardSettings::default()
            },
            Some(&seoul()),
        );
        assert!(inked(&on, corner) > 500 && text_ink(&on) > 0);
    }

    #[test]
    fn fr_132_every_character_printed_is_in_a_face() {
        for face in [PostcardFace::NanumPen, PostcardFace::Gaegu] {
            let settings = PostcardSettings {
                face,
                ..PostcardSettings::default()
            };
            let Some((main, fallback)) = faces(&settings) else {
                panic!("the Korean faces are bundled");
            };
            for c in "우편엽서 우표 보내는 사람 받는 사람 귀하 서울특별시 종로구 세종대로 175, 3층 #(B) 03172 Sejong-daero"
                .chars()
                .filter(|c| !c.is_whitespace())
            {
                assert!(
                    main.font.glyph_id(c).0 != 0 || fallback.font.glyph_id(c).0 != 0,
                    "{c:?} is in neither face"
                );
            }
        }
    }

    #[test]
    fn fr_132_the_address_reads_in_the_road_name_order() {
        let Some(mut to) = seoul().recipients.into_iter().next() else {
            return;
        };
        assert_eq!(address_text(&to), "서울특별시 종로구 세종대로 175\n3층");
        to.latin = true;
        to.line = "175".into();
        assert_eq!(
            address_text(&to),
            "175 Sejong-daero, Jongno-gu\nSeoul 03172"
        );
    }

    /// Board K1's sample, measured in Chromium in Nanum Pen (scratch `board-measure`).
    fn board_k1() -> (PostcardSettings, PostcardText) {
        let mut to = Address::new(Country::Kr);
        to.code = "03172".into();
        to.region = "서울특별시".into();
        to.city = "종로구".into();
        to.road = "세종대로".into();
        to.line = "175 세종문화회관 3층".into();
        to.name = "홍길동".into();
        let mut from = Address::new(Country::Kr);
        from.code = "48094".into();
        from.region = "부산광역시".into();
        from.city = "해운대구".into();
        from.road = "해운대해변로".into();
        from.line = "264".into();
        from.name = "김민지".into();
        let settings = PostcardSettings {
            face: PostcardFace::NanumPen,
            sender_on: true,
            senders: vec![from],
            ..PostcardSettings::default()
        };
        let text = PostcardText {
            message: String::new(),
            address: String::new(),
            recipients: vec![to],
        };
        (settings, text)
    }

    /// Ink, not the grey dotted rules.
    fn dark(p: [u8; 3]) -> bool {
        p.iter().all(|c| *c < 140)
    }

    #[test]
    fn fr_132_the_landscape_card_matches_board_k1() {
        use crate::hagaki::tests::{assert_board, moved, runs, zone};
        // The board's card: 148 × 100 mm, its writing half from 74 mm, the zone from 83 mm.
        let (settings, text) = board_k1();
        let half = zone(74.0, 0.0, 74.0, 100.0);
        let canvas = drawn(
            half,
            Some(zone(0.0, 83.0, 148.0, 17.0)),
            &settings,
            Some(&text),
        );
        let parts: [(&str, RectMm, &[[f32; 4]]); 3] = [
            (
                "heading",
                zone(80.0, 0.0, 40.0, 5.6),
                &[[91.88, 2.19, 108.69, 4.13]],
            ),
            (
                "sender",
                zone(76.0, 6.0, 45.0, 24.0),
                &[
                    [78.06, 7.56, 86.0, 9.13],
                    [78.06, 11.13, 86.44, 14.38],
                    [78.13, 17.25, 118.75, 20.19],
                    [78.0, 23.25, 98.75, 28.0],
                ],
            ),
            (
                "recipient",
                // Korea Post's rules over the board move the block 12.2 mm up: the words'
                // strip above the zone (6 mm), 6 mm round the larger boxes (5.5 × 7 inside), the
                // code's right end 22 mm from the edge (the Korea Post review, 2026-11-15).
                zone(80.0, 30.0, 66.0, 48.0),
                &[
                    moved([82.63, 44.56, 89.0, 46.13], 0.0, -12.2),
                    moved([121.0, 47.63, 143.0, 54.81], 0.0, -12.2),
                    moved([82.56, 57.25, 117.06, 61.13], 0.0, -12.2),
                    moved([82.56, 63.38, 101.63, 67.25], 0.0, -12.2),
                    [91.5, 62.1, 126.0, 69.7],
                ],
            ),
        ];
        for (part, z, board) in parts {
            assert_board(part, &runs(&canvas, z, false, dark), board, 0.0);
        }
    }

    #[test]
    fn fr_132_the_message_under_the_photo_matches_board_k1() {
        use crate::hagaki::tests::{assert_board, runs, zone};
        // Board K1's photo ends at 78 mm; the band starts there.
        let (settings, _) = board_k1();
        let half = zone(0.0, 0.0, 74.0, 100.0);
        let mut canvas = drawn(half, None, &PostcardSettings::default(), None);
        canvas.data.fill(255);
        let band = zone(5.0, 78.0, 64.0, 8.0);
        draw_message(
            &mut canvas,
            DENSITY,
            band,
            "광화문에서 안부 전해요. 건강하세요!",
            &settings,
            BorderColor::White,
        );
        let line = runs(&canvas, band, false, dark);
        assert_board("message", &line, &[[5.13, 80.81, 53.69, 84.75]], 0.0);
    }

    /// Pixels inside `r` that are not white, any colour.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn any_ink(canvas: &RgbImage, r: RectMm) -> usize {
        let (x0, y0) = ((r.x * DENSITY) as u32, (r.y * DENSITY) as u32);
        let (x1, y1) = (
            ((r.x + r.width) * DENSITY) as u32,
            ((r.y + r.height) * DENSITY) as u32,
        );
        let mut n = 0;
        for y in y0..y1.min(canvas.height) {
            for x in x0..x1.min(canvas.width) {
                let i = ((y * canvas.width + x) * 3) as usize;
                n += usize::from(canvas.data[i..i + 3].iter().any(|c| *c < 250));
            }
        }
        n
    }

    /// An upright card's writing half (102 × 76 mm) and its zone down the right edge.
    const UPRIGHT: RectMm = RectMm {
        x: 0.0,
        y: 0.0,
        width: 102.0,
        height: 76.0,
    };
    const UPRIGHT_ZONE: RectMm = RectMm {
        x: 85.0,
        y: 0.0,
        width: 17.0,
        height: 152.0,
    };

    #[test]
    fn fr_132_the_code_ends_22_mm_from_the_right_edge() {
        // Korea Post: 20 mm clear right of the code, 22 for a kiosk's cut, landscape and upright.
        use crate::hagaki::tests::runs;
        let (settings, text) = board_k1();
        for (half, zone) in [(LANDSCAPE, ZONE), (UPRIGHT, UPRIGHT_ZONE)] {
            let canvas = drawn(half, Some(zone), &settings, Some(&text));
            let low = RectMm {
                y: half.y + half.height * 0.55,
                height: half.height * 0.45,
                ..half
            };
            // The boxes' frames only (their own grey), not the address above them.
            let right = runs(&canvas, low, true, |p| p == KOREA_BOXES.color)
                .iter()
                .map(|b| b[2])
                .fold(0.0, f32::max);
            assert!(
                half.x + half.width - right >= 20.0 - 0.3,
                "{right} in {half:?}"
            );
            // Nothing in an upright card's zone.
            assert_eq!(
                any_ink(
                    &canvas,
                    RectMm {
                        height: half.height,
                        ..zone
                    }
                ),
                0
            );
        }
    }

    #[test]
    fn fr_132_long_text_never_leaves_its_place() {
        // A long name shrinks to its width; five address rows at the large size drop rows
        // rather than push the boxes into the zone; drawn, it says something was left out.
        let (mut settings, mut text) = board_k1();
        settings.text_size = crate::settings::StampSize::Large;
        if let Some(to) = text.recipients.first_mut() {
            to.name = "Nguyễn Thị Phương Thảo Hoàng".into();
            to.line = "175 세종문화회관 대극장 별관 3층 302호 사무국 담당자 앞, 세종로 1가 81-3 옆 건물 2층 안내데스크 옆 우편함, 지하 1층 주차장 입구 맞은편 관리사무소 전달 부탁드립니다".into();
        }
        if let Some(from) = settings.senders.first_mut() {
            from.name = "Nguyễn Thị Phương Thảo Hoàng".into();
        }
        let (w, h) = ((LANDSCAPE.x + LANDSCAPE.width) * DENSITY, 102.0 * DENSITY);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let mut canvas = RgbImage::from_raw(
            w as u32,
            h as u32,
            vec![255; (w as u32 * h as u32 * 3) as usize],
        )
        .unwrap_or_else(|| RgbImage::new(1, 1));
        let spec = KoreaSpec {
            card: LANDSCAPE,
            settings: &settings,
            text: Some(&text),
            paper: BorderColor::White,
            zone: Some(ZONE),
            message: None,
        };
        let cut = draw(&mut canvas, DENSITY, &spec);
        assert!(cut.address_cut, "the rows left out are reported");
        let photo_half = RectMm {
            x: 0.0,
            y: 0.0,
            width: LANDSCAPE.x,
            height: 102.0,
        };
        assert_eq!(
            any_ink(&canvas, photo_half),
            0,
            "nothing runs onto the photo"
        );
        let zone_in_half = RectMm {
            x: LANDSCAPE.x,
            ..ZONE
        };
        assert_eq!(
            any_ink(&canvas, zone_in_half),
            0,
            "nothing in the 17 mm zone"
        );
    }

    #[test]
    fn fr_132_no_recipient_still_draws_its_boxes() {
        // FR-132: five empty boxes to fill by hand.
        let canvas = drawn(LANDSCAPE, Some(ZONE), &PostcardSettings::default(), None);
        let low = RectMm {
            x: LANDSCAPE.x,
            y: 60.0,
            width: LANDSCAPE.width,
            height: 25.0,
        };
        assert!(any_ink(&canvas, low) > 1000);
    }

    #[test]
    fn fr_132_a_vietnamese_name_prints_whole() {
        let settings = PostcardSettings::default();
        let Some((main, _)) = faces(&settings) else {
            panic!("the Korean faces are bundled");
        };
        for c in "Trương Minh Hồng".chars().filter(|c| !c.is_whitespace()) {
            assert!(main.has(c), "{c:?}");
        }
    }
}
