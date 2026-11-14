// SPDX-License-Identifier: GPL-3.0-only
//! Korea in `AddAssist` (FR-133, FR-132): the Ministry of the Interior's road-name addresses
//! (도로명주소 한글 전체분), packed by `crates/addassistdata` into `kr.bin` (format in
//! `assets/addassist/ADDASSIST.md`), compiled in and checked against a pinned BLAKE3 sum on
//! first use. A road and a building number give the 5-digit code; a code gives its roads.

use std::sync::OnceLock;

use super::romaji;
use super::{AddAssistError, Found};

const DATA: &[u8] = include_bytes!("../../assets/addassist/kr.bin");
const DATA_BLAKE3: &str = "8930992ebc99c7c63df17a518d0c0927a55bc870c3199de6e1942501d3dff7a7";
const MAGIC: &[u8; 4] = b"FADK";
const FORMAT_VERSION: u16 = 1;

/// Bounds a malformed count cannot exceed: about twice today's.
const MAX_STRINGS: u32 = 400_000;
const MAX_ROADS: u32 = 400_000;

struct Area {
    sido: &'static str,
    sigungu: &'static str,
    eupmyeon: &'static str,
}

struct Road {
    area: u16,
    name: &'static str,
    underground: bool,
    /// Its runs: `runs[first..first + len]`.
    first: u32,
    len: u16,
}

struct Dataset {
    snapshot: &'static str,
    areas: Vec<Area>,
    codes: Vec<u32>,
    roads: Vec<Road>,
    /// (first building number, code index), per road in number order.
    runs: Vec<(u16, u16)>,
}

struct Reader {
    bytes: &'static [u8],
    at: usize,
}

impl Reader {
    fn take(&mut self, n: usize) -> Result<&'static [u8], AddAssistError> {
        let end = self.at.checked_add(n).ok_or(AddAssistError::Malformed)?;
        let slice = self
            .bytes
            .get(self.at..end)
            .ok_or(AddAssistError::Malformed)?;
        self.at = end;
        Ok(slice)
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N], AddAssistError> {
        self.take(N)?
            .try_into()
            .map_err(|_| AddAssistError::Malformed)
    }
    fn u8(&mut self) -> Result<u8, AddAssistError> {
        Ok(self.array::<1>()?[0])
    }
    fn u16(&mut self) -> Result<u16, AddAssistError> {
        Ok(u16::from_le_bytes(self.array()?))
    }
    fn u32(&mut self) -> Result<u32, AddAssistError> {
        Ok(u32::from_le_bytes(self.array()?))
    }
    fn str(&mut self, len: usize) -> Result<&'static str, AddAssistError> {
        std::str::from_utf8(self.take(len)?).map_err(|_| AddAssistError::Malformed)
    }
}

fn parse(bytes: &'static [u8]) -> Result<Dataset, AddAssistError> {
    let mut r = Reader { bytes, at: 0 };
    if &r.array::<4>()? != MAGIC || r.u16()? != FORMAT_VERSION {
        return Err(AddAssistError::Malformed);
    }
    let snapshot = r.str(10)?;
    let n_strings = r.u32()?;
    if n_strings == 0 || n_strings > MAX_STRINGS {
        return Err(AddAssistError::Malformed);
    }
    let mut strings = Vec::with_capacity(n_strings as usize);
    for _ in 0..n_strings {
        let len = usize::from(r.u16()?);
        strings.push(r.str(len)?);
    }
    let string = |id: u32| {
        strings
            .get(id as usize)
            .copied()
            .ok_or(AddAssistError::Malformed)
    };
    let n_areas = r.u16()?;
    let mut areas = Vec::with_capacity(usize::from(n_areas));
    for _ in 0..n_areas {
        areas.push(Area {
            sido: string(r.u32()?)?,
            sigungu: string(r.u32()?)?,
            eupmyeon: string(r.u32()?)?,
        });
    }
    let n_codes = r.u16()?;
    let mut codes = Vec::with_capacity(usize::from(n_codes));
    for _ in 0..n_codes {
        let code = r.u32()?;
        if code > 99_999 {
            return Err(AddAssistError::Malformed);
        }
        codes.push(code);
    }
    let n_roads = r.u32()?;
    if n_roads > MAX_ROADS {
        return Err(AddAssistError::Malformed);
    }
    let mut roads = Vec::with_capacity(n_roads as usize);
    let mut first: u32 = 0;
    for _ in 0..n_roads {
        let area = r.u16()?;
        let name = string(r.u32()?)?;
        let underground = r.u8()? != 0;
        let len = r.u16()?;
        if usize::from(area) >= areas.len() || len == 0 {
            return Err(AddAssistError::Malformed);
        }
        roads.push(Road {
            area,
            name,
            underground,
            first,
            len,
        });
        first = first
            .checked_add(u32::from(len))
            .ok_or(AddAssistError::Malformed)?;
    }
    let mut runs = Vec::with_capacity(first as usize);
    for _ in 0..first {
        let number = r.u16()?;
        let code = r.u16()?;
        if usize::from(code) >= codes.len() {
            return Err(AddAssistError::Malformed);
        }
        runs.push((number, code));
    }
    if r.at != bytes.len() {
        return Err(AddAssistError::Malformed);
    }
    Ok(Dataset {
        snapshot,
        areas,
        codes,
        roads,
        runs,
    })
}

fn dataset() -> Result<&'static Dataset, AddAssistError> {
    static CELL: OnceLock<Result<Dataset, AddAssistError>> = OnceLock::new();
    CELL.get_or_init(|| {
        if blake3::hash(DATA).to_hex().as_str() != DATA_BLAKE3 {
            return Err(AddAssistError::Checksum);
        }
        parse(DATA)
    })
    .as_ref()
    .map_err(Clone::clone)
}

/// Each road's name folded for the search (Hangul without spaces, and its romanization),
/// made once on the first search.
fn index() -> Result<&'static Vec<(String, String)>, AddAssistError> {
    static CELL: OnceLock<Result<Vec<(String, String)>, AddAssistError>> = OnceLock::new();
    CELL.get_or_init(|| {
        let d = dataset()?;
        Ok(d.roads
            .iter()
            .map(|r| {
                (
                    r.name.replace(' ', ""),
                    romaji::fold(&romaji::revised(r.name)),
                )
            })
            .collect())
    })
    .as_ref()
    .map_err(Clone::clone)
}

/// The road-name data's date, for Settings and the manual.
pub fn snapshot() -> Result<&'static str, AddAssistError> {
    Ok(dataset()?.snapshot)
}

fn runs<'a>(d: &'a Dataset, road: &Road) -> &'a [(u16, u16)] {
    let start = road.first as usize;
    &d.runs[start..start + usize::from(road.len)]
}

/// The code of a building number on a road: the last run starting at or before it.
fn code_at(d: &Dataset, road: &Road, number: u16) -> Option<u32> {
    let runs = runs(d, road);
    let at = runs.partition_point(|&(first, _)| first <= number);
    at.checked_sub(1).map(|i| d.codes[usize::from(runs[i].1)])
}

/// A road's code when it has one only.
fn only_code(d: &Dataset, road: &Road) -> Option<u32> {
    let runs = runs(d, road);
    let first = runs.first()?.1;
    runs.iter()
        .all(|&(_, c)| c == first)
        .then(|| d.codes[usize::from(first)])
}

fn found(d: &Dataset, road: &Road, code: Option<u32>) -> Found {
    let a = &d.areas[usize::from(road.area)];
    let name = if road.underground {
        format!("{} 지하", road.name)
    } else {
        road.name.to_owned()
    };
    Found {
        code: code.map(|c| format!("{c:05}")).unwrap_or_default(),
        region: a.sido.to_owned(),
        city: a.sigungu.to_owned(),
        town: a.eupmyeon.to_owned(),
        road: name,
        note: String::new(),
        region_latin: romaji::korean_place(a.sido),
        city_latin: romaji::korean_place(a.sigungu),
        town_latin: if a.eupmyeon.is_empty() {
            String::new()
        } else {
            romaji::korean_place(a.eupmyeon)
        },
        road_latin: romaji::korean_place(road.name),
    }
}

/// The digits of a code as typed (`03172`, `031-72`).
fn digits(code: &str) -> String {
    code.chars()
        .map(romaji::normalize_char)
        .filter(char::is_ascii_digit)
        .collect()
}

/// Every road a full 5-digit code covers, each with its area; none for a code not listed.
pub fn lookup(code: &str) -> Result<Vec<Found>, AddAssistError> {
    let d = dataset()?;
    let digits = digits(code);
    let (5, Ok(code)) = (digits.len(), digits.parse::<u32>()) else {
        return Ok(Vec::new());
    };
    let Some(index) = d.codes.iter().position(|&c| c == code) else {
        return Ok(Vec::new());
    };
    let index = u16::try_from(index).map_err(|_| AddAssistError::Malformed)?;
    Ok(d.roads
        .iter()
        .filter(|road| runs(d, road).iter().any(|&(_, c)| c == index))
        .map(|road| found(d, road, Some(code)))
        .collect())
}

/// `세종대로 175` → (`세종대로`, Some(175)); a query without a trailing number → (query, None).
fn split_number(q: &str) -> (&str, Option<u16>) {
    let trimmed = q.trim_end();
    let start = trimmed
        .char_indices()
        .rev()
        .take_while(|(_, c)| c.is_ascii_digit())
        .last()
        .map(|(i, _)| i);
    match start {
        Some(i) if i > 0 => (trimmed[..i].trim_end(), trimmed[i..].parse().ok()),
        _ => (q, None),
    }
}

/// Roads whose name starts with the query, in Hangul or in romanized letters (`세종대로`,
/// `sejong-daero`), or the code's first three digits. A building number after the road
/// (`세종대로 175`) gives that building's code; a road with one code only gives it anyway.
pub fn search(query: &str, limit: usize) -> Result<Vec<Found>, AddAssistError> {
    let d = dataset()?;
    let q: String = query.trim().chars().map(romaji::normalize_char).collect();
    if q.is_empty() {
        return Ok(Vec::new());
    }
    if q.chars()
        .all(|c| c.is_ascii_digit() || matches!(c, '-' | ' '))
    {
        let digits = digits(&q);
        if digits.len() == 5 {
            return lookup(&digits).map(|mut v| {
                v.truncate(limit);
                v
            });
        }
        return Ok(Vec::new());
    }
    let ix = index()?;
    let (name, number) = split_number(&q);
    let hangul: String = name.chars().filter(|c| *c != ' ').collect();
    let latin = name.is_ascii();
    let folded = romaji::fold(name);
    if (latin && folded.is_empty()) || hangul.is_empty() {
        return Ok(Vec::new());
    }
    let mut hits: Vec<(bool, &Road)> = d
        .roads
        .iter()
        .zip(ix)
        .filter_map(|(road, (h, r))| {
            let (field, q) = if latin { (r, &folded) } else { (h, &hangul) };
            field.starts_with(q.as_str()).then_some((field == q, road))
        })
        .collect();
    // A name met whole first, then on the surface before underground, by area.
    hits.sort_by_key(|(exact, r)| (!exact, r.underground, r.area));
    Ok(hits
        .into_iter()
        .filter_map(|(_, road)| {
            let code = match number {
                Some(n) => Some(code_at(d, road, n)?),
                None => only_code(d, road),
            };
            Some(found(d, road, code))
        })
        .take(limit)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok<T>(r: Result<T, AddAssistError>) -> T {
        match r {
            Ok(v) => v,
            Err(e) => panic!("{e}"),
        }
    }

    #[test]
    fn fr_133_a_road_and_number_give_the_code() {
        let hits = ok(search("세종대로 175", 10));
        let sejong = hits
            .iter()
            .find(|f| f.city == "종로구")
            .map(|f| f.code.as_str());
        assert_eq!(sejong, Some("03172"));
        let latin = ok(search("sejong-daero 175", 10));
        assert!(latin.iter().any(|f| f.code == "03172"));
        assert!(latin.iter().any(|f| f.road_latin == "Sejong-daero"));
    }

    #[test]
    fn fr_133_a_code_finds_its_roads() {
        let roads = ok(lookup("03172"));
        assert!(
            roads
                .iter()
                .any(|f| f.region == "서울특별시" && f.city == "종로구" && f.road == "세종대로")
        );
        assert!(ok(lookup("03048")).iter().any(|f| f.road == "청와대로"));
        assert_eq!(ok(lookup("3172")), Vec::new());
        assert_eq!(ok(lookup("00000")), Vec::new());
        assert!(
            ok(search("cheongwadae-ro", 5))
                .iter()
                .any(|f| f.road == "청와대로")
        );
    }

    #[test]
    fn fr_133_the_area_is_written_in_latin_too() {
        let f = &ok(lookup("03172"))[0];
        assert_eq!(
            (f.region_latin.as_str(), f.city_latin.as_str()),
            ("Seoul", "Jongno-gu")
        );
    }

    #[test]
    fn damaged_data_is_rejected() {
        assert_eq!(parse(b"FADK").err(), Some(AddAssistError::Malformed));
        assert_eq!(
            parse(b"NOPE\x01\x00").err(),
            Some(AddAssistError::Malformed)
        );
        assert!(snapshot().is_ok_and(|s| s.len() == 10));
    }
}
