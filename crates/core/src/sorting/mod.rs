// SPDX-License-Identifier: GPL-3.0-only
//! Where each photo of a roll came from (FR-141, ADR-0054): the phone's own camera, a
//! panorama, a screenshot, an app, an editor, a film scanner, another camera or another phone,
//! decided from the metadata alone, the rule that decided it kept with the answer. The rules
//! are data (`assets/sorting/rules.json`), measured on the owner's labelled roll; what no rule
//! decides is `Unknown`, for the pixels to settle later (S6).

pub mod clip;
pub mod dupes;
pub mod events;
pub mod locations;
pub mod pass;
pub mod stacks;
pub mod time;
pub mod trips;

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

use crate::metadata::{CommentKind, PhotoMeta};

/// Where a photo came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceClass {
    /// The phone's own camera (Apple's `MakerNote`).
    Camera,
    /// The phone's panorama: Apple's make, no `MakerNote`, a long side past the sensor.
    Panorama,
    /// A screenshot, marked by the system or at a phone screen's size.
    Screenshot,
    /// Probably a screenshot cropped or marked up, its metadata lost: a phone screen's width.
    /// A guess (13 of 23 labelled were screenshots, the rest memes and saved images), for the
    /// pixels to confirm (S6).
    ScreenshotCropped,
    /// Saved or shot through an app ([`Source::app`] names it).
    App,
    /// Saved by an editor ([`Source::app`] names it).
    Edited,
    /// A film scan from a lab's scanner (FR-140).
    FilmScan,
    /// A camera other than a phone.
    OtherCamera,
    /// An iPhone that is not one of this library's own.
    AnotherPhone,
    /// No rule decided: saved from the web or a chat, a meme, a received photo.
    Unknown,
}

impl SourceClass {
    /// Whether the metadata decided it for certain (99.6 % of 237 labelled files, 2026-11-17);
    /// otherwise it is a guess or undecided, for the pixels (S6).
    pub const fn is_certain(self) -> bool {
        !matches!(self, Self::ScreenshotCropped | Self::Unknown)
    }

    /// Written by an app or an editor: a copy, never a capture's original (S3's stacks).
    pub const fn is_derived(self) -> bool {
        matches!(self, Self::App | Self::Edited)
    }
}

/// The answer, with the rule that gave it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub class: SourceClass,
    /// The app or editor, as the rules name it.
    pub app: Option<&'static str>,
    pub rule: &'static str,
}

#[derive(Deserialize)]
struct App {
    name: String,
    #[serde(default)]
    software: Vec<String>,
    #[serde(default)]
    make: Vec<String>,
}

#[derive(Deserialize)]
struct Panorama {
    min_long_side: u32,
    min_ratio: f32,
}

#[derive(Deserialize)]
struct FileTime {
    window_h: i64,
    share: f32,
}

#[derive(Deserialize)]
struct Rules {
    apps: Vec<App>,
    editors: Vec<String>,
    scanners: Vec<String>,
    screen_sizes: Vec<(u32, u32)>,
    screen_widths: Vec<u32>,
    panorama: Panorama,
    my_phone_share: f32,
    file_time: FileTime,
    events: events::EventRules,
    trips: trips::TripRules,
    duplicates: dupes::DupRules,
}

/// The near-duplicates' thresholds ([`dupes`]), as measured on the owner's labels.
pub fn dup_rules() -> &'static dupes::DupRules {
    &rules().duplicates
}

fn rules() -> &'static Rules {
    static RULES: OnceLock<Rules> = OnceLock::new();
    RULES.get_or_init(|| {
        serde_json::from_str(include_str!("../../assets/sorting/rules.json")).unwrap_or_else(|e| {
            unreachable!("assets/sorting/rules.json is checked by its test: {e}")
        })
    })
}

/// Whether the library's file times can be trusted, by the rules' window and share
/// ([`time::file_time_confirmed`]).
pub fn file_time_confirmed(pairs: impl Iterator<Item = (i64, i64)>) -> bool {
    let r = &rules().file_time;
    time::file_time_confirmed(pairs, r.window_h * 3600, r.share)
}

/// The roll's events, by the rules ([`events::events`]).
pub fn events_of(moments: &[events::Moment]) -> Vec<Vec<usize>> {
    events::events(moments, &rules().events)
}

/// The roll's trips, by the rules ([`trips::trips`]).
pub fn trips_of(moments: &[events::Moment]) -> Vec<trips::Trip> {
    trips::trips(moments, &rules().trips)
}

/// A roll's events, trips and places (S4), indexes into the photos given to [`roll`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Roll {
    pub events: Vec<Vec<usize>>,
    pub trips: Vec<trips::Trip>,
    /// Stays at the other homes (a family's town), never the main one.
    pub visits: Vec<trips::Visit>,
    /// Each photo's place, its own or borrowed from its event.
    pub places: Vec<Option<locations::Located>>,
    pub file_time_confirmed: bool,
}

/// Events and trips over the photos whose time is `firm` (an unconfirmed file time cuts no
/// event), then places lent within events.
pub fn roll(
    moments: &[events::Moment],
    firm: &[bool],
    mut places: Vec<Option<locations::Located>>,
    file_time_confirmed: bool,
) -> Roll {
    let kept: Vec<usize> = (0..moments.len())
        .filter(|&i| firm.get(i).copied().unwrap_or(false))
        .collect();
    let sub: Vec<events::Moment> = kept.iter().map(|&i| moments[i]).collect();
    let back = |v: Vec<usize>| v.into_iter().map(|j| kept[j]).collect::<Vec<_>>();
    let events: Vec<Vec<usize>> = events_of(&sub).into_iter().map(back).collect();
    let trips = trips_of(&sub)
        .into_iter()
        .map(|t| trips::Trip {
            members: back(t.members),
            ..t
        })
        .collect();
    let visits = trips::visits(&sub, &rules().trips)
        .into_iter()
        .map(|v| trips::Visit {
            members: back(v.members),
            ..v
        })
        .collect();
    locations::borrow_from_events(&mut places, &events);
    Roll {
        events,
        trips,
        visits,
        places,
        file_time_confirmed,
    }
}

/// The first of `needles` found in `haystack`, ignoring case; the rules are `'static`, so
/// is the name returned.
fn contains(haystack: &str, needles: &'static [String]) -> Option<&'static str> {
    let lower = haystack.to_lowercase();
    needles
        .iter()
        .find(|n| lower.contains(&n.to_lowercase()))
        .map(String::as_str)
}

/// "15.4.1": the system wrote the file, not an app.
fn is_system_version(software: &str) -> bool {
    !software.is_empty()
        && software
            .split('.')
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

const fn source(class: SourceClass, rule: &'static str) -> Source {
    Source {
        class,
        app: None,
        rule,
    }
}

/// Classifies one photo from its metadata and its extension (lower case, no dot). The order
/// is the one measured: the writer (`Software`) before `UserComment`, which an app may fill.
pub fn classify(meta: &PhotoMeta, ext: &str) -> Source {
    let r = rules();
    let software = meta.software.as_deref().unwrap_or("");
    let make = meta.make.as_deref().unwrap_or("");
    if meta.comment == Some(CommentKind::Screenshot) {
        return source(SourceClass::Screenshot, "user-comment-screenshot");
    }
    for app in &r.apps {
        if contains(software, &app.software).is_some() || contains(make, &app.make).is_some() {
            return Source {
                class: SourceClass::App,
                app: Some(app.name.as_str()),
                rule: "software-app",
            };
        }
    }
    if meta.comment == Some(CommentKind::AppEffect) {
        return Source {
            class: SourceClass::App,
            app: Some("Instagram"),
            rule: "user-comment-effect",
        };
    }
    if contains(&format!("{make} {software}"), &r.scanners).is_some() {
        return source(SourceClass::FilmScan, "scanner");
    }
    if let Some(editor) = contains(software, &r.editors) {
        return Source {
            class: SourceClass::Edited,
            app: Some(editor),
            rule: "software-editor",
        };
    }
    if meta.apple.is_some() {
        return source(SourceClass::Camera, "apple-makernote");
    }
    let size = meta.pixel_size.map(|(w, h)| (w.min(h), w.max(h)));
    if make.eq_ignore_ascii_case("apple") {
        let panorama = size.is_some_and(|(short, long)| {
            long > r.panorama.min_long_side
                && short > 0
                && precision(long) / precision(short) >= r.panorama.min_ratio
        });
        if panorama && is_system_version(software) {
            return source(SourceClass::Panorama, "apple-wide-no-makernote");
        }
        return source(SourceClass::Unknown, "apple-no-makernote");
    }
    if !make.is_empty() {
        return source(SourceClass::OtherCamera, "camera-make");
    }
    if ext == "png" && size.is_some_and(|s| r.screen_sizes.contains(&s)) {
        return source(SourceClass::Screenshot, "png-screen-size");
    }
    if meta
        .pixel_size
        .is_some_and(|(w, _)| r.screen_widths.contains(&w))
    {
        return source(SourceClass::ScreenshotCropped, "screen-width-no-camera");
    }
    source(SourceClass::Unknown, "no-camera")
}

#[allow(clippy::cast_precision_loss)]
const fn precision(v: u32) -> f32 {
    v as f32
}

/// The phone models that are the library's own: those behind at least the rules' share of
/// its camera shots. A camera shot from any other model came from another phone.
pub fn my_phones<'a>(camera_models: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    let mut total = 0usize;
    for m in camera_models {
        *counts.entry(m).or_default() += 1;
        total += 1;
    }
    #[allow(clippy::cast_precision_loss)]
    let min = (total as f32 * rules().my_phone_share).ceil();
    #[allow(clippy::cast_precision_loss)]
    let mut mine: Vec<String> = counts
        .into_iter()
        .filter(|&(_, n)| n as f32 >= min)
        .map(|(m, _)| m.to_string())
        .collect();
    mine.sort();
    mine
}

/// A camera shot from a model that is not one of `mine` came from another phone.
pub fn refine(source: Source, model: Option<&str>, mine: &[String]) -> Source {
    if source.class == SourceClass::Camera
        && !mine.is_empty()
        && model.is_none_or(|m| !mine.iter().any(|x| x == m))
    {
        return Source {
            class: SourceClass::AnotherPhone,
            app: None,
            rule: "apple-makernote-other-model",
        };
    }
    source
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apple_note::AppleNote;

    fn meta(make: Option<&str>, software: Option<&str>, size: Option<(u32, u32)>) -> PhotoMeta {
        PhotoMeta {
            make: make.map(str::to_string),
            software: software.map(str::to_string),
            pixel_size: size,
            orientation: 1,
            ..PhotoMeta::default()
        }
    }

    #[test]
    fn fr_141_the_rules_file_parses() {
        let r = rules();
        assert!(r.editors.iter().any(|e| e == "Snapseed"));
        assert!(!r.screen_widths.is_empty() && r.my_phone_share > 0.0);
    }

    #[test]
    fn fr_141_each_source_rule_decides_its_class() {
        let camera = PhotoMeta {
            apple: Some(AppleNote::default()),
            ..meta(Some("Apple"), Some("15.4.1"), Some((4032, 3024)))
        };
        assert_eq!(classify(&camera, "heic").class, SourceClass::Camera);
        let screenshot = PhotoMeta {
            comment: Some(CommentKind::Screenshot),
            ..meta(None, None, Some((1284, 2778)))
        };
        assert_eq!(classify(&screenshot, "png").class, SourceClass::Screenshot);
        assert_eq!(
            classify(&meta(None, None, Some((1242, 2688))), "png").rule,
            "png-screen-size"
        );
        let cropped = classify(&meta(None, None, Some((1242, 667))), "jpeg").class;
        assert_eq!(cropped, SourceClass::ScreenshotCropped);
        assert!(!cropped.is_certain() && SourceClass::Screenshot.is_certain());
        assert_eq!(
            classify(&meta(None, None, Some((600, 914))), "jpg").class,
            SourceClass::Unknown
        );
        assert_eq!(
            classify(
                &meta(Some("Apple"), Some("13.4.1"), Some((13976, 3628))),
                "jpg"
            )
            .class,
            SourceClass::Panorama
        );
        let insta = classify(
            &meta(Some("Apple"), Some("Instagram"), Some((1284, 2282))),
            "jpg",
        );
        assert_eq!(
            (insta.class, insta.app),
            (SourceClass::App, Some("Instagram"))
        );
        let snap = classify(
            &meta(Some("Canon"), Some("Snapseed 2.19.280302127"), None),
            "jpg",
        );
        assert_eq!(
            (snap.class, snap.app),
            (SourceClass::Edited, Some("Snapseed"))
        );
        let lr = classify(
            &meta(
                Some("SONY"),
                Some("Adobe Photoshop Lightroom Classic 9.2.1 (Windows)"),
                None,
            ),
            "jpg",
        );
        assert_eq!(lr.class, SourceClass::Edited);
        assert_eq!(
            classify(
                &meta(Some("NORITSU KOKI"), Some("EZ Controller 5.01.024"), None),
                "jpg"
            )
            .class,
            SourceClass::FilmScan
        );
        assert_eq!(
            classify(
                &meta(
                    Some("FUJI PHOTO FILM CO., LTD."),
                    Some("FRONTIER355/375-3.0"),
                    None
                ),
                "jpg"
            )
            .class,
            SourceClass::FilmScan
        );
        assert_eq!(
            classify(&meta(Some("Canon"), None, None), "jpg").class,
            SourceClass::OtherCamera
        );
    }

    #[test]
    fn fr_141_the_writer_wins_over_the_comment() {
        // Instagram's camera writes an effect into UserComment; Software still names Instagram.
        let m = PhotoMeta {
            comment: Some(CommentKind::AppEffect),
            apple: Some(AppleNote::default()),
            ..meta(Some("Apple"), Some("Instagram"), None)
        };
        assert_eq!(classify(&m, "jpg").app, Some("Instagram"));
    }

    #[test]
    fn fr_141_a_camera_shot_from_a_rare_model_came_from_another_phone() {
        let models: Vec<&str> = std::iter::repeat_n("iPhone 12 Pro Max", 90)
            .chain(["iPhone 13"])
            .chain(std::iter::repeat_n("iPhone X", 9))
            .collect();
        let mine = my_phones(models.into_iter());
        assert_eq!(
            mine,
            vec!["iPhone 12 Pro Max".to_string(), "iPhone X".to_string()]
        );
        let shot = source(SourceClass::Camera, "apple-makernote");
        assert_eq!(
            refine(shot.clone(), Some("iPhone 13"), &mine).class,
            SourceClass::AnotherPhone
        );
        assert_eq!(
            refine(shot, Some("iPhone X"), &mine).class,
            SourceClass::Camera
        );
    }
}
