// SPDX-License-Identifier: GPL-3.0-only
//! The user's changes to the trips (FR-012, FR-141 S7c; owner 2026-11-18): rename, split after
//! a day, merge with the trip before. Kept as [`TripEdits`] in the project, applied on the
//! trips found each time they are worked out.

use super::Library;
use super::sense::{day32, place_name};
use crate::sorting::facets::TripDay;
use crate::sorting::trips::{Trip, TripEdits};

/// Longest trip name, in characters, as a place's (FR-005).
const MAX_TRIP_NAME: usize = 80;

/// Why a change to the trips was refused; each has its `ERR-` code (`ERROR_CONTROL.md`).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TripEditError {
    #[error("No trip starts on that day")]
    Unknown,
    #[error("A trip is split after one of its days but its last")]
    BadDay,
    #[error("The first trip has none before it to merge with")]
    NothingBefore,
}

impl TripEditError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Unknown => "trip.unknown",
            Self::BadDay => "trip.bad_day",
            Self::NothingBefore => "trip.nothing_before",
        }
    }
}

impl Library {
    pub fn trip_edits(&self) -> &TripEdits {
        &self.trip_edits
    }

    /// The edits read from a project, or none.
    pub fn set_trip_edits(&mut self, edits: TripEdits) {
        self.trip_edits = edits;
        self.forget_roll();
    }

    fn forget_roll(&mut self) {
        self.roll = std::sync::OnceLock::new();
        self.sort_index = std::sync::OnceLock::new();
    }

    fn trip(&self, first_day: i64) -> Result<(usize, &Trip), TripEditError> {
        self.roll()
            .trips
            .iter()
            .enumerate()
            .find(|(_, t)| t.first_day == first_day)
            .ok_or(TripEditError::Unknown)
    }

    /// Names the trip that starts on `first_day`; `None` or an empty name brings the suggested
    /// one back. Trimmed, without control characters, cut to 80 characters.
    pub fn rename_trip(&mut self, first_day: i64, name: Option<&str>) -> Result<(), TripEditError> {
        self.trip(first_day)?;
        let name = name
            .map(|n| {
                n.trim()
                    .chars()
                    .filter(|c| !c.is_control())
                    .take(MAX_TRIP_NAME)
                    .collect::<String>()
            })
            .filter(|n| !n.is_empty());
        match name {
            Some(n) => self.trip_edits.names.insert(first_day, n),
            None => self.trip_edits.names.remove(&first_day),
        };
        self.forget_roll();
        Ok(())
    }

    /// Splits the trip that starts on `first_day` after `after_day`: undoes a join there if
    /// there is one, else cuts.
    pub fn split_trip(&mut self, first_day: i64, after_day: i64) -> Result<(), TripEditError> {
        let next = {
            let (_, trip) = self.trip(first_day)?;
            if after_day < trip.first_day || after_day >= trip.last_day {
                return Err(TripEditError::BadDay);
            }
            let day = self.member_day();
            trip.members
                .iter()
                .map(|&m| day(m))
                .filter(|&d| d > after_day)
                .min()
        };
        let Some(next) = next else {
            return Err(TripEditError::BadDay);
        };
        if !self.trip_edits.joins.remove(&next) {
            self.trip_edits.cuts.insert(after_day);
        }
        self.forget_roll();
        Ok(())
    }

    /// Merges the trip that starts on `first_day` into the one before it: undoes a cut
    /// between them if there is one, else joins.
    pub fn merge_trip(&mut self, first_day: i64) -> Result<(), TripEditError> {
        let (i, _) = self.trip(first_day)?;
        let prev_last = i
            .checked_sub(1)
            .and_then(|p| self.roll().trips.get(p))
            .map(|p| p.last_day)
            .ok_or(TripEditError::NothingBefore)?;
        let cut: Vec<i64> = self
            .trip_edits
            .cuts
            .range(prev_last..first_day)
            .copied()
            .collect();
        if cut.is_empty() {
            self.trip_edits.joins.insert(first_day);
        } else {
            for c in cut {
                self.trip_edits.cuts.remove(&c);
            }
        }
        self.forget_roll();
        Ok(())
    }

    /// The days of the trip that starts on `first_day`, for its page (S7c).
    pub fn trip_days(&self, first_day: i64) -> Result<Vec<TripDay>, TripEditError> {
        let (_, trip) = self.trip(first_day)?;
        let day = self.member_day();
        let mut by_day: std::collections::BTreeMap<i64, Vec<usize>> =
            std::collections::BTreeMap::new();
        for &m in &trip.members {
            by_day.entry(day(m)).or_default().push(m);
        }
        let places: Vec<Option<u32>> = self
            .roll()
            .places
            .iter()
            .map(|p| p.as_ref().map(|l| l.place_id))
            .collect();
        Ok(by_day
            .into_iter()
            .map(|(d, members)| {
                let one = Trip {
                    first_day: d,
                    last_day: d,
                    members,
                };
                TripDay {
                    day: day32(d),
                    count: u32::try_from(one.members.len()).unwrap_or(u32::MAX),
                    place: place_name(crate::sorting::trips::main_place(&one, &places)),
                }
            })
            .collect())
    }

    /// A photo's day (days since 1970, in its place's local time, FR-145), as the trips count it.
    fn member_day(&self) -> impl Fn(usize) -> i64 + '_ {
        move |i| self.local_seconds(i).div_euclid(86_400)
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::import::ImportedFile;
    use crate::metadata::PhotoMeta;

    /// Sixty days at home, then five days 200 km away: one trip of 20 photos.
    fn roll() -> Library {
        let mut lib = Library::new();
        let f = lib.add_folder(Path::new("/roll"));
        let start = chrono::NaiveDate::from_ymd_opt(2022, 3, 1).unwrap_or_default();
        let mut k = 0;
        let mut add = |day: i64, per_day: u32, gps: (f64, f64)| {
            for h in 0..per_day {
                let meta = PhotoMeta {
                    capture: (start + chrono::Days::new(u64::try_from(day).unwrap_or(0)))
                        .and_hms_opt(9 + h, 0, 0),
                    gps: Some(gps),
                    orientation: 1,
                    ..PhotoMeta::default()
                };
                let file = ImportedFile {
                    path: PathBuf::from(format!("/roll/{k}.jpg")),
                    size: 1,
                    mtime: 0,
                };
                k += 1;
                lib.push(f, &file, &meta);
            }
        };
        for d in 0..60 {
            add(d, 2, (47.5, 19.0));
        }
        for d in 60..65 {
            add(d, 4, (48.2, 16.4));
        }
        lib
    }

    #[test]
    fn fr_012_trips_are_renamed_split_and_merged_back() {
        let mut lib = roll();
        let trips = lib.facets(&[]).trips;
        assert_eq!(trips.len(), 1);
        let first = i64::from(trips[0].first_day);
        assert_eq!(lib.trip_days(first).map(|d| d.len()), Ok(5));
        lib.rename_trip(first, Some("  Vienna\u{7} with friends "))
            .ok();
        assert_eq!(
            lib.facets(&[]).trips[0].name.as_deref(),
            Some("Vienna with friends")
        );
        // Split after its second day: two trips, the name stays with the first.
        assert_eq!(lib.split_trip(first, first + 5), Err(TripEditError::BadDay));
        lib.split_trip(first, first + 1).ok();
        let two = lib.facets(&[]).trips;
        assert_eq!(two.iter().map(|t| t.count).collect::<Vec<_>>(), [8, 12]);
        assert_eq!(lib.merge_trip(first), Err(TripEditError::NothingBefore));
        // Merged back: the cut is gone, no join is left behind.
        lib.merge_trip(first + 2).ok();
        assert_eq!(lib.facets(&[]).trips.len(), 1);
        assert!(lib.trip_edits().cuts.is_empty() && lib.trip_edits().joins.is_empty());
        lib.rename_trip(first, None).ok();
        assert!(lib.trip_edits().is_empty());
    }

    #[test]
    fn fr_030_a_clock_correction_moves_a_photo_to_its_true_year_and_day() {
        // A camera two hours slow: 23:00 on New Year's Eve was really 01:00 on 1 January.
        let mut lib = Library::new();
        let f = lib.add_folder(Path::new("/cam"));
        let meta = PhotoMeta {
            capture: chrono::NaiveDate::from_ymd_opt(2022, 12, 31)
                .and_then(|d| d.and_hms_opt(23, 0, 0)),
            make: Some("Panasonic".into()),
            model: Some("DC-GX9".into()),
            orientation: 1,
            ..PhotoMeta::default()
        };
        let file = ImportedFile {
            path: PathBuf::from("/cam/a.jpg"),
            size: 1,
            mtime: 0,
        };
        let id = lib
            .push(f, &file, &meta)
            .unwrap_or_else(|| panic!("pushed"));
        assert_eq!(lib.facets(&[]).years[0].year, 2022);
        let body = lib.clock_of(id).unwrap_or_else(|| panic!("a body"));
        lib.set_clock(body, 7_200, None).ok();
        let years = lib.facets(&[]).years;
        assert_eq!(years[0].year, 2023);
        assert_eq!(years[0].months[0].month, 1);
    }
}
