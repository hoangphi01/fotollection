// SPDX-License-Identifier: GPL-3.0-only
//! A photo's effective place (ADR-0027): a place set by hand (FR-032), otherwise one borrowed
//! from another device's photo of the same scene (FR-031), otherwise its own GPS through the
//! bundled places (FR-005); a place the user renamed is written with the new name.

use std::collections::HashMap;
use std::hash::BuildHasher;

use serde::Serialize;
use specta::Type;

use crate::geocode::{self, Place, PlaceFormat};
use crate::library::{PhotoRecord, PlaceSource};

/// The place a photo is stamped with, and where it comes from.
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EffectivePlace {
    pub geonames_id: u32,
    /// The text printed, in the chosen format and with the user's name for the place.
    pub label: String,
    /// The place's own name in the bundled data, before any rename.
    pub original: String,
    pub source: PlaceSource,
}

/// The place of `record`, or `None` when it has no place set, borrowed or from GPS.
/// `names` holds the places the user renamed (FR-005).
pub fn effective<S: BuildHasher>(
    record: &PhotoRecord,
    names: &HashMap<u32, String, S>,
    format: PlaceFormat,
) -> Option<EffectivePlace> {
    let (place, source) = if let Some(p) = record.edits.place {
        (geocode::by_id(p.geonames_id).ok()??, p.source)
    } else {
        let (lat, lon) = record.gps?;
        let place = geocode::reverse(f64::from(lat), f64::from(lon)).ok()??;
        (place, PlaceSource::Gps)
    };
    Some(described(&place, names, format, source))
}

/// `place` written in `format`, with the user's name for it when renamed.
pub fn described<S: BuildHasher>(
    place: &Place,
    names: &HashMap<u32, String, S>,
    format: PlaceFormat,
    source: PlaceSource,
) -> EffectivePlace {
    let name = names.get(&place.geonames_id).unwrap_or(&place.name);
    EffectivePlace {
        geonames_id: place.geonames_id,
        label: geocode::label(name, &place.country_code, &place.country, format),
        original: place.name.clone(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::{PhotoEdits, PhotoId, PhotoPlace, ShotInfo};

    fn record(gps: Option<(f32, f32)>, place: Option<PhotoPlace>) -> PhotoRecord {
        let time = chrono::NaiveDateTime::default();
        PhotoRecord {
            id: PhotoId(0),
            path: "a.jpg".into(),
            file_name: "a.jpg".into(),
            size: 1,
            mtime: 1,
            capture: time,
            orientation: 1,
            flags: 0,
            gps,
            shot: ShotInfo::default(),
            edits: PhotoEdits {
                place,
                ..PhotoEdits::default()
            },
            camera_time: time,
        }
    }

    fn id_of(name: &str) -> u32 {
        geocode::search(name, 1)
            .ok()
            .and_then(|v| v.into_iter().next())
            .map_or(0, |p| p.geonames_id)
    }

    #[test]
    fn fr_032_a_manual_place_wins_over_gps() {
        // GPS in Debrecen, place set to Prague by hand.
        let prague = id_of("Prague");
        let manual = Some(PhotoPlace {
            geonames_id: prague,
            source: PlaceSource::Manual,
        });
        let names = HashMap::new();
        let place = effective(
            &record(Some((47.53, 21.63)), manual),
            &names,
            PlaceFormat::City,
        );
        assert_eq!(place.as_ref().map(|p| p.label.as_str()), Some("Prague"));
        assert_eq!(place.map(|p| p.source), Some(PlaceSource::Manual));
        let own = effective(
            &record(Some((47.53, 21.63)), None),
            &names,
            PlaceFormat::City,
        );
        assert_eq!(own.as_ref().map(|p| p.label.as_str()), Some("Debrecen"));
        assert_eq!(own.map(|p| p.source), Some(PlaceSource::Gps));
        assert_eq!(
            effective(&record(None, None), &names, PlaceFormat::City),
            None
        );
    }

    #[test]
    fn fr_005_a_renamed_place_keeps_its_country() {
        let debrecen = id_of("Debrecen");
        let names = HashMap::from([(debrecen, "Home".to_owned())]);
        let place = effective(
            &record(Some((47.53, 21.63)), None),
            &names,
            PlaceFormat::CityCountryCode,
        );
        assert_eq!(place.as_ref().map(|p| p.label.as_str()), Some("Home, HU"));
        assert_eq!(place.map(|p| p.original), Some("Debrecen".to_owned()));
    }

    #[test]
    fn fr_032_an_unknown_place_id_is_no_place() {
        let unknown = Some(PhotoPlace {
            geonames_id: u32::MAX,
            source: PlaceSource::Manual,
        });
        assert_eq!(
            effective(&record(None, unknown), &HashMap::new(), PlaceFormat::City),
            None
        );
    }
}
