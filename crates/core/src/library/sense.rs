// SPDX-License-Identifier: GPL-3.0-only
//! RollSense in the library (FR-141 S7): what the pixels' pass found kept beside each photo,
//! what each photo is under the view's picks (`SortIndex`, worked out once after a change),
//! the filter's test for one pick, and the facets the view lists with their counts. A child
//! of `library`, so it reads the columns without making them public.

use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use chrono::Datelike;

use super::{Library, NO_DATE, PhotoId};
use crate::sorting::facets::{
    Count, CountryFacet, EventFacet, Facets, MonthFacet, PlaceFacet, SortPick, TripFacet,
    VisitFacet, YearFacet,
};
use crate::sorting::pass::Pixels;

/// What each photo is under RollSense's picks, by index.
#[derive(Debug, Default)]
pub(crate) struct SortIndex {
    t: Vec<i64>,
    year: Vec<i32>,
    month: Vec<u8>,
    /// The id of its event's first photo, `u32::MAX` outside every event.
    event: Vec<u32>,
    trip: Vec<Option<i32>>,
    visit: Vec<Option<(u8, i32)>>,
    place: Vec<Option<u32>>,
    in_stack: Vec<bool>,
    /// Not in a stack, or its cover.
    shown: Vec<bool>,
}

fn year_month(t: i64) -> (i32, u8) {
    chrono::DateTime::from_timestamp(t, 0).map_or((1970, 1), |d| {
        (d.year(), u8::try_from(d.month()).unwrap_or(1))
    })
}

fn place_name(id: Option<u32>) -> Option<String> {
    id.and_then(|id| crate::geocode::by_id(id).ok().flatten())
        .map(|p| p.name)
}

/// Days since 1970 fit an `i32` for 5.8 million years; the UI's numbers stay exact.
fn day32(day: i64) -> i32 {
    i32::try_from(day).unwrap_or(if day < 0 { i32::MIN } else { i32::MAX })
}

fn count(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

impl Library {
    /// The pass's result for a photo, kept if the photo is still the one read (a library
    /// replaced meanwhile has other photos under the same ids). Returns whether it was kept.
    pub fn set_pixels(&mut self, id: PhotoId, path: &Path, pixels: &Pixels) -> bool {
        let Some(i) = self.index(id) else {
            return false;
        };
        if self.record(id).is_none_or(|r| r.path != path) {
            return false;
        }
        let keys = crate::sorting::clip::subject_keys();
        let best = crate::sorting::clip::subjects(&pixels.embedding);
        self.subject[i] = best
            .first()
            .filter(|(_, p)| *p >= crate::sorting::subject_min_p())
            .and_then(|(k, _)| keys.iter().position(|x| x == k))
            .and_then(|k| u8::try_from(k).ok());
        self.hash[i] = Some(pixels.hash);
        self.analysed[i] = true;
        self.sort_index = std::sync::OnceLock::new();
        true
    }

    /// Pairs of photos whose hashes are close and which are not one capture: the
    /// candidates the shell proves on their pictures (FR-015).
    pub fn duplicate_candidates(&self) -> Vec<(PhotoId, PhotoId)> {
        let rules = crate::sorting::dup_rules();
        crate::sorting::dupes::candidate_pairs(&self.hash, rules.max_distance, self.stacks())
            .into_iter()
            .filter_map(|(a, b)| {
                Some((
                    PhotoId(u32::try_from(a).ok()?),
                    PhotoId(u32::try_from(b).ok()?),
                ))
            })
            .collect()
    }

    /// The photos proven duplicates, replacing the last set.
    pub fn set_duplicates(&mut self, ids: &[PhotoId]) {
        self.duplicate.iter_mut().for_each(|d| *d = false);
        for id in ids {
            if let Some(i) = self.index(*id) {
                self.duplicate[i] = true;
            }
        }
        self.sort_index = std::sync::OnceLock::new();
    }

    pub(crate) fn sort_index(&self) -> &SortIndex {
        self.sort_index.get_or_init(|| {
            let n = self.len();
            let roll = self.roll();
            let t: Vec<i64> = (0..n)
                .map(|i| {
                    if self.manual_date[i] == NO_DATE {
                        self.capture[i]
                    } else {
                        self.manual_date[i]
                    }
                })
                .collect();
            let (year, month) = t.iter().map(|&t| year_month(t)).unzip();
            let mut idx = SortIndex {
                year,
                month,
                event: vec![u32::MAX; n],
                trip: vec![None; n],
                visit: vec![None; n],
                place: roll.places.iter().map(|p| p.map(|p| p.place_id)).collect(),
                in_stack: vec![false; n],
                shown: vec![true; n],
                t,
            };
            for e in &roll.events {
                let first = e.first().map_or(u32::MAX, |&f| count(f));
                for &m in e {
                    idx.event[m] = first;
                }
            }
            for tr in &roll.trips {
                for &m in &tr.members {
                    idx.trip[m] = Some(day32(tr.first_day));
                }
            }
            for v in &roll.visits {
                let year = year_month(v.first_day * 86_400).0;
                for &m in &v.members {
                    idx.visit[m] = Some((v.home, year));
                }
            }
            for st in self.stacks() {
                for &m in &st.members {
                    let m = m as usize;
                    idx.in_stack[m] = true;
                    idx.shown[m] = m == st.cover as usize;
                }
            }
            idx
        })
    }

    /// Whether RollSense's pick keeps the photo (the filter's test, `CullFilter::sort`).
    pub(crate) fn picks(&self, pick: &SortPick, id: PhotoId) -> bool {
        let Some(i) = self.index(id) else {
            return false;
        };
        let idx = self.sort_index();
        if pick.folds_stacks() && !idx.shown[i] {
            return false;
        }
        match pick {
            SortPick::Source { class } => self.source(id).is_some_and(|s| s.class == *class),
            SortPick::Subject { key } => {
                let keys = crate::sorting::clip::subject_keys();
                self.subject[i].is_some_and(|s| keys.get(usize::from(s)) == Some(&key.as_str()))
            }
            SortPick::NoSubject => self.analysed[i] && self.subject[i].is_none(),
            SortPick::Year { year } => idx.year[i] == *year,
            SortPick::Month { year, month } => idx.year[i] == *year && idx.month[i] == *month,
            SortPick::Event { first } => idx.event[i] == *first,
            SortPick::Trip { first_day } => idx.trip[i] == Some(*first_day),
            SortPick::Visit { home, year } => idx.visit[i] == Some((*home, *year)),
            SortPick::Place { id } => idx.place[i] == Some(*id),
            SortPick::Duplicates => self.duplicate[i],
            SortPick::Stacks => idx.in_stack[i],
        }
    }

    /// Everything RollSense lists, with counts.
    pub fn facets(&self) -> Facets {
        let n = self.len();
        let idx = self.sort_index();
        let roll = self.roll();
        let ids: Vec<PhotoId> = (0..n)
            .filter_map(|i| Some(PhotoId(u32::try_from(i).ok()?)))
            .collect();
        let mut sources: BTreeMap<crate::sorting::SourceClass, usize> = BTreeMap::new();
        for &id in &ids {
            if let Some(s) = self.source(id) {
                *sources.entry(s.class).or_default() += 1;
            }
        }
        let keys = crate::sorting::clip::subject_keys();
        let mut subjects: BTreeMap<u8, usize> = BTreeMap::new();
        for s in self.subject.iter().flatten() {
            *subjects.entry(*s).or_default() += 1;
        }
        let mut subjects: Vec<Count<String>> = subjects
            .into_iter()
            .filter_map(|(k, c)| {
                Some(Count {
                    value: (*keys.get(usize::from(k))?).to_string(),
                    count: count(c),
                })
            })
            .collect();
        subjects.sort_by_key(|c| std::cmp::Reverse(c.count));
        let place_ids: Vec<Option<u32>> = idx.place.clone();
        Facets {
            total: count(n),
            analysed: count(self.analysed.iter().filter(|a| **a).count()),
            sources: sources
                .into_iter()
                .map(|(value, c)| Count {
                    value,
                    count: count(c),
                })
                .collect(),
            subjects,
            no_subject: count(
                (0..n)
                    .filter(|&i| self.analysed[i] && self.subject[i].is_none())
                    .count(),
            ),
            years: self.years(idx, &roll.events, &place_ids),
            trips: roll
                .trips
                .iter()
                .map(|t| TripFacet {
                    first_day: day32(t.first_day),
                    last_day: day32(t.last_day),
                    count: count(t.members.len()),
                    place: place_name(crate::sorting::trips::main_place(t, &place_ids)),
                })
                .collect(),
            visits: visits(&roll.visits, &place_ids),
            countries: countries(&roll.places),
            duplicates: count(self.duplicate.iter().filter(|d| **d).count()),
            stacks: count(self.stacks().len()),
        }
    }

    /// Years › months › events, newest first; an event under the month of its first photo.
    fn years(
        &self,
        idx: &SortIndex,
        events: &[Vec<usize>],
        places: &[Option<u32>],
    ) -> Vec<YearFacet> {
        let mut months: BTreeMap<(i32, u8), (usize, Vec<EventFacet>)> = BTreeMap::new();
        for i in 0..self.len() {
            months.entry((idx.year[i], idx.month[i])).or_default().0 += 1;
        }
        for e in events {
            let (Some(&first), Some(&last)) = (e.first(), e.last()) else {
                continue;
            };
            let mut seen: HashMap<u32, usize> = HashMap::new();
            for p in e.iter().filter_map(|&m| places[m]) {
                *seen.entry(p).or_default() += 1;
            }
            let place = seen
                .into_iter()
                .max_by_key(|&(p, c)| (c, std::cmp::Reverse(p)))
                .map(|(p, _)| p);
            months
                .entry((idx.year[first], idx.month[first]))
                .or_default()
                .1
                .push(EventFacet {
                    first: count(first),
                    first_day: day32(idx.t[first].div_euclid(86_400)),
                    last_day: day32(idx.t[last].div_euclid(86_400)),
                    count: count(e.len()),
                    place: place_name(place),
                });
        }
        let mut years: BTreeMap<i32, YearFacet> = BTreeMap::new();
        for ((year, month), (c, events)) in months.into_iter().rev() {
            let y = years.entry(year).or_insert_with(|| YearFacet {
                year,
                count: 0,
                months: Vec::new(),
            });
            y.count += count(c);
            y.months.push(MonthFacet {
                month,
                count: count(c),
                events,
            });
        }
        years.into_values().rev().collect()
    }
}

/// The visits to each other home, per year, with the home's most frequent place.
fn visits(visits: &[crate::sorting::trips::Visit], places: &[Option<u32>]) -> Vec<VisitFacet> {
    let mut by: BTreeMap<(u8, i32), (u32, usize)> = BTreeMap::new();
    let mut home_places: HashMap<u8, HashMap<u32, usize>> = HashMap::new();
    for v in visits {
        let year = year_month(v.first_day * 86_400).0;
        let e = by.entry((v.home, year)).or_default();
        e.0 += 1;
        e.1 += v.members.len();
        for p in v
            .members
            .iter()
            .filter_map(|&m| places.get(m).copied().flatten())
        {
            *home_places.entry(v.home).or_default().entry(p).or_default() += 1;
        }
    }
    by.into_iter()
        .rev()
        .map(|((home, year), (visits, c))| VisitFacet {
            home,
            year,
            visits,
            count: count(c),
            place: place_name(home_places.get(&home).and_then(|m| {
                m.iter()
                    .max_by_key(|&(p, c)| (*c, std::cmp::Reverse(*p)))
                    .map(|(p, _)| *p)
            })),
        })
        .collect()
}

/// Countries › places, most photos first.
fn countries(places: &[Option<crate::sorting::locations::Located>]) -> Vec<CountryFacet> {
    let mut by: BTreeMap<String, CountryFacet> = BTreeMap::new();
    for g in crate::sorting::locations::by_place(places) {
        let Some(p) = crate::geocode::by_id(g.place_id).ok().flatten() else {
            continue;
        };
        let c = by.entry(p.country.clone()).or_insert_with(|| CountryFacet {
            country: p.country.clone(),
            count: 0,
            places: Vec::new(),
        });
        c.count += count(g.members.len());
        c.places.push(PlaceFacet {
            id: g.place_id,
            name: p.name,
            count: count(g.members.len()),
        });
    }
    let mut out: Vec<CountryFacet> = by.into_values().collect();
    out.sort_by_key(|c| std::cmp::Reverse(c.count));
    out
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::import::ImportedFile;
    use crate::metadata::PhotoMeta;
    use crate::sorting::SourceClass;

    fn shot(day: u32, hour: u32) -> PhotoMeta {
        PhotoMeta {
            capture: chrono::NaiveDate::from_ymd_opt(2022, 7, day)
                .and_then(|d| d.and_hms_opt(hour, 0, 0)),
            make: Some("Canon".into()),
            orientation: 1,
            ..PhotoMeta::default()
        }
    }

    #[test]
    fn fr_141_rollsense_picks_filter_the_grid_and_facets_count_them() {
        let mut lib = Library::new();
        let f = lib.add_folder(Path::new("/roll"));
        for (k, (day, hour)) in [(3, 9), (3, 10), (20, 9)].into_iter().enumerate() {
            let file = ImportedFile {
                path: PathBuf::from(format!("/roll/{k}.jpg")),
                size: 1,
                mtime: 0,
            };
            lib.push(f, &file, &shot(day, hour));
        }
        let mut filter = super::super::CullFilter {
            sort: Some(SortPick::Source {
                class: SourceClass::OtherCamera,
            }),
            ..super::super::CullFilter::default()
        };
        assert_eq!(lib.filtered_ids(&filter).len(), 3);
        filter.sort = Some(SortPick::Month {
            year: 2022,
            month: 7,
        });
        assert_eq!(lib.filtered_ids(&filter).len(), 3);
        let facets = lib.facets();
        assert_eq!(facets.total, 3);
        assert_eq!(facets.years[0].months[0].events.len(), 2);
        let first = facets.years[0].months[0].events[0].first;
        filter.sort = Some(SortPick::Event { first });
        assert_eq!(lib.filtered_ids(&filter).len(), 2);
        // A subject set by the pass, kept only for the photo still at that path.
        let cat = crate::sorting::clip::labels().labels["cat"].clone();
        let pixels = Pixels {
            hash: 1,
            embedding: cat,
        };
        assert!(!lib.set_pixels(PhotoId(0), Path::new("/elsewhere.jpg"), &pixels));
        assert!(lib.set_pixels(PhotoId(0), Path::new("/roll/0.jpg"), &pixels));
        filter.sort = Some(SortPick::Subject { key: "cat".into() });
        assert_eq!(lib.filtered_ids(&filter), vec![PhotoId(0)]);
        assert_eq!(lib.facets().subjects[0].value, "cat");
        // Old filters without a pick still read.
        let old: super::super::CullFilter =
            serde_json::from_str(r#"{"flag":"all","minRating":0}"#).unwrap_or_default();
        assert_eq!(old.sort, None);
    }
}
