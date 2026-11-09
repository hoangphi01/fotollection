// SPDX-License-Identifier: GPL-3.0-only
//! The prefetch (FR-091): once the photo on screen has landed, the preview asks for the next
//! ones in the direction of travel, and one worker thread loads them into memory from their
//! standard previews, behind the photo on screen.
//!
//! State kept (`STYLE_RULE` §2): a queue of at most [`MAX_PREFETCH`] photo ids, owned by
//! `AppState`. A new request replaces what is left, so fast browsing never piles up work; a
//! photo whose request was replaced, or whose library was closed, is dropped before its RAW
//! develop starts. One thread for the app's life, started by the first request (audit
//! 2026-11-06: before, each request started a thread of its own, and an old one still
//! developing a RAW kept going).

use std::collections::VecDeque;
use std::sync::atomic::Ordering;
use std::sync::{Condvar, Mutex, Once};

use fotollection_core::library::PhotoId;
use fotollection_core::raw::{Priority, with_priority};
use tauri::{AppHandle, Manager};

use crate::state::AppState;

/// Photos a prefetch takes at most; more would only push out what is near.
pub const MAX_PREFETCH: usize = 8;

/// The photos left to prefetch, from the latest request only.
pub struct Prefetch {
    queue: Mutex<Queue>,
    ready: Condvar,
    worker: Once,
}

impl Default for Prefetch {
    fn default() -> Self {
        Self {
            queue: Mutex::default(),
            ready: Condvar::new(),
            worker: Once::new(),
        }
    }
}

#[derive(Default)]
struct Queue {
    ids: VecDeque<PhotoId>,
    /// Bumped by each request.
    generation: u64,
    /// The library's generation when the request came (`AppState::library_generation`).
    library: u64,
}

/// One photo to prefetch, and the request it came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Next {
    id: PhotoId,
    generation: u64,
    library: u64,
}

impl Prefetch {
    fn lock(&self) -> std::sync::MutexGuard<'_, Queue> {
        // The queue is plain data; a panic elsewhere leaves it usable.
        self.queue
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Replaces what is left with the first [`MAX_PREFETCH`] of `ids`, asked in the library of
    /// generation `library`.
    fn replace(&self, mut ids: Vec<PhotoId>, library: u64) {
        ids.truncate(MAX_PREFETCH);
        let mut queue = self.lock();
        queue.generation += 1;
        queue.library = library;
        queue.ids = ids.into();
        drop(queue);
        self.ready.notify_one();
    }

    /// The next photo, waiting for one. Only the worker's plain thread calls it.
    fn next(&self) -> Next {
        let mut queue = self.lock();
        loop {
            if let Some(id) = queue.ids.pop_front() {
                return Next {
                    id,
                    generation: queue.generation,
                    library: queue.library,
                };
            }
            queue = self
                .ready
                .wait(queue)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
    }

    /// Whether `generation` is still the latest request.
    fn current(&self, generation: u64) -> bool {
        self.lock().generation == generation
    }
}

/// Asks for `ids` to be loaded (the latest request wins), starting the worker the first time.
pub fn request(app: &AppHandle, ids: Vec<PhotoId>) {
    let state = app.state::<AppState>();
    let library = state.library_generation.load(Ordering::SeqCst);
    state.prefetch.replace(ids, library);
    let worker = app.clone();
    // Started again if it panics (ADR-0048): before, a dead worker stayed dead (`Once`).
    state.prefetch.worker.call_once(move || {
        fotollection_core::supervise::forever("prefetch", move || work(&worker));
    });
}

/// The worker: one photo at a time, behind the photo on screen, for the app's life.
fn work(app: &AppHandle) {
    let state = app.state::<AppState>();
    loop {
        let next = state.prefetch.next();
        let wanted = || {
            state.prefetch.current(next.generation)
                && state.library_generation.load(Ordering::SeqCst) == next.library
        };
        if !wanted() {
            continue;
        }
        let record = state.library.read().ok().and_then(|l| l.record(next.id));
        let settings = state.settings.read().ok().map(|s| s.0.clone());
        if let (Some(record), Some(settings)) = (record, settings) {
            let _ = with_priority(Priority::Background, || {
                state
                    .previews
                    .prefetch(&record, &settings, state.codec.as_ref(), &wanted)
            });
        }
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)] // Tests start bare threads to race the code.
mod tests {
    use super::*;

    /// Audit 2026-11-06: a new request replaces what is left of the old one, which is no
    /// longer current, and never holds more than `MAX_PREFETCH` photos.
    #[test]
    fn fr_091_a_new_prefetch_replaces_the_old_one() {
        let prefetch = Prefetch::default();
        prefetch.replace((0..20).map(PhotoId).collect(), 0);
        let first = prefetch.next();
        assert_eq!(first.id, PhotoId(0));
        assert_eq!(prefetch.lock().ids.len(), MAX_PREFETCH - 1);
        prefetch.replace(vec![PhotoId(40), PhotoId(41)], 0);
        assert!(!prefetch.current(first.generation));
        assert_eq!(prefetch.next().id, PhotoId(40));
        assert_eq!(prefetch.next().id, PhotoId(41));
        assert!(prefetch.lock().ids.is_empty());
    }

    /// Audit 2026-11-06: twenty requests start one worker, not twenty threads.
    #[test]
    fn fr_091_one_worker_for_any_number_of_requests() {
        let prefetch = Prefetch::default();
        let started = std::sync::atomic::AtomicUsize::new(0);
        for i in 0..20 {
            prefetch.replace(vec![PhotoId(i)], 0);
            prefetch.worker.call_once(|| {
                started.fetch_add(1, Ordering::SeqCst);
            });
        }
        assert_eq!(started.load(Ordering::SeqCst), 1);
    }

    /// The worker waits for a request, on its plain thread, and wakes when one comes.
    #[test]
    fn the_worker_waits_for_a_request() {
        let prefetch = std::sync::Arc::new(Prefetch::default());
        let taken = {
            let prefetch = std::sync::Arc::clone(&prefetch);
            std::thread::spawn(move || prefetch.next())
        };
        prefetch.replace(vec![PhotoId(7)], 3);
        let next = taken.join().ok();
        assert_eq!(next.map(|n| (n.id, n.library)), Some((PhotoId(7), 3)));
    }
}
