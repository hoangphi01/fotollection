// SPDX-License-Identifier: GPL-3.0-only
//! The files of one capture gathered into a stack, one photo shown (FR-141 S3): a burst's
//! frames (Apple's `BurstUUID`), HEIC and JPG twins (Apple's `PhotoIdentifier`, or the same
//! name, folder and second: a name alone never decides, as a phone's counter wraps at 9999),
//! and an edited copy beside its original (`IMG_E0596` beside `IMG_0596`). A Live Photo's video
//! is not in the library: [`live_video`] finds it beside its still when asked.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::apple_note::AppleNote;

/// What ties one photo to another.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StackKind {
    /// The frames of one burst.
    Burst,
    /// One capture saved twice (HEIC and JPG), or two stills of one Live Photo.
    Twins,
    /// An edited copy beside its original: the phone's `IMG_E…`, or an app's or editor's
    /// version that kept Apple's photo identifier (60 of the roll's 69 stacks, 2026-11-17).
    EditedCopy,
}

/// What [`stacks`] reads of each photo.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StackKey {
    pub folder: u16,
    /// The file name without its extension, lower case, `IMG_E` read as `IMG_`.
    pub stem: String,
    /// Named `IMG_E…`: the phone's edited copy.
    pub edited: bool,
    /// Written by an app or an editor, not the camera (FR-141's source): the copy, never the
    /// original. Set by the caller, who knows the source.
    pub derived: bool,
    /// Lower case, no dot.
    pub ext: String,
    /// The capture time in whole seconds, `None` when the file has no date of its own.
    pub capture_s: Option<i64>,
    pub content_id: Option<String>,
    pub burst_id: Option<String>,
    pub photo_id: Option<String>,
}

impl StackKey {
    pub fn new(
        folder: u16,
        file_name: &str,
        capture_s: Option<i64>,
        apple: Option<&AppleNote>,
    ) -> Self {
        let path = Path::new(file_name);
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let (stem, edited) = match stem.strip_prefix("img_e") {
            Some(rest) if rest.starts_with(|c: char| c.is_ascii_digit()) => {
                (format!("img_{rest}"), true)
            }
            _ => (stem, false),
        };
        Self {
            folder,
            stem,
            edited,
            derived: false,
            ext,
            capture_s,
            content_id: apple.and_then(|a| a.content_id.clone()),
            burst_id: apple.and_then(|a| a.burst_id.clone()),
            photo_id: apple.and_then(|a| a.photo_id.clone()),
        }
    }
}

/// Photos of one capture: indexes into the keys given to [`stacks`], the one to show first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stack {
    pub kinds: Vec<StackKind>,
    /// In the order of the keys.
    pub members: Vec<u32>,
    /// The photo shown: the camera's original before any copy, HEIC before JPG, then the first.
    pub cover: u32,
}

/// A union-find over the photos, each union remembering why.
struct Groups {
    parent: Vec<usize>,
    links: Vec<(usize, StackKind)>,
}

impl Groups {
    fn root(&mut self, mut i: usize) -> usize {
        while self.parent[i] != i {
            self.parent[i] = self.parent[self.parent[i]];
            i = self.parent[i];
        }
        i
    }

    fn join(&mut self, a: usize, b: usize, kind: StackKind) {
        let (ra, rb) = (self.root(a), self.root(b));
        if ra != rb {
            self.parent[rb] = ra;
        }
        self.links.push((a, kind));
    }

    /// Joins every photo that shares a value with the first that had it.
    fn by<'a, K: std::hash::Hash + Eq>(
        &mut self,
        keys: &'a [StackKey],
        kind: impl Fn(&StackKey, &StackKey) -> StackKind,
        value: impl Fn(&'a StackKey) -> Option<K>,
    ) {
        let mut first: HashMap<K, usize> = HashMap::new();
        for (i, k) in keys.iter().enumerate() {
            if let Some(v) = value(k) {
                match first.get(&v) {
                    Some(&f) => self.join(f, i, kind(&keys[f], k)),
                    None => {
                        first.insert(v, i);
                    }
                }
            }
        }
    }
}

/// Every stack of two photos or more, in the order of their first member.
pub fn stacks(keys: &[StackKey]) -> Vec<Stack> {
    let mut g = Groups {
        parent: (0..keys.len()).collect(),
        links: Vec::new(),
    };
    let copy = |a: &StackKey, b: &StackKey| {
        if a.derived || b.derived {
            StackKind::EditedCopy
        } else {
            StackKind::Twins
        }
    };
    g.by(keys, |_, _| StackKind::Burst, |k| k.burst_id.as_deref());
    g.by(keys, copy, |k| k.photo_id.as_deref());
    g.by(keys, copy, |k| k.content_id.as_deref());
    g.by(keys, copy, |k| {
        Some((k.folder, k.stem.as_str(), k.edited, k.capture_s?))
    });
    // An edited copy joins the original of its folder and name.
    let originals: HashMap<(u16, &str), usize> = keys
        .iter()
        .enumerate()
        .filter(|(_, k)| !k.edited)
        .map(|(i, k)| ((k.folder, k.stem.as_str()), i))
        .collect();
    for (i, k) in keys.iter().enumerate().filter(|(_, k)| k.edited) {
        if let Some(&o) = originals.get(&(k.folder, k.stem.as_str())) {
            g.join(o, i, StackKind::EditedCopy);
        }
    }

    let mut by_root: HashMap<usize, Vec<u32>> = HashMap::new();
    let mut order = Vec::new();
    for i in 0..keys.len() {
        let r = g.root(i);
        let members = by_root.entry(r).or_insert_with(|| {
            order.push(r);
            Vec::new()
        });
        members.push(u32::try_from(i).unwrap_or(u32::MAX));
    }
    let mut kinds: HashMap<usize, Vec<StackKind>> = HashMap::new();
    for (a, kind) in std::mem::take(&mut g.links) {
        let list = kinds.entry(g.root(a)).or_default();
        if !list.contains(&kind) {
            list.push(kind);
        }
    }
    order
        .into_iter()
        .filter_map(|r| {
            let members = by_root.remove(&r).filter(|m| m.len() > 1)?;
            let cover = *members.iter().min_by_key(|&&i| {
                let k = &keys[i as usize];
                let heif = matches!(k.ext.as_str(), "heic" | "heif");
                (
                    k.edited || k.derived,
                    !heif,
                    k.capture_s.unwrap_or(i64::MAX),
                    i,
                )
            })?;
            Some(Stack {
                kinds: kinds.remove(&r).unwrap_or_default(),
                members,
                cover,
            })
        })
        .collect()
}

/// A Live Photo's video beside its still: the same name with `.MOV`, its content identifier
/// read back to be sure. One header read, only when asked.
pub fn live_video(still: &Path, content_id: &str) -> Option<PathBuf> {
    let stem = still.file_stem()?;
    ["MOV", "mov"]
        .into_iter()
        .map(|ext| still.with_file_name(stem).with_extension(ext))
        .find(|p| {
            p.is_file()
                && crate::quicktime::read_path(p)
                    .and_then(|v| v.content_id)
                    .is_some_and(|id| id == content_id)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(name: &str, capture_s: Option<i64>, note: &AppleNote) -> StackKey {
        StackKey::new(0, name, capture_s, Some(note))
    }

    #[test]
    fn fr_141_bursts_twins_and_edited_copies_stack() {
        let burst = AppleNote {
            burst_id: Some("B1".to_string()),
            ..AppleNote::default()
        };
        let keys = [
            key("IMG_0001.HEIC", Some(100), &burst),
            key("IMG_0002.HEIC", Some(101), &burst),
            // Twins by Apple's photo identifier, names apart.
            key(
                "IMG_0003.JPG",
                Some(200),
                &AppleNote {
                    photo_id: Some("P3".to_string()),
                    ..AppleNote::default()
                },
            ),
            key(
                "IMG_0003.HEIC",
                Some(200),
                &AppleNote {
                    photo_id: Some("P3".to_string()),
                    ..AppleNote::default()
                },
            ),
            // Twins by name and second, no identifier.
            key("IMG_0004.JPG", Some(300), &AppleNote::default()),
            key("IMG_0004.HEIC", Some(300), &AppleNote::default()),
            // The same name a counter's wrap later: not a twin.
            key("IMG_0005.HEIC", Some(400), &AppleNote::default()),
            key("IMG_0005.JPG", Some(90_000_400), &AppleNote::default()),
            // The phone's edited copy.
            key("IMG_E0006.JPG", Some(600), &AppleNote::default()),
            key("IMG_0006.HEIC", Some(500), &AppleNote::default()),
            // No date of its own: a name never decides.
            StackKey::new(0, "IMG_0007.JPG", None, None),
            StackKey::new(0, "IMG_0007.PNG", None, None),
        ];
        let s = stacks(&keys);
        let found: Vec<(Vec<StackKind>, Vec<u32>, u32)> = s
            .into_iter()
            .map(|s| (s.kinds, s.members, s.cover))
            .collect();
        assert_eq!(
            found,
            vec![
                (vec![StackKind::Burst], vec![0, 1], 0),
                (vec![StackKind::Twins], vec![2, 3], 3),
                (vec![StackKind::Twins], vec![4, 5], 5),
                (vec![StackKind::EditedCopy], vec![8, 9], 9),
            ]
        );
    }

    #[test]
    fn fr_141_a_burst_frame_saved_twice_is_one_stack() {
        let frame = |name: &str, s: i64| {
            key(
                name,
                Some(s),
                &AppleNote {
                    burst_id: Some("B".to_string()),
                    ..AppleNote::default()
                },
            )
        };
        let keys = [
            frame("IMG_0010.HEIC", 10),
            frame("IMG_0011.HEIC", 11),
            StackKey::new(0, "IMG_0011.JPG", Some(11), None),
        ];
        let s = stacks(&keys);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].members, vec![0, 1, 2]);
        assert_eq!(s[0].kinds, vec![StackKind::Burst, StackKind::Twins]);
    }

    #[test]
    fn fr_141_an_apps_version_stacks_under_the_cameras_original() {
        let note = AppleNote {
            photo_id: Some("P".to_string()),
            ..AppleNote::default()
        };
        let mut insta = key("4D127416.jpg", Some(7), &note);
        insta.derived = true;
        let keys = [insta, key("IMG_9279_1.HEIC", Some(7), &note)];
        let s = stacks(&keys);
        assert_eq!(s[0].kinds, vec![StackKind::EditedCopy]);
        assert_eq!(s[0].cover, 1);
    }

    #[test]
    fn fr_141_folders_keep_their_names_apart() {
        let keys = [
            StackKey::new(0, "IMG_0001.JPG", Some(5), None),
            StackKey::new(1, "IMG_0001.HEIC", Some(5), None),
        ];
        assert_eq!(stacks(&keys), Vec::new());
        assert!(!StackKey::new(0, "IMG_Edit.JPG", None, None).edited);
    }
}
