// SPDX-License-Identifier: GPL-3.0-only
//! Each photo's instant and local time (FR-145, ADR-0058), from its columns, its place and the
//! home zone, worked out once when first asked and dropped on a change of a time, a place, a
//! camera's clock or the home zone. RollSense's days, events and trips read
//! [`Library::local_seconds`]; sorting and stamps keep `effective` (ADR-0027).
//! A child of `library` so it reads the columns without making them public.

use chrono_tz::Tz;

use super::{Library, NO_DATE, NO_GPS_TIME, NO_OFFSET, PhotoId, flags};
use crate::time_base::{self, PhotoTime, TimeFacts};

/// Each photo's instant and the GeoNames id of its place, by photo index. Owned by the
/// library (`times`): about 24 bytes a photo, emptied with `roll` and `sort_index`.
pub(super) struct TimeTable {
    times: Vec<PhotoTime>,
    pub(super) places: Vec<Option<u32>>,
}

impl Library {
    /// The zone clocks without one are read in (FR-145): `None` follows the system's.
    /// A change works every photo's time out again.
    pub fn set_home_zone(&mut self, home: Option<Tz>) {
        if self.home != home {
            self.home = home;
            self.times = std::sync::OnceLock::new();
            self.roll = std::sync::OnceLock::new();
            self.sort_index = std::sync::OnceLock::new();
        }
    }

    /// The photo's place: one set by hand or borrowed (FR-032, FR-031), else its own GPS.
    fn place_id(&self, i: usize) -> Option<u32> {
        let own = self
            .edits
            .get(&PhotoId(u32::try_from(i).ok()?))
            .and_then(|e| e.place.as_ref());
        own.map(|p| p.geonames_id).or_else(|| {
            if self.lat[i].is_nan() {
                return None;
            }
            crate::geocode::reverse(f64::from(self.lat[i]), f64::from(self.lon[i]))
                .ok()??
                .geonames_id
                .into()
        })
    }

    fn facts(&self, i: usize) -> TimeFacts {
        let f = self.flags[i];
        let clock = f & (flags::DATE_FROM_FILE | flags::DATE_FROM_NAME) == 0;
        TimeFacts {
            local: self.effective(i),
            photo_offset: (self.offset[i] != NO_OFFSET).then_some(self.offset[i]),
            gps_utc: (self.gps_utc[i] != NO_GPS_TIME).then_some(self.gps_utc[i]),
            camera_tz: clock
                .then(|| self.bodies.get(usize::from(self.body[i]))?.tz_minutes)
                .flatten(),
            from_file: f & flags::DATE_FROM_FILE != 0,
            manual: self.manual_date[i] != NO_DATE,
        }
    }

    pub(super) fn time_table(&self) -> &TimeTable {
        self.times.get_or_init(|| {
            let home = self.home.or_else(time_base::system_zone).unwrap_or(Tz::UTC);
            let places: Vec<Option<u32>> = (0..self.len()).map(|i| self.place_id(i)).collect();
            let times = places
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let zone = p.and_then(crate::zones::of_place);
                    time_base::resolve(&self.facts(i), zone, home)
                })
                .collect();
            TimeTable { times, places }
        })
    }

    /// The wall-clock seconds where photo `i` was taken (FR-145), 0 past the end.
    pub(super) fn local_seconds(&self, i: usize) -> i64 {
        self.time_table()
            .times
            .get(i)
            .map_or(0, PhotoTime::local_seconds)
    }

    /// The photo's instant, its offset and its anchor (FR-145).
    pub fn photo_time(&self, id: PhotoId) -> Option<PhotoTime> {
        let i = self.index(id)?;
        self.time_table().times.get(i).copied()
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::import::ImportedFile;
    use crate::metadata::PhotoMeta;
    use crate::time_base::TimeAnchor;

    fn file(path: &str) -> ImportedFile {
        ImportedFile {
            path: PathBuf::from(path),
            size: 10,
            mtime: 0,
        }
    }

    fn shot(d: u32, h: u32, gps: Option<(f64, f64)>) -> PhotoMeta {
        PhotoMeta {
            capture: chrono::NaiveDate::from_ymd_opt(2019, 7, d)
                .and_then(|d| d.and_hms_opt(h, 0, 0)),
            gps,
            make: Some("Panasonic".into()),
            model: Some("DC-GX9".into()),
            orientation: 1,
            ..PhotoMeta::default()
        }
    }

    fn day(d: u32) -> i64 {
        chrono::NaiveDate::from_ymd_opt(2019, 7, d)
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .map_or(0, |t| t.and_utc().timestamp().div_euclid(86_400))
    }

    const SEOUL: (f64, f64) = (37.5665, 126.978);

    /// FR-145: a camera left on Budapest time in Seoul: its evening shot is on the next Seoul
    /// day for the trips, the events and RollSense's months; a home zone change moves it.
    #[test]
    fn fr_145_a_camera_on_home_time_in_seoul_counts_seoul_days() {
        let mut lib = Library::new();
        lib.set_home_zone(Some(Tz::Europe__Budapest));
        let f = lib.add_folder(Path::new("/trip"));
        let ids: Vec<PhotoId> = [
            lib.push(f, &file("/trip/a.jpg"), &shot(10, 10, Some(SEOUL))),
            lib.push(f, &file("/trip/b.jpg"), &shot(10, 20, Some(SEOUL))),
        ]
        .into_iter()
        .flatten()
        .collect();
        assert_eq!(ids.len(), 2);
        let t = lib.photo_time(ids[1]);
        assert_eq!(
            t.map(|t| (t.source, t.offset_min)),
            Some((TimeAnchor::HomeZone, 540))
        );
        assert_eq!(t.map(|t| t.local_day()), Some(day(11)));
        assert_eq!(lib.local_seconds(1).div_euclid(86_400), day(11));
        assert_eq!(lib.sort_index().t[1].div_euclid(86_400), day(11));
        let trip = lib.roll().trips.first().map(|t| (t.first_day, t.last_day));
        if let Some((first, last)) = trip {
            assert_eq!((first, last), (day(10), day(11)));
        }
        // Read in Seoul's zone the clock is Seoul time: both shots on the 10th.
        lib.set_home_zone(Some(Tz::Asia__Seoul));
        assert_eq!(lib.photo_time(ids[1]).map(|t| t.local_day()), Some(day(10)));
        assert_eq!(lib.sort_index().t[1].div_euclid(86_400), day(10));
    }

    /// FR-145: the GPS time wins over a camera's clock in the library too, and a set clock
    /// zone (FR-030) is the anchor of a photo without GPS time or offset.
    #[test]
    fn fr_145_gps_time_and_the_cameras_zone_in_the_library() {
        let mut lib = Library::new();
        lib.set_home_zone(Some(Tz::Europe__Budapest));
        let f = lib.add_folder(Path::new("/trip"));
        let gps_utc = chrono::NaiveDate::from_ymd_opt(2019, 7, 10)
            .and_then(|d| d.and_hms_opt(1, 0, 0))
            .map(|t| t.and_utc().timestamp());
        let with_gps = PhotoMeta {
            gps_utc,
            ..shot(10, 3, Some(SEOUL))
        };
        let a = lib.push(f, &file("/trip/a.jpg"), &with_gps);
        let b = lib.push(f, &file("/trip/b.jpg"), &shot(10, 3, None));
        assert_eq!(
            lib.photo_time(a.unwrap_or(PhotoId(9)))
                .map(|t| (t.utc, t.source)),
            gps_utc.map(|g| (g, TimeAnchor::Gps))
        );
        let b = b.unwrap_or(PhotoId(9));
        assert_eq!(
            lib.photo_time(b).map(|t| t.source),
            Some(TimeAnchor::HomeZone)
        );
        let body = lib.clocks().first().map_or(u16::MAX, |c| c.id);
        assert!(lib.set_clock(body, 0, Some(9 * 60)).is_ok());
        assert_eq!(
            lib.photo_time(b).map(|t| t.source),
            Some(TimeAnchor::CameraZone)
        );
    }
}
