// SPDX-License-Identifier: GPL-3.0-only
//! The 우편엽서's address side (FR-132): Korea Post's rules on the writing half of a Korean
//! card (board K1, owner 2026-11-15), its photo on the other half. 「우편엽서」 centred at the
//! top, the sender at the top left (보내는 사람, its own five boxes) when its switch is on, the
//! 우표 box at the top right with `fotollection.org/ko` under it, the recipient at the bottom
//! right (받는 사람, the name the largest, 귀하 when chosen, the address in the road-name
//! order, the five boxes right-aligned with 4 mm clear round them), the machine zone's 17 mm
//! at the card's foot kept free. A square card (4 × 4 in) sets the same on its top half.

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
/// The stamp box and the card's margins.
pub const STAMP_MM: (f32, f32) = (20.0, 25.0);
const EDGE_MM: f32 = 5.0;
/// Clear round the recipient's code boxes (FR-132: 4 mm).
const CODE_CLEAR_MM: f32 = 4.0;
/// The small labels (보내는 사람, 받는 사람) and 「우편엽서」.
const LABEL_MM: f32 = 2.4;
const HEADING_MM: f32 = 2.8;
const GREY: [u8; 3] = [110, 110, 110];

/// What a 우편엽서's writing half draws.
pub struct KoreaSpec<'a> {
    pub card: RectMm,
    pub settings: &'a PostcardSettings,
    pub text: Option<&'a PostcardText>,
    pub paper: BorderColor,
    /// The machine zone, when it crosses the writing half (a landscape card's foot).
    pub zone: Option<RectMm>,
    /// The message to set on the writing half, between the sender and the recipient (a
    /// landscape card whose photo fills its half has no band under it).
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

/// An address in the road-name order: 시·도 시·군·구 읍·면 도로명, then the typed line (건물번호,
/// 상세주소); in Latin letters the international order, the code last.
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
    [
        a.region.as_str(),
        a.city.as_str(),
        a.town.as_str(),
        a.road.as_str(),
        a.line.trim(),
    ]
    .into_iter()
    .filter(|s| !s.is_empty())
    .collect::<Vec<_>>()
    .join(" ")
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

/// Draws a 우편엽서's writing half onto `canvas`, `density` pixels per millimetre.
#[allow(clippy::too_many_lines, clippy::cast_precision_loss)]
pub fn draw(canvas: &mut RgbImage, density: f32, spec: &KoreaSpec<'_>) {
    let Some(faces) = faces(spec.settings) else {
        return;
    };
    let card = spec.card;
    let settings = spec.settings;
    let (outline, _) = crate::postcard::inks(spec.paper);
    let ink = settings.ink.rgb(spec.paper);
    let mm = |v: f32| v * density;
    let em = crate::settings::postcard_text_mm(settings.text_size);
    let stamp = stamp_box(card);
    // The writing ends above the machine zone when the zone crosses this half.
    let bottom = spec
        .zone
        .filter(|z| z.y < card.y + card.height)
        .map_or(card.y + card.height, |z| z.y)
        - 2.0;

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
        let w = line(None, faces, heading, mm(HEADING_MM), 0.0, 0.0, ink);
        let centre = card.x + (card.width - STAMP_MM.0 - EDGE_MM) / 2.0;
        line(
            Some(canvas),
            faces,
            heading,
            mm(HEADING_MM),
            mm(centre) - w / 2.0,
            mm(card.y + 1.5),
            ink,
        );
    }

    // Where the message may go: under the stamp box and the sender, above the recipient.
    let mut message_top = stamp.y + stamp.height + 4.0;
    let mut message_bottom = bottom;
    // The sender at the top left, left of the stamp box.
    if settings.sender_on
        && let Some(from) = settings.sender.as_ref().filter(|a| !a.is_blank())
    {
        let left = card.x + EDGE_MM;
        let width = stamp.x - 3.0 - left;
        let mut y = card.y + EDGE_MM + 1.0;
        line(
            Some(canvas),
            faces,
            "보내는 사람",
            mm(LABEL_MM),
            mm(left),
            mm(y),
            GREY,
        );
        y += LABEL_MM * 1.4;
        let small = em * 0.8;
        if !from.name.trim().is_empty() {
            line(
                Some(canvas),
                faces,
                from.name.trim(),
                mm(small * 1.15),
                mm(left),
                mm(y),
                ink,
            );
            y += small * 1.15 * 1.3;
        }
        for row in wrap(faces, &address_text(from), mm(small), mm(width))
            .iter()
            .take(2)
        {
            line(Some(canvas), faces, row, mm(small), mm(left), mm(y), ink);
            y += small * 1.25;
        }
        if !from.latin {
            draw_boxes(
                canvas,
                density,
                faces,
                left,
                y + 0.8,
                &KOREA_SENDER_BOXES,
                &from.code,
                ink,
            );
            y += KOREA_SENDER_BOXES.box_mm.1 + 1.0;
        }
        message_top = message_top.max(y + 3.0);
    }

    // The recipient at the bottom right: the label, the name, the address, the boxes.
    if let Some(to) = recipient(spec.text) {
        let left = card.x + card.width * 0.16;
        let right = card.x + card.width - EDGE_MM;
        let width = right - left;
        let boxes_h = if to.latin {
            0.0
        } else {
            KOREA_BOXES.box_mm.1 + CODE_CLEAR_MM
        };
        let address = wrap(faces, &address_text(&to), mm(em), mm(width));
        let name_em = em * 1.5;
        let block = LABEL_MM * 1.4 + name_em * 1.25 + em * 1.3 * address.len() as f32 + boxes_h;
        let mut y = (bottom - block).max(stamp.y + stamp.height + 3.0);
        message_bottom = y - 2.0;
        line(
            Some(canvas),
            faces,
            "받는 사람",
            mm(LABEL_MM),
            mm(left),
            mm(y),
            GREY,
        );
        y += LABEL_MM * 1.4;
        let name = to.name.trim();
        if !name.is_empty() {
            let label = if to.honorific && !to.latin {
                format!("{name} 귀하")
            } else {
                name.to_owned()
            };
            let w = line(None, faces, &label, mm(name_em), 0.0, 0.0, ink);
            line(
                Some(canvas),
                faces,
                &label,
                mm(name_em),
                mm(right) - w,
                mm(y),
                ink,
            );
        }
        y += name_em * 1.25;
        for row in &address {
            line(Some(canvas), faces, row, mm(em), mm(left), mm(y), ink);
            y += em * 1.3;
        }
        if !to.latin {
            let bx = right - KOREA_BOXES.width_mm();
            draw_boxes(
                canvas,
                density,
                faces,
                bx,
                y + CODE_CLEAR_MM / 2.0,
                &KOREA_BOXES,
                &to.code,
                ink,
            );
        }
    }
    // The message, when the photo has no band for it, in the room between.
    if let Some(message) = spec.message.filter(|m| !m.trim().is_empty()) {
        let small = em * 0.85;
        let left = card.x + EDGE_MM;
        let mut y = message_top;
        'lines: for paragraph in message.lines() {
            for row in wrap(faces, paragraph, mm(small), mm(card.width - 2.0 * EDGE_MM)) {
                if y + small > message_bottom {
                    break 'lines;
                }
                line(Some(canvas), faces, &row, mm(small), mm(left), mm(y), ink);
                y += small * 1.3;
            }
        }
    }
}

/// The message on a bordered photo's half (board K1): one line across the band under the photo,
/// as much of it as fits, in the card's face.
pub fn draw_message(
    canvas: &mut RgbImage,
    density: f32,
    band: RectMm,
    message: &str,
    settings: &PostcardSettings,
    paper: BorderColor,
) {
    let Some(faces) = faces(settings) else {
        return;
    };
    let first = message
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("");
    if first.is_empty() {
        return;
    }
    let em =
        (band.height * 0.6).min(crate::settings::postcard_text_mm(settings.text_size)) * density;
    let room = (band.width - 4.0) * density;
    let mut shown = String::new();
    for c in first.chars() {
        shown.push(c);
        if line(None, faces, &shown, em, 0.0, 0.0, [0; 3]) > room {
            shown.pop();
            break;
        }
    }
    line(
        Some(canvas),
        faces,
        &shown,
        em,
        (band.x + 2.0) * density,
        band.y * density + (band.height * density - em) / 2.0,
        settings.ink.rgb(paper),
    );
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
        assert_eq!(inked(&off, corner), 0);
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
                sender: Some(from),
                ..PostcardSettings::default()
            },
            Some(&seoul()),
        );
        assert!(inked(&on, corner) > 500);
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
        assert_eq!(address_text(&to), "서울특별시 종로구 세종대로 175, 3층");
        to.latin = true;
        to.line = "175".into();
        assert_eq!(
            address_text(&to),
            "175 Sejong-daero, Jongno-gu\nSeoul 03172"
        );
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
