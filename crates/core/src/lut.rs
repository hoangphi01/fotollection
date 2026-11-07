// SPDX-License-Identifier: GPL-3.0-only
//! Looks from LUT files (FR-070, ADR-0036): Adobe's `.cube` (1D, 3D, or a 1D shaper before a
//! 3D table, as `DaVinci Resolve` writes; LUMIX Lab's are 3D with a `#LUMIXPHOTOSTYLE` tag) and
//! `HaldCLUT` PNGs. Every file is untrusted: at most [`MAX_BYTES`], sizes within the format's
//! ranges, exactly the number of entries the header promises, every value finite and within
//! [`VALUE_RANGE`]; anything else is refused with its reason, never guessed. Parsed only in
//! Rust, from a file the user chose in a Rust-side dialog (SECURITY.md §3).

use std::io::Cursor;

/// Largest LUT file read: a 65³ `.cube` is about 7 MB, a level-16 Hald a few.
pub const MAX_BYTES: usize = 8 * 1024 * 1024;
/// Points a side of a 3D table, as the specification allows and LUMIX, Resolve and Photoshop
/// write (17, 33, 65 in practice).
pub const SIZE_3D: std::ops::RangeInclusive<usize> = 2..=65;
/// Entries of a 1D table.
pub const SIZE_1D: std::ops::RangeInclusive<usize> = 2..=65_536;
/// Values a table may hold: a little outside 0–1 for LUTs that overshoot, no more.
pub const VALUE_RANGE: std::ops::RangeInclusive<f32> = -0.5..=1.5;
/// Hald levels: a level-L image is L³ pixels a side and holds an L²-point cube.
pub const HALD_LEVELS: std::ops::RangeInclusive<usize> = 2..=16;

/// Why a LUT file was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LutError {
    #[error("the file is larger than 8 MB")]
    TooLarge,
    #[error("not a text .cube file")]
    NotText,
    #[error("line {0} is not a keyword or three numbers")]
    BadLine(usize),
    #[error("the table size is outside the format's range")]
    BadSize,
    #[error("the header promises {expected} entries, the file holds {found}")]
    WrongCount { expected: usize, found: usize },
    #[error("line {0} holds a value that is not a finite number in range")]
    BadValue(usize),
    #[error("the domain is empty or reversed")]
    BadDomain,
    #[error("no table in the file")]
    NoTable,
    #[error("not a HaldCLUT: {0}")]
    Hald(String),
    #[error("the LUT could not be kept: {0}")]
    Io(String),
}

impl LutError {
    /// The stable code the interface shows it by (`ERROR_CONTROL.md`).
    pub fn code(&self) -> &'static str {
        match self {
            Self::TooLarge => "look.lut_too_large",
            Self::Io(_) => "look.lut_unreadable",
            _ => "look.lut_invalid",
        }
    }
}

/// A 3D table: `size`³ entries, red fastest, then green, then blue, over `domain`.
#[derive(Debug, Clone, PartialEq)]
pub struct Cube {
    pub size: usize,
    pub domain: [[f32; 3]; 2],
    pub data: Vec<[f32; 3]>,
}

/// A 1D table per channel over `domain`, applied before the cube when both are present.
#[derive(Debug, Clone, PartialEq)]
pub struct Shaper {
    pub domain: [[f32; 3]; 2],
    pub data: Vec<[f32; 3]>,
}

/// A parsed LUT.
#[derive(Debug, Clone, PartialEq)]
pub struct Lut {
    pub title: Option<String>,
    pub shaper: Option<Shaper>,
    pub cube: Option<Cube>,
    /// Tagged by LUMIX Lab as made for V-Log footage: far too strong on a photo (FR-070).
    pub vlog: bool,
}

const UNIT: [[f32; 3]; 2] = [[0.0; 3], [1.0; 3]];

fn numbers<const N: usize>(words: &[&str]) -> Option<[f32; N]> {
    if words.len() != N {
        return None;
    }
    let mut out = [0f32; N];
    for (o, w) in out.iter_mut().zip(words) {
        *o = w.parse().ok()?;
    }
    Some(out)
}

fn domain_ok(d: &[[f32; 3]; 2]) -> bool {
    (0..3).all(|c| d[0][c].is_finite() && d[1][c].is_finite() && d[1][c] > d[0][c])
}

/// What a `.cube` says before its data.
struct Header {
    title: Option<String>,
    vlog: bool,
    size1: Option<usize>,
    size3: Option<usize>,
    domain1: [[f32; 3]; 2],
    domain3: [[f32; 3]; 2],
}

impl Header {
    /// The entries its sizes promise: the 1D table's, then the cube's.
    fn expected(&self) -> usize {
        self.size1.unwrap_or(0) + self.size3.map_or(0, |s| s * s * s)
    }

    /// One keyword line, `first` its keyword and `rest` its words after it.
    fn keyword(
        &mut self,
        line: &str,
        first: &str,
        rest: &[&str],
        line_no: usize,
    ) -> Result<(), LutError> {
        match first {
            "TITLE" => {
                let t = line["TITLE".len()..].trim().trim_matches('"');
                self.title = Some(t.chars().filter(|c| !c.is_control()).take(120).collect());
            }
            "LUT_1D_SIZE" | "LUT_3D_SIZE" => {
                let n: usize = rest
                    .first()
                    .filter(|_| rest.len() == 1)
                    .and_then(|w| w.parse().ok())
                    .ok_or(LutError::BadLine(line_no))?;
                let (range, slot) = if first == "LUT_1D_SIZE" {
                    (SIZE_1D, &mut self.size1)
                } else {
                    (SIZE_3D, &mut self.size3)
                };
                if !range.contains(&n) || slot.is_some() {
                    return Err(LutError::BadSize);
                }
                *slot = Some(n);
            }
            "DOMAIN_MIN" | "DOMAIN_MAX" => {
                let v: [f32; 3] = numbers(rest).ok_or(LutError::BadLine(line_no))?;
                let at = usize::from(first == "DOMAIN_MAX");
                self.domain1[at] = v;
                self.domain3[at] = v;
            }
            "LUT_1D_INPUT_RANGE" | "LUT_3D_INPUT_RANGE" => {
                let [lo, hi]: [f32; 2] = numbers(rest).ok_or(LutError::BadLine(line_no))?;
                let d = [[lo; 3], [hi; 3]];
                if first == "LUT_1D_INPUT_RANGE" {
                    self.domain1 = d;
                } else {
                    self.domain3 = d;
                }
            }
            _ => return Err(LutError::BadLine(line_no)),
        }
        Ok(())
    }
}

/// Parses a `.cube` file (Adobe Cube LUT Specification 1.0, with Resolve's shaper and input
/// range keywords).
pub fn parse_cube(bytes: &[u8]) -> Result<Lut, LutError> {
    if bytes.len() > MAX_BYTES {
        return Err(LutError::TooLarge);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| LutError::NotText)?;
    let mut h = Header {
        title: None,
        vlog: false,
        size1: None,
        size3: None,
        domain1: UNIT,
        domain3: UNIT,
    };
    let mut values: Vec<[f32; 3]> = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let line_no = i + 1;
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(comment) = line.strip_prefix('#') {
            let upper = comment.to_ascii_uppercase();
            h.vlog |= upper.contains("LUMIXPHOTOSTYLE") && upper.contains("VLOG");
            continue;
        }
        let words: Vec<&str> = line.split_whitespace().collect();
        let Some(&first) = words.first() else {
            continue;
        };
        // A data line starts with a number, `NaN` and `inf` included, so they are refused as
        // values with their reason rather than as unknown keywords.
        let starts_number = first.parse::<f32>().is_ok()
            || first.starts_with(|c: char| c.is_ascii_digit() || c == '-' || c == '+' || c == '.');
        if !starts_number {
            // Keywords come before the data.
            if !values.is_empty() {
                return Err(LutError::BadLine(line_no));
            }
            h.keyword(line, first, &words[1..], line_no)?;
            continue;
        }
        let v: [f32; 3] = numbers(&words).ok_or(LutError::BadLine(line_no))?;
        if !v.iter().all(|x| x.is_finite() && VALUE_RANGE.contains(x)) {
            return Err(LutError::BadValue(line_no));
        }
        values.push(v);
        if values.len() > h.expected() {
            return Err(LutError::WrongCount {
                expected: h.expected(),
                found: values.len(),
            });
        }
    }
    if h.size1.is_none() && h.size3.is_none() {
        return Err(LutError::NoTable);
    }
    if !domain_ok(&h.domain1) || !domain_ok(&h.domain3) {
        return Err(LutError::BadDomain);
    }
    if values.len() != h.expected() {
        return Err(LutError::WrongCount {
            expected: h.expected(),
            found: values.len(),
        });
    }
    let mut values = values.into_iter();
    let shaper = h.size1.map(|n| Shaper {
        domain: h.domain1,
        data: values.by_ref().take(n).collect(),
    });
    let cube = h.size3.map(|size| Cube {
        size,
        domain: h.domain3,
        data: values.collect(),
    });
    Ok(Lut {
        title: h.title,
        shaper,
        cube,
        vlog: h.vlog,
    })
}

/// Parses a `HaldCLUT` PNG: a square image `level³` pixels a side holding a `level²`-point cube,
/// red fastest along each row.
pub fn parse_hald(bytes: &[u8]) -> Result<Lut, LutError> {
    if bytes.len() > MAX_BYTES {
        return Err(LutError::TooLarge);
    }
    let hald = |m: &str| LutError::Hald(m.to_owned());
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND);
    let mut reader = decoder.read_info().map_err(|e| hald(&e.to_string()))?;
    let (w, h) = (reader.info().width as usize, reader.info().height as usize);
    let level = HALD_LEVELS
        .clone()
        .find(|l| l * l * l == w)
        .filter(|_| w == h)
        .ok_or_else(|| hald("its side is not a level cubed (8, 27, 64 … 4096 pixels)"))?;
    let size = reader
        .output_buffer_size()
        .ok_or_else(|| hald("unknown buffer size"))?;
    let mut buf = vec![0u8; size];
    let frame = reader
        .next_frame(&mut buf)
        .map_err(|e| hald(&e.to_string()))?;
    let channels = match frame.color_type {
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        _ => return Err(hald("not an RGB image")),
    };
    let sixteen = frame.bit_depth == png::BitDepth::Sixteen;
    let bytes_per = if sixteen { 2 } else { 1 };
    if buf.len() < w * h * channels * bytes_per {
        return Err(hald("unexpected pixel layout"));
    }
    let sample = |i: usize| -> f32 {
        if sixteen {
            f32::from(u16::from_be_bytes([buf[i * 2], buf[i * 2 + 1]])) / 65535.0
        } else {
            f32::from(buf[i]) / 255.0
        }
    };
    let data = (0..w * h)
        .map(|p| [0, 1, 2].map(|c| sample(p * channels + c)))
        .collect();
    Ok(Lut {
        title: None,
        shaper: None,
        cube: Some(Cube {
            size: level * level,
            domain: UNIT,
            data,
        }),
        vlog: false,
    })
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn normalise(v: f32, lo: f32, hi: f32) -> f32 {
    ((v - lo) / (hi - lo)).clamp(0.0, 1.0)
}

impl Shaper {
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    fn apply(&self, rgb: [f32; 3]) -> [f32; 3] {
        let n = self.data.len();
        [0, 1, 2].map(|c| {
            let t = normalise(rgb[c], self.domain[0][c], self.domain[1][c]) * (n - 1) as f32;
            let i = (t as usize).min(n - 2);
            let f = t - i as f32;
            self.data[i][c] + (self.data[i + 1][c] - self.data[i][c]) * f
        })
    }
}

impl Cube {
    /// The table at `rgb`, by tetrahedral interpolation (as `looks::lookup`, in floats).
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    pub fn apply(&self, rgb: [f32; 3]) -> [f32; 3] {
        let n = self.size;
        let last = (n - 1) as f32;
        let t = [0, 1, 2].map(|c| normalise(rgb[c], self.domain[0][c], self.domain[1][c]) * last);
        let base = t.map(|v| (v as usize).min(n - 2));
        let f = [
            t[0] - base[0] as f32,
            t[1] - base[1] as f32,
            t[2] - base[2] as f32,
        ];
        let at = |r: usize, g: usize, b: usize| self.data[(b * n + g) * n + r];
        let (r, g, b) = (base[0], base[1], base[2]);
        let c000 = at(r, g, b);
        let c111 = at(r + 1, g + 1, b + 1);
        let (fr, fg, fb) = (f[0], f[1], f[2]);
        // The six tetrahedra of the cell, chosen by the order of the fractions.
        let (c1, c2, w) = if fr >= fg {
            if fg >= fb {
                (at(r + 1, g, b), at(r + 1, g + 1, b), [fr, fg, fb])
            } else if fr >= fb {
                (at(r + 1, g, b), at(r + 1, g, b + 1), [fr, fb, fg])
            } else {
                (at(r, g, b + 1), at(r + 1, g, b + 1), [fb, fr, fg])
            }
        } else if fb >= fg {
            (at(r, g, b + 1), at(r, g + 1, b + 1), [fb, fg, fr])
        } else if fb >= fr {
            (at(r, g + 1, b), at(r, g + 1, b + 1), [fg, fb, fr])
        } else {
            (at(r, g + 1, b), at(r + 1, g + 1, b), [fg, fr, fb])
        };
        [0, 1, 2].map(|c| {
            c000[c] + w[0] * (c1[c] - c000[c]) + w[1] * (c2[c] - c1[c]) + w[2] * (c111[c] - c2[c])
        })
    }
}

impl Lut {
    /// The LUT at `rgb` (encoded values, 0–1): the shaper, then the cube.
    pub fn apply(&self, rgb: [f32; 3]) -> [f32; 3] {
        let shaped = self.shaper.as_ref().map_or(rgb, |s| s.apply(rgb));
        self.cube.as_ref().map_or(shaped, |c| c.apply(shaped))
    }
}

/// Where imported LUTs are kept: `<config>/looks/<hash>.cube|.png`, the original file by its
/// content hash (ADR-0036), set once at start-up; none in tests and tools.
static DIR: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

/// Parsed LUTs by hash, most recent last. Owned by this module, at most `KEPT` (a 65³ table is
/// about 3.3 MB), read again from `DIR` when dropped; never emptied otherwise, since a hash
/// always names the same file.
type Kept = std::sync::Mutex<Vec<(String, std::sync::Arc<Lut>)>>;
static PARSED: std::sync::LazyLock<Kept> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(Vec::new()));
const KEPT: usize = 8;

/// Sets the folder imported LUTs live in; the first call wins.
pub fn set_dir(dir: std::path::PathBuf) {
    let _ = DIR.set(dir);
}

/// A file's kind, from its first bytes: a PNG is a Hald, anything else is read as `.cube`.
fn parse(bytes: &[u8]) -> Result<(Lut, &'static str), LutError> {
    if bytes.starts_with(b"\x89PNG") {
        parse_hald(bytes).map(|l| (l, "png"))
    } else {
        parse_cube(bytes).map(|l| (l, "cube"))
    }
}

fn keep(hash: &str, lut: std::sync::Arc<Lut>) {
    if let Ok(mut kept) = PARSED.lock() {
        kept.retain(|(h, _)| h != hash);
        if kept.len() >= KEPT {
            kept.remove(0);
        }
        kept.push((hash.to_owned(), lut));
    }
}

/// Imports a LUT file's bytes: parsed (refused with its reason), then kept under its content
/// hash, written once. Returns the hash (64 hex characters) and the LUT.
pub fn import(bytes: &[u8]) -> Result<(String, std::sync::Arc<Lut>), LutError> {
    let (lut, ext) = parse(bytes)?;
    let hash = blake3::hash(bytes).to_hex().to_string();
    if let Some(dir) = DIR.get() {
        let path = dir.join(format!("{hash}.{ext}"));
        if !path.exists() {
            let _ = std::fs::create_dir_all(dir);
            // Through a part of its own, then a rename: a file under a hash is whole or absent.
            let part = path.with_extension(format!("{ext}.part"));
            std::fs::write(&part, bytes)
                .and_then(|()| std::fs::rename(&part, &path))
                .map_err(|e| LutError::Io(e.to_string()))?;
        }
    }
    let lut = std::sync::Arc::new(lut);
    keep(&hash, std::sync::Arc::clone(&lut));
    Ok((hash, lut))
}

/// The LUT kept under `hash`, read from its file when not in memory; `None` when the hash is
/// not one or no file holds it (a project from another machine without it).
pub fn get(hash: &str) -> Option<std::sync::Arc<Lut>> {
    if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    if let Ok(kept) = PARSED.lock()
        && let Some((_, lut)) = kept.iter().find(|(h, _)| h == hash)
    {
        return Some(std::sync::Arc::clone(lut));
    }
    let dir = DIR.get()?;
    let bytes = ["cube", "png"]
        .iter()
        .find_map(|ext| std::fs::read(dir.join(format!("{hash}.{ext}"))).ok())?;
    let lut = std::sync::Arc::new(parse(&bytes).ok()?.0);
    keep(hash, std::sync::Arc::clone(&lut));
    Some(lut)
}

#[cfg(test)]
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::format_push_string,
    clippy::format_collect
)]
mod tests {
    use super::*;

    /// A `.cube` written as the specification describes: red fastest.
    fn identity_cube(n: usize, header: &str) -> String {
        let mut s = format!("{header}LUT_3D_SIZE {n}\n");
        let step = 1.0 / (n - 1) as f32;
        for b in 0..n {
            for g in 0..n {
                for r in 0..n {
                    s.push_str(&format!(
                        "{} {} {}\n",
                        r as f32 * step,
                        g as f32 * step,
                        b as f32 * step
                    ));
                }
            }
        }
        s
    }

    #[test]
    fn fr_070_an_identity_cube_changes_nothing_at_any_size() -> Result<(), LutError> {
        for n in [2, 17, 33] {
            let lut = parse_cube(identity_cube(n, "TITLE \"Same\"\n# a comment\n").as_bytes())?;
            assert_eq!(lut.title.as_deref(), Some("Same"));
            for rgb in [
                [0.0, 0.0, 0.0],
                [0.25, 0.5, 0.75],
                [1.0, 0.1, 0.9],
                [0.333, 0.333, 0.333],
            ] {
                let out = lut.apply(rgb);
                assert!(
                    out.iter().zip(rgb).all(|(a, b)| (a - b).abs() < 1e-5),
                    "{n}: {rgb:?} → {out:?}"
                );
            }
        }
        Ok(())
    }

    /// Red fastest: a cube that swaps red and blue must swap them, which a reader taking blue
    /// fastest would not (the order every reader gets wrong once, LESSONS 2026-11-02).
    #[test]
    fn fr_070_red_runs_fastest() -> Result<(), LutError> {
        let mut s = String::from("LUT_3D_SIZE 2\n");
        for b in 0..2 {
            for g in 0..2 {
                for r in 0..2 {
                    s.push_str(&format!("{b} {g} {r}\n"));
                }
            }
        }
        let lut = parse_cube(s.as_bytes())?;
        assert_eq!(lut.apply([1.0, 0.0, 0.0]), [0.0, 0.0, 1.0]);
        Ok(())
    }

    #[test]
    fn fr_070_a_shaper_domain_crlf_and_the_lumix_tag() -> Result<(), LutError> {
        let mut s = String::from(
            "#LUMIXPHOTOSTYLE VLOG\r\nLUT_1D_SIZE 2\r\nLUT_3D_INPUT_RANGE 0 1\r\nLUT_3D_SIZE 2\r\n",
        );
        // The shaper halves every channel; the cube is the identity.
        s.push_str("0 0 0\r\n0.5 0.5 0.5\r\n");
        for b in 0..2 {
            for g in 0..2 {
                for r in 0..2 {
                    s.push_str(&format!("{r} {g} {b}\r\n"));
                }
            }
        }
        let lut = parse_cube(s.as_bytes())?;
        assert!(lut.vlog);
        let out = lut.apply([1.0, 0.5, 0.0]);
        assert!(
            (out[0] - 0.5).abs() < 1e-5 && (out[1] - 0.25).abs() < 1e-5,
            "{out:?}"
        );
        Ok(())
    }

    #[test]
    fn fr_070_bad_files_are_refused_with_their_reason() {
        let ok = identity_cube(2, "");
        // Cut short, as the "33-point" file found online that holds 31 entries.
        let short: String = ok.lines().take(5).map(|l| format!("{l}\n")).collect();
        assert!(matches!(
            parse_cube(short.as_bytes()),
            Err(LutError::WrongCount {
                expected: 8,
                found: 4
            })
        ));
        assert_eq!(
            parse_cube(b"LUT_3D_SIZE 1\n0 0 0\n"),
            Err(LutError::BadSize)
        );
        assert_eq!(parse_cube(b"LUT_3D_SIZE 66\n"), Err(LutError::BadSize));
        assert_eq!(
            parse_cube(ok.replacen("0 0 0", "NaN 0 0", 1).as_bytes()),
            Err(LutError::BadValue(2))
        );
        assert_eq!(
            parse_cube(ok.replacen("0 0 0", "9 0 0", 1).as_bytes()),
            Err(LutError::BadValue(2))
        );
        assert_eq!(
            parse_cube(ok.replacen("0 0 0", "0 0", 1).as_bytes()),
            Err(LutError::BadLine(2))
        );
        assert_eq!(
            parse_cube(format!("{ok}0 0 0\n").as_bytes()),
            Err(LutError::WrongCount {
                expected: 8,
                found: 9
            })
        );
        assert_eq!(
            parse_cube(format!("DOMAIN_MIN 1 1 1\n{ok}").as_bytes()),
            Err(LutError::BadDomain)
        );
        assert_eq!(parse_cube(b"TITLE \"x\"\n"), Err(LutError::NoTable));
        assert_eq!(parse_cube(b"\xff\xfe"), Err(LutError::NotText));
        assert_eq!(
            parse_cube(format!("SOMETHING 1\n{ok}").as_bytes()),
            Err(LutError::BadLine(1))
        );
        assert_eq!(
            parse_cube(&vec![b' '; MAX_BYTES + 1]),
            Err(LutError::TooLarge)
        );
    }

    fn hald_png(level: usize) -> Vec<u8> {
        let side = level * level * level;
        let n = level * level;
        let mut data = Vec::with_capacity(side * side * 3);
        for i in 0..side * side {
            let (r, g, b) = (i % n, (i / n) % n, i / (n * n));
            for v in [r, g, b] {
                data.push(u8::try_from(v * 255 / (n - 1)).unwrap_or(255));
            }
        }
        let mut out = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut out, side as u32, side as u32);
            enc.set_color(png::ColorType::Rgb);
            enc.set_depth(png::BitDepth::Eight);
            if let Ok(mut w) = enc.write_header() {
                let _ = w.write_image_data(&data);
            }
        }
        out
    }

    #[test]
    fn fr_070_an_identity_hald_changes_nothing_and_a_bad_one_is_refused() -> Result<(), LutError> {
        let lut = parse_hald(&hald_png(4))?;
        assert_eq!(lut.cube.as_ref().map(|c| c.size), Some(16));
        let out = lut.apply([0.2, 0.6, 0.9]);
        assert!(
            out.iter()
                .zip([0.2, 0.6, 0.9])
                .all(|(a, b)| (a - b).abs() < 0.01),
            "{out:?}"
        );
        let mut not_square = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut not_square, 10, 10);
            enc.set_color(png::ColorType::Rgb);
            if let Ok(mut w) = enc.write_header() {
                let _ = w.write_image_data(&[0u8; 300]);
            }
        }
        assert!(matches!(parse_hald(&not_square), Err(LutError::Hald(_))));
        assert!(matches!(parse_hald(b"not a png"), Err(LutError::Hald(_))));
        Ok(())
    }

    /// FR-070: an imported LUT goes into the look's table first, mixed by its strength and
    /// the look's amount; a hash with no file leaves the photo as it is.
    #[test]
    fn fr_070_a_looks_lut_goes_into_its_table() -> Result<(), LutError> {
        let mut s = String::from("LUT_3D_SIZE 2\n");
        for b in 0..2 {
            for g in 0..2 {
                for r in 0..2 {
                    s.push_str(&format!("{b} {g} {r}\n"));
                }
            }
        }
        let (hash, _) = import(s.as_bytes())?;
        let through = |strength: u8, amount: u8| {
            let look = crate::looks::LookSettings {
                lut: Some(hash.clone()),
                lut_strength: strength,
                amount,
                ..crate::looks::LookSettings::default()
            };
            look.table().map(|t| crate::looks::lookup(&t, [255, 0, 0]))
        };
        assert_eq!(through(100, 100), Some([0, 0, 255]), "red becomes blue");
        let half = through(50, 100).unwrap_or_default();
        assert!(
            (126..=129).contains(&half[0]) && (126..=129).contains(&half[2]),
            "{half:?}"
        );
        assert_eq!(through(100, 0), None, "no amount, no table");
        let missing = crate::looks::LookSettings {
            lut: Some("ab".repeat(32)),
            ..crate::looks::LookSettings::default()
        };
        assert_eq!(
            missing
                .table()
                .map(|t| crate::looks::lookup(&t, [255, 0, 0])),
            Some([255, 0, 0])
        );
        Ok(())
    }
}
