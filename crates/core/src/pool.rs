// SPDX-License-Identifier: GPL-3.0-only
//! The app's thread pools, built one way (ADR-0048): a panic in a job is logged by the app's
//! panic hook and stops there. Without a handler, rayon aborts the whole process on a job
//! that panics, so one bad photo would take the app down.

use rayon::{ThreadPool, ThreadPoolBuildError, ThreadPoolBuilder};

/// A pool of `threads` (rayon's choice when `None`), its threads named `fotollection-<name>-N`,
/// that survives a panicking job.
pub fn contained(
    threads: Option<usize>,
    name: &'static str,
) -> Result<ThreadPool, ThreadPoolBuildError> {
    let mut builder = ThreadPoolBuilder::new()
        .thread_name(move |i| format!("fotollection-{name}-{i}"))
        .panic_handler(|_| {});
    if let Some(n) = threads {
        builder = builder.num_threads(n.max(1));
    }
    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    /// ADR-0048: a job that panics on a pool leaves the process and the pool alive; the next
    /// job still runs. On a pool without a handler the test process itself aborts.
    #[test]
    fn a_panicking_job_does_not_abort_the_app() -> Result<(), Box<dyn std::error::Error>> {
        let pool = contained(Some(2), "test")?;
        let quiet = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        pool.spawn(|| panic!("a corrupt photo"));
        let (tx, rx) = mpsc::channel();
        pool.spawn(move || {
            let _ = tx.send(42);
        });
        let got = rx.recv_timeout(Duration::from_secs(5));
        std::panic::set_hook(quiet);
        assert_eq!(got?, 42);
        Ok(())
    }
}
