// SPDX-License-Identifier: GPL-3.0-only
//! Undo/redo for photo edits (FR-023): a stack of inverse operations, never snapshots
//! (DATABASE.md §3.3). Each step has a serial number, so the develop view's History list
//! (FR-095) can name one and bring a photo back to it.
//!
//! State kept (`STYLE_RULE` §2): at most [`MAX_STEPS`] steps each way, owned by `AppState`;
//! emptied when another project opens ([`History::clear`]).

use serde::Serialize;
use specta::Type;

use crate::library::{Library, PhotoEdits, PhotoId};
use crate::looks::{Adjust, LookSettings};

pub const MAX_STEPS: usize = 200;

#[derive(Debug, Clone, PartialEq)]
pub struct Change {
    pub id: PhotoId,
    pub before: PhotoEdits,
    pub after: PhotoEdits,
}

/// What a step was, when it was more than its changes say (FR-097): a paste of copied
/// settings or a reset, named so in the History list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum StepKind {
    Paste,
    Reset,
}

/// One step: the photos it changed, its serial number and, for a paste or reset, its kind.
#[derive(Debug)]
struct Step {
    serial: u64,
    changes: Vec<Change>,
    kind: Option<StepKind>,
}

#[derive(Debug, Default)]
pub struct History {
    undo: Vec<Step>,
    redo: Vec<Step>,
    /// The next step's serial number; never reused, so a name stays true after a trim.
    next: u64,
}

/// One of a photo's steps for the develop view's History (FR-095): what it was before and
/// after, and the step's serial number.
#[derive(Debug, Clone, PartialEq)]
pub struct PhotoStep {
    pub serial: u64,
    pub before: PhotoEdits,
    pub after: PhotoEdits,
    pub kind: Option<StepKind>,
}

/// What a step changed, to name it in the History list as Lightroom does (*Warmth +8*).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum HistoryEdit {
    /// The colour look: one slider (`field`, as the look cards name it, `amount` among them,
    /// and its new value), a look of the library picked (`name`, FR-079), or several at once
    /// (all none: *Apply to all*, a reset, a paste).
    Look {
        field: Option<String>,
        value: Option<i32>,
        name: Option<String>,
    },
    RawLook,
    Crop,
    Straighten,
    Rotate,
    /// The print size laid the other way from the photo, or back (FR-120).
    PrintTurn,
    Stamp,
    Caption,
}

/// Whether a step touched what the develop view shows: the look, the crop and straightening,
/// the turns, the stamp and its line. Ratings, flags, the basket, dates and places are left
/// out, as Lightroom's History leaves out ratings and flags.
fn develops(before: &PhotoEdits, after: &PhotoEdits) -> bool {
    !describe(before, after).is_empty()
}

/// The look sliders that hold a number, by the name the look cards give them.
fn sliders(a: &Adjust) -> [(&'static str, i32); 12] {
    [
        ("brightness", a.brightness.into()),
        ("contrast", a.contrast.into()),
        ("highlights", a.highlights.into()),
        ("shadows", a.shadows.into()),
        ("warmth", a.warmth.into()),
        ("tint", a.tint.into()),
        ("saturation", a.saturation.into()),
        ("vibrance", a.vibrance.into()),
        ("fade", a.fade.into()),
        ("balance", a.balance.into()),
        ("vignette", a.vignette.into()),
        ("dehaze", a.dehaze.into()),
    ]
}

/// What changed between two states of a photo, in the order the cards show it; empty when
/// nothing the develop view shows changed.
pub fn describe(before: &PhotoEdits, after: &PhotoEdits) -> Vec<HistoryEdit> {
    let mut edits = Vec::new();
    if before.raw_look != after.raw_look {
        edits.push(HistoryEdit::RawLook);
    }
    if before.look != after.look {
        let none = LookSettings::default();
        let (b, a) = (
            before.look.as_ref().unwrap_or(&none),
            after.look.as_ref().unwrap_or(&none),
        );
        let moved: Vec<(&str, i32)> = sliders(&b.adjust)
            .into_iter()
            .zip(sliders(&a.adjust))
            .filter(|(x, y)| x.1 != y.1)
            .map(|(_, y)| y)
            .collect();
        let only_one_slider = {
            let mut same = b.clone();
            same.adjust = a.adjust;
            moved.len() == 1 && same == *a && before.look.is_some() == after.look.is_some()
        };
        // A pick from the library names its look; the amount alone is a slider of its own.
        let picked = a.base.is_some() && a.base != b.base;
        let amount_only = {
            let mut same = b.clone();
            same.amount = a.amount;
            b.amount != a.amount && same == *a && before.look.is_some() == after.look.is_some()
        };
        edits.push(match moved.first() {
            _ if picked => HistoryEdit::Look {
                field: None,
                value: None,
                name: a.name.clone(),
            },
            _ if amount_only => HistoryEdit::Look {
                field: Some("amount".to_owned()),
                value: Some(a.amount.into()),
                name: None,
            },
            Some((field, value)) if only_one_slider => HistoryEdit::Look {
                field: Some((*field).to_owned()),
                value: Some(*value),
                name: None,
            },
            _ => HistoryEdit::Look {
                field: None,
                value: None,
                name: None,
            },
        });
    }
    if before.crop != after.crop {
        edits.push(HistoryEdit::Crop);
    }
    if before.straighten.to_bits() != after.straighten.to_bits() {
        edits.push(HistoryEdit::Straighten);
    }
    if before.rotation != after.rotation {
        edits.push(HistoryEdit::Rotate);
    }
    if before.print_turned != after.print_turned {
        edits.push(HistoryEdit::PrintTurn);
    }
    if before.stamp != after.stamp {
        edits.push(HistoryEdit::Stamp);
    }
    if before.caption != after.caption {
        edits.push(HistoryEdit::Caption);
    }
    edits
}

impl History {
    /// Applies `after` to each photo and records the step. Returns the changed photos.
    pub fn apply(
        &mut self,
        library: &mut Library,
        edits: &[(PhotoId, PhotoEdits)],
    ) -> Vec<PhotoId> {
        self.apply_as(library, edits, None)
    }

    /// Like [`apply`](Self::apply), recording what the step was (FR-097: a paste, a reset).
    pub fn apply_as(
        &mut self,
        library: &mut Library,
        edits: &[(PhotoId, PhotoEdits)],
        kind: Option<StepKind>,
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
            self.undo.push(Step {
                serial: self.next,
                changes: step,
                kind,
            });
            self.next += 1;
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
            step.changes.len() == edits.len()
                && step
                    .changes
                    .iter()
                    .zip(edits)
                    .all(|(c, (id, _))| c.id == *id)
        });
        let Some(step) = self.undo.last_mut().filter(|_| same) else {
            return self.apply(library, edits);
        };
        for (change, (id, after)) in step.changes.iter_mut().zip(edits) {
            library.set_edits(*id, after.clone());
            change.after = after.clone();
        }
        self.redo.clear();
        edits.iter().map(|(id, _)| *id).collect()
    }

    pub fn undo(&mut self, library: &mut Library) -> Option<Vec<PhotoId>> {
        let step = self.undo.pop()?;
        for c in step.changes.iter().rev() {
            library.set_edits(c.id, c.before.clone());
        }
        let ids = step.changes.iter().map(|c| c.id).collect();
        self.redo.push(step);
        Some(ids)
    }

    pub fn redo(&mut self, library: &mut Library) -> Option<Vec<PhotoId>> {
        let step = self.redo.pop()?;
        for c in &step.changes {
            library.set_edits(c.id, c.after.clone());
        }
        let ids = step.changes.iter().map(|c| c.id).collect();
        self.undo.push(step);
        Some(ids)
    }

    /// `id`'s steps that changed what the develop view shows, newest first (FR-095). Steps
    /// undone are not listed: as in Lightroom, a new edit drops them.
    pub fn photo_steps(&self, id: PhotoId) -> Vec<PhotoStep> {
        self.undo
            .iter()
            .rev()
            .filter_map(|step| {
                let c = step.changes.iter().find(|c| c.id == id)?;
                develops(&c.before, &c.after).then(|| PhotoStep {
                    serial: step.serial,
                    before: c.before.clone(),
                    after: c.after.clone(),
                    kind: step.kind,
                })
            })
            .collect()
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

    fn library_with(n: u32) -> (Library, Vec<PhotoId>) {
        let mut lib = Library::new();
        let folder = lib.add_folder(Path::new("/x"));
        let ids = (0..n)
            .filter_map(|i| {
                let file = ImportedFile {
                    path: PathBuf::from(format!("/x/{i}.jpg")),
                    size: 1,
                    mtime: 1,
                };
                lib.push(folder, &file, &PhotoMeta::default())
            })
            .collect();
        (lib, ids)
    }

    // `Some`, as a photo's own look is stored.
    #[allow(clippy::unnecessary_wraps)]
    fn look(edit: impl FnOnce(&mut Adjust)) -> Option<LookSettings> {
        let mut l = LookSettings::default();
        edit(&mut l.adjust);
        Some(l)
    }

    /// FR-095: one slider names itself and its value, as Lightroom's History does; several
    /// at once, or a look given or taken away, is "the look"; each other kind names itself;
    /// a rating is not a develop step.
    #[test]
    fn fr_095_a_step_says_what_it_changed() {
        let none = PhotoEdits::default();
        let warm = PhotoEdits {
            look: look(|a| a.warmth = 8),
            ..PhotoEdits::default()
        };
        let one = |field: &str, value| HistoryEdit::Look {
            field: Some(field.to_owned()),
            value: Some(value),
            name: None,
        };
        let many = HistoryEdit::Look {
            field: None,
            value: None,
            name: None,
        };
        let warmer = PhotoEdits {
            look: look(|a| a.warmth = 20),
            ..PhotoEdits::default()
        };
        assert_eq!(describe(&warm, &warmer), vec![one("warmth", 20)]);
        assert_eq!(describe(&none, &warm), vec![many.clone()]);
        let both = PhotoEdits {
            look: look(|a| {
                a.warmth = 20;
                a.fade = 10;
            }),
            ..PhotoEdits::default()
        };
        assert_eq!(describe(&warm, &both), vec![many]);
        // FR-079: a look picked says its name; the amount alone is a slider of its own.
        let picked = PhotoEdits {
            look: look(|a| a.warmth = 25).map(|l| LookSettings {
                name: Some("Warm old film".into()),
                base: Some("builtin:warm-film".into()),
                ..l
            }),
            ..PhotoEdits::default()
        };
        assert_eq!(
            describe(&warm, &picked),
            vec![HistoryEdit::Look {
                field: None,
                value: None,
                name: Some("Warm old film".into()),
            }]
        );
        let softer = PhotoEdits {
            look: picked
                .look
                .clone()
                .map(|l| LookSettings { amount: 60, ..l }),
            ..PhotoEdits::default()
        };
        assert_eq!(describe(&picked, &softer), vec![one("amount", 60)]);
        let mut all = warm.clone();
        all.raw_look = Some(crate::raw_finish::RawLook::Camera);
        all.crop = Some(crate::plan::CropFrame {
            cx: 0.4,
            cy: 0.5,
            zoom: 1.2,
        });
        all.straighten = 1.5;
        all.rotation = 1;
        all.stamp = Some(crate::settings::RenderSettings::default().stamp);
        all.caption = Some("x".into());
        assert_eq!(
            describe(&warm, &all),
            vec![
                HistoryEdit::RawLook,
                HistoryEdit::Crop,
                HistoryEdit::Straighten,
                HistoryEdit::Rotate,
                HistoryEdit::Stamp,
                HistoryEdit::Caption,
            ]
        );
        let rated = PhotoEdits {
            rating: 4,
            ..warm.clone()
        };
        assert_eq!(describe(&warm, &rated), Vec::new());
    }

    /// FR-095: a photo's History lists only its develop steps, newest first, and a step
    /// keeps its serial after the stack trims its oldest.
    #[test]
    fn fr_095_a_photos_history_is_its_develop_steps_newest_first() {
        let (mut lib, ids) = library_with(2);
        let (a, b) = (ids[0], ids[1]);
        let mut h = History::default();
        let warm = |v| PhotoEdits {
            look: look(|x| x.warmth = v),
            ..PhotoEdits::default()
        };
        h.apply(&mut lib, &[(a, warm(5))]);
        h.apply(&mut lib, &[(b, warm(9))]);
        let mut rated = lib.edits(a);
        rated.rating = 3;
        h.apply(&mut lib, &[(a, rated)]);
        h.apply(
            &mut lib,
            &[(
                a,
                PhotoEdits {
                    rating: 3,
                    ..warm(7)
                },
            )],
        );
        let steps = h.photo_steps(a);
        assert_eq!(
            steps.iter().map(|s| s.serial).collect::<Vec<_>>(),
            vec![3, 0]
        );
        assert_eq!(steps[0].after.look, warm(7).look);
        for v in 0..MAX_STEPS {
            h.apply(&mut lib, &[(b, warm(i8::try_from(v % 100).unwrap_or(0)))]);
        }
        assert!(h.photo_steps(a).is_empty(), "trimmed away");
        let newest = h.photo_steps(b);
        assert_eq!(newest.first().map(|s| s.serial), Some(h.next - 1));
    }

    /// FR-095: going back in History brings the develop fields back and keeps the rating,
    /// flag and basket the photo has now.
    #[test]
    fn fr_095_a_restore_keeps_the_rating() {
        let now = PhotoEdits {
            rating: 5,
            flag: Some(crate::library::Flag::Pick),
            look: look(|a| a.contrast = 30),
            rotation: 2,
            ..PhotoEdits::default()
        };
        let then = PhotoEdits {
            rating: 1,
            look: look(|a| a.contrast = 10),
            ..PhotoEdits::default()
        };
        let back = now.with_develop_of(&then);
        assert_eq!(
            (back.rating, back.flag),
            (5, Some(crate::library::Flag::Pick))
        );
        assert_eq!((back.look, back.rotation), (then.look, 0));
    }

    /// FR-097: settings pasted to many photos are one step, named a paste, and one undo
    /// brings every photo back; only the chosen parts travel.
    #[test]
    fn fr_097_paste_is_one_step_for_every_ticked_photo() {
        use crate::library::SettingsParts;
        let (mut lib, ids) = library_with(4);
        let mut history = History::default();
        let source = PhotoEdits {
            look: look(|a| a.warmth = 30),
            crop: Some(crate::plan::CropFrame {
                cx: 0.4,
                cy: 0.5,
                zoom: 1.5,
            }),
            straighten: 2.5,
            rating: 5,
            ..PhotoEdits::default()
        };
        let parts = SettingsParts {
            look: true,
            crop: true,
            ..SettingsParts::default()
        };
        let targets = &ids[1..];
        let edits: Vec<_> = targets
            .iter()
            .map(|&id| (id, lib.edits(id).with_parts_of(&source, parts, false)))
            .collect();
        history.apply_as(&mut lib, &edits, Some(StepKind::Paste));
        for &id in targets {
            let e = lib.edits(id);
            assert_eq!(e.look, source.look);
            assert_eq!(
                (e.crop, e.straighten.to_bits()),
                (source.crop, 2.5f32.to_bits())
            );
            assert_eq!(e.rating, 0, "a paste never carries the rating");
        }
        assert_eq!(history.photo_steps(ids[1])[0].kind, Some(StepKind::Paste));
        assert!(history.undo(&mut lib).is_some());
        for &id in targets {
            assert_eq!(lib.edits(id), PhotoEdits::default());
        }
        assert!(!history.can_undo(), "one step for the whole paste");
    }

    /// FR-097: the stamp travels with its line; parts not chosen stay the photo's own.
    #[test]
    fn fr_097_paste_copies_only_the_chosen_parts() {
        use crate::library::SettingsParts;
        let own = PhotoEdits {
            look: look(|a| a.contrast = -20),
            raw_look: Some(crate::raw_finish::RawLook::Camera),
            ..PhotoEdits::default()
        };
        let source = PhotoEdits {
            look: look(|a| a.contrast = 40),
            stamp: Some(crate::settings::RenderSettings::default().stamp),
            caption: Some("PRAHA".into()),
            ..PhotoEdits::default()
        };
        let stamp_only = SettingsParts {
            stamp: true,
            ..SettingsParts::default()
        };
        let pasted = own.with_parts_of(&source, stamp_only, false);
        assert_eq!(pasted.look, own.look);
        assert_eq!(pasted.raw_look, own.raw_look);
        assert_eq!(
            (pasted.stamp, pasted.caption),
            (source.stamp, source.caption)
        );
        assert!(!SettingsParts::default().any());
    }

    /// FR-097: on a collage sheet the crop pasted is the collage crop, and the print crop
    /// is left alone.
    #[test]
    fn fr_097_a_collage_paste_moves_the_collage_crop() {
        use crate::library::SettingsParts;
        let frame = crate::plan::CropFrame {
            cx: 0.3,
            cy: 0.6,
            zoom: 2.0,
        };
        let source = PhotoEdits {
            collage_crop: Some(frame),
            collage_straighten: Some(-3.0),
            ..PhotoEdits::default()
        };
        let crop = SettingsParts {
            crop: true,
            ..SettingsParts::default()
        };
        let pasted = PhotoEdits::default().with_parts_of(&source, crop, true);
        assert_eq!(
            (pasted.collage_crop, pasted.collage_straighten),
            (Some(frame), Some(-3.0))
        );
        assert_eq!(pasted.crop, None);
    }

    /// FR-097: Reset gives the photo back to the shared look and stamp, with no crop of its
    /// own, as one step named a reset; its stars, flag and turns stay.
    #[test]
    fn fr_097_reset_returns_the_shared_look_and_stamp() {
        let (mut lib, ids) = library_with(1);
        let mut history = History::default();
        let edited = PhotoEdits {
            look: look(|a| a.saturation = 25),
            raw_look: Some(crate::raw_finish::RawLook::Camera),
            crop: Some(crate::plan::CropFrame {
                cx: 0.5,
                cy: 0.5,
                zoom: 1.2,
            }),
            straighten: 4.0,
            stamp: Some(crate::settings::RenderSettings::default().stamp),
            caption: Some("HA NOI".into()),
            rating: 4,
            rotation: 1,
            ..PhotoEdits::default()
        };
        history.apply(&mut lib, &[(ids[0], edited.clone())]);
        let reset = lib.edits(ids[0]).reset_develop(false);
        history.apply_as(&mut lib, &[(ids[0], reset)], Some(StepKind::Reset));
        let back = lib.edits(ids[0]);
        assert_eq!(
            (
                back.look,
                back.raw_look,
                back.crop,
                back.stamp,
                back.caption
            ),
            (None, None, None, None, None)
        );
        assert_eq!(back.straighten.to_bits(), 0f32.to_bits());
        assert_eq!((back.rating, back.rotation), (4, 1));
        assert_eq!(history.photo_steps(ids[0])[0].kind, Some(StepKind::Reset));
        assert!(history.undo(&mut lib).is_some());
        assert_eq!(lib.edits(ids[0]), edited);
    }

    /// FR-120: laying the print size the other way is a step of its own, named in History.
    #[test]
    fn fr_120_a_turned_print_size_is_a_named_step() {
        let before = PhotoEdits::default();
        let after = PhotoEdits {
            print_turned: true,
            ..PhotoEdits::default()
        };
        assert_eq!(describe(&before, &after), vec![HistoryEdit::PrintTurn]);
        assert!(develops(&before, &after));
    }
}
