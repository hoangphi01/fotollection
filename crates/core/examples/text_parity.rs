// SPDX-License-Identifier: GPL-3.0-only
//! FR-134: the glyphs every text of the app is drawn from, written down so a change of font
//! engine can be compared with the one before it. For each bundled face (Inter at the three
//! weights the info frame uses) and any font given with `--font`, it records the face's
//! metrics, then for each character of a set covering the app's scripts (Latin with
//! Vietnamese, Greek, Cyrillic, kana and kanji, Hangul): its glyph, advance and kerning, and at
//! three sizes and two sub-pixel positions its pixel bounds and coverage. `write` saves them;
//! `compare` reads a saved set and prints every difference, with the largest change of coverage
//! in 8-bit levels.
//!
//! `cargo run --release -p fotollection-core --example text_parity -- write <dir> [--font <ttf|otf>]…`
//! `cargo run --release -p fotollection-core --example text_parity -- compare <dir> [--font <ttf|otf>]…`
#![allow(
    clippy::expect_used,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::similar_names
)]

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use fotollection_core::glyph::{Font, FontArc, FontRef, PxScale, ScaleFont, VariableFont, point};

const SIZES: [f32; 3] = [11.0, 24.0, 64.0];
const OFFSETS: [(f32, f32); 2] = [(0.0, 0.0), (0.3, 0.7)];

fn bundled() -> Vec<(String, FontArc)> {
    let face = |name: &str, bytes: &'static [u8]| {
        (
            name.to_string(),
            FontArc::try_from_slice(bytes).expect(name),
        )
    };
    let inter = |weight: f32| {
        let mut font =
            FontRef::try_from_slice(include_bytes!("../assets/fonts/Inter.ttf")).expect("Inter");
        font.set_variation(b"wght", weight);
        (format!("Inter-{weight}"), FontArc::new(font))
    };
    vec![
        face(
            "DSEG14",
            include_bytes!("../assets/fonts/DSEG14Classic-Italic.ttf"),
        ),
        inter(400.0),
        inter(600.0),
        inter(800.0),
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

/// The characters: printable ASCII, Vietnamese, Greek, Cyrillic, the stamp's and frame's
/// signs, kana, the kanji and Hangul of the cards, and a spread of Hangul syllables.
fn characters() -> Vec<char> {
    let mut s: String = (' '..='~').collect();
    s.push_str("ăâđêôơưĂÂĐÊÔƠƯàáảãạằắẳẵặầấẩẫậèéẻẽẹềếểễệìíỉĩịòóỏõọồốổỗộờớởỡợùúủũụừứửữựỳýỷỹỵ");
    s.push_str("ΑΒΓΔΩαβγδλπωКАБВГДЖЗИЛПФЦЧШЩЭЮЯабвгджзилпфцчшщэюяёЁ");
    s.push_str("°·–—…“”‘’×€£¥№ƒ");
    s.extend('\u{3041}'..='\u{3096}');
    s.extend('\u{30A1}'..='\u{30FA}');
    s.push_str("ー〒、。「」東京都港区芝公園丁目番地様郵便年月日鎌倉極楽寺駅富士河口湖");
    s.push_str("서울특별시종로구세종대로문화회관김민준귀하받는사람우편엽서단풍아래에서안부전해요");
    s.extend(('\u{AC00}'..='\u{D7A3}').step_by(97));
    let mut seen = std::collections::BTreeSet::new();
    s.chars().filter(|c| seen.insert(*c)).collect()
}

/// One face's record: its metrics and every glyph's facts, each a line of text, and the
/// coverage of every drawn glyph as 8-bit levels (for the size of a difference).
fn record(font: &FontArc, chars: &[char]) -> (Vec<String>, BTreeMap<String, Vec<u8>>) {
    let mut lines = vec![format!(
        "face upem={:?} ascent={:?} descent={:?} height={:?}",
        font.units_per_em(),
        font.ascent_unscaled(),
        font.descent_unscaled(),
        font.height_unscaled()
    )];
    let mut coverage = BTreeMap::new();
    for (i, &c) in chars.iter().enumerate() {
        let id = font.glyph_id(c);
        let next = font.glyph_id(chars[(i + 1) % chars.len()]);
        lines.push(format!(
            "U+{:04X} id={} advance={:?} kern={:?}",
            c as u32,
            id.0,
            font.h_advance_unscaled(id),
            font.kern_unscaled(id, next)
        ));
        for px in SIZES {
            let scaled = font.as_scaled(PxScale::from(px));
            let mut line = format!(
                "U+{:04X} px={px} ascent={:?} descent={:?} advance={:?}",
                c as u32,
                scaled.ascent(),
                scaled.descent(),
                scaled.h_advance(id)
            );
            for (dx, dy) in OFFSETS {
                let glyph = id.with_scale_and_position(PxScale::from(px), point(dx, 100.0 + dy));
                match font.outline_glyph(glyph) {
                    None => line.push_str(" none"),
                    Some(outlined) => {
                        let b = outlined.px_bounds();
                        let (w, h) = (b.width() as usize, b.height() as usize);
                        let mut levels = vec![0u8; w * h];
                        let mut exact = 0u64;
                        outlined.draw(|x, y, v| {
                            exact = exact.wrapping_mul(1_000_003).wrapping_add(
                                u64::from(v.to_bits())
                                    ^ (u64::from(x) << 40)
                                    ^ (u64::from(y) << 20),
                            );
                            if let Some(slot) = levels.get_mut(y as usize * w + x as usize) {
                                *slot = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                            }
                        });
                        let _ = write!(
                            line,
                            " [{:?},{:?},{:?},{:?}] {exact:016x}",
                            b.min.x, b.min.y, b.max.x, b.max.y
                        );
                        coverage.insert(format!("U+{:04X} {px} {dx}", c as u32), levels);
                    }
                }
            }
            lines.push(line);
        }
    }
    (lines, coverage)
}

fn coverage_file(dir: &Path, name: &str) -> PathBuf {
    dir.join(format!("{name}.cov"))
}

fn save_coverage(path: &Path, coverage: &BTreeMap<String, Vec<u8>>) {
    let mut out = Vec::new();
    for (key, levels) in coverage {
        out.extend_from_slice(&(key.len() as u32).to_le_bytes());
        out.extend_from_slice(key.as_bytes());
        out.extend_from_slice(&(levels.len() as u32).to_le_bytes());
        out.extend_from_slice(levels);
    }
    std::fs::write(path, out).expect("write coverage");
}

fn load_coverage(path: &Path) -> BTreeMap<String, Vec<u8>> {
    let bytes = std::fs::read(path).unwrap_or_default();
    let mut map = BTreeMap::new();
    let mut at = 0;
    let take = |n: usize, at: &mut usize| {
        let slice = bytes.get(*at..*at + n).map(<[u8]>::to_vec);
        *at += n;
        slice
    };
    while let Some(len) = take(4, &mut at) {
        let len = u32::from_le_bytes(len.try_into().unwrap_or([0; 4])) as usize;
        let Some(key) = take(len, &mut at) else { break };
        let Some(n) = take(4, &mut at) else { break };
        let n = u32::from_le_bytes(n.try_into().unwrap_or([0; 4])) as usize;
        let Some(levels) = take(n, &mut at) else {
            break;
        };
        map.insert(String::from_utf8_lossy(&key).into_owned(), levels);
    }
    map
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (Some(mode), Some(dir)) = (args.first(), args.get(1)) else {
        eprintln!("usage: text_parity write|compare <dir> [--font <ttf|otf>]…");
        std::process::exit(2);
    };
    let dir = PathBuf::from(dir);
    let mut faces = bundled();
    for pair in args[2..].windows(2) {
        if pair[0] == "--font" {
            let bytes = std::fs::read(&pair[1]).expect("read --font");
            let name = Path::new(&pair[1])
                .file_stem()
                .map(|s| s.to_string_lossy().replace(' ', "_"))
                .unwrap_or_default();
            faces.push((
                format!("user-{name}"),
                FontArc::try_from_vec(bytes).expect("parse --font"),
            ));
        }
    }
    let chars = characters();
    std::fs::create_dir_all(&dir).expect("create dir");
    let mut differing = 0usize;
    for (name, font) in &faces {
        let (lines, coverage) = record(font, &chars);
        let text = lines.join("\n") + "\n";
        let facts = dir.join(format!("{name}.txt"));
        if mode == "write" {
            std::fs::write(&facts, &text).expect("write facts");
            save_coverage(&coverage_file(&dir, name), &coverage);
            println!(
                "{name}: {} lines, {} glyphs drawn",
                lines.len(),
                coverage.len()
            );
            continue;
        }
        let before = std::fs::read_to_string(&facts).unwrap_or_default();
        let old: Vec<&str> = before.lines().collect();
        let changed: Vec<(usize, &str)> = lines
            .iter()
            .enumerate()
            .filter(|(i, l)| old.get(*i).copied() != Some(l.as_str()))
            .map(|(i, _)| (i, old.get(i).copied().unwrap_or("(none)")))
            .collect();
        let old_cov = load_coverage(&coverage_file(&dir, name));
        let (mut glyphs, mut pixels, mut worst) = (0usize, 0usize, 0u8);
        for (key, levels) in &coverage {
            match old_cov.get(key) {
                Some(prev) if prev.len() == levels.len() => {
                    let d: Vec<u8> = prev
                        .iter()
                        .zip(levels)
                        .map(|(a, b)| a.abs_diff(*b))
                        .collect();
                    let n = d.iter().filter(|v| **v > 0).count();
                    if n > 0 {
                        glyphs += 1;
                        pixels += n;
                        worst = worst.max(d.iter().copied().max().unwrap_or(0));
                    }
                }
                _ => glyphs += 1,
            }
        }
        differing += changed.len() + glyphs;
        println!(
            "{name}: {} of {} lines differ; coverage: {glyphs} glyphs, {pixels} pixels, at most {worst} levels",
            changed.len(),
            lines.len()
        );
        for (i, was) in changed.iter().take(6) {
            println!("   was: {was}\n   now: {}", lines[*i]);
        }
    }
    if mode == "compare" && differing > 0 {
        std::process::exit(1);
    }
}
