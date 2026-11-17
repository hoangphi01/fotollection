// SPDX-License-Identifier: GPL-3.0-only
//! Trips (FR-012, FR-141 S4): runs of days away from home. Home is the grid cell (about
//! 10 km) with photos on the most days, so a busy trip never outweighs the days at home, and
//! any cell visited in at least `home_min_months` different months is a home too (a family's
//! town visited all year round is no trip; owner, 2026-11-17). A day is away when most of its
//! located photos lie further than `away_km` from every home; a day with no location between
//! two days away stays in the trip. Homes are worked out in memory and never stored, logged
//! or shown.

use std::collections::{BTreeMap, HashMap, HashSet};

use super::events::Moment;

/// The trips' thresholds, as data.
#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize)]
pub struct TripRules {
    pub home_cell_deg: f32,
    pub away_km: f64,
    /// Days without photos allowed inside one trip.
    pub max_gap_days: i64,
    pub min_photos: usize,
    /// A cell with photos in this many different months (30-day spans) is a home.
    pub home_min_months: usize,
}

/// One trip: its first and last day (days since 1970, camera-local) and its photos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trip {
    pub first_day: i64,
    pub last_day: i64,
    pub members: Vec<usize>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Day {
    Home,
    Away,
    Unknown,
}

fn cell(p: (f32, f32), size: f32) -> (i32, i32) {
    #[allow(clippy::cast_possible_truncation)]
    let c = |v: f32| (v / size).floor() as i32;
    (c(p.0), c(p.1))
}

/// The homes' centres, the main home first: the cell with photos on the most days, then
/// every cell with photos in at least `min_months` different months.
fn homes(moments: &[Moment], size: f32, min_months: usize) -> Vec<(f32, f32)> {
    let mut days: HashMap<(i32, i32), HashSet<i64>> = HashMap::new();
    for m in moments {
        if let Some(p) = m.gps {
            days.entry(cell(p, size))
                .or_default()
                .insert(m.t.div_euclid(86_400));
        }
    }
    let Some(first) = days
        .iter()
        .max_by_key(|(c, d)| (d.len(), std::cmp::Reverse(**c)))
        .map(|(c, _)| *c)
    else {
        return Vec::new();
    };
    #[allow(clippy::cast_precision_loss)]
    let centre = |(y, x): (i32, i32)| ((y as f32 + 0.5) * size, (x as f32 + 0.5) * size);
    let mut others: Vec<(i32, i32)> = days
        .iter()
        .filter(|(c, d)| {
            **c != first
                && d.iter()
                    .map(|day| day.div_euclid(30))
                    .collect::<HashSet<_>>()
                    .len()
                    >= min_months
        })
        .map(|(c, _)| *c)
        .collect();
    others.sort_unstable();
    std::iter::once(first).chain(others).map(centre).collect()
}

fn km(a: (f32, f32), b: (f32, f32)) -> f64 {
    crate::geocode::distance_km(
        f64::from(a.0),
        f64::from(a.1),
        f64::from(b.0),
        f64::from(b.1),
    )
}

/// The trips, in time order.
pub fn trips(moments: &[Moment], rules: &TripRules) -> Vec<Trip> {
    let homes = homes(moments, rules.home_cell_deg, rules.home_min_months);
    if homes.is_empty() {
        return Vec::new();
    }
    let mut by_day: BTreeMap<i64, (Vec<usize>, usize, usize)> = BTreeMap::new();
    for (i, m) in moments.iter().enumerate() {
        let e = by_day.entry(m.t.div_euclid(86_400)).or_default();
        e.0.push(i);
        if let Some(p) = m.gps {
            let far = homes.iter().all(|&h| km(h, p) > rules.away_km);
            if far {
                e.1 += 1;
            } else {
                e.2 += 1;
            }
        }
    }
    let mut out = Vec::new();
    // The trip being built, and the days without a location waiting for the next day away.
    let mut open: Option<Trip> = None;
    let mut pending: Vec<(i64, Vec<usize>)> = Vec::new();
    let close = |open: &mut Option<Trip>, out: &mut Vec<Trip>| {
        if let Some(t) = open.take().filter(|t| t.members.len() >= rules.min_photos) {
            out.push(t);
        }
    };
    for (day, (members, away, at_home)) in by_day {
        let kind = match (away, at_home) {
            (0, 0) => Day::Unknown,
            (a, h) if a > h => Day::Away,
            _ => Day::Home,
        };
        let last = open.as_ref().map_or(i64::MIN, |t| t.last_day);
        let near = pending.last().map_or(last, |p| p.0);
        if open.is_some() && day.saturating_sub(near) > rules.max_gap_days + 1 {
            close(&mut open, &mut out);
            pending.clear();
        }
        match kind {
            Day::Away => {
                let t = open.get_or_insert_with(|| Trip {
                    first_day: day,
                    last_day: day,
                    members: Vec::new(),
                });
                for (_, m) in pending.drain(..) {
                    t.members.extend(m);
                }
                t.members.extend(members);
                t.last_day = day;
            }
            Day::Unknown if open.is_some() => pending.push((day, members)),
            Day::Unknown => {}
            Day::Home => {
                close(&mut open, &mut out);
                pending.clear();
            }
        }
    }
    close(&mut open, &mut out);
    out
}

/// A day's photos, its located photos by other home, and those located elsewhere.
type DayCount = (Vec<usize>, HashMap<u8, usize>, usize);

/// A stay at one of the other homes (a family's town): its days and photos. `home` numbers
/// those homes; the main home is never one of them, nor ever returned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Visit {
    pub home: u8,
    pub first_day: i64,
    pub last_day: i64,
    pub members: Vec<usize>,
}

/// The visits to the other homes, in time order (owner, 2026-11-17: "how many times this
/// year, how many photos"). Homes closer than `away_km` to the main one are the main one;
/// homes closer than that to each other are one.
pub fn visits(moments: &[Moment], rules: &TripRules) -> Vec<Visit> {
    let homes = homes(moments, rules.home_cell_deg, rules.home_min_months);
    let Some((&main, rest)) = homes.split_first() else {
        return Vec::new();
    };
    let mut others: Vec<(f32, f32)> = Vec::new();
    let mut group: Vec<u8> = Vec::new();
    for &h in rest.iter().filter(|&&h| km(main, h) > rules.away_km) {
        let g = others
            .iter()
            .position(|&o| km(o, h) <= rules.away_km)
            .map_or_else(
                || u8::try_from(others.len()).unwrap_or(u8::MAX),
                |i| group[i],
            );
        others.push(h);
        group.push(g);
    }
    let mut by_day: BTreeMap<i64, DayCount> = BTreeMap::new();
    for (i, m) in moments.iter().enumerate() {
        let e = by_day.entry(m.t.div_euclid(86_400)).or_default();
        e.0.push(i);
        if let Some(p) = m.gps {
            match others.iter().position(|&o| km(o, p) <= rules.away_km) {
                Some(o) => *e.1.entry(group[o]).or_default() += 1,
                None => e.2 += 1,
            }
        }
    }
    let mut out: Vec<Visit> = Vec::new();
    for (day, (members, at, elsewhere)) in by_day {
        let Some((&home, &n)) = at.iter().max_by_key(|&(h, n)| (n, std::cmp::Reverse(h))) else {
            continue;
        };
        if n <= elsewhere {
            continue;
        }
        match out.last_mut() {
            Some(v) if v.home == home && day - v.last_day <= rules.max_gap_days + 1 => {
                v.last_day = day;
                v.members.extend(members);
            }
            _ => out.push(Visit {
                home,
                first_day: day,
                last_day: day,
                members,
            }),
        }
    }
    out
}

/// Per year (by a visit's first day) and home: the number of visits and of photos.
pub fn visits_by_year(visits: &[Visit]) -> BTreeMap<(i32, u8), (usize, usize)> {
    let mut out: BTreeMap<(i32, u8), (usize, usize)> = BTreeMap::new();
    for v in visits {
        let year = chrono::DateTime::from_timestamp(v.first_day * 86_400, 0)
            .map_or(1970, |t| chrono::Datelike::year(&t));
        let e = out.entry((year, v.home)).or_default();
        e.0 += 1;
        e.1 += v.members.len();
    }
    out
}

/// The place met most often among a trip's photos, for its name (`2022-07 Đà Lạt`).
pub fn main_place(trip: &Trip, places: &[Option<u32>]) -> Option<u32> {
    let mut counts: HashMap<u32, usize> = HashMap::new();
    for &i in &trip.members {
        if let Some(Some(p)) = places.get(i) {
            *counts.entry(*p).or_default() += 1;
        }
    }
    counts
        .into_iter()
        .max_by_key(|&(p, n)| (n, std::cmp::Reverse(p)))
        .map(|(p, _)| p)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RULES: TripRules = TripRules {
        home_cell_deg: 0.1,
        away_km: 100.0,
        max_gap_days: 3,
        min_photos: 10,
        home_min_months: 6,
    };
    const DAY: i64 = 86_400;

    fn days(from: i64, n: i64, gps: Option<(f32, f32)>, per_day: i64) -> Vec<Moment> {
        (from..from + n)
            .flat_map(|d| {
                (0..per_day).map(move |k| Moment {
                    t: d * DAY + 36_000 + k * 600,
                    gps,
                })
            })
            .collect()
    }

    #[test]
    fn fr_012_days_away_from_home_make_a_trip() {
        let home = Some((47.49, 19.04));
        let away = Some((44.43, 26.10));
        let mut m = days(0, 60, home, 3);
        m.extend(days(60, 2, away, 8));
        m.extend(days(62, 1, None, 4)); // a day with no location, between days away
        m.extend(days(63, 2, away, 8));
        m.extend(days(65, 20, home, 3));
        let t = trips(&m, &RULES);
        assert_eq!(t.len(), 1);
        assert_eq!((t[0].first_day, t[0].last_day), (60, 64));
        assert_eq!(t[0].members.len(), 36);
        let mut places = vec![None; m.len()];
        for &i in &t[0].members {
            places[i] = Some(683_506);
        }
        assert_eq!(main_place(&t[0], &places), Some(683_506));
    }

    #[test]
    fn fr_012_a_town_visited_all_year_is_a_second_home() {
        let home = Some((47.49, 19.04));
        let family = Some((46.25, 20.15)); // 170 km away, a weekend every month
        let mut m = days(0, 300, home, 2);
        for month in 0..8 {
            m.extend(days(month * 35 + 5, 2, family, 8));
        }
        m.extend(days(310, 2, Some((44.43, 26.10)), 8));
        let t = trips(&m, &RULES);
        assert_eq!(t.iter().map(|t| t.first_day).collect::<Vec<_>>(), vec![310]);
    }

    #[test]
    fn fr_012_visits_to_a_second_home_count_by_year() {
        let home = Some((47.49, 19.04));
        let family = Some((46.25, 20.15));
        let mut m = days(0, 400, home, 2);
        for month in 0..8 {
            m.extend(days(month * 35 + 5, 2, family, 8));
        }
        let v = visits(&m, &RULES);
        assert_eq!(v.len(), 8);
        assert!(
            v.iter()
                .all(|v| v.home == 0 && v.last_day == v.first_day + 1)
        );
        // Only the days at the second home: 8 visits of 2 days, 10 photos a day.
        assert_eq!(v.iter().map(|v| v.members.len()).sum::<usize>(), 160);
        let by_year = visits_by_year(&v);
        assert_eq!(by_year.get(&(1970, 0)), Some(&(8, 8 * 2 * (8 + 2))));
    }

    #[test]
    fn fr_012_a_short_outing_or_a_long_absence_is_not_one_trip() {
        let home = Some((47.49, 19.04));
        let away = Some((44.43, 26.10));
        let mut m = days(0, 60, home, 3);
        m.extend(days(60, 1, away, 4)); // fewer than ten photos
        m.extend(days(70, 1, away, 12));
        m.extend(days(80, 1, away, 12)); // ten days later: another trip
        let t = trips(&m, &RULES);
        assert_eq!(
            t.iter().map(|t| t.first_day).collect::<Vec<_>>(),
            vec![70, 80]
        );
        assert_eq!(trips(&days(0, 3, None, 3), &RULES), Vec::new());
    }
}
