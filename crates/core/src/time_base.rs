// SPDX-License-Identifier: GPL-3.0-only
//! One time base (FR-145, ADR-0058): every photo gets one instant in UTC, with the anchor it
//! came from, and is shown in the local time of the place it was taken. Pure functions: the
//! home zone is passed in, so nothing here reads the process's zone or the computer's clock.

use jiff::Timestamp;
use jiff::tz::{Offset, TimeZone};

/// A GPS time further than this from the camera's clock is not the shot's: some receivers
/// write the time of their last fix, days old. Every real offset lies within UTC−12…+14, so a
/// true GPS time is within 14 h of the camera's local clock; an hour more allows a clock that
/// drifted or was set an hour off.
pub const GPS_TRUST_S: i64 = 15 * 3600;
/// IANA names are at most 32 characters (`America/Argentina/ComodRivadavia`).
pub const MAX_ZONE_NAME: usize = 64;
const MAX_OFFSET_MIN: i64 = 14 * 60;

/// Where a photo's instant came from, most trusted first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeAnchor {
    /// The GPS receiver's time in the EXIF (UTC from the satellites).
    Gps,
    /// The photo's own offset (`OffsetTimeOriginal`).
    PhotoOffset,
    /// The camera's clock read in the zone the user gave that camera (FR-030).
    CameraZone,
    /// A clock that carries no zone, read in the home zone.
    HomeZone,
    /// A date set by hand (FR-032): wall time in the place's zone, else the home zone.
    Manual,
    /// The file's modified time, already UTC and weak.
    FileTime,
}

/// A photo's instant and the offset it is shown in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhotoTime {
    /// Seconds since 1970, UTC.
    pub utc: i64,
    /// Minutes from UTC of the time shown: the place's zone at that instant when the photo
    /// has a place, else the offset its time came with.
    pub offset_min: i16,
    pub source: TimeAnchor,
}

impl PhotoTime {
    /// The wall-clock time where the photo was taken, as seconds of a UTC calendar.
    pub fn local_seconds(&self) -> i64 {
        self.utc + i64::from(self.offset_min) * 60
    }

    /// The local day, days since 1970: what RollSense, trips and the calendar count.
    pub fn local_day(&self) -> i64 {
        self.local_seconds().div_euclid(86_400)
    }
}

/// What the library knows of one photo's time.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TimeFacts {
    /// Wall-clock seconds as a UTC calendar: the camera's time with its correction (FR-030),
    /// a time from the file's name, or a date set by hand; for `from_file`, the file's UTC
    /// time.
    pub local: i64,
    /// The photo's own offset in minutes.
    pub photo_offset: Option<i16>,
    /// The GPS time, UTC.
    pub gps_utc: Option<i64>,
    /// The zone the user gave the camera, in minutes (FR-030).
    pub camera_tz: Option<i16>,
    /// `local` is the file's modified time.
    pub from_file: bool,
    /// `local` is a date set by hand.
    pub manual: bool,
}

/// An IANA time zone, its rules from the IANA database bundled in the binary (never the
/// system's copy, so every computer reads a photo the same way).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Zone(TimeZone);

impl Zone {
    pub const UTC: Zone = Zone(TimeZone::UTC);

    /// The IANA name, `Asia/Ho_Chi_Minh`.
    pub fn name(&self) -> &str {
        self.0.iana_name().unwrap_or("UTC")
    }
}

/// The offset of `zone` at the instant `utc`, in minutes.
pub fn offset_at(zone: &Zone, utc: i64) -> i16 {
    Timestamp::from_second(utc).map_or(0, |t| {
        i16::try_from(zone.0.to_offset(t).seconds() / 60).unwrap_or(0)
    })
}

/// The instant of the wall-clock time `local` in `zone`. A time that occurs twice (the hour
/// summer time ends) is the earlier instant; one that never occurs (the hour skipped when it
/// starts) is read with the offset before the change, landing after it, as a clock moved on
/// by hand would.
pub fn local_to_utc(local: i64, zone: &Zone) -> i64 {
    let Ok(wall) = Timestamp::from_second(local).map(|t| Offset::UTC.to_datetime(t)) else {
        return local;
    };
    // `compatible`: the earlier instant in a fold, the offset before the change in a gap.
    zone.0
        .to_ambiguous_timestamp(wall)
        .compatible()
        .map_or(local, Timestamp::as_second)
}

/// A photo's instant, from the most trusted anchor it has (ADR-0058): a date set by hand;
/// the file's time; the GPS time within [`GPS_TRUST_S`] of the clock; the photo's offset; the
/// camera's zone; else the home zone. `place` is the zone of the photo's place, if any.
pub fn resolve(f: &TimeFacts, place: Option<&Zone>, home: &Zone) -> PhotoTime {
    let (utc, came_with, source) = if f.manual {
        (
            local_to_utc(f.local, place.unwrap_or(home)),
            None,
            TimeAnchor::Manual,
        )
    } else if f.from_file {
        (f.local, None, TimeAnchor::FileTime)
    } else if let Some(gps) = f.gps_utc.filter(|g| (f.local - g).abs() <= GPS_TRUST_S) {
        // The offset the clock showed, to the nearest quarter hour (a drift of minutes).
        let shown = f.photo_offset.unwrap_or_else(|| {
            let min = (f.local - gps + 450).div_euclid(900) * 15;
            i16::try_from(min.clamp(-MAX_OFFSET_MIN, MAX_OFFSET_MIN)).unwrap_or(0)
        });
        (gps, Some(shown), TimeAnchor::Gps)
    } else if let Some(off) = f.photo_offset {
        (
            f.local - i64::from(off) * 60,
            Some(off),
            TimeAnchor::PhotoOffset,
        )
    } else if let Some(tz) = f.camera_tz {
        (
            f.local - i64::from(tz) * 60,
            Some(tz),
            TimeAnchor::CameraZone,
        )
    } else {
        (local_to_utc(f.local, home), None, TimeAnchor::HomeZone)
    };
    let offset_min = match (place, came_with) {
        (Some(zone), _) => offset_at(zone, utc),
        (None, Some(off)) => off,
        (None, None) => offset_at(home, utc),
    };
    PhotoTime {
        utc,
        offset_min,
        source,
    }
}

/// The zone of an IANA name (`Asia/Ho_Chi_Minh`), spelt exactly, `None` for anything else.
/// The bundled database parses each zone once and keeps it.
pub fn zone(name: &str) -> Option<Zone> {
    if name.is_empty() || name.len() > MAX_ZONE_NAME {
        return None;
    }
    let tz = jiff::tz::db().get(name).ok()?;
    (tz.iana_name() == Some(name) && !tz.is_unknown()).then_some(Zone(tz))
}

/// The zone the system reports, if it is an IANA name we know: only a suggestion for the
/// home zone (FR-145), written to the settings only by the welcome's *Start*. Read with plain system calls (the `/etc/localtime`
/// link, `GetDynamicTimeZoneInformation` on Windows) and cached by jiff behind a lock.
pub fn system_zone() -> Option<Zone> {
    zone(TimeZone::try_system().ok()?.iana_name()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> i64 {
        chrono::NaiveDate::from_ymd_opt(y, mo, d)
            .and_then(|d| d.and_hms_opt(h, mi, 0))
            .map_or(0, |t| t.and_utc().timestamp())
    }

    fn day(y: i32, mo: u32, d: u32) -> i64 {
        at(y, mo, d, 0, 0).div_euclid(86_400)
    }

    fn z(name: &str) -> Zone {
        zone(name).unwrap_or(Zone::UTC)
    }

    /// FR-145: the GPS time wins over a camera's clock, even one set to another zone; a GPS
    /// time days away from the clock is a stale fix and is ignored.
    #[test]
    fn fr_145_gps_time_wins_over_a_camera_clock() {
        let f = TimeFacts {
            local: at(2019, 7, 10, 10, 3),
            gps_utc: Some(at(2019, 7, 10, 1, 0)),
            camera_tz: Some(60),
            ..TimeFacts::default()
        };
        let t = resolve(&f, Some(&z("Asia/Seoul")), &z("Europe/Budapest"));
        assert_eq!((t.utc, t.source), (at(2019, 7, 10, 1, 0), TimeAnchor::Gps));
        assert_eq!(t.offset_min, 9 * 60);
        // No place: the offset the clock showed, to the quarter hour.
        assert_eq!(resolve(&f, None, &z("Europe/Budapest")).offset_min, 9 * 60);
        let stale = TimeFacts {
            gps_utc: Some(at(2019, 7, 7, 1, 0)),
            ..f
        };
        assert_eq!(
            resolve(&stale, None, &z("Europe/Budapest")).source,
            TimeAnchor::CameraZone
        );
    }

    /// FR-145: a camera left on Budapest time in Seoul lands on the Seoul day: 20:00 in
    /// Budapest is 03:00 the next morning in Seoul.
    #[test]
    fn fr_145_a_camera_on_home_time_lands_on_the_seoul_day() {
        let f = TimeFacts {
            local: at(2019, 7, 10, 20, 0),
            ..TimeFacts::default()
        };
        let t = resolve(&f, Some(&z("Asia/Seoul")), &z("Europe/Budapest"));
        assert_eq!(t.source, TimeAnchor::HomeZone);
        assert_eq!(t.utc, at(2019, 7, 10, 18, 0));
        assert_eq!(t.local_seconds(), at(2019, 7, 11, 3, 0));
        assert_eq!(t.local_day(), day(2019, 7, 11));
        // Without a place it stays on the clock's day.
        assert_eq!(
            resolve(&f, None, &z("Europe/Budapest")).local_day(),
            day(2019, 7, 10)
        );
    }

    /// FR-145: European summer time of 2019 from the compiled rules: Budapest is UTC+2 in
    /// July and UTC+1 in January.
    #[test]
    fn fr_145_european_summer_time_2019() {
        let july = TimeFacts {
            local: at(2019, 7, 10, 12, 0),
            ..TimeFacts::default()
        };
        let t = resolve(&july, None, &z("Europe/Budapest"));
        assert_eq!((t.utc, t.offset_min), (at(2019, 7, 10, 10, 0), 120));
        let january = TimeFacts {
            local: at(2019, 1, 10, 12, 0),
            ..TimeFacts::default()
        };
        let t = resolve(&january, None, &z("Europe/Budapest"));
        assert_eq!((t.utc, t.offset_min), (at(2019, 1, 10, 11, 0), 60));
        // The clocks went forward on 31 March 2019 at 02:00 and back on 27 October at 03:00.
        assert_eq!(
            local_to_utc(at(2019, 3, 31, 2, 30), &z("Europe/Budapest")),
            at(2019, 3, 31, 1, 30)
        );
        assert_eq!(
            local_to_utc(at(2019, 10, 27, 2, 30), &z("Europe/Budapest")),
            at(2019, 10, 27, 0, 30)
        );
    }

    /// FR-145: a photo at 00:30 in Seoul, whose UTC day is the day before, is on its Seoul day.
    #[test]
    fn fr_145_half_past_midnight_in_seoul_is_not_the_day_before() {
        let f = TimeFacts {
            local: at(2019, 7, 11, 0, 30),
            photo_offset: Some(9 * 60),
            ..TimeFacts::default()
        };
        let t = resolve(&f, Some(&z("Asia/Seoul")), &z("Europe/Budapest"));
        assert_eq!(t.utc.div_euclid(86_400), day(2019, 7, 10));
        assert_eq!(t.local_day(), day(2019, 7, 11));
        assert_eq!(
            resolve(&f, None, &z("Europe/Budapest")).local_day(),
            day(2019, 7, 11)
        );
    }

    /// FR-145: the result depends only on what is passed: a photo with its own offset or a
    /// GPS time resolves the same whatever the home zone, and nothing reads the process's
    /// zone (no `Local` anywhere in this module).
    #[test]
    fn fr_145_the_computers_zone_changes_nothing() {
        let f = TimeFacts {
            local: at(2019, 7, 11, 0, 30),
            photo_offset: Some(9 * 60),
            ..TimeFacts::default()
        };
        let homes = [
            z("Europe/Budapest"),
            z("America/New_York"),
            z("Asia/Ho_Chi_Minh"),
            Zone::UTC,
        ];
        let seoul = z("Asia/Seoul");
        let all: Vec<PhotoTime> = homes.iter().map(|h| resolve(&f, Some(&seoul), h)).collect();
        assert!(all.windows(2).all(|w| w[0] == w[1]), "{all:?}");
        let source = include_str!("time_base.rs");
        assert!(!source.contains(concat!("chrono::", "Local")));
    }

    /// FR-145: a file's time is already UTC; a date set by hand is wall time in the place.
    #[test]
    fn fr_145_file_times_and_hand_dates() {
        let file = TimeFacts {
            local: at(2019, 7, 10, 22, 0),
            from_file: true,
            ..TimeFacts::default()
        };
        let t = resolve(&file, Some(&z("Asia/Seoul")), &z("Europe/Budapest"));
        assert_eq!(
            (t.utc, t.source),
            (at(2019, 7, 10, 22, 0), TimeAnchor::FileTime)
        );
        assert_eq!(t.local_day(), day(2019, 7, 11));
        let hand = TimeFacts {
            local: at(2019, 7, 10, 12, 0),
            manual: true,
            gps_utc: Some(at(2019, 7, 10, 1, 0)),
            ..TimeFacts::default()
        };
        let t = resolve(&hand, Some(&z("Asia/Seoul")), &z("Europe/Budapest"));
        assert_eq!(
            (t.utc, t.source),
            (at(2019, 7, 10, 3, 0), TimeAnchor::Manual)
        );
        assert_eq!(t.local_seconds(), at(2019, 7, 10, 12, 0));
    }

    #[test]
    fn fr_145_zone_names_are_checked() {
        assert_eq!(
            zone("Asia/Ho_Chi_Minh").as_ref().map(Zone::name),
            Some("Asia/Ho_Chi_Minh")
        );
        assert_eq!(zone("UTC"), Some(Zone::UTC));
        for bad in [
            "",
            "Mars/Olympus",
            "asia/seoul ",
            "asia/seoul",
            "Etc/Unknown",
            &"A".repeat(200),
            "Europe/Budapest\0",
        ] {
            assert_eq!(zone(bad), None, "{bad:?}");
        }
    }
}
