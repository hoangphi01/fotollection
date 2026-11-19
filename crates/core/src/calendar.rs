// SPDX-License-Identifier: GPL-3.0-only
//! A month laid out in weeks (FR-141 C1): calendar arithmetic only, nothing about photos, so
//! RollSense's Time card, the printed calendar (FR-083) and a journal (FR-049) share it. Days
//! are days since 1970, the numbering RollSense's local days use (FR-145).

use std::ops::RangeInclusive;

use chrono::{Datelike, Days, NaiveDate, Weekday};
use serde::Serialize;
use specta::Type;

/// The years a grid is laid out for.
pub const YEARS: RangeInclusive<i32> = 1900..=2100;

/// 1970-01-01 counted from 0001-01-01 (`num_days_from_ce`).
const EPOCH_CE: i32 = 719_163;

/// One cell of a month's grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CalDay {
    /// Days since 1970.
    pub day: i32,
    /// The day of its month, 1…31.
    pub date: u8,
    /// False for the days of the months before and after that fill the first and last weeks.
    pub in_month: bool,
}

/// Days since 1970 of a date, `None` for a date that does not exist.
pub fn days_from_ymd(year: i32, month: u32, date: u32) -> Option<i32> {
    NaiveDate::from_ymd_opt(year, month, date).map(days_of)
}

/// The year, month and day of a day since 1970.
pub fn ymd_from_days(day: i32) -> Option<(i32, u32, u32)> {
    NaiveDate::from_num_days_from_ce_opt(day.checked_add(EPOCH_CE)?)
        .map(|d| (d.year(), d.month(), d.day()))
}

fn days_of(d: NaiveDate) -> i32 {
    d.num_days_from_ce() - EPOCH_CE
}

/// The weeks of `month` in `year`, each starting on `first_weekday`: 4 to 6 full rows, the
/// days of the neighbour months marked `in_month: false`. `None` for a month outside 1…12 or
/// a year outside [`YEARS`].
pub fn month_grid(year: i32, month: u32, first_weekday: Weekday) -> Option<Vec<[CalDay; 7]>> {
    if !YEARS.contains(&year) {
        return None;
    }
    let start = NaiveDate::from_ymd_opt(year, month, 1)?;
    let next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }?;
    let lead =
        (start.weekday().num_days_from_monday() + 7 - first_weekday.num_days_from_monday()) % 7;
    let len = u32::try_from((next - start).num_days()).ok()?;
    let cells = (lead + len).div_ceil(7) * 7;
    let mut d = start.checked_sub_days(Days::new(u64::from(lead)))?;
    let mut all = Vec::with_capacity(cells as usize);
    for _ in 0..cells {
        all.push(CalDay {
            day: days_of(d),
            date: u8::try_from(d.day()).ok()?,
            in_month: d.month() == month,
        });
        d = d.succ_opt()?;
    }
    // `cells` is whole weeks, so nothing is left over.
    Some(all.as_chunks::<7>().0.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dates(grid: &[[CalDay; 7]]) -> Vec<Vec<(u8, bool)>> {
        grid.iter()
            .map(|w| w.iter().map(|c| (c.date, c.in_month)).collect())
            .collect()
    }

    fn in_month(grid: &[[CalDay; 7]]) -> Vec<u8> {
        grid.iter()
            .flatten()
            .filter(|c| c.in_month)
            .map(|c| c.date)
            .collect()
    }

    #[test]
    fn fr_141_days_since_1970_both_ways() {
        assert_eq!(days_from_ymd(1970, 1, 1), Some(0));
        assert_eq!(days_from_ymd(1969, 12, 31), Some(-1));
        assert_eq!(days_from_ymd(2024, 2, 29), Some(19_782));
        assert_eq!(days_from_ymd(2023, 2, 29), None);
        assert_eq!(ymd_from_days(19_782), Some((2024, 2, 29)));
        assert_eq!(ymd_from_days(-1), Some((1969, 12, 31)));
    }

    #[test]
    fn fr_141_february_in_a_leap_year_and_not() {
        let leap = month_grid(2024, 2, Weekday::Mon).unwrap_or_default();
        assert_eq!(in_month(&leap), (1..=29).collect::<Vec<u8>>());
        // 2024-02-01 is a Thursday: Mon 29, Tue 30, Wed 31 January before it.
        assert_eq!(
            dates(&leap)[0],
            vec![
                (29, false),
                (30, false),
                (31, false),
                (1, true),
                (2, true),
                (3, true),
                (4, true)
            ]
        );
        assert_eq!(leap[0][3].day, days_from_ymd(2024, 2, 1).unwrap_or(0));
        let plain = month_grid(2023, 2, Weekday::Mon).unwrap_or_default();
        assert_eq!(in_month(&plain), (1..=28).collect::<Vec<u8>>());
        // Every row is a week of consecutive days.
        for grid in [&leap, &plain] {
            let days: Vec<i32> = grid.iter().flatten().map(|c| c.day).collect();
            assert!(days.windows(2).all(|w| w[1] == w[0] + 1));
        }
    }

    #[test]
    fn fr_141_the_week_starts_on_the_languages_first_day() {
        // 1 September 2024 is a Sunday.
        let monday = month_grid(2024, 9, Weekday::Mon).unwrap_or_default();
        assert_eq!(
            monday[0][6],
            CalDay {
                day: days_from_ymd(2024, 9, 1).unwrap_or(0),
                date: 1,
                in_month: true,
            }
        );
        assert!(monday[0][..6].iter().all(|c| !c.in_month));
        assert_eq!(monday[0][0].date, 26);
        assert_eq!(monday.len(), 6);
        let sunday = month_grid(2024, 9, Weekday::Sun).unwrap_or_default();
        assert_eq!((sunday[0][0].date, sunday[0][0].in_month), (1, true));
        assert_eq!(sunday.len(), 5);
        // The last row ends with October's first days.
        assert_eq!(
            dates(&sunday)[4],
            vec![
                (29, true),
                (30, true),
                (1, false),
                (2, false),
                (3, false),
                (4, false),
                (5, false)
            ]
        );
    }

    #[test]
    fn fr_141_four_five_or_six_rows() {
        let rows = |y, m, w| month_grid(y, m, w).map(|g| g.len());
        // February 2021 starts on a Monday and has 28 days: exactly four weeks.
        assert_eq!(rows(2021, 2, Weekday::Mon), Some(4));
        assert_eq!(rows(2021, 2, Weekday::Sun), Some(5));
        assert_eq!(rows(2024, 2, Weekday::Mon), Some(5));
        // March 2025 starts on a Saturday with 31 days.
        assert_eq!(rows(2025, 3, Weekday::Mon), Some(6));
        assert_eq!(rows(2025, 3, Weekday::Sat), Some(5));
        // December rolls into the next year.
        let dec = month_grid(2100, 12, Weekday::Mon).unwrap_or_default();
        assert_eq!(in_month(&dec).len(), 31);
    }

    #[test]
    fn fr_141_an_invalid_month_or_year_has_no_grid() {
        assert_eq!(month_grid(2024, 0, Weekday::Mon), None);
        assert_eq!(month_grid(2024, 13, Weekday::Mon), None);
        assert_eq!(month_grid(1899, 12, Weekday::Mon), None);
        assert_eq!(month_grid(2101, 1, Weekday::Mon), None);
        assert!(month_grid(1900, 1, Weekday::Mon).is_some());
    }
}
