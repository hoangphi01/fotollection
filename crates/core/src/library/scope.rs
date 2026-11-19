// SPDX-License-Identifier: GPL-3.0-only
//! The right column's facts for the folders shown (owner, 2026-11-19: "the right column is
//! always about the folder chosen: one folder, several, or the Workspace"): the camera bodies
//! and their photos, and the photos with a stamp of their own. Empty `folders` is the Workspace,
//! every folder.

use super::{CameraClock, Library, PhotoId};

impl Library {
    /// Whether photo `i` is in `folders` (all of them when empty).
    fn in_scope(&self, i: usize, folders: &[u16]) -> bool {
        folders.is_empty() || self.folder.get(i).is_some_and(|f| folders.contains(f))
    }

    /// The camera bodies with photos in `folders`, each counting only those (FR-030).
    pub fn clocks_in(&self, folders: &[u16]) -> Vec<CameraClock> {
        let mut photos = vec![0u32; self.bodies.len()];
        for (i, &b) in self.body.iter().enumerate() {
            if self.in_scope(i, folders)
                && let Some(n) = photos.get_mut(b as usize)
            {
                *n += 1;
            }
        }
        let mut clocks = self.clocks();
        clocks.retain_mut(|c| {
            c.photos = photos.get(usize::from(c.id)).copied().unwrap_or(0);
            c.photos > 0
        });
        clocks
    }

    /// The photos of `folders` with their own stamp or caption (FR-044, FR-036): the Reset's
    /// count and what it takes.
    pub fn custom_stamps_in(&self, folders: &[u16]) -> Vec<PhotoId> {
        self.edited()
            .filter(|(id, e)| {
                (e.stamp.is_some() || e.caption.is_some())
                    && self.index(*id).is_some_and(|i| self.in_scope(i, folders))
            })
            .map(|(id, _)| id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::import::ImportedFile;
    use crate::library::Library;
    use crate::metadata::PhotoMeta;

    fn shot(make: &str) -> PhotoMeta {
        PhotoMeta {
            capture: chrono::NaiveDate::from_ymd_opt(2026, 11, 7)
                .and_then(|d| d.and_hms_opt(9, 0, 0)),
            make: Some(make.into()),
            model: Some(format!("{make} 1")),
            orientation: 1,
            ..PhotoMeta::default()
        }
    }

    #[test]
    fn fr_030_the_camera_clocks_are_the_folders_shown() {
        let mut lib = Library::new();
        let phone = lib.add_folder(Path::new("/phone"));
        let trip = lib.add_folder(Path::new("/trip"));
        for (k, (folder, make)) in [(phone, "Apple"), (phone, "Apple"), (trip, "Panasonic")]
            .into_iter()
            .enumerate()
        {
            let file = ImportedFile {
                path: PathBuf::from(format!("/f/{k}.jpg")),
                size: 1,
                mtime: 0,
            };
            lib.push(folder, &file, &shot(make));
        }
        let names = |folders: &[u16]| {
            lib.clocks_in(folders)
                .into_iter()
                .map(|c| (c.make.unwrap_or_default(), c.photos))
                .collect::<Vec<_>>()
        };
        assert_eq!(names(&[phone]), [("Apple".to_owned(), 2)]);
        assert_eq!(names(&[trip]), [("Panasonic".to_owned(), 1)]);
        assert_eq!(names(&[]).len(), 2, "the Workspace: every body");
        assert_eq!(names(&[phone, trip]).len(), 2);
    }
}
