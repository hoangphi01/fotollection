// SPDX-License-Identifier: GPL-3.0-only
//! The font layer every text of the app is drawn through (FR-134): faces, their metrics,
//! glyphs and their coverage, read with `skrifa` (MIT OR Apache-2.0, no `unsafe`) and drawn
//! with `ab_glyph_rasterizer` (Apache-2.0). It keeps the meaning of the `ab_glyph` calls the
//! app was written against, number for number (`examples/text_parity.rs` compares them): a
//! `PxScale` is the line's height, ascent to descent; the metrics are read as `ttf-parser`
//! read them (the typographic ones when the face asks, else `hhea`; advances rounded after
//! `HVAR`; the first Unicode `cmap` subtable that maps a character; the first horizontal `kern`
//! subtable that has a pair); a glyph's bounds span every point of its outline, control points
//! included. The drawing follows `ab_glyph`'s `OutlinedGlyph::draw` (MIT OR Apache-2.0).
use std::sync::Arc;

use ab_glyph_rasterizer::{Point as RPoint, Rasterizer};
use skrifa::instance::{Location, LocationRef, Size};
use skrifa::outline::pen::PathStyle;
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::raw::tables::cmap::PlatformId;
use skrifa::raw::tables::kern::SubtableKind;
use skrifa::raw::tables::os2::SelectionFlags;
use skrifa::raw::{FontRef as RawFont, TableProvider};
use skrifa::{MetadataProvider, Tag};

/// The largest glyph drawn, in pixels a side: a 30 cm print at 600 DPI is 7,087 px long.
const MAX_GLYPH_PX: f32 = 8192.0;
/// And in pixels in all: its coverage is 4 bytes a pixel, 64 MB at most.
const MAX_GLYPH_AREA: f32 = 16_777_216.0;

/// A glyph of a face, 0 the missing glyph.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GlyphId(pub u16);

impl GlyphId {
    pub fn with_scale_and_position(self, scale: impl Into<PxScale>, position: Point) -> Glyph {
        Glyph {
            id: self,
            scale: scale.into(),
            position,
        }
    }

    pub fn with_scale(self, scale: impl Into<PxScale>) -> Glyph {
        self.with_scale_and_position(scale, Point::default())
    }
}

/// A size in pixels: the line's height, ascent to descent, as `ab_glyph`'s.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct PxScale {
    pub x: f32,
    pub y: f32,
}

impl From<f32> for PxScale {
    fn from(s: f32) -> Self {
        Self { x: s, y: s }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

pub const fn point(x: f32, y: f32) -> Point {
    Point { x, y }
}

impl std::ops::Add for Point {
    type Output = Self;
    fn add(self, o: Self) -> Self {
        point(self.x + o.x, self.y + o.y)
    }
}

impl std::ops::Sub for Point {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        point(self.x - o.x, self.y - o.y)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Rect {
    pub min: Point,
    pub max: Point,
}

impl Rect {
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }
}

/// A glyph at a size and a position (its origin on the baseline).
#[derive(Clone, Debug, PartialEq)]
pub struct Glyph {
    pub id: GlyphId,
    pub scale: PxScale,
    pub position: Point,
}

/// The bytes are not a font this layer can read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidFont;

impl std::fmt::Display for InvalidFont {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid font data")
    }
}

impl std::error::Error for InvalidFont {}

#[derive(Debug)]
enum Bytes {
    Static(&'static [u8]),
    Owned(Vec<u8>),
}

impl Bytes {
    fn get(&self) -> &[u8] {
        match self {
            Self::Static(b) => b,
            Self::Owned(b) => b,
        }
    }
}

/// A face's metrics, read once.
#[derive(Debug, Clone, Copy)]
struct Metrics {
    units_per_em: f32,
    ascent: f32,
    descent: f32,
}

#[derive(Debug)]
struct Face {
    bytes: Bytes,
    /// The axes set by `set_variation`, user values, and where they put the face.
    settings: Vec<(Tag, f32)>,
    location: Location,
    metrics: Metrics,
}

impl Face {
    fn new(bytes: Bytes, location: Location) -> Result<Self, InvalidFont> {
        let metrics = {
            let font = RawFont::new(bytes.get()).map_err(|_| InvalidFont)?;
            metrics(&font, &location)?
        };
        Ok(Self {
            bytes,
            settings: Vec::new(),
            location,
            metrics,
        })
    }

    fn font(&self) -> Option<RawFont<'_>> {
        RawFont::new(self.bytes.get()).ok()
    }
}

/// A face's ascent and descent as `ttf-parser` reads them: the typographic ones when
/// `USE_TYPO_METRICS` is set (with `MVAR`), else `hhea`'s, falling back to the typographic and
/// then the Windows ones when `hhea` gives 0. The face must have `head`, `hhea` and `maxp`, as
/// `ttf-parser` asks, and a size per em it accepts.
// The casts are ttf-parser's: an f32 truncated to i16, the Windows metrics read as i16.
// (A method path in place of the closures does not fit `pick`'s higher-ranked signature.)
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::redundant_closure_for_method_calls
)]
fn metrics(font: &RawFont<'_>, location: &Location) -> Result<Metrics, InvalidFont> {
    let head = font.head().map_err(|_| InvalidFont)?;
    let hhea = font.hhea().map_err(|_| InvalidFont)?;
    font.maxp().map_err(|_| InvalidFont)?;
    let upem = head.units_per_em();
    if !(16..=16384).contains(&upem) {
        return Err(InvalidFont);
    }
    let coords = location.coords();
    let mvar = |tag: &[u8; 4], value: i16| -> i16 {
        if coords.is_empty() {
            return value;
        }
        let delta = font
            .mvar()
            .ok()
            .and_then(|m| m.metric_delta(Tag::new(tag), coords))
            .map_or(0.0, |d| d.to_f64() as f32);
        // ttf-parser adds the delta as f32 and truncates back to i16.
        (f32::from(value) + delta) as i16
    };
    let os2 = font.os2().ok();
    let pick = |typo: fn(&skrifa::raw::tables::os2::Os2<'_>) -> i16,
                win: fn(&skrifa::raw::tables::os2::Os2<'_>) -> i16,
                hhea_value: i16,
                t: (&[u8; 4], &[u8; 4])| {
        if let Some(os2) = &os2
            && os2
                .fs_selection()
                .contains(SelectionFlags::USE_TYPO_METRICS)
        {
            return mvar(t.0, typo(os2));
        }
        if hhea_value != 0 {
            return hhea_value;
        }
        let Some(os2) = &os2 else { return 0 };
        let value = typo(os2);
        if value == 0 {
            mvar(t.1, win(os2))
        } else {
            mvar(t.0, value)
        }
    };
    let ascent = pick(
        |o| o.s_typo_ascender(),
        |o| o.us_win_ascent() as i16,
        hhea.ascender().to_i16(),
        (b"hasc", b"hcla"),
    );
    let descent = pick(
        |o| o.s_typo_descender(),
        |o| -(o.us_win_descent() as i16),
        hhea.descender().to_i16(),
        (b"hdsc", b"hcld"),
    );
    Ok(Metrics {
        units_per_em: f32::from(upem),
        ascent: f32::from(ascent),
        descent: f32::from(descent),
    })
}

/// A face shared between threads, with its variation applied.
#[derive(Clone, Debug)]
pub struct FontArc(Arc<Face>);

impl FontArc {
    pub fn new(font: FontRef) -> Self {
        Self(Arc::new(font.0))
    }

    pub fn try_from_vec(bytes: Vec<u8>) -> Result<Self, InvalidFont> {
        Face::new(Bytes::Owned(bytes), Location::default()).map(|f| Self(Arc::new(f)))
    }

    pub fn try_from_slice(bytes: &'static [u8]) -> Result<Self, InvalidFont> {
        Face::new(Bytes::Static(bytes), Location::default()).map(|f| Self(Arc::new(f)))
    }

    /// Every glyph read once (its outline and advance), so a damaged file is refused when it
    /// is loaded rather than when a glyph of it is drawn (SECURITY.md §3); `false` if one
    /// cannot be read.
    pub fn read_whole(&self) -> bool {
        let Some(font) = self.0.font() else {
            return false;
        };
        // Every table the face lists lies inside the file (a file cut short does not).
        if font
            .table_directory()
            .table_records()
            .iter()
            .any(|r| font.table_data(r.tag()).is_none())
        {
            return false;
        }
        let Ok(maxp) = font.maxp() else { return false };
        if font.hmtx().is_err() {
            return false;
        }
        let ids = (0..maxp.num_glyphs()).map(|g| skrifa::GlyphId::new(u32::from(g)));
        // A TrueType face: every glyph's data inside `glyf` (an empty glyph is fine, one that
        // points past the end is not), which skrifa would only show as a glyph with no outline.
        if let (Ok(glyf), Ok(loca)) = (font.glyf(), font.loca(None))
            && ids.clone().any(|id| loca.get_glyf(id, &glyf).is_err())
        {
            return false;
        }
        let outlines = font.outline_glyphs();
        let mut pen = Bounds;
        ids.filter_map(|id| outlines.get(id)).all(|glyph| {
            glyph
                .draw(
                    DrawSettings::unhinted(Size::unscaled(), LocationRef::default()),
                    &mut pen,
                )
                .is_ok()
        })
    }
}

/// A face being set up, before it is shared: its variation can still change.
#[derive(Debug)]
pub struct FontRef(Face);

impl FontRef {
    pub fn try_from_slice(bytes: &'static [u8]) -> Result<Self, InvalidFont> {
        Face::new(Bytes::Static(bytes), Location::default()).map(Self)
    }
}

/// A face with axes: its design space set by axis and user value (`wght` 600).
pub trait VariableFont {
    fn set_variation(&mut self, axis: &[u8; 4], value: f32) -> bool;
}

impl VariableFont for FontRef {
    fn set_variation(&mut self, axis: &[u8; 4], value: f32) -> bool {
        let tag = Tag::new(axis);
        let Some(font) = self.0.font() else {
            return false;
        };
        let design = font.axes();
        if !design.iter().any(|a| a.tag() == tag) {
            return false;
        }
        let mut settings = self.0.settings.clone();
        settings.retain(|(t, _)| *t != tag);
        settings.push((tag, value));
        // Normalised and through `avar` by skrifa, as ttf-parser's `set_variation`.
        let location = design.location(settings.iter().copied());
        let Ok(metrics) = metrics(&font, &location) else {
            return false;
        };
        self.0.settings = settings;
        self.0.location = location;
        self.0.metrics = metrics;
        true
    }
}

/// What the app asks of a face (the names and meanings of `ab_glyph`'s `Font`).
pub trait Font {
    fn units_per_em(&self) -> Option<f32>;
    fn ascent_unscaled(&self) -> f32;
    fn descent_unscaled(&self) -> f32;
    fn height_unscaled(&self) -> f32 {
        self.ascent_unscaled() - self.descent_unscaled()
    }
    fn glyph_id(&self, c: char) -> GlyphId;
    fn h_advance_unscaled(&self, id: GlyphId) -> f32;
    fn kern_unscaled(&self, first: GlyphId, second: GlyphId) -> f32;
    fn outline_glyph(&self, glyph: Glyph) -> Option<OutlinedGlyph>;
    fn as_scaled<S: Into<PxScale>>(&self, scale: S) -> PxScaleFont<'_, Self>
    where
        Self: Sized,
    {
        PxScaleFont {
            font: self,
            scale: scale.into(),
        }
    }
}

impl Font for FontArc {
    fn units_per_em(&self) -> Option<f32> {
        Some(self.0.metrics.units_per_em)
    }

    fn ascent_unscaled(&self) -> f32 {
        self.0.metrics.ascent
    }

    fn descent_unscaled(&self) -> f32 {
        self.0.metrics.descent
    }

    /// The first Unicode `cmap` subtable (Unicode platform, or Windows BMP or full) that maps
    /// the character, as `ttf-parser`; 0 when none does.
    fn glyph_id(&self, c: char) -> GlyphId {
        let Some(cmap) = self.0.font().and_then(|f| f.cmap().ok()) else {
            return GlyphId(0);
        };
        let data = cmap.offset_data();
        cmap.encoding_records()
            .iter()
            .filter(|r| match r.platform_id() {
                PlatformId::Unicode => true,
                PlatformId::Windows => matches!(r.encoding_id(), 1 | 10),
                _ => false,
            })
            .filter_map(|r| r.subtable(data).ok())
            .find_map(|s| s.map_codepoint(c))
            .and_then(|g| u16::try_from(g.to_u32()).ok())
            .map_or(GlyphId(0), GlyphId)
    }

    /// `hmtx`'s advance, plus `HVAR`'s delta on a varied face, rounded as `ttf-parser` does.
    #[allow(clippy::cast_possible_truncation)] // a 16.16 delta, well within f32
    fn h_advance_unscaled(&self, id: GlyphId) -> f32 {
        let Some(font) = self.0.font() else {
            return 0.0;
        };
        let Ok(hmtx) = font.hmtx() else { return 0.0 };
        let long = hmtx.h_metrics();
        let Some(advance) = long
            .get(usize::from(id.0))
            .or_else(|| long.last())
            .map(|m| f32::from(m.advance()))
        else {
            return 0.0;
        };
        let coords = self.0.location.coords();
        if coords.is_empty() {
            return advance;
        }
        let delta = font
            .hvar()
            .ok()
            .and_then(|h| h.advance_delta(skrifa::GlyphId::new(u32::from(id.0)), coords))
            .map(|d| d.to_f64() as f32);
        delta.map_or(advance, |d| {
            (advance + d + 0.5).trunc().clamp(0.0, f32::from(u16::MAX))
        })
    }

    /// The first horizontal, fixed `kern` subtable (formats 0, 2 and 3) holding the pair.
    #[allow(clippy::cast_precision_loss)] // a kerning value, a few thousand units at most
    fn kern_unscaled(&self, first: GlyphId, second: GlyphId) -> f32 {
        let Some(kern) = self.0.font().and_then(|f| f.kern().ok()) else {
            return 0.0;
        };
        let (a, b) = (
            skrifa::GlyphId::new(u32::from(first.0)),
            skrifa::GlyphId::new(u32::from(second.0)),
        );
        kern.subtables()
            .filter_map(Result::ok)
            .filter(|s| s.is_horizontal() && !s.is_variable())
            .find_map(|s| match s.kind().ok()? {
                SubtableKind::Format0(t) => t.kerning(a, b),
                SubtableKind::Format2(t) => t.kerning(a, b),
                SubtableKind::Format3(t) => t.kerning(a, b),
                SubtableKind::Format1(_) => None,
            })
            .map_or(0.0, |v| v as f32)
    }

    fn outline_glyph(&self, glyph: Glyph) -> Option<OutlinedGlyph> {
        let font = self.0.font()?;
        let outline = font
            .outline_glyphs()
            .get(skrifa::GlyphId::new(u32::from(glyph.id.0)))?;
        let mut pen = Curves::default();
        // In font units, deltas kept whole in f32 (the FreeType style rounds them to units),
        // as ttf-parser read them.
        let settings = DrawSettings::unhinted(Size::unscaled(), &self.0.location)
            .with_path_style(PathStyle::HarfBuzz);
        let adjusted = outline.draw(settings, &mut pen).ok()?;
        // The scaler moves every point left by the origin phantom point, so the outline starts
        // at the face's left side bearing; ttf-parser kept the glyph's own coordinates.
        let shift = adjusted.lsb.unwrap_or(0.0);
        let curves = pen.finish();
        let curves = if shift == 0.0 {
            curves
        } else {
            curves.into_iter().map(|c| c.shifted(shift)).collect()
        };
        let (min, max) = bounds(&curves)?;
        let factor = self.as_scaled(glyph.scale).scale_factor();
        let outlined = OutlinedGlyph::new(
            glyph,
            Outline {
                bounds: Rect {
                    min: point(min.x, max.y),
                    max: point(max.x, min.y),
                },
                curves,
            },
            factor,
        );
        // A damaged face can claim a glyph of any size, and its coverage is held whole in
        // memory: none larger than a print can be is drawn (font_mutate, SECURITY.md §3).
        let b = outlined.px_bounds();
        let (w, h) = (b.width(), b.height());
        (w.is_finite()
            && h.is_finite()
            && w <= MAX_GLYPH_PX
            && h <= MAX_GLYPH_PX
            && w * h <= MAX_GLYPH_AREA)
            .then_some(outlined)
    }
}

/// A face at a size: its metrics in pixels (`ab_glyph`'s `ScaleFont`).
#[derive(Clone, Copy, Debug)]
pub struct PxScaleFont<'a, F> {
    font: &'a F,
    scale: PxScale,
}

pub trait ScaleFont {
    fn scale(&self) -> PxScale;
    fn h_scale_factor(&self) -> f32;
    fn v_scale_factor(&self) -> f32;
    fn scale_factor(&self) -> ScaleFactor {
        ScaleFactor {
            horizontal: self.h_scale_factor(),
            vertical: self.v_scale_factor(),
        }
    }
    fn ascent(&self) -> f32;
    fn descent(&self) -> f32;
    fn height(&self) -> f32 {
        self.scale().y
    }
    fn glyph_id(&self, c: char) -> GlyphId;
    fn scaled_glyph(&self, c: char) -> Glyph {
        self.glyph_id(c).with_scale(self.scale())
    }
    fn h_advance(&self, id: GlyphId) -> f32;
    fn kern(&self, first: GlyphId, second: GlyphId) -> f32;
    fn outline_glyph(&self, glyph: Glyph) -> Option<OutlinedGlyph>;
}

impl<F: Font> ScaleFont for PxScaleFont<'_, F> {
    fn scale(&self) -> PxScale {
        self.scale
    }

    fn h_scale_factor(&self) -> f32 {
        self.scale.x / self.font.height_unscaled()
    }

    fn v_scale_factor(&self) -> f32 {
        self.scale.y / self.font.height_unscaled()
    }

    fn ascent(&self) -> f32 {
        self.v_scale_factor() * self.font.ascent_unscaled()
    }

    fn descent(&self) -> f32 {
        self.v_scale_factor() * self.font.descent_unscaled()
    }

    fn glyph_id(&self, c: char) -> GlyphId {
        self.font.glyph_id(c)
    }

    fn h_advance(&self, id: GlyphId) -> f32 {
        self.h_scale_factor() * self.font.h_advance_unscaled(id)
    }

    fn kern(&self, first: GlyphId, second: GlyphId) -> f32 {
        self.h_scale_factor() * self.font.kern_unscaled(first, second)
    }

    fn outline_glyph(&self, glyph: Glyph) -> Option<OutlinedGlyph> {
        self.font.outline_glyph(glyph)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScaleFactor {
    pub horizontal: f32,
    pub vertical: f32,
}

#[derive(Clone, Copy, Debug)]
enum Curve {
    Line(Point, Point),
    Quad(Point, Point, Point),
    Cubic(Point, Point, Point, Point),
}

impl Curve {
    fn shifted(self, dx: f32) -> Self {
        let m = |p: Point| point(p.x + dx, p.y);
        match self {
            Self::Line(a, b) => Self::Line(m(a), m(b)),
            Self::Quad(a, b, c) => Self::Quad(m(a), m(b), m(c)),
            Self::Cubic(a, b, c, d) => Self::Cubic(m(a), m(b), m(c), m(d)),
        }
    }
}

/// The path of an outline in font units, as `ab_glyph`'s `OutlineCurveBuilder` builds it: a
/// contour left open is closed.
#[derive(Default)]
struct Curves {
    last: Point,
    last_move: Option<Point>,
    path: Vec<Curve>,
}

impl Curves {
    fn finish(mut self) -> Vec<Curve> {
        self.close();
        self.path
    }
}

impl OutlinePen for Curves {
    fn move_to(&mut self, x: f32, y: f32) {
        self.last = point(x, y);
        self.last_move = Some(self.last);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let p = point(x, y);
        self.path.push(Curve::Line(self.last, p));
        self.last = p;
    }

    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        let p = point(x, y);
        self.path.push(Curve::Quad(self.last, point(cx, cy), p));
        self.last = p;
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        let p = point(x, y);
        self.path
            .push(Curve::Cubic(self.last, point(cx0, cy0), point(cx1, cy1), p));
        self.last = p;
    }

    fn close(&mut self) {
        if let Some(m) = self.last_move.take() {
            self.path.push(Curve::Line(self.last, m));
        }
    }
}

/// A pen that only checks an outline can be read (`FontArc::read_whole`).
#[derive(Default)]
struct Bounds;

impl OutlinePen for Bounds {
    fn move_to(&mut self, _: f32, _: f32) {}
    fn line_to(&mut self, _: f32, _: f32) {}
    fn quad_to(&mut self, _: f32, _: f32, _: f32, _: f32) {}
    fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {}
    fn close(&mut self) {}
}

/// Every point's extent, control points included, truncated to whole units as `ttf-parser`
/// returns a glyph's box; `None` for an empty or flat outline, which draws nothing.
fn bounds(curves: &[Curve]) -> Option<(Point, Point)> {
    let mut points = curves.iter().flat_map(|c| match *c {
        Curve::Line(a, b) => vec![a, b],
        Curve::Quad(a, b, c) => vec![a, b, c],
        Curve::Cubic(a, b, c, d) => vec![a, b, c, d],
    });
    let first = points.next()?;
    let (mut min, mut max) = (first, first);
    for p in points {
        min = point(min.x.min(p.x), min.y.min(p.y));
        max = point(max.x.max(p.x), max.y.max(p.y));
    }
    let (min, max) = (
        point(min.x.trunc(), min.y.trunc()),
        point(max.x.trunc(), max.y.trunc()),
    );
    (min.x < max.x && min.y < max.y).then_some((min, max))
}

#[derive(Clone, Debug)]
struct Outline {
    /// Font units, `min` the top left (`y` up).
    bounds: Rect,
    curves: Vec<Curve>,
}

/// A glyph's outline placed at its size and position, ready to draw.
#[derive(Clone, Debug)]
pub struct OutlinedGlyph {
    glyph: Glyph,
    px_bounds: Rect,
    factor: ScaleFactor,
    outline: Outline,
}

impl OutlinedGlyph {
    fn new(glyph: Glyph, outline: Outline, factor: ScaleFactor) -> Self {
        let Rect { min, max } = outline.bounds;
        let p = glyph.position;
        let (xt, xf, yt, yf) = (p.x.trunc(), p.x.fract(), p.y.trunc(), p.y.fract());
        let px_bounds = Rect {
            min: point(
                (min.x * factor.horizontal + xf).floor() + xt,
                (min.y * -factor.vertical + yf).floor() + yt,
            ),
            max: point(
                (max.x * factor.horizontal + xf).ceil() + xt,
                (max.y * -factor.vertical + yf).ceil() + yt,
            ),
        };
        Self {
            glyph,
            px_bounds,
            factor,
            outline,
        }
    }

    pub fn glyph(&self) -> &Glyph {
        &self.glyph
    }

    /// The whole pixels the glyph may cover.
    pub fn px_bounds(&self) -> Rect {
        self.px_bounds
    }

    /// Each pixel's coverage, 0 to 1, relative to `px_bounds().min`.
    // The bounds are whole, finite and at most MAX_GLYPH_PX a side (`outline_glyph`).
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn draw<O: FnMut(u32, u32, f32)>(&self, o: O) {
        let (h, v) = (self.factor.horizontal, -self.factor.vertical);
        let offset = self.glyph.position - self.px_bounds.min;
        let at = |p: Point| {
            let q = point(p.x * h, p.y * v) + offset;
            RPoint { x: q.x, y: q.y }
        };
        let mut r = Rasterizer::new(
            self.px_bounds.width() as usize,
            self.px_bounds.height() as usize,
        );
        for curve in &self.outline.curves {
            match *curve {
                Curve::Line(a, b) => r.draw_line(at(a), at(b)),
                Curve::Quad(a, b, c) => r.draw_quad(at(a), at(b), at(c)),
                Curve::Cubic(a, b, c, d) => r.draw_cubic(at(a), at(b), at(c), at(d)),
            }
        }
        r.for_each_pixel_2d(o);
    }
}

#[cfg(test)]
mod tests {
    // Coverage measured as 8-bit levels, bounds as whole pixels, as text_parity wrote them.
    #![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    use super::*;

    /// The bundled faces, Inter at the info frame's three weights, as `text_parity` names them.
    fn faces() -> Vec<(&'static str, FontArc)> {
        let face = |name, bytes: &'static [u8]| {
            (
                name,
                FontArc::try_from_slice(bytes).unwrap_or_else(|e| panic!("{name}: {e}")),
            )
        };
        let inter = |name, weight| {
            let mut font = FontRef::try_from_slice(include_bytes!("../assets/fonts/Inter.ttf"))
                .unwrap_or_else(|e| panic!("Inter: {e}"));
            assert!(font.set_variation(b"wght", weight));
            (name, FontArc::new(font))
        };
        vec![
            face(
                "DSEG14",
                include_bytes!("../assets/fonts/DSEG14Classic-Italic.ttf"),
            ),
            inter("Inter-400", 400.0),
            inter("Inter-600", 600.0),
            inter("Inter-800", 800.0),
            face(
                "XanhMono",
                include_bytes!("../assets/fonts/XanhMono-Regular.ttf"),
            ),
            face(
                "XanhMono-Italic",
                include_bytes!("../assets/fonts/XanhMono-Italic.ttf"),
            ),
            face(
                "PatrickHand",
                include_bytes!("../assets/fonts/PatrickHand-Regular.ttf"),
            ),
            face(
                "KleeOne",
                include_bytes!("../assets/fonts/KleeOne-SemiBold-subset.ttf"),
            ),
            face(
                "Gaegu",
                include_bytes!("../assets/fonts/Gaegu-Regular-subset.ttf"),
            ),
            face(
                "NanumPen",
                include_bytes!("../assets/fonts/NanumPenScript-Regular-subset.ttf"),
            ),
        ]
    }

    fn fixture(name: &str, ext: &str) -> Vec<u8> {
        let path = format!(
            "{}/tests/fixtures/text/{name}.{ext}",
            env!("CARGO_MANIFEST_DIR")
        );
        std::fs::read(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
    }

    fn coverage(bytes: &[u8]) -> std::collections::BTreeMap<String, Vec<u8>> {
        let mut map = std::collections::BTreeMap::new();
        let mut at = 0;
        let mut take = |n: usize| {
            let s = bytes.get(at..at + n).map(<[u8]>::to_vec);
            at += n;
            s
        };
        while let Some(len) = take(4) {
            let len = u32::from_le_bytes(len.try_into().unwrap_or_default()) as usize;
            let key = String::from_utf8(take(len).unwrap_or_default()).unwrap_or_default();
            let n = u32::from_le_bytes(take(4).unwrap_or_default().try_into().unwrap_or_default())
                as usize;
            map.insert(key, take(n).unwrap_or_default());
        }
        map
    }

    #[test]
    fn fr_134_text_renders_as_before() {
        // The glyphs every text is drawn from, against those ab_glyph and ttf-parser drew
        // (tests/fixtures/text, written by examples/text_parity.rs before FR-134): the faces'
        // metrics, every character's glyph, advance and kerning, and at 24 px its bounds and
        // coverage. Identical, but Inter between its masters, whose gvar deltas read-fonts
        // interpolates in 16.16 fixed point where ttf-parser used f32: at most one 8-bit level.
        for (name, font) in faces() {
            let facts = String::from_utf8(fixture(name, "txt")).unwrap_or_default();
            let before = coverage(&fixture(name, "cov24"));
            let mut lines = facts.lines();
            let face = format!(
                "face upem={:?} ascent={:?} descent={:?} height={:?}",
                font.units_per_em(),
                font.ascent_unscaled(),
                font.descent_unscaled(),
                font.height_unscaled()
            );
            assert_eq!(lines.next(), Some(face.as_str()), "{name}");
            let chars: Vec<char> = facts
                .lines()
                .filter(|l| l.contains(" id="))
                .filter_map(|l| {
                    u32::from_str_radix(l.get(2..l.find(' ')?)?, 16)
                        .ok()
                        .and_then(char::from_u32)
                })
                .collect();
            let tolerance = u8::from(name == "Inter-600");
            let (mut worst, mut drawn) = (0u8, 0usize);
            for (i, &c) in chars.iter().enumerate() {
                let id = font.glyph_id(c);
                let next = font.glyph_id(chars[(i + 1) % chars.len()]);
                let line = format!(
                    "U+{:04X} id={} advance={:?} kern={:?}",
                    c as u32,
                    id.0,
                    font.h_advance_unscaled(id),
                    font.kern_unscaled(id, next)
                );
                assert_eq!(lines.next(), Some(line.as_str()), "{name}");
                let scaled = font.as_scaled(PxScale::from(24.0));
                let glyph = id.with_scale_and_position(PxScale::from(24.0), point(0.0, 100.0));
                let outlined = font.outline_glyph(glyph);
                let bounds = outlined.as_ref().map_or_else(
                    || " none".to_string(),
                    |o| {
                        let b = o.px_bounds();
                        format!(" [{:?},{:?},{:?},{:?}]", b.min.x, b.min.y, b.max.x, b.max.y)
                    },
                );
                let line = format!(
                    "U+{:04X} px=24 ascent={:?} descent={:?} advance={:?}{bounds}",
                    c as u32,
                    scaled.ascent(),
                    scaled.descent(),
                    scaled.h_advance(id)
                );
                assert_eq!(lines.next(), Some(line.as_str()), "{name}");
                let Some(outlined) = outlined else { continue };
                let w = outlined.px_bounds().width() as usize;
                let mut levels = vec![0u8; w * outlined.px_bounds().height() as usize];
                outlined.draw(|x, y, v| {
                    if let Some(slot) = levels.get_mut(y as usize * w + x as usize) {
                        *slot = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                    }
                });
                let key = format!("U+{:04X} 24 0", c as u32);
                let was = before
                    .get(&key)
                    .unwrap_or_else(|| panic!("{name} {key}: not in the fixture"));
                assert_eq!(was.len(), levels.len(), "{name} {key}");
                worst = worst.max(
                    was.iter()
                        .zip(&levels)
                        .map(|(a, b)| a.abs_diff(*b))
                        .max()
                        .unwrap_or(0),
                );
                drawn += 1;
            }
            assert!(
                worst <= tolerance,
                "{name}: coverage differs by {worst} levels"
            );
            assert!(drawn > 100, "{name}: only {drawn} glyphs drawn");
        }
    }

    #[test]
    fn fr_134_a_damaged_font_is_refused() {
        // Not a font, or its tables cut away: refused when loaded. A font cut short after its
        // tables: refused by the whole read fonts::load makes (its glyphs point past the end).
        // Bytes overwritten in its glyphs: every glyph drawn without a panic, whatever it reads.
        let inter = include_bytes!("../assets/fonts/Inter.ttf");
        assert!(FontArc::try_from_vec(b"OTTO not a font".to_vec()).is_err());
        assert!(FontArc::try_from_vec(inter[..64].to_vec()).is_err());
        assert!(FontArc::try_from_slice(inter).is_ok_and(|f| f.read_whole()));
        let cut = FontArc::try_from_vec(inter[..inter.len() * 6 / 10].to_vec());
        assert!(
            cut.is_err() || cut.is_ok_and(|f| !f.read_whole()),
            "a font cut to 60 % reads whole"
        );
        let mut damaged = inter.to_vec();
        let middle = damaged.len() / 2;
        damaged[middle..middle + 4096].fill(0xFF);
        if let Ok(font) = FontArc::try_from_vec(damaged) {
            let _ = font.read_whole();
            for c in ['A', 'g', 'ư', 'Ж'] {
                let glyph = font
                    .glyph_id(c)
                    .with_scale_and_position(PxScale::from(40.0), point(0.0, 40.0));
                if let Some(o) = font.outline_glyph(glyph) {
                    o.draw(|_, _, _| {});
                }
            }
        }
    }
}
