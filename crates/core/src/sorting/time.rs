// SPDX-License-Identifier: GPL-3.0-only
//! When a photo was taken, and where that time came from (FR-141 S4): EXIF (with or without
//! its offset), a date in the file's name, or the file's own time, which a copy may rewrite.
//! The file's time is trusted only when the library shows it can be: most of its dated photos
//! carry a file time within a time zone of their EXIF time (10,755 of 10,756 on the owner's
//! NAS copy, 2026-11-17).

use chrono::{NaiveDate, NaiveDateTime};

/// Where a photo's time came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeSource {
    /// EXIF's capture time with its offset from UTC.
    Exif,
    /// EXIF's capture time, its time zone unknown.
    ExifNoOffset,
    /// A date written in the file's name (`2020-06-18 21-51-22`, `PXL_20220801_104234`).
    FileName,
    /// The file's modification time; `confirmed` when the library's dated photos agree with
    /// theirs.
    FileTime { confirmed: bool },
}

/// The date and time in a file's name, if it holds one: `YYYYMMDDHHMMSS`, `YYYYMMDD` with an
/// optional `HHMMSS` after one separator, or `YYYY-MM-DD` with an optional `HH-MM-SS`. A date
/// alone reads as noon. Digit runs of other lengths (a scanner's frame numbers) never match.
pub fn name_date(file_name: &str) -> Option<NaiveDateTime> {
    let stem = file_name.rsplit_once('.').map_or(file_name, |(s, _)| s);
    let bytes = stem.as_bytes();
    // Runs of digits, with whether one separator joins each to the next.
    let mut runs: Vec<(&str, bool)> = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            let joined = matches!(bytes.get(i), Some(b'-' | b'_' | b' ' | b'.'))
                && bytes.get(i + 1).is_some_and(u8::is_ascii_digit);
            runs.push((&stem[start..i], joined));
        } else {
            i += 1;
        }
    }
    (0..runs.len()).find_map(|at| from_runs(&runs[at..]))
}

fn from_runs(runs: &[(&str, bool)]) -> Option<NaiveDateTime> {
    let num = |s: &str| s.parse::<u32>().ok();
    let (first, joined) = *runs.first()?;
    let (date, rest) = match first.len() {
        14 => {
            let d = ymd(&first[..4], &first[4..6], &first[6..8])?;
            return d.and_hms_opt(
                num(&first[8..10])?,
                num(&first[10..12])?,
                num(&first[12..])?,
            );
        }
        8 => (ymd(&first[..4], &first[4..6], &first[6..])?, 1),
        4 if joined && runs.get(1).is_some_and(|r| r.0.len() == 2 && r.1) => {
            let (m, d) = (runs.get(1)?.0, runs.get(2)?.0);
            (d.len().eq(&2).then(|| ymd(first, m, d))??, 3)
        }
        _ => return None,
    };
    let time = match rest {
        1 if joined => runs
            .get(1)
            .filter(|r| r.0.len() == 6)
            .and_then(|r| date.and_hms_opt(num(&r.0[..2])?, num(&r.0[2..4])?, num(&r.0[4..])?)),
        3 if runs.get(2).is_some_and(|r| r.1) => {
            let parts: Vec<&str> = runs.get(3..6)?.iter().map(|r| r.0).collect();
            (parts.iter().all(|p| p.len() == 2) && runs[3].1 && runs[4].1)
                .then(|| date.and_hms_opt(num(parts[0])?, num(parts[1])?, num(parts[2])?))
                .flatten()
        }
        _ => None,
    };
    time.or_else(|| date.and_hms_opt(12, 0, 0))
}

fn ymd(y: &str, m: &str, d: &str) -> Option<NaiveDate> {
    let y: i32 = y.parse().ok()?;
    (1990..=2100)
        .contains(&y)
        .then(|| NaiveDate::from_ymd_opt(y, m.parse().ok()?, d.parse().ok()?))
        .flatten()
}

/// Whether the library's file times can be trusted: at least `share` of its photos with an
/// EXIF time (camera-local seconds) have a file time (UTC seconds) within `window_s` of it.
pub fn file_time_confirmed(
    pairs: impl Iterator<Item = (i64, i64)>,
    window_s: i64,
    share: f32,
) -> bool {
    let (mut near, mut all) = (0u32, 0u32);
    for (exif, file) in pairs {
        all += 1;
        near += u32::from(exif.abs_diff(file) <= window_s.unsigned_abs());
    }
    #[allow(clippy::cast_precision_loss)]
    let ratio = near as f32 / all.max(1) as f32;
    all > 0 && ratio >= share
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(s: &str) -> Option<String> {
        name_date(s).map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
    }

    #[test]
    fn fr_141_dates_in_file_names() {
        let cases = [
            ("2020-06-18 21-51-22_5078.jpeg", "2020-06-18 21:51:22"),
            (
                "41002001PH20220801-104234-000023.JPG",
                "2022-08-01 10:42:34",
            ),
            ("PXL_20230105_093012345.jpg", "2023-01-05 12:00:00"),
            ("PXL_20230105_093012.jpg", "2023-01-05 09:30:12"),
            ("Screenshot_2021-03-04-18-22-01.png", "2021-03-04 18:22:01"),
            ("IMG-20220127-WA0001.jpg", "2022-01-27 12:00:00"),
            ("20190704123000.jpg", "2019-07-04 12:30:00"),
        ];
        for (name, want) in cases {
            assert_eq!(at(name).as_deref(), Some(want), "{name}");
        }
    }

    #[test]
    fn fr_141_numbers_that_are_not_dates() {
        for name in [
            "000039550038.jpeg",
            "IMG_4455.JPEG",
            "0012_###.jpeg",
            "B1152873-7DF4-40A9-86BF-856E5228F500.jpg",
            "20221332_101010.jpg",
            "1890-01-01.jpg",
        ] {
            assert_eq!(at(name), None, "{name}");
        }
    }

    #[test]
    fn fr_141_file_times_are_trusted_only_when_they_agree() {
        let h = 3600;
        let agree = (0..100).map(|i| (i * 86_400, i * 86_400 + 7 * h));
        assert!(file_time_confirmed(agree, 14 * h, 0.95));
        let copied = (0..100).map(|i| (i * 86_400, 1_700_000_000));
        assert!(!file_time_confirmed(copied, 14 * h, 0.95));
        assert!(!file_time_confirmed(std::iter::empty(), 14 * h, 0.95));
    }
}
