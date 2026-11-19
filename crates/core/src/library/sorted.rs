// SPDX-License-Identifier: GPL-3.0-only
//! The library's sorting (FR-141): each photo's source, the stacks, and the roll's events,
//! trips and places, worked out from its columns when first asked and dropped on a change.
//! A child of `library` so it reads the columns without making them public.

use std::collections::HashMap;

use super::{Library, NO_OFFSET, PhotoId, flags};

impl Library {
    /// Where a photo came from (FR-141): its class at import, a camera shot from a model this
    /// library rarely holds told apart as another phone's.
    pub fn source(&self, id: PhotoId) -> Option<crate::sorting::Source> {
        let i = self.index(id)?;
        let mine = self.my_phones.get_or_init(|| {
            let models = (0..self.len())
                .filter(|&j| self.sources[j].class == crate::sorting::SourceClass::Camera)
                .filter_map(|j| self.cameras.get(usize::from(self.camera[j]))?.1.as_deref());
            crate::sorting::my_phones(models)
        });
        let model = self
            .cameras
            .get(usize::from(self.camera[i]))
            .and_then(|c| c.1.as_deref());
        Some(crate::sorting::refine(self.sources[i].clone(), model, mine))
    }

    /// The library's stacks (FR-141 S3) and each photo's, worked out once after a change.
    fn stack_table(&self) -> &(Vec<crate::sorting::stacks::Stack>, HashMap<u32, usize>) {
        self.stacks.get_or_init(|| {
            let stacks = crate::sorting::stacks::stacks(&self.stack_keys);
            let of = stacks
                .iter()
                .enumerate()
                .flat_map(|(s, st)| st.members.iter().map(move |&m| (m, s)))
                .collect();
            (stacks, of)
        })
    }

    /// The roll's events, trips and places (FR-141 S4); indexes are photo ids.
    pub fn roll(&self) -> &crate::sorting::Roll {
        use crate::sorting::{events::Moment, locations::Located};
        self.roll.get_or_init(|| {
            let n = self.len();
            let exif =
                |i: usize| self.flags[i] & (flags::DATE_FROM_FILE | flags::DATE_FROM_NAME) == 0;
            let confirmed = crate::sorting::file_time_confirmed(
                (0..n)
                    .filter(|&i| exif(i))
                    .map(|i| (self.capture[i], self.mtime[i])),
            );
            let gps = |i: usize| (!self.lat[i].is_nan()).then_some((self.lat[i], self.lon[i]));
            // The place's local time (FR-145), so a day is the day where the photo was taken.
            let moments: Vec<Moment> = (0..n)
                .map(|i| Moment {
                    t: self.local_seconds(i),
                    gps: gps(i),
                })
                .collect();
            let firm: Vec<bool> = (0..n)
                .map(|i| confirmed || self.flags[i] & flags::DATE_FROM_FILE == 0)
                .collect();
            let places = self
                .time_table()
                .places
                .iter()
                .map(|p| {
                    p.map(|place_id| Located {
                        place_id,
                        borrowed: false,
                    })
                })
                .collect();
            let mut roll = crate::sorting::roll(&moments, &firm, places, confirmed);
            // The user's cuts and joins (S7c), on the trips found.
            roll.trips = crate::sorting::trips::apply_edits(
                std::mem::take(&mut roll.trips),
                &self.trip_edits,
                |i| moments.get(i).map_or(0, |m| m.t.div_euclid(86_400)),
            );
            roll
        })
    }

    /// Where a photo's time came from (FR-141 S4).
    pub fn time_source(&self, id: PhotoId) -> Option<crate::sorting::time::TimeSource> {
        use crate::sorting::time::TimeSource;
        let i = self.index(id)?;
        let f = self.flags[i];
        Some(if f & flags::DATE_FROM_NAME != 0 {
            TimeSource::FileName
        } else if f & flags::DATE_FROM_FILE != 0 {
            TimeSource::FileTime {
                confirmed: self.roll().file_time_confirmed,
            }
        } else if self.offset[i] == NO_OFFSET {
            TimeSource::ExifNoOffset
        } else {
            TimeSource::Exif
        })
    }

    /// Every stack of two photos or more; members are photo ids.
    pub fn stacks(&self) -> &[crate::sorting::stacks::Stack] {
        &self.stack_table().0
    }

    /// The stack a photo belongs to, if any.
    pub fn stack_of(&self, id: PhotoId) -> Option<&crate::sorting::stacks::Stack> {
        let (stacks, of) = self.stack_table();
        stacks.get(*of.get(&id.0)?)
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::import::ImportedFile;
    use crate::metadata::PhotoMeta;

    fn file(path: &str, mtime: i64) -> ImportedFile {
        ImportedFile {
            path: PathBuf::from(path),
            size: 10,
            mtime,
        }
    }

    fn meta_at(day: u32) -> PhotoMeta {
        PhotoMeta {
            capture: chrono::NaiveDate::from_ymd_opt(2026, 7, day)
                .and_then(|d| d.and_hms_opt(10, 0, 0)),
            orientation: 1,
            ..PhotoMeta::default()
        }
    }

    /// FR-141 S4: a file with no EXIF time takes the one in its name; each time names its
    /// source; two days apart are two events.
    #[test]
    fn fr_141_times_name_their_source_and_cut_events() {
        use crate::sorting::time::TimeSource;
        let mut lib = Library::new();
        let f = lib.add_folder(Path::new("/roll"));
        let shot = lib.push(f, &file("/roll/IMG_0001.HEIC", 0), &meta_at(1));
        let named = lib.push(
            f,
            &file("/roll/2026-07-03 10-00-00.jpg", 0),
            &PhotoMeta::default(),
        );
        let bare = lib.push(f, &file("/roll/meme.jpg", 1_000), &PhotoMeta::default());
        let ids: Vec<PhotoId> = [shot, named, bare].into_iter().flatten().collect();
        assert_eq!(ids.len(), 3);
        let (shot, named, bare) = (ids[0], ids[1], ids[2]);
        assert_eq!(lib.time_source(shot), Some(TimeSource::ExifNoOffset));
        assert_eq!(lib.time_source(named), Some(TimeSource::FileName));
        assert_eq!(
            lib.time_source(bare),
            Some(TimeSource::FileTime { confirmed: false })
        );
        // The file time is not confirmed, so the bare file joins no event.
        assert_eq!(lib.roll().events, vec![vec![0], vec![1]]);
    }
}
