// SPDX-License-Identifier: GPL-3.0-only
//! Undo/redo for photo edits (FR-023): a stack of inverse operations, never snapshots
//! (DATABASE.md §3.3).

use crate::library::{Library, PhotoEdits, PhotoId};

pub const MAX_STEPS: usize = 200;

#[derive(Debug, Clone, PartialEq)]
pub struct Change {
    pub id: PhotoId,
    pub before: PhotoEdits,
    pub after: PhotoEdits,
}

#[derive(Debug, Default)]
pub struct History {
    undo: Vec<Vec<Change>>,
    redo: Vec<Vec<Change>>,
}

impl History {
    /// Applies `after` to each photo and records the step. Returns the changed photos.
    pub fn apply(
        &mut self,
        library: &mut Library,
        edits: &[(PhotoId, PhotoEdits)],
    ) -> Vec<PhotoId> {
        let step: Vec<Change> = edits
            .iter()
            .map(|(id, after)| Change {
                id: *id,
                before: library.set_edits(*id, after.clone()),
                after: after.clone(),
            })
            .filter(|c| c.before != c.after)
            .collect();
        let ids = step.iter().map(|c| c.id).collect();
        if !step.is_empty() {
            self.undo.push(step);
            if self.undo.len() > MAX_STEPS {
                self.undo.remove(0);
            }
            self.redo.clear();
        }
        ids
    }

    /// Like [`apply`](Self::apply), but folded into the last step when that step changed
    /// exactly the same photos: a drag sends many positions and is undone in one go (FR-006).
    pub fn amend(
        &mut self,
        library: &mut Library,
        edits: &[(PhotoId, PhotoEdits)],
    ) -> Vec<PhotoId> {
        let same = self.undo.last().is_some_and(|step| {
            step.len() == edits.len() && step.iter().zip(edits).all(|(c, (id, _))| c.id == *id)
        });
        let Some(step) = self.undo.last_mut().filter(|_| same) else {
            return self.apply(library, edits);
        };
        for (change, (id, after)) in step.iter_mut().zip(edits) {
            library.set_edits(*id, after.clone());
            change.after = after.clone();
        }
        self.redo.clear();
        edits.iter().map(|(id, _)| *id).collect()
    }

    pub fn undo(&mut self, library: &mut Library) -> Option<Vec<PhotoId>> {
        let step = self.undo.pop()?;
        for c in step.iter().rev() {
            library.set_edits(c.id, c.before.clone());
        }
        let ids = step.iter().map(|c| c.id).collect();
        self.redo.push(step);
        Some(ids)
    }

    pub fn redo(&mut self, library: &mut Library) -> Option<Vec<PhotoId>> {
        let step = self.redo.pop()?;
        for c in &step {
            library.set_edits(c.id, c.after.clone());
        }
        let ids = step.iter().map(|c| c.id).collect();
        self.undo.push(step);
        Some(ids)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Forget everything, e.g. when a different project is opened.
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::ImportedFile;
    use crate::metadata::PhotoMeta;
    use std::path::{Path, PathBuf};

    #[test]
    fn fr_023_undo_redo_round_trip() {
        let mut lib = Library::new();
        let folder = lib.add_folder(Path::new("/x"));
        let file = ImportedFile {
            path: PathBuf::from("/x/a.jpg"),
            size: 1,
            mtime: 1,
        };
        let Some(id) = lib.push(folder, &file, &PhotoMeta::default()) else {
            panic!("photo");
        };
        let mut history = History::default();
        let rated = PhotoEdits {
            rating: 3,
            ..PhotoEdits::default()
        };
        history.apply(&mut lib, &[(id, rated.clone())]);
        assert_eq!(lib.edits(id).rating, 3);
        assert!(history.undo(&mut lib).is_some());
        assert_eq!(lib.edits(id), PhotoEdits::default());
        assert!(history.can_redo());
        history.redo(&mut lib);
        assert_eq!(lib.edits(id).rating, 3);
        // A no-op change does not create a step.
        history.apply(&mut lib, &[(id, rated)]);
        assert!(history.undo(&mut lib).is_some());
        assert!(!history.can_undo());
    }

    #[test]
    fn fr_006_a_drag_is_one_undo_step() {
        let mut lib = Library::new();
        let folder = lib.add_folder(Path::new("/x"));
        let file = |name: &str| ImportedFile {
            path: PathBuf::from(format!("/x/{name}")),
            size: 1,
            mtime: 1,
        };
        let (Some(a), Some(b)) = (
            lib.push(folder, &file("a.jpg"), &PhotoMeta::default()),
            lib.push(folder, &file("b.jpg"), &PhotoMeta::default()),
        ) else {
            panic!("photos");
        };
        let at = |cy: f32| PhotoEdits {
            crop: Some(crate::plan::CropFrame {
                cx: 0.5,
                cy,
                zoom: 1.0,
            }),
            ..PhotoEdits::default()
        };
        let mut history = History::default();
        history.apply(&mut lib, &[(a, at(0.4))]);
        history.amend(&mut lib, &[(a, at(0.3))]);
        history.amend(&mut lib, &[(a, at(0.2))]);
        assert_eq!(lib.edits(a), at(0.2));
        // Another photo starts its own step.
        history.amend(&mut lib, &[(b, at(0.6))]);
        assert!(history.undo(&mut lib).is_some());
        assert_eq!(lib.edits(b), PhotoEdits::default());
        // One undo takes the whole drag back.
        assert!(history.undo(&mut lib).is_some());
        assert_eq!(lib.edits(a), PhotoEdits::default());
        assert!(!history.can_undo());
    }
}
