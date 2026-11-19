// SPDX-License-Identifier: GPL-3.0-only
//! RollSense in the library (FR-141 S7): what the pixels' pass found kept beside each photo,
//! what each photo is under the view's picks (`SortIndex`, worked out once after a change),
//! the filter's test for one pick, and the facets the view lists with their counts. A child
//! of `library`, so it reads the columns without making them public.

use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use chrono::Datelike;

use super::{Library, PhotoId};
use crate::sorting::facets::{
    CleanupKind, CleanupRow, Count, CountryFacet, DayFacet, EventFacet, Facets, MonthFacet,
    PlaceFacet, SortPick, TripFacet, VisitFacet, YearFacet,
};
use crate::sorting::pass::Pixels;

/// What each photo is under RollSense's picks, by index.
#[derive(Debug, Default)]
pub(crate) struct SortIndex {
    pub(super) t: Vec<i64>,
    year: Vec<i32>,
    month: Vec<u8>,
    /// The local day, days since 1970.
    day: Vec<i32>,
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

pub(super) fn place_name(id: Option<u32>) -> Option<String> {
    id.and_then(|id| crate::geocode::by_id(id).ok().flatten())
        .map(|p| p.name)
}

/// Days since 1970 fit an `i32` for 5.8 million years; the UI's numbers stay exact.
pub(super) fn day32(day: i64) -> i32 {
    i32::try_from(day).unwrap_or(if day < 0 { i32::MIN } else { i32::MAX })
}

/// The subjects a photo is named by, one bit each in `clip::subject_keys`' order: its likeliest
/// at `subject_min_p` or more, and people too at `people_also_p` (a street with walkers).
fn named(best: &[(&'static str, f32)]) -> u16 {
    let keys = crate::sorting::clip::subject_keys();
    let bit = |k: &str| keys.iter().position(|x| *x == k).map_or(0, |b| 1u16 << b);
    let mut mask = best
        .first()
        .filter(|(_, p)| *p >= crate::sorting::subject_min_p())
        .map_or(0, |(k, _)| bit(k));
    if best
        .iter()
        .any(|(k, p)| *k == "people" && *p >= crate::sorting::people_also_p())
    {
        mask |= bit("people");
    }
    mask
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
        self.subject[i] = named(&crate::sorting::clip::subjects(&pixels.embedding));
        self.hash[i] = Some(pixels.hash);
        self.analysed[i] = true;
        self.sort_index = std::sync::OnceLock::new();
        true
    }

    /// What a pixels' pass changes, taken before it: a stopped pass puts it back, so it keeps
    /// nothing (owner, 2026-11-18) while its photos show as they are read.
    pub fn sense_snapshot(&self) -> SenseSnapshot {
        SenseSnapshot {
            subject: self.subject.clone(),
            hash: self.hash.clone(),
            analysed: self.analysed.clone(),
        }
    }

    /// Puts back a snapshot; false, and nothing changed, when the library was rebuilt since.
    pub fn restore_sense(&mut self, snapshot: SenseSnapshot) -> bool {
        if snapshot.analysed.len() != self.analysed.len() {
            return false;
        }
        self.subject = snapshot.subject;
        self.hash = snapshot.hash;
        self.analysed = snapshot.analysed;
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
            // The place's local time (FR-145): years and months where the photo was taken.
            let t: Vec<i64> = (0..n).map(|i| self.local_seconds(i)).collect();
            let (year, month) = t.iter().map(|&t| year_month(t)).unzip();
            let mut idx = SortIndex {
                year,
                month,
                day: t.iter().map(|&t| day32(t.div_euclid(86_400))).collect(),
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

    /// Whether RollSense's picks keep the photo (the filter's test, `CullFilter::sort`): every
    /// card picked from has one pick that keeps it.
    pub(crate) fn picks(&self, picks: &[SortPick], id: PhotoId) -> bool {
        let Some(i) = self.index(id) else {
            return false;
        };
        let idx = self.sort_index();
        if picks.iter().all(SortPick::folds_stacks) && !idx.shown[i] {
            return false;
        }
        (0..7u8).all(|card| {
            let mut of_card = picks.iter().filter(|p| p.card() == card).peekable();
            of_card.peek().is_none() || of_card.any(|p| self.picked(p, i, id))
        })
    }

    fn picked(&self, pick: &SortPick, i: usize, id: PhotoId) -> bool {
        let idx = self.sort_index();
        match pick {
            SortPick::Source { class } => self.source(id).is_some_and(|s| s.class == *class),
            SortPick::Subject { key } => {
                let keys = crate::sorting::clip::subject_keys();
                keys.iter()
                    .position(|k| k == key)
                    .is_some_and(|b| self.subject[i] & (1 << b) != 0)
            }
            SortPick::NoSubject => self.analysed[i] && self.subject[i] == 0,
            SortPick::Year { year } => idx.year[i] == *year,
            SortPick::Month { year, month } => idx.year[i] == *year && idx.month[i] == *month,
            SortPick::Event { first } => idx.event[i] == *first,
            SortPick::Day { day } => idx.day[i] == *day,
            SortPick::Trip { first_day } => idx.trip[i] == Some(*first_day),
            SortPick::Visit { home, year } => idx.visit[i] == Some((*home, *year)),
            SortPick::Place { id } => idx.place[i] == Some(*id),
            SortPick::Duplicates => self.duplicate[i],
            SortPick::Stacks => idx.in_stack[i],
        }
    }

    /// Everything RollSense lists, with counts, over the photos of `folders` (every folder
    /// when empty: the Workspace; owner, 2026-11-18: "only the tab chosen").
    pub fn facets(&self, folders: &[u16]) -> Facets {
        let n = self.len();
        let idx = self.sort_index();
        let roll = self.roll();
        let inn: Vec<bool> = (0..n)
            .map(|i| folders.is_empty() || folders.contains(&self.folder[i]))
            .collect();
        let ids: Vec<PhotoId> = (0..n)
            .filter(|&i| inn[i])
            .filter_map(|i| Some(PhotoId(u32::try_from(i).ok()?)))
            .collect();
        let only = |members: &[usize]| -> Vec<usize> {
            members.iter().copied().filter(|&m| inn[m]).collect()
        };
        let mut sources: BTreeMap<crate::sorting::SourceClass, usize> = BTreeMap::new();
        for &id in &ids {
            if let Some(s) = self.source(id) {
                *sources.entry(s.class).or_default() += 1;
            }
        }
        let keys = crate::sorting::clip::subject_keys();
        let mut subjects: BTreeMap<usize, usize> = BTreeMap::new();
        for mask in (0..n).filter(|&i| inn[i]).map(|i| self.subject[i]) {
            for b in (0..keys.len()).filter(|b| mask & (1 << b) != 0) {
                *subjects.entry(b).or_default() += 1;
            }
        }
        let mut subjects: Vec<Count<String>> = subjects
            .into_iter()
            .filter_map(|(k, c)| {
                Some(Count {
                    value: (*keys.get(k)?).to_string(),
                    count: count(c),
                })
            })
            .collect();
        subjects.sort_by_key(|c| std::cmp::Reverse(c.count));
        let place_ids: Vec<Option<u32>> = idx.place.clone();
        Facets {
            total: count(ids.len()),
            analysed: count((0..n).filter(|&i| inn[i] && self.analysed[i]).count()),
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
                    .filter(|&i| inn[i] && self.analysed[i] && self.subject[i] == 0)
                    .count(),
            ),
            years: self.years(idx, &roll.events, &place_ids, &inn),
            days: days(idx, &inn),
            trips: self.trip_facets(&roll.trips, &inn, &place_ids),
            visits: visits(
                &roll
                    .visits
                    .iter()
                    .map(|v| crate::sorting::trips::Visit {
                        members: only(&v.members),
                        ..v.clone()
                    })
                    .filter(|v| !v.members.is_empty())
                    .collect::<Vec<_>>(),
                &place_ids,
            ),
            countries: countries(
                &roll
                    .places
                    .iter()
                    .enumerate()
                    .map(|(i, p)| p.filter(|_| inn[i]))
                    .collect::<Vec<_>>(),
            ),
            duplicates: count((0..n).filter(|&i| inn[i] && self.duplicate[i]).count()),
            stacks: count(
                self.stacks()
                    .iter()
                    .filter(|st| st.members.iter().any(|&m| inn[m as usize]))
                    .count(),
            ),
            cleanup: self.cleanup(&inn, keys),
        }
    }

    /// The trips with photos in `inn`, each with the user's name if it has one (S7c).
    fn trip_facets(
        &self,
        trips: &[crate::sorting::trips::Trip],
        inn: &[bool],
        place_ids: &[Option<u32>],
    ) -> Vec<TripFacet> {
        trips
            .iter()
            .filter_map(|t| {
                let t = crate::sorting::trips::Trip {
                    members: t.members.iter().copied().filter(|&m| inn[m]).collect(),
                    ..t.clone()
                };
                (!t.members.is_empty()).then(|| TripFacet {
                    name: self.trip_edits.names.get(&t.first_day).cloned(),
                    first_day: day32(t.first_day),
                    last_day: day32(t.last_day),
                    count: count(t.members.len()),
                    place: place_name(crate::sorting::trips::main_place(&t, place_ids)),
                })
            })
            .collect()
    }

    /// The clean-up rows over the photos `inn` keeps (FR-141): each one's count and bytes, the
    /// rows with no photo left out.
    fn cleanup(&self, inn: &[bool], keys: &[&str]) -> Vec<CleanupRow> {
        use crate::sorting::SourceClass as C;
        let meme = keys.iter().position(|k| *k == "meme");
        let source = |class| SortPick::Source { class };
        let kinds = [
            CleanupKind::Duplicates,
            CleanupKind::Screenshots,
            CleanupKind::Saved,
            CleanupKind::Memes,
        ];
        kinds
            .into_iter()
            .filter_map(|kind| {
                let has = |i: usize| {
                    let class = u32::try_from(i)
                        .ok()
                        .and_then(|id| self.source(PhotoId(id)))
                        .map(|s| s.class);
                    match kind {
                        CleanupKind::Duplicates => self.duplicate[i],
                        CleanupKind::Screenshots => {
                            matches!(class, Some(C::Screenshot | C::ScreenshotCropped))
                        }
                        CleanupKind::Saved => matches!(class, Some(C::App | C::Unknown)),
                        CleanupKind::Memes => meme.is_some_and(|b| self.subject[i] & (1 << b) != 0),
                    }
                };
                let picks = match kind {
                    CleanupKind::Duplicates => vec![SortPick::Duplicates],
                    CleanupKind::Screenshots => {
                        vec![source(C::Screenshot), source(C::ScreenshotCropped)]
                    }
                    CleanupKind::Saved => vec![source(C::App), source(C::Unknown)],
                    CleanupKind::Memes => vec![SortPick::Subject { key: "meme".into() }],
                };
                let members: Vec<usize> = (0..self.len()).filter(|&i| inn[i] && has(i)).collect();
                #[allow(clippy::cast_precision_loss)]
                let bytes = members.iter().map(|&i| self.size[i] as f64).sum();
                (!members.is_empty()).then(|| CleanupRow {
                    kind,
                    count: u32::try_from(members.len()).unwrap_or(u32::MAX),
                    bytes,
                    picks,
                })
            })
            .collect()
    }

    /// Years › months › events, newest first; an event under the month of its first photo.
    fn years(
        &self,
        idx: &SortIndex,
        events: &[Vec<usize>],
        places: &[Option<u32>],
        inn: &[bool],
    ) -> Vec<YearFacet> {
        let mut months: BTreeMap<(i32, u8), (usize, Vec<EventFacet>)> = BTreeMap::new();
        for i in (0..self.len()).filter(|&i| inn[i]) {
            months.entry((idx.year[i], idx.month[i])).or_default().0 += 1;
        }
        for whole in events {
            // The pick names the event by its first photo; the counts are the scope's.
            let Some(&id) = whole.first() else {
                continue;
            };
            let e: Vec<usize> = whole.iter().copied().filter(|&m| inn[m]).collect();
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
                    first: count(id),
                    first_day: day32(idx.t[first].div_euclid(86_400)),
                    last_day: day32(idx.t[last].div_euclid(86_400)),
                    count: count(e.len()),
                    place: place_name(place),
                    start_min: minute_of_day(idx.t[first]),
                    end_min: minute_of_day(idx.t[last]),
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

/// Minutes after midnight of local seconds.
fn minute_of_day(t: i64) -> u16 {
    u16::try_from(t.rem_euclid(86_400) / 60).unwrap_or(0)
}

/// The days with photos in `inn`, oldest first, with their counts.
fn days(idx: &SortIndex, inn: &[bool]) -> Vec<DayFacet> {
    let mut by: BTreeMap<i32, usize> = BTreeMap::new();
    for (&d, _) in idx.day.iter().zip(inn).filter(|(_, inn)| **inn) {
        *by.entry(d).or_default() += 1;
    }
    by.into_iter()
        .map(|(day, c)| DayFacet {
            day,
            count: count(c),
        })
        .collect()
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

/// The pixels' facts of every photo before a pass (`Library::sense_snapshot`).
#[derive(Clone, Debug)]
pub struct SenseSnapshot {
    subject: Vec<u16>,
    hash: Vec<Option<u64>>,
    analysed: Vec<bool>,
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
    fn fr_141_a_stopped_pass_puts_the_library_back() {
        let mut lib = Library::new();
        let f = lib.add_folder(Path::new("/roll"));
        let file = ImportedFile {
            path: PathBuf::from("/roll/0.jpg"),
            size: 1,
            mtime: 0,
        };
        lib.push(f, &file, &shot(3, 9));
        let before = lib.sense_snapshot();
        let cat = crate::sorting::clip::labels().labels["cat"][0].clone();
        let pixels = Pixels {
            hash: 7,
            embedding: cat,
        };
        assert!(lib.set_pixels(PhotoId(0), Path::new("/roll/0.jpg"), &pixels));
        assert_eq!(lib.facets(&[]).analysed, 1);
        assert!(lib.restore_sense(before));
        assert_eq!(lib.facets(&[]).analysed, 0);
    }

    fn roll(shots: &[PhotoMeta]) -> Library {
        let mut lib = Library::new();
        lib.set_home_zone(Some(crate::time_base::Zone::UTC));
        let f = lib.add_folder(Path::new("/roll"));
        for (k, meta) in shots.iter().enumerate() {
            let file = ImportedFile {
                path: PathBuf::from(format!("/roll/{k}.jpg")),
                size: 1,
                mtime: 0,
            };
            lib.push(f, &file, meta);
        }
        lib
    }

    fn july(day: u32) -> i32 {
        crate::calendar::days_from_ymd(2022, 7, day).unwrap_or(0)
    }

    /// C1: the calendar's days count the photos of each local day, and a ticked day sits in
    /// the Time card with years, months and events, so its ticks add up.
    #[test]
    fn fr_141_a_ticked_day_keeps_its_photos_and_adds_up_with_events() {
        let lib = roll(&[shot(3, 9), shot(3, 10), shot(20, 9)]);
        let facets = lib.facets(&[]);
        assert_eq!(
            facets.days,
            vec![
                DayFacet {
                    day: july(3),
                    count: 2
                },
                DayFacet {
                    day: july(20),
                    count: 1
                },
            ]
        );
        let mut filter = super::super::CullFilter {
            sort: vec![SortPick::Day { day: july(20) }],
            ..super::super::CullFilter::default()
        };
        assert_eq!(lib.filtered_ids(&filter), vec![PhotoId(2)]);
        assert_eq!(
            SortPick::Day { day: 0 }.card(),
            SortPick::Event { first: 0 }.card()
        );
        // The 3 July event and the 20th: one card, so both.
        let first = facets.years[0].months[0]
            .events
            .iter()
            .find(|e| e.first_day == july(3))
            .map_or(u32::MAX, |e| e.first);
        filter.sort.push(SortPick::Event { first });
        assert_eq!(lib.filtered_ids(&filter).len(), 3);
        // Another card narrows: no photo of the 20th from another source class.
        filter.sort = vec![
            SortPick::Day { day: july(20) },
            SortPick::Source {
                class: SourceClass::Screenshot,
            },
        ];
        assert_eq!(lib.filtered_ids(&filter), Vec::<PhotoId>::new());
        // A day without photos keeps none, and the scope's days are the scope's.
        filter.sort = vec![SortPick::Day { day: july(4) }];
        assert_eq!(lib.filtered_ids(&filter), Vec::<PhotoId>::new());
    }

    /// C1, FR-145: an event's times are the place's: a camera on Budapest time in Seoul shot at
    /// 10:00 and 11:30, which were 17:00 and 18:30 in Seoul.
    #[test]
    fn fr_141_event_times_are_the_places_local_time() {
        let seoul = |hour, min| PhotoMeta {
            capture: chrono::NaiveDate::from_ymd_opt(2019, 7, 10)
                .and_then(|d| d.and_hms_opt(hour, min, 0)),
            gps: Some((37.5665, 126.978)),
            make: Some("Panasonic".into()),
            orientation: 1,
            ..PhotoMeta::default()
        };
        let mut lib = roll(&[seoul(10, 0), seoul(11, 30)]);
        lib.set_home_zone(crate::time_base::zone("Europe/Budapest"));
        let facets = lib.facets(&[]);
        let events: Vec<(u16, u16, u32)> = facets
            .years
            .iter()
            .flat_map(|y| &y.months)
            .flat_map(|m| &m.events)
            .map(|e| (e.start_min, e.end_min, e.count))
            .collect();
        assert_eq!(events, vec![(17 * 60, 18 * 60 + 30, 2)]);
        assert_eq!(
            facets.days.iter().map(|d| d.count).collect::<Vec<_>>(),
            vec![2]
        );
    }

    #[test]
    fn fr_141_cleanup_rows_count_their_photos_and_bytes() {
        let mut lib = Library::new();
        let f = lib.add_folder(Path::new("/roll"));
        for (k, size) in [100u64, 250].into_iter().enumerate() {
            let file = ImportedFile {
                path: PathBuf::from(format!("/roll/{k}.jpg")),
                size,
                mtime: 0,
            };
            lib.push(f, &file, &shot(3, 9));
        }
        // No row without photos: a camera's shots are no clean-up.
        assert_eq!(lib.facets(&[]).cleanup, Vec::new());
        lib.set_duplicates(&[PhotoId(1)]);
        assert_eq!(
            lib.facets(&[]).cleanup,
            vec![CleanupRow {
                kind: CleanupKind::Duplicates,
                count: 1,
                bytes: 250.0,
                picks: vec![SortPick::Duplicates],
            }]
        );
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
            sort: vec![SortPick::Source {
                class: SourceClass::OtherCamera,
            }],
            ..super::super::CullFilter::default()
        };
        assert_eq!(lib.filtered_ids(&filter).len(), 3);
        filter.sort = vec![SortPick::Month {
            year: 2022,
            month: 7,
        }];
        assert_eq!(lib.filtered_ids(&filter).len(), 3);
        let facets = lib.facets(&[]);
        assert_eq!(facets.total, 3);
        assert_eq!(facets.years[0].months[0].events.len(), 2);
        let first = facets.years[0].months[0].events[0].first;
        filter.sort = vec![SortPick::Event { first }];
        assert_eq!(lib.filtered_ids(&filter).len(), 2);
        // A subject set by the pass, kept only for the photo still at that path.
        let cat = crate::sorting::clip::labels().labels["cat"][0].clone();
        let pixels = Pixels {
            hash: 1,
            embedding: cat,
        };
        assert!(!lib.set_pixels(PhotoId(0), Path::new("/elsewhere.jpg"), &pixels));
        assert!(lib.set_pixels(PhotoId(0), Path::new("/roll/0.jpg"), &pixels));
        filter.sort = vec![SortPick::Subject { key: "cat".into() }];
        assert_eq!(lib.filtered_ids(&filter), vec![PhotoId(0)]);
        assert_eq!(lib.facets(&[]).subjects[0].value, "cat");
        // One card's picks widen, different cards' narrow.
        filter.sort = vec![SortPick::Subject { key: "cat".into() }, SortPick::NoSubject];
        assert_eq!(lib.filtered_ids(&filter), vec![PhotoId(0)]);
        filter.sort = vec![
            SortPick::Subject { key: "cat".into() },
            SortPick::Event { first },
        ];
        assert_eq!(lib.filtered_ids(&filter), vec![PhotoId(0)]);
        filter.sort = vec![
            SortPick::Subject { key: "cat".into() },
            SortPick::Month {
                year: 2021,
                month: 1,
            },
        ];
        assert_eq!(lib.filtered_ids(&filter), Vec::<PhotoId>::new());
        // Only the tab chosen: a second folder's photo counts in its own scope alone.
        let g = lib.add_folder(Path::new("/other"));
        let file = ImportedFile {
            path: PathBuf::from("/other/x.jpg"),
            size: 1,
            mtime: 0,
        };
        lib.push(g, &file, &shot(25, 9));
        assert_eq!(
            (
                lib.facets(&[]).total,
                lib.facets(&[f]).total,
                lib.facets(&[g]).total
            ),
            (4, 3, 1)
        );
        assert_eq!(lib.facets(&[g]).subjects.len(), 0);
        // A street with walkers is scenery and people; people alone below the subject's bar
        // still name people; nothing near the bar names nothing.
        let keys = crate::sorting::clip::subject_keys();
        let bits = |mask: u16| -> Vec<&str> {
            (0..keys.len())
                .filter(|b| mask & (1 << b) != 0)
                .map(|b| keys[b])
                .collect()
        };
        assert_eq!(
            bits(named(&[("scenery", 0.6), ("people", 0.35)])),
            vec!["people", "scenery"]
        );
        assert_eq!(
            bits(named(&[("people", 0.4), ("scenery", 0.35)])),
            vec!["people"]
        );
        assert_eq!(
            named(&[("scenery", 0.45), ("food", 0.3), ("people", 0.1)]),
            0
        );
        // Old filters without a pick still read.
        let old: super::super::CullFilter =
            serde_json::from_str(r#"{"flag":"all","minRating":0}"#).unwrap_or_default();
        assert_eq!(old.sort, Vec::new());
    }
}
