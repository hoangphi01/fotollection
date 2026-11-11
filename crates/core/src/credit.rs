// SPDX-License-Identifier: GPL-3.0-only
//! The small "fotollection.org" on a print (FR-121): on by default, off with one switch in
//! Settings, on prints only, never on a photo exported for sharing.
//!
//! It goes where it costs the photo least, in this order: the paper round the photo (the
//! 3 mm border, the info frame's bar, a sheet's margin, beside a postcard's stamp box), then a
//! corner of the photo away from the date stamp and any face the best-shot analysis found.
//! Its size is in millimetres on the print, so it reads the same at 10 × 15 and 20 × 30. On
//! paper it is a quiet grey; on the photo a half-clear white with a soft shadow under it.
//! Where no place is free it is left out rather than drawn over something.

use ab_glyph::{Font, FontArc, PxScale, ScaleFont, point};

use crate::image::RgbImage;
use crate::settings::Corner;
use crate::text::{TextRun, draw_text, text_width};

/// What is printed.
pub const TEXT: &str = "fotollection.org";
/// Its em on the print: about 4.5 pt, so its letters stand about 1.2 mm tall.
pub const EM_MM: f32 = 1.6;
const TRACKING: f32 = 0.02;
/// On the photo: the text's opacity, and its shadow's.
const ON_PHOTO_ALPHA: f32 = 0.6;
const SHADOW_ALPHA: f32 = 0.35;
/// The room kept round the stamp and faces, in millimetres.
const CLEARANCE_MM: f32 = 1.0;

/// A box in canvas pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoxF {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl BoxF {
    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.x < other.right()
            && other.x < self.right()
            && self.y < other.bottom()
            && other.y < self.bottom()
    }

    pub fn contains(&self, other: &Self) -> bool {
        other.x >= self.x
            && other.y >= self.y
            && other.right() <= self.right()
            && other.bottom() <= self.bottom()
    }

    /// The box grown by `by` on every side.
    #[must_use]
    pub fn grown(&self, by: f32) -> Self {
        Self {
            x: self.x - by,
            y: self.y - by,
            w: self.w + 2.0 * by,
            h: self.h + 2.0 * by,
        }
    }
}

/// The credit as it is drawn: its text's top-left corner, size and colour, in canvas pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CreditSpec {
    pub left: f32,
    pub top: f32,
    pub font_px: f32,
    pub color: [u8; 3],
    pub alpha: f32,
    /// On the photo: a soft dark shadow under the letters.
    pub shadow: bool,
}

impl CreditSpec {
    /// The box its ink covers.
    pub fn ink(&self) -> Option<BoxF> {
        let ink = Ink::of(self.font_px)?;
        Some(BoxF {
            x: self.left,
            y: self.top + ink.ascent - ink.above,
            w: ink.width,
            h: ink.above + ink.below,
        })
    }
}

/// How the text sits in a place: centred top to bottom (a strip of paper) or in a corner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fit {
    Centred,
    Corner,
}

/// One place the credit may go.
#[derive(Debug, Clone, PartialEq)]
pub struct Place {
    pub area: BoxF,
    /// The paper's colour, or `None` on the photo.
    pub paper: Option<[u8; 3]>,
    /// The corners to try, in order (with `Fit::Centred`, only left or right counts).
    pub corners: Vec<Corner>,
    pub fit: Fit,
    /// Room kept from the area's edges, across and down.
    pub inset_x: f32,
    pub inset_y: f32,
}

/// The text's ink, measured once for a size: its width, how far it rises above the baseline
/// and drops below it, and the font's ascent (the baseline's distance from the text's top).
struct Ink {
    width: f32,
    above: f32,
    below: f32,
    ascent: f32,
}

impl Ink {
    fn of(px: f32) -> Option<Self> {
        let font = font()?;
        let scale = PxScale::from(px);
        let scaled = font.as_scaled(scale);
        let (mut above, mut below) = (0.0f32, 0.0f32);
        let mut caret = 0.0;
        for c in TEXT.chars() {
            let glyph = scaled
                .glyph_id(c)
                .with_scale_and_position(scale, point(caret, 0.0));
            caret += scaled.h_advance(glyph.id) + TRACKING * px;
            if let Some(outline) = font.outline_glyph(glyph) {
                let b = outline.px_bounds();
                above = above.max(-b.min.y);
                below = below.max(b.max.y);
            }
        }
        Some(Self {
            width: text_width(font, px, TEXT, TRACKING),
            above,
            below,
            ascent: scaled.ascent(),
        })
    }
}

fn font() -> Option<&'static FontArc> {
    crate::render::regular()
}

/// A quiet grey on paper of colour `paper`: darker on light paper, lighter on dark.
fn grey_on(paper: [u8; 3]) -> [u8; 3] {
    let lum =
        0.2126 * f32::from(paper[0]) + 0.7152 * f32::from(paper[1]) + 0.0722 * f32::from(paper[2]);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    paper.map(|c| {
        let c = f32::from(c);
        let v = if lum > 128.0 {
            c * 0.55
        } else {
            c + (255.0 - c) * 0.45
        };
        v.round().clamp(0.0, 255.0) as u8
    })
}

/// The first place, in order, where the credit fits whole and clear of everything in
/// `avoid`; `None` when there is none. `px_per_mm` is the print's density.
pub fn place(px_per_mm: f32, places: &[Place], avoid: &[BoxF]) -> Option<CreditSpec> {
    if !(px_per_mm.is_finite() && px_per_mm > 0.0) {
        return None;
    }
    let font_px = EM_MM * px_per_mm;
    let ink = Ink::of(font_px)?;
    let height = ink.above + ink.below;
    let clear = CLEARANCE_MM * px_per_mm;
    let avoid: Vec<BoxF> = avoid.iter().map(|b| b.grown(clear)).collect();
    for place in places {
        let a = place.area;
        for corner in &place.corners {
            let right = matches!(corner, Corner::BottomRight | Corner::TopRight);
            let x = if right {
                a.right() - place.inset_x - ink.width
            } else {
                a.x + place.inset_x
            };
            let y = match (place.fit, corner) {
                (Fit::Centred, _) => a.y + (a.h - height) / 2.0,
                (Fit::Corner, Corner::BottomRight | Corner::BottomLeft) => {
                    a.bottom() - place.inset_y - height
                }
                (Fit::Corner, Corner::TopRight | Corner::TopLeft) => a.y + place.inset_y,
            };
            let box_ = BoxF {
                x,
                y,
                w: ink.width,
                h: height,
            };
            let inside = a.contains(&box_)
                && (place.fit == Fit::Corner || height + 2.0 * place.inset_y <= a.h);
            if !inside || avoid.iter().any(|b| b.intersects(&box_)) {
                continue;
            }
            let (color, alpha, shadow) = match place.paper {
                Some(paper) => (grey_on(paper), 1.0, false),
                None => ([255, 255, 255], ON_PHOTO_ALPHA, true),
            };
            return Some(CreditSpec {
                left: x,
                top: y - (ink.ascent - ink.above),
                font_px,
                color,
                alpha,
                shadow,
            });
        }
    }
    None
}

/// The corners of the photo to try: the bottom corner away from the date stamp first, then
/// the other bottom one, then the top ones.
pub fn photo_corners(stamp: Option<Corner>) -> Vec<Corner> {
    match stamp {
        Some(Corner::BottomRight) => vec![
            Corner::BottomLeft,
            Corner::BottomRight,
            Corner::TopLeft,
            Corner::TopRight,
        ],
        _ => vec![
            Corner::BottomRight,
            Corner::BottomLeft,
            Corner::TopRight,
            Corner::TopLeft,
        ],
    }
}

/// Draws the credit.
pub fn draw(image: &mut RgbImage, spec: &CreditSpec) {
    let Some(font) = font() else {
        return;
    };
    let run = TextRun {
        font,
        text: TEXT,
        px: spec.font_px,
        tracking: TRACKING,
    };
    if spec.shadow {
        let offset = (spec.font_px * 0.06).max(1.0);
        draw_text(
            image,
            &run,
            spec.left + offset,
            spec.top + offset,
            [0, 0, 0],
            SHADOW_ALPHA,
        );
    }
    draw_text(image, &run, spec.left, spec.top, spec.color, spec.alpha);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strip(paper: [u8; 3]) -> Place {
        Place {
            area: BoxF {
                x: 0.0,
                y: 900.0,
                w: 1500.0,
                h: 36.0,
            },
            paper: Some(paper),
            corners: vec![Corner::BottomRight],
            fit: Fit::Centred,
            inset_x: 0.0,
            inset_y: 0.0,
        }
    }

    #[test]
    fn fr_121_grey_is_darker_on_white_and_lighter_on_black() {
        assert!(grey_on([250, 250, 249])[0] < 160);
        assert!(grey_on([13, 13, 13])[0] > 100);
    }

    #[test]
    fn fr_121_the_credit_sits_inside_its_strip_and_skips_what_it_must_avoid() {
        // 12 px a millimetre: a 3 mm strip holds the 1.6 mm text.
        let spec = place(12.0, &[strip([250, 250, 249])], &[]);
        let ink = spec.and_then(|s| s.ink());
        assert!(
            ink.is_some_and(|i| strip([0; 3]).area.contains(&i)),
            "{ink:?}"
        );
        // Something in the way: nowhere else to go, so none.
        let avoid = [BoxF {
            x: 1400.0,
            y: 905.0,
            w: 50.0,
            h: 10.0,
        }];
        assert_eq!(place(12.0, &[strip([250, 250, 249])], &avoid), None);
    }

    #[test]
    fn fr_121_on_the_photo_it_takes_the_bottom_corner_away_from_the_stamp() {
        let photo = Place {
            area: BoxF {
                x: 0.0,
                y: 0.0,
                w: 1800.0,
                h: 1200.0,
            },
            paper: None,
            corners: photo_corners(Some(Corner::BottomRight)),
            fit: Fit::Corner,
            inset_x: 36.0,
            inset_y: 36.0,
        };
        let spec = place(12.0, std::slice::from_ref(&photo), &[]);
        assert!(spec.is_some_and(|s| s.shadow && s.left < 100.0 && s.top > 1000.0));
    }
}
