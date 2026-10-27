// SPDX-License-Identifier: GPL-3.0-only
//! A photo's effective place (ADR-0027): a place set by hand (FR-032), otherwise one borrowed
//! from another device's photo of the same scene (FR-031), otherwise its own GPS through the
//! bundled places (FR-005); a place the user renamed is written with the new name.

use std::collections::HashMap;
use std::hash::BuildHasher;

use serde::Serialize;
use specta::Type;

use rayon::prelude::*;

use crate::culling::similar::{self, Signature};
use crate::geocode::{self, Place, PlaceFormat};
use crate::library::{Library, PhotoId, PhotoRecord, PlaceSource};

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

/// How sure a borrowed place is (FR-031).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Confidence {
    /// Shown as a suggestion to accept or skip.
    Medium,
    /// Applied by itself, as one undoable step.
    High,
}

/// Photos further apart in time than this are never compared (FR-031's default window).
pub const BORROW_WINDOW_S: i64 = 10 * 60;
/// Donors compared per photo, nearest in time first: four keeps the worst case (every
/// donor of a similar colour but another scene) within NFR-020 (RESULT.md §17).
pub const DONORS_PER_PHOTO: usize = 4;
/// The zooms a phone's and a camera's lenses usually differ by (RESULT.md §14).
const ZOOMS: [f32; 5] = [1.0, 1.25, 1.5, 2.0, 2.5];
/// Same scene, provisional (RESULT.md §14): layout and colour scores at least these.
const MEDIUM: (f32, f32) = (0.6, 0.65);
/// Applied without asking: stricter than the measured threshold and within five minutes,
/// until real camera and phone pairs have been scored.
const HIGH: (f32, f32) = (0.75, 0.8);
const HIGH_GAP_S: i64 = 5 * 60;

/// A place one photo can borrow from another device's photo of the same scene.
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Borrow {
    pub photo: PhotoId,
    /// The photo with GPS it borrows from.
    pub donor: PhotoId,
    pub geonames_id: u32,
    /// The place's name in the bundled data.
    pub name: String,
    pub confidence: Confidence,
    /// Seconds between the two photos.
    #[specta(type = f64)]
    pub gap_s: i64,
}

/// How alike two photos' scenes are: the best layout score over the usual zooms, and the
/// colour score. Colour is checked first because it costs a few µs; `None` when it fails.
pub fn same_scene(a: &Signature, b: &Signature) -> Option<(f32, f32)> {
    let colour = similar::color(a, b);
    if colour < MEDIUM.1 {
        return None;
    }
    // Zooms in order of how common they are; a layout already sure enough ends the search.
    let mut layout = -1f32;
    for &zoom in &ZOOMS {
        layout = layout.max(similar::structure_zoomed(a, b, zoom));
        if layout >= HIGH.0 {
            break;
        }
    }
    Some((layout, colour))
}

fn confidence((layout, colour): (f32, f32), gap_s: i64) -> Option<Confidence> {
    if layout >= HIGH.0 && colour >= HIGH.1 && gap_s <= HIGH_GAP_S {
        Some(Confidence::High)
    } else if layout >= MEDIUM.0 && colour >= MEDIUM.1 {
        Some(Confidence::Medium)
    } else {
        None
    }
}

/// The places photos without one can borrow (FR-031): for each, among the other devices'
/// photos with GPS within `window_s`, the most alike that shows the same scene. `signature`
/// gives a photo's visual signature from the best-shot analysis, if it has one.
pub fn borrow<'a>(
    library: &Library,
    window_s: i64,
    signature: impl Fn(PhotoId) -> Option<&'a Signature> + Sync,
) -> Vec<Borrow> {
    let candidates = library.borrow_candidates(window_s, DONORS_PER_PHOTO);
    candidates
        .par_iter()
        .filter_map(|(photo, donors)| {
            let own = signature(*photo)?;
            // Donors come nearest in time first: the first sure match is the one to keep.
            let mut best: Option<(PhotoId, i64, Confidence, f32)> = None;
            for &(donor, gap) in donors {
                let Some(scene) = signature(donor).and_then(|d| same_scene(own, d)) else {
                    continue;
                };
                let Some(conf) = confidence(scene, gap) else {
                    continue;
                };
                if best.is_none_or(|(_, _, c, l)| (conf, scene.0) > (c, l)) {
                    best = Some((donor, gap, conf, scene.0));
                }
                if conf == Confidence::High {
                    break;
                }
            }
            let (donor, gap_s, conf, _) = best?;
            let (lat, lon) = library.record(donor)?.gps?;
            let place = geocode::reverse(f64::from(lat), f64::from(lon)).ok()??;
            Some(Borrow {
                photo: *photo,
                donor,
                geonames_id: place.geonames_id,
                name: place.name,
                confidence: conf,
                gap_s,
            })
        })
        .collect()
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

    /// A scene: a gradient with a few blocks, different for each `seed`.
    fn scene(seed: u32) -> Signature {
        let (w, h) = (192u32, 128u32);
        let mut data = Vec::with_capacity((w * h * 3) as usize);
        for y in 0..h {
            for x in 0..w {
                let block = ((x / 32 + y / 32 + seed) % 3) as u8;
                #[allow(clippy::cast_possible_truncation)]
                let v = ((x * 255 / w) as u8).wrapping_add(seed as u8 * 40);
                data.extend_from_slice(&[v, block * 90, 255 - v / (block + 1)]);
            }
        }
        let image = crate::image::RgbImage::from_raw(w, h, data)
            .unwrap_or_else(|| crate::image::RgbImage::new(1, 1));
        similar::signature(&image)
    }

    fn photo(
        lib: &mut Library,
        name: &str,
        model: &str,
        minute: u32,
        gps: Option<(f64, f64)>,
    ) -> PhotoId {
        let folder = lib.add_folder(std::path::Path::new("/trip"));
        let meta = crate::metadata::PhotoMeta {
            capture: chrono::NaiveDate::from_ymd_opt(2026, 7, 1)
                .and_then(|d| d.and_hms_opt(12, minute, 0)),
            gps,
            make: Some("Maker".into()),
            model: Some(model.into()),
            ..crate::metadata::PhotoMeta::default()
        };
        let file = crate::import::ImportedFile {
            path: format!("/trip/{name}").into(),
            size: 1,
            mtime: 1,
        };
        lib.push(folder, &file, &meta).unwrap_or(PhotoId(u32::MAX))
    }

    #[test]
    fn fr_031_a_camera_photo_borrows_the_phones_place_of_the_same_scene() {
        let mut lib = Library::new();
        let debrecen = Some((47.53, 21.63));
        let gx9 = photo(&mut lib, "P1.JPG", "DC-GX9", 10, None);
        let phone = photo(&mut lib, "IMG_1.HEIC", "iPhone", 12, debrecen);
        let other_scene = photo(&mut lib, "P2.JPG", "DC-GX9", 11, None);
        let too_late = photo(&mut lib, "P3.JPG", "DC-GX9", 40, None);
        let sigs: HashMap<PhotoId, Signature> = HashMap::from([
            (gx9, scene(1)),
            (phone, scene(1)),
            (other_scene, scene(2)),
            (too_late, scene(1)),
        ]);
        let found = borrow(&lib, BORROW_WINDOW_S, |id| sigs.get(&id));
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!((found[0].photo, found[0].donor), (gx9, phone));
        assert_eq!(found[0].confidence, Confidence::High);
        assert_eq!(found[0].gap_s, 120);
        assert_eq!(
            geocode::by_id(found[0].geonames_id)
                .ok()
                .flatten()
                .map(|p| p.name),
            Some("Debrecen".into())
        );
    }

    #[test]
    fn fr_031_never_replaces_a_place_set_by_hand_or_compares_one_camera_with_itself() {
        let mut lib = Library::new();
        let gx9 = photo(&mut lib, "P1.JPG", "DC-GX9", 10, None);
        let same_body = photo(&mut lib, "P0.JPG", "DC-GX9", 11, Some((47.53, 21.63)));
        let sigs = HashMap::from([(gx9, scene(1)), (same_body, scene(1))]);
        assert_eq!(
            borrow(&lib, BORROW_WINDOW_S, |id| sigs.get(&id)),
            Vec::new()
        );
        let phone = photo(&mut lib, "IMG_1.HEIC", "iPhone", 12, Some((47.53, 21.63)));
        let mut sigs = sigs;
        sigs.insert(phone, scene(1));
        lib.set_edits(
            gx9,
            PhotoEdits {
                place: Some(PhotoPlace {
                    geonames_id: id_of("Prague"),
                    source: PlaceSource::Manual,
                }),
                ..PhotoEdits::default()
            },
        );
        assert_eq!(
            borrow(&lib, BORROW_WINDOW_S, |id| sigs.get(&id)),
            Vec::new()
        );
    }
}
