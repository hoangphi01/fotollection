// SPDX-License-Identifier: GPL-3.0-only
//! RollSense's facets (FR-141 S7): what the view lists, each with its count, and the pick the
//! grid filters by (`CullFilter::sort`). Picks name what they choose by something that stays
//! put while the library is worked out again: a source class, a subject's key, a year and
//! month, an event's first photo, a trip's first day, a home and a year, a place's GeoNames
//! id.

use serde::{Deserialize, Serialize};
use specta::Type;

use super::SourceClass;

/// What the grid keeps in RollSense.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SortPick {
    Source {
        class: SourceClass,
    },
    Subject {
        key: String,
    },
    /// Photos whose subject no label reached the rules' probability for.
    NoSubject,
    Year {
        year: i32,
    },
    Month {
        year: i32,
        month: u8,
    },
    /// The event that starts with this photo.
    Event {
        first: u32,
    },
    /// One day (days since 1970, the place's local time; FR-145), ticked on the calendar.
    Day {
        day: i32,
    },
    /// The trip that starts on this day (days since 1970, camera-local).
    Trip {
        #[serde(rename = "firstDay")]
        first_day: i32,
    },
    /// The visits to one of the other homes in one year.
    Visit {
        home: u8,
        year: i32,
    },
    Place {
        id: u32,
    },
    /// Photos with a near-duplicate (FR-015).
    Duplicates,
    /// Photos of a stack (S3), every member shown.
    Stacks,
}

impl SortPick {
    /// Whether the grid shows each stack by its cover alone: every pick but the stacks'
    /// and the duplicates' own, which are about the members.
    pub const fn folds_stacks(&self) -> bool {
        !matches!(self, Self::Stacks | Self::Duplicates)
    }

    /// The card a pick belongs to: picks of one card widen the grid (people or scenery), picks
    /// of different cards narrow it (cats, shot by the camera, in 2022; owner, 2026-11-18).
    pub const fn card(&self) -> u8 {
        match self {
            Self::Source { .. } => 0,
            Self::Subject { .. } | Self::NoSubject => 1,
            Self::Year { .. } | Self::Month { .. } | Self::Event { .. } | Self::Day { .. } => 2,
            Self::Trip { .. } => 3,
            Self::Visit { .. } => 4,
            Self::Place { .. } => 5,
            Self::Duplicates | Self::Stacks => 6,
        }
    }
}

/// A count under a name.
#[derive(Clone, Debug, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Count<T> {
    pub value: T,
    pub count: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EventFacet {
    pub first: u32,
    pub first_day: i32,
    pub last_day: i32,
    pub count: u32,
    pub place: Option<String>,
    /// Minutes after local midnight of its first and last photo, in the place's time (FR-145).
    pub start_min: u16,
    pub end_min: u16,
}

/// A day with photos, for the calendar (C1): days since 1970, the place's local time.
#[derive(Clone, Debug, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DayFacet {
    pub day: i32,
    pub count: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MonthFacet {
    pub month: u8,
    pub count: u32,
    pub events: Vec<EventFacet>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct YearFacet {
    pub year: i32,
    pub count: u32,
    pub months: Vec<MonthFacet>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TripFacet {
    /// The name the user gave it (S7c), else none: the UI shows the suggested one.
    pub name: Option<String>,
    pub first_day: i32,
    pub last_day: i32,
    pub count: u32,
    /// The place met most often, for the suggested name (`2022-07 Đà Lạt`).
    pub place: Option<String>,
}

/// One day of a trip, for its page (S7c): how many photos, and the place met most often.
#[derive(Clone, Debug, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TripDay {
    pub day: i32,
    pub count: u32,
    pub place: Option<String>,
}

/// The visits to one other home in one year (FR-012): how many, how many photos.
#[derive(Clone, Debug, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VisitFacet {
    pub home: u8,
    pub year: i32,
    pub visits: u32,
    pub count: u32,
    pub place: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PlaceFacet {
    pub id: u32,
    pub name: String,
    pub count: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CountryFacet {
    pub country: String,
    pub count: u32,
    pub places: Vec<PlaceFacet>,
}

/// A ready-made clean-up row (FR-141, owner 2026-11-18, two UX reviews): what a roll's owner
/// most often clears out, by the task and not by the facet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum CleanupKind {
    /// The extra copies of a picture (its keeper left out).
    Duplicates,
    Screenshots,
    /// Saved from apps, the web or chats.
    Saved,
    /// Memes, from the pixels: never ticked for the user.
    Memes,
}

/// One clean-up row: its photos, their bytes on disk, and the picks that show them.
#[derive(Clone, Debug, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CleanupRow {
    pub kind: CleanupKind,
    pub count: u32,
    pub bytes: f64,
    pub picks: Vec<SortPick>,
}

/// Everything RollSense lists, each with its count.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Facets {
    pub total: u32,
    /// Photos the pixels' pass has read (subjects and duplicates come from it).
    pub analysed: u32,
    pub sources: Vec<Count<SourceClass>>,
    pub subjects: Vec<Count<String>>,
    pub no_subject: u32,
    pub years: Vec<YearFacet>,
    /// The days with photos, oldest first, for the calendar's shading.
    pub days: Vec<DayFacet>,
    pub trips: Vec<TripFacet>,
    pub visits: Vec<VisitFacet>,
    pub countries: Vec<CountryFacet>,
    pub duplicates: u32,
    pub stacks: u32,
    /// The clean-up rows with photos, in their order.
    pub cleanup: Vec<CleanupRow>,
}
