// SPDX-License-Identifier: GPL-3.0-only
//! Packs Japan Post's postal-code data (`utf_ken_all.csv`, the UTF-8 one-record-per-line
//! edition) into `jp.bin`, `AddAssist`'s Japanese table (FR-133, FR-130). Run by
//! `scripts/addassist/fetch-and-pack.sh`; the format is read by
//! `crates/core/src/addassist/jp.rs` and described in
//! `crates/core/assets/addassist/ADDASSIST.md`.
//!
//! `fotollection-addassistdata jp <utf_ken_all.csv> <snapshot YYYY-MM-DD> <out.bin>`
//!
//! The placeholder rows are kept with an empty town, matched exactly, never by an ending
//! (PRD FR-130): 以下に掲載がない場合, a municipality's ○○一円 (while a town really named 一円
//! stays), and ○○の次に番地がくる場合. A note in parentheses (大通西（１〜１９丁目）) is kept
//! apart from the town.

use std::collections::HashMap;
use std::io::Write;
use std::process::ExitCode;

const NO_TOWN: &str = "以下に掲載がない場合";
const NO_TOWN_KANA: &str = "イカニケイサイガナイバアイ";
const NUMBER_NEXT: &str = "の次に番地がくる場合";
const WHOLE: &str = "一円";

fn fail(message: &str) -> ExitCode {
    eprintln!("addassistdata: {message}");
    ExitCode::FAILURE
}

/// Every distinct string once, numbered in the order first seen; 0 is "".
#[derive(Default)]
struct Strings {
    index: HashMap<String, u32>,
    list: Vec<String>,
}

impl Strings {
    fn id(&mut self, s: &str) -> Option<u32> {
        if self.list.is_empty() {
            self.list.push(String::new());
            self.index.insert(String::new(), 0);
        }
        if let Some(&i) = self.index.get(s) {
            return Some(i);
        }
        let i = u32::try_from(self.list.len()).ok()?;
        self.list.push(s.to_owned());
        self.index.insert(s.to_owned(), i);
        Some(i)
    }
}

/// `大通西（１〜１９丁目）` → (`大通西`, `１〜１９丁目`); no parentheses → (text, "").
fn split_note(text: &str) -> (&str, &str) {
    match text.split_once('（') {
        Some((town, rest)) => (town, rest.strip_suffix('）').unwrap_or(rest)),
        None => (text, ""),
    }
}

/// The town a row names, or "" for a placeholder row.
fn town_of<'a>(town: &'a str, kana: &'a str) -> (&'a str, &'a str) {
    let placeholder = town == NO_TOWN
        || kana == NO_TOWN_KANA
        || town.ends_with(NUMBER_NEXT)
        || (town.ends_with(WHOLE) && town != WHOLE);
    if placeholder { ("", "") } else { (town, kana) }
}

fn unquote(field: &str) -> &str {
    let f = field.trim();
    f.strip_prefix('"')
        .and_then(|f| f.strip_suffix('"'))
        .unwrap_or(f)
        .trim()
}

struct Row {
    code: u32,
    muni: u16,
    town: u32,
    note: u32,
    town_kana: u32,
}

fn pack_jp(csv: &str, snapshot: &str) -> Result<(Vec<u8>, usize, usize), String> {
    let mut strings = Strings::default();
    // JIS municipality code → index, with its (prefecture, city, their kana) string ids.
    let mut muni_index: HashMap<u32, u16> = HashMap::new();
    let mut munis: Vec<[u32; 4]> = Vec::new();
    let mut rows: Vec<Row> = Vec::new();
    for (n, line) in csv.lines().enumerate() {
        let c: Vec<&str> = line.split(',').map(unquote).collect();
        if c.len() != 15 {
            return Err(format!("line {}: {} columns, 15 expected", n + 1, c.len()));
        }
        let (Ok(jis), Ok(code)) = (c[0].parse::<u32>(), c[2].parse::<u32>()) else {
            return Err(format!("line {}: bad code", n + 1));
        };
        if c[2].len() != 7 {
            return Err(format!("line {}: the postal code is not 7 digits", n + 1));
        }
        let muni = if let Some(&m) = muni_index.get(&jis) {
            m
        } else {
            let m = u16::try_from(munis.len()).map_err(|_| "too many municipalities")?;
            let ids = [c[6], c[7], c[3], c[4]].map(|s| strings.id(s));
            let [Some(a), Some(b), Some(cc), Some(d)] = ids else {
                return Err("too many strings".into());
            };
            munis.push([a, b, cc, d]);
            muni_index.insert(jis, m);
            m
        };
        let (town, kana) = town_of(c[8], c[5]);
        let (town, note) = split_note(town);
        let (kana, _) = split_note(kana);
        let (Some(town), Some(note), Some(town_kana)) =
            (strings.id(town), strings.id(note), strings.id(kana))
        else {
            return Err("too many strings".into());
        };
        rows.push(Row {
            code,
            muni,
            town,
            note,
            town_kana,
        });
    }
    // By code, so a lookup is a binary search and a code's towns come out together.
    rows.sort_by_key(|r| r.code);

    let mut bin = Vec::with_capacity(rows.len() * 18 + 4_000_000);
    bin.extend_from_slice(b"FADJ");
    bin.extend_from_slice(&1u16.to_le_bytes());
    bin.extend_from_slice(snapshot.as_bytes());
    let n_strings = u32::try_from(strings.list.len()).map_err(|_| "too many strings")?;
    bin.extend_from_slice(&n_strings.to_le_bytes());
    for s in &strings.list {
        let len = u16::try_from(s.len()).map_err(|_| "a string is too long")?;
        bin.extend_from_slice(&len.to_le_bytes());
        bin.extend_from_slice(s.as_bytes());
    }
    let n_munis = u16::try_from(munis.len()).map_err(|_| "too many municipalities")?;
    bin.extend_from_slice(&n_munis.to_le_bytes());
    for m in &munis {
        for id in m {
            bin.extend_from_slice(&id.to_le_bytes());
        }
    }
    let n_rows = u32::try_from(rows.len()).map_err(|_| "too many rows")?;
    bin.extend_from_slice(&n_rows.to_le_bytes());
    for r in &rows {
        bin.extend_from_slice(&r.code.to_le_bytes());
        bin.extend_from_slice(&r.muni.to_le_bytes());
        bin.extend_from_slice(&r.town.to_le_bytes());
        bin.extend_from_slice(&r.note.to_le_bytes());
        bin.extend_from_slice(&r.town_kana.to_le_bytes());
    }
    Ok((bin, rows.len(), munis.len()))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [country, input, snapshot, out] = args.as_slice() else {
        return fail(
            "usage: fotollection-addassistdata jp <utf_ken_all.csv> <YYYY-MM-DD> <out.bin>",
        );
    };
    if country != "jp" {
        return fail("only jp is packed so far");
    }
    if snapshot.len() != 10 {
        return fail("snapshot must be YYYY-MM-DD");
    }
    let Ok(csv) = std::fs::read_to_string(input) else {
        return fail("cannot read the CSV (UTF-8 expected)");
    };
    let (bin, rows, munis) = match pack_jp(&csv, snapshot) {
        Ok(packed) => packed,
        Err(e) => return fail(&e),
    };
    if std::fs::File::create(out)
        .and_then(|mut f| f.write_all(&bin))
        .is_err()
    {
        return fail("cannot write the output");
    }
    eprintln!(
        "addassistdata: jp {rows} rows, {munis} municipalities, {} bytes, BLAKE3 {}",
        bin.len(),
        blake3::hash(&bin).to_hex()
    );
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholders_are_matched_exactly() {
        assert_eq!(town_of(NO_TOWN, NO_TOWN_KANA), ("", ""));
        assert_eq!(town_of("利島村一円", "トシマムライチエン"), ("", ""));
        assert_eq!(town_of("一円", "イチエン"), ("一円", "イチエン"));
        assert_eq!(town_of("犬山市の次に番地がくる場合", "x"), ("", ""));
        assert_eq!(
            town_of("稲村ガ崎", "イナムラガサキ"),
            ("稲村ガ崎", "イナムラガサキ")
        );
    }

    #[test]
    fn a_note_is_kept_apart() {
        assert_eq!(
            split_note("大通西（１〜１９丁目）"),
            ("大通西", "１〜１９丁目")
        );
        assert_eq!(split_note("稲村ガ崎"), ("稲村ガ崎", ""));
    }
}
