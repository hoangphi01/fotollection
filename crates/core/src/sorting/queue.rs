// SPDX-License-Identifier: GPL-3.0-only
//! RollSense's queue of folders (FR-141; owner, 2026-11-20: ten folders pressed in turn are all
//! read, one after another). Each press asks for a scope; one pass reads one scope at a time on
//! every core, the next starting when it ends. A scope is kept by its folders' paths, never their
//! ids, which are renumbered when a folder is removed.
//!
//! Owner: the shell's `AppState`, behind one lock taken only between passes, never across
//! the pixels' work. Bound: no waiting scope lies within another or within the running one, so
//! the queue holds at most one entry per folder set asked for. Emptied by *Stop all* and by a
//! project switch.

use std::collections::VecDeque;
use std::path::PathBuf;

/// The folders one pass reads, by path; empty: the Workspace (every folder).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scope(Vec<PathBuf>);

impl Scope {
    /// A scope of these folders, sorted and without repeats; none is the Workspace.
    pub fn new(mut folders: Vec<PathBuf>) -> Self {
        folders.sort();
        folders.dedup();
        Self(folders)
    }

    pub fn folders(&self) -> &[PathBuf] {
        &self.0
    }

    pub fn is_workspace(&self) -> bool {
        self.0.is_empty()
    }

    /// Every photo of `self` is also one of `other`'s.
    pub fn within(&self, other: &Scope) -> bool {
        other.is_workspace() || (!self.is_workspace() && self.0.iter().all(|f| other.0.contains(f)))
    }
}

/// The scope being read and the scopes waiting, in the order asked.
#[derive(Debug, Default)]
pub struct SortQueue {
    running: Option<Scope>,
    waiting: VecDeque<Scope>,
}

impl SortQueue {
    pub fn running(&self) -> Option<&Scope> {
        self.running.as_ref()
    }

    pub fn waiting(&self) -> impl Iterator<Item = &Scope> {
        self.waiting.iter()
    }

    /// Asks for a scope. Nothing is added when it lies within the running scope or a waiting
    /// one; a waiting scope that lies within it gives way (the Workspace takes every folder's
    /// place). Returns whether the queue changed.
    pub fn push(&mut self, scope: Scope) -> bool {
        if self
            .running
            .iter()
            .chain(&self.waiting)
            .any(|s| scope.within(s))
        {
            return false;
        }
        self.waiting.retain(|s| !s.within(&scope));
        self.waiting.push_back(scope);
        true
    }

    /// The next scope to read, taken as running; `None` while one runs or none waits.
    pub fn start_next(&mut self) -> Option<Scope> {
        if self.running.is_some() {
            return None;
        }
        self.running = self.waiting.pop_front();
        self.running.clone()
    }

    /// The running scope's pass ended: read, stopped or failed.
    pub fn finish(&mut self) {
        self.running = None;
    }

    /// A waiting scope taken out of the queue. Returns whether it was there.
    pub fn remove(&mut self, scope: &Scope) -> bool {
        let before = self.waiting.len();
        self.waiting.retain(|s| s != scope);
        self.waiting.len() != before
    }

    /// *Stop all*: no scope waits any more. The running pass is stopped by its caller, which
    /// holds its token, and ends with [`SortQueue::finish`].
    pub fn clear(&mut self) {
        self.waiting.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope(folders: &[&str]) -> Scope {
        Scope::new(folders.iter().map(PathBuf::from).collect())
    }

    #[test]
    fn fr_141_a_folder_asked_while_another_reads_is_read_next() {
        let mut q = SortQueue::default();
        assert!(q.push(scope(&["/a"])));
        assert_eq!(q.start_next(), Some(scope(&["/a"])));
        // B is pressed while A reads: it waits, nothing else starts.
        assert!(q.push(scope(&["/b"])));
        assert_eq!(q.start_next(), None);
        q.finish();
        assert_eq!(q.start_next(), Some(scope(&["/b"])));
        q.finish();
        assert_eq!(q.start_next(), None);
    }

    #[test]
    fn fr_141_ten_folders_asked_in_turn_are_all_read() {
        let mut q = SortQueue::default();
        let names: Vec<String> = (0..10).map(|i| format!("/trip-{i}")).collect();
        for name in &names {
            q.push(scope(&[name]));
        }
        let mut read = Vec::new();
        while let Some(s) = q.start_next() {
            read.push(s);
            q.finish();
        }
        let asked: Vec<Scope> = names.iter().map(|n| scope(&[n])).collect();
        assert_eq!(read, asked);
    }

    #[test]
    fn fr_141_a_folder_inside_the_running_scope_is_not_queued() {
        let mut q = SortQueue::default();
        q.push(scope(&["/a", "/b"]));
        q.start_next();
        assert!(!q.push(scope(&["/b"])), "within the running scope");
        assert!(
            !q.push(scope(&["/b", "/a", "/a"])),
            "the same set, asked again"
        );
        assert!(q.push(scope(&["/c"])));
        assert!(!q.push(scope(&["/c"])), "already waiting");
        assert_eq!(q.waiting().count(), 1);
        // The Workspace takes every waiting folder's place, and covers anything asked after it.
        assert!(q.push(Scope::new(Vec::new())));
        assert_eq!(q.waiting().collect::<Vec<_>>(), [&Scope::new(Vec::new())]);
        assert!(!q.push(scope(&["/d"])));
    }

    #[test]
    fn fr_141_stopping_the_running_folder_starts_the_next() {
        let mut q = SortQueue::default();
        for name in ["/a", "/b", "/c"] {
            q.push(scope(&[name]));
        }
        q.start_next();
        // Stopped: its pass ends, the next one starts.
        q.finish();
        assert_eq!(q.start_next(), Some(scope(&["/b"])));
        // A waiting one removed by its row's button; Stop all empties the rest.
        assert!(q.remove(&scope(&["/c"])));
        assert!(!q.remove(&scope(&["/c"])));
        q.push(scope(&["/d"]));
        q.clear();
        q.finish();
        assert_eq!(q.start_next(), None);
    }
}
