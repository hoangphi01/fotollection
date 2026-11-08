// SPDX-License-Identifier: GPL-3.0-only
//! Lightroom® presets read into looks (FR-089): an `.xmp` develop preset
//! (`crs:PresetType="Normal"`), a legacy `.lrtemplate`, or a `.zip` of them. Every file is
//! untrusted: at most [`MAX_BYTES`], no DTD (so no entity), nesting at most [`MAX_DEPTH`],
//! an `.lrtemplate` read by a small parser of Lua's table constructors and never run, only
//! the settings of [`KEYS`] kept, every number finite. Read only in Rust, from files the user
//! chose in a Rust-side dialog (SECURITY.md §3). No Adobe code or file is used: the names
//! below are the ones the files themselves hold.

use std::collections::BTreeMap;
use std::io::Read;

use quick_xml::events::Event;
use quick_xml::name::ResolveResult;
use quick_xml::{NsReader, XmlVersion};

/// Largest preset file read; Lightroom's are a few kilobytes.
pub const MAX_BYTES: usize = 1024 * 1024;
/// Deepest nesting of elements or tables.
pub const MAX_DEPTH: usize = 32;
/// Largest `.zip` of presets, its entries and their unpacked size.
pub const MAX_ZIP_BYTES: usize = 20 * 1024 * 1024;
pub const MAX_ZIP_ENTRIES: usize = 200;

const CRS: &str = "http://ns.adobe.com/camera-raw-settings/1.0/";
const RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";

/// The settings read, by the name the files give them: those a look can take, and those it
/// cannot but names after an import (FR-089: listed, never silently dropped).
pub const KEYS: &[&str] = &[
    "Exposure2012",
    "Contrast2012",
    "Highlights2012",
    "Shadows2012",
    "Whites2012",
    "Blacks2012",
    "Saturation",
    "Vibrance",
    "IncrementalTemperature",
    "IncrementalTint",
    "Temperature",
    "Tint",
    "WhiteBalance",
    "HueAdjustmentRed",
    "HueAdjustmentOrange",
    "HueAdjustmentYellow",
    "HueAdjustmentGreen",
    "HueAdjustmentAqua",
    "HueAdjustmentBlue",
    "HueAdjustmentPurple",
    "HueAdjustmentMagenta",
    "SaturationAdjustmentRed",
    "SaturationAdjustmentOrange",
    "SaturationAdjustmentYellow",
    "SaturationAdjustmentGreen",
    "SaturationAdjustmentAqua",
    "SaturationAdjustmentBlue",
    "SaturationAdjustmentPurple",
    "SaturationAdjustmentMagenta",
    "LuminanceAdjustmentRed",
    "LuminanceAdjustmentOrange",
    "LuminanceAdjustmentYellow",
    "LuminanceAdjustmentGreen",
    "LuminanceAdjustmentAqua",
    "LuminanceAdjustmentBlue",
    "LuminanceAdjustmentPurple",
    "LuminanceAdjustmentMagenta",
    "ColorGradeShadowHue",
    "ColorGradeShadowSat",
    "ColorGradeShadowLum",
    "ColorGradeMidtoneHue",
    "ColorGradeMidtoneSat",
    "ColorGradeMidtoneLum",
    "ColorGradeHighlightHue",
    "ColorGradeHighlightSat",
    "ColorGradeHighlightLum",
    "ColorGradeGlobalHue",
    "ColorGradeGlobalSat",
    "ColorGradeGlobalLum",
    "ColorGradeBlending",
    "ColorGradeBalance",
    "SplitToningShadowHue",
    "SplitToningShadowSaturation",
    "SplitToningHighlightHue",
    "SplitToningHighlightSaturation",
    "SplitToningBalance",
    "ConvertToGrayscale",
    "GrayMixerRed",
    "GrayMixerOrange",
    "GrayMixerYellow",
    "GrayMixerGreen",
    "GrayMixerAqua",
    "GrayMixerBlue",
    "GrayMixerPurple",
    "GrayMixerMagenta",
    "GrainAmount",
    "GrainSize",
    "GrainFrequency",
    "PostCropVignetteAmount",
    "PostCropVignetteMidpoint",
    "ParametricShadows",
    "ParametricDarks",
    "ParametricLights",
    "ParametricHighlights",
    "ParametricShadowSplit",
    "ParametricMidtoneSplit",
    "ParametricHighlightSplit",
    "Clarity2012",
    "Texture",
    "Dehaze",
    "Sharpness",
    "LuminanceSmoothing",
    "ColorNoiseReduction",
    "LensProfileEnable",
    "ToneCurvePV2012",
    "ToneCurvePV2012Red",
    "ToneCurvePV2012Green",
    "ToneCurvePV2012Blue",
];

/// One setting's value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f32),
    Text(String),
    /// A point curve, 0–255 on both axes, in order.
    Curve(Vec<(f32, f32)>),
}

/// A preset as read: its name and group when it gives them, and its settings.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Preset {
    pub name: Option<String>,
    pub group: Option<String>,
    /// `crs:ProcessVersion`: older than 6.7 (Lightroom's 2012 process) maps only roughly.
    pub process: Option<String>,
    pub values: BTreeMap<String, Value>,
}

/// Why a preset file was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PresetError {
    #[error("the file is larger than 1 MB")]
    TooLarge,
    #[error("the file declares a DTD, which a preset never needs")]
    Dtd,
    #[error("the file nests deeper than {MAX_DEPTH} levels")]
    TooDeep,
    #[error("not a develop preset: {0}")]
    NotPreset(String),
    #[error("the file cannot be read: {0}")]
    Invalid(String),
    #[error("the preset holds no setting a look can take")]
    Empty,
}

impl PresetError {
    /// The stable code the interface shows it by (`ERROR_CONTROL.md`).
    pub fn code(&self) -> &'static str {
        match self {
            Self::TooLarge => "look.preset_too_large",
            Self::Empty => "look.preset_empty",
            _ => "look.preset_invalid",
        }
    }
}

fn clean_text(t: &str) -> String {
    t.chars()
        .filter(|c| !c.is_control())
        .take(120)
        .collect::<String>()
        .trim()
        .to_owned()
}

/// A value as a file writes it: a number (`+8`, `-0.35`), or text (`True`, `Custom`).
fn value_of(raw: &str) -> Option<Value> {
    let t = raw.trim();
    if t.is_empty() {
        return None;
    }
    match t.trim_start_matches('+').parse::<f32>() {
        Ok(v) if v.is_finite() => Some(Value::Number(v.clamp(-100_000.0, 100_000.0))),
        Ok(_) => None,
        Err(_) => Some(Value::Text(clean_text(t))),
    }
}

/// A point of a curve written `"x, y"`.
fn point(raw: &str) -> Option<(f32, f32)> {
    let (x, y) = raw.split_once(',')?;
    let (x, y): (f32, f32) = (x.trim().parse().ok()?, y.trim().parse().ok()?);
    (x.is_finite() && y.is_finite()).then(|| (x.clamp(0.0, 255.0), y.clamp(0.0, 255.0)))
}

fn known(key: &str) -> bool {
    KEYS.contains(&key)
}

/// One `crs:` attribute of a preset's `rdf:Description`.
fn crs_attribute(preset: &mut Preset, kind: &mut Option<String>, key: String, value: &str) {
    match key.as_str() {
        "PresetType" => *kind = Some(value.to_owned()),
        "ProcessVersion" => preset.process = Some(clean_text(value)),
        "Name" if preset.name.is_none() => preset.name = Some(clean_text(value)),
        "Group" if preset.group.is_none() => preset.group = Some(clean_text(value)),
        k if known(k) => {
            if let Some(v) = value_of(value) {
                preset.values.insert(key, v);
            }
        }
        _ => {}
    }
}

/// Text inside the `crs:` element `key`: a setting written as an element, a name or a group
/// in its `rdf:Alt` (the `x-default` one wins), or a curve's point in its `rdf:Seq`.
fn crs_text(preset: &mut Preset, key: &str, text: &str, in_li: bool, alt_default: bool) {
    match key {
        "Name" | "Group" if in_li => {
            let slot = if key == "Name" {
                &mut preset.name
            } else {
                &mut preset.group
            };
            if slot.is_none() || alt_default {
                *slot = Some(clean_text(text));
            }
        }
        k if k.starts_with("ToneCurvePV2012") && in_li => {
            if let Some(p) = point(text) {
                let entry = preset
                    .values
                    .entry(key.to_owned())
                    .or_insert(Value::Curve(Vec::new()));
                if let Value::Curve(points) = entry
                    && points.len() < 256
                {
                    points.push(p);
                }
            }
        }
        k if known(k) && !in_li => {
            if let Some(v) = value_of(text) {
                preset.values.insert(key.to_owned(), v);
            }
        }
        _ => {}
    }
}

/// Parses an `.xmp` develop preset, its settings as attributes of `rdf:Description` or as
/// elements, its name and group from their `rdf:Alt`, its curves from their `rdf:Seq`.
pub fn parse_xmp(bytes: &[u8]) -> Result<Preset, PresetError> {
    if bytes.len() > MAX_BYTES {
        return Err(PresetError::TooLarge);
    }
    let mut reader = NsReader::from_reader(bytes);
    let mut preset = Preset::default();
    let mut kind: Option<String> = None;
    // The crs element we are inside, the depth it opened at, and whether an `rdf:li` is open.
    let mut open: Option<(String, usize)> = None;
    let mut in_li = false;
    let mut alt_default = false;
    let mut depth = 0usize;
    let invalid = |e: &dyn std::fmt::Display| PresetError::Invalid(e.to_string());
    loop {
        let (ns, event) = reader.read_resolved_event().map_err(|e| invalid(&e))?;
        let crs = matches!(ns, ResolveResult::Bound(n) if n.as_ref() == CRS);
        let rdf = matches!(ns, ResolveResult::Bound(n) if n.as_ref() == RDF);
        match event {
            Event::DocType(_) => return Err(PresetError::Dtd),
            Event::Eof => break,
            Event::Start(ref e) | Event::Empty(ref e) => {
                let empty = matches!(event, Event::Empty(_));
                if !empty {
                    depth += 1;
                    if depth > MAX_DEPTH {
                        return Err(PresetError::TooDeep);
                    }
                }
                let local = e.local_name().as_ref().to_owned();
                for attr in e.attributes() {
                    let attr = attr.map_err(|e| invalid(&e))?;
                    let (ans, alocal) = reader.resolver().resolve_attribute(attr.key);
                    let value = attr
                        .normalized_value(XmlVersion::Implicit1_0)
                        .map_err(|e| invalid(&e))?
                        .into_owned();
                    let key = alocal.as_ref().to_owned();
                    if matches!(ans, ResolveResult::Bound(n) if n.as_ref() == CRS) {
                        crs_attribute(&mut preset, &mut kind, key, &value);
                    } else if key == "lang" {
                        alt_default = value == "x-default";
                    }
                }
                if crs && !empty && open.is_none() {
                    open = Some((local, depth));
                } else if rdf && local == "li" && !empty {
                    in_li = true;
                }
            }
            Event::Text(ref t) => {
                if let Some((key, _)) = &open {
                    crs_text(&mut preset, key, &t.xml10_content(), in_li, alt_default);
                }
            }
            Event::End(_) => {
                if in_li {
                    in_li = false;
                    alt_default = false;
                } else if open.as_ref().is_some_and(|(_, d)| *d == depth) {
                    open = None;
                }
                depth = depth.saturating_sub(1);
            }
            _ => {}
        }
    }
    if let Some(kind) = kind
        && kind != "Normal"
    {
        return Err(PresetError::NotPreset(format!(
            "a {} preset (a profile or a mask)",
            clean_text(&kind)
        )));
    }
    if preset.values.is_empty() {
        return Err(PresetError::Empty);
    }
    Ok(preset)
}

/// A value of Lua's table constructors, all an `.lrtemplate` holds.
#[derive(Debug, Clone, PartialEq)]
enum Lua {
    Str(String),
    Num(f64),
    Bool(bool),
    Nil,
    Table(Vec<(Option<String>, Lua)>),
}

/// A reader of Lua's table constructors: `name = { key = value, ["key"] = value, value }`,
/// strings with their escapes, numbers, booleans, `nil` and `--` comments. Nothing is
/// evaluated: an expression, a call or anything else is an error.
struct LuaReader<'a> {
    s: &'a [u8],
    at: usize,
}

impl LuaReader<'_> {
    fn skip(&mut self) {
        loop {
            while self.at < self.s.len() && self.s[self.at].is_ascii_whitespace() {
                self.at += 1;
            }
            if self.s[self.at..].starts_with(b"--") {
                while self.at < self.s.len() && self.s[self.at] != b'\n' {
                    self.at += 1;
                }
            } else {
                return;
            }
        }
    }

    fn eat(&mut self, c: u8) -> bool {
        self.skip();
        if self.s.get(self.at) == Some(&c) {
            self.at += 1;
            true
        } else {
            false
        }
    }

    fn name(&mut self) -> Option<String> {
        self.skip();
        let start = self.at;
        while self.at < self.s.len()
            && (self.s[self.at].is_ascii_alphanumeric() || self.s[self.at] == b'_')
        {
            self.at += 1;
        }
        (self.at > start && !self.s[start].is_ascii_digit())
            .then(|| String::from_utf8_lossy(&self.s[start..self.at]).into_owned())
    }

    fn string(&mut self) -> Result<String, PresetError> {
        let quote = self.s[self.at];
        self.at += 1;
        let mut out = Vec::new();
        while let Some(&c) = self.s.get(self.at) {
            self.at += 1;
            match c {
                b'\\' => {
                    let e = *self
                        .s
                        .get(self.at)
                        .ok_or_else(|| PresetError::Invalid("an escape at the end".into()))?;
                    self.at += 1;
                    out.push(match e {
                        b'n' => b'\n',
                        b't' => b'\t',
                        b'r' => b'\r',
                        other => other,
                    });
                }
                c if c == quote => return Ok(String::from_utf8_lossy(&out).into_owned()),
                b'\n' => break,
                c => out.push(c),
            }
        }
        Err(PresetError::Invalid("a string without its end".into()))
    }

    fn number(&mut self) -> Result<f64, PresetError> {
        let start = self.at;
        while self.at < self.s.len()
            && matches!(
                self.s[self.at],
                b'0'..=b'9' | b'.' | b'-' | b'+' | b'e' | b'E'
            )
        {
            self.at += 1;
        }
        let text = std::str::from_utf8(&self.s[start..self.at]).unwrap_or_default();
        text.parse::<f64>()
            .ok()
            .filter(|v| v.is_finite())
            .ok_or_else(|| PresetError::Invalid(format!("not a number: {text}")))
    }

    fn value(&mut self, depth: usize) -> Result<Lua, PresetError> {
        if depth > MAX_DEPTH {
            return Err(PresetError::TooDeep);
        }
        self.skip();
        match self.s.get(self.at) {
            Some(b'{') => self.table(depth + 1),
            Some(b'"' | b'\'') => self.string().map(Lua::Str),
            Some(c) if c.is_ascii_digit() || *c == b'-' || *c == b'.' => {
                self.number().map(Lua::Num)
            }
            _ => match self.name().as_deref() {
                Some("true") => Ok(Lua::Bool(true)),
                Some("false") => Ok(Lua::Bool(false)),
                Some("nil") => Ok(Lua::Nil),
                _ => Err(PresetError::Invalid(format!(
                    "an expression at byte {}",
                    self.at
                ))),
            },
        }
    }

    fn table(&mut self, depth: usize) -> Result<Lua, PresetError> {
        self.at += 1; // `{`
        let mut fields = Vec::new();
        loop {
            if self.eat(b'}') {
                return Ok(Lua::Table(fields));
            }
            if fields.len() >= 10_000 {
                return Err(PresetError::Invalid("a table too long".into()));
            }
            let key = if self.eat(b'[') {
                self.skip();
                let k = match self.s.get(self.at) {
                    Some(b'"' | b'\'') => self.string()?,
                    _ => return Err(PresetError::Invalid("a key that is not a string".into())),
                };
                if !(self.eat(b']') && self.eat(b'=')) {
                    return Err(PresetError::Invalid("a key without `] =`".into()));
                }
                Some(k)
            } else {
                let save = self.at;
                match self.name() {
                    Some(n) if self.eat(b'=') => Some(n),
                    _ => {
                        self.at = save;
                        None
                    }
                }
            };
            fields.push((key, self.value(depth)?));
            if !(self.eat(b',') || self.eat(b';')) {
                if self.eat(b'}') {
                    return Ok(Lua::Table(fields));
                }
                return Err(PresetError::Invalid(format!(
                    "a table without `,` or `}}` at byte {}",
                    self.at
                )));
            }
        }
    }
}

fn field<'a>(table: &'a Lua, key: &str) -> Option<&'a Lua> {
    match table {
        Lua::Table(fields) => fields
            .iter()
            .find(|(k, _)| k.as_deref() == Some(key))
            .map(|(_, v)| v),
        _ => None,
    }
}

/// Parses a legacy `.lrtemplate`: `s = { title = …, value = { settings = { … } } }`.
pub fn parse_lrtemplate(bytes: &[u8]) -> Result<Preset, PresetError> {
    if bytes.len() > MAX_BYTES {
        return Err(PresetError::TooLarge);
    }
    let mut r = LuaReader { s: bytes, at: 0 };
    // `s = {…}` as Lightroom writes it, or `return {…}`.
    let Some(head) = r.name() else {
        return Err(PresetError::NotPreset("no `s = {` at the start".into()));
    };
    if !(head == "return" || r.eat(b'=')) {
        return Err(PresetError::NotPreset("no `s = {` at the start".into()));
    }
    r.skip();
    if r.s.get(r.at) != Some(&b'{') {
        return Err(PresetError::NotPreset("no table".into()));
    }
    let root = r.table(1)?;
    let text = |v: Option<&Lua>| match v {
        Some(Lua::Str(s)) => Some(clean_text(s)),
        _ => None,
    };
    let mut preset = Preset {
        name: text(field(&root, "title")).or_else(|| text(field(&root, "internalName"))),
        ..Preset::default()
    };
    if let Some(Lua::Str(t)) = field(&root, "type")
        && t != "Develop"
    {
        return Err(PresetError::NotPreset(format!(
            "a {} template",
            clean_text(t)
        )));
    }
    let settings = field(&root, "value").and_then(|v| field(v, "settings"));
    let Some(Lua::Table(settings)) = settings else {
        return Err(PresetError::NotPreset("no `value.settings`".into()));
    };
    for (key, value) in settings {
        let Some(key) = key else { continue };
        if key == "ProcessVersion" {
            preset.process = text(Some(value));
            continue;
        }
        if !known(key) {
            continue;
        }
        let v = match value {
            #[allow(clippy::cast_possible_truncation)]
            Lua::Num(n) => Some(Value::Number((*n as f32).clamp(-100_000.0, 100_000.0))),
            Lua::Bool(b) => Some(Value::Text(if *b { "True" } else { "False" }.to_owned())),
            Lua::Str(s) => value_of(s),
            // A curve: a flat list `x1, y1, x2, y2…`.
            Lua::Table(items) if key.starts_with("ToneCurvePV2012") => {
                #[allow(clippy::cast_possible_truncation)]
                let numbers: Vec<f32> = items
                    .iter()
                    .filter_map(|(_, v)| {
                        if let Lua::Num(n) = v {
                            Some(*n as f32)
                        } else {
                            None
                        }
                    })
                    .collect();
                let points = numbers
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .take(256)
                    .filter(|p| p[0].is_finite() && p[1].is_finite())
                    .map(|p| (p[0].clamp(0.0, 255.0), p[1].clamp(0.0, 255.0)))
                    .collect();
                Some(Value::Curve(points))
            }
            _ => None,
        };
        if let Some(v) = v {
            preset.values.insert(key.clone(), v);
        }
    }
    if preset.values.is_empty() {
        return Err(PresetError::Empty);
    }
    Ok(preset)
}

/// A preset file by its first bytes: `<` is an XMP, anything else an `.lrtemplate`.
pub fn parse(bytes: &[u8]) -> Result<Preset, PresetError> {
    let start = bytes
        .iter()
        .position(|b| !b.is_ascii_whitespace())
        .unwrap_or(0);
    let body = bytes[start..]
        .strip_prefix(b"\xEF\xBB\xBF")
        .unwrap_or(&bytes[start..]);
    if body.starts_with(b"<") {
        parse_xmp(body)
    } else {
        parse_lrtemplate(body)
    }
}

/// A `.zip`'s presets: each file's name and what it gave.
pub type ZipPresets = Vec<(String, Result<Preset, PresetError>)>;

/// The presets of a `.zip`: each `.xmp` or `.lrtemplate` entry read, its file name kept for
/// the import's report; folders, other files and paths with `..` skipped. At most
/// [`MAX_ZIP_ENTRIES`] presets and [`MAX_ZIP_BYTES`] unpacked.
pub fn parse_zip(bytes: &[u8]) -> Result<ZipPresets, PresetError> {
    if bytes.len() > MAX_ZIP_BYTES {
        return Err(PresetError::TooLarge);
    }
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes))
        .map_err(|e| PresetError::Invalid(e.to_string()))?;
    let mut out = Vec::new();
    let mut unpacked = 0usize;
    for i in 0..zip.len() {
        let mut entry = zip
            .by_index(i)
            .map_err(|e| PresetError::Invalid(e.to_string()))?;
        let name = entry
            .name()
            .map_err(|e| PresetError::Invalid(e.to_string()))?
            .into_owned();
        if entry.is_dir()
            || name.contains("..")
            || name.starts_with("__MACOSX")
            || !std::path::Path::new(&name).extension().is_some_and(|e| {
                e.eq_ignore_ascii_case("xmp") || e.eq_ignore_ascii_case("lrtemplate")
            })
        {
            continue;
        }
        if out.len() >= MAX_ZIP_ENTRIES {
            break;
        }
        let file = name.rsplit('/').next().unwrap_or(&name).to_owned();
        let mut data = Vec::new();
        let read = (&mut entry)
            .take(MAX_BYTES as u64 + 1)
            .read_to_end(&mut data);
        unpacked += data.len();
        if unpacked > MAX_ZIP_BYTES {
            return Err(PresetError::TooLarge);
        }
        let result = match read {
            Ok(_) if data.len() > MAX_BYTES => Err(PresetError::TooLarge),
            Ok(_) => parse(&data),
            Err(e) => Err(PresetError::Invalid(e.to_string())),
        };
        out.push((file, result));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Written for these tests, in the shape Lightroom writes its presets.
    const ATTRS: &str = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about="" xmlns:crs="http://ns.adobe.com/camera-raw-settings/1.0/"
   crs:PresetType="Normal" crs:ProcessVersion="15.4" crs:Contrast2012="+25" crs:Shadows2012="-10"
   crs:HueAdjustmentBlue="+12" crs:Clarity2012="20" crs:Unknown="7">
   <crs:Name><rdf:Alt><rdf:li xml:lang="vi">Ấm</rdf:li><rdf:li xml:lang="x-default">Warm test</rdf:li></rdf:Alt></crs:Name>
   <crs:Group><rdf:Alt><rdf:li xml:lang="x-default">Test group</rdf:li></rdf:Alt></crs:Group>
   <crs:ToneCurvePV2012><rdf:Seq><rdf:li>0, 0</rdf:li><rdf:li>128, 150</rdf:li><rdf:li>255, 255</rdf:li></rdf:Seq></crs:ToneCurvePV2012>
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>"#;

    const ELEMENTS: &str = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/"><rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description xmlns:c="http://ns.adobe.com/camera-raw-settings/1.0/">
   <c:PresetType>Normal</c:PresetType><c:Vibrance>30</c:Vibrance><c:ConvertToGrayscale>True</c:ConvertToGrayscale>
  </rdf:Description></rdf:RDF></x:xmpmeta>"#;

    #[test]
    fn fr_089_an_xmp_preset_as_attributes_or_elements() -> Result<(), PresetError> {
        let p = parse_xmp(ATTRS.as_bytes())?;
        assert_eq!(p.name.as_deref(), Some("Warm test"), "x-default wins");
        assert_eq!(p.group.as_deref(), Some("Test group"));
        assert_eq!(p.process.as_deref(), Some("15.4"));
        assert_eq!(p.values.get("Contrast2012"), Some(&Value::Number(25.0)));
        assert_eq!(p.values.get("Shadows2012"), Some(&Value::Number(-10.0)));
        assert_eq!(
            p.values.get("Clarity2012"),
            Some(&Value::Number(20.0)),
            "kept, to be named as not applied"
        );
        assert!(!p.values.contains_key("Unknown"), "only known settings");
        assert_eq!(
            p.values.get("ToneCurvePV2012"),
            Some(&Value::Curve(vec![
                (0.0, 0.0),
                (128.0, 150.0),
                (255.0, 255.0)
            ]))
        );
        let e = parse_xmp(ELEMENTS.as_bytes())?;
        assert_eq!(e.values.get("Vibrance"), Some(&Value::Number(30.0)));
        assert_eq!(
            e.values.get("ConvertToGrayscale"),
            Some(&Value::Text("True".into()))
        );
        Ok(())
    }

    #[test]
    fn fr_089_unsafe_or_wrong_xmp_is_refused() {
        let dtd = format!("<!DOCTYPE x [<!ENTITY a \"aaaa\">]>{ATTRS}");
        assert_eq!(parse_xmp(dtd.as_bytes()), Err(PresetError::Dtd));
        let deep = format!("{}{}", "<a>".repeat(1000), "</a>".repeat(1000));
        assert_eq!(parse_xmp(deep.as_bytes()), Err(PresetError::TooDeep));
        let profile = ATTRS.replace("PresetType=\"Normal\"", "PresetType=\"Look\"");
        assert!(matches!(
            parse_xmp(profile.as_bytes()),
            Err(PresetError::NotPreset(_))
        ));
        assert_eq!(parse_xmp(b"<x/>"), Err(PresetError::Empty));
        assert_eq!(
            parse_xmp(&vec![b' '; MAX_BYTES + 1]),
            Err(PresetError::TooLarge)
        );
        assert!(matches!(
            parse_xmp(b"<a><b></a>"),
            Err(PresetError::Invalid(_))
        ));
    }

    /// Written for these tests, in the shape of Lightroom's legacy templates.
    const TEMPLATE: &str = r#"s = {
	id = "0D1B-TEST",
	internalName = "Internal",
	title = "Cool \"night\"",
	type = "Develop",
	-- a comment
	value = {
		settings = {
			Contrast2012 = 15,
			IncrementalTemperature = -12,
			ConvertToGrayscale = false,
			["Saturation"] = -20,
			ToneCurvePV2012 = { 0, 0, 64, 50, 255, 255, },
			ProcessVersion = "10.0",
			LensProfileSetup = "LensDefaults",
		},
		uuid = "X",
	},
	version = 0,
}
"#;

    #[test]
    fn fr_089_an_lrtemplate_read_without_running_it() -> Result<(), PresetError> {
        let p = parse_lrtemplate(TEMPLATE.as_bytes())?;
        assert_eq!(p.name.as_deref(), Some("Cool \"night\""));
        assert_eq!(p.process.as_deref(), Some("10.0"));
        assert_eq!(p.values.get("Contrast2012"), Some(&Value::Number(15.0)));
        assert_eq!(
            p.values.get("IncrementalTemperature"),
            Some(&Value::Number(-12.0))
        );
        assert_eq!(p.values.get("Saturation"), Some(&Value::Number(-20.0)));
        assert_eq!(
            p.values.get("ConvertToGrayscale"),
            Some(&Value::Text("False".into()))
        );
        assert_eq!(
            p.values.get("ToneCurvePV2012"),
            Some(&Value::Curve(vec![
                (0.0, 0.0),
                (64.0, 50.0),
                (255.0, 255.0)
            ]))
        );
        assert_eq!(
            parse(TEMPLATE.as_bytes()),
            Ok(p.clone()),
            "sniffed as a template"
        );
        assert_eq!(
            parse(ATTRS.as_bytes()).map(|p| p.name),
            Ok(Some("Warm test".into())),
            "sniffed as XMP"
        );
        Ok(())
    }

    #[test]
    fn fr_089_a_template_that_is_code_or_broken_is_refused() {
        for bad in [
            "s = { value = { settings = { Contrast2012 = os.execute(\"x\") } } }",
            "s = { value = { settings = { Contrast2012 = 1 + 2 } } }",
            "print(1)",
            "s = { title = \"open",
            "s = { value = { settings = { Contrast2012 = 15 }",
        ] {
            assert!(parse_lrtemplate(bad.as_bytes()).is_err(), "{bad}");
        }
        let deep = format!("s = {}{}", "{".repeat(100), "}".repeat(100));
        assert_eq!(parse_lrtemplate(deep.as_bytes()), Err(PresetError::TooDeep));
        assert!(matches!(
            parse_lrtemplate(
                b"s = { type = \"Export\", value = { settings = { Contrast2012 = 1 } } }"
            ),
            Err(PresetError::NotPreset(_))
        ));
    }

    #[test]
    fn fr_089_a_zip_of_presets_skips_what_is_not_one() -> Result<(), PresetError> {
        let mut buf = Vec::new();
        {
            let mut w = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
            let opts = zip::write::SimpleFileOptions::default();
            for (name, body) in [
                ("Pack/Warm.xmp", ATTRS),
                ("Pack/Cool.lrtemplate", TEMPLATE),
                ("Pack/readme.txt", "hello"),
                ("../evil.xmp", ATTRS),
                ("__MACOSX/Pack/._Warm.xmp", "junk"),
                ("Pack/Broken.xmp", "<a>"),
            ] {
                w.start_file(name, opts)
                    .map_err(|e| PresetError::Invalid(e.to_string()))?;
                std::io::Write::write_all(&mut w, body.as_bytes())
                    .map_err(|e| PresetError::Invalid(e.to_string()))?;
            }
            w.finish()
                .map_err(|e| PresetError::Invalid(e.to_string()))?;
        }
        let read = parse_zip(&buf)?;
        let names: Vec<&str> = read.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["Warm.xmp", "Cool.lrtemplate", "Broken.xmp"]);
        assert!(read[0].1.is_ok() && read[1].1.is_ok() && read[2].1.is_err());
        Ok(())
    }
}
