// SPDX-License-Identifier: GPL-3.0-only
//! Locks that heal (ADR-0048). The same API as `std::sync::{Mutex, RwLock}`, but a lock left
//! poisoned by a panic in another thread gives its data back and is cleared, once logged as
//! `ERR-RUS-0003`: with std's, a poisoned library lock failed every later command and stopped
//! autosave for the rest of the run. The `Err` of `lock`, `read` and `write` never happens;
//! it is kept so callers read as with std's locks.

use std::sync::{MutexGuard, RwLockReadGuard, RwLockWriteGuard};

/// The error a healing lock never returns (`Display` for `map_err` at the call sites).
#[derive(Debug)]
pub struct Never(());

impl std::fmt::Display for Never {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("a lock that heals never fails")
    }
}

/// A lock another thread holds now (`Mutex::try_lock`).
#[derive(Debug)]
pub struct Busy(());

impl From<Never> for String {
    fn from(never: Never) -> Self {
        never.to_string()
    }
}

fn healed(what: &str) {
    crate::error_log::record(
        "internal",
        &format!("a {what} was poisoned by a panic; its data kept, the lock cleared"),
    );
}

#[derive(Debug, Default)]
pub struct Mutex<T>(std::sync::Mutex<T>);

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Self(std::sync::Mutex::new(value))
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn lock(&self) -> Result<MutexGuard<'_, T>, Never> {
        Ok(self.locked())
    }

    /// The guard itself, for code that has no error to pass on.
    pub fn locked(&self) -> MutexGuard<'_, T> {
        self.0.lock().unwrap_or_else(|poisoned| {
            healed("mutex");
            self.0.clear_poison();
            poisoned.into_inner()
        })
    }

    /// The guard if the lock is free now, `Err` while another holds it; a poisoned lock heals
    /// as in [`Mutex::lock`].
    #[allow(clippy::missing_errors_doc)]
    pub fn try_lock(&self) -> Result<MutexGuard<'_, T>, Busy> {
        match self.0.try_lock() {
            Ok(guard) => Ok(guard),
            Err(std::sync::TryLockError::Poisoned(poisoned)) => {
                healed("mutex");
                self.0.clear_poison();
                Ok(poisoned.into_inner())
            }
            Err(std::sync::TryLockError::WouldBlock) => Err(Busy(())),
        }
    }

    pub fn into_inner(self) -> T {
        self.0
            .into_inner()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[derive(Debug, Default)]
pub struct RwLock<T>(std::sync::RwLock<T>);

impl<T> RwLock<T> {
    pub const fn new(value: T) -> Self {
        Self(std::sync::RwLock::new(value))
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn read(&self) -> Result<RwLockReadGuard<'_, T>, Never> {
        Ok(self.0.read().unwrap_or_else(|poisoned| {
            healed("read lock");
            self.0.clear_poison();
            poisoned.into_inner()
        }))
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn write(&self) -> Result<RwLockWriteGuard<'_, T>, Never> {
        Ok(self.0.write().unwrap_or_else(|poisoned| {
            healed("write lock");
            self.0.clear_poison();
            poisoned.into_inner()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// ADR-0048: a panic while holding the lock leaves its data usable and the lock clear;
    /// with std's lock every later `lock()` fails.
    #[test]
    fn a_poisoned_lock_heals() {
        let lock = Arc::new(Mutex::new(vec![1, 2]));
        let held = Arc::clone(&lock);
        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let Ok(mut guard) = held.lock() else { return };
            guard.push(3);
            panic!("a bug while editing the library");
        }));
        std::panic::set_hook(hook);
        let Ok(guard) = lock.lock() else {
            panic!("a healing lock never fails");
        };
        assert_eq!(*guard, vec![1, 2, 3]);
        drop(guard);
        assert!(
            !lock.0.is_poisoned(),
            "cleared, so the next panic is logged again"
        );
    }

    #[test]
    fn a_poisoned_rwlock_heals() {
        let lock = Arc::new(RwLock::new(0));
        let held = Arc::clone(&lock);
        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let Ok(mut guard) = held.write() else { return };
            *guard = 5;
            panic!("a bug");
        }));
        std::panic::set_hook(hook);
        let Ok(value) = lock.read().map(|g| *g) else {
            panic!("a healing lock never fails");
        };
        assert_eq!(value, 5);
    }
}
