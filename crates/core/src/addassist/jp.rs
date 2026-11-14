// SPDX-License-Identifier: GPL-3.0-only
//! Japan in `AddAssist` (FR-133, FR-130): Japan Post's postal-code data, packed by
//! `crates/addassistdata` into `jp.bin` (format in `assets/addassist/ADDASSIST.md`), compiled
//! into the binary and checked against a pinned BLAKE3 sum on first use. A 7-digit code gives
//! its prefecture, municipality and town; a search takes kanji, kana or romaji.

use std::collections::HashMap;
use std::sync::OnceLock;

use super::romaji;
use super::{AddAssistError, Found};

const DATA: &[u8] = include_bytes!("../../assets/addassist/jp.bin");
const DATA_BLAKE3: &str = "7e42b98b6ac3fce948c525f373f5af50a74e0bbf9c9f8dc09a32c74aa187b58a";
const MAGIC: &[u8; 4] = b"FADJ";
const FORMAT_VERSION: u16 = 1;

struct Muni {
    pref: &'static str,
    city: &'static str,
    pref_kana: &'static str,
    city_kana: &'static str,
}

struct Row {
    code: u32,
    muni: u16,
    town: u32,
    note: u32,
    town_kana: u32,
}

struct Dataset {
    snapshot: &'static str,
    strings: Vec<&'static str>,
    munis: Vec<Muni>,
    /// Sorted by code.
    rows: Vec<Row>,
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

/// The most strings, municipalities and rows a table may hold: twice today's, so a malformed
/// count cannot ask for gigabytes.
const MAX_STRINGS: u32 = 400_000;
const MAX_ROWS: u32 = 250_000;

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
    let n_munis = r.u16()?;
    let mut munis = Vec::with_capacity(usize::from(n_munis));
    for _ in 0..n_munis {
        munis.push(Muni {
            pref: string(r.u32()?)?,
            city: string(r.u32()?)?,
            pref_kana: string(r.u32()?)?,
            city_kana: string(r.u32()?)?,
        });
    }
    let n_rows = r.u32()?;
    if n_rows > MAX_ROWS {
        return Err(AddAssistError::Malformed);
    }
    let mut rows = Vec::with_capacity(n_rows as usize);
    for _ in 0..n_rows {
        let row = Row {
            code: r.u32()?,
            muni: r.u16()?,
            town: r.u32()?,
            note: r.u32()?,
            town_kana: r.u32()?,
        };
        let ok = row.code <= 9_999_999
            && usize::from(row.muni) < munis.len()
            && [row.town, row.note, row.town_kana]
                .iter()
                .all(|&id| id < n_strings)
            && rows.last().is_none_or(|p: &Row| p.code <= row.code);
        if !ok {
            return Err(AddAssistError::Malformed);
        }
        rows.push(row);
    }
    if r.at != bytes.len() {
        return Err(AddAssistError::Malformed);
    }
    Ok(Dataset {
        snapshot,
        strings,
        munis,
        rows,
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

/// What the search compares, made once on the first search: each municipality's names and
/// each town's, folded.
struct Index {
    /// Per municipality: (prefecture, city) folded from their kana, and the city's Latin name.
    munis: Vec<(String, String, String)>,
    /// The prefecture's Latin name per municipality.
    prefs: Vec<String>,
    /// Folded reading per town string id.
    towns: HashMap<u32, String>,
}

/// The reading a designated city's wards share (サッポロシ for 札幌市中央区, 札幌市北区…).
fn city_readings(d: &Dataset) -> HashMap<&'static str, &'static str> {
    let mut by_city: HashMap<&str, &str> = HashMap::new();
    for m in &d.munis {
        let Some(at) = m.city.find('市').filter(|_| m.city.ends_with('区')) else {
            continue;
        };
        let city = &m.city[..at + '市'.len_utf8()];
        by_city
            .entry(city)
            .and_modify(|common| {
                let shared: usize = common
                    .char_indices()
                    .zip(m.city_kana.chars())
                    .take_while(|((_, a), b)| a == b)
                    .last()
                    .map_or(0, |((i, a), _)| i + a.len_utf8());
                *common = &common[..shared];
            })
            .or_insert(m.city_kana);
    }
    // The shared reading ends where the city's does: at its last シ.
    by_city
        .into_iter()
        .filter_map(|(city, common)| {
            common
                .rfind('シ')
                .map(|at| (city, &common[..at + 'シ'.len_utf8()]))
        })
        .collect()
}

fn index() -> Result<&'static Index, AddAssistError> {
    static CELL: OnceLock<Result<Index, AddAssistError>> = OnceLock::new();
    CELL.get_or_init(|| {
        let d = dataset()?;
        let readings = city_readings(d);
        let munis = d
            .munis
            .iter()
            .map(|m| {
                let city_reading = m
                    .city
                    .find('市')
                    .and_then(|at| readings.get(&m.city[..at + '市'.len_utf8()]))
                    .copied();
                (
                    romaji::fold(&romaji::hepburn(m.pref_kana)),
                    romaji::fold(&romaji::hepburn(m.city_kana)),
                    romaji::municipality(m.city, m.city_kana, city_reading),
                )
            })
            .collect();
        let prefs = d
            .munis
            .iter()
            .map(|m| romaji::place(m.pref, m.pref_kana))
            .collect();
        let mut towns = HashMap::new();
        for r in &d.rows {
            towns
                .entry(r.town_kana)
                .or_insert_with(|| romaji::fold(&romaji::hepburn(d.strings[r.town_kana as usize])));
        }
        Ok(Index {
            munis,
            prefs,
            towns,
        })
    })
    .as_ref()
    .map_err(Clone::clone)
}

/// The Japan Post data's date, for Settings and the manual.
pub fn snapshot() -> Result<&'static str, AddAssistError> {
    Ok(dataset()?.snapshot)
}

fn found(d: &Dataset, ix: &Index, r: &Row) -> Found {
    let m = &d.munis[usize::from(r.muni)];
    let town_kana = d.strings[r.town_kana as usize];
    let town_latin = romaji::hepburn(town_kana);
    let mut town_latin_cap = town_latin.trim().to_owned();
    if let Some(first) = town_latin_cap.get(..1) {
        town_latin_cap.replace_range(..1, &first.to_uppercase());
    }
    Found {
        code: format!("{:07}", r.code),
        region: m.pref.to_owned(),
        city: m.city.to_owned(),
        town: d.strings[r.town as usize].to_owned(),
        note: d.strings[r.note as usize].to_owned(),
        region_latin: ix.prefs[usize::from(r.muni)].clone(),
        city_latin: ix.munis[usize::from(r.muni)].2.clone(),
        town_latin: town_latin_cap,
    }
}

/// The digits of a code as typed: `248-0024`, `２４８０００２４`, `〒248 0024`.
fn digits(code: &str) -> String {
    code.chars()
        .map(romaji::normalize_char)
        .filter(char::is_ascii_digit)
        .collect()
}

/// Every town a full 7-digit code covers (several when the code is shared); none for a code
/// Japan Post does not list.
pub fn lookup(code: &str) -> Result<Vec<Found>, AddAssistError> {
    let d = dataset()?;
    let digits = digits(code);
    let Ok(code) = digits.parse::<u32>() else {
        return Ok(Vec::new());
    };
    if digits.len() != 7 {
        return Ok(Vec::new());
    }
    let ix = index()?;
    let start = d.rows.partition_point(|r| r.code < code);
    Ok(d.rows[start..]
        .iter()
        .take_while(|r| r.code == code)
        .map(|r| found(d, ix, r))
        .collect())
}

fn is_kana(c: char) -> bool {
    matches!(c, 'ァ'..='ヺ' | 'ー')
}

/// Places whose municipality or town starts with the query, in kanji, kana or romaji
/// (`kamakura`, `かまくら`, `鎌倉`), or whose code starts with its digits (three at least);
/// a name met exactly first, then by code.
pub fn search(query: &str, limit: usize) -> Result<Vec<Found>, AddAssistError> {
    let d = dataset()?;
    let ix = index()?;
    let q: String = query.trim().chars().map(romaji::normalize_char).collect();
    if q.is_empty() {
        return Ok(Vec::new());
    }
    let code_digits = digits(&q);
    let by_code = !code_digits.is_empty()
        && q.chars()
            .all(|c| c.is_ascii_digit() || matches!(c, '-' | ' ' | '〒'));
    let latin = q.is_ascii();
    let kana = q.chars().all(|c| is_kana(c) || c == ' ');
    let folded = romaji::fold(&q);
    let kana_q: String = q.chars().filter(|c| *c != ' ').collect();

    // 0: a name met exactly, 1: a municipality starting so, 2: a town starting so.
    let score = |r: &Row| -> Option<u8> {
        let m = &d.munis[usize::from(r.muni)];
        let town = d.strings[r.town as usize];
        let town_kana = d.strings[r.town_kana as usize];
        if latin {
            let (pref, city, _) = &ix.munis[usize::from(r.muni)];
            let town_f = ix.towns.get(&r.town_kana).map_or("", String::as_str);
            let city_stem = city
                .strip_suffix("shi")
                .or_else(|| city.strip_suffix("ku"))
                .unwrap_or(city);
            if city_stem == folded || town_f == folded {
                Some(0)
            } else if city.starts_with(&folded) || pref.starts_with(&folded) {
                Some(1)
            } else if town_f.starts_with(&folded) {
                Some(2)
            } else {
                None
            }
        } else if kana {
            if m.city_kana.starts_with(&kana_q) || m.pref_kana.starts_with(&kana_q) {
                Some(1)
            } else if town_kana.starts_with(&kana_q) {
                Some(2)
            } else {
                None
            }
        } else if town == q {
            Some(0)
        } else if m.city.starts_with(&q)
            || m.pref.starts_with(&q)
            || format!("{}{}", m.pref, m.city).starts_with(&q)
        {
            Some(1)
        } else if town.starts_with(&q) || format!("{}{}", m.city, town).starts_with(&q) {
            Some(2)
        } else {
            None
        }
    };

    let mut hits: Vec<(u8, &Row)> = if by_code {
        if code_digits.len() < 3 {
            return Ok(Vec::new());
        }
        d.rows
            .iter()
            .filter(|r| format!("{:07}", r.code).starts_with(&code_digits))
            .take(limit)
            .map(|r| (0, r))
            .collect()
    } else {
        d.rows
            .iter()
            .filter_map(|r| score(r).map(|s| (s, r)))
            .collect()
    };
    hits.sort_by_key(|(s, r)| (*s, r.code));
    Ok(hits
        .into_iter()
        .take(limit)
        .map(|(_, r)| found(d, ix, r))
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
    fn fr_133_a_code_finds_its_town() {
        let found = ok(lookup("248-0024"));
        assert_eq!(found.len(), 1);
        let f = &found[0];
        assert_eq!(
            (f.region.as_str(), f.city.as_str(), f.town.as_str()),
            ("神奈川県", "鎌倉市", "稲村ガ崎")
        );
        assert_eq!(
            (f.region_latin.as_str(), f.city_latin.as_str()),
            ("Kanagawa-ken", "Kamakura-shi")
        );
        assert_eq!(f.town_latin, "Inamuragasaki");
        assert_eq!(ok(lookup("２４８００２４")), found);
        assert_eq!(ok(lookup("248002")), Vec::new());
        assert_eq!(ok(lookup("0000000")), Vec::new());
    }

    #[test]
    fn fr_133_placeholder_rows_leave_the_town_blank() {
        // 060-0000 is 札幌市中央区's 以下に掲載がない場合; 100-0301 利島村一円.
        let sapporo = ok(lookup("0600000"));
        assert_eq!(sapporo[0].town, "");
        assert_eq!(sapporo[0].city_latin, "Sapporo-shi Chuo-ku");
        assert_eq!(ok(lookup("1000301"))[0].town, "");
    }

    #[test]
    fn fr_133_a_note_stays_apart() {
        let f = ok(lookup("0600042"));
        assert_eq!(f[0].town, "大通西");
        assert_eq!(f[0].note, "１〜１９丁目");
    }

    #[test]
    fn fr_133_a_shared_code_offers_a_pick() {
        let d = ok(dataset());
        let shared = d
            .rows
            .windows(2)
            .find(|w| w[0].code == w[1].code)
            .map(|w| w[0].code)
            .unwrap_or_default();
        assert!(ok(lookup(&format!("{shared:07}"))).len() >= 2);
    }

    #[test]
    fn fr_133_the_search_takes_kanji_kana_and_romaji() {
        for q in ["kamakura", "Kamakura", "かまくら", "カマクラ", "鎌倉"] {
            let hits = ok(search(q, 20));
            assert!(hits.iter().any(|f| f.city == "鎌倉市"), "{q}");
        }
        assert!(
            ok(search("inamuragasaki", 5))
                .iter()
                .any(|f| f.code == "2480024")
        );
        assert!(
            ok(search("tokyo", 5))
                .iter()
                .all(|f| f.region == "東京都" || f.city.contains("東京"))
        );
        assert_eq!(ok(search("248", 3)).len(), 3);
        assert_eq!(ok(search("24", 3)), Vec::new());
        assert_eq!(ok(search("", 3)), Vec::new());
    }

    #[test]
    fn damaged_data_is_rejected() {
        assert_eq!(parse(b"FADJ").err(), Some(AddAssistError::Malformed));
        assert_eq!(
            parse(b"NOPE\x01\x00").err(),
            Some(AddAssistError::Malformed)
        );
        assert!(snapshot().is_ok_and(|s| s.len() == 10));
    }
}
