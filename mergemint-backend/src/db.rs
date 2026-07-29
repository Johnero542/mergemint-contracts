// mergemint-backend/src/db.rs
//
// Database connection pool and shared state helpers.
//
// ## Mutex-poison recovery (#473)
//
// Rust's `Mutex::lock()` returns `Err(PoisonError)` if a thread panicked while
// holding the lock.  Calling `.unwrap()` would re-panic every subsequent
// caller, effectively taking the whole service down for what is often a
// transient edge case.
//
// We use `.unwrap_or_else(|e| e.into_inner())` instead: when the lock is
// poisoned we recover the inner value and continue under the assumption that
// the data is still in a consistent-enough state to serve requests.  If the
// data truly is corrupt the next business-logic validation will catch it and
// return an error to the client rather than crashing the process.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Lightweight in-memory store used during development / integration tests.
/// Production deployments replace this with a real database pool.
#[derive(Debug, Default)]
pub struct DbStore {
    pub records: HashMap<String, String>,
}

/// Shared, thread-safe handle to the database store.
pub type SharedDb = Arc<Mutex<DbStore>>;

/// Create a new, empty shared database handle.
pub fn new_shared_db() -> SharedDb {
    Arc::new(Mutex::new(DbStore::default()))
}

/// Acquire the database lock, recovering gracefully from mutex poison.
///
/// If a previous thread panicked while holding this lock, `.into_inner()`
/// extracts the guarded value so the service can keep running instead of
/// propagating the panic to every subsequent request.
pub fn acquire_db(db: &SharedDb) -> std::sync::MutexGuard<'_, DbStore> {
    db.lock().unwrap_or_else(|e| e.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Placeholder test — verifies that the test harness compiles and is wired
    /// correctly.  See issue #487.
    #[test]
    fn it_compiles() {}

    #[test]
    fn test_acquire_db_normal() {
        let db = new_shared_db();
        let mut guard = acquire_db(&db);
        guard.records.insert("key".to_string(), "value".to_string());
        assert_eq!(guard.records.get("key").map(|s| s.as_str()), Some("value"));
    }

    #[test]
    fn test_acquire_db_poison_recovery() {
        let db = new_shared_db();

        // Simulate a panic while holding the lock.
        let db_clone = Arc::clone(&db);
        let _ = std::panic::catch_unwind(move || {
            let _guard = db_clone.lock().unwrap();
            panic!("simulated panic");
        });

        // The lock is now poisoned; acquire_db must not propagate the poison.
        let guard = acquire_db(&db);
        assert!(guard.records.is_empty(), "recovered store should be intact");
    }
}
