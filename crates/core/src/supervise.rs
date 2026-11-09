// SPDX-License-Identifier: GPL-3.0-only
//! Threads that never take the app down and never leave it waiting (ADR-0048). A panic in
//! one is logged by the app's panic hook; `spawn` then runs the job's `ended`, which sends its
//! end event and clears what it registered, so no progress bar or "running" state outlives
//! it; `forever` starts a worker of the app's life again after a short wait.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::Duration;

/// Runs `work` on a thread named `fotollection-<name>`. If `work` panics, `ended` runs on
/// the same thread; it does not run when `work` returns.
#[allow(clippy::disallowed_methods)] // The one place threads start.
pub fn spawn(
    name: &str,
    work: impl FnOnce() + Send + 'static,
    ended: impl FnOnce() + Send + 'static,
) {
    let started = std::thread::Builder::new()
        .name(format!("fotollection-{name}"))
        .spawn(move || {
            if catch_unwind(AssertUnwindSafe(work)).is_err() {
                // `ended` only sends and clears; a panic in it too stays on this thread.
                let _ = catch_unwind(AssertUnwindSafe(ended));
            }
        });
    if let Err(e) = started {
        crate::error_log::record("internal", &format!("no thread for {name}: {e}"));
    }
}

/// A worker for the app's life: `work` normally never returns; after a panic it starts again,
/// waiting 1 s, then 2, 4… up to a minute, so a fault that repeats does not spin.
pub fn forever(name: &'static str, work: impl Fn() + Send + 'static) {
    forever_from(name, Duration::from_secs(1), work);
}

fn forever_from(name: &'static str, first: Duration, work: impl Fn() + Send + 'static) {
    spawn(
        name,
        move || {
            let mut wait = first;
            while catch_unwind(AssertUnwindSafe(&work)).is_err() {
                std::thread::sleep(wait);
                wait = (wait * 2).min(Duration::from_secs(60));
            }
        },
        || {},
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::mpsc;

    fn quiet<T>(f: impl FnOnce() -> T) -> T {
        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let out = f();
        std::panic::set_hook(hook);
        out
    }

    /// ADR-0048: a job that panics still sends its end, so nothing waits for it for good.
    #[test]
    fn a_job_that_panics_still_ends() -> Result<(), mpsc::RecvTimeoutError> {
        let (tx, rx) = mpsc::channel();
        let got = quiet(|| {
            spawn(
                "test-job",
                || panic!("an export bug"),
                move || {
                    let _ = tx.send("ended");
                },
            );
            rx.recv_timeout(Duration::from_secs(5))
        });
        assert_eq!(got?, "ended");
        Ok(())
    }

    /// A job that returns does not run `ended`: its own code sent its end.
    #[test]
    fn a_job_that_returns_does_not_end_twice() {
        let (tx, rx) = mpsc::channel::<&str>();
        let done = tx.clone();
        spawn(
            "test-ok",
            move || {
                let _ = done.send("work");
            },
            move || {
                let _ = tx.send("ended");
            },
        );
        assert_eq!(rx.recv_timeout(Duration::from_secs(5)).ok(), Some("work"));
        assert!(rx.recv_timeout(Duration::from_millis(200)).is_err());
    }

    /// A worker of the app's life starts again after it panics.
    #[test]
    fn a_worker_starts_again_after_a_panic() -> Result<(), mpsc::RecvTimeoutError> {
        let runs = Arc::new(AtomicU32::new(0));
        let (tx, rx) = mpsc::channel();
        let got = quiet(|| {
            let counted = Arc::clone(&runs);
            forever_from("test-worker", Duration::from_millis(5), move || {
                let n = counted.fetch_add(1, Ordering::SeqCst) + 1;
                assert!(n >= 3, "fails twice");
                let _ = tx.send(n);
            });
            rx.recv_timeout(Duration::from_secs(5))
        });
        assert_eq!(got?, 3);
        Ok(())
    }
}
