// SPDX-License-Identifier: GPL-3.0-only
//! The gallery's sections under RollSense's time picks (FR-141 C2, owner 2026-11-19): a day
//! ticked splits the grid by its events, a month or a year by its days, each run of photos
//! under a header with its local times (FR-145). The UI draws the headers; the runs are here.

use std::collections::HashMap;

use serde::Serialize;
use specta::Type;

use super::sense::{minute_of_day, place_name};
use super::{CullFilter, Library};
use crate::sorting::facets::SortPick;

/// What a section's header names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum SectionKind {
    Day,
    Event,
}

/// A run of the grid's photos under one header.
#[derive(Clone, Debug, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GallerySection {
    pub kind: SectionKind,
    /// Its first photo's position in the grid, and how many follow under the header.
    pub start: u32,
    pub count: u32,
    /// Its local day, days since 1970.
    pub day: i32,
    /// Minutes after local midnight of its first and last photo, in the place's time.
    pub start_min: u16,
    pub end_min: u16,
    /// The place most of its photos were taken.
    pub place: Option<String>,
}

impl Library {
    /// The sections of the photos `filter` keeps, in their order: by event when a day or an
    /// event is picked, by day when a month or a year is, none without a time pick.
    pub fn gallery_sections(&self, filter: &CullFilter) -> Vec<GallerySection> {
        let picks = &filter.sort;
        let kind = if picks
            .iter()
            .any(|p| matches!(p, SortPick::Day { .. } | SortPick::Event { .. }))
        {
            SectionKind::Event
        } else if picks
            .iter()
            .any(|p| matches!(p, SortPick::Year { .. } | SortPick::Month { .. }))
        {
            SectionKind::Day
        } else {
            return Vec::new();
        };
        let idx = self.sort_index();
        let rows: Vec<usize> = self
            .filtered_ids(filter)
            .into_iter()
            .filter_map(|id| self.index(id))
            .collect();
        // A photo outside every event keeps to its day.
        let key = |i: usize| match kind {
            SectionKind::Day => (idx.day[i], u32::MAX),
            SectionKind::Event => (idx.day[i], idx.event[i]),
        };
        let mut out = Vec::new();
        let mut at = 0;
        while let Some(&first) = rows.get(at) {
            let len = rows[at..]
                .iter()
                .take_while(|&&i| key(i) == key(first))
                .count();
            let run = &rows[at..at + len];
            let last = run.last().copied().unwrap_or(first);
            let mut seen: HashMap<u32, usize> = HashMap::new();
            for p in run.iter().filter_map(|&i| idx.place[i]) {
                *seen.entry(p).or_default() += 1;
            }
            out.push(GallerySection {
                kind,
                start: u32::try_from(at).unwrap_or(u32::MAX),
                count: u32::try_from(len).unwrap_or(u32::MAX),
                day: idx.day[first],
                start_min: minute_of_day(idx.t[first]),
                end_min: minute_of_day(idx.t[last]),
                place: place_name(
                    seen.into_iter()
                        .max_by_key(|&(p, c)| (c, std::cmp::Reverse(p)))
                        .map(|(p, _)| p),
                ),
            });
            at += len;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::import::ImportedFile;
    use crate::metadata::PhotoMeta;

    fn roll(shots: &[(u32, u32, u32)]) -> Library {
        let mut lib = Library::new();
        lib.set_home_zone(Some(crate::time_base::Zone::UTC));
        let f = lib.add_folder(Path::new("/roll"));
        for (k, &(day, hour, minute)) in shots.iter().enumerate() {
            let meta = PhotoMeta {
                capture: chrono::NaiveDate::from_ymd_opt(2022, 7, day)
                    .and_then(|d| d.and_hms_opt(hour, minute, 0)),
                make: Some("Canon".into()),
                orientation: 1,
                ..PhotoMeta::default()
            };
            let file = ImportedFile {
                path: PathBuf::from(format!("/roll/{k}.jpg")),
                size: 1,
                mtime: 0,
            };
            lib.push(f, &file, &meta);
        }
        lib
    }

    fn july(day: u32) -> i32 {
        crate::calendar::days_from_ymd(2022, 7, day).unwrap_or(0)
    }

    fn sections(lib: &Library, sort: Vec<SortPick>) -> Vec<(SectionKind, u32, u32, i32, u16, u16)> {
        let filter = CullFilter {
            sort,
            ..CullFilter::default()
        };
        lib.gallery_sections(&filter)
            .into_iter()
            .map(|s| (s.kind, s.start, s.count, s.day, s.start_min, s.end_min))
            .collect()
    }

    #[test]
    fn fr_141_the_gallery_splits_by_event_for_days_and_by_day_for_a_month() {
        let lib = roll(&[(3, 9, 0), (3, 9, 5), (20, 9, 0), (20, 9, 1), (20, 9, 2)]);
        // No time pick: one grid, no headers.
        assert_eq!(sections(&lib, Vec::new()), Vec::new());
        // The month: one header a day, with its first and last photo's local times.
        let month = vec![SortPick::Month {
            year: 2022,
            month: 7,
        }];
        assert_eq!(
            sections(&lib, month),
            vec![
                (SectionKind::Day, 0, 2, july(3), 540, 545),
                (SectionKind::Day, 2, 3, july(20), 540, 542),
            ]
        );
        // Two days ticked: their events, each under its own header.
        let days = vec![
            SortPick::Day { day: july(3) },
            SortPick::Day { day: july(20) },
        ];
        let got = sections(&lib, days);
        assert_eq!(got.len(), 2);
        assert_eq!(
            got.iter().map(|s| (s.0, s.2)).collect::<Vec<_>>(),
            [(SectionKind::Event, 2), (SectionKind::Event, 3)]
        );
        // The runs cover the grid, in order, with nothing left out.
        assert_eq!(got[1].1, 2);
    }
}
